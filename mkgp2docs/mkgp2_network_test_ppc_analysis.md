# MKGP2 NETWORK TEST - PPC Reverse Engineering Analysis

## Overview

Mario Kart Arcade GP 2 (Game ID: GNLJ82) の Service Menu → TEST → NETWORK TEST の解析。
Triforce アーケードシステムの Media Board (DIMM Board) を経由してネットワークテストを実行する。

- Execute1: DMA address 0x84000040, interrupt 0x04
- Execute2: DMA address 0x88000000, interrupt 0x10
- NETWORK TEST は主に Execute2 を使用

## テスト状態構造体 (0x800D18FC, 260 bytes)

```c
struct TestState {          // at 0x800D18FC
    u32 command;            // [0x00] コマンドワード
    u32 context;            // [0x04] コンテキストポインタ
    u32 error_flag;         // [0x08] エラーフラグ (0=OK, 1=error)
    u32 state;              // [0x0C] テスト状態
                            //   1=init, 3=running, 4=passed,
                            //   5=failed, 6=next_phase, 0=idle/reset
    u8  test_type;          // [0x10] テストタイプバイト
    u8  result;             // [0x11] 結果バイト
    u8  max_iterations;     // [0x12] 最大反復回数 (default 8)
    u8  _pad;
    u32 progress;           // [0x14] 進捗カウンタ
    u32 status;             // [0x18] ステータスワード
    u32 callback1;          // [0x1C] コールバック1
    u32 callback2;          // [0x20] コールバック2
    // ... total 260 bytes
};
```

## State Handler ジャンプテーブル (0x80077C84)

| Index | Address    | Description |
|-------|-----------|-------------|
| 0     | NULL      | なし (スキップ) |
| 1     | 0x8006822C | Init: setup関数を (0, 2, 0) で呼ぶ |
| 2     | 0x80068170 | Execute1 TestHardware 送信 |
| 3     | 0x800680C0 | Execute2 TestHardware 送信 |

## 実際のPPCコールスタック (Execute2 DMA発火時)

```
frame[9] sp=800d0328 lr=8005490c  ← メインループ / OS
frame[8] sp=800d0308 lr=8006adb0  ← TestPollLoop
frame[7] sp=800d02f8 lr=80064e6c  ← TestManager_Run (frame=0x10)
frame[6] sp=800d0228 lr=800651dc  ← TestManager_Dispatch (frame=0xD0) ★r28設定
frame[5] sp=800d0188 lr=80068110  ← State3Handler (frame=0xA0, r31のみ保存)
frame[4] sp=800d0150 lr=800592c8  ← SendCommand (frame=0x38)
frame[3] sp=800d0140 lr=800587c4  ← DMA helper (frame=0x10)
frame[2] sp=800d0138 lr=800588e0  ← DMA helper (frame=0x08)
frame[1] sp=800d0130 lr=80056048  ← DMA write (frame=0x08)
frame[0] sp=800d0108 lr=80055fa0  ← Execute2 DMA発火点 (frame=0x28)
```

**重要**: State3Handler (frame[5]) は r31 **のみ** 保存する。
r28 は保存されないため、TestManager_Dispatch で設定された r28 の値は
Execute2 DMA 発火時点まで **ライブレジスタとして保持される**。
→ `ppc_state.gpr[28]` で直接読める。

## テスト全体フロー (0x8006A800+)

```c
// === Phase 1: データ取得 (0x8006A800) ===
memcpy(dst, src, 8);
result = some_utility(global_ptr, saved_val);
if (result == 0) {
    clear_buf(&stack_buf);
    goto after_setup;
}
// データがある場合: build_something して必要に応じて update
// test_struct->status = 1 をセットする場合あり

// === Phase 2: セットアップ (0x8006A894) ===
test_struct = (TestState*)0x800D18FC;
r30 = &test_struct->callback1;  // 0x800D1918
test_type = test_struct->test_type;  // byte at +0x10
r = setup_func(test_type, r30);  // 0x800666F4
if (r != 0) goto FAIL;  // → 0x8006AC8C

// === Phase 3: イテレーション情報取得 (0x8006A8B4) ===
max_iter = get_test_info();  // 0x80066698
test_struct->max_iter = max_iter;
if (test_struct->status != 0) {
    // 前回データのクリーンアップ
    if (r30->field_0 != 0 && r30->field_4 != 0) {
        r30->field_0 = 0;
        update_utility(global_ptr, r30->field_4);
        r30->field_4 = 0;
    }
}

// === Phase 4: テスト開始 (0x8006A900) ===
test_struct->state = 3;  // STATE_RUNNING
*(r13 - 0x76EC) = 0;    // iteration counter
*(r13 - 0x76F0) = 0;    // secondary counter
*(r13 - 0x76E8) = 0;    // result byte

// === Phase 5: 初回テスト (0x8006A92C) ===
r = InitTest(0, 1);  // 0x80068FE4
*(r13 - 0x76E8) = r & 0xFF;  // result byte
if (r == 0) {
    // テスト即座完了 (InitTest が 0 → 何も投入されなかった)
    goto END;
}

// === Phase 6: ポーリングループ (0x8006A948) ===
while (true) {
    r = PollTest(result_byte, &local);  // 0x80068F54
    if (r == 0) break;  // テスト完了!
    counter1++;
    if (counter1 >= 60) {
        counter2++;
        if (counter2 >= 10) {
            test_struct->state = 0;  // タイムアウト
        }
        counter1 = 0;
    }
}

// === Phase 7: 最終判定 (0x8006AAA8) ===
final = check_result(test_type);  // 0x80067D9C
test_struct->result = final & 0xFF;
if (final == 0) test_struct->state = 4;  // PASSED
else            test_struct->state = 5;  // FAILED
```

## TestManager_Dispatch (0x80064E98) ★最重要

TestManager_Dispatch はテストインデックスから状態ハンドラを呼び出す中核関数。
r28 に設定される ctx_entry ポインタが **ディスパッチの鍵** となる。

### メモリレイアウト

```
base        = 0x800C8C20   (lis 0x800D; addi -0x73E0)
log_ring    = base + 0x0180 = 0x800C8DA0  (リングバッファ)
test_table  = base + 0x4180 = 0x800CCDA0  (テストエントリテーブル, stride 0x60)
g_test_ctx  = base + 0x5980 = 0x800CE5A0  (テストコンテキスト)
```

### g_test_ctx 構造体 (0x800CE5A0)

```c
struct g_test_ctx {
    u32 count;             // [0x00] エントリ数
    struct ctx_entry {     // [0x04]~ 各0x10バイト
        u8  type;          // [0x0] タイプバイト (マッチキー)
        u8  status;        // [0x1] ★ステータスバイト (ディスパッチ制御)
        u8  pad[2];
        u32 field_04;      // [0x4]
        u32 field_08;      // [0x8] bit0: InitTestのDMA投入フラグ
        u32 field_0C;      // [0xC] 補助データ (r23 に読まれる)
    } entries[];
    // ...
    u32 state_field;       // [0xB4] ステートフィールド
};
```

### 逆アセンブリ & 擬似コード

```asm
; === Prologue ===
80064e98: stwu    r1, -0xD0(r1)       ; フレーム 0xD0
80064e9c: mflr    r0
80064ea8: stw     r0, 0xD4(r1)        ; LR保存
80064eb0: stmw    r23, 0xAC(r1)       ; r23-r31保存 (9レジスタ)

; === ベースアドレス計算 ===
80064eb4: addi    r30, r4, -0x73E0    ; r30 = 0x800C8C20 (base)
80064eb8: addi    r24, r30, 0x5980    ; r24 = 0x800CE5A0 (g_test_ctx)
80064ebc: clrlwi  r31, r3, 24         ; r31 = arg & 0xFF (test_index)

; === テストインデックス検証 ===
80064ec0: cmplwi  r31, 0xFF           ; 0xFF = 無効
80064ee8: bne     check_range
80064eec: li      r3, 3 → return 3   ; 無効値

; === テストエントリテーブルからマッチキー取得 ===
80064ef4: cmplwi  r31, 0x3F           ; index > 63 ならスキップ
80064efc: mulli   r0, r31, 0x60       ; stride = 0x60
80064f00: addi    r3, r30, 0x4180     ; r3 = 0x800CCDA0 (テーブルベース)
80064f04: add     r3, r3, r0          ; r3 = &test_entry[index]
80064f08: lbz     r29, 0x40(r3)       ; ★ r29 = test_entry[0x40] (マッチキー)

; === g_test_ctx エントリ検索 (stride 0x10) ===
80064f14: lwz     r0, 0x5980(r30)     ; count = *(0x800CE5A0)
80064f20: mtctr   r0                  ; ループカウンタ = count
80064f28: addi    r28, r28, 4         ; r28 = 0x800CE5A4 (最初のエントリ)
80064f30: lbz     r0, 0(r28)          ; entry->type
80064f34: cmplw   r0, r3              ; type == match_key ?
80064f38: bne     next                ; 不一致 → 次へ
80064f3c: b       found               ; ★ 一致 → r28 = このエントリ
80064f40: addi    r28, r28, 0x10      ; 次のエントリ
80064f44: bdnz    loop                ; ループ
80064f48: li      r28, 0              ; 見つからない → r28 = NULL

; === 後続処理 ===
80064f50: lwz     r23, 0xC(r28)       ; r23 = entry->field_0C (補助データ)
```

```c
int TestManager_Dispatch(u32 arg) {
    u8 test_index = arg & 0xFF;

    // ログリングバッファに記録
    log_ring[counter++ % 1024] = (test_index << 16) | 0x20000000 | state_low16;

    if (test_index == 0xFF) return 3;
    if (test_index > 63) { match_key = 0; goto search; }

    // テストエントリテーブルからマッチキー取得
    u8 match_key = test_table[test_index * 0x60 + 0x40];

search:
    // g_test_ctx (0x800CE5A0) のエントリを検索
    u32 count = g_test_ctx->count;
    ctx_entry* r28 = NULL;
    for (u32 i = 0; i < count; i++) {
        if (g_test_ctx->entries[i].type == match_key) {
            r28 = &g_test_ctx->entries[i];  // ★ r28 設定!
            break;
        }
    }

    // r28[1] = status byte がディスパッチを制御
    // status <= 3 → ステートハンドラを呼ぶ (ループ継続)
    // status > 3 → ハンドラスキップ (テスト完了)
    if (r28 && r28->status <= 3) {
        // ジャンプテーブル経由でステートハンドラを呼ぶ
        state_handlers[r28->status]();
    }

    return result;
}
```

## State3Handler (0x800680C0) - Execute2 TestHardware 送信

```asm
; === Prologue ===
800680c0: stwu    r1, -0xA0(r1)       ; フレーム 0xA0
800680c4: mflr    r0
800680cc: stw     r0, 0xA4(r1)        ; LR保存
800680d4: stw     r31, 0x9C(r1)       ; ★ r31のみ保存! r28は保存しない!

; === DMAバッファ準備 ===
800680d8: rlwinm  r31, r0, 0, 0, 26   ; r31 = 32バイトアラインドバッファ
800680e0: bl      memzero              ; memzero(aligned_buf, 0x20)
800680f0: bl      memset               ; memset(cmd_buf, 0, 0x30)

; === Execute2 DMAコマンド送信 ===
80068100: lis     r6, 0x8800           ; r6 = 0x88000000 (Execute2 DMA addr)
80068104: li      r7, 0                ; callback = NULL
80068108: li      r8, 2                ; mode = 2 (read from device)
8006810c: bl      SendCommand          ; SendCommand(cmd, buf, 0x20, 0x88000000, NULL, 2)

; === ポーリングループ ===
80068110: addi    r3, r1, 0x28         ; cmd_buf
80068114: bl      PollCompletion
80068118: cmpwi   r3, 0
8006811c: bne     0x80068110           ; 完了まで待つ

; === レスポンスコピー & リターン ===
8006812c: bl      memcpy               ; memcpy(local, aligned_buf, 0x20)
80068140: blr
```

