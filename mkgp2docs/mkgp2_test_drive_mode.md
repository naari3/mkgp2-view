> **非推奨**: このドキュメントは初期の誤った解釈（試乗モード）に基づいています。
> 正しい解析結果は `mkgp2_reverse_challenge.md` を参照してください。
> Ghidra上の関数名も TestDrive_Init → ReverseChallenge_Init に変更済みです。

# TestDrive_Init (0x800ad910) - 試乗モード初期化 [DEPRECATED]

## 概要

courseId=0xb のとき `MiniGame_CreateByCourseId` (0x80123050) から呼ばれる、試乗モードのコンストラクタ。
カート1台のみの走行デモで、プレイヤーがコースを試走する。ccVariant=0 (試乗パラメータ: speedCap=110, mue=0.4) のコンテキストで使用される。

## 呼び出し経路

```
RaceContext_InitDefaults (0x8009d6cc)
  └→ g_kartVariant = -1 (リセット)
       ↓
  UI でカート選択 → SetKartVariant(0) or SetKartVariant(2)
       ↓
  MiniGame_CreateByCourseId (0x80123050)
    └→ case 0xb: Alloc(0x4c) → TestDrive_Init(self)
```

## TestDriveState 構造体 (0x4C = 76 bytes)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| +0x00 | void* | vtable | PTR_PTR_80419e70 |
| +0x04 | float | timerScale | FLOAT_806d4f54 (0.0) で初期化 |
| +0x08 | void* | inputManager | Alloc(0x37c) → InputManager_Init |
| +0x0C | void* | camera | Alloc(0x2c) → Camera_Create(1000.0, 0.1, 256x256) |
| +0x10 | void* | renderTarget | RenderTarget_Create |
| +0x14 | void* | carObject | Alloc(0x118) → CarObject_Init |
| +0x18 | void* | lakituStart | Alloc(0x58) → LakituStart_Init (jyugemu_start.dat) |
| +0x1C | void* | scene3D | Alloc(0x3084) → Scene3D_Init (キャラモデル名使用) |
| +0x20 | void* | goalCamera | Alloc(0x8) → ChallengeGoalCamera_Init |
| +0x24 | void* | hud | Alloc(0x80) → HUD_Init |
| +0x28 | void* | cameraPath | Alloc(0x30) → カメラパス方向ベクトル(減算+正規化) |
| +0x2C | void* | cameraSpeedDeltas | Alloc(0x20) → キーフレーム間の差分速度テーブル |
| +0x30 | void* | bananaData | Alloc(0x10) → BananaPositionData_Init |
| +0x34 | float | cameraLerpT | 0.0 で初期化 |
| +0x38 | float | cameraSpeed | 0.0 で初期化 |
| +0x3C | float | cameraTargetSpeed | 0.0 で初期化 |
| +0x40 | byte | stateFlags | 0 で初期化 |
| +0x41 | byte | inputFlag1 | 0 で初期化 |
| +0x42 | byte | inputFlag2 | 0 で初期化 |
| +0x44 | int | frameCounter | 0 で初期化 |
| +0x48 | byte | isFinished | 0 で初期化 |

## 初期化フロー

1. **基底クラス初期化**: GameMode_BaseInit → vtable 設定 (PTR_PTR_80419e70)
2. **マネージャ初期化**:
   - DrawManager_GetOrCreate
   - DMAChannelManager_Init
   - TransitionEffect_GetOrCreate
   - SoundChannels_ClearAll
   - ItemObjectManager_Init
   - CourseObjectManager_Init
3. **入力**: InputManager_Init (0x37c bytes)
4. **レンダリング**: RenderTarget_Create → Camera_Create (FOV=1000.0, near=0.1, 256x256)
5. **カート生成**:
   - GetSpawnPosition → スポーン座標取得
   - GetCourseStartYaw + PI → 初期向き算出
   - CarObject_Init(spawnPos, orientation, charId, ccClass=0, kartVariant=0, isPlayer=1, ...)
   - KartMovement+0x238 = 1 (何らかのフラグ)
   - SetPlayerCarObject で g_playerCarObject に登録
