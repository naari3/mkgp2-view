# MKGP2 main.dol 解析メモ

バイナリ: Mario Kart Arcade GP 2 (GNLJ82) main.dol (PPC / Triforce)
ビルド日時: Feb 6 2007 20:29:25

## リネーム済み関数一覧

### テキストレンダリング
| Address | Name | 説明 |
|---|---|---|
| 0x801db548 | DrawText | テキストエントリをレンダラーオブジェクトのキューに追加 (479 xrefs) |
| 0x801db278 | TextRenderer_Flush | キュー内の全テキストを GX 経由で描画、カウンタリセット (5 xrefs) |
| 0x801daf38 | TextRenderer_DrawEntry | 個別テキストエントリの描画 (Flush内部から呼ばれる) |

### デバッグ/プロファイラ (非機能)
| Address | Name | 説明 |
|---|---|---|
| 0x8002fe34 | OverlayText | vsprintf で DAT_805960f0 に書くだけ。レンダリングなし (127 xrefs) |
| 0x8002ceec | DebugPrintf | OverlayText と同様、表示バックエンド未実装 |
| 0x8002ce9c | DebugOutput_Stub | 空関数 |
| 0x8002be18 | DebugOverlay_Dispatch | OverlayText 経由のため画面出力なし |
| 0x8002fd4c | DebugTextOutput | OverlayText 系 |
| 0x8002ffb8 | IsDisplayModeEnabled | OverlayText/DebugPrintf のゲート条件 |
| 0x80039c40 | Profiler_DrawBar | OverlayText でプロファイラエントリ表示 (dead code) |
| 0x80039cf4 | Profiler_RecordFrame | フレームタイミング記録 + OverlayText |
| 0x8002ff10 | Profiler_ResetWriteIndex | DAT_805940ec = 0 するだけ |

### ゲームループ
| Address | Name | 説明 |
|---|---|---|
| 0x8002dd58 | MainGameLoop | メイン初期化 + NOTICE画面ループ |
| 0x8002c5e8 | GameFrameLoop | ゲーム中のフレームループ |

### ブート/PCBチェック
| Address | Name | 説明 |
|---|---|---|
| 0x80084df8 | BootPCBCheck_Init | テキストレンダラー生成 + PCBチェック開始 |
| 0x80084a84 | BootPCBCheck_Display | DrawText で PCB ID 表示 |
| 0x80084bf4 | BootPCBCheck_Poll | PCBチェック状態ポーリング |
| 0x80084d98 | BootPCBCheck_Cleanup | テキストレンダラー破棄 |

### 入力
| Address | Name | 説明 |
|---|---|---|
| 0x80039020 | InputMgr_GetPlayer | プレイヤー入力オブジェクト取得 |
| 0x8003907c | GetInputManager | 入力マネージャ取得 |
| 0x8003916c | ButtonHeld | ボタン押下中判定 |
| 0x80039184 | ButtonPressed | ボタン新規押下判定 |

### サウンド
| Address | Name | 説明 |
|---|---|---|
| 0x80058f5c | SoundVolumePan_Update | サウンドボリューム/パン更新 |
| 0x80064d2c | CalcSoundDirection | サウンド方向計算 |
| 0x800650e0 | EffectSpeed_Init | エフェクト速度初期化 |

### MediaBoard通信
| Address | Name | 説明 |
|---|---|---|
| 0x800652d4 | MediaBoard_SendAndCheck | MediaBoard コマンド送信 + 確認 |

### カードデータ / ビットパック
| Address | Name | 説明 |
|---|---|---|
| 0x801d5244 | card_pack | プレイヤーデータ→69バイトトラックデータへビットパックシリアライズ |
| 0x801d43c4 | card_unpack | 69バイトトラックデータ→プレイヤー構造体へデシリアライズ (card_packの逆) |
| 0x801d7160 | player_data_construct | プレイヤーデータ構造体を構築 (game_card2.cpp) |
| 0x80093c28 | card_save_trigger | カード保存開始 (プレイヤーデータ→保存バッファコピー) |
| 0x800746a8 | card_backup | カード保存バッファのバックアップ |
| 0x80096a5c | card_rw_state_machine | カード読み書き状態マシン (6ステート) |
| 0x800776cc | card_send_track_data | カードリーダーへトラックデータ送信 |
| 0x80077a54 | card_eject | カード排出 |
| 0x80088a5c | bitpack_write_field | ビットパック: 名前付きフィールドに値書き込み |
| 0x80088bb8 | bitpack_init | ビットパック: スキーマからコンテキスト初期化 |
| 0x80088bb0 | bitpack_get_used_bits | ビットパック: 使用ビット数取得 |
| 0x80088ba8 | bitpack_get_used_bytes | ビットパック: 使用バイト数取得 |
| 0x801f8e08 | wchar_to_card_char | u16文字→カード用8bit文字変換 |
| 0x800a5960 | calc_title_params | タイトルID/パラメータ計算 |
| 0x8008a0a0 | card_task_manager_create | カードUI タスクマネージャ生成 |

### サービスメニュー/テスト
| Address | Name | 説明 |
|---|---|---|
| 0x80066350 | CardRW_ErrorDisplay | カード読み書きエラー表示 |
| 0x800668fc | UI_PageDispatcher | UI ページディスパッチ |
| 0x80067494 | NetDiag_QualityPage | ネットワーク品質テストページ |
| 0x80067f74 | NetDiag_CounterPage | ネットワークカウンタページ |
| 0x80068d00 | NetDiag_PacketPage | ネットワークパケットページ |
| 0x8006ad0c | CardCreate_Page | カード作成ページ |
| 0x8006fa1c | SpeakerTest_Page | スピーカーテストページ |
| 0x80070238 | LampTest_Page | ランプテストページ |
| 0x8007044c | CameraTest_Page | カメラテストページ |
| 0x800712d8 | LampTestIndiv_Page | ランプ個別テストページ |
| 0x800715ac | SwitchTest_Page | スイッチテストページ |
| 0x80071ae0 | HandleInitialize_Page | 初期化ハンドルページ |
| 0x80071e1c | MonitorTest_Page | モニターテストページ |
| 0x8007307c | GameSetting_Page | ゲーム設定ページ |
| 0x80073878 | GameTestTopMenu_Page | ゲームテストトップメニュー |
| 0x80073c70 | ServiceMenu_Init | サービスメニュー初期化 |
| 0x80073e7c | OpenFullServiceMenu | フルサービスメニューを開く |

### その他
| Address | Name | 説明 |
|---|---|---|
| 0x8002d0b0 | ServiceButton_Handler | サービスボタン処理 |
| 0x80065458 | InterpolationStep | 補間ステップ |
| 0x80075690 | FrameUpdate | フレーム更新 |
| 0x8007593c | GameBoot_Init | ゲーム起動初期化 |
| 0x80086170 | PCBComm_Process | PCB通信処理 |
| 0x80087a54 | Timer_Decrement | タイマーデクリメント |
| 0x80087c74 | PCBCheck_StateSet | PCBチェック状態設定 |
| 0x80259a1c | DisableInterrupts | MSR 割り込み無効化 |
| 0x80259a44 | RestoreInterrupts | MSR 割り込み復帰 |
| 0x8025cc80 | GetTickCount | タイマー/クロック取得 |
| 0x80276e04 | vsprintf | 標準ライブラリ vsprintf |
| 0x80279178 | memcpy | 標準ライブラリ memcpy |
| 0x8000540c | memset | 標準ライブラリ memset |
| 0x8003b1fc | HeapAlloc | ヒープメモリ確保 (引数: サイズ) |
| 0x80039aac | InputObj_vtable14 | 入力オブジェクト vtable エントリ |
| 0x80039abc | InputObj_Update | 入力オブジェクト更新 |

---

## テキストレンダリングシステム

MKGP2 には2つのテキストシステムが存在する。

### 1. プロダクションテキスト (DrawText) -- 機能する

```
DrawText(float scale, TextRenderer* obj, int x, int y, int color, char* format, ...)
  → TextRenderer_Flush(TextRenderer* obj)
    → TextRenderer_DrawEntry(obj, entryData)  // 各エントリを GX で描画
```

- DrawText はテキストエントリを obj のバッファにキューイング
- 各エントリは 0x84 バイト:
  - +0x00: x座標 (int)
  - +0x04: y座標 (int)
  - +0x08: 色 (byte)
  - +0x0c: スケール (float)
  - +0x10: テキスト (0x74 バイト, memcpy で格納)
- TextRenderer_Flush が GX パイプラインをセットアップし全エントリを描画後、カウンタをリセット
- 479 箇所から呼ばれている

#### TextRenderer オブジェクト構造 (0xD8 バイト)

| Offset | Type | 説明 |
|---|---|---|
| +0x00 | ptr | フォントデータポインタ |
| +0x88 | ptr | テキストエントリバッファ (0x84 * N) |
| +0x8c | int | 現在のエントリ数 (最大 0x7f = 127) |
| +0x90 | ... | GX 状態保存領域 (Flush 時に save/restore) |

- 生成: `HeapAlloc(0xD8)` → 初期化関数
- グローバルインスタンス例: DAT_806d11b8 (BootPCBCheck 用), DAT_806d1880 (MainGameLoop NOTICE 用)

