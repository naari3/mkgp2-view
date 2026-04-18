mod dolphin;
mod ppc_regs;

use dolphin_memory::Dolphin;
use ppc_regs::PpcRegisterReader;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut dolphin: Option<Dolphin> = None;
    let mut reg_reader: Option<PpcRegisterReader> = None;

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(_) => break,
        };

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = request.get("id").cloned();
        let method = request["method"].as_str().unwrap_or("");

        // Notifications have no id and need no response
        if id.is_none() {
            continue;
        }

        let response = match method {
            "initialize" => {
                dolphin = Dolphin::new().ok();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": {
                            "name": "mkgp2-mcp",
                            "version": "0.1.0"
                        }
                    }
                })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools_list()
                    }
                })
            }
            "tools/call" => {
                let params = &request["params"];
                let tool_name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                handle_tool_call(&mut dolphin, &mut reg_reader, tool_name, args, id)
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                })
            }
        };

        writeln!(out, "{}", serde_json::to_string(&response).unwrap()).unwrap();
        out.flush().unwrap();
    }
}

fn tools_list() -> Value {
    json!([
        {
            "name": "read_u32",
            "description": "PPCアドレスからu32値を読み取る。値は16進数と10進数の両方で返す",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "PPCアドレス (例: \"0x806d1098\")" }
                },
                "required": ["address"]
            }
        },
        {
            "name": "read_f32",
            "description": "PPCアドレスからf32値を読み取る",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "PPCアドレス (例: \"0x806d1470\")" }
                },
                "required": ["address"]
            }
        },
        {
            "name": "read_u8",
            "description": "PPCアドレスから1バイト読み取る",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "PPCアドレス" }
                },
                "required": ["address"]
            }
        },
        {
            "name": "read_bytes",
            "description": "PPCアドレスからNバイト読み取り、16進ダンプで返す",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "PPCアドレス" },
                    "length": { "type": "integer", "description": "読み取りバイト数 (最大256)", "default": 16 }
                },
                "required": ["address"]
            }
        },
        {
            "name": "read_string",
            "description": "PPCアドレスからNULL終端文字列を読み取る (最大256バイト)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "PPCアドレス" }
                },
                "required": ["address"]
            }
        },
        {
            "name": "read_pointer_chain",
            "description": "ベースアドレスからオフセット配列に沿ってポインタチェーンを辿り、各ステップの中間アドレスと値を表示する。例: address=0x806d1098, offsets=[0x28, 0x340, 0x34] → g_playerCarObject → KM → ISE → itemId",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "address": { "type": "string", "description": "起点PPCアドレス" },
                    "offsets": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "オフセット配列 (例: [\"0x28\", \"0x340\", \"0x34\"])"
                    },
                    "final_type": { "type": "string", "description": "最終値の型: u32(default), f32, u8", "default": "u32" }
                },
                "required": ["address", "offsets"]
            }
        },
        {
            "name": "get_game_state",
            "description": "現在のゲーム状態を一括取得 (コース、モード、全カート情報等)",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "read_registers",
            "description": "PPCレジスタ（GPR[0-31], FPR[0-31], PC, LR, CTR, CR, XER, MSR等）をDolphinプロセスメモリから直接読み取る。JIT実行中でもスナップショットを取得できる",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "reconnect",
            "description": "Dolphinへの接続をリセットして再接続する",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }
    ])
}

fn ensure_connected(dolphin: &mut Option<Dolphin>) -> bool {
    if let Some(d) = dolphin.as_ref() {
        if d.read_u8(0, None).is_ok() {
            return true;
        }
    }
    *dolphin = Dolphin::new().ok();
    dolphin.is_some()
}

