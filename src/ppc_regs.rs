/// PPCレジスタをDolphinプロセスメモリから直接読み取る。
/// dolphin-memory crateはエミュレートRAM (MEM1) しか読めないため、
/// PowerPCState構造体のホストアドレスをPVRシグネチャスキャンで特定し、
/// ReadProcessMemoryで各レジスタを読む。
///
/// 前提: x86_64 MSVC ビルドの Dolphin。構造体レイアウトは PowerPC.h に準拠。

use std::mem;

use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::{
    HANDLE, MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_MAPPED, PROCESS_QUERY_INFORMATION,
    PROCESS_VM_READ,
};

// ── PowerPCState field offsets (x86_64 MSVC layout) ──────────────────────
//
// Source: Source/Core/Core/PowerPC/PowerPC.h
//
// +0x000  u32 pc
// +0x004  u32 npc
// +0x008  u8* stored_stack_pointer   (8B)
// +0x010  u8* gather_pipe_ptr        (8B)
// +0x018  u8* gather_pipe_base_ptr   (8B)
// +0x020  u32 gpr[32]                (128B)
// +0x0A0  ConditionRegister cr       (u64 fields[8] = 64B)
// +0x0E0  UReg_MSR msr              (u32)
// +0x0E4  UReg_FPSCR fpscr          (u32)
// +0x0E8  CPUEmuFeatureFlags        (u32)
// +0x0EC  u32 Exceptions
// +0x0F0  int downcount
// +0x0F4  u8 xer_ca
// +0x0F5  u8 xer_so_ov
// +0x0F6  u16 xer_stringctrl
// +0x0F8  u32 reserve_address
// +0x0FC  bool reserve
// +0x0FD  bool pagetable_update_pending
// +0x0FE  bool m_enable_dcache
// +0x0FF  std::tuple<> above_fits_in_first_0x100
// +0x100  alignas(16) PairedSingle ps[32]  (32*16 = 512B)
// +0x300  u32 sr[16]                 (64B)
// +0x340  alignas(8) u32 spr[1024]   (4096B)
// +0x1340 u8* mem_ptr                (8B)

const OFF_PC: usize = 0x000;
const OFF_GPR: usize = 0x020;
const OFF_CR_FIELDS: usize = 0x0A0;
const OFF_MSR: usize = 0x0E0;
const OFF_FPSCR: usize = 0x0E4;
const OFF_XER_CA: usize = 0x0F4;
const OFF_XER_SO_OV: usize = 0x0F5;
const OFF_XER_STRINGCTRL: usize = 0x0F6;
const OFF_PS: usize = 0x100;
const OFF_SPR: usize = 0x340;
const OFF_MEM_PTR: usize = 0x1340;

// SPR indices (Gekko.h)
const SPR_LR: usize = 8;
const SPR_CTR: usize = 9;
const SPR_SRR0: usize = 26;
const SPR_SRR1: usize = 27;
const SPR_PVR: usize = 287;

// PVR values
const PVR_GEKKO: u32 = 0x00083214;
const PVR_BROADWAY: u32 = 0x00087102;

// PVR offset from PowerPCState base
const PVR_OFFSET: usize = OFF_SPR + SPR_PVR * 4; // 0x340 + 0x47C = 0x7BC

// XER reconstruction shifts (Gekko.h)
const XER_CA_SHIFT: u32 = 29;
const XER_OV_SHIFT: u32 = 30;

pub struct PpcRegisterReader {
    handle: HANDLE,
    ppc_state_base: usize,
}

#[allow(dead_code)]
pub struct PpcSnapshot {
    pub pc: u32,
    pub npc: u32,
    pub lr: u32,
    pub ctr: u32,
    pub cr: u32,
    pub xer: u32,
    pub msr: u32,
    pub fpscr: u32,
    pub srr0: u32,
    pub srr1: u32,
    pub gpr: [u32; 32],
    pub fpr: [f64; 32],
}