### 2. デバッグテキスト (OverlayText) -- 非機能

```
OverlayText(...)
  → DisableInterrupts()
  → vsprintf(&DAT_805960f0, format, va_args)
  → RestoreInterrupts()
  → return  // ← バッファを読む関数が存在しない
```

- OverlayText は vsprintf でグローバルバッファ DAT_805960f0 にフォーマット書き込みするのみ
- DAT_805960f0 を読み取る関数はバイナリ全体に存在しない (参照元は OverlayText 内部のみ)
- レンダリングバックエンド (DrawText への接続) がリリースビルドで削除されたか、未実装
- DebugOverlay_Dispatch → Profiler_DrawBar → OverlayText の全経路が dead code
- メモリパッチでは有効化不可能

---

## 主要グローバル変数

| Address | Type | 説明 |
|---|---|---|
| DAT_80598a8a | byte | デバッグフラグ。GameBoot_Init で 0 にリセット |
| DAT_805960f0 | char[] | OverlayText の vsprintf 出力バッファ (誰も読まない) |
| DAT_805940ec | int | Profiler_ResetWriteIndex でリセットされるカウンタ |
| DAT_806d11b8 | ptr | BootPCBCheck 用 TextRenderer オブジェクト |
| DAT_806d1880 | ptr | MainGameLoop NOTICE 用 TextRenderer オブジェクト |
| DAT_806d0f80 | ptr | ゲーム状態構造体ポインタ (実行時: 0x80a57e60) |
| DAT_806d0f8e | byte | 1 のとき MainGameLoop 内で MAIN_BD_ID / MEDIA_BD_ID を OverlayText 表示 |
| DAT_806d1191 | byte | GameFrameLoop 内のサービスモード切替フラグ |
| DAT_806ced30 | byte | 1 のときバックアップバッファ処理を有効化 |

---

## MainGameLoop (0x8002dd58) の構造

```
MainGameLoop(param_1):
  1. 初期化フェーズ
     - FUN_80255640() でディスプレイモード設定
     - FUN_80293bcc() ループ (vsync 待ち?)
     - GameBoot_Init() → DAT_80598a8a = 0
     - 入力/サウンド/通信の初期化

  2. NOTICE 画面 (DAT_80598a8a == 0 のときのみ)
     - BootPCBCheck_Cleanup() で既存チェックを破棄
     - TextRenderer 生成 (HeapAlloc(0xD8))
     - 599 フレームのカウントダウンループ:
       - DrawText x10: PTR_s_NOTICE_803f5670 の文字列テーブル
       - DrawText x1: 残り秒数表示 (x=588, y=24)
       - TextRenderer_Flush()
     - DAT_80598a8a != 0 の場合はこのブロックをスキップ

  3. return (後続は呼び出し元で GameFrameLoop へ)
```

### DAT_80598a8a の役割
- **MainGameLoop**: `== 0` で NOTICE 画面表示、`!= 0` でスキップ
- **GameFrameLoop**: `!= 0` でページ切替ボタン (0x40/0x80) を有効化
- **OverlayText/DebugPrintf**: `!= 0` で vsprintf を実行 (ただし描画はされない)
- メモリパッチで 1 にしても、表示に影響するのは NOTICE スキップのみ

---

## GameFrameLoop (0x8002c5e8) の構造

```
GameFrameLoop(param_1):
  無限ループ:
    1. DebugOverlay_Dispatch(param_1)  // dead code (OverlayText 経由)
    2. Profiler_RecordFrame 系計測
    3. FUN_8002dc3c()  // フレーム処理
    4. FUN_8002cd9c()  // シーン描画
    5. FUN_8002cdc8()  // シーン更新 → 戻り値で遷移判定
    6. Timer_Decrement() + PCBComm_Process()
    7. ServiceButton_Handler() → テストモード遷移
    8. Profiler_ResetWriteIndex()
    9. DAT_80598a8a チェック → ページ番号 (param_1+0x14) 変更
       - ボタン 0x40: page--
       - ボタン 0x80: page++
       - 範囲: 0 ~ 0xd
```

---

## DebugOverlay_Dispatch ページ一覧

param_1+0x14 の値に応じた switch:

| Page | 内容 | 表示される文字列例 |
|---|---|---|
| 0 | (なし - case なし) | |
| 1 | フレームタイミング | JOB/DRAW/RENDER/SYNC/ALL ave %f |
| 2 | プロファイラ + 3D | clRom, cLNormal3DWrap |
| 7 | メモリ情報 | all alloc memory %d kb, max alloc memory %d kb |
| 0xd | ビルド情報 + カード | BUILD Feb 6 2007, CARD REMAIN %d |

全て OverlayText 経由のため、画面には何も表示されない。

---

## メモリマップ

| Range | 説明 |
|---|---|
| 0x8002bca0 - 0x802e88df | MAIN_.text1 (コード) |
| 0x802e89a0 - 0x803f561f | MAIN_.data4 (データ) |
| 0x803f5670 | PTR_s_NOTICE 文字列テーブル (10エントリ) |
| 0x800CCDA0 | test_table (テストテーブル) |
| 0x800CE5A0 | g_test_ctx |
| 0x800CE5C4 | TestContext |
| 0x800D01E0 | Execute2 DMA バッファ |
| 0x806D32AC | カードタスク共有データ (4タスクが参照) |
| 0x806D7580-0x806D75C8 | プレイヤーデータ構造体 (BSS、実行時のみ) |
| 0x80312BA8 | Card R/W メッセージポインタテーブル (19エントリ) |
| 0x8048E780 | Card Life ディスプレイテーブル (string_ptr, data_ptr ペア) |

---

## カードデータ解析

### Dolphin側のカードデータフロー
- **ファイル**: `tricard_{game_id}.bin` (3トラック × 69バイト = 207バイト)
- **パス**: `D_TRIUSER_IDX` (Triforceユーザーディレクトリ)
- **カードリーダーモデル**: MKGP2は `C1231LR`
- **ベースボードコマンド**: `GCAMCommand::SerialB` (0x32)
- **読み込み**: `Command_33_ReadData(CardCapture)` → `ReadCardFile()`
- **書き込み**: `Command_53_WriteData` → `WriteCardFile()`
- **トラックデータ**: `m_card_data[3]` = `std::array<std::optional<std::array<u8, 69>>, 3>`

### ソースファイル
- `game_card2.cpp` — カードデータ管理 (DebugPrintf文字列から判明)

### プレイヤーデータ構造体 (DAT_805d2580, 0x1DB バイト)

ゲーム内メモリに展開されたプレイヤーデータ。カード保存バッファ (DAT_80598AF0) に
フィールド毎にコピーされ、最終的に FUN_801d5244 (card_pack) でビットパックされる。

```
+0x00..+0x09: ドライバー名 (5 x u16, FUN_801f8e08 で変換)
+0x0C:        BGM volume (u8)
+0x0D:        LAST_GET_ITEM (u8)
+0x0E:        スコア1 (u16)
+0x10:        スコア2 (u16)
+0x14:        TA_RECORD_TA (u32, タイムアタック記録)
+0x18:        TA_RECORD_STG (u8)
+0x19:        TA_RECORD_CRS (u8)
+0x1A:        TA_RECORD_DIR (u8, bool)
+0x1C:        TA_RECORD_CLASS (u32)
+0x20:        TA_RECORD_CHARA (u32)
+0x24:        TA_RECORD_KART (u32)
+0x48..+0x67: GHOST flags (8 stages x 4 courses, u8 each, 2 bits used)
+0x88..+0xA7: WIN_GHOST flags (8 stages x 4 courses, u8 each, 2 bits used)
+0xB6..+0xDF: アイテムアンロックフラグ (param_11=1 で全解放)
+0xE0..+0xE3: 追加アイテムフラグ
+0x121..+0x189: GET_ITEM_LIST (105=0x69 items, u8 bool each)
+0x124..+0x151: コースアンロックフラグ
+0x18C,+0x190,+0x194: ITEMPACK_1[3] (u32 x 3)
+0x198:       REMAIN (Card life, s8)
+0x199:       UPDATE flag (u8, bool)
+0x19A:       MISS_GP (u8)
+0x19B:       TitleID (u8, 0-186。0x42='B'はバトルモード=TitleID#66)
+0x19C:       title_display_param (u8, type2表示用数値 / バトル時パラメータ)
+0x19D:       CAMERA_TAG (u8, bool)
+0x19E:       LIVE_TAG (u8, bool)
+0x1A4:       COUNTRY (u32)
+0x1AC:       PLAYCOUNT_TAG (u32)
+0x1B0:       VERSION / location_test flag (u8)
+0x1B1:       NameID (u8, 0-37。NameIDテーブルへのインデックス)
+0x1B4:       COIN_MILAGE_TAG (u32, mileage)
+0x1B8:       LAST_CHARA_TAG (u8, 0-12)
+0x1B9:       LAST_KART_TAG (u8, bit 0)
+0x1BA..+0x1D1: GP結果 (3 CC x 8 courses, u8 4bit)
  +0x1BA..+0x1C1: RESULT_GP_50[8]
  +0x1C2..+0x1C9: RESULT_GP_100[8]
  +0x1CA..+0x1D1: RESULT_GP_150[8]
+0x1D4:       LAST_CLEAR_CUP_CLASS_TAG (u32, conditional)
+0x1D8:       LAST_CLEAR_CUP_URA_TAG (u8, conditional)
+0x1DA:       RANKIN_TAG (s8)
```

