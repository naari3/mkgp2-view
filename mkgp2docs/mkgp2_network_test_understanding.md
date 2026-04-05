# MKGP2 NETWORK TEST - 現状の理解

## 概要

Mario Kart Arcade GP 2 (GNLJ82) のサービスメニュー NETWORK TEST を通過させるための解析。
NETWORK TEST のコードは **segaboot.gcm** (Segaシステムメニュー) に含まれる。

**現状: NETWORK TEST 通過に成功** (2フェーズレスポンス実装済み)

---

## アーキテクチャ: 3スレッドモデル

segaboot.gcm は3つのスレッドで DMA コマンド処理を行う:

| スレッド | 関数 | 役割 |
|----------|------|------|
| **Main thread** | SendTestHardwareCmd (0x80068284) | コマンド発行・結果ポーリング |
| **DMA Poll thread** | DMA_PollLoop (0x8006ade0) | EXI割り込み待機 → DMA応答処理 |
| **Dispatch thread** | TestDispatchLoop | コマンドディスパッチ・コールバック呼び出し |

---

## データ構造

### test_table (DAT_800ccda0)
- 64エントリ × 0x60バイト
- 各エントリの構成:

| オフセット | サイズ | 名称 | 説明 |
|-----------|--------|------|------|
| 0x00-0x1F | 0x20 | DMA slot | DMA応答データ格納先 |
| 0x20-0x3F | 0x20 | Result slot | コマンド/結果管理 |
| 0x40-0x5F | 0x20 | Ctx association | テストコンテキスト関連付け |

**DMA slot のバイトレイアウト (PPC メモリ上、ビッグエンディアン):**
- `[0]` = entry_id (スロット識別子)
- `[1]` = 0x00
- `[2]` = sub_cmd (コマンドのサブコマンド番号)
- `[3]` = cmd_class | 0x80 (完了フラグ付きコマンドクラス)
- `[4..7]` = パラメータ1 (LE格納、PPC側で bswap32 して使用)
- `[8..11]` = パラメータ2 (同上)

**Result slot の重要フィールド:**
- `[0x20]` = entry_id
- `[0x22..0x23]` = ステータス/コマンドワード

### g_test_ctx (0x800CE5A0)
- `[0]` = テスト数 (count)
- `[4 + i*0x10]` = ctx_entry配列:
  - `[0]` type, `[1]` event_type, `[4]` flags (bit1=init), `[8]` field_08 (DMA read addr), `[C]` field_0C (DMA write addr)

### TestContext (0x800CE5C4)
- `[0x24]` (abs: 0x800CE5E8) = testStatus: 0=未開始, 1=チェック中, 2=成功(G), 3=失敗(B)
- `[0x28]` (abs: 0x800CE5EC) = checkProgress: 0-100 (%)

### ProcessDMAResponse ディスパッチテーブル

ProcessDMAResponse は応答の sub_cmd/cmd_class に基づいてハンドラを呼び分ける。
**ただしディスパッチは `(uStack_5e & 0x80) == 0` パスでのみ実行される。**

| cmd_id | count | handler_table |
|--------|-------|---------------|
| 0x0000 | 12    | 0x8006eaf8    |
| 0x0300 | 4     | 0x8006eb28    |

**0x0300 の handler_table (0x8006eb28):**

| sub_cmd | ハンドラ | 説明 |
|---------|---------|------|
| 0       | DefaultResponseHandler | デフォルト |
| 1       | DefaultResponseHandler | デフォルト |
| **2**   | **TestHardwareResponseHandler** | testStatus/checkProgress 設定 |
| 3       | DefaultResponseHandler | デフォルト |

---

## ProcessDMAResponse の2つのパス (解決済み)

