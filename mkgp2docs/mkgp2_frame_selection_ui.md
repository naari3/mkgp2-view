# MKGP2 フレーム選択UI と g_selectedFrameIdx 解析

## 概要

MKGP2 のレース/バトル開始前に表示される「フレーム選択メニュー」の仕組みと、
グローバル変数 `g_selectedFrameIdx` (0x806d12d0) のライフサイクルをまとめる。

この画面は NamCam で撮影した顔写真に被せる**フォトフレームの選択画面**である。

**結論: `g_selectedFrameIdx` はカードデータからロードされない。
ゲーム起動時に 0 に初期化され、プレイヤーがフレーム選択メニューで明示的に選択した場合のみ値がセットされる。**

---

## g_selectedFrameIdx の WRITE 箇所 (XREF 解析結果)

| # | 関数 | アドレス | 内容 |
|---|------|---------|------|
| 1 | InitializeFrameSelection | 0x801ba194 | `g_selectedFrameIdx = 0` |
| 2 | InitializeFrameSelection2 | 0x801ba2d0 | `g_selectedFrameIdx = 0` (直後にカード読み込み呼び出し) |
| 3 | FrameSelection_Confirm | 0x801c53a0 | `g_selectedFrameIdx = this->visualIdx` (UIでの選択値) |

カードの TITLE-PARAM フィールドにはフレーム番号が保存されているが、
card_unpack (0x801d43c4) はこれをフレーム称号デコード用パラメータとして使うだけで
`g_selectedFrameIdx` には書き込まない。

---

## DetermineTitle での参照 (CheckSelectedFrame)

Race テーブル index 13 のチェック関数。アセンブリ解析で全体が判明:

```c
// 0x800a7d20 - 0x800a7d58 (leaf function, 15命令)
bool CheckSelectedFrame() {
    int tier = g_selectedFrameIdx;  // lwz r5,-0x5a50(r13)
    if (tier <= 0) return false;       // cmpwi + ble
    g_titleParam = 0;                  // stw r0,-0x5948(r13)
    g_stagingTitleId = g_frameTitleIdTable[tier];  // lwz + stw
    return true;
}
```

- **選択値の実行時バリデーションは行わない**
- `g_selectedFrameIdx > 0` のみがゲート条件
- tier=0 のとき不成立 → 次のチェック関数 (Race[14]) へフォールスルー

---

## フレーム選択画面の関数構成 (Ghidra 命名済み)

| 関数 | アドレス | 役割 |
|------|---------|------|
| FrameSelection_Init | 0x801c4ea8 | 初期化 (コンストラクタ) |
| FrameSelection_Draw | 0x801c569c | 描画 (スプライト表示) |
| FrameSelection_Update | 0x801c5890 | 更新 (入力処理・描画計算・NamCam) |
| FrameSelection_Confirm | 0x801c53a0 | 確定処理 (this のみ、drawMgr引数は除去済み) |
| FrameSelection_GetFrameIdxFromVisualIdx | 0x801c3714 | ビジュアル順 -> フレーム番号変換 |

呼び出し元: `FUN_801b8ec0` (ケース 0x0D で `malloc(0x1D8)` して `Init` を呼ぶ)

vtable: `PTR_PTR_8049ac7c`

---

## FrameSelectionUI 構造体 (Size: 0x1D8)

最新の解析（Init, Update, Confirm, Draw）により、各フィールドの意味が判明。
0x50-0x77 の領域は Init が非スプライト値を書き込むため、現在の型定義に問題がある（後述の注記参照）。

| オフセット | 型 | 名前 | 内容 |
|:---:|:---|:---|:---|
| 0x00 | void* | vtable | `PTR_PTR_8049ac7c` |
| 0x04 | int | visualIdx | 現在の選択位置 (0-10) |
| 0x08 | int | field_0x08 | 不明 (Initで0) |
| 0x0C | byte | isSelectionDone | 選択完了フラグ (1:完了) |
| 0x0D | byte[13] | lockStatus | 選択可否フラグ (0:不可, 1:可) |
| 0x1C | int | counter | 汎用カウンタ (アニメーション用) |
| 0x20 | int | animIndex | アニメーション切り替え用インデックス |
| 0x24 | int | transitionTimer | 画面遷移用タイマー (0以外で入力不可) |
| 0x28 | int | confirmTimer | 確定演出用タイマー |
| 0x2C | int | state | メニューの遷移状態 |
| 0x30 | Sprite* | pSpriteBg | 背景スプライト |
| 0x34 | Sprite* | pSpriteTitle | タイトル/フレーム名スプライト |
| 0x38 | Sprite* | pSpriteInfo | 操作説明等スプライト |
| 0x3C | Sprite* | pSpriteCursor | カーソル/選択枠スプライト |
| 0x40 | int | field_0x40 | 不明 (Initで0) |
| 0x44 | int | field_0x44 | 不明 (Initで0) |
| 0x48 | Sprite* | pCursorLeft | 左選択矢印 |
| 0x4C | Sprite* | pCursorRight | 右選択矢印 |
| 0x50-0x77 | | **要再定義** | 下記「構造体レイアウト問題」参照 |
| 0x78-0xA7 | | **要再定義** | 下記「構造体レイアウト問題」参照 |
| 0xA8 | int[13] | unlockState | 解放状態 (0:ロック, 1:一部解放, 2:完全解放) |
| 0xDC | int[13] | array_0xDC | Initで0埋め (用途は Update/Draw で判明予定) |
| 0x110 | int[13] | array_0x110 | Initで0埋め |
| 0x144 | int[13] | array_0x144 | Initで0埋め |
| 0x178 | void* | pTexMgr | テクスチャマネージャ (Alloc(0x230)) |
| 0x17C+ | ResCtrl | resCtrl | リソースコントロール (ResCtrl_Init で初期化) |