### カードシリアライズ — FUN_801d5244 (card_pack)

プレイヤーデータ (param_1) → 69バイトトラックデータ (param_2) への変換。
**ビットパック方式**: FUN_80088a5c が名前付きタグでフィールドを詰める。
スキーマ定義: PTR_s_APP_CODE_8049b6a0 (0x49 = 73 エントリのタグ定義テーブル)。

```
FUN_80088bb8(ctx, schema_table, 0x49, output_buf, 1)  // 初期化
FUN_80088a5c(ctx, value, tag_name, index)              // フィールド書き込み
memmove(param_2, output_buf, 0x45)                     // 69バイトコピー
```

チェックサム: 全69バイトの単純加算 (u8)。CHECKSUM タグに格納。

#### カードトラック ビットレイアウト (69バイト = 552ビット、うち484ビット使用)

スキーマテーブル (PTR_s_APP_CODE_8049b6a0) の73エントリが定義するフィールド順。
各エントリは16バイト: {name_ptr(4), bits_per_elem(4), count(4), pad(4)}

カップ略称: MR=Mario, DK=DK, WC=Wario, PC=Pac-Man, KP=Kinopio, RB=Rainbow, YI=Yoshi, DN=Waluigi

| # | ビット | バイト | タグ名 | bits×count | ソース | 説明 |
|---|--------|--------|--------|-----------|--------|------|
|  0|   0- 23|  0- 2| APP-CODE[3]| 8×3=24| DAT_806da110 | アプリケーションコード |
|  1|  24- 31|  3- 3| CHECKSUM| 8×1=8| 計算値 | 全69バイトの単純加算 |
|  2|  32- 34|  4- 4| VERSION| 3×1=3| +0x1B0 | バージョン (0:通常, 2:location_test) |
|  3|  35- 35|  4- 4| COUNTRY| 1×1=1| +0x1A4 | 国フラグ |
|  4|  36- 36|  4- 4| CAMERA-TAG| 1×1=1| +0x19D | カメラ有効フラグ |
|  5|  37- 42|  4- 5| REMAIN| 6×1=6| +0x198 | カードライフ残量 (s8, max 63) |
|  6|  43- 43|  5- 5| UPDATE| 1×1=1| +0x199 | 更新フラグ |
|  7|  44- 83|  5-10| NAME[5]| 8×5=40| +0x00 (u16×5) | ドライバー名 (FUN_801f8e08で8bit変換) |
|  8|  84- 87| 10-10| LAST_CHARA_TAG| 4×1=4| +0x1B8 | 最後に使用したキャラ (0-12) |
|  9|  88- 88| 11-11| LAST_KART_TAG| 1×1=1| +0x1B9 bit0 | 最後に使用したカート |
| 10|  89- 98| 11-12| TITLE| 10×1=10| FUN_800a5960 | タイトルID |
| 11|  99-101| 12-12| TITLE-PARAM| 3×1=3| FUN_800a5960 | タイトルパラメータ |
| 12| 102-104| 12-13| VOLUME| 3×1=3| +0x0C | BGM音量 (0-7) |
| 13| 105-114| 13-14| WINS| 10×1=10| +0x0E (u16) | 勝利数 (max 1023) |
| 14| 115-124| 14-15| LOSE| 10×1=10| +0x10 (u16) | 敗北数 (max 1023) |
| 15| 125-229| 15-28| GET-ITEM-LIST[105]| 1×105=105| +0x121..+0x189 | アイテムアンロックフラグ |
| 16| 230-236| 28-29| LAST-GET-ITEM| 7×1=7| +0x0D | 最後に取得したアイテムID (0-127) |
| 17| 237-253| 29-31| COIN_MILAGE_TAG| 17×1=17| +0x1B4 (u32) | 走行距離 (max 131071) |
| 18| 254-274| 31-34| ITEMPACK-1[3]| 7×3=21| +0x18C,+0x190,+0x194 | アイテムパック選択 |
| 19| 275-278| 34-34| RESULT-GP-50-MR| 4×1=4| +0x1BA | GP 50cc Mario Cup結果 |
| 20| 279-282| 34-35| RESULT-GP-50-DK| 4×1=4| +0x1BB | GP 50cc DK Cup |
| 21| 283-286| 35-35| RESULT-GP-50-WC| 4×1=4| +0x1BC | GP 50cc Wario Cup |
| 22| 287-290| 35-36| RESULT-GP-50-PC| 4×1=4| +0x1BD | GP 50cc Pac-Man Cup |
| 23| 291-294| 36-36| RESULT-GP-50-KP| 4×1=4| +0x1BE | GP 50cc Kinopio Cup |
| 24| 295-298| 36-37| RESULT-GP-50-RB| 4×1=4| +0x1BF | GP 50cc Rainbow Cup |
| 25| 299-302| 37-37| RESULT-GP-50-YI| 4×1=4| +0x1C0 | GP 50cc Yoshi Cup |
| 26| 303-306| 37-38| RESULT-GP-50-DN| 4×1=4| +0x1C1 | GP 50cc Waluigi Cup |
| 27| 307-310| 38-38| RESULT-GP-100-MR| 4×1=4| +0x1C2 | GP 100cc Mario Cup |
| 28| 311-314| 38-39| RESULT-GP-100-DK| 4×1=4| +0x1C3 | GP 100cc DK Cup |
| 29| 315-318| 39-39| RESULT-GP-100-WC| 4×1=4| +0x1C4 | GP 100cc Wario Cup |
| 30| 319-322| 39-40| RESULT-GP-100-PC| 4×1=4| +0x1C5 | GP 100cc Pac-Man Cup |
| 31| 323-326| 40-40| RESULT-GP-100-KP| 4×1=4| +0x1C6 | GP 100cc Kinopio Cup |
| 32| 327-330| 40-41| RESULT-GP-100-RB| 4×1=4| +0x1C7 | GP 100cc Rainbow Cup |
| 33| 331-334| 41-41| RESULT-GP-100-YI| 4×1=4| +0x1C8 | GP 100cc Yoshi Cup |
| 34| 335-338| 41-42| RESULT-GP-100-DN| 4×1=4| +0x1C9 | GP 100cc Waluigi Cup |
| 35| 339-342| 42-42| RESULT-GP-150-MR| 4×1=4| +0x1CA | GP 150cc Mario Cup |
| 36| 343-346| 42-43| RESULT-GP-150-DK| 4×1=4| +0x1CB | GP 150cc DK Cup |
| 37| 347-350| 43-43| RESULT-GP-150-WC| 4×1=4| +0x1CC | GP 150cc Wario Cup |
| 38| 351-354| 43-44| RESULT-GP-150-PC| 4×1=4| +0x1CD | GP 150cc Pac-Man Cup |
| 39| 355-358| 44-44| RESULT-GP-150-KP| 4×1=4| +0x1CE | GP 150cc Kinopio Cup |
| 40| 359-362| 44-45| RESULT-GP-150-RB| 4×1=4| +0x1CF | GP 150cc Rainbow Cup |
| 41| 363-366| 45-45| RESULT-GP-150-YI| 4×1=4| +0x1D0 | GP 150cc Yoshi Cup |
| 42| 367-370| 45-46| RESULT-GP-150-DN| 4×1=4| +0x1D1 | GP 150cc Waluigi Cup |
| 43| 371-374| 46-46| GHOST-MR[4]| 1×4=4| +0x48..+0x4B | ゴースト有無 (4コース) |
| 44| 375-378| 46-47| GHOST-DK[4]| 1×4=4| +0x4C..+0x4F | |
| 45| 379-382| 47-47| GHOST-WC[4]| 1×4=4| +0x50..+0x53 | |
| 46| 383-386| 47-48| GHOST-PC[4]| 1×4=4| +0x54..+0x57 | |
| 47| 387-390| 48-48| GHOST-KP[4]| 1×4=4| +0x58..+0x5B | |
| 48| 391-394| 48-49| GHOST-RB[4]| 1×4=4| +0x5C..+0x5F | |
| 49| 395-398| 49-49| GHOST-YI[4]| 1×4=4| +0x60..+0x63 | |
| 50| 399-402| 49-50| GHOST-DN[4]| 1×4=4| +0x64..+0x67 | |
| 51| 403-406| 50-50| WIN-GHOST-MR[4]| 1×4=4| +0x88..+0x8B | ゴースト勝利フラグ |
| 52| 407-410| 50-51| WIN-GHOST-DK[4]| 1×4=4| +0x8C..+0x8F | |
| 53| 411-414| 51-51| WIN-GHOST-WC[4]| 1×4=4| +0x90..+0x93 | |
| 54| 415-418| 51-52| WIN-GHOST-PC[4]| 1×4=4| +0x94..+0x97 | |
| 55| 419-422| 52-52| WIN-GHOST-KP[4]| 1×4=4| +0x98..+0x9B | |
| 56| 423-426| 52-53| WIN-GHOST-RB[4]| 1×4=4| +0x9C..+0x9F | |
| 57| 427-430| 53-53| WIN-GHOST-YI[4]| 1×4=4| +0xA0..+0xA3 | |
| 58| 431-434| 53-54| WIN-GHOST-DN[4]| 1×4=4| +0xA4..+0xA7 | |
| 59| 435-435| 54-54| MISS_GP| 1×1=1| +0x19A | GPミスカウント |
| 60| 436-441| 54-55| PLAYCOUNT_TAG| 6×1=6| +0x1AC (u32) | プレイ回数 (max 63) |
| 61| 442-460| 55-57| TA_RECORD_TA| 19×1=19| +0x14 (u32) | TAベストタイム |
| 62| 461-463| 57-57| TA_RECORD_STG| 3×1=3| +0x18 | TAステージ (0-7) |
| 63| 464-464| 58-58| TA_RECORD_CRS| 1×1=1| +0x19 | TAコース |
| 64| 465-465| 58-58| TA_RECORD_DIR| 1×1=1| +0x1A | TA方向 (bool) |
| 65| 466-466| 58-58| TITLE_PARAM2| 1×1=1| FUN_800a5960 | タイトルパラメータ2 |
| 66| 467-468| 58-58| TA_RECORD_CLASS| 2×1=2| +0x1C (u32) | TAクラス (0-2) |
| 67| 469-472| 58-59| TA_RECORD_CHARA| 4×1=4| +0x20 (u32) | TAキャラ (0-12) |
| 68| 473-474| 59-59| TA_RECORD_KART| 2×1=2| +0x24 (u32) | TAカート (0-3) |
| 69| 475-476| 59-59| LAST_CLEAR_CUP_CLASS_TAG| 2×1=2| +0x1D4 (u32) | 最終クリアクラス (条件付き) |
| 70| 477-477| 59-59| LAST_CLEAR_CUP_URA_TAG| 1×1=1| +0x1D8 | 裏カップフラグ (条件付き) |
| 71| 478-478| 59-59| LIVE_TAG| 1×1=1| +0x19E | ライブフラグ |
| 72| 479-483| 59-60| RANKIN_TAG[5]| 1×5=5| +0x1DA (s8) | ランキングフラグ |
|  | 484-551| 60-68| (未使用)| | | 68ビット (8.5バイト) パディング |