```
ProcessDMAResponse(event_type)
  │
  ├─ ctx_entry を検索 (event_type == ctx_entry.event_type)
  ├─ DMA_Read from field_08 (0x89000000) → 応答データ読み出し
  ├─ uStack_5e = response[2..3] (PPC BE u16)
  │
  ├─ Init flag チェック: (flags & 2) != 0 必須
  │    → GetDIMMSize (0x0001) 応答の 0x0180 で設定される
  │
  ├─ ★ (uStack_5e & 0x80) != 0:  【確認応答パス】
  │    ├─ entry_id で既存エントリを検索 (DMA slot の slotId == response[0])
  │    ├─ 見つかれば DMA slot に応答データをコピー (memcpy32)
  │    └─ ハンドラディスパッチなし
  │
  └─ ★ (uStack_5e & 0x80) == 0:  【結果通知パス】
       ├─ 空きスロットを探して新エントリを割当
       ├─ 応答データを DMA slot にコピー
       ├─ entryId = ctx_entry.type (応答の byte[0] ではない)
       ├─ cmd_class = (uStack_5e & 0x7f) << 8
       ├─ sub_cmd = uStack_5e >> 8
       └─ ディスパッチテーブルでハンドラ呼び出し
            sub_cmd=0x02, cmd_class=0x0300 → TestHardwareResponseHandler
```

**これが2フェーズプロトコルの根拠:**
- Phase 1 (0x80あり): データ格納のみ。PollCommandResult/SendTestHardwareCmd が結果を読む。
- Phase 2 (0x80なし): ハンドラ呼び出し。TestHardwareResponseHandler が testStatus を設定。

---

## NETWORK TEST 実行フロー (実装済み・動作確認済み)

### Phase 0: 起動時の初期化

```
Boot → GetDIMMSize (0x0001) via Execute2
  → Dolphin: s_media_buffer_32[1] = 1, s_media_buffer[3] |= 0x80
  → 応答 byte[2..3] = 0x0180
  → ProcessDMAResponse: uStack_5e == 0x0180 → flags |= 2 (init flag)
```

### Phase 1 (Phase 1 応答): 確認応答

```
PPC: SendTestHardwareCmd(test_type=4, string_ptr)
  → AllocTestTableSlot → SubmitCommand → TriggerExecute2
  → DI Read 0x88000000 → Dolphin Execute2 ハンドラ

Dolphin Execute2:
  ├─ "TEST OK\0" を string_ptr に書き込み
  ├─ s_media_buffer_32[1] = test_type (エコー)  ← SendTestHardwareCmd のバリデーション用
  ├─ CoreTiming::ScheduleEvent(50000, Phase2CB)  ← Phase 2 を遅延スケジュール
  ├─ s_media_buffer[3] |= 0x80                   ← 完了フラグ
  ├─ → s_exec2_last_response に保存
  └─ GenerateInterrupt(0x10)                      ← ★ 1回目の割り込み

PPC ProcessDMAResponse:
  ├─ DI Read 0x89000000 → Phase 1 応答取得
  ├─ uStack_5e = 0x0183, (0x0183 & 0x80) != 0 → 既存エントリ更新
  ├─ DMA slot にコピー: [0a000183 04000000 ...]
  └─ return

PPC Main thread:
  ├─ PollCommandResult → DMA slot[2..3] != 0 → 発見
  ├─ FindTestTableByEntryID → bswap16(DMA[2..3]) != 0x8000 → 成功
  ├─ *param_3 = bswap32(DMA[4..7]) = test_type(4) → バリデーション通過
  └─ SendTestHardwareCmd return 0 (成功)
```

### Phase 2 (Phase 2 応答): テスト結果

```
CoreTiming コールバック (TestHwPhase2Callback):
  ├─ s_exec2_last_response を Phase 2 データで上書き:
  │    [0] = 0x03020000  → byte[2]=0x02(sub_cmd), byte[3]=0x03(cmd_class), 0x80なし
  │    [1] = 2           → testStatus = GOOD (bswap32 後)
  │    [2] = 100         → checkProgress = 100% (bswap32 後)
  └─ GenerateInterrupt(0x10)                      ← ★ 2回目の割り込み

PPC ProcessDMAResponse:
  ├─ DI Read 0x89000000 → Phase 2 応答取得
  ├─ uStack_5e = 0x0203, (0x0203 & 0x80) == 0 → 新エントリパス
  ├─ 空きスロット割当、応答コピー
  ├─ cmd_class = (0x03 & 0x7f) << 8 = 0x0300
  ├─ sub_cmd = 0x0203 >> 8 = 0x02
  └─ ディスパッチ → TestHardwareResponseHandler (0x800691a0)

TestHardwareResponseHandler:
  ├─ testStatus = bswap32(DMA[4..7]) = bswap32(0x02000000) = 2 (GOOD)
  ├─ checkProgress = bswap32(DMA[8..11]) = bswap32(0x64000000) = 100
  ├─ DMA slot → Result slot コピー (result[2..3] に完了フラグ付与)
  └─ SignalCommandComplete → TestManager_Dispatch
       → result[2] & 0x80 == 0 → エントリクリア (TriggerExecute2 は呼ばれない)
```