### ResCtrl サブ構造体 (0x17C からの相対)

| 相対オフセット | 絶対 | 型 | 名前 | 内容 |
|:---:|:---:|:---|:---|:---|
| 0x00 | 0x17C | int | resourceId | リソースID (0x1D5C) |
| 0x10 | 0x18C | int | displayFlag | 表示フラグ (1) |
| 0x14 | 0x190 | float | posY | テクスチャ表示Y (330.0) |
| 0x18 | 0x194 | float | posX | テクスチャ表示X (276.0) |
| 0x1C | 0x198 | float | alpha | テクスチャ透明度 (1.0) |
| 0x2C | 0x1A8 | float | scaleX | テクスチャ拡大率X (0.5) |
| 0x30 | 0x1AC | float | scaleY | テクスチャ拡大率Y (0.5) |

### 構造体レイアウト問題 (0x50-0xA7)

現在 Ghidra では `Sprite*[11] pFrameSprites` (0x50) + `Sprite*[11] pFrameShadowSprites` (0x7C) と定義されているが、
Init が書き込む値を見ると **実際にはスプライトポインタ以外のデータが混在** している:

| オフセット | Initでの書込値 | 実際の意味 |
|:---:|:---|:---|
| 0x50-0x5F | null x4 | 不明 (未使用スロット or 後で上書き) |
| 0x60 | 0xFFFFFFFF (-1) | センチネル値 |
| 0x64 | byte 0 | フラグ/状態 |
| 0x65 | byte 0 | フラグ/状態 |
| 0x6C | 20.0 (float) | Z深度 |
| 0x70 | Scene3D* | 3Dシーンポインタ (Alloc 0x3084, "select_mario.dat") |
| 0x74 | SceneModel* | 3Dモデルポインタ (Alloc 0x28, "select_mario.dat") |

Update が pFrameSprites を反復する際にこれらの非スプライト値を読むと不整合が起きるため、
**実際の反復範囲やスプライト生成タイミングの確認が必要**。
Update/Draw の再解析により、0x50-0xA7 の正しいフィールド境界を確定させること。

---

## Sprite 構造体 (主要フィールド)

| オフセット | 型 | 名前 | 内容 |
|:---:|:---|:---|:---|
| 0x00 | byte | loopFlag | ループ再生フラグ |
| 0x08 | float | currentTime | 現在のアニメーション時刻 |
| 0x0C | float | posX | X座標 |
| 0x10 | float | posY | Y座標 |
| 0x14 | float | scaleX | Xスケール (非フォーカス0.6, フォーカス1.0) |
| 0x18 | float | scaleY | Yスケール |
| 0x20 | float | alpha | 透明度 (非フォーカス0.7, フォーカス1.0) |
| 0x28 | void* | pAnimData | アニメーションデータポインタ (→+0x14 = 総再生時間) |

---

## FrameSelection_Update 詳細解析 (0x801c5890)

3つのフェーズで構成される更新ループ。変数・関数はすべてGhidra上でリネーム済み。

### フェーズ1: 遷移アニメーション (`transitionTimer != 0`)
- `pSpriteBg`, `pSpriteTitle` のアニメーション進行 (`Sprite_AdvanceAnim`)
- `pSpriteBg2` が存在すれば 2x速度でアニメーション
- タイマーが0になったら `isSelectionDone = 1` にして `StartScreenFade(-57.0)` を呼ぶ

### フェーズ2: メイン入力・アニメーション (`transitionTimer == 0 && isSelectionDone == 0`)

#### NamCam画像読み込み
- `GetVBlankFlag()` が真かつ `pNamCamSlot->isAvailable` が非0のとき
- `NamCam_GetImageHandle` でハンドル取得 → `NamCam_LoadImage` で展開
- `pNamCamSlot->isProcessed = 1` で重複読み込み防止