impl PpcRegisterReader {
    pub fn new() -> Result<Self, String> {
        let pid = find_dolphin_pid()?;
        let handle = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid)
        };
        if handle.is_null() {
            return Err("Dolphinプロセスを開けません".into());
        }

        match find_ppc_state(handle) {
            Ok(base) => Ok(Self {
                handle,
                ppc_state_base: base,
            }),
            Err(e) => {
                unsafe { CloseHandle(handle); }
                Err(e)
            }
        }
    }

    /// PVR がまだ正しいか確認
    pub fn is_valid(&self) -> bool {
        match read_host_u32(self.handle, self.ppc_state_base + PVR_OFFSET) {
            Ok(v) => v == PVR_GEKKO || v == PVR_BROADWAY,
            Err(_) => false,
        }
    }

    /// 全レジスタを一括取得
    pub fn read_all(&self) -> Result<PpcSnapshot, String> {
        let base = self.ppc_state_base;

        // First 0x100 bytes: PC, NPC, GPR[32], CR.fields[8], MSR, FPSCR, XER parts
        let blk0 = read_host_bytes(self.handle, base, 0x100)?;

        // PS array: 32 * 16 = 512 bytes at offset 0x100
        let ps_blk = read_host_bytes(self.handle, base + OFF_PS, 0x200)?;

        // Specific SPRs (read individually since they're spread across 4KB)
        let lr = read_host_u32(self.handle, base + OFF_SPR + SPR_LR * 4)?;
        let ctr = read_host_u32(self.handle, base + OFF_SPR + SPR_CTR * 4)?;
        let srr0 = read_host_u32(self.handle, base + OFF_SPR + SPR_SRR0 * 4)?;
        let srr1 = read_host_u32(self.handle, base + OFF_SPR + SPR_SRR1 * 4)?;

        // --- Parse blk0 ---
        let pc = le_u32(&blk0, OFF_PC);
        let npc = le_u32(&blk0, 0x004);

        let mut gpr = [0u32; 32];
        for i in 0..32 {
            gpr[i] = le_u32(&blk0, OFF_GPR + i * 4);
        }

        // CR: 8 x u64 internal format → standard 32-bit CR
        let mut cr: u32 = 0;
        for i in 0..8 {
            let field = le_u64(&blk0, OFF_CR_FIELDS + i * 8);
            cr |= cr_field_to_ppc(field) << (28 - i as u32 * 4);
        }

        let msr = le_u32(&blk0, OFF_MSR);
        let fpscr = le_u32(&blk0, OFF_FPSCR);

        // XER reconstruction
        let xer_ca = blk0[OFF_XER_CA];
        let xer_so_ov = blk0[OFF_XER_SO_OV];
        let xer_sc = le_u16(&blk0, OFF_XER_STRINGCTRL);
        let xer = (xer_sc as u32)
            | ((xer_ca as u32) << XER_CA_SHIFT)
            | ((xer_so_ov as u32) << XER_OV_SHIFT);

        // --- Parse ps_blk ---
        let mut fpr = [0.0f64; 32];
        for i in 0..32 {
            let bits = le_u64(&ps_blk, i * 16); // ps0 at start of each PairedSingle
            fpr[i] = f64::from_bits(bits);
        }

        Ok(PpcSnapshot {
            pc,
            npc,
            lr,
            ctr,
            cr,
            xer,
            msr,
            fpscr,
            srr0,
            srr1,
            gpr,
            fpr,
        })
    }
}

impl Drop for PpcRegisterReader {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { CloseHandle(self.handle); }
        }
    }
}