```c
void State3Handler(/* r28 はライブ = TestManager_Dispatch の ctx_entry */) {
    u8 aligned_buf[0x20] __attribute__((aligned(32)));
    u8 cmd_buf[0x30];

    memzero(aligned_buf, 0x20);
    memset(cmd_buf, 0, 0x30);

    // Execute2 DMA: TestHardware コマンドを送信
    SendCommand(cmd_buf, aligned_buf, 0x20, 0x88000000, NULL, 2);

    // 完了まで busy-wait
    while (PollCompletion(cmd_buf) != 0) {}

    // レスポンスをローカルにコピー
    memcpy(local_result, aligned_buf, 0x20);
}
```

**r28 は保存されない** → Execute2 DMA 発火時の `ppc_state.gpr[28]` で取得可能。

## PollCompletion (0x80059664)

```c
int PollCompletion(MediaBoardCmd* cmd) {
    disable_interrupts();
    int state = cmd->field_0xC;
    int result = (state == 3) ? 1 : state;  // 3=processing→1, 0=done
    enable_interrupts();
    return result;
}
```

## DMA 処理フロー

```
SendCommand (0x80059218)
  ├ struct[8] = mode (1=Execute2, 4=Execute1)
  ├ struct[0xC] = 2 (submitted)
  └ Submit to queue at 0x800B3618 (0x80059D94)

DMA State Machine (0x800585B0)
  ├ Dequeue command
  ├ struct[0xC] = 3 (processing)
  ├ Set up DI DMA registers
  ├ Process through states 1-8
  └ On completion (0x80058864):
      struct[0xC] = 0 (done)
      call callback if present
```

## InitTest (0x80068FE4) ★完全逆アセンブリ

```asm
80068fe4: stwu    r1, -0x20(r1)       ; フレーム 0x20
80068fe8: mflr    r0
80068fec: stw     r0, 0x24(r1)
80068ff0: stw     r31, 0x1C(r1)       ; save r31
80068ff4: stw     r30, 0x18(r1)       ; save r30
80068ff8: mr      r30, r4             ; r30 = arg2 (= 1)
80068ffc: addi    r4, r1, 0x08        ; r4 = &local
80069000: bl      GetEntry             ; GetEntry(0, &local) → 0x80066610
80069004: cmpwi   r3, 0
80069008: bne     got_entry
8006900c: li      r3, 0 → return 0   ; エントリなし → 即座パス

got_entry:
80069014: bl      Lock                 ; Lock() → 0x80065F10
80069018: clrlwi. r0, r3, 24          ; r0 = lock & 0xFF
8006901c: mr      r31, r3             ; r31 = lock handle
80069020: bne     locked
80069024: b       epilogue            ; lock失敗 → return 0

locked:
80069028: lbz     r4, 0x08(r1)        ; entry index from local
8006902c: bl      GetBuffer            ; GetBuffer(lock, entry_idx) → 0x80065B28
80069034: bl      GetCommandBuffer     ; → 0x80065C14 (r3 = cmd ptr)
80069038: addi    r5, r3, 4           ; r5 = cmd + 4
80069040: stwcx.  r30, 0, r5          ; atomic: *(cmd+4) = 1
80069044: li      r0, 0x100
80069048: stb     r30, 0(r3)          ; cmd[0] = 1 (test type)
8006904c: stb     r4, 1(r3)           ; cmd[1] = 0
80069050: sth     r0, 2(r3)           ; cmd[2..3] = 0x0100 (command ID)
80069058: bl      SubmitCommand        ; → 0x80066114
8006905c: mr      r3, r31             ; return lock handle (non-zero)
```

```c
int InitTest(int test_index, int flags) {
    u8 entry_idx;
    if (GetEntry(test_index, &entry_idx) == 0)
        return 0;  // エントリなし → 即座パス

    int lock = Lock();
    if ((lock & 0xFF) == 0)
        return lock;  // lock失敗

    GetBuffer(lock, entry_idx);
    u8* cmd = GetCommandBuffer(lock);  // lwarx 含む

    *(u32*)(cmd + 4) = flags;   // stwcx. (atomic)
    cmd[0] = (u8)flags;        // 1
    cmd[1] = 0;
    *(u16*)(cmd + 2) = 0x0100; // コマンドID

    SubmitCommand(lock);
    return lock;  // 非ゼロ → ポーリング必要
}
```

## PollTest (0x80068F54) ★完全逆アセンブリ

```asm
; フレームなし (callerのparam save area使用)
80068f54: mflr    r0
80068f58: stw     r0, 0x14(r1)
80068f5c: stw     r31, 0x0C(r1)       ; save r31
80068f60: mr      r31, r4             ; r31 = local_ptr (arg2)
80068f64: stw     r30, 0x08(r1)       ; save r30
80068f68: mr      r30, r3             ; r30 = result_ptr (arg1)
80068f6c: bl      PollDMAComplete      ; → 0x80065A60
80068f70: cmpwi   r3, 0
80068f74: bne     dma_done
80068f78: li      r3, -5 → return     ; ★ DMA未完了 → CHECKING 0% ループ

dma_done:
80068f80: mr      r3, r30
80068f84: bl      GetResponse          ; → 0x80065CF8
80068f88: lhz     r0, 2(r3)           ; response[2..3] = status halfword
80068f8c: rlwinm  r4, r0, 24, 24, 31  ; high byte → dead code (上書きされる)
80068f90: rlwinm  r4, r0, 8, 16, 23   ; r4 = (status_hw & 0xFF) << 8
80068f94: addis   r0, r4, 0           ; r0 = r4
80068f98: cmplwi  r0, 0x8000          ; response[3] == 0x80 ?
80068f9c: bne     success

; エラーパス
80068fa0: mr      r3, r30
80068fa4: bl      FreeResponse
80068fa8: li      r3, -3 → return     ; エラー

success:
80068fb0: cmplwi  r31, 0              ; local_ptr != NULL?
80068fb8: lwz     r3, 4(r3)           ; result data word
80068fbc: stwcx.  r3, 0, r31          ; *local_ptr = result (atomic)
80068fc4: bl      FreeResponse
80068fc8: li      r3, 0 → return      ; ★ テスト完了
```

```c
int PollTest(u32* result_ptr, u32* local_ptr) {
    if (PollDMAComplete() == 0)
        return -5;  // ★ DMA未完了 → "CHECKING 0%" ループの原因

    u8* resp = GetResponse(result_ptr);
    u16 status_hw = *(u16*)(resp + 2);
    u8 status_byte = status_hw & 0xFF;  // response[3]

    if (status_byte == 0x80) {
        FreeResponse(result_ptr);
        return -3;  // エラー
    }

    if (local_ptr != NULL) {
        *local_ptr = *(u32*)(resp + 4);
    }
    FreeResponse(result_ptr);
    return 0;  // ★ テスト完了
}
```

## check_result (0x80067D9C) ★完全逆アセンブリ

```asm
80067d9c: clrlwi  r0, r3, 24          ; r0 = test_type & 0xFF
80067da0: cmpwi   r0, 0x29            ; type == 0x29 ?
80067da4: beq     lookup              ; → 0x80067DB8
80067da8: bge     pass                ; type > 41 → PASS
80067dac: cmpwi   r0, 0x21            ; (dead code)
80067db0: li      r3, 0 → blr         ; ★ type != 0x29 → 常に PASS

lookup:  ; type == 0x29 のみ
80067db8: lis     r3, 0x800D
80067dbc: lwz     r4, -0x1A60(r3)     ; count = *(0x800CE5A0)
80067dc8: mtctr   r4
80067dd0: ble     not_found            ; count == 0 → not_found

; ループ: 0xA9 エントリを検索
80067dd4: lbz     r0, 0(r3)           ; entry->type
80067dd8: cmplwi  r0, 0xA9
80067ddc: bne     next
80067de0: b       found

next:
80067de4: addi    r3, r3, 0x10        ; 次のエントリ (stride 0x10)
80067de8: bdnz    loop

not_found:
80067df0: cmplwi  r3, 0               ; エントリ見つかった?
80067df4: bne     return_pass          ; 見つかった → PASS
80067df8: cmplwi  r4, 2               ; count >= 2 ?
80067dfc: bge     return_pass          ; → PASS

; 新規 0xA9 エントリ追加 & FAIL
80067e00: lis     r3, 0x800D
80067e08: addi    r6, r3, -0x1A60     ; r6 = 0x800CE5A0
80067e0c: li      r3, 0xA9            ; return 0xA9 (FAIL)
80067e10: lwz     r5, 0(r6)           ; count
80067e14: addi    r4, r5, 1
80067e18: rlwinm  r0, r5, 4, 0, 27   ; offset = count * 16
80067e1c: stw     r4, 0(r6)           ; count++
80067e20: stbx    r7, r8, r0          ; entries[count].type = 0xA9
80067e24: blr                          ; return 0xA9 (FAIL)

return_pass:
80067e28: li      r3, 0 → blr         ; return 0 (PASS)
```

```c
int check_result(u8 test_type) {
    if (test_type != 0x29) return 0;  // ★ type != 0x29 → 常に PASS!

    // g_test_ctx (0x800CE5A0) で 0xA9 エントリを検索
    u32 count = *(u32*)0x800CE5A0;
    for (u32 i = 0; i < count; i++) {
        if (entries_base[i * 16] == 0xA9) return 0;  // PASS
    }
    if (count >= 2) return 0;  // PASS

    // 0xA9 なし & count < 2: エントリ追加して FAIL
    // (次回以降は 0xA9 が存在するので PASS)
    *(u32*)0x800CE5A0 = count + 1;
    entries_base[count * 16] = 0xA9;
    return 0xA9;  // FAIL
}
```

## fn_IsComplete (0x80065A60)

```c
int fn_IsComplete(u8 test_byte) {
    entry_t* table = (entry_t*)0x800CCDA0;  // テストエントリテーブル
    for (int i = 0; i < 64; i++) {
        entry_t* e = &table[i];  // stride = 0x60
        if (!(e->field_0x22 & 0x80) && e->field_0x20 == test_byte) {
            u16 f02 = e->field_02;
            if (f02 == 0) return 0;  // NOT complete
            return 1;                 // COMPLETE
        }
    }
    return 0;  // 見つからない → NOT complete
}
```

## fn_80066610 (InitTest の GetEntry)

```c
int fn_80066610(u8 test_type_param, u8* result_buf) {
    u32 flag = test_type_param | 1;  // = 1 (for param=0)
    u32 count = g_test_ctx->count;
    ctx_entry* entries = g_test_ctx->entries;  // at 0x800CE5A4

    for (u32 i = 0; i < count; i++) {
        if (entries[i].field_08 & flag) {  // bit 0 set?
            if (result_buf) *result_buf = entries[i].field_04;
            return 1;  // → InitTest will submit DMA
        }
    }
    return 0;  // → InitTest returns 0 → immediate PASS
}
```

## setup_func (0x800666F4) ★部分逆アセンブリ

大規模関数 (フレームサイズ 0x660)。テストタイプ別にハンドラをディスパッチする。
ダンプ範囲 0x80066600-0x80066800 のみ (関数全体は 0x800678BC+ まで続く)。