#### ジョイスティック入力処理
- `g_pInputState` (0x806d1068) のオフセット+0x14 でY方向を読み取り
  - `+0x14 > 0`: 上方向 → `visualIdx++` (0→10ラップ)
  - `+0x14 < 0`: 下方向 → `visualIdx--` (10→0ラップ)
- `lockStatus[visualIdx] == 0` の場合、次の項目へ自動スキップ (最大11回ループ)

#### 選択変更時のスプライト更新
- 前の選択: `alpha = 0.7`, `scaleX/Y = 0.6` (非フォーカス)
- 新しい選択: `alpha = 1.0`, `scaleX/Y = 1.0` (フォーカス)
- タイトルスプライトの位置: `posX = 51.0 * visualIdx + 65.0`, `posY = 308.0`
- `PlayCursorMoveSE()` でSE#6再生

#### スプライトアニメーション
- 11個のフレームスプライト + シャドウスプライトを `FLOAT_DELTA_TIME` (≈0.0165) で進行
- タイトルスプライト、情報スプライト、カーソルスプライトも同様
- **点滅アニメーション**: `animIndex` が4→5→4→5と交互に切り替わり、`Sprite_SetAnimTime(0.0)` でリセット

### フェーズ3: 確定カウントダウン (`isSelectionDone == 1`)
- `SetSyncTarget(0x14)` で同期ターゲット設定
- `IsSyncReached()` が真になったら `confirmTimer = 8` でカウントダウン開始
- カウントダウン完了後:
  - `lockStatus[0xc] != 0` なら `ResetIOPorts()` 呼び出し
  - `state = 0x19` で次のゲーム状態へ遷移

### ヘルパー関数 (Ghidra リネーム済み)

| 関数 | アドレス | 役割 |
|------|---------|------|
| Sprite_AdvanceAnim | 0x801a0290 | スプライトアニメーション進行 (deltaTime, sprite) → 0/1 |
| Sprite_SetAnimTime | 0x8019ff18 | スプライトアニメーション時刻設定 (time, sprite) |
| GetVBlankFlag | 0x8002d040 | VBlankフラグ取得 (DAT_806d0f95) |
| PlayCursorMoveSE | 0x801b7c88 | SE#6再生 (カーソル移動音) |
| RumbleUpdate | 0x80169a14 | 振動コントローラ更新 |
| SetSyncTarget | 0x8009bb70 | 同期ターゲット値設定 |
| IsSyncReached | 0x80085dac | 同期到達判定 |
| ResetIOPorts | 0x80084ff0 | IOボード状態リセット |
| StartScreenFade | 0x801b7b40 | 画面フェード開始 |
| NamCam_GetImageHandle | 0x80048e0c | NamCam画像ハンドル取得 |
| NamCam_LoadImage | 0x80048bec | NamCam画像データ展開 |
| Sprite_SetupAnim | 0x801a0a30 | スプライトアニメーション初期設定 (sprite, animId, p3, p4) |
| Alloc | 0x8003b1fc | メモリ確保 (size) → void* |
| PlaySoundDSP | 0x801b3928 | DSPサウンド再生 (soundId) → bool |
| SetRumbleMode | 0x80169978 | 振動モード設定 (playerIdx, mode, param) |
| Sprite_Destroy | 0x801a0684 | スプライト破棄 (sprite, freeMemory) |
| GX_BeginDraw | 0x801f7b98 | GX描画セットアップ (NamCam描画前) |
| GX_ResetVertex | 0x80266798 | GX頂点バッファリセット |
| GetDisplayBufferIndex | 0x8009bf20 | 表示バッファインデックス取得 |
| GetDisplayContext | 0x8009bef4 | DisplayContext取得 (バッファIdx→ポインタ) |
| DrawNamCamImage | 0x8007cc9c | NamCam画像をGXテクスチャとして描画 (12引数) |
| GX_EndDraw | 0x801f7c18 | GX描画終了・後処理 |
| SoundMgr_Play | 0x80192884 | サウンドマネージャ再生 (soundId) |
| GetInputManager | (既存) | 入力マネージャ取得 |
| InputMgr_GetPlayer | (既存) | プレイヤー入力取得 (inputMgr, playerIdx) |

### FLOAT定数 (Ghidra リネーム済み)