### Phase 3: テスト結果表示

```
FUN_80015154 (テスト表示関数):
  ├─ testStatus & 3 = 2 → "G" (GOOD)
  ├─ checkProgress = 100 → "CHECKING 100%"
  └─ ステータス >= 2 → ボタン入力受付 → 次のテストへ
```

---

## SendTestHardwareCmd のバリデーション (解決済み)

```c
// SendTestHardwareCmd (0x80068284) の結果チェック:
uVar8 = FindTestTableByEntryID(cVar6);      // DMA slot を取得
iVar3 = (int)(uVar8 >> 0x20);
uVar1 = *(ushort *)(iVar3 + 2);             // DMA[2..3]
if (bswap16(uVar1) == 0x8000) {
    return -3;  // MEDIA COMMAND ERROR
}
// 成功: *param_3 = bswap32(DMA[4..7])      // 呼び出し元が test_type と比較
```

- DMA[2..3] = 0x0183 → bswap16 = 0x8301 ≠ 0x8000 → 成功 ✓
- DMA[4..7] = test_type(4) のエコー → 呼び出し元バリデーション通過 ✓
- buf[1] を test_type 以外にすると MEDIA COMMAND ERROR になる

---

## バイトレイアウトの理解 (解決済み)

### s_media_buffer ↔ PPC メモリのマッピング

`s_media_buffer` は x86 メモリ上のバイト配列。`CopyToEmu` で PPC メモリにコピーされる際、
**バイト順序はそのまま保存される**。PPC はビッグエンディアンで読む。

```
s_media_buffer (x86):     [0x0a] [0x00] [0x01] [0x83]
                            ↓ CopyToEmu (バイトそのまま)
PPC メモリ:               [0x0a] [0x00] [0x01] [0x83]
PPC lwz (BE u32):         0x0a000183
PPC lhz at +2 (BE u16):  0x0183  ← これが uStack_5e
```

### s_media_buffer_32 との関係

`s_media_buffer_32[0]` = x86 LE u32 = 0x8301000a (bytes: 0a 00 01 83)
- `s_media_buffer[0]` = 0x0a (entry_id)
- `s_media_buffer[2]` = 0x01 (sub_cmd)
- `s_media_buffer[3]` = 0x83 (cmd_class | 0x80)

`s_media_buffer_32[1]` = x86 LE u32 (例: test_type=4 → 0x00000004)
- PPC bytes: [0x04] [0x00] [0x00] [0x00]
- PPC lwz: 0x04000000
- bswap32(0x04000000) = 4

---

## 現在の変更点 (AMMediaboard.cpp)

### 変更1: TestHardware 2フェーズレスポンス (Execute1/Execute2 両対応)
- Phase 1: test_type エコー + 0x80 完了フラグ → SendTestHardwareCmd 成功
- Phase 2: sub_cmd=0x02, testStatus=2, checkProgress=100 → TestHardwareResponseHandler で設定
- トリガー: CoreTiming::ScheduleEvent (50000サイクル ≈ 0.1ms 遅延)
  - userdata=0: Execute1 → s_exec1_last_response + interrupt 0x04
  - userdata=1: Execute2 → s_exec2_last_response + interrupt 0x10
- コールバック: TestHwPhase2Callback (CoreTiming イベント "AMMediaboardTestHwPhase2")

### 変更2: s_exec1_last_response / s_exec2_last_response
- Execute1/Execute2 の応答を個別バッファに保存
- 相互上書き防止