```asm
; === Prologue ===
800666f4: stwu    r1, -0x660(r1)       ; 巨大フレーム (0x660 bytes!)
800666f8: mflr    r0
80066708: stmw    r26, 0x648(r1)       ; r26-r31保存

; === ベースアドレス計算 ===
800666fc: lis     r5, 0x800D
80066700: lis     r6, 0x8007
8006670c: mr      r26, r4              ; r26 = data_ptr (arg2)
80066710: addi    r30, r6, -0x1598     ; r30 = 0x8006EA68 (関数テーブル)
80066714: clrlwi  r4, r3, 24           ; r4 = test_type_byte & 0xFF
80066718: lwzu    r0, -0x1A60(r5)      ; r0 = *(0x800CE5A0); r5 = 0x800CE5A0 (g_test_ctx)
8006671c: addi    r31, r5, 4           ; r31 = 0x800CE5A4 (entries)

; === g_test_ctx エントリ検索 (test_type_byte でマッチ) ===
80066720: mtctr   r0                   ; CTR = entry_count
80066724: cmplwi  r0, 0
80066728: ble     not_found
; loop:
8006672c: lbz     r0, 0(r31)           ; entry->type
80066730: cmplw   r0, r4               ; == test_type_byte ?
80066734: bne     next
80066738: b       found
8006673c: addi    r31, r31, 0x10       ; stride 0x10
80066740: bdnz    loop

; not_found:
80066744: li      r31, 0               ; r31 = NULL
80066748: cmplwi  r31, 0
8006674c: bne     dispatch
80066750: li      r3, 1                ; return 1 (FAIL: エントリなし)
80066754: b       epilogue             ; → 0x800678BC

; === テストタイプ別ディスパッチ ===
; dispatch:
80066758: clrlwi  r0, r3, 24           ; r0 = test_type_byte & 0xFF
8006675c: cmpwi   r0, 0x29
80066760: beq     0x80066E08           ; ★ type 0x29 → ネットワークテスト処理
80066764: bge     check_a9
80066768: cmpwi   r0, 0x21
8006676c: beq     0x80066780           ; type 0x21 → ハードウェアテスト処理
80066770: b       default              ; その他 → 0x800678B8
80066774: cmpwi   r0, 0xA9
80066778: beq     0x800673B4           ; type 0xA9 → 別テスト処理
8006677c: b       default
```

```c
int setup_func(u8 test_type_byte, void* data_ptr) {
    // g_test_ctx から test_type_byte に一致するエントリを検索
    u32 count = g_test_ctx->count;
    ctx_entry* found = NULL;
    for (u32 i = 0; i < count; i++) {
        if (g_test_ctx->entries[i].type == test_type_byte) {
            found = &g_test_ctx->entries[i];
            break;
        }
    }
    if (found == NULL) return 1;  // FAIL: エントリなし

    // テストタイプ別ディスパッチ
    switch (test_type_byte) {
    case 0x21:  // ハードウェアテスト
        // Execute1 (0x84000040) でDMA、結果をdata_ptrにコピー
        // (一部のみダンプ済み)
        break;
    case 0x29:  // ★ネットワークテスト → 0x80066E08 (ダンプ範囲外)
        break;
    case 0xA9:  // 別テスト → 0x800673B4 (ダンプ範囲外)
        break;
    default:
        // → 0x800678B8 (エピローグ付近、おそらく return 0)
        break;
    }
}
```

**type 0x21 ハンドラ (0x80066780)**: 32バイトのアラインドバッファを確保し、
Execute1 DMA (アドレス 0x84000040) を発行、PollCompletion で完了待ち、
結果をスタックローカルにコピーする。(関数の続きはダンプ範囲外)

## ヘルパー関数アドレス一覧 (新規発見)

| Address | Name | Description |
|---------|------|-------------|
| 0x80065A64 | fn_IsComplete | DMA完了チェック (0 = 未完了, 非0 = 完了) |
| 0x80065B2C | fn_GetEntryPtr | エントリインデックスからポインタ取得 |
| 0x80065CFC | fn_GetTestEntry | test_byteからテストエントリ取得 |
| 0x80065DB8 | fn_Cleanup | テスト完了後クリーンアップ |
| 0x80065F14 | fn_AllocEntry | DMAコマンドスロット割り当て |
| 0x80066118 | fn_SubmitCommand | DMAコマンド投入 |
| 0x80066614 | fn_LookupEntry | InitTestのエントリ検索 |
| 0x80059218 | SendCommand | DMAコマンド構築 |
| 0x80059668 | PollCompletion | DMA完了ポーリング |
| 0x8004FE50 | DCInvalidateRange | キャッシュ無効化 (DMA Read前) |
| 0x8004FE7C | DCFlushRange | キャッシュフラッシュ (DMA Write前) |
| 0x8006EA68 | - | 関数/ディスパッチテーブル (r30 in setup_func) |

## PollTest の追加詳細 (af610f3 発見)

既存の擬似コードに対する補足:
- `stwbrx` (byte-reversed store) で結果をresult_ptrに書き込む
- InitTest も `stwbrx` でflagsフィールドを書き込む
- これらはDMAバッファのエンディアン変換のため

## 主要メモリアドレス (修正済み)

| Address | Size | Description |
|---------|------|-------------|
| 0x800C8C20 | - | ベースアドレス (lis 0x800D; addi -0x73E0) |
| 0x800C8DA0 | ~4KB | ログリングバッファ |
| 0x800CCDA0 | 64*0x60 | テストエントリテーブル (旧: 0x800DCDA0 は誤り) |
| 0x800CE5A0 | var | g_test_ctx (旧: 0x800DE5A0 は誤り) |
| 0x800CE5A4 | N*0x10 | g_test_ctx entries (type/status/flags) |
| 0x800D18FC | 260 bytes | テスト状態構造体 (TestState) |
| 0x800D1708 | ~500 bytes | テストノード構造体 |
| 0x80077C84 | 16 bytes | ステートハンドラジャンプテーブル |
| 0x8008FD84 | 8 bytes | TestHardware の buf[2] が指すアドレス (コードセクション内、CardSaveData_Parser 内の `cmpwi r29,0x4` 命令)。str_ptr 解釈は疑わしい |

## TestHardware レスポンス実験結果

| buf[0] | buf[1] | 画面表示 |
|--------|--------|---------|
| 0x0301000a (元のまま) | 0 | MEDIA COMMAND ERROR |
| 同上 | 1 | MEDIA COMMAND ERROR |
| 同上 | 2 | MEDIA COMMAND ERROR |
| 同上 | 3 | MEDIA COMMAND ERROR |
| 同上 | 4 (=test_type) | CHECKING 0% (ループ) |
| 同上 | 5~12 | MEDIA COMMAND ERROR |
| 0x00000000 | 0 | ハング |
| 0x03010180 | test_type | ハング |

## Execute2 コマンドシーケンス (NETWORK TEST)

1. `GetNetworkConfig` (0x0104) - buf[0]=0x01040009 - ネットワーク設定読み込み (ReadNetworkConfig @ 0x800686a4)
2. `TestHardware` (0x0301) - buf[0]=0x0301000a, buf[1]=4(test_type), buf[2]=string_ptr
3. `Unknown_8000` (0x8000) - buf[0]=0x8000000a, params all 0 - TestHardware直後に送信

### Unknown_8000 分析

TestHardware 完了の約3ms後に送信される。パラメータなし。
シーケンス番号 (下位16ビット=0x000a) が TestHardware と同一。

コマンドワード 0x8000000a のバイト構造 (LE):
- byte[0]=0x0a, byte[1]=0x00, byte[2]=0x00, byte[3]=0x80
- byte[3] が 0x80 → generic completion `|= 0x80` はノーオプ
- Dispatch ポストハンドラ Phase 8: `lhz field_0x20+2 & 0x80` = (0x00<<8)|0x80 & 0x80 = 0x80
  → test_table エントリがクリアされる (これが期待動作: テスト完了後のクリーンアップ)

**PollTest との関係**: PollTest は response byte[3] == 0x80 で -3 (エラー) を返す。
しかし、0x8000 は TestHardware **完了後** に送信されるため、PollTest はもう実行されていない。
テストはすでに state=4 (PASSED) に遷移している。

### Unknown_8000 ハング問題と修正 ★

**初回実装 (ハング)**:
```cpp
s_media_buffer[3] = 0;  // byte[3] をクリア
return 0;               // generic completion をスキップ
```
→ NETWORK TEST を押した瞬間にハング。

**根本原因: s_media_buffer の二重領域アーキテクチャ**

s_mediaboard_ranges は 0x89000000 に size=0x220 で s_media_buffer_32 をマッピングしている。
これにより s_media_buffer 内に2つの独立した領域が存在する:

```
s_media_buffer[0x000..0x01F]  ← ベースアドレス (0x89000000) でアクセス
                                 DMA State Machine がここを読む (レスポンス領域)
                                 ★ byte[3] = completion flag (0x80)

s_media_buffer[0x200..0x21F]  ← オフセット+0x200 (0x89000200) でアクセス
                                 Dispatch Phase 1 がここを読む (コマンド領域)
                                 Execute2 処理で memset 0 されている
```

Execute2 ハンドラの処理順序:
1. `memcpy(s_media_buffer, s_media_buffer + 0x200, 0x20)` — コマンドを+0x000にコピー
2. `memset(s_media_buffer + 0x200, 0, 0x20)` — コマンド領域をクリア
3. コマンド処理 (TestHardware等)
4. generic completion: `s_media_buffer[3] |= 0x80` — レスポンス領域に完了フラグ

`s_media_buffer[3] = 0` はレスポンス領域 (+0x000) の byte[3] をゼロにした。
DMA State Machine は +0x000 を読み、byte[3] の 0x80 ビットで完了を判定するため、
これが消えると PollCompletion が永久に「処理中」を返し、ハングが発生した。

**修正 (generic completion を使用)**:
```cpp
case AMMBCommand::Unknown_8000:
    NOTICE_LOG_FMT(AMMEDIABOARD, "GC-AM: Unknown_8000 (Execute2): cleanup command");
    break;  // → generic completion (s_media_buffer[3] |= 0x80) が実行される
```

0x8000 コマンドでは byte[3] はすでに 0x80 (コマンドワードの一部) なので、
`|= 0x80` はノーオプ。DMA SM は正常に完了を検知する。
Dispatch ポストハンドラは 0x80 を見て test_table エントリをクリアする (期待動作)。

## "CHECKING 0%" ループの原因分析 (★解決済み)

**PollTest が -5 を返し続けた理由:**
`PollDMAComplete()` (fn_IsComplete, 0x80065A60) が常に 0 を返していた。

InitTest が DMA を投入し、State3Handler がそれを実行し、
Dolphin 側の Execute2 TestHardware ハンドラがレスポンスを返す。
PollCompletion は 0 を返す (DMA 自体は完了)。

しかし、PollTest が呼ぶ `PollDMAComplete` は **別の完了チェック** を行う:
テストエントリテーブル (0x800CCDA0) の field_02 を見る。
このフィールドが更新されていないため、完了と認識されなかった。

**解決**: TestHardware ハンドラで generic completion をスキップ (`return 0`)。
これにより s_media_buffer[3] の 0x80 ビットが設定されず、
Dispatch ポストハンドラが test_table エントリをクリアしない。
fn_IsComplete がエントリを見つけられるようになり、PollTest が正常に完了する。

## 修正アプローチ (参考: 最終的に TestHardware は独自 return 0、Unknown_8000 は generic completion)

### アプローチ A: ppc_state.gpr[28] を直接読む

State3Handler は r28 を保存しないため、Execute2 DMA 発火時に
`ppc_state.gpr[28]` には TestManager_Dispatch が設定した ctx_entry ポインタが
そのまま残っている。

```cpp
const u32 r28 = ppc_state.gpr[28];
if (r28 != 0) {
    u8 status = memory.Read_U8(r28 + 1);
    if (status <= 3) {
        memory.Write_U8(4, r28 + 1);  // status = 4 → ハンドラスキップ
    }
}
```

### アプローチ B: メモリ直接検索

テストエントリテーブルからマッチキーを取得し、g_test_ctx を検索:
```cpp
u8 match_key = memory.Read_U8(0x800CCDA0 + test_index * 0x60 + 0x40);
u32 count = memory.Read_U32(0x800CE5A0);
for (u32 i = 0; i < count; i++) {
    u32 addr = 0x800CE5A4 + i * 0x10;
    if (memory.Read_U8(addr) == match_key) {
        memory.Write_U8(4, addr + 1);  // status byte
        break;
    }
}
```

### アプローチ C: ctx_entry.field_08 の bit 0 をクリア

fn_80066610 は `entry->field_08 & 1` をチェックする。
bit 0 が 0 なら InitTest は 0 を返し (DMA を投入せず)、テストは即座にパスする。