**注**: LAST_CLEAR_CUP_CLASS_TAG と LAST_CLEAR_CUP_URA_TAG は条件付き。
現在のスピードクラス (DAT_806d12cc) で1つでもクリア済みカップがあるときのみ書き込まれる。

### カードR/W状態マシン — FUN_80096a5c

DAT_806d1220 で状態遷移:
```
State 0: FUN_801d5244 でプレイヤーデータ→トラックバッファ(DAT_805ac55c) シリアライズ
         FUN_800776cc でトラックデータ(0x45バイト)をカードリーダーに送信開始
State 1: 最初のトラック書き込み完了待ち
State 2: FUN_80095388/80094d80 でカードR/Wコマンド発行
State 3: カードR/Wレスポンス待ち + エラーハンドリング
         成功時: プレイヤーデータ→保存バッファ再コピー + FUN_800746a8 バックアップ
State 4: カード排出 (FUN_80077a54)
State 5: 排出完了待ち (FUN_80098a90)
```

### カード保存フロー全体像

```
CardCreate_Page (サービスメニュー)
  └→ FUN_801d7160 (プレイヤーデータ構築: DAT_805d2580)
  └→ FUN_80093c28 (カード保存トリガー)
       ├→ プレイヤーデータ → カード保存バッファ (DAT_80598AF0) コピー
       ├→ FUN_800746a8 (バックアップ: 80598xxx → 8059Fxxx)
       └→ FUN_80096a5c (カードR/W状態マシン)
            ├→ FUN_801d5244 (card_pack: ビットパックシリアライズ → DAT_805ac55c)
            ├→ FUN_800776cc (トラックデータ送信: 69バイト)
            └→ FUN_80095388/80094d80 (カードR/Wプロトコル)
```

### 主要メモリアドレス
| Address | Size | 説明 |
|---------|------|------|
| DAT_805d2580 | 0x1DB | プレイヤーデータ構造体 (ゲーム内) |
| DAT_80598AF0 | ~0x1DC | カード保存バッファ (保存時コピー先) |
| DAT_805ac55c | 0x45+ | カードトラックバッファ (シリアライズ済み69バイト) |
| DAT_80598AC4 | 1 | カード保存フラグ |
| DAT_80598AC8 | 0x28 | タイムスタンプ (FUN_8025ce88) |
| DAT_80598CCC | u32 | カードライフカウンタ (max 200) |

### カードタスクファクトリテーブル (0x803FE7E8) — 修正版
```
task[0]: FUN_8008A2A0  vtable=0x803FE898  (条件付き、alloc 0x1C) — カードコントローラ
task[1]: FUN_8008AA20  vtable=0x803FE86C  (常に生成、alloc 0x10) — カードメッセージ表示
task[2]: FUN_8008AD28  vtable=0x803FE840  (条件付き) — CardErrDisplay
task[3]: FUN_8008BF30  (条件付き)
```
※ 前回の解析で task[1]=0x8008A2E0 は誤り (FUN_8008A2A0 の関数内部だった)

タスクマネージャ: `FUN_8008a0a0(param_1, hasTask0, hasTask2, hasTask3)`
共有データ: `0x806D32AC, 0x806D32B4, 0x806D32BC, 0x806D32C4`

### CardErrDisplay クラス (task[2])
- **vtable**: `0x803FE840` (15メソッド)
- **コンストラクタ**: `FUN_8008AD28` (alloc 0x3C)
- **表示関数**: `FUN_8008B8C8` (vtable[4]) — param_1[5] = state で分岐

### ビットパックシステム (FUN_80088a5c / FUN_80088bb8)
- **スキーマ定義テーブル**: PTR_s_APP_CODE_8049b6a0 (0x8049b6a0〜0x8049bb30)
- **73 (0x49) エントリ** × 16バイト = 1168バイト
- エントリ構造: `{ char* name (4), u32 bits_per_elem (4), u32 count (4), u32 pad (4) }`
- **FUN_80088bb8(ctx, schema, 0x49, buf, direction)**: コンテキスト初期化。各フィールドのビットオフセットを計算
- **FUN_80088a5c(ctx, value, tag_name, index)**: 名前付きタグでフィールド値をビットストリームに書き込み
- **FUN_80088bb0(ctx)**: 使用ビット数を返す
- **FUN_80088ba8(ctx)**: 使用バイト数を返す
- 合計: 484ビット使用 / 552ビット (69バイト) 中
- チェックサム: 全69バイトの単純加算 (u8 wrap-around)、CHECKSUMタグ (byte 3) に格納

### Card R/W エラーメッセージ (0x803128CC〜)
```
0x803129CC: "Card R/W initialize error."
0x80312A68: "Card will be printed after game."
0x80312B74: "Card R/W write error."
0x80312B8C: "Card R/W read error."
0x80313560: "Card jam."
```

---

## 称号システム (Title/Name)

### 概要
称号は **TitleID** (接頭語, 0-186) × **NameID** (接尾語, 0-37) の組み合わせで構成される。

カードデータ上では:
- **TITLE** (10bit): calc_title_params (0x800a5960) が計算
- **TITLE-PARAM** (3bit): 同上
- **TITLE_PARAM2** (1bit): 同上

### 称号表示関数 — get_title_name (FUN_800a6bc8)

```c
char* get_title_name(context, TitleID, param, color, NameID)
// TitleID: 0-0xBA (186), NameID: 0-0x25 (37)
// is_japanese() == 0 → 日本語, else 英語
// type==0: sprintf("%s%s", prefix, suffix)    // 接頭語+接尾語
// type==1: sprintf("%s", prefix)              // 接頭語のみ
// type==2: sprintf("%d%s%s", num, prefix, suffix)  // 数値+接頭語+接尾語
```

- 呼び出し元: FUN_801d1eac (ランキング表示)
  - `get_title_name(ctx, rankData[0x26], rankData[0x27], rankData[4]>>4, rankData[0x28])`

### TitleIDテーブル (0x803260b8, DATA section)

187エントリ × 12バイト = `{type(4), jp_ptr(4), en_ptr(4)}`

**Type分布**:
- Type 0: 141エントリ (prefix+suffix "%s%s") — IDs: 0, 46-186
- Type 1: 45エントリ (standalone "%s") — IDs: 1-45
- Type 2: 1エントリ (number+prefix+suffix "%d%s%s") — ID: 66

#### 全TitleID一覧 (JP/EN)