/// フォーマット: GPR + 主要SPR のテキスト表示
pub fn format_snapshot(s: &PpcSnapshot) -> String {
    let mut out = String::with_capacity(2048);

    out.push_str(&format!(
        "PC:  0x{:08X}    LR:  0x{:08X}    CTR: 0x{:08X}\n",
        s.pc, s.lr, s.ctr
    ));
    out.push_str(&format!(
        "CR:  0x{:08X}    XER: 0x{:08X}    MSR: 0x{:08X}\n",
        s.cr, s.xer, s.msr
    ));
    out.push_str(&format!(
        "SRR0:0x{:08X}    SRR1:0x{:08X}    FPSCR:0x{:08X}\n",
        s.srr0, s.srr1, s.fpscr
    ));
    out.push_str(&format!("NPC: 0x{:08X}\n\n", s.npc));

    out.push_str("GPR:\n");
    for row in 0..8 {
        for col in 0..4 {
            let i = row * 4 + col;
            out.push_str(&format!("  r{:<2}=0x{:08X}", i, s.gpr[i]));
        }
        out.push('\n');
    }

    out.push_str("\nFPR:\n");
    for row in 0..8 {
        for col in 0..4 {
            let i = row * 4 + col;
            let v = s.fpr[i];
            if v == 0.0 || (v.abs() >= 0.001 && v.abs() < 1e10) {
                out.push_str(&format!("  f{:<2}={:<14.6}", i, v));
            } else {
                out.push_str(&format!("  f{:<2}={:<14.6e}", i, v));
            }
        }
        out.push('\n');
    }

    out
}

// ── CR internal format → PPC 4-bit format ────────────────────────────────
// Source: ConditionRegister.h GetField()
//   SO iff bit 59 set
//   EQ iff lower 32 bits == 0
//   GT iff (s64)val > 0
//   LT iff bit 62 set
fn cr_field_to_ppc(cr_val: u64) -> u32 {
    let mut ppc_cr: u32 = 0;
    // LT (bit 62→bit3) and SO (bit 59→bit0)
    ppc_cr |= ((cr_val >> 59) & 0b1001) as u32;
    // EQ: lower 32 bits == 0
    ppc_cr |= (((cr_val & 0xFFFF_FFFF) == 0) as u32) << 1;
    // GT: (s64)cr_val > 0
    ppc_cr |= (((cr_val as i64) > 0) as u32) << 2;
    ppc_cr
}

// ── Dolphin process discovery ────────────────────────────────────────────