## TestManager_Dispatch ポストハンドラ (0x80064F5C - 0x800652B0) ★完全逆アセンブリ

State handler 呼び出し後の処理。DMA レスポンス読み取り、コマンドキュー、test_table エントリクリア判定を行う。

### レジスタコンテキスト

```
r30 = base = 0x800C8C20
r24 = g_test_ctx = 0x800CE5A0
r27 = 最初: DMAレスポンスバッファ (aligned stack) → 後: test_table[idx].field_0x20
r28 = ctx_entry (g_test_ctx内のマッチしたエントリ)
r29 = match_key (test_table[idx].field_0x40)
r31 = test_index
r23 = ctx_entry->field_0C (DMAアドレス, e.g. 0x89000200)
r26 = DMAアドレスのコピー
```

### Phase 1: DMA Read リトライループ (0x80064F5C - 0x80064FD4)

```asm
; === dma_addr < 0x80000000 チェック ===
80064f5c: lis     r26, 0x8000          ; r26 = 0x80000000
80064f60: cmplw   r23, r26             ; dma_addr < 0x80000000 ?
80064f64: bge     skip_init            ; ≥ ならスキップ
; (初回: cmd_bufクリア、config読み込み)

retry:
; === DMA Read: デバイスからレスポンス読み取り ===
80064f84: mr      r3, r27              ; r3 = response_buf
80064f88: li      r4, 0x20             ; size = 0x20
80064f8c: bl      DCInvalidateRange    ; キャッシュ無効化 (0x8004FE50)
80064f90: memset(cmd_buf, 0, 0x30)
80064fa0: mr      r4, r27              ; data = response_buf
80064fa4: mr      r6, r23              ; addr = dma_addr (0x89000200)
80064fb4: li      r8, 2                ; mode = 2
80064fb8: bl      SendCommand          ; DMA Read

; === PollCompletion ===
80064fbc: addi    r3, r1, 0x08         ; cmd_buf
80064fc0: bl      PollCompletion
80064fc4: cmpwi   r3, 0
80064fc8: bne     80064fbc             ; 完了まで待機

; === レスポンスチェック ===
80064fcc: lhz     r0, 0x02(r27)        ; ★ response[2..3] = status halfword
80064fd0: cmplwi  r0, 0x0000
80064fd4: bne     retry                ; ★ 0以外 → リトライ!
```

**重要**: `response[2..3] != 0` ならDMA Read全体をリトライする。
デバイスは `0x0000` を返して「idle/ready」を示す必要がある。

### Phase 2: r27 の再割り当て (0x80064FD8 - 0x80064FF4)

```asm
80064fd8: cmplwi  r31, 0x3F            ; test_index > 63 ?
80064fdc: bgt     null_r27
80064fe0: mulli   r0, r31, 0x60        ; offset = index * 96
80064fe4: addi    r3, r30, 0x4180      ; r3 = test_table base (0x800CCDA0)
80064fe8: addi    r27, r3, 0x20        ; r27 = base + 0x20 (field_0x20)
80064fec: add     r27, r27, r0         ; ★ r27 = &test_table[idx].field_0x20
80064ff0: b       continue
80064ff4: li      r27, 0               ; r27 = NULL (無効インデックス)
```

**r27 が再利用される**: DMAレスポンスバッファ → test_table[idx].field_0x20 ポインタ。

### Phase 3: コマンドキュー投入 (0x80064FF8 - 0x800650CC)

```asm
; === base[0] のフラグチェック ===
80064ff8: lbz     r0, 0(r30)           ; base[0] = flags byte
80064ffc: rlwinm. r0, r0, 0, 30, 30   ; test bit 1 (& 0x02)
80065000: beq     skip_TR              ; bit 1 クリア → スキップ

; === match_key == 0xA9 チェック ===
80065004: clrlwi  r0, r29, 24          ; r0 = match_key & 0xFF
80065008: cmplwi  r0, 0xA9
8006500c: bne     skip_TR

; === "T/R" キューエントリ投入 ===
80065040: stb     r5, 1(r8)            ; qe[1] = seq
80065044: addi    r3, r8, 0x10         ; qe->data = qe + 0x10
8006504c: stb     r7, 2(r8)            ; qe[2] = 0x54 ('T')
80065050: stb     r6, 3(r8)            ; qe[3] = 0x52 ('R')
80065054: stw     r27, 4(r8)           ; qe[4..7] = dest (field_0x20 ptr)
80065058: stw     r9, 8(r8)            ; qe[8..B] = dma_addr
8006505c: stw     r0, 0xC(r8)          ; qe[C..F] = size (0x20)
80065060: bl      memcpy               ; memcpy(qe->data, field_0x20, 0x20)
```

同様に base[0] & 0x01 かつ match_key == 0xA9 で **"W/R"** エントリを投入 (0x80065064 - 0x800650CC)。

### Phase 4: DMA Write-back (0x800650D0 - 0x8006513C)

```asm
; === dma_addr < 0x80000000 チェック ===
800650d0: lis     r0, 0x8000
800650d4: cmplw   r26, r0
800650d8: bge     skip_init2

; === test_table[idx].field_0x20 をフラッシュして DMA Write ===
800650f8: mr      r3, r27              ; field_0x20
800650fc: li      r4, 0x20
80065100: bl      DCFlushRange         ; ★ キャッシュフラッシュ (0x8004FE7C)
                                        ; (memzeroではない! データは保持される)
80065104: memset(cmd_buf2, 0, 0x30)
80065114: mr      r4, r27              ; data = field_0x20
80065118: mr      r6, r26              ; addr = dma_addr
80065128: li      r8, 2                ; mode = 2
8006512c: bl      SendCommand          ; DMA Write (field_0x20 → デバイス)

; === PollCompletion ===
80065130: addi    r3, r1, 0x38
80065134: bl      PollCompletion
80065138: cmpwi   r3, 0
8006513c: bne     80065130             ; 完了まで待機
```

**重要**: `DCFlushRange` はデータをゼロにしない。test_table[idx].field_0x20 のコマンドデータ
(例: 0x0301コマンド) がそのままデバイス (+0x200) に書き込まれる。

### Phase 5: "W/2" キューエントリ (0x80065140 - 0x800651AC)

base[0] & 0x04 かつ match_key == 0xA9 で "W/2" (書き戻し確認) エントリを投入。
dma_addr = 0xFFFFFFFF, size = 0xFFFFFFFF (センチネル値)。

### Phase 6: State Handler ディスパッチ (0x800651B0 - 0x800651D8)

```asm
800651b0: lbz     r0, 1(r28)           ; r0 = ctx_entry[1] (status byte)
800651b4: cmplwi  r0, 3
800651b8: bgt     skip_handler         ; status > 3 → ハンドラスキップ

800651bc: lis     r3, 0x8007
800651c4: addi    r3, r3, 0x7C84       ; jump_table = 0x80077C84
800651c8: lwzx    r12, r3, r0          ; r12 = handler_func[status]
800651cc: cmplwi  r12, 0
800651d0: beq     skip_handler         ; NULL → スキップ
800651d4: mtctr   r12
800651d8: bctrl                        ; ★ handler() 呼び出し
```

### Phase 7: 履歴ログ & 完了チェック (0x800651DC - 0x800652B0)

```asm
; === 完了カウンタ更新 ===
800651dc: lwz     r4, 0xB4(r24)        ; completion_count = g_test_ctx[0xB4]
800651e0: addi    r3, r30, 0x0180      ; history_base = base + 0x180
800651e8: addi    r0, r4, 1            ; r0 = count + 1
800651ec: stw     r0, 0xB4(r24)        ; g_test_ctx[0xB4]++

; === field_0x20 からログエントリ構築 ===
800651f4: rlwinm  r0, r5, 2, 18, 29   ; r0 = (counter * 4) & 0x3FFC
800651f8: lbz     r6, 0(r27)           ; r6 = field_0x20[0]
800651fc: lhz     r7, 2(r27)           ; r7 = field_0x20[2..3]
80065200: rlwinm  r6, r6, 16, 0, 15   ; r6 = byte << 16
80065204: rlwinm  r5, r7, 24, 24, 31  ; r5 = r7 >> 8 (高バイト)
8006520c: oris    r4, r6, 0x3000       ; r4 = 0x30000000 | (byte << 16)
80065210: rlwimi  r5, r7, 8, 16, 23   ; r5 |= (r7_lo << 8) (バイトスワップ)
80065214: or      r4, r4, r5           ; r4 = 0x30BBYYXX (ログエントリ)
80065218: stwx    r4, r3, r0           ; history[offset] = r4

; ======================================================
; ★★★ 最重要チェック: テスト完了判定 ★★★
; ======================================================
8006521c: lhz     r0, 0x02(r27)        ; r0 = field_0x20[2..3] (status halfword)
80065220: rlwinm. r0, r0, 0, 24, 24   ; r0 = r0 & 0x80; CR0設定
80065224: beq     0x800652AC           ; ★ bit 0x80 未セット → エピローグへ (クリアしない)

; === bit 0x80 セット: test_table エントリ全体クリア ===
80065228: addi    r23, r30, 0x5980     ; r23 = g_test_ctx (0x800CE5A0)
8006522c: addi    r23, r23, 0xBC       ; r23 = sync_obj (0x800CE65C)
80065230: mr      r3, r23
80065234: bl      lock                 ; lock(sync_obj)

80065238: cmplwi  r31, 0x3F            ; test_index > 63 ?
8006523c: bgt     unlock_skip          ; → アンロックしてスキップ

80065240: mr      r3, r23
80065244: bl      unlock               ; unlock(sync_obj)

; === test_table[index] の96バイト全体をゼロクリア ===
80065248: mulli   r0, r31, 0x60        ; offset = index * 96
8006524c: addi    r3, r30, 0x4180      ; test_table base
8006525c: add     r3, r3, r0           ; r3 = &test_table[index]

; (履歴に 0xC1XXYY ログエントリ記録)
80065270-80065294: ...

80065298: bl      memset               ; ★ memset(&test_table[index], 0, 0x60)

; === クリーンアップ ===
8006529c: mr      r3, r23
800652a0: bl      lock
800652a4: mr      r3, r23
800652a8: bl      unlock

; === エピローグ ===
800652ac: li      r3, 0                ; return 0
800652b0: lmw     r23, 0xAC(r1)       ; r23-r31 復元
```

### 擬似コード

```c
int TestManager_Dispatch_PostHandler(void) {
    u32 dma_addr = ctx_entry->field_0C;  // e.g. 0x89000200

    // ==== Phase 1: DMA Read (デバイスの状態読み取り) ====
retry:
    DCInvalidateRange(response_buf, 0x20);
    memset(cmd_buf, 0, 0x30);
    SendCommand(cmd_buf, response_buf, 0x20, dma_addr, NULL, 2);
    while (PollCompletion(cmd_buf) != 0) {}

    // デバイスがready (response[2..3]==0) でなければリトライ
    if (*(u16*)(response_buf + 2) != 0)
        goto retry;

    // ==== Phase 2: r27 再割り当て ====
    u8* field_0x20;
    if (test_index <= 63)
        field_0x20 = &test_table[test_index * 0x60 + 0x20];
    else
        field_0x20 = NULL;

    // ==== Phase 3: コマンドキュー投入 (0xA9 のみ) ====
    if ((base[0] & 0x02) && match_key == 0xA9)
        enqueue("T/R", field_0x20, dma_addr);  // Read snapshot
    if ((base[0] & 0x01) && match_key == 0xA9)
        enqueue("W/R", field_0x20, dma_addr);  // Write snapshot

    // ==== Phase 4: DMA Write-back ====
    DCFlushRange(field_0x20, 0x20);  // ★ キャッシュフラッシュのみ、クリアではない!
    memset(cmd_buf2, 0, 0x30);
    SendCommand(cmd_buf2, field_0x20, 0x20, dma_addr, NULL, 2);  // Write to device
    while (PollCompletion(cmd_buf2) != 0) {}

    // ==== Phase 5: "W/2" キューエントリ (0xA9 のみ) ====
    if ((base[0] & 0x04) && match_key == 0xA9)
        enqueue("W/2", field_0x20, 0xFFFFFFFF);

    // ==== Phase 6: State Handler ====
    u8 status = ctx_entry[1];
    if (status <= 3) {
        void (*handler)(void) = jump_table[status];  // 0x80077C84
        if (handler) handler();  // → State3Handler (Execute2 TestHardware)
    }

    // ==== Phase 7: 履歴ログ ====
    g_test_ctx->completion_count++;
    u8 result_byte = field_0x20[0];
    u16 result_half = *(u16*)(field_0x20 + 2);
    u32 log_entry = 0x30000000 | (result_byte << 16) | bswap16(result_half);
    history[counter++ & 0xFFF] = log_entry;

    // ==== Phase 8: 完了判定 ★★★ ====
    u16 status_hw = *(u16*)(field_0x20 + 2);
    if (status_hw & 0x80) {
        // bit 0x80 セット → テスト完了: エントリ全体をクリア
        lock(sync_obj);
        if (test_index <= 63) {
            unlock(sync_obj);
            memset(&test_table[test_index * 0x60], 0, 0x60);  // ★全96バイトクリア
            lock(sync_obj);
        }
        unlock(sync_obj);
    }

    return 0;
}
```

