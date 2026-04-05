# MiniGame_CoinChallenge (courseId=15, ccVariant=6)

## 概要

コイン集めミニゲーム。制限時間内に目標枚数のコインを収集するチャレンジモード。

## 関数一覧

| Address | Name | 役割 |
|---------|------|------|
| 0x80214db8 | MiniGame_CoinChallenge_Init | 初期化 (構造体0x48バイト) |
| 0x80214594 | MiniGame_CoinChallenge_Update | 毎フレーム更新 |
| 0x802143e4 | MiniGame_CoinChallenge_Destroy | 終了処理・リソース解放 |

### 関連関数 (Init から呼ばれる)

| Address | Name | 役割 |
|---------|------|------|
| 0x80123050 | MiniGame_CreateByCourseId | ミニゲームディスパッチャ (courseId switch) |
| 0x801231fc | MiniGame_GetTimerValue | 制限時間取得 (カード有無でテーブル切替) |
| 0x80122efc | MiniGame_GetCourseData | コース別データ取得 (目標コイン数等) |
| 0x802151c8 | ScoreDisplay_Init | スコア表示UI初期化 (数字スプライト) |
| 0x80124cf4 | ChallengeGoalCamera_Init | ゴール演出用カメラ初期化 |
| 0x8025421c | HudManager_Init | HUD管理初期化 |
| 0x80253b14 | HudManager_SetupElement | HUD要素登録 |
| 0x8003db40 | LakituStart_Init | ジュゲム(Lakitu)スタート演出初期化 |
| 0x80138ce4 | CourseObjects_Activate | コース上オブジェクト有効化 |
| 0x800def74 | PreloadEffectResources | エフェクトリソースプリロード |
| 0x8016c730 | WeatherSystem_Init | 天候システム初期化 |
| 0x80122bb0 | MiniGame_AccumulateBoost | ブーストタイマー蓄積 |
| 0x8011f094 | SpriteLayer_SetResource | スプライトレイヤーにリソースID設定 |

## 構造体: MiniGame_CoinChallenge (0x48 = 72 bytes)

| Offset | Type | Name | 説明 |
|--------|------|------|------|
| 0x00 | ptr | vtable | PTR_PTR_804eda98 |
| 0x04 | float | boostAccumulator | ブースト蓄積値 |
| 0x08 | ptr | carObject | CarObject* (0x118 bytes) |
| 0x0C | ptr | lakituStartObj | ジュゲムスタート演出 (0x58 bytes) |
| 0x10 | ptr | subObject | サブオブジェクト (0x37C bytes) |
| 0x14 | ptr | scene3D | 3Dシーン (0x3084 bytes) |
| 0x18 | ptr | goalCameraObj | ゴール演出カメラ (8 bytes) |
| 0x1C | ptr | cameraRenderer | カメラレンダラ (0x2C bytes) |
| 0x20 | ptr | resourceLoader | リソースローダ (RenderTarget) |
| 0x24 | int | padding24 | (未使用) |
| 0x28 | ptr | hudManager | HUD管理 (0x80 bytes) |
| 0x2C | ptr | scoreDisplay | スコア表示 (0x2C bytes, 数字スプライト9個) |
| 0x30 | byte | isInitialized | 初期化完了フラグ |
| 0x31 | byte | goalReached | 目標達成フラグ |
| 0x34 | int | postGoalCountdown | ゴール後のカウントダウン (初期値0x1E=30F) |
| 0x38 | int | resultPhaseCounter | 結果表示フェーズカウンタ |
| 0x3C | int | goalCoinCount | 目標コイン数 (MiniGame_GetCourseDataから) |
| 0x40 | int | bonusCoinLaps | ボーナスコイン周回数 (カード有効時) |
| 0x44 | int | bonusCoinInterval | ボーナスコイン間隔 (FUN_80122e80から) |

## vtable (PTR_PTR_804eda98)

| Offset | Address | Name |
|--------|---------|------|
| 0x00 | 0x806d0408 | (データポインタ) |
| 0x04 | 0x00000000 | (null) |
| 0x08 | 0x80214594 | MiniGame_CoinChallenge_Update |
| 0x0C | 0x802143e4 | MiniGame_CoinChallenge_Destroy |

## CarObject_Init 引数

```
CarObject_Init(
  startPosX, startPosZ, startYaw,
  allocatedMem,    // 0x118 bytes
  displayBufIdx,
  g_characterId,   // charId
  0,               // ccClass (固定)
  0,               // kartVariant (固定)
  1,               // isPlayer
  6,               // ccVariant → g_kartParamCcVariantTable[18] (speed=300, mue=0.4)
  0,               // param_11
  0                // param_12
);
```

## KartMovement 設定

Init後に以下を設定:
- `CarObject+0xFD` = 0, `CarObject+0xFE` = 0 (ランキング/AI関連フラグ無効化)
- `KartMovement+0x238` = 1 (不明、物理関連)
- `KartMovement+0x2D0` = 1 (コイン収集有効化フラグ)

## ゲームフロー (Update)

1. **通常フェーズ** (DAT_806d1289 == 0):
   - 入力取得 → ブースト蓄積判定 → CarObject更新
   - コイン数を KartMovement+0x2CC から読み取り
   - スコア表示更新 (2桁: 10の位+1の位)
   - カード有効時: bonusCoinInterval毎にbonusCoinLaps++
   - 目標達成判定: coinCount >= goalCoinCount → goalReached=1, postGoalCountdown=0x1E

2. **タイマーカウントダウン**:
   - FLOAT_806d132c を毎フレーム減算
   - 0到達 → DAT_806d1289=1 (時間切れ)

3. **終了フェーズ** (DAT_806d1289 == 1):
   - postGoalCountdown をデクリメント
   - 0到達 → SetCoinCount(bonusCoinLaps), 結果表示
   - goalReached に基づいて成功/失敗演出を分岐

## ミニゲームディスパッチャ (MiniGame_CreateByCourseId)

| courseId | Alloc Size | Init Function | ccVariant |
|----------|-----------|---------------|-----------|
| 9 | 0x6C | FUN_80133398 | 3 |
| 10 | 0x48 | FUN_80129938 | 1 (TT) |
| 11 | 0x4C | FUN_800ad910 | 0 (試乗) |
| 12 | 0x44 | FUN_80134ff0 | 3 |
| 13 | 0xC4 | FUN_8012fc58 | ? |
| 14 | 0x3C | FUN_80136ea0 | ? |
| **15** | **0x48** | **MiniGame_CoinChallenge_Init** | **6** |
| 16 | 0x8C | CoinRunMode_Init | 7 |