fn find_dolphin_pid() -> Result<DWORD, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("プロセススナップショット作成失敗".into());
        }

        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as DWORD;

        let mut found = None;
        if Process32FirstW(snapshot, &mut entry) != FALSE {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
                if name.eq_ignore_ascii_case("Dolphin.exe") {
                    found = Some(entry.th32ProcessID);
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == FALSE {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        found.ok_or_else(|| "Dolphinプロセスが見つかりません".into())
    }
}

// ── PowerPCState location via PVR signature scan ─────────────────────────

fn find_ppc_state(handle: HANDLE) -> Result<usize, String> {
    let target = PVR_GEKKO.to_le_bytes();
    let target_alt = PVR_BROADWAY.to_le_bytes();
    let mut candidates = Vec::new();

    let mut address: usize = 0;
    unsafe {
        loop {
            let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();
            let ret = VirtualQueryEx(
                handle,
                address as LPVOID,
                &mut mbi,
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );
            if ret == 0 {
                break;
            }

            let base = mbi.BaseAddress as usize;
            let size = mbi.RegionSize;

            // Scan committed private/image memory (skip MEM_MAPPED = game RAM)
            if mbi.State == MEM_COMMIT && mbi.Type != MEM_MAPPED && size >= PVR_OFFSET + 4 {
                scan_for_pvr(handle, base, size, &target, &target_alt, &mut candidates);
            }

            address = base.wrapping_add(size);
            if address <= base {
                break;
            }
        }
    }

    for &base in &candidates {
        if validate_ppc_state(handle, base) {
            return Ok(base);
        }
    }

    Err("PowerPCStateが見つかりません（ゲーム実行中か確認してください）".into())
}

fn scan_for_pvr(
    handle: HANDLE,
    region_base: usize,
    region_size: usize,
    target: &[u8; 4],
    target_alt: &[u8; 4],
    candidates: &mut Vec<usize>,
) {
    const CHUNK: usize = 256 * 1024;
    let mut buf = vec![0u8; CHUNK + 4]; // +4 for cross-boundary matches

    let mut off = 0;
    while off < region_size {
        let read_len = (region_size - off).min(CHUNK + 4);
        let addr = region_base + off;

        let mut bytes_read: usize = 0;
        let ok = unsafe {
            ReadProcessMemory(
                handle,
                addr as LPVOID,
                buf.as_mut_ptr() as LPVOID,
                read_len,
                &mut bytes_read as *mut _ as *mut _,
            )
        };
        if ok == FALSE || bytes_read < 4 {
            off += CHUNK;
            continue;
        }

        // Scan on 4-byte alignment (SPR array is u32-aligned)
        let scan_end = bytes_read.saturating_sub(3);
        let mut i = 0;
        while i < scan_end {
            if buf[i..i + 4] == target[..] || buf[i..i + 4] == target_alt[..] {
                let pvr_host = addr + i;
                if pvr_host >= PVR_OFFSET {
                    candidates.push(pvr_host - PVR_OFFSET);
                }
            }
            i += 4; // spr is u32-aligned
        }

        off += CHUNK;
    }
}

fn validate_ppc_state(handle: HANDLE, base: usize) -> bool {
    // 1. PVR must match
    let pvr = match read_host_u32(handle, base + PVR_OFFSET) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if pvr != PVR_GEKKO && pvr != PVR_BROADWAY {
        return false;
    }

    // 2. PC should be in valid PPC code range or 0
    let pc = match read_host_u32(handle, base + OFF_PC) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if pc != 0 && (pc < 0x8000_0000 || pc >= 0x8180_0000) {
        return false;
    }

    // 3. gpr[1] (SP) should be in valid PPC range or 0
    let sp = match read_host_u32(handle, base + OFF_GPR + 4) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if sp != 0 && (sp < 0x8000_0000 || sp >= 0x8180_0000) {
        return false;
    }

    // 4. LR should be in valid PPC range or 0
    let lr = match read_host_u32(handle, base + OFF_SPR + SPR_LR * 4) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if lr != 0 && (lr < 0x8000_0000 || lr >= 0x8180_0000) {
        return false;
    }

    // 5. mem_ptr should be non-null when emulation is running
    let mem_ptr = match read_host_u64(handle, base + OFF_MEM_PTR) {
        Ok(v) => v,
        Err(_) => return false,
    };
    mem_ptr != 0
}

// ── Host process memory read helpers ─────────────────────────────────────

fn read_host_bytes(handle: HANDLE, addr: usize, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    let mut bytes_read: usize = 0;
    let ok = unsafe {
        ReadProcessMemory(
            handle,
            addr as LPVOID,
            buf.as_mut_ptr() as LPVOID,
            len,
            &mut bytes_read as *mut _ as *mut _,
        )
    };
    if ok == FALSE || bytes_read != len {
        return Err(format!(
            "ReadProcessMemory at 0x{:X}: read {}/{}",
            addr, bytes_read, len
        ));
    }
    Ok(buf)
}

fn read_host_u32(handle: HANDLE, addr: usize) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    let mut bytes_read: usize = 0;
    let ok = unsafe {
        ReadProcessMemory(
            handle,
            addr as LPVOID,
            buf.as_mut_ptr() as LPVOID,
            4,
            &mut bytes_read as *mut _ as *mut _,
        )
    };
    if ok == FALSE || bytes_read != 4 {
        return Err(format!("ReadProcessMemory at 0x{:X}", addr));
    }
    Ok(u32::from_le_bytes(buf))
}

fn read_host_u64(handle: HANDLE, addr: usize) -> Result<u64, String> {
    let mut buf = [0u8; 8];
    let mut bytes_read: usize = 0;
    let ok = unsafe {
        ReadProcessMemory(
            handle,
            addr as LPVOID,
            buf.as_mut_ptr() as LPVOID,
            8,
            &mut bytes_read as *mut _ as *mut _,
        )
    };
    if ok == FALSE || bytes_read != 8 {
        return Err(format!("ReadProcessMemory at 0x{:X}", addr));
    }
    Ok(u64::from_le_bytes(buf))
}

// ── Little-endian helpers for parsing pre-read buffers ────────────────────

fn le_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
}

fn le_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(buf[off..off + 8].try_into().unwrap())
}

fn le_u16(buf: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(buf[off..off + 2].try_into().unwrap())
}