### 発見事項まとめ

1. **DCFlushRange ≠ memzero**: 0x8004FE7C はキャッシュフラッシュ。field_0x20 のデータは保持される。
   DMA Write はコマンドデータ (例: 0x0301...) をデバイスに送信する。

2. **DMA Read リトライ**: response[2..3] が 0 でなければ、DMA Read 全体をリトライする。
   デバイスは `0x0000` で "idle/ready" を返す必要がある。

3. **test_table エントリクリア条件**: field_0x20[2..3] の bit 0x80 がセットされていれば、
   test_table エントリの96バイト全体がゼロクリアされる。これがテスト完了のシグナル。

4. **State Handler はfield_0x20に書かない**: State3Handler は応答をスタック上のローカル変数に
   コピーする。field_0x20 はハンドラ呼び出し前後で変化しない。

## Dolphin ログからのDMAパターン分析

### ブート Execute2 (7フェーズプロトコル)

```
Write +0x000 (0x20)      ← データ書き込み
Write +0x200 (0x20)      ← コマンド書き込み
Read  +0x200 (0x20)      ← コマンド確認読み取り
Write +0x200 (0x20)      ← 最終化書き込み
Execute trigger           ← DI Read 0x88000000
Read  +0x000 (0x20)      ← レスポンス読み取り
Write +0x000 (0x20)      ← クリーンアップ
```

### Dispatch 経由 Execute2 (5フェーズ: Dispatch前処理 + State3Handler)

```
Read  +0x200 (0x20)      ← Dispatch: デバイス状態読み取り (response[2..3]==0確認)
Write +0x200 (0x20)      ← Dispatch: field_0x20のコマンドデータをデバイスに書き込み
Execute trigger           ← State3Handler: SendCommand to 0x88000000
Read  +0x000 (0x20)      ← State3Handler DMA: レスポンス読み取り
Write +0x000 (0x20)      ← State3Handler DMA: クリーンアップ
```

### Unknown_104 vs TestHardware (ログ比較)

DMAパターンは**完全に同一**。5フェーズが同じ順序で実行される。

```
Unknown_104: R+200, W+200, Execute(0104), R+000, W+000 → 正常継続
TestHardware: R+200, W+200, Execute(0301), R+000, W+000 → ハング
```

ハングはDMAプロトコルの違いではなく、Execute2完了後のPPCコードパスの問題。

### ハング箇所の推定

Execute2 完了後、State3Handler の PollCompletion がリターンし、
Dispatch のポストハンドラが実行される。しかし、その後のテストループが
ISR ポーリング (:04 00) で永久に停止する。

2つ目の Dispatch 呼び出し (DMA Read +0x200) がログに現れないことから、
テストループ自体が Dispatch を再呼び出ししていないか、
Dispatch 内の別のポイントでブロックされている可能性がある。

## バイトオーダー分析 (DMA ↔ s_media_buffer)

### DMA データのバイトオーダー

PPC は MediaBoard への DMA データを **stwbrx** (byte-reversed store) で書き込む。
そのため PPC メモリ内の DMA データは **リトルエンディアン** 形式。

```
PPC stw   0x0301000A → PPC memory: [03, 01, 00, 0A] (BE)
PPC stwbrx 0x0301000A → PPC memory: [0A, 00, 01, 03] (LE)  ← DMAはこちら
```

CopyFromEmu はバイトスワップなしの raw copy なので:
- s_media_buffer に入るバイト列: [0A, 00, 01, 03]
- x86 LE で s_media_buffer_32[0] = 0x0301000A (値がそのまま読める)

### s_media_buffer[3] |= 0x80 の影響

Execute2 完了時に `s_media_buffer[3] |= 0x80` が実行される。
s_media_buffer[3] は LE ワードの MSB (最上位バイト)。

```
コマンドワード: s_media_buffer_32[0] = 0x0301000A
LE バイト列: [0A, 00, 01, 03]
byte[3] = 0x03  ← コマンド番号の上位バイト

byte[3] |= 0x80 → 0x83
DMA Read で PPC に返るバイト: [0A, 00, 01, 83]
PPC lhz at +2 = [01, 83] = 0x0183
```

### Big Test Function (0x80065324) ★完全逆アセンブリ

Dispatch内のState Handlerとは別に、テストエントリテーブルへのレスポンス書き込みを行う関数。
Execute2 DMAを発行してレスポンスを取得し、test_tableエントリにコピーする。

```c
int BigTestFunc(u8 test_type) {
    // Phase 1: g_test_ctx からtest_typeに一致し field_04 & 1 のエントリを検索
    ctx_entry* entry = NULL;
    for (int i = 0; i < g_test_ctx->count; i++) {
        if (entries[i].type == test_type && (entries[i].field_04 & 1)) {
            entry = &entries[i];
            break;
        }
    }
    if (!entry) return 4;  // テストタイプ未登録

    // Phase 2: field_08 (field_0x20ポインタ) を DMAアドレスとして使用
    u32 field_0x20_ptr = entry->field_08;  // r25
    if (field_0x20_ptr < 0x80000000) {
        // 初回: コマンドバッファ初期化
        memset(local_buf, 0, 0x30);
        helper_setup_cmd(local_buf, *(u8*)(g_test_ctx + 0xAA));
    }

    // Phase 3: Execute2 DMA 発行
    DCInvalidateRange(dma_buf, 0x20);
    memset(local_buf, 0, 0x30);
    SendCommand(local_buf, dma_buf, 0x20, field_0x20_ptr, 0, 2);
    while (PollCompletion(local_buf) != 0) {}

    // Phase 4: レスポンスチェック
    u16 status = *(u16*)(dma_buf + 2);  // response[2..3]
    if (status == 0) return 3;  // ★ COMMUNICATION ERROR

    // Phase 5: 初回ステータス確認
    if (!(entry->field_04 & 0x2)) {
        if (status != 0x0180) {
            result = 4;  // 期待外のステータス → CHECKING 0%
        } else {
            entry->field_04 |= 0x2;  // "初期化済み" フラグをセット
        }
    }
    if (!(entry->field_04 & 0x2)) goto cleanup;

    // Phase 6: レスポンス bit 7 チェック
    if (status & 0x80) {
        // ★ bit 7 セット: test_table からマッチするエントリを検索
        // 64エントリ (16 outer × 4 sub-records) をスキャン
        // field_0x22 & 0x80 == 0 かつ field_0x20 == response[0] で一致
        u8* match = scan_test_table(test_table, dma_buf[0]);
        if (match) {
            memcpy(match, dma_buf, 0x20);  // ★ DMAレスポンスをtest_tableにコピー
        }
    } else {
        // bit 7 未セット: g_test_ctx+0xBC 経由の別処理
        // (ダンプ範囲 0x800655FF で途切れ)
    }

cleanup:
    return result;
}
```

### BigTestFunc レスポンス判定表

| response[2..3] | field_04 bit 1 | 結果 |
|----------------|---------------|------|
| 0x0000 | any | return 3 = **COMMUNICATION ERROR** |
| 0x0180 | 未セット | bit 1 セット → 次回以降進行可能 |
| != 0x0180 | 未セット | return 4 = CHECKING 0% |
| & 0x80 セット | セット済み | test_table にレスポンスコピー → 成功 |
| & 0x80 未セット | セット済み | 別処理パス |

### 旧ログとの対応

```asm
lhz  r0, 2(r27)       ; response+2 の halfword
cmplwi r0, 0           ; 0 なら COMMUNICATION ERROR (return 3)
cmplwi r0, 0x0180      ; 0x0180 でなければ CHECKING 0% (return 4)
; 0x0180 → テスト進行 (field_04 bit 1 セット)
```

### テスト失敗の原因 (2つ)

**原因1: completion bit (0x80) によるエントリクリア**

`s_media_buffer[3] |= 0x80` がレスポンスの byte[3] に設定される。
Dispatch ポストハンドラ Phase 8 が `field_0x20[2..3] & 0x80` をチェックし、
ビットがセットされていれば test_table エントリ全体を memset 0 する。
これにより `fn_IsComplete` (PollDMAComplete) がエントリを見つけられず、
`PollTest` が永久に -5 を返す → "CHECKING 0%" 無限ループ。

**原因2: 文字列バイトオーダー**

ゲームは DMA バッファの文字列を `lwbrx` で読む。
`Write_U32` は BE 格納なので、PPC memory [54,45,53,54] → lwbrx → "TSET"。
`Write_U32_Swap` で LE 格納すれば、PPC memory [54,53,45,54] → lwbrx → "TEST"。

### 修正

1. TestHardware ハンドラで `return 0` を使い、generic completion (`s_media_buffer[3] |= 0x80`) をスキップ
   - EXI 割り込みは手動で GenerateInterrupt(0x10) を呼ぶ
   - 0x80 ビットが無いのでポストハンドラ Phase 8 がエントリをクリアしない
   - fn_IsComplete がエントリを見つけられる → PollTest が 0 を返せる可能性

2. 文字列書き込みを `Write_U32_Swap` に変更 (LE 格納)
   - lwbrx で "TEST OK" と正しく読める

## g_test_ctx ランタイムダンプ (NETWORK TEST 時) ★新規

```
g_test_ctx count=2, state_B4=0x0000000a
ctx[0] @800CE5A4: type=29 status=02 field_04=00000003 field_08=84000000 field_0C=84000020
ctx[1] @800CE5B4: type=A9 status=03 field_04=0000000B field_08=89000000 field_0C=89000200
```

### フィールド解析

| Field | ctx[0] (type=0x29, Execute1) | ctx[1] (type=0xA9, Execute2) |
|-------|------------------------------|-------------------------------|
| type | 0x29 (ネットワークテスト主) | 0xA9 (Execute2 サブテスト) |
| status | 0x02 (Dispatch state 2) | 0x03 (Dispatch state 3) |
| field_04 | 0x03 (bit0=1, bit1=1) | 0x0B (bit0=1, bit1=1, bit3=1) |
| field_08 | **0x84000000** (Execute1 Read) | **0x89000000** (Execute2 Read) |
| field_0C | 0x84000020 (Dispatch DMA addr) | 0x89000200 (Dispatch DMA addr) |

### field_08 と field_0C の違い

- **field_08**: BigTestFunc が使う DMA 読み取りアドレス (ベースアドレス)
- **field_0C**: TestManager_Dispatch が使う DMA アドレス (コマンド/レスポンス領域)

Execute1 の場合:
- field_08 = 0x84000000 → s_media_buffer[0x000] (レスポンス読み取り)
- field_0C = 0x84000020 → s_media_buffer[0x020] (Dispatch Phase 1 読み取り)

Execute2 の場合:
- field_08 = 0x89000000 → s_media_buffer[0x000] (レスポンス読み取り)
- field_0C = 0x89000200 → s_media_buffer[0x200] (Dispatch Phase 1 読み取り)