fn handle_tool_call(
    dolphin: &mut Option<Dolphin>,
    reg_reader: &mut Option<PpcRegisterReader>,
    tool: &str,
    args: &Value,
    id: Option<Value>,
) -> Value {
    if tool == "reconnect" {
        *dolphin = Dolphin::new().ok();
        *reg_reader = None; // force re-scan on next register read
        return tool_result(
            id,
            if dolphin.is_some() {
                "再接続成功"
            } else {
                "再接続失敗: Dolphinプロセスが見つかりません"
            },
        );
    }

    if tool == "read_registers" {
        return handle_read_registers(reg_reader, id);
    }

    if !ensure_connected(dolphin) {
        return tool_error(id, "Dolphinに接続されていません（自動再接続も失敗）");
    }

    if tool == "read_pointer_chain" {
        return read_pointer_chain_handler(dolphin.as_ref().unwrap(), args, id);
    }

    if tool == "get_game_state" {
        return get_game_state_response(dolphin.as_ref().unwrap(), id);
    }

    let d = dolphin.as_ref().unwrap();

    let addr_str = args["address"].as_str().unwrap_or("0");
    let addr = parse_address(addr_str);

    match tool {
        "read_u32" => match read_u32(d, addr) {
            Ok(val) => tool_result(id, &format!("0x{:08X} ({})", val, val)),
            Err(e) => tool_error(id, &e),
        },
        "read_f32" => match read_f32(d, addr) {
            Ok(val) => tool_result(id, &format!("{} (bits: 0x{:08X})", val, val.to_bits())),
            Err(e) => tool_error(id, &e),
        },
        "read_u8" => match read_u8(d, addr) {
            Ok(val) => tool_result(id, &format!("0x{:02X} ({})", val, val)),
            Err(e) => tool_error(id, &e),
        },
        "read_bytes" => {
            let len = args["length"].as_u64().unwrap_or(16).min(256) as usize;
            match read_bytes(d, addr, len) {
                Ok(bytes) => {
                    let hex_lines = format_hex_dump(addr, &bytes);
                    tool_result(id, &hex_lines)
                }
                Err(e) => tool_error(id, &e),
            }
        }
        "read_string" => match read_cstring(d, addr) {
            Ok(s) => tool_result(id, &format!("\"{}\"", s)),
            Err(e) => tool_error(id, &e),
        },
        _ => tool_error(id, &format!("Unknown tool: {}", tool)),
    }
}

fn get_game_state_response(d: &Dolphin, id: Option<Value>) -> Value {
    let state = dolphin::try_read_state(d);
    if let Some(ref err) = state.error {
        return tool_error(id, err);
    }

    let mut ai_list = Vec::new();
    for ai in &state.ai_karts {
        let total = state.total_laps;
        let lap_display = if total > 0 && ai.remaining_laps <= total + 1 {
            format!("{}/{}", total + 1 - ai.remaining_laps, total)
        } else {
            format!("remaining={}", ai.remaining_laps)
        };
        ai_list.push(json!({
            "slot": ai.slot,
            "position": ai.race_position + 1,
            "lap": lap_display,
            "target_speed": format!("{:.1}-{:.1}", ai.target_speed_min / 10000.0, ai.target_speed_max / 10000.0),
            "pos": { "x": ai.pos[0], "y": ai.pos[1], "z": ai.pos[2] }
        }));
    }

    let p = &state.player;
    let player_json = json!({
        "char": p.char_name(),
        "char_id": p.char_id,
        "speed": p.speed_magnitude(),
        "speed_cap": p.speed_cap,
        "travel_progress": p.travel_progress,
        "speed_table_index": p.speed_table_index,
        "speed_scale": p.speed_scale,
        "coin_speed_bonus": p.coin_speed_bonus,
        "coin_count": p.coin_count,
        "mue": p.mue,
        "pos": { "x": p.pos[0], "y": p.pos[1], "z": p.pos[2] },
        "velocity": { "x": p.velocity[0], "y": p.velocity[1], "z": p.velocity[2] },
        "is_airborne": p.is_airborne,
    });

    let result_text = serde_json::to_string_pretty(&json!({
        "connected": state.connected,
        "game_mode": state.game_mode_name(),
        "course": state.course_name(),
        "cc": state.cc_label(),
        "total_laps": state.total_laps,
        "race_started": state.race_started,
        "countdown_phase": state.countdown_phase,
        "player": player_json,
        "ai_karts": ai_list,
    }))
    .unwrap();

    tool_result(id, &result_text)
}