6. **ジュゲム**: LakituStart_Init ("jyugemu_start.dat")
7. **3Dシーン**: Scene3D_Init (GetKartModelNameEntry でモデル名取得)
8. **ゴールカメラ**: ChallengeGoalCamera_Init
9. **カメラ配置**: Scene3D_GetCameraPos → Scene3D_SetupProjection
10. **HUD**: HUD_Init → HUD_RegisterOverlay(6), (0x11), (0x12), (0xe)
11. **タイマー**: MiniGame_GetTimerValue → FLOAT_806d132c に float 変換
12. **エフェクト**: PreloadEffectResources(1=smoke), (9=race_effects), (10=hit_effects)
13. **バナナ配置**: BananaPositionData_Init (IsUraCourseFromSubMode で表/裏判定)
14. **カメラパス**: 減算+正規化で方向ベクトル計算、キーフレーム差分テーブル生成
15. **天候**: WeatherSystem_Init (DAT_806d15c0 シングルトン)

## ccVariant との関係

- TestDrive_Init 自身は `SetKartVariant` を直接呼ばない
- ccVariant=0 は UI のカート選択フロー（`clFlowKart_HandleConfirm` 等）で事前に設定される
- CarObject_Init の引数 `kartVariant=0` で NORMAL カートとして生成
- `g_kartParamCcVariantTable` の ccVariant=0 エントリ (speedCap=110, mue=0.4) が適用される

## リネーム結果

### 関数リネーム
| アドレス | 旧名 | 新名 |
|----------|-------|------|
| 0x800ad910 | FUN_800ad910 | TestDrive_Init |
| 0x80123050 | MiniGame_CreateByCourseId | MiniGame_CreateByCourseId |
| 0x8002ce3c | FUN_8002ce3c | GameMode_BaseInit |
| 0x80079cdc | FUN_80079cdc | DrawManager_GetOrCreate |
| 0x80034d1c | FUN_80034d1c | SetResourceLoadingFlag |
| 0x800662b4 | FUN_800662b4 | DMAChannelManager_Init |
| 0x8007e8b0 | FUN_8007e8b0 | TransitionEffect_GetOrCreate |
| 0x8003ad4c | FUN_8003ad4c | SoundChannels_ClearAll |
| 0x800d81e0 | FUN_800d81e0 | ItemObjectManager_Init |
| 0x800afcd8 | FUN_800afcd8 | CourseObjectManager_Init |
| 0x800a0bdc | FUN_800a0bdc | GetCourseDataPtr |
| 0x802c60fc | FUN_802c60fc | RenderTarget_Create |
| 0x8003aacc | FUN_8003aacc | Camera_Create |
| 0x8002d74c | FUN_8002d74c | SetActiveCamera |
| 0x8009d5f8 | FUN_8009d5f8 | ProcessSystemTick |
| 0x8009c15c | FUN_8009c15c | GetRaceContextPtr |
| 0x8005fc48 | FUN_8005fc48 | SoundDriver_GetOrCreate |
| 0x800d63a0 | FUN_800d63a0 | GetSpawnPosition |
| 0x8009c634 | FUN_8009c634 | GetCourseStartYaw |
| 0x800d3bd4 | FUN_800d3bd4 | BuildOrientationFromYaw |
| 0x8004f640 | FUN_8004f640 | CarObject_SetPosition |
| 0x8004f13c | FUN_8004f13c | CarObject_GetKartMovement |
| 0x8009c168 | FUN_8009c168 | SetPlayerCarObject |
| 0x8003db40 | (既存) | LakituStart_Init |
| 0x8009c7fc | FUN_8009c7fc | GetKartModelNameEntry |
| 0x8009c178 | FUN_8009c178 | SetCourseScene3D |
| 0x800302ec | FUN_800302ec | Scene3D_GetCameraPos |
| 0x8003031c | FUN_8003031c | Scene3D_SetupProjection |
| 0x80253b14 | HudManager_SetupElement | HUD_RegisterOverlay |
| 0x80196d9c | FUN_80196d9c | BananaPositionData_Init |
| 0x80049f3c | FUN_80049f3c | CarObject_ResetPhysics |
| 0x8025e288 | FUN_8025e288 | Vec3_Subtract |
| 0x8025e2c8 | FUN_8025e2c8 | Vec3_Normalize |
| 0x8012318c | FUN_8012318c | IsUraCourseFromSubMode |
| 0x8009d6cc | FUN_8009d6cc | RaceContext_InitDefaults |

### 構造体
- **TestDriveState**: 21フィールド、73バイト (Ghidra上で作成済み)