| ID | Type | JP | EN | カテゴリ |
|----|------|----|----|----------|
| 0 | 0 | (空) | (空) | デフォルト |
| 1 | 1 | 表５０ＧＰ無敗制覇 | M50GP Flawless | GP無敗 |
| 2 | 1 | 表１００ＧＰ無敗制覇 | M100GP Flawless | GP無敗 |
| 3 | 1 | ５０ＧＰ無敗制覇 | 50GP Flawless | GP無敗 |
| 4 | 1 | １００ＧＰ無敗制覇 | 100GP Flawless | GP無敗 |
| 5 | 1 | 表１５０ＧＰ無敗制覇 | M150GP Flawless | GP無敗 |
| 6 | 1 | １５０ＧＰ無敗制覇 | 150GP Flawless | GP無敗 |
| 7 | 1 | オモテ５０ＧＰ制覇 | M50GP CLEAR | GPクリア |
| 8 | 1 | オモテ１００ＧＰ制覇 | M100GP CLEAR | GPクリア |
| 9 | 1 | ウラ５０ＧＰ制覇 | SP50GP CLEAR | GPクリア |
| 10 | 1 | ウラ１００ＧＰ制覇 | SP100GP CLEAR | GPクリア |
| 11 | 1 | オモテ１５０ＧＰ制覇 | M150GP CLEAR | GPクリア |
| 12 | 1 | ウラ１５０ＧＰ制覇 | SP150GP CLEAR | GPクリア |
| 13 | 1 | ヨッシーカップ制覇 | Yoshi Cup CLEAR | カップクリア |
| 14 | 1 | マリオカップ制覇 | Mario Cup CLEAR | カップクリア |
| 15 | 1 | ワルイージカップ制覇 | Waluigi Cup CLEAR | カップクリア |
| 16 | 1 | ＤＫカップ制覇 | DK Cup CLEAR | カップクリア |
| 17 | 1 | ワリオカップ制覇 | Wario Cup CLEAR | カップクリア |
| 18 | 1 | パックマンカップ制覇 | Pac-Man Cup CLEAR | カップクリア |
| 19 | 1 | クッパカップ制覇 | Bowser Cup CLEAR | カップクリア |
| 20 | 1 | レインボーカップ制覇 | Rainbow Cup CLEAR | カップクリア |
| 21 | 1 | 裏ヨッシーカップ制覇 | SP Yoshi Cup CLEAR | 裏カップクリア |
| 22 | 1 | 裏マリオカップ制覇 | SP Mario Cup CLEAR | 裏カップクリア |
| 23 | 1 | 裏ワルイージカップ制覇 | SP Waluigi Cup CLEAR | 裏カップクリア |
| 24 | 1 | 裏ＤＫカップ制覇 | SP DK Cup CLEAR | 裏カップクリア |
| 25 | 1 | 裏ワリオカップ制覇 | SP Wario C CLEAR | 裏カップクリア |
| 26 | 1 | 裏パックマンカップ制覇 | SP Pac Cup CLEAR | 裏カップクリア |
| 27 | 1 | 裏クッパカップ制覇 | SP BowserC CLEAR | 裏カップクリア |
| 28 | 1 | 裏レインボーカップ制覇 | SP Rainbow CLEAR | 裏カップクリア |
| 29 | 1 | １５０ヨッシー制覇 | 150 Yoshi CLEAR | 150ccカップクリア |
| 30 | 1 | １５０マリオ制覇 | 150 Mario CLEAR | 150ccカップクリア |
| 31 | 1 | １５０ワルイージ制覇 | 150 Waluigi CLEAR | 150ccカップクリア |
| 32 | 1 | １５０ＤＫ制覇 | 150 DK CLEAR | 150ccカップクリア |
| 33 | 1 | １５０ワリオ制覇 | 150 Wario CLEAR | 150ccカップクリア |
| 34 | 1 | １５０パックマン制覇 | 150 Pac-man CLEAR | 150ccカップクリア |
| 35 | 1 | １５０クッパ制覇 | 150 Bowser CLEAR | 150ccカップクリア |
| 36 | 1 | １５０レインボー制覇 | 150 Rainbow CLEAR | 150ccカップクリア |
| 37 | 1 | 裏１５０ヨッシー制覇 | SP150 Yoshi CLEAR | 裏150ccカップクリア |
| 38 | 1 | 裏１５０マリオ制覇 | SP150 Mario CLEAR | 裏150ccカップクリア |
| 39 | 1 | 裏１５０ワルイージ制覇 | SP150 Waluigi CLEAR | 裏150ccカップクリア |
| 40 | 1 | 裏１５０ＤＫ制覇 | SP150 DK CLEAR | 裏150ccカップクリア |
| 41 | 1 | 裏１５０ワリオ制覇 | SP150 Wario CLEAR | 裏150ccカップクリア |
| 42 | 1 | 裏１５０パックマン制覇 | SP150 Pac-man CLEAR | 裏150ccカップクリア |
| 43 | 1 | 裏１５０クッパ制覇 | SP150 Bowser CLEAR | 裏150ccカップクリア |
| 44 | 1 | 裏１５０レインボー制覇 | SP150 Rainbow CLEAR | 裏150ccカップクリア |
| 45 | 1 | ＴＡランキング制覇 | TA Ranking CLEAR | タイムアタック |
| 46 | 0 | 無敗 | Undefeated | バトル系 |
| 47 | 0 | 元無敗 | First Loss | バトル系 |
| 48 | 0 | 期待のバトル | Expected | バトル系 |
| 49 | 0 | 注目のバトル | Attractive | バトル系 |
| 50 | 0 | 強きバトル | Strong | バトル系 |
| 51 | 0 | 激しきバトル | Slashing | バトル系 |
| 52 | 0 | １０人斬り | 10 Wins | バトル系 |
| 53 | 0 | 腕利きバトル | Hotshot | バトル系 |
| 54 | 0 | 凄腕バトル | Courage | バトル系 |
| 55 | 0 | 豪腕バトル | Awesome | バトル系 |
| 56 | 0 | ３０人斬り | 30 Wins | バトル系 |
| 57 | 0 | 激強バトル | Heavy | バトル系 |
| 58 | 0 | 偉大なバトル | Great | バトル系 |
| 59 | 0 | 闘神となりし | Mighty | バトル系 |
| 60 | 0 | ５０人斬り | 50 Wins | バトル系 |
| 61 | 0 | 天才バトル | Genius | バトル系 |
| 62 | 0 | 噂のバトル | Reputed | バトル系 |
| 63 | 0 | 恐怖のバトル | Fearful | バトル系 |
| 64 | 0 | カリスマ | Charisma | バトル系 |
| 65 | 0 | １００人斬り | 100 Wins | バトル系 |
| 66 | 2 | 連勝バトル | Consecutive | バトル系(数値付き) |
| 67 | 0 | 無敵のバトル | Invincible | バトル系 |
| 68 | 0 | 遅咲きバトル | Come Back | バトル系 |
| 69 | 0 | 百戦錬磨 | Victorious | バトル系 |
| 70 | 0 | 闘魂 | Military | 速度/バトル系 |
| 71 | 0 | 対戦歓迎 | Champion | 速度/バトル系 |
| 72 | 0 | 熱きバトル | Blazing | 速度/バトル系 |
| 73 | 0 | 速き | Speedy | 速度系 |
| 74 | 0 | 弾丸 | Bullet | 速度系 |
| 75 | 0 | 猛烈 | Fierce | 速度系 |
| 76 | 0 | 全開 | Full Out | 速度系 |
| 77 | 0 | 爆走 | Velocity | 速度系 |
| 78 | 0 | 高速 | Speed | 速度系 |
| 79 | 0 | 快速 | Swift | 速度系 |
| 80 | 0 | 駿速 | Fleet | 速度系 |
| 81 | 0 | 音速 | Sonic | 速度系 |
| 82 | 0 | 光速 | Streaking | 速度系 |
| 83 | 0 | 激速 | Rapid | 速度系 |
| 84 | 0 | 超速 | Cannonball | 速度系 |
| 85 | 0 | 驚速 | Arrow | 速度系 |
| 86 | 0 | 怪速 | Swift | 速度系 |
| 87 | 0 | 鬼速 | Zippy | 速度系 |
| 88 | 0 | 究極 | Ultimate | 速度系 |
| 89 | 0 | １５０cc速き | 150 Speedy | 150cc速度系 |
| 90 | 0 | １５０cc弾丸 | 150 Bullet | 150cc速度系 |
| 91 | 0 | １５０cc猛烈 | 150 Fierce | 150cc速度系 |
| 92 | 0 | １５０cc全開 | 150 FullOut | 150cc速度系 |
| 93 | 0 | １５０cc爆走 | 150Velocity | 150cc速度系 |
| 94 | 0 | １５０cc高速 | 150 Speed | 150cc速度系 |
| 95 | 0 | １５０cc快速 | 150 Swift | 150cc速度系 |
| 96 | 0 | １５０cc駿速 | 150 Fleet | 150cc速度系 |
| 97 | 0 | １５０cc音速 | 150 Sonic | 150cc速度系 |
| 98 | 0 | １５０cc光速 | 150 Streak | 150cc速度系 |
| 99 | 0 | １５０cc激速 | 150 Rapid | 150cc速度系 |
| 100 | 0 | １５０cc超速 | 150 Cannon | 150cc速度系 |
| 101 | 0 | １５０cc驚速 | 150 Arrow | 150cc速度系 |
| 102 | 0 | １５０cc怪速 | 150 Swift | 150cc速度系 |
| 103 | 0 | １５０cc鬼速 | 150 Zippy | 150cc速度系 |
| 104 | 0 | １５０cc究極 | 150Ultimate | 150cc速度系 |
| 105 | 0 | カートコンプ | Kart Comp | ゲームプレイ系 |
| 106 | 0 | コインコンプ | Coin Comp | ゲームプレイ系 |
| 107 | 0 | 万枚獲得 | 10000 coin | ゲームプレイ系 |
| 108 | 0 | ピカピカ | Brand New | ゲームプレイ系 |
| 109 | 0 | 比類なき | Massive | 属性系 |
| 110 | 0 | 情熱的な | Passionate | 属性系 |
| 111 | 0 | すてきな | Awesome | 属性系 |
| 112 | 0 | ＶＩＰ | VIP | 属性系 |
| 113 | 0 | ベテラン | Veteran | 属性系 |
| 114 | 0 | ゴールデン | Golden | 属性系 |
| 115 | 0 | ナンパな | Loving | 属性系 |
| 116 | 0 | 不運な | Unlucky | 属性系 |
| 117 | 0 | ミラクル | Miracle | 属性系 |
| 118 | 0 | 守りの堅い | Defensive | 属性系 |
| 119 | 0 | 衝撃的な | Shocking | 属性系 |
| 120 | 0 | ドリフト | Drift | 属性系 |
| 121 | 0 | 猛特訓中の | Training | 属性系 |
| 122 | 0 | 空飛ぶ | Flying | 属性系 |
| 123 | 0 | 正義の | Justice | 属性系 |
| 124 | 0 | 武闘派 | Assaultive | 属性系 |
| 125 | 0 | 技巧派 | Virtuosity | 属性系 |
| 126 | 0 | オキテ破りの | Evading | 属性系 |
| 127 | 0 | 豪快な | Dynamic | 属性系 |
| 128 | 0 | 闘志に満ちた | Spiritual | 属性系 |
| 129 | 0 | 百発百中 | Sniper | 属性系 |
| 130 | 0 | やさしい | Kind | 性格系 |
| 131 | 0 | 進化する | Evolution | 性格系 |
| 132 | 0 | 強化された | Encahnted | 性格系 |
| 133 | 0 | 宅配 | Delivery | 性格系 |
| 134 | 0 | 傷だらけの | Bruising | 性格系 |
| 135 | 0 | 通りすがりの | Passing | 性格系 |
| 136 | 0 | 超一流の | Top | 性格系 |
| 137 | 0 | パーフェクト | Perfect | 性格系 |
| 138 | 0 | 噂の | Rumored | 性格系 |
| 139 | 0 | 紙一重の | Close | 性格系 |
| 140 | 0 | ダンシング | Dancing | アクション系 |
| 141 | 0 | チャンバラ | Clanging | アクション系 |
| 142 | 0 | 艶やかな | Sparkling | アクション系 |
| 143 | 0 | 萌え系 | Manga | アクション系 |
| 144 | 0 | 大海原の | Blue Water | ロケーション系 |
| 145 | 0 | かわいい | Cute | 性格系 |
| 146 | 0 | 戦国の | Warring | ロケーション系 |
| 147 | 0 | マジカル | Magical | 属性系 |
| 148 | 0 | 極悪 | Evil | 属性系 |
| 149 | 0 | 危機一髪 | Crisis | 属性系 |
| 150 | 0 | 攻撃的 | Hostile | 属性系 |
| 151 | 0 | 先行逃げ切り | Litter | レース戦略系 |
| 152 | 0 | 完全武装 | Armed | レース戦略系 |
| 153 | 0 | さまよえる | Wandering | レース戦略系 |
| 154 | 0 | サンデー | Sunday | 時間帯系 |
| 155 | 0 | バトル | Battle | カテゴリ系 |
| 156 | 0 | １５０バトル | 150 Battle | カテゴリ系 |
| 157 | 0 | 早起きの | Early | 時間帯系 |
| 158 | 0 | 目覚めた | Groggy | 時間帯系 |
| 159 | 0 | ハングリー | Hungry | 時間帯系 |
| 160 | 0 | ランチタイム | Lunchtime | 時間帯系 |
| 161 | 0 | 真昼の | Noon | 時間帯系 |
| 162 | 0 | 灼熱の | Blazing | 時間帯系 |
| 163 | 0 | おやつ大好き | Tea Break | 時間帯系 |
| 164 | 0 | 夕暮れ時の | Evening | 時間帯系 |
| 165 | 0 | サンセット | Sunset | 時間帯系 |
| 166 | 0 | キラキラ | Brilliant | 時間帯系 |
| 167 | 0 | ムーンライト | Moonlight | 時間帯系 |
| 168 | 0 | 夜遊び好きの | Nightlife | 時間帯系 |
| 169 | 0 | ミッドナイト | Midnight | 時間帯系 |
| 170 | 0 | 闇夜に潜む | Darkness | 時間帯系 |
| 171 | 0 | 高速コースの | High speed | コース系 |
| 172 | 0 | 遊園地の | Fun Park | コース系 |
| 173 | 0 | ハイウェイの | Highway | コース系 |
| 174 | 0 | 海岸線の | Beach | コース系 |
| 175 | 0 | アリーナの | Arena | コース系 |
| 176 | 0 | スタジアムの | Stadium | コース系 |
| 177 | 0 | ジャングルの | Jungle | コース系 |
| 178 | 0 | ダートの | Dirty | コース系 |
| 179 | 0 | ストリートの | Street | コース系 |
| 180 | 0 | 雪道の | Snow | コース系 |
| 181 | 0 | 峠の | Peak | コース系 |
| 182 | 0 | 迷宮の | Maze | コース系 |
| 183 | 0 | キャッスルの | Castle | コース系 |
| 184 | 0 | 城壁の | Rampart | コース系 |
| 185 | 0 | コースターの | Coaster | コース系 |
| 186 | 0 | ダウンヒルの | Downhill | コース系 |