### 変更3: GetNetworkConfig (0x0104) + NetConfig Read インターセプト
- GetNetworkConfig (0x0104) 後の 128バイト DMA Read を trinetcfg.bin から提供

### 変更4: Execute1 inner switch に TestHardware 追加
- Execute1 コマンドバッファ: [9]=test_type, [10]=string_ptr (Execute2 とは異なるインデックス)
- MEDIA BOARD TEST (type=3) が Execute1 経由で来た場合に対応

### 変更5: 0x80xx Cleanup コマンドの汎用処理
- TestManager_Dispatch_Inner は result slot (cmd_class | 0x80) を field_0C に書き戻す
- これが Execute1/Execute2 の cleanup コマンド (例: 0x8302) として発行される
- Execute1/Execute2 の default case で `ammb_command & 0x8000` を検出し acknowledge

### ~~変更6: 診断ログ~~ (削除済み)
- CoreTiming 移行時に診断ダンプログ (g_test_ctx, test_table, POST-PDR) を削除

---

## Cleanup コマンド (0x80xx) の仕組み

TestHardwareResponseHandler が DMA slot → result slot コピー時に cmd_class に 0x80 を OR する。
TestManager_Dispatch_Inner は **result slot** (DMA slot ではない) を field_0C に DMA_Write し、
コールバック経由で Execute1/Execute2 をトリガーする。

例: Phase 2 応答 sub_cmd=0x02, cmd_class=0x03 → result slot の cmd_class = 0x83
→ Execute コマンドとして 0x8302 が発行される
→ Dolphin は acknowledge (0x80 フラグ + interrupt) するだけでよい
→ PPC 側は `result[2..3] & 0x80 != 0` を検出してエントリをクリア

DmaCommandSlot 構造体:
- offset 0x00: DMA slot (entryId = channelByte)
- offset 0x20: Result slot (slotId)
TestManager_Dispatch_Inner の pbVar6 = &.slotId → Result slot を指す

---

## 今後の課題

1. ~~**診断ログの削除 + ハードコードアドレスの除去**~~: 完了
2. ~~**CoreTiming ベースの Phase 2**~~: 完了
3. ~~**enum リネーム**~~: Unknown_104 → GetNetworkConfig, Unknown_8000 削除
4. ~~**MMU.cpp 診断コード削除**~~: 完了
5. **SaveState 互換性**: DoState のバージョン番号更新 (STATE_VERSION bump、マージ時に調整)

---

## GetNetworkConfig (0x0104) の PPC 側処理

FUN_800686a4 (segaboot.gcm) が 0x0104 コマンドを発行する関数:

```
pcVar3[2] = 0x04;  // sub_cmd
pcVar3[3] = 0x01;  // cmd_class
SubmitCommand(slotId);
```

レスポンス処理:
```c
iVar6 = FUN_800665e0(1);              // DAT_800ce5f0[1] を取得 (ベースアドレス)
uVar8 = *(uint *)(DMA_slot + 4);      // s_media_buffer[4] の値 (s_media_buffer_32[1])
address = bswap32(uVar8) + iVar6;     // オフセットとして加算
FUN_8006637c(address, buf, 0x80);      // DMA Read 128バイト
memcpy32(param_1, buf, 0x80);          // 出力にコピー
*param_1 = bswap32(*param_1);          // 先頭 u32 をバイトスワップ
```

- `s_media_buffer[4] = 1` (元の実装) → bswap32 後のオフセット = 1
- Dolphin 側では s_netconfig_read_pending で 128バイト DMA Read をインターセプトして
  trinetcfg.bin を返すため、オフセット値自体は実質無視される
- ただし元の値を変える理由はないため `s_media_buffer[4] = 1` を維持

---