| シンボル | アドレス | 値 | 用途 |
|---------|---------|-----|------|
| FLOAT_DELTA_TIME | 0x806d9c0c | 0.0165 | 1/60秒 (deltaTime) |
| FLOAT_FOCUSED_SCALE | 0x806d9c40 | 1.0 | フォーカス時スケール |
| FLOAT_UNFOCUSED_SCALE | 0x806d9c50 | 0.6 | 非フォーカス時スケール |
| FLOAT_UNFOCUSED_ALPHA | 0x806d9c4c | 0.7 | 非フォーカス時アルファ |
| FLOAT_TITLE_BASE_X | 0x806d9c54 | 65.0 | タイトルX基準位置 |
| FLOAT_TITLE_STEP_X | 0x806d9c58 | 51.0 | タイトルXステップ/idx |
| FLOAT_TITLE_POS_Y | 0x806d9c5c | 308.0 | タイトルY位置 |
| FLOAT_DELTA_TIME_2X | 0x806d9c44 | 0.033 | 2x deltaTime (Bg2用) |
| FLOAT_ANIM_RESET | 0x806d9c3c | 0.0 | アニメーション時刻リセット |
| FLOAT_FADE_OFFSET | 0x806d9c48 | -57.0 | フェード開始パラメータ |
| FLOAT_CURSOR_LEFT_X | 0x806d9c00 | 124.0 | 確定カーソル左X座標 |
| FLOAT_CURSOR_Y | 0x806d9c04 | 148.0 | 確定カーソルY座標 |
| FLOAT_CURSOR_RIGHT_X | 0x806d9c08 | 514.0 | 確定カーソル右X座標 |
| FLOAT_NAMCAM_LEFT | 0x806d9bf0 | 96.75 | NamCam描画矩形 左 |
| FLOAT_NAMCAM_TOP | 0x806d9bf4 | 32.375 | NamCam描画矩形 上 |
| FLOAT_NAMCAM_RIGHT | 0x806d9bf8 | 239.25 | NamCam描画矩形 右 |
| FLOAT_NAMCAM_BOTTOM | 0x806d9bfc | 207.625 | NamCam描画矩形 下 |

### グローバル変数

| シンボル | アドレス | 用途 |
|---------|---------|------|
| g_pInputState | 0x806d1068 | 入力状態ポインタ (+0x14 = joyY方向) |
| g_displayOffsetX | 0x80598aac | 表示オフセットX (NamCam座標補正用) |
| g_displayOffsetY | 0x80598aa8 | 表示オフセットY (NamCam座標補正用) |
| g_namCamFlag | 0x806cf125 | NamCam有効フラグ (Confirm時に1にセット) |
| g_confirmFlag | 0x806d1194 | 確定フラグ (Confirm時に1にセット) |

---

## FrameSelection_Confirm 詳細解析 (0x801c53a0)

フレーム選択の「確定」処理。`state == -1` のときのみ実行される。変数・関数はGhidra上でリネーム済み。

### フロー概要

1. **カウントダウンフェーズ**: `counter` を毎フレームインクリメント
   - **counter=360 (0x168)**: 左右カーソルスプライトを生成 (anim 0x5b)
     - 左カーソル: X=124.0, Y=148.0
     - 右カーソル: X=514.0, Y=148.0
   - **counter=480 (0x1E0)**: カウントダウンSE 1 (0x276)
   - **counter=540 (0x21C)**: カウントダウンSE 2 (0x277)
   - **counter=600 (0x258)**: カウントダウンSE 3 (0x278)

2. **確定判定**: 以下のいずれかで `isConfirmed = true`
   - カーソルアニメーション終了 (`Sprite_AdvanceAnim` が 0 を返す)
   - NamCam画像がロード済み (`pNamCamSlot->isProcessed != 0`) かつ確定ボタン押下 (vtable+0x28, button 0x2000)

3. **確定時の処理** (`isConfirmed == true`):
   - `SetRumbleMode(0, 1, 1)` で振動
   - カーソルスプライト破棄 (`Sprite_Destroy`)
   - NamCam画像がある場合:
     - `GX_BeginDraw` → `GX_ResetVertex` → `DrawNamCamImage` → `GX_EndDraw`
     - 描画矩形: g_displayOffset で補正した FLOAT_NAMCAM_* 座標を使用
     - `lockStatus[12] = 1`, `SoundMgr_Play(0x16)`, フラグ設定
   - `g_selectedFrameIdx = this->visualIdx` (フレーム選択値を確定)
   - ゲームモードに応じた状態遷移:
     - RACE → `state = 0x15`
     - BATTLE → `state = 0x17`
   - `transitionTimer = 0x50` (80フレーム) で遷移タイマー開始

### デコンパイラ注意事項
- `counter++` が `pSprite = (Sprite*)counter; counter = (int)&pSprite->field_0x1;` と表示される
  - PPCの `addi r0, r0, 1` をSprite構造体のフィールドオフセットとして誤解釈
  - 実際は単なる `counter = counter + 1`
- レジスタ再利用により、一部のローカル変数名が意味と一致しない箇所がある
  - 例: `animResult` が `GetInputManager` の戻り値に割り当てられる等

---

## FrameSelection_Init 詳細解析 (0x801c4ea8)

UI 初期化のコンストラクタ。リソースプリロード、3Dシーン構築、フレームアンロック判定、
テクスチャマネージャ設定、入力状態リセットを行う。変数・関数はすべてGhidra上でリネーム済み。

### フロー概要