**注**: EN ID 132 "Encahnted" はゲームデータ内のタイポ (正しくは "Enchanted")。
EN ID 86 と ID 102 はどちらも "Swift" (JP では怪速/１５０cc怪速 で区別される)。

### NameIDテーブル (0x80411b18, BSS section)

38エントリ × 8バイト = `{jp_ptr(4), en_ptr(4)}`

BSS領域だがGhidraのメモリダンプにデータが存在し読み取り可能。

| ID | JP | EN |
|----|----|----|
| 0 | ルーキー | Rookie |
| 1 | アタッカー( | Attacker |
| 2 | ドライバー( | Driver |
| 3 | レーサー | Racer |
| 4 | リーダー | Leader |
| 5 | エース | Ace |
| 6 | スター | Star |
| 7 | ファイター( | Fighter |
| 8 | マスター | Master |
| 9 | チャンプ | Champ |
| 10 | イナズマ | Thunder |
| 11 | サムライ | Samurai |
| 12 | ナイト | Knight |
| 13 | ヒーロー | Hero |
| 14 | ヒロイン | Heroine |
| 15 | プリンス | Prince |
| 16 | プリンセス( | Princess |
| 17 | キング | King |
| 18 | クイーン | Queen |
| 19 | カイザー | Conqueror |
| 20 | マリカ名人( | Artist |
| 21 | 走り屋 | Runner |
| 22 | 一匹狼 | Lone Wolf |
| 23 | 暴れ馬 | Mustang |
| 24 | 黒豹 | Panther |
| 25 | 勝負師 | Duelist |
| 26 | 戦闘機 | WarMachine( |
| 27 | 暴走列車 | TrainFleet( |
| 28 | 韋駄天 | Idaten |
| 29 | 獅子王 | Lion King |
| 30 | 撃壊王 | Blast King( |
| 31 | 爆走王 | Speed King( |
| 32 | マリカ大王( | MK King |
| 33 | マリカ覇王( | MK Kaiser |
| 34 | マリカ帝王( | MK Emperor( |
| 35 | マリカ魔王( | MK Demon |
| 36 | マリカ鬼神( | MK Lord |
| 37 | 最強伝説 | Legendary |

称号の組み合わせ例:
- type=0: 「かわいい」(TitleID=145) + 「ルーキー」(NameID=0) → **「かわいいルーキー」** / **「Cute Rookie」**
- type=1: 「５０ＧＰ無敗制覇」(TitleID=3) → **「５０ＧＰ無敗制覇」** (接尾語なし)
- type=2: 「連勝バトル」(TitleID=66) + 数値 + 接尾語 → **「5連勝バトルファイター」**

### calc_title_params エンコード詳細 (0x800a5960)

プレイヤー構造体の3フィールドをカードの14ビットに圧縮する。

```
入力:
  title_id     = player[0x19B]  (u8, 0-186 or 'B'=0x42)
  battle_param = player[0x19C]  (u8)
  name_id      = player[0x1B1]  (u8, 0-37)

処理:
  if title_id == 0x42 ('B'):  // バトルモード (TitleID #66)
    mode_bits = 0xF           // 特殊マーカー
    param = battle_param      // 0x19Cの値をそのまま使用
  else:                       // 通常モード
    mode_bits = title_id >> 4     // TitleIDの上位ニブル (0-11)
    param = title_id & 0xF        // TitleIDの下位ニブル (0x19Cは無視される)

出力:
  TITLE      = (mode_bits << 6) | (name_id & 0x3F)   // 10ビット
  TITLE_PARAM = param & 0xF                            //  4ビット (3+1に分割格納)
```

カード書き込み (card_pack):
```
bitpack("TITLE",        TITLE & 0x3FF)             // 10 bits
bitpack("TITLE_PARAM",  TITLE_PARAM & 0x7)         //  3 bits
bitpack("TITLE_PARAM2", (TITLE_PARAM >> 3) & 0x1)  //  1 bit
                                           合計 14 bits
```

### title_decode デコード詳細 (0x800a59ac)

card_unpack (0x801d43c4) から呼ばれる。calc_title_paramsの完全な逆変換。

```
入力:
  title_10bit       = bitpack("TITLE") & 0xFFFF
  title_param_4bit  = (bitpack("TITLE_PARAM") & 0xFF) | ((bitpack("TITLE_PARAM2") & 1) << 3)

処理:
  mode_bits = (title_10bit >> 6) & 0xF
  name_id   = title_10bit & 0x3F

  if mode_bits == 0xF:  // バトルモード
    player[0x19B] = 'B'                                    // TitleID = 0x42
    player[0x19C] = title_param_4bit & 0xF                 // battle_param復元
  else:                 // 通常モード
    player[0x19B] = (mode_bits << 4) | (title_param_4bit & 0xF)  // TitleID復元
    player[0x19C] = 0                                      // battle_paramクリア

  player[0x1B1] = name_id
```

注: 非バトルモードではplayer[0x19C] (type2表示用数値) がカードround-tripで0にクリアされる。
ゲーム側でプレイ統計から再計算される想定。

### TitleIDテーブルエントリ構造体

```c
// TitleIDテーブル (0x803260b8), 187エントリ
struct TitleEntry {      // 12 bytes
    int32_t type;        // +0x00: 表示タイプ (0=prefix+suffix, 1=standalone, 2=num+prefix+suffix)
    char*   en_str;      // +0x04: 英語文字列ポインタ
    char*   jp_str;      // +0x08: 日本語文字列ポインタ
};
```

### NameIDテーブルエントリ構造体

```c
// NameIDテーブル (0x80411b18), 38エントリ, BSS領域
struct NameEntry {       // 8 bytes
    char*   en_str;      // +0x00: 英語文字列ポインタ
    char*   jp_str;      // +0x04: 日本語文字列ポインタ
};
```

### get_title_name 表示フォーマット (0x800a6bc8)

```c
char* get_title_name(void* game_ctx, int title_id, byte type2_number, int unused, int name_id)
// title_id: 0-186 (0xBA), name_id: 0-37 (0x25)
// TitleEntry.type による分岐:
//   type 0: sprintf(buf, "%s%s", title_str, name_str)
//   type 1: sprintf(buf, "%s",   title_str)
//   type 2: sprintf(buf, "%d%s%s", type2_number, title_str, name_str)
// is_japanese()==0 → 英語, else → 日本語
```

### 称号決定フロー

プレイ統計に基づいてTitleID/NameIDが決定されプレイヤーデータに書き込まれるまでの全体像:

```
1. race_result_handler (0x800a6578)
   └→ 勝敗/コースカウント更新後、determine_title(0) を呼ぶ

2. determine_title (0x800a70fc) — 称号決定のコアロジック
   ├→ クリア済みコース/ゴーストを集計
   ├→ FUN_800a8a28(score, lastChara) → NameID (DAT_806d13dc) を算出
   ├→ param==1 なら NameID計算のみで早期リターン
   ├→ ゲームモード (DAT_806d1294) に応じてチェック関数テーブルを選択:
   │     mode 0 (Race/GP): PTR_LAB_80411c48 (0x20=32 entries), Extra: FUN_800a7600
   │     mode 1 (Battle):  PTR_LAB_80411cc8 (0x10=16 entries), Extra: FUN_800a7548
   │     mode 2 (TimeAttack): PTR_LAB_80411d08 (0x28=40 entries), Extra: なし
   ├→ "Extra" チェックを先に実行 → DAT_806d13d4 (staging TitleID) にセット
   └→ "Fixed" テーブルを順にイテレーション、各チェック関数がTitleIDを返す
       → 有効かつ現在と異なるTitleIDが見つかったらbreak
   結果: DAT_806d13d4=TitleID, DAT_806d13d8=param, DAT_806d13dc=NameID

3. get_title_result (0x800a6ffc) — staging値のgetter
   ├→ DAT_806d13d0==0 なら determine_title(1) を先に呼ぶ (NameID遅延初期化)
   └→ DAT_806d13d4/d8/dc を返す (TitleID < 1 なら失敗=0)

4. has_title_changed (0x801602bc) — 称号変化チェック
   ├→ get_title_result で staging 値を取得
   └→ DAT_805d271b/1c/2731 (現在のプレイヤーデータ) と比較
       → 異なれば 1 (変化あり), 同じなら 0

5. result_screen_update (0x8015f210) — リザルト画面ステートマシン
   ├→ case 4 (称号比較なし):
   │     get_title_result → DAT_805d271b/1c/2731 に直接書き込み
   ├→ case 5-6 (称号変化演出):
   │     新旧称号の比較アニメーション表示
   │     → case 6→7遷移時: get_title_result → DAT_805d271b/1c/2731 に書き込み
   └→ case 99: 完了

書き込み先グローバル:
  DAT_805d271b = player[0x19B] = TitleID
  DAT_805d271c = player[0x19C] = title_display_param
  DAT_805d2731 = player[0x1B1] = NameID
```

### Staging グローバル変数

| Address | 説明 |
|---------|------|
| DAT_806d13d0 | NameID初期化済みフラグ (0=未初期化, 1=初期化済み) |
| DAT_806d13d4 | staging TitleID (determine_title が書き込み) |
| DAT_806d13d8 | staging title_display_param |
| DAT_806d13dc | staging NameID (FUN_800a8a28 が算出) |
| DAT_806d1294 | ゲームモード (0=Race/GP, 1=Battle, 2=TimeAttack) |

### 称号チェック関数テーブル

| モード | テーブルアドレス | エントリ数 | Extra関数 |
|--------|-----------------|-----------|-----------|
| Race/GP (0) | PTR_LAB_80411c48 | 32 (0x20) | FUN_800a7600 |
| Battle (1) | PTR_LAB_80411cc8 | 16 (0x10) | FUN_800a7548 |
| TimeAttack (2) | PTR_LAB_80411d08 | 40 (0x28) | なし |

各テーブルエントリはチェック関数へのポインタ。チェック関数はプレイ統計
(勝利数、コースクリア状況、ゴースト撃破数など) を評価し、条件を満たすTitleIDを返す。

### 関連関数

| Address | Name | 説明 |
|---------|------|------|
| 0x800a70fc | determine_title | 称号決定コアロジック (チェック関数テーブルでTitleID選定) |
| 0x800a6ffc | get_title_result | staging称号値のgetter (DAT_806d13d4/d8/dc) |
| 0x800a6578 | race_result_handler | レース結果処理 → determine_title(0) 呼び出し |
| 0x801602bc | has_title_changed | staging vs 現在の称号を比較 (1=変化あり) |
| 0x8015f210 | result_screen_update | リザルト画面SM → case4/6でプレイヤーデータに書き戻し |
| 0x800a8a28 | (unnamed) | NameID算出 (スコア+キャラ→NameID) |
| 0x800a7600 | (unnamed) | Race Extra チェック関数 |
| 0x800a7548 | (unnamed) | Battle Extra チェック関数 |
| 0x800a7a7c-800a8a20 | check_* (41関数) | 称号チェック関数群 → `mkgp2_title_check_functions.md` 参照 |
| 0x800a6bc8 | get_title_name | 称号名フォーマット (TitleID + NameID → 文字列) |
| 0x800a6ddc | get_title_name_global | グローバル変数版 (DAT_805d271b等から読む) |
| 0x800a5960 | calc_title_params | TitleID/param計算 (カード保存用エンコード) |
| 0x800a59ac | title_decode | タイトルデコード (card_unpackから呼ばれる) |
| 0x8002f8cc | is_japanese | 言語判定 (0=日本語) |
| 0x801d1eac | ranking_display | ランキング表示 (get_title_nameの呼び出し元) |

---

## キャラクター一覧

g_characterId (0x806d125c) の値。LAST_CHARA_TAG (4bit, 0-12) としてカードに保存される。
ポインタテーブル: PTR_s_chara_mario_r_hand_null_joint_803fec24 (13エントリ)。
確認元: FUN_80044d50 (キャラクター/カートセットアップ関数) でIDによる分岐・テーブル参照。

| ID | 内部名 | キャラクター名 (JP) | 英語名 | 備考 |
|----|--------|---------------------|--------|------|
| 0 | mario | マリオ | Mario | |
| 1 | luigi | ルイージ | Luigi | |
| 2 | wario | ワリオ | Wario | |
| 3 | peach | ピーチ | Peach | 固有: pony_null_joint (ポニーテール) |
| 4 | koopa | クッパ | Bowser | 固有: fire_null_joint (炎) |
| 5 | kinopio | キノピオ | Toad | |
| 6 | donkey | ドンキーコング | Donkey Kong | |
| 7 | yoshi | ヨッシー | Yoshi | |
| 8 | pacman | パックマン | Pac-Man | |
| 9 | mspacman | ミズ・パックマン | Ms. Pac-Man | |
| 10 | monster | アカベイ | Blinky | 固有: propeller/wing_joint (プロペラ/翼) |
| 11 | waluigi | ワルイージ | Waluigi | 固有: Suspension_joint (サスペンション) |
| 12 | mametch | まめっち | Mametchi | 固有: haed_happy/normal/sad_joint (表情3種) |

### キャラクター固有データ

FUN_80044d50 内で character ID (param_3, `param_1[0x7e]`) による特殊処理:
- **ID 3 (Peach)**: `chara_peach_pony_null_joint` + `peach_pony_dat` 読み込み
- **ID 4 (Koopa)**: `chara_koopa_fire_null_joint` 読み込み (火炎エフェクト)
- **ID 10 (Monster)**: `monster_cart_propeller_joint` + `monster_cart_wing_joint` (プロペラカート)
- **ID 11 (Waluigi)**: `waluigi_cart_Suspension_joint` (サスペンションギミック)
- **ID 12 (Mametch)**: `chara_mametch_haed_happy/normal/sad_joint` (3表情切替)、hand_goo/pa_joint なし

### 関連ポインタテーブル (DATA section, 0x803F59A0〜)

各テーブルは13エントリ (キャラクターID順) のポインタ配列:

| Address | テーブル名 | 説明 |
|---------|-----------|------|
| 0x803F59A0 | mario_shadow_dat | シャドウモデル |
| 0x803F59D4 | mario_sp_shadow_dat | 特殊シャドウモデル |
| 0x803FE918 | mario_cart_tire_fl_joint | タイヤ (前左) |
| 0x803FE94C | mario_cart_tire_fr_joint | タイヤ (前右) |
| 0x803FE980 | mario_cart_tire_rl_joint | タイヤ (後左) |
| 0x803FE9B4 | mario_cart_tire_rr_joint | タイヤ (後右) |
| 0x803FE9E8 | mario_cart_ground_fl_joint | 接地点 (前左) |
| 0x803FEA1C | mario_cart_ground_fr_joint | 接地点 (前右) |
| 0x803FEA50 | mario_cart_ground_rl_joint | 接地点 (後左) |
| 0x803FEA84 | mario_cart_ground_rr_joint | 接地点 (後右) |
| 0x803FEAB8 | mario_cart_engine_joint | エンジン |
| 0x803FEAEC | mario_cart_body_joint | ボディ |
| 0x803FEB20 | mario_cart_muffler_l_joint | マフラー (左) |
| 0x803FEB54 | mario_cart_muffler_r_joint | マフラー (右) |
| 0x803FEB88 | mario_cart_teresa_null_joint | テレサ |
| 0x803FEBBC | mario_cart_position_joint | ポジション |
| 0x803FEBF0 | mario_cart_info_null_joint | 情報 |
| 0x803FEC24 | chara_mario_r_hand_null_joint | 右手 |
| 0x803FEC58 | chara_mario_hand_goo_joint | グー (握り拳) |
| 0x803FEC8C | chara_mario_hand_pa_joint | パー (開き手) |
| 0x803FECC0 | chara_mario_hand_fin_joint | 指 |
| 0x803FECF4 | chara_mario_head_null_joint | 頭 |

---

## カップ・コース一覧

カードスキーマのRESULT-GP/GHOST/WIN-GHOSTフィールドのサフィックスで使われるカップコード。
g_courseId (1-based) → cup index (0-based) の変換は `cupIndex = g_courseId - 1`。

| Cup Index | Code | courseId | カップ名 (JP) | 英語名 | 確度 |
|-----------|------|---------|--------------|--------|------|
| 0 | MR | 1 | マリオカップ | Mario Cup | 確定 |
| 1 | DK | 2 | DKカップ | DK Cup | 確定 |
| 2 | WC | 3 | ワリオカップ | Wario Cup | 確定 (称号チェック関数で検証) |
| 3 | PC | 4 | パックマンカップ | Pac-Man Cup | 確定 |
| 4 | KP | 5 | クッパカップ | Bowser Cup | 確定 |
| 5 | RB | 6 | レインボーカップ | Rainbow Cup | 確定 |
| 6 | YI | 7 | ヨッシーカップ | Yoshi Cup | 確定 |
| 7 | DN | 8 | ワルイージカップ | Waluigi Cup | 確定 (称号チェック関数で検証) |

### カップコード検証方法

FUN_800a7600 (Race Extra チェック関数) が DAT_80326b04 のテーブル (8エントリ) で
courseId → カップ称号 TitleID のマッピングを定義:

```
DAT_80326b04[8] = {7, 1, 8, 2, 3, 4, 5, 6}
→ [ヨッシー, マリオ, ワルイージ, DK, ワリオ, パックマン, クッパ, レインボー]
→ TitleID 13-20 (表カップ制覇), 21-28 (裏), 29-36 (150cc表), 37-44 (150cc裏)
```

- courseId 3 → TitleID 17 (ワリオカップ制覇) → cup index 2 = WC = **Wario Cup** 確定
- courseId 8 → TitleID 15 (ワルイージカップ制覇) → cup index 7 = DN = **Waluigi Cup** 確定

### カップに関する注記

- **v0スキーマ (MKGP1互換)**: 6カップ = MR, DK, WC, PC, KP, RB
- **v2スキーマ (MKGP2)**: 8カップ = 上記 + YI, DN
- **DN コード名の由来**: ワルイージカップだが、コードは "DN"。開発初期に別のカップ (おそらくどんちゃん=Don-chan) 用に割り当てられたコードが、ワルイージカップに差し替えられた際にそのまま残ったと推定
- **card_pack の LAST_CLEAR_CUP チェック**: cup index 0-6 のみ確認。index 7 (DN) はスキップされている
  → 開発中の変更の痕跡。RESULT-GP自体にはDNカップのデータは正常に書き込まれる
- **ステージとカップの関係**: 各カップは1つのステージ (ロケーション) の4コースで構成される

### 称号に登場するカップ名 (TitleID 13-20, 21-28, 29-36, 37-44)

表カップ制覇 (TitleID 13-20):
ヨッシー(13), マリオ(14), ワルイージ(15), DK(16), ワリオ(17), パックマン(18), クッパ(19), レインボー(20)

裏カップ制覇 (TitleID 21-28): 同順

150ccカップ制覇 (TitleID 29-36): 同順

裏150ccカップ制覇 (TitleID 37-44): 同順

### 各カップ4コースの方向

GHOST[4] / WIN-GHOST[4] の4要素は各カップの4コースに対応。
TA_RECORD_DIR (bool) フラグがあることから、各コースに正逆方向がある。

---

## g_courseId 関連

| Address | 説明 |
|---------|------|
| g_courseId | コースID (1-8, 1-based) |
| g_isUraCourse | 裏コースフラグ (0=表, 1=裏) |
| g_ccClass (0x806d12cc) | CCクラス (0=50cc?, 1=100cc, 2=150cc) |
| DAT_806d1270 | コース方向? (0 or 1) |

RESULT-GP インデックス計算: `player[0x1BA + g_ccClass*8 + (g_courseId-1)]`
ゴーストインデックス計算: `player[0x48 + (g_courseId-1)*4 + direction*2 + isUra]`

---

### 次にやるべき作業
1. ~~**ビットパックスキーマ解析**~~ — **完了**: 73エントリ全解読、484ビットのビットレイアウト完成
2. **FUN_80088bb8 / FUN_80088a5c の詳細解析** — ビットパックエンコーダの実装 (ビット順序: MSB first? LSB first?)
3. ~~**デシリアライズ関数の特定**~~ — **完了**: card_unpack (0x801d43c4)
4. **FUN_80095388 / FUN_80094d80 の解析** — カードR/Wプロトコル (2種類ある理由)
5. **task[3] FUN_8008BF30 の解析** — CardPrinter関連の可能性
6. ~~**キャラクターテーブル解読**~~ — **完了**: 13キャラ (ID 0-12) の完全マッピング確定
7. **NAME文字エンコーディング** — FUN_801f8e08 の解析 (u16 → u8 変換ロジック)
8. ~~**称号割り当てロジック**~~ — **完了**: determine_title (0x800a70fc) + result_screen_update (0x8015f210) で全フロー解明
9. ~~**称号チェック関数テーブルの個別解析**~~ — **完了**: 全41関数デコンパイル+解析、Ghidraリネーム済み → `mkgp2_title_check_functions.md`
10. **FUN_800a8a28 の解析** — NameID決定ロジック (スコア+キャラクター→NameID)
11. ~~**WC/DN カップコードの特定**~~ — **完了**: FUN_800a7600 + DAT_80326b04 テーブルで確定 (WC=ワリオ, DN=ワルイージ)