### COMMUNICATION ERROR の推定原因 (★後の「全テストフロー再解析」セクションも参照)

setup_func type 0x29 ハンドラは **BigTestFunc** を呼ぶ。
BigTestFunc は ctx[0].field_08 = **0x84000000** に DMA Read を送信する。

0x84000000 は Execute1 のレスポンス領域 (s_media_buffer[0x000..0x05F]) にマッピングされている。
BigTestFunc は response[2..3] (halfword at offset +2) を確認し:
- `== 0` → return 3 → **COMMUNICATION ERROR**
- `== 0x0180` → field_04 の bit 1 をセット → 成功パス
- `& 0x80` → test_table にコピー

Execute1 レスポンス領域に適切なステータス (0x0180) が設定されていなければ、
BigTestFunc は return 3 を返し COMMUNICATION ERROR となる。

**ただし**: 後の「全テストフロー再解析」セクションで判明したように、
メインテストフロー (setup_func → InitTest → PollTest → check_result) からは
BigTestFunc は**呼ばれない**。BigTestFunc の実際の呼び出し元は未特定。
ctx[0].field_04 = 3 (bit 1 セット済み) から、BigTestFunc は過去に成功しているが、
2回目以降の呼び出しで失敗 (response[2..3]==0) → COMMUNICATION ERROR の可能性がある。

## "COMMUNICATION ERROR" 文字列テーブル ★新規

3箇所に "COMMUNICATION ERROR" のバイトパターンが存在 (コードセクション内):

| Address | テストカテゴリ | 備考 |
|---------|--------------|------|
| 0x8006D960 | MEDIA BOARD TEST | コード領域 (.text1) 内 |
| 0x8006DAC4 | NETWORK SETTING | コード領域 (.text1) 内 |
| 0x8006E120 | SYSTEM INFORMATION | コード領域 (.text1) 内 |

**注意**: これらのアドレスは .text1 セクション (0x8002bca0-0x802e88df) 内にある。
Ghidra の UI ハンドラ解析で、0x8006xxxx の関数は全て DrawText データテーブル
(DAT_803fXXXX) を参照する表示専用ハンドラと判明。"COMMUNICATION ERROR" の文字列自体は
DrawText のタイルインデックステーブルとして .data4 セクション内に存在する可能性が高い。
上記アドレスは PPC 命令バイトが ASCII と一致しているだけの可能性がある。

NETWORK TEST で表示される COMMUNICATION ERROR は NETWORK SETTING カテゴリ。
隣接文字列: "CHECKING", "DIMM BOARD", "NETWORK SETTING", "COMMUNICATION ERROR", "MEDIA COMMAND ERROR"

## setup_func type 0x29 ハンドラ (0x80066E08 - 0x800678CC) ★完全逆アセンブリ

958命令の大規模関数。Execute1 (0x84000000) 経由のネットワークテストを設定する。
setup_func (0x800666F4) のフレーム (0x660) 内で実行される。

### 呼び出される関数

| Address | Name | 用途 |
|---------|------|------|
| 0x8004FE50 | DCInvalidateRange | DMA Read 前のキャッシュ無効化 |
| 0x8004FE7C | DCFlushRange | DMA Write 前のキャッシュフラッシュ |
| 0x8000528C | memset | バッファゼロクリア |
| 0x80059214 | SendCommand_v1 | FirmwareStatus 系 DMA (0x8000xxxx) |
| 0x80056BC8 | SendCommand_v2 | Execute1/Firmware DMA (0x84xxxxxx) |
| 0x80059664 | PollCompletion | DMA 完了ポーリング |
| 0x800698A4 | SetupTestData_1 | g_test_ctx データ設定 (type 0x29 用) |
| 0x80069078 | SetupTestData_2 | g_test_ctx データ設定 (type 0xA9 用) |

### SendCommand バリアント

```
SendCommand_v1 (0x80059214):
  - FirmwareStatus2 (0x80000140) の読み書きに使用
  - FirmwareStatus1 (0x80000120) の読み取りに使用
  - Unknown Status (0x80000160) の読み取りに使用

SendCommand_v2 (0x80056BC8):
  - FirmwareAddress (0x84800000) の読み取りに使用
  - Execute1 応答領域の読み取りに使用
  - Execute2 応答領域の読み取りに使用
```

### 擬似コード (Function 1: 0x80066E08 - 0x800678CC)

```c
// setup_func type 0x29 handler
// r31 = ctx_entry (g_test_ctx 内の type==0x29 エントリ)
// r26 = data_ptr (setup_func の arg2)
int setup_func_0x29(ctx_entry* r31, void* data_ptr) {
    u8 aligned_buf_A[0x20] __attribute__((aligned(32)));  // sp+0x3B7 aligned
    u8 cmd_buf_A[0x30];  // sp+0x1B8

    // ============================================================
    // Phase 1: FirmwareStatus2 (0x80000140) 読み取り
    // ============================================================
    DCInvalidateRange(aligned_buf_A, 0x20);
    memset(cmd_buf_A, 0, 0x30);
    SendCommand_v1(cmd_buf_A, aligned_buf_A, 0x20, 0x80000140, 0, 2);
    while (PollCompletion(cmd_buf_A) != 0) {}

    // レスポンスのバイトスワップ & ビットチェック
    u32 fw_status2 = bswap32(*(u32*)aligned_buf_A);
    if (!(fw_status2 & 1)) {
        // ★ FirmwareStatus2 bit 0 未セット = MediaBoard 未準備
        goto firmware_recovery;
    }

    // ============================================================
    // Phase 2: メインパス - テストコンテキスト設定
    // ============================================================
    g_test_ctx->field_0xAA = 1;  // フラグセット

    // ============================================================
    // Phase 3: FirmwareStatus1 (0x80000120) 読み取り
    // ============================================================
    u8 aligned_buf_B[0x20] __attribute__((aligned(32)));
    u8 cmd_buf_B[0x30];
    memset(cmd_buf_B, 0, 0x30);
    SendCommand_v1(cmd_buf_B, aligned_buf_B, 0x20, 0x80000120, 0, 2);
    while (PollCompletion(cmd_buf_B) != 0) {}

    u32 fw_status1 = bswap32(*(u32*)aligned_buf_B);

    // ============================================================
    // Phase 4: ctx_entry (type=0x29) 設定 ★★★
    // ============================================================
    g_test_ctx->field_0xA8 = (u16)(fw_status1 >> 16);  // sth
    r31->status = 2;                     // ctx_entry[1] = 2
    r31->field_04 |= 1;                  // DMA投入フラグ
    r31->field_08 = 0x84000000;          // ★ Execute1 レスポンス領域!
    r31->field_0C = 0x84000020;          // ★ Execute1 コマンド領域!

    // r29 = r31->field_08 = 0x84000000
    // r28 = r31->type = 0x29

    // ============================================================
    // Phase 5: テストサブエントリ設定 (type 0x29 用)
    // 繰り返しブロック: global_state チェック → test_table エントリ投入
    // ============================================================
    // 各ブロック構造:
    //   1. lbz r0, *(0x800C8C20)  → global state flags
    //   2. rlwinm. r0, r0, 0, 30, 30 → test bit 1 (& 0x02)
    //   3. beq skip → bit 未セットならスキップ
    //   4. cmplwi r28, 0xA9 → type チェック
    //   5. bne skip → 0xA9 でなければスキップ
    //   6. test_table エントリ投入 (stride 0x30、'C','L' or 'W','R' タグ付き)
    //
    // type=0x29 の場合、0xA9 チェックを通過しないので
    // これらのブロックはスキップされる。

    // ============================================================
    // Phase 6: Execute1 レスポンス領域チェック (0x84000000)
    // ============================================================
    // r29 = 0x84000000 (ctx[0].field_08)
    if (r29 >= 0x80000000) {
        // アドレスが 0x80000000 以上 → 特殊処理
        // ... (Execute1 固有の初期化)
    }

    // ============================================================
    // Phase 7: type 0xA9 コンテキスト設定
    // (0x800673B4 から分岐で到達する type 0xA9 ハンドラ部分)
    // ============================================================

    // --- type 0xA9 ハンドラ (0x800673B4) ---
    // Unknown Status (0x80000160) 読み取り
    u8 aligned_buf_C[0x20] __attribute__((aligned(32)));
    u8 cmd_buf_C[0x30];
    DCInvalidateRange(aligned_buf_C, 0x20);
    memset(cmd_buf_C, 0, 0x30);
    SendCommand_v1(cmd_buf_C, aligned_buf_C, 0x20, 0x80000160, 0, 2);
    while (PollCompletion(cmd_buf_C) != 0) {}

    u32 unknown_status = bswap32(*(u32*)aligned_buf_C);
    if (!(unknown_status & 1)) {
        return 5;  // ★ 0x80000160 bit 0 未セット → エラー
    }

    // ctx_entry (type=0xA9) 設定 ★★★
    // g_test_ctx->field_0x54 = 0x89000000 (何らかの追加フィールド)
    r31->status = 3;                     // Dispatch state 3
    r31->field_04 |= 0x09;              // bit 0 + bit 3
    r31->field_08 = 0x89000000;          // ★ Execute2 レスポンス領域!
    r31->field_0C = 0x89000200;          // ★ Execute2 コマンド領域!

    // ============================================================
    // Phase 8: テストサブエントリ設定 (type 0xA9 用)
    // type==0xA9 なので test_table エントリ投入ブロックが実行される
    // ============================================================
    // 各ブロックで test_table に 'C','L' (0x43, 0x4C) または
    // 'W','R' (0x57, 0x52) タグ付きエントリを投入

    // ============================================================
    // Phase 9: g_test_ctx データ設定
    // ============================================================
    // bl SetupTestData_1 (0x800698A4) → type 0x29 用データ
    // bl SetupTestData_2 (0x80069078) → type 0xA9 用データ

    // ============================================================
    // Phase 10: 完了
    // ============================================================
    return 0;  // 成功 → テストポーリング開始

firmware_recovery:
    // FirmwareStatus2 bit 0 未セット
    if (data_ptr == NULL) goto error;
    u32 buf_ptr = *(u32*)(data_ptr + 0);
    if (buf_ptr == 0) goto error;
    u32 buf_val = *(u32*)(data_ptr + 4);
    if (buf_val == 0) goto error;

    // ファームウェア書き込み
    DCFlushRange(buf_val, buf_ptr);
    memset(cmd_buf, 0, 0x30);
    SendCommand_v2(cmd_buf, buf_val, buf_ptr, 0x84800000, 0, 2);
    while (PollCompletion(cmd_buf) != 0) {}

    // FirmwareStatus2 に 0x01000000 を書き込み
    u8 aligned_buf_D[0x20] __attribute__((aligned(32)));
    memset(aligned_buf_D, 0, 0x20);
    *(u32*)aligned_buf_D = 0x01000000;
    DCFlushRange(aligned_buf_D, 0x20);
    memset(cmd_buf, 0, 0x30);
    SendCommand_v1(cmd_buf, aligned_buf_D, 0x20, 0x80000140, 0, 2);
    while (PollCompletion(cmd_buf) != 0) {}

error:
    return 5;  // エラー
}
```

### Function 2: (0x800678D0 - 0x80067D7C+) テスト結果ヘルパー

setup_func外の独立関数。g_test_ctx からエントリを検索し、test_table サブエントリを設定する。