#### 1. 初期設定とリソースプリロード
- `FUN_801ba194()` (InitializeFrameSelection) で `g_selectedFrameIdx = 0`
- vtable = `PTR_PTR_8049ac7c`
- `ResCtrl_Init(&this->resCtrl)` でリソースコントロール初期化
- `SpriteSystem_EnsureInit()`, `SetSyncTarget(0xD)`, `SetCourseParams(7,0,0,0)`
- 固定リソース ID を `PreloadResource()` で登録:
  - 0x167F, 0x169B, 0x16AC, 0x16AE, 0x16AD, 0x16AF (UIスプライト)
  - 0x16A8, 0x1D55, 0x1D3E, 0x186C, 0x16B0, 0x16CC, 0x1D5C (追加リソース)
- `FrameParam_ARRAY_8049aad0[0..12]` の `iconId1`, `iconId2` を各フレームごとにプリロード (計26個)

#### 2. 構造体フィールド初期化
- `FrameParam[0..12].field_0x8 = 0` (ランタイムカウンタ全リセット)
- `counter`, `animIndex`, `transitionTimer`, `confirmTimer`, `state` = 0
- `pSpriteBg`, `pSpriteTitle`, `pSpriteInfo`, `pSpriteCursor` = null
- `visualIdx = 0`, `field_0x08 = 0`, `isSelectionDone = 0`
- オフセット 0x50-0x77: 各種初期値 (z-depth=20.0, sentinel=-1, Scene3D/SceneModel=null)

#### 3. カーソル矢印スプライト作成
- 左矢印: `Alloc(0x30)` → `Sprite_CreateWithParams(82.0, 135.0, -0.7, 0.7, 0.0, 1.0, sprite, 0x9C, 1)`
  - scaleX=-0.7 で左右反転
- 右矢印: `Sprite_CreateWithParams(182.0, 135.0, 0.7, 0.7, 0.0, 1.0, sprite, 0x9C, 1)`

#### 4. テクスチャマネージャ確保
- `Alloc(0x230)` → `TexMgr_Init()` → `this->pTexMgr` に保存

#### 5. 前回選択フレームの復元
- `g_lastSelectedFrame` (0x805d2738) を参照
- 13個のフレームを走査し、`GetFrameIdxFromVisualIdx` でマッチした `visualIdx` を復元
- カーソル矢印の位置を `g_frameCursorPosTable` のXY座標に合わせる
  - 右矢印は `posX += FLOAT_ARROW_RIGHT_OFFSET` (100.0) でオフセット
  - `Sprite.field_0x4 = 5` (アニメーションフレーム番号)

#### 6. フレームアンロック判定
- `GetPlayerStat_0x1b4(&g_playerData)` でプレイヤー統計値を取得
- 各フレームに対して:
  - `stat >= g_frameUnlockThresholds_Full[frameIdx]` → `unlockState[frameIdx] = 2` (完全解放)
  - `stat >= g_frameUnlockThresholds_Partial[frameIdx]` → `unlockState[frameIdx] = 1` (一部解放)
  - それ以外 → `unlockState[frameIdx] = 0` (ロック)
  - `IsCardValid() == false` の場合、強制的に `unlockState = 1` (一部解放にダウングレード)
- `array_0xDC`, `array_0x110`, `array_0x144` は全て0埋め

#### 7. 3Dシーン構築
- `Alloc(0x3084)` → `Scene3D_Init(ptr, "select_mario.dat")` → オフセット 0x70 に保存
- `Alloc(0x28)` → `SceneModel_Init(ptr, "select_mario.dat", 1)` → オフセット 0x74 に保存

#### 8. テクスチャ表示設定
- `TexMgr_Init(localTexMgr)` でスタックローカルのTexMgr (572 bytes) を初期化
- 選択中フレームのテクスチャを取得: `GetTextureByIdAndLang(g_frameTextureIdTable[frameIdx] + 0x95, -1)`
- `TexMgr_SetTexture()` → `TexMgr_Apply()` で適用
- ResCtrl 設定: resourceId=0x1D5C, displayFlag=1, posY=330.0, posX=276.0, alpha=1.0, scaleX/Y=0.5

#### 9. 入力・画面セットアップ
- `g_pInputState` のフラグ設定 (byte[4]=1, int[0x18]=0, int[0x1C]=0)
- `SetScreenBrightness(-1.0)` で画面を完全暗転 (フェードイン前)
- `InitBGM()`, `SetupBgSprites(1)`, `InitRumbleController(1)`

### Init ヘルパー関数 (Ghidra リネーム済み)