fn read_pointer_chain_handler(d: &Dolphin, args: &Value, id: Option<Value>) -> Value {
    let base_str = args["address"].as_str().unwrap_or("0");
    let base = parse_address(base_str);
    let offsets: Vec<u32> = args["offsets"]
        .as_array()
        .map(|arr| arr.iter().map(|v| parse_address(v.as_str().unwrap_or("0"))).collect())
        .unwrap_or_default();
    let final_type = args["final_type"].as_str().unwrap_or("u32");

    if offsets.is_empty() {
        return tool_error(id, "offsets配列が空です");
    }

    let mut result = String::new();
    let mut current = base;
    result.push_str(&format!("base: 0x{:08X}\n", current));

    // Follow pointer chain: all but last offset dereference as u32 pointer
    for (i, &offset) in offsets.iter().enumerate() {
        let is_last = i == offsets.len() - 1;
        let addr = current.wrapping_add(offset);

        if is_last {
            // Final read with specified type
            match final_type {
                "f32" => match read_f32(d, addr) {
                    Ok(val) => result.push_str(&format!("  +0x{:X} → [0x{:08X}] = {} (0x{:08X})\n", offset, addr, val, val.to_bits())),
                    Err(e) => return tool_error(id, &e),
                },
                "u8" => match read_u8(d, addr) {
                    Ok(val) => result.push_str(&format!("  +0x{:X} → [0x{:08X}] = 0x{:02X} ({})\n", offset, addr, val, val)),
                    Err(e) => return tool_error(id, &e),
                },
                _ => match read_u32(d, addr) {
                    Ok(val) => result.push_str(&format!("  +0x{:X} → [0x{:08X}] = 0x{:08X} ({})\n", offset, addr, val, val)),
                    Err(e) => return tool_error(id, &e),
                },
            }
        } else {
            // Intermediate: dereference as pointer
            match read_u32(d, addr) {
                Ok(ptr) => {
                    result.push_str(&format!("  +0x{:X} → [0x{:08X}] = 0x{:08X}\n", offset, addr, ptr));
                    if ptr < 0x80000000 || ptr >= 0x81800000 {
                        return tool_error(id, &format!("{}ステップ目で無効なポインタ: 0x{:08X}", i + 1, ptr));
                    }
                    current = ptr;
                }
                Err(e) => return tool_error(id, &e),
            }
        }
    }

    tool_result(id, &result)
}

fn handle_read_registers(reg_reader: &mut Option<PpcRegisterReader>, id: Option<Value>) -> Value {
    // Lazy init / reconnect if stale
    if reg_reader.as_ref().map_or(true, |r| !r.is_valid()) {
        *reg_reader = PpcRegisterReader::new().ok();
    }
    let reader = match reg_reader.as_ref() {
        Some(r) => r,
        None => {
            return tool_error(
                id,
                "レジスタリーダー初期化失敗（ゲーム実行中か確認してください）",
            )
        }
    };
    match reader.read_all() {
        Ok(snap) => tool_result(id, &ppc_regs::format_snapshot(&snap)),
        Err(e) => {
            *reg_reader = None; // stale – force re-scan next time
            tool_error(id, &e)
        }
    }
}

// --- Helpers ---

fn parse_address(s: &str) -> u32 {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

fn read_u32(d: &Dolphin, addr: u32) -> Result<u32, String> {
    d.read_u32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| format!("読み取りエラー at 0x{:08X}: {}", addr, e))
}

fn read_f32(d: &Dolphin, addr: u32) -> Result<f32, String> {
    d.read_f32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| format!("読み取りエラー at 0x{:08X}: {}", addr, e))
}

fn read_u8(d: &Dolphin, addr: u32) -> Result<u8, String> {
    d.read_u8((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| format!("読み取りエラー at 0x{:08X}: {}", addr, e))
}

fn read_bytes(d: &Dolphin, addr: u32, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    for i in 0..len {
        buf[i] = d
            .read_u8(((addr + i as u32) & 0x01FFFFFF) as usize, None)
            .map_err(|e| format!("読み取りエラー at 0x{:08X}: {}", addr + i as u32, e))?;
    }
    Ok(buf)
}

fn read_cstring(d: &Dolphin, addr: u32) -> Result<String, String> {
    let mut bytes = Vec::new();
    for i in 0..256u32 {
        let b = d
            .read_u8(((addr + i) & 0x01FFFFFF) as usize, None)
            .map_err(|e| format!("読み取りエラー at 0x{:08X}: {}", addr + i, e))?;
        if b == 0 {
            break;
        }
        bytes.push(b);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn format_hex_dump(base_addr: u32, bytes: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let addr = base_addr + (i * 16) as u32;
        result.push_str(&format!("{:08X}: ", addr));
        for (j, b) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{:02X} ", b));
        }
        // Pad if short
        let pad = 16 - chunk.len();
        for j in 0..pad {
            if chunk.len() + j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }
        result.push_str(" |");
        for b in chunk {
            if b.is_ascii_graphic() || *b == b' ' {
                result.push(*b as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }
    result
}

fn tool_result(id: Option<Value>, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": text }]
        }
    })
}

fn tool_error(id: Option<Value>, msg: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": msg }],
            "isError": true
        }
    })
}