```c
// 0x800678D0: frame 0x170
int test_result_helper(u8 test_type) {
    test_type &= 0xFF;

    // g_test_ctx からマッチするエントリを検索
    u32 count = g_test_ctx->count;
    ctx_entry* found = NULL;
    for (u32 i = 0; i < count; i++) {
        if (g_test_ctx->entries[i].type == test_type) {
            found = &g_test_ctx->entries[i];
            break;
        }
    }
    if (!found) return 1;  // エントリなし

    u32 dma_addr = found->field_08;   // DMA読み取りアドレス
    u8 entry_type = found->type;      // テストタイプ
    u8 aligned_buf[0x20] __attribute__((aligned(32)));

    memset(aligned_buf, 0, 0x20);

    // 繰り返しブロック: global_state & test_table エントリ設定
    // type 0x29 と同じパターンで test_table にサブエントリを投入
    // ...

    // DMA addr >= 0x80000000 の場合の特殊処理
    if (dma_addr >= 0x80000000) {
        // Execute1 固有の初期化
        memset(cmd_buf, 0, 0x30);
        u8 seq = g_test_ctx->field_0xAA;
        SendCommand_v2(cmd_buf, aligned_buf, 0x20, dma_addr, seq, 2);
        while (PollCompletion(cmd_buf) != 0) {}
    }

    // ... さらにサブエントリ設定 ...

    // 末尾: found->status チェック
    if (found->status == 2) {
        // ... 追加処理 (ダンプ範囲外に続く)
    }

    return result;
}
```

### DMA コマンド一覧 (setup_func 0x29 + 0xA9)

| # | DMA Address | R/W | SendCommand | 目的 | 期待レスポンス |
|---|-------------|-----|-------------|------|------------|
| 1 | 0x80000140 | Read | v1 (0x80059214) | FirmwareStatus2 確認 | bswap32(word[0]) bit 0 = 1 |
| 2 | 0x84800000 | Read | v2 (0x80056BC8) | FirmwareAddress 読み取り (回復パスのみ) | ファームウェアデータ |
| 3 | 0x80000140 | Write | v1 | FirmwareStatus2 書き込み (回復パスのみ) | - |
| 4 | 0x80000120 | Read | v1 | FirmwareStatus1 読み取り | bswap32(word[0]) → g_test_ctx[0xA8] |
| 5 | 0x80000160 | Read | v1 | Unknown Status 読み取り (0xA9用) | bswap32(word[0]) bit 0 = 1 |
| 6 | 0x84000000 | Read | (BigTestFunc) | Execute1 レスポンス読み取り | response[2..3] != 0, 初回 0x0180 |
| 7 | 0x84000020 | R/W | (Dispatch) | Execute1 コマンド領域 | response[2..3] == 0 (idle) |
| 8 | 0x89000000 | Read | (BigTestFunc) | Execute2 レスポンス読み取り | response[2..3] != 0 |
| 9 | 0x89000200 | R/W | (Dispatch) | Execute2 コマンド領域 | response[2..3] == 0 (idle) |

### COMMUNICATION ERROR 解決に必要な対応

1. **FirmwareStatus2 (0x80000140)**: bswap32 後の bit 0 が 1 であること
   → MediaBoard 読み取りで word[0] の最下位ビットが 1 を返す

2. **FirmwareStatus1 (0x80000120)**: 読み取り成功すればよい (値は g_test_ctx[0xA8] に保存)

3. **Unknown Status (0x80000160)**: bswap32 後の bit 0 が 1 であること
   → 0x80000140 と同様

4. **Execute1 レスポンス (0x84000000)**: BigTestFunc が読む
   - response[2..3] が 0 → COMMUNICATION ERROR
   - response[2..3] = 0x0180 → 初回成功、field_04 bit 1 セット
   - response[2..3] & 0x80 → test_table にコピー (テスト進行)

**現在の Dolphin 実装 (更新済み)**:
- FirmwareStatus2 (0x80000140): `Write_U32_Swap(1)` → bswap32 後 bit 0 = 1 ✓
- FirmwareStatus1 (0x80000120): `Write_U32_Swap(0x01FA)` → bswap32 後 0x000001FA ✓
- 0x80000160: `Write_U32(0x00001E00)` → bswap32 後 0x001E0000, **bit 0 = 0 ✗**
- Execute1 ハンドラ: TestHardware 未対応 (PanicAlert)

## 全テストフロー再解析 ★★★ (最新)

### 発見: COMMUNICATION ERROR は想定した経路からは発生しない

NETWORK TEST のメイン実行パス:
```
setup_func(0x29) → InitTest(0,1) → PollTest loop → check_result(0x29) → 判定
```

#### 1. setup_func(0x29) の結果

Phase 1: FirmwareStatus2 (0x80000140) → bswap32(0x01000000) = 1, bit 0=1 → ✓ パス
Phase 3: FirmwareStatus1 (0x80000120) → bswap32(0xFA010000) = 0x01FA → 値保存のみ
Phase 4: ctx[0] 設定 (type=0x29, status=2, field_08=0x84000000)
Phase 7: 0x80000160 → **bswap32(0x00001E00) = 0x001E0000, bit 0=0 → ✗ return 5?**

**矛盾**: runtime dump で ctx[1] (type=0xA9) が存在 → setup_func は Phase 7 を通過したはず
**可能性**:
- a) 0x80000160 のチェックが bswap32 でない (plain lwz & 別ビットマスク)
- b) ctx[1] はブート時に別パスで設定済み
- c) Write_U32(0x00001E00) のバイトオーダーが逆 (Write_U32_Swap にすべき)

#### 2. InitTest(0, 1) の結果

fn_80066610 は `entries[i].field_08 & 1` をチェック:
- ctx[0].field_08 = 0x84000000 → & 1 = **0**
- ctx[1].field_08 = 0x89000000 → & 1 = **0**
→ GetEntry returns 0 → **InitTest returns 0** → DMA 未投入

#### 3. check_result(0x29) の結果

- 0xA9 エントリ検索 → ctx[1] (type=0xA9) 発見 → **return 0 (PASS)**
- count=2 >= 2 → バックアップ条件でも **PASS**

#### 結論

**このフローでは COMMUNICATION ERROR は発生しない！** test は PASS になるはず。

### BigTestFunc の呼び出し元問題

BigTestFunc (0x80065324) は response[2..3]==0 で return 3 (COMMUNICATION ERROR) を返すが、
上記テストフロー (0x8006A800+) からは**呼ばれない**。

BigTestFunc は別の実行パスから呼ばれている:
- TestManager_Run 内の別ディスパッチ?
- テスト結果確認コールバック?
- メインテストとは別の NETWORK SETTING サブテスト?

**未解決**: BigTestFunc の正確な呼び出し元を特定する必要がある。

### ctx[0].field_04 = 3 の意味

field_04 の bit 1 (0x02) がセット済み。BigTestFunc Phase 5 で:
```c
if (status == 0x0180) {
    entry->field_04 |= 0x2;  // "初期化済み" フラグ
}
```
→ BigTestFunc は過去に **response[2..3] = 0x0180 で成功している**！
→ 少なくとも1回は 0x84000000 の DMA Read で正しいレスポンスを得た。

### Dolphin コードとの照合

#### DMA Read from 0x84000000 のパス

1. ExecuteCommand → AMMBDICommand::Read
2. FirmwareStatus check: `(0x84000000 & 0x8FFF0000) == 0x80000000` → 0x84000000 ≠ → スキップ
3. 保存バッファ intercept: `offset == DIMMCommandVersion2(0x84000000) && length == 0x20` → **マッチ**
4. s_exec1_last_response から serve

しかしログに "Read Execute1 response (saved)" が出ない → DMA 自体が発生していない

#### Execute1 ハンドラの状態

Execute1 (0x84000040) には TestHardware ハンドラがない。
もし State2Handler が Execute1 DMA を発行すると **PanicAlert** で停止する。
→ Execute1 TestHardware の実装が必要だが、State2Handler が呼ばれるか自体が不明。

### 未解決の重大な疑問

1. **BigTestFunc はどこから呼ばれるか？** → GDB breakpoint または PPC disasm で特定
2. **0x80000160 のチェックは本当に `bswap32 & 1` か？** → PPC 命令の確認が必要
3. **ctx[0].field_04 bit 1 はいつセットされたか？** → BigTestFunc が成功した時点の特定
4. **Execute1 TestHardware は必要か？** → Dispatch が ctx[0] を処理するか次第

### 次のステップ

ExecuteCommand の NOTICE_LOG 変更済み (DEBUG→NOTICE)。新ログで以下を確認:
- 0x84000000 への DMA Read が発生するか
- 0x84000020 への DMA Read/Write が発生するか
- FirmwareStatus レジスタの読み取りタイミング

## UI ハンドラ解析結果 (Ghidra デコンパイル確認済み)

UI_PageDispatcher (0x800668fc) の全 case ハンドラは**表示専用**。
テスト判定ロジックは UI 層には存在しない。

- param_2==0: 初期化, param_2==1: 入力処理, param_2==2: 描画
- 全ハンドラが `*(param_1 + 0x274)` からテスト結果を**読み取り**、DrawText テーブルで表示
- テスト結果の**書き込み**は UI 外の PPC コード (TestManager 系) が行う

確認済み case:
- case 6 (0x8006aa84): トグル設定 UI
- case 10 (SystemInfo_UI): 日時、HW版 表示
- case 0x1b-0x1f: 各種統計表示
- CardError_Display: カードエラーコード表示
- CardDataTransfer_UI, CardDataCheck_UI: カード操作 UI

未確認 case: 0-5, 7-9, 11-0x1a, 0x20 (いずれも UI パターンに従うと推定)

## Ghidra リネーム済み関数一覧

Ghidra上で確認・リネームした関数の一覧。

### DI/DMA 低レベル層
| Address    | Ghidra Name              | Description |
|------------|--------------------------|-------------|
| 0x80285cb8 | DI_DMA_Read_LowLevel     | DI レジスタ (0xCC006014/18/1C) への書き込みで DMA Read を実行 |
| 0x80285df8 | DI_DMA_Write_LowLevel    | DI レジスタへの書き込みで DMA Write を実行 |
| 0x80285d88 | DI_DMA_Read_Wrapper      | DI_DMA_Read_LowLevel のラッパー、4箇所から呼ばれる |
| 0x80296ce8 | DVD_DI_CommandDispatcher  | DVD/DI コマンドの大きな switch 文ディスパッチャ (cases 1-0xf) |
| 0x80295b24 | DVD_ChangeDisk            | DVD ディスクチェンジ関連 |

### MediaBoard コマンド層
| Address    | Ghidra Name              | Description |
|------------|--------------------------|-------------|
| 0x8005982c | MediaBoard_DMA_Submit    | コマンドをリンクリスト (0x800B3618) にエンキュー。非同期DMA処理 |
| 0x80059644 | MediaBoard_PollCompletion| cmd->field_0xC をチェック。3=処理中→return 1, 0=完了 |
| 0x800652d4 | MediaBoard_SendAndCheck  | Execute2 送信後レスポンスをチェック。status halfword で分岐 |

### UI ハンドラ層 (全て表示専用、テスト判定ロジックは含まない)
| Address    | Ghidra Name              | UI_PageDispatcher case |
|------------|--------------------------|------------------------|
| 0x800668fc | UI_PageDispatcher        | switch on *(param_1+0x10), cases 0-0x20 |
| 0x80066b74 | SystemInfo_UI            | case 10: 日時、HW版 "MK2100_x_NA_MPR0_A36" 表示 |
| 0x80066350 | CardError_Display        | カードエラー表示 (0x65=LINK, 0x66=WRITE等) |
| 0x80067494 | Stats_PacketPercent_UI   | case 0x1f: パケット割合統計 |
| 0x80067f74 | Stats_PacketCounts_UI    | case 0x1e: パケットカウント (%06d形式) |
| 0x8006830c | Stats_PacketStats_UI     | case 0x1d: パケット統計 |
| 0x80068d00 | Stats_Percentages_UI     | case 0x1c: パーセンテージ統計 |
| 0x80069720 | Stats_Display_UI         | case 0x1b: 統計表示 |
| 0x8006d308 | CardDataTransfer_UI      | カードデータ転送UI (STARTING/NOW LOADING等) |
| 0x8006db90 | CardDataCheck_UI         | カードデータチェックUI (NOW CHECKING等) |

### その他
| Address    | Ghidra Name              | Description |
|------------|--------------------------|-------------|
| 0x8008fa2c | CardSaveData_Parser      | セーブデータ解析 (COUNTRY, CHARA, TITLE等) |