| 関数 | アドレス | 役割 |
|------|---------|------|
| PreloadResource | 0x80120d80 | リソース参照カウント/プリロード |
| IsCardValid | 0x8023f65c | カード/ゲーム状態チェック (false→一部解放強制) |
| ResCtrl_Init | 0x80121eac | リソースコントロール初期化 |
| SpriteSystem_EnsureInit | 0x801a16f0 | スプライトシステム初期化 (DAT_806cfd08が-1なら実行) |
| SetCourseParams | 0x8009cbfc | コースパラメータ設定 (g_courseId=7等) |
| Scene3D_Init | 0x80030bdc | 3Dシーン初期化 (0x3084, "select_mario.dat") |
| SceneModel_Init | 0x80076494 | 3Dモデル初期化 (0x28, "select_mario.dat") |
| SetScreenBrightness | 0x801b8154 | 画面輝度設定 (-1.0=完全暗転) |
| InitBGM | 0x801b833c | BGMサウンドシステム初期化 |
| SetupBgSprites | 0x801b8a80 | 背景スプライト設定 (ゲームモード別: 0x197/0x198/0x199) |
| InitRumbleController | 0x80169d60 | 振動コントローラ初期化 (Alloc 0x20) |
| GetTextureByIdAndLang | 0x8013c2d4 | テクスチャ取得 (PTR_DAT_8048e548テーブルから) |
| Sprite_CreateWithParams | 0x801a06f8 | スプライト生成 (posX, posY, scaleX, scaleY, initTime, alpha, sprite, animId, loop) |
| TexMgr_Init | 0x801f93f8 | テクスチャマネージャ初期化 (0x230 byte struct) |
| TexMgr_SetTexture | 0x801f91ec | テクスチャ設定 |
| TexMgr_Apply | 0x801f947c | テクスチャ適用 |

### Init FLOAT定数 (Ghidra リネーム済み)

| シンボル | アドレス | 値 | 用途 |
|---------|---------|-----|------|
| FLOAT_INIT_Z_DEPTH | 0x806d9bd4 | 20.0 | Z深度 (0x6C) |
| FLOAT_ARROW_LEFT_X | 0x806d9bd8 | 82.0 | 左矢印X座標 |
| FLOAT_ARROW_Y | 0x806d9bdc | 135.0 | 矢印Y座標 |
| FLOAT_ARROW_LEFT_SCALE_X | 0x806d9be0 | -0.7 | 左矢印Xスケール (反転) |
| FLOAT_ARROW_SCALE | 0x806d9bb8 | 0.7 | 矢印スケール |
| FLOAT_ARROW_RIGHT_X | 0x806d9be4 | 182.0 | 右矢印X座標 |
| FLOAT_ARROW_RIGHT_OFFSET | 0x806d9b70 | 100.0 | 右矢印オフセット |
| FLOAT_TEX_SCALE | 0x806d9be8 | 0.5 | テクスチャスケール |
| FLOAT_TEX_POS_X | 0x806d9ba4 | 276.0 | テクスチャX座標 |
| FLOAT_TEX_POS_Y | 0x806d9bec | 330.0 | テクスチャY座標 |
| FLOAT_BRIGHTNESS_DARK | 0x806d9b94 | -1.0 | 完全暗転 |
| FLOAT_SPRITE_INIT_TIME | 0x806d9b5c | 0.0 | スプライト初期アニメ時刻 |
| FLOAT_FULL_ALPHA | 0x806d9b58 | 1.0 | フルアルファ |

### Init データテーブル (Ghidra リネーム済み)

| シンボル | アドレス | サイズ | 内容 |
|---------|---------|------|------|
| FrameParam_ARRAY_8049aad0 | 0x8049aad0 | 312 (24*13) | フレームパラメータ配列 |
| g_frameCursorPosTable | 0x8049ac08 | 104 (8*13) | カーソル座標テーブル (float XY pairs) |
| g_frameTextureIdTable | 0x8039aea4 | 52 (4*13) | テクスチャリソースIDテーブル |
| g_lastSelectedFrame | 0x805d2738 | 4 | 前回選択フレームインデックス (BSS) |
| g_frameUnlockThresholds_Partial | 0x8039ae3c | 52 (4*13) | 一部解放閾値テーブル |
| g_frameUnlockThresholds_Full | 0x8039ae70 | 52 (4*13) | 完全解放閾値テーブル |

### FrameParam 構造体 (Size: 0x18 / 24 bytes)

| オフセット | 型 | 名前 | 内容 |
|:---:|:---|:---|:---|
| 0x00 | u16 (BE) | iconId1 | フレームアイコンリソースID 1 |
| 0x02 | u16 (BE) | iconId2 | フレームアイコンリソースID 2 |
| 0x04 | u16 (BE) | iconId3 | フレームアイコンリソースID 3 (未プリロード) |
| 0x06 | u16 | padding | パディング |
| 0x08 | int | field_0x8 | ランタイムカウンタ/状態 (Initで全エントリ0リセット) |
| 0x0C-0x17 | | reserved | 不明 (要解析) |

---

## NamCam (画像表示) システム

フレーム選択画面には、特定の画像を読み込んで表示する「NamCam」処理が組み込まれている。

### NamCamSlot 構造体 (Size: 0x10)
| オフセット | 型 | 名前 | 内容 |
|:---:|:---|:---|:---|
| 0x00 | byte | isAvailable | 画像データが存在するか |
| 0x01 | byte | isProcessed | 処理済みフラグ |
| 0x04 | int | imageId | 読み込む画像のリソースID |