## Ghidra でリネーム済みの関数 (segaboot.gcm)

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x80065f10 | AllocTestTableSlot | test_table エントリ割り当て |
| 0x80066114 | SubmitCommand | コマンド提出 |
| 0x800660c0 | SignalCommandComplete | TestManager_Dispatch を同期呼び出し |
| 0x80065a60 | PollCommandResult | result slot をポーリング |
| 0x80065cf8 | FindTestTableByEntryID | entry_id で test_table 検索 (DMA slot 返却) |
| 0x80068284 | SendTestHardwareCmd | TestHardware の完全プロトコル |
| 0x80065324 | ProcessDMAResponse | DMA応答処理 (2パス: 確認応答/結果通知) |
| 0x80064e98 | TestManager_Dispatch_Inner | コマンドディスパッチ + コールバック |
| 0x800680C0 | TriggerExecute2 | DMA_Read(0x88000000) で Execute2 トリガー |
| 0x80065be8 | GetTestTableResultSlot | test_table の result slot 取得 |
| 0x80065cd0 | GetTestTableDMASlot | test_table の DMA slot 取得 |
| 0x80065b28 | SetTestTableCtxType | ctx_entry type を test_table に設定 |
| 0x80065c14 | GetTestTableCommandSlot | test_table の command slot 取得 |
| 0x80069840 | DefaultResponseHandler | デフォルト応答ハンドラ (0x0080 設定) |
| 0x800691a0 | TestHardwareResponseHandler | testStatus/checkProgress 設定 |
| 0x8006ade0 | DMA_PollLoop | DMAイベント待機ループ |
| 0x80065918 | WaitForDMAEvent | EXIステータスビット監視 |
| 0x80005374 | memcpy32 | メモリコピー |
| 0x8000528c | memset32 | メモリゼロ埋め |
| 0x80053f8c | Mutex_Lock | ミューテックスロック |
| 0x80053ec8 | Mutex_Unlock | ミューテックスアンロック |
| 0x8004fe50 | DCache_Flush | データキャッシュフラッシュ |
| 0x8004fe7c | DCache_Invalidate | データキャッシュ無効化 |
| 0x80056d08 | DMA_SetupChannel | DMAチャネル設定 |
| 0x80015904 | TestBoard_Init | DIMM BOARD テスト (type=3, Execute1) |
| 0x80016a24 | TestNetworkBoard | NETWORK BOARD テスト (type=4, Execute2) |
| 0x80065db4 | CleanupTestTableEntry | test_table エントリクリーンアップ |
| 0x80066540 | GetTestContext | TestContext ポインタ取得 |
| 0x800686a4 | ReadNetworkConfig | GetNetworkConfig (0x0104) 発行 + 128バイト DMA Read |
| 0x800665e0 | GetDMABaseAddress | DAT_800ce5f0[param] 取得 (境界チェック付き、0-1) |
| 0x8006637c | DMA_ReadSync | 同期 DMA Read (チャネル設定 → キャッシュフラッシュ → Read → PollCompletion) |
| 0x8000eb54 | CreateTestInstance | テストファクトリ (switch/case でテスト種別ごとにオブジェクト生成) |
| 0x8004f924 | HeapAlloc | ヒープアロケータ (フリーリスト走査、32バイトアライメント) |
| 0x80066610 | FindCtxEntry | ctx_entry 検索 (event_type マッチ) |
| 0x80059214 | DMA_Read | DMA読み出し (低レベル) |
| 0x80059664 | DMA_PollCompletion | DMA完了ポーリング |
| 0x8006620c | DMA_WriteSync | 同期 DMA Write (キャッシュ無効化 → Write → PollCompletion) |
| 0x80068554 | WriteNetworkConfig | ネットワーク設定書き込み (0x0204) + 128バイト DMA Write |
| 0x8001875c | NetSettingsMenuInit | ネットワーク設定メニュー初期化 (config[0]でメニューテーブル選択、0=クラッシュ) |
| 0x80018f44 | NetTestMenuInit | ネットワークテストメインメニュー初期化 |
| 0x80016fdc | NetNameEditorInit | ネットワーク名エディタ初期化 |
| 0x800170cc | NetNameEditorUpdate | ネットワーク名エディタ更新 (Write後Read-back) |
| 0x800176fc | NetIPEditorUpdate | IP設定エディタ更新 (Write後Read-back) |
| 0x80017954 | NetModeCycler | ネットワークモード切替 (config[1]インクリメント→Write→Read-back) |