## DMA PC/LR パターン分析 (ログ新発見)

ログのPC/LRから、Execute1/Execute2 の発火元が2系統あることが判明:

### ブート時 (35:54 台)
- Execute1 write: PC=80285e4c, LR=80285e30 → DI_DMA_Write_LowLevel 内部
- Execute2 read: PC=80285d28, LR=80285d0c → DI_DMA_Read_LowLevel 内部
- DMA addr: 0x006a2f80 (高アドレス)

### テスト時 (36:38 以降)
- Execute1 write: PC=800560e0, LR=800560c4 → FUN_800560c0 (C++デストラクタ風の小関数)
- Execute2 read: PC=80055fbc, LR=80055fa0 → FUN_80055fa0 (同様のデストラクタ風)
- DMA addr: 0x000d01e0 (低アドレス)

**注意**: テスト時のPC/LRは非同期サンプリングのため、実際のDI書き込み命令の位置ではない。
PPC がDIレジスタ書き込み後に別の処理に移っている状態でキャプチャされた可能性がある。

### Execute2 コマンドタイムライン (テスト実行時)
```
36:38:441  Execute2 0x0001 (Init)
36:38:450  Execute2 0x0101 (Init)
36:38:458  Execute2 0x0103 (Init)
37:12:927  Execute2 0x0104 (Unknown_104) ← 0x0103から34秒のギャップ！
37:20:021  Execute2 0x0301 (TestHardware type=4)
37:20:027  Execute2 0x8000 (Cleanup)
37:20:652  Execute2 0x0104 (Unknown_104 2回目)
```

0x0103 → 0x0104 間の34秒ギャップは NETWORK SETTING テストのタイムアウト待ちと推定。
0x0104 は `s_netconfig_read_pending = true` を設定し `s_media_buffer_32[1] = 0` を返す。
PPC側がこの応答を「ネットワーク未設定/未接続」と解釈して COMMUNICATION ERROR を出している可能性が高い。

## WriteNetworkConfig (0x80068554) - Ghidra 解析

ReadNetworkConfig (0x800686a4) の書き込み対応関数。NETWORK SETTING メニューでの IP アドレス設定等に使用。

### コマンド

- sub_cmd=0x04, cmd_class=0x02 → Execute2 コマンド **0x0204**
- ReadNetworkConfig は cmd_class=0x01 (0x0104)、WriteNetworkConfig は cmd_class=0x02 (0x0204)

### 処理フロー

1. `GetTestContext()` → `pTVar2->field_0x20` (DMA アドレス) が非ゼロか確認
   - このアドレスは先行する ReadNetworkConfig が設定する
2. `FindCtxEntry(8, ...)` → `AllocTestTableSlot()` → `GetTestTableCommandSlot()`
3. `memcpy32(localBuf, param_1, 0x80)` — 128バイトを PPC メモリからローカルバッファにコピー
4. 先頭 u32 をバイトスワップ (PPC BE → MediaBoard LE)
5. `DMA_WriteSync(localBuf, pTVar2->field_0x20, 0x80)` — 128バイトを MediaBoard に DMA 書き込み
6. コマンドヘッダ設定 (entry_id, 0x00, 0x04, 0x02) → `SubmitCommand()`
7. 応答ポーリング → CleanupTestTableEntry

### DMA_WriteSync (0x8006620c)

DMA_ReadSync (0x8006637c) の書き込み対応関数。

```
void DMA_WriteSync(src_buf, dest_addr, length) {
    if (dest_addr < 0x80000000) DMA_SetupChannel(...)
    DCache_Invalidate(src_buf, length)
    DMA_Write(ctx, src_buf, length, dest_addr, 0, 2)
    while (DMA_PollCompletion(ctx) != 0) {}
}
```

### 呼び出し元 (7箇所)

- FUN_80016bf8 (3回: 0x80016c24, 0x80016c58, 0x80016c6c)
- FUN_800170cc (1回: 0x80017540)
- FUN_800176fc (2回: 0x80017910, 0x800179a4)
- FUN_80018974 (1回: 0x800189d4)

これらは NETWORK SETTING / SET IP ADDRESS メニューハンドラと推定。

### Dolphin 側の対応

WriteNetworkConfig は Execute2 コマンド 0x0204 を発行する。
Dolphin の AMMediaboard.cpp で対応が必要な場合:
- Execute2 で cmd 0x0204 を処理し、後続の DMA Write (0x80バイト) を trinetcfg.bin に保存

## ReadNetworkConfig (0x800686a4) - 詳細解析

### 関数概要

MediaBoard コマンド 0x0104 (GetNetworkConfig) を発行し、128バイトのネットワーク設定を DMA で読み取る。

```c
undefined4 ReadNetworkConfig(uint *param_1)
```

- `param_1`: 128バイト (32 u32) の出力バッファ
- 戻り値: 0=成功, 0xFFFFFFFE=スロット割当失敗, 0xFFFFFFFC=NULL引数, 0xFFFFFFFB=ポーリング失敗, 0xFFFFFFFD=0x8000エラー応答

### 処理フロー

1. FindCtxEntry(8, ...) でコンテキストを取得
2. AllocTestTableSlot でスロット確保
3. コマンドヘッダ `[slot, 0x00, 0x04, 0x01]` = コマンド 0x0104 を構築して SubmitCommand
4. PollCommandResult でポーリング待ち
5. DMA[2..3] の bswap16 が 0x8000 なら失敗 → return -3
6. 成功時: DMA[4..7] をアドレスオフセットとして GetDMABaseAddress(1) + offset に DMA_ReadSync(128バイト)
7. **param_1[0] のみ bswap32** (LE→BE変換)。残りの124バイトはそのまま
8. CleanupTestTableEntry で後処理

### config[0] (最初のバイト) のセマンティクス

`config[0] & 0xFF` がネットワーク設定モードを表す:

| 値 | 意味 | メニューテーブル | メニュー項目数 |
|----|------|-----------------|---------------|
| 0 | 未設定 | **NULL (0x00000000)** | **クラッシュ** |
| 1 | DHCP? | 0x8006dcac | 10 (全項目 + NETWORK SETTING) |
| 2 | Static IP? | 0x8006dcac | 10 (同上) |
| 3 | 縮小メニュー | 0x800d2790 | 6 (基本項目のみ) |
| 4 | IP設定のみ | 0x800d2798 | 5 (IP関連 + 接続テスト) |
| >=5 | (→0にクランプ) | **NULL** | **クラッシュ** |

### メニューテーブル詳細

テーブルは2つの配列 (offline/online) が 0x8006F964 と 0x8006F978 に存在するが、内容は同一。

- `0x8006dcac`: `[0,1,2,3,4,5,10,6,7,8, 0x0B]` (0x0B=終端)
- `0x800d2790`: `[0,1,2,3,4,5, 0x0B]`
- `0x800d2798`: `[6,7,8,4,5, 0x0B]`

### config[0] == 0 のクラッシュメカニズム (FUN_8001875c @ 0x8001875c)

FUN_8001875c はネットワーク設定メニューの初期化関数(コンストラクタ, vtable=PTR_PTR_8006fbd0)。

1. `ReadNetworkConfig(param_1 + 0x14)` で config を読み取り
2. `uVar2 = param_1[0x14] & 0xff` → config[0] を抽出
3. `if (4 < uVar2) uVar2 = 0` → 5以上は0にクランプ
4. `param_1[0x11] = table[uVar2]` → **値0のときNULLポインタ**
5. `for (pcVar3 = (char *)param_1[0x11]; *pcVar3 != '\v'; ...)` → **NULL参照でクラッシュ**

### 呼び出し元一覧 (6箇所)

| 呼び出し元 | アドレス | 呼び出し箇所 | config用途 |
|-----------|---------|-------------|-----------|
| FUN_80016fdc | 0x80016fdc | 0x800170a4 | コンストラクタ、config at +0x3c、戻り値のみ保存 |
| FUN_800170cc | 0x800170cc | 0x8001754c | Write後のRead-back (検証用) |
| FUN_800176fc | 0x800176fc | 0x8001791c | IP設定エディタ、Write後のRead-back |
| FUN_80017954 | 0x80017954 | 0x800179b0 | config[1]インクリメント→Write→Read-back |
| **FUN_8001875c** | 0x8001875c | 0x800187fc | **メニュー初期化、config[0]でテーブル選択→クラッシュ** |
| FUN_80018f44 | 0x80018f44 | 0x80019030 | メインメニューコンストラクタ、戻り値のみ保存 |

### FUN_80017954 の config 操作

```c
*(int *)(param_1 + 0x50) += 0x10000;           // config[1] (2nd byte) をインクリメント
*(uint *)(param_1 + 0x50) &= 0x100ff;          // config[0]保持、config[1]は0/1のみ、残りクリア
WriteNetworkConfig(param_1 + 0x50);
ReadNetworkConfig((uint *)(param_1 + 0x50));    // Write-back verification
```

bswap32後のメモリレイアウト (PPC u32): `config[3]<<24 | config[2]<<16 | config[1]<<8 | config[0]`
- `& 0xff` = config[0] (ネットワークモード)
- `+ 0x10000` = config[2] をインクリメント (実際にはraw byte[2])
- `& 0x100ff` = config[0]保持 + config[2]を0/1にクランプ

### Dolphin 側の対応 (AMMediaboard.cpp)

```cpp
// Status byte 0 causes a crash. Default to 2 (configured).
if (config[0] == 0)
    config[0] = 2;
```

trinetcfg.bin が空(全0)の場合、config[0]=0 → FUN_8001875c で NULL テーブル参照 → クラッシュ。
Dolphin はデフォルト値 2 (Static IP 設定済み) を設定してクラッシュを回避。

## ステータス

- [x] TestHardware ハンドラを Execute2 に追加
- [x] 正しい文字列エンコーディング ("TEST OK")
- [x] コールチェーン解析完了
- [x] テスト状態マシン解析完了
- [x] TestManager_Dispatch 完全逆アセンブリ (r28 設定ロジック)
- [x] State3Handler 完全逆アセンブリ (r28 非保存を確認)
- [x] InitTest 完全逆アセンブリ
- [x] PollTest 完全逆アセンブリ
- [x] check_result 完全逆アセンブリ
- [x] 実PPCコールスタック取得
- [x] g_test_ctx 構造体レイアウト特定
- [x] メモリアドレス修正 (0x800CCDA0, 0x800CE5A0)
- [x] TestManager_Dispatch ポストハンドラ完全逆アセンブリ (0x80064F5C-0x800652B0)
- [x] setup_func 部分逆アセンブリ (0x800666F4、テストタイプ別ディスパッチ)
- [x] BigTestFunc 完全逆アセンブリ (0x80065324、レスポンス → test_table コピー)
- [x] DMAパターン分析 (ブート7フェーズ vs Dispatch5フェーズ)
- [x] ヘルパー関数アドレス特定 (DCFlushRange, DCInvalidateRange 等)
- [x] バイトオーダー分析 (stwbrx による LE DMA、status 0x0183 → 0x0180 修正)
- [x] TestHardware → 成功 (ゲームが次のコマンドに進んだ)
- [x] Unknown_8000 コマンドの解析 (クリーンアップコマンド、generic completion 使用)
- [x] s_media_buffer 二重領域アーキテクチャ解明 (+0x000 レスポンス vs +0x200 コマンド)
- [x] Unknown_8000 ハング原因特定 & 修正 (byte[3]=0 → DMA SM 完了不能)
- [x] setup_func type 0x29 完全逆アセンブリ (0x80066E08-0x800678CC、DMAコマンド一覧・擬似コード付き)
- [x] WriteNetworkConfig (0x80068554) 特定 — ReadNetworkConfig の書き込み対応、コマンド 0x0204
- [x] DMA_WriteSync (0x8006620c) 特定 — DMA_ReadSync の書き込み対応
- [ ] COMMUNICATION ERROR 修正 (Execute1 レスポンス領域に適切なステータスを返す)
- [ ] NETWORK TEST パス確認 (ビルド & テスト必要)