### 画像読み込みフロー
1. `this->pNamCamSlot` の `isAvailable` が 1 かつ `isProcessed` が 0 であることを確認。
2. `NamCam_GetImageHandle` (0x80048e0c) を呼び出し、画像ハンドルを取得。
3. `imgParams` 構造体（12バイト）を準備し、`NamCam_LoadImage` (0x80048bec) で画像データを展開・表示。
4. `isProcessed = 1` をセットして重複読み込みを防止。

---

## フレームアンロック判定 (FrameSelection_Init)

```c
int coinMilage = GetCoinMilage(&g_playerData); // player + 0x1b4 = COIN_MILAGE

for (int i = 0; i < 13; i++) {
    this->array_0xDC[i] = 0;   // 0xDC+i*4
    this->array_0x110[i] = 0;  // 0x110+i*4
    this->array_0x144[i] = 0;  // 0x144+i*4

    int frameIdx = GetFrameIdxFromVisualIdx(this, i);
    // 閾値テーブルは 0x8039ae70 (Full), 0x8039ae3c (Partial)
    if (coinMilage >= g_frameUnlockThresholds_Full[frameIdx]) {
        this->unlockState[frameIdx] = 2;  // 完全解放 (0xA8+frameIdx*4)
    } else if (coinMilage >= g_frameUnlockThresholds_Partial[frameIdx]) {
        this->unlockState[frameIdx] = 1;  // 一部解放
    } else {
        this->unlockState[frameIdx] = 0;  // ロック
    }

    if (!IsCardValid()) {
        this->unlockState[frameIdx] = 1;  // カード無効時は一部解放に強制
    }
}
```

- **アンロック状態の格納先**: `unlockState[13]` はオフセット **0xA8** (旧ドキュメントの 0xDC は誤り)
  - デコンパイラ上は `pFrameShadowSprites[frameIdx + 0xb]` と表示される (0x7C + (frameIdx+11)*4 = 0xA8 + frameIdx*4)
- **統計値 (Player+0x1b4) = COIN_MILAGE**: カードの `COIN_MILAGE_TAG` フィールドに対応。レース/バトル後に獲得コインが加算される累積値。
- **等差数列による閾値**:
    - 部分解放 = 30 + 120 x n (n=0..12)
    - 完全解放 = 部分解放 + 1560
    - ステップ値 120 はコインマイレージの蓄積単位に対応。
- **IsCardValid()**: カード/ゲーム状態が無効 (戻り値0) の場合、完全解放でも一部解放にダウングレードされる。

---

## COIN_MILAGE システム (player+0x1b4)

### 概要

フレーム解放判定に使用される `player+0x1b4` は、カードの **COIN_MILAGE** フィールドに対応する。
レース/バトルを完了するたびにコインが加算され、累積値に応じてフレームやアイテムがアンロックされる。

### カードデータとの対応

- カードタグ名: `COIN_MILAGE_TAG` (文字列アドレス: 0x8039da14)
- card_unpack (0x801d43c4): `coinMilage = bitpack_read("COIN_MILAGE_TAG") & 0xFFFF`
- card_pack (0x801d5244): `bitpack_write(coinMilage, "COIN_MILAGE_TAG")`
- u16としてカードに保存されるが、player struct内ではu32フィールドとして扱われる

### 関数一覧

| 関数 | アドレス | 役割 |
|------|---------|------|
| GetCoinMilage | 0x801d5d68 | `return player->coinMilage` (2命令: lwz + blr) |
| AddCoinMilage | 0x801d5d70 | `player->coinMilage += amount; if (coinMilage > 99999) coinMilage = 99999;` |
| CalcEarnedCoins | 0x80216568 | レース/バトル後の獲得コイン計算 |
| GetEarnedCoins | 0x80216204 | `return g_earnedCoins` (計算済みの獲得コイン数) |
| AddCoinsFromExtraStage | 0x80216540 | エクストラステージ分のコイン加算 (courseId >= 9) |
| CheckCoinMilestones | 0x8021623c | コインマイレージのマイルストーンテーブルを参照しアイテムアンロック |
| CoinResultScene_Init | 0x80223798 | リザルト画面初期化 (コイン加算 + 3Dコイン演出) |

### 獲得コイン計算 (CalcEarnedCoins)

レース/バトル後に呼ばれ、以下のコンポーネントを合算:

```
g_earnedCoins = baseCoins + winBonus + positionBonus + battleBonus + extraBattleBonus
```

| 変数 | アドレス | 内容 |
|------|---------|------|
| g_earnedCoins | 0x806d1a4c | 最終獲得コイン数 (合算結果) |
| baseCoins | 0x806d1a50 | レース結果ベースコイン (result+0x18) |
| winBonus | 0x806d0438 | 1位ボーナス (ccClass別: DAT_803c4358[ccClass]) |
| positionBonus | 0x806d043c | 順位ボーナス (RACE: DAT_803c4364[ccClass][rank], TIME_ATTACK: DAT_803c43a4[ccClass][rank]) |
| battleBonus | 0x806d0444 | バトル成績ボーナス (タイム差に基づく段階評価) |
| extraBattleBonus | 0x806d0448 | バトル追加ボーナス (特定条件で付与) |

- 150cc裏コースでフレーム選択連勝中 (DAT_806d1acc > 0) の場合、positionBonusはccClass=3の特別テーブルを使用
- 上限値: **99999** (AddCoinMilageでキャップ)

### マイルストーンアイテムアンロック (CheckCoinMilestones)

- マイルストーンテーブル: `DAT_80679748` (最大93エントリ、3int/エントリ = {閾値, 種別, データ})
- コインマイレージが閾値を超えるたびに、種別=1のエントリでアイテムがアンロックされる
- アンロック処理: `FUN_801d5f74(playerData, itemIdx, 1)` (アイテムフラグON)
- デバッグ文字列: "Get item error  Milage coin i" (0x803c4b4a)

---

## ビジュアルインデックス → フレームインデックス マッピング (FUN_801c3714)

13個のビジュアル位置を13個のフレームインデックスに変換する。
選択可能なのはビジュアル 0-10 の **11項目** のみ。

| ビジュアル | フレーム | TitleID | 称号名 | 解放閾値 (部分/完全) | 選択可否 |
|:---:|:---:|:---:|:---|:---:|:---:|
| 0 | 0 | 50 | (デフォルト) | 30 / 1590 | 可 |
| 1 | 1 | 141 | チャンバラ | 1470 / 3030 | 可 |
| 2 | 8 | 143 | 萌え系 | 990 / 2550 | 可 |
| 3 | 11 | 52 | (不明) | 510 / 2070 | 可 |
| 4 | 3 | 140 | ダンシング | 150 / 1710 | 可 |
| 5 | 7 | 145 | かわいい | 630 / 2190 | 可 |
| 6 | 10 | 149 | 危機一髪 | 1110 / 2670 | 可 |
| 7 | 12 | 46 | (不明) | 870 / 2430 | 可 |
| 8 | 9 | 147 | マジカル | 390 / 1950 | 可 |
| 9 | 5 | 144 | 大海原の | 1350 / 2910 | 可 |
| 10 | 2 | 148 | 極悪 | 1230 / 2790 | 可 |
| (11) | 6 | 146 | 戦国の | 270 / 1830 | **不可** |
| (12) | 4 | 142 | 艶やかな | 750 / 2310 | **不可** |

---

## 称号テーブル (g_frameTitleIdTable)

| フレーム | raw TitleID | 最終 TitleID (raw+45) |
|:---:|:---:|:---:|
| 0 | 5 | 50 |
| 1 | 96 | 141 |
| 2 | 103 | 148 |
| 3 | 95 | 140 |
| 4 | 97 | 142 |
| 5 | 99 | 144 |
| 6 | 101 | 146 |
| 7 | 100 | 145 |
| 8 | 98 | 143 |
| 9 | 102 | 147 |
| 10 | 104 | 149 |
| 11 | 7 | 52 |
| 12 | 1 | 46 |

---

## ライフサイクルフロー

```
[ゲーム起動]
    ↓
InitializeFrameSelection: g_selectedFrameIdx = 0
    ↓
[カード読み込み (card_unpack)]
    → player 構造体にデータ展開
    → COIN_MILAGE (player+0x1b4) をカードから復元
    → g_selectedFrameIdx は 0 のまま
    ↓
[フレーム選択画面 (FrameSelection_Init)]
    → GetCoinMilage() で COIN_MILAGE を読み取り
    → 閾値テーブル (部分解放/完全解放) と比較してアンロック状態を決定
    → 前回選択フレームを初期位置に復元
    ↓
[入力更新 (FrameSelection_Update)]
    → ジョイスティックで 0-10 を選択
    → 必要に応じて NamCam 画像を読み込み
    ↓
[ボタン確定 (FrameSelection_Confirm)]
    → g_selectedFrameIdx = 選択値 (0-10)
    ↓
[レース開始 → DetermineTitle]
    → CheckSelectedFrame: g_selectedFrameIdx > 0 ?
    → Yes: TitleID = g_frameTitleIdTable[tier]
    ↓
[レース/バトル終了 → CoinResultScene_Init]
    → CalcEarnedCoins() で獲得コイン計算
    → CheckCoinMilestones() でマイルストーン到達アイテムアンロック
    → AddCoinMilage() で COIN_MILAGE に加算 (上限 99999)
    → mario_coin_dat の3Dコイン演出
    ↓
[カード保存 (card_pack)]
    → COIN_MILAGE を COIN_MILAGE_TAG としてカードに書き戻し
```
