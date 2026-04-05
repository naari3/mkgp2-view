# TimeTrial Mode 解析 (FUN_80129938 → NokoNokoChallenge_Init)

**注意**: 内部名は "TimeTrial" だが、実態はノコノコ退治ミニゲーム。
詳細な解析は `mkgp2_nokonoko_challenge.md` を参照。

## 概要

NokoNokoChallenge_Init (0x80129938) は courseId=10 のときに呼ばれるノコノコチャレンジの初期化関数。
SpecialMode_Create (0x80123050) 内の switch 文から、0x48バイトの TimeTrialScene オブジェクトを Alloc して呼び出される。

## ccVariant=1 の経路

```
TimeTrial_Init
  → CarObject_Init(... ccClass=0, kartVariant=0, isPlayer=1, ccVariant=1, ...)
    → GetKartParamBlock(charId, ccClass=0, kartVariant=0, ccVariant=1)
      → ccVariant >= 0 なので g_kartParamCcVariantTable[1*3 + 0] = g_kartParamCcVariantTable[3]
        → 0x80384578 (TT用パラメータ: speedCap=300, mue=0.21)
```

- ccVariant=1 は**タイムトライアル専用**。他のモードでは使われない
- GetKartParamBlock は CarObject_Init からのみ呼ばれ、ccVariant>=0 を渡すのは特殊モード初期化のみ
- mue=0.21 は通常レースの Profile A (低グリップ) と同等

## TimeTrialScene 構造体 (0x48 = 72 bytes)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| 0x00 | void* | vtable | PTR_PTR_8048e338 (TT固有vtable) |
| 0x04 | float | initFloat | 初期浮動小数値 |
| 0x08 | void* | carObject | CarObject* (プレイヤーカート) |
| 0x0C | void* | signalLight | ジュゲム信号灯 (jyugemu_start) |
| 0x10 | void* | courseTrack | コーストラックオブジェクト |
| 0x14 | void* | scene3d | Scene3D* (3Dシーン) |
| 0x18 | void* | goalCamera | ゴールカメラ (cam_challenge_goal) |
| 0x1C | void* | cameraObj | カメラオブジェクト |
| 0x20 | void* | displayBuf | レンダーターゲット |
| 0x28 | void* | hudOverlay | HUDオーバーレイ |
| 0x2C | void* | hudResManager | HUDリソースマネージャ (0x2c8 bytes) |
| 0x30 | void* | mapWaterEffect | マップ水面エフェクト |
| 0x34 | byte | initialized | 初期化済みフラグ |
| 0x36 | byte | timeUpFlag | タイムアップフラグ |
| 0x38 | int | countdownTimer | カウントダウンフレーム |
| 0x3C | int | resultPhase | リザルト遷移フェーズ |
| 0x40 | int | timeLimit | 制限時間 (ラップ数 or 秒数) |
| 0x44 | void* | dkCoinManager | DKコイン管理 (0x8a0 bytes, 6スロット) |

## vtable (PTR_PTR_8048e338)

| Index | アドレス | 関数名 | 説明 |
|-------|----------|--------|------|
| 2 | 0x801289b8 | TimeTrial_Update | 毎フレーム更新 |
| 3 | 0x801287f8 | TimeTrial_Render | 描画 |
| 4 | 0x80129674 | TimeTrial_Destroy | デストラクタ |

## 呼び出し元

```
SpecialMode_Create (0x80123050)
  switch(g_courseId) {
    case 10: TimeTrial_Init(Alloc(0x48))
    ...
  }
```

SpecialMode_Create は関数ポインタ経由で呼ばれる (直接xrefなし)。

## 初期化フロー

1. GameMode_BaseInit → 基底vtable設定
2. TT固有vtable上書き (PTR_PTR_8048e338)
3. サブシステム初期化: DrawManager, DMAChannel, Sound, ItemObject, CourseObject
4. コースモデル読み込み (FUN_80048754)
5. カメラ作成 (Camera_Create, 256x256)
6. RankingTable_Init
7. GetStartPosition → 開始位置取得
8. **CarObject_Init (ccVariant=1)** → プレイヤーカート生成
9. KartMovement 設定: +0x238=1, +0x2d0=1
10. ジュゲム信号灯 (LakituStart_Init)
11. Scene3D_Init (キャラモデル)
12. GoalCamera_Init (チャレンジゴール)
13. MapWaterEffect_Init (水面エフェクト)
14. HUD_Init + オーバーレイ登録 (6=タイマー, 0x11/0x12=ラップ, 0xe=速度)
15. GetTimeLimitForCourse → 制限時間取得 → FLOAT_806d132c
16. PreloadEffectResources (smoke=2, items=9, hit=10)
17. GetBestTimeRecord → ベストタイム取得
18. NokoNoko_Init → BGM・障害物
19. HUDリソースブロック作成 (タイマー数字スプライト)
20. DKコインマネージャ作成 (0x8a0 bytes, mario_coin/k_kira13 SE)
21. WeatherSystem_Init (天候エフェクト)

## TimeTrial_Update (0x801289b8) 概要

- 入力処理: アクセル/ブレーキ/ステアリング → CarObject_FrameUpdate
- DKコイン収集: "CGameCoin.hpp" → カード有効時のみコイン回収、6スロット管理
- ノコノコ障害物: 位置トラッキング、衝突判定
- タイマー: FLOAT_806d132c をカウントダウン、0到達でタイムアップ
- ゴール判定: ラップカウント >= timeLimit で DAT_806d1289 = 1 (完了)
- リザルト: SetCoinCount → TimeTrial_CalcResultText → HUD遷移 → TimeTrial_SaveResult

## 関連グローバル変数

| アドレス | 名前 | 説明 |
|----------|------|------|
| 0x806d132c | FLOAT_806d132c | TT残時間 (float, 毎フレーム減少) |
| 0x806d1288 | DAT_806d1288 | レース開始フラグ |
| 0x806d1289 | DAT_806d1289 | TT完了フラグ (0=走行中, 1=完了) |
| 0x806d128c | DAT_806d128c | アクティブカメラ参照 |
| 0x806d15c0 | DAT_806d15c0 | 天候システムオブジェクト |
| 0x806d12bc | DAT_806d12bc | 0=初期状態 |

## Ghidra リネーム一覧

| アドレス | 旧名 | 新名 |
|----------|------|------|
| 0x80129938 | FUN_80129938 | NokoNokoChallenge_Init (旧 TimeTrial_Init) |
| 0x80123050 | MiniGame_CreateScene | SpecialMode_Create |
| 0x801289b8 | FUN_801289b8 | NokoNokoChallenge_Update (旧 TimeTrial_Update) |
| 0x801287f8 | FUN_801287f8 | TimeTrial_Render |
| 0x80129674 | FUN_80129674 | TimeTrial_Destroy |
| 0x80122efc | FUN_80122efc | GetBestTimeRecord |
| 0x801231fc | FUN_801231fc | GetTimeLimitForCourse |
| 0x800def74 | FUN_800def74 | PreloadEffectResources |
| 0x80188e9c | FUN_80188e9c | NokoNoko_Init |
| 0x80195b94 | FUN_80195b94 | MapWaterEffect_Init |
| 0x80124cf4 | FUN_80124cf4 | GoalCamera_Init |
| 0x8009c688 | FUN_8009c688 | GetStartPosition |
| 0x8016c730 | FUN_8016c730 | WeatherSystem_Init |
| 0x800a4c18 | FUN_800a4c18 | RankingTable_Init |
| 0x80122bb0 | MiniGame_AccumulateBoost | NokoNokoChallenge_HandleBrakeInput |
| 0x80122e58 | FUN_80122e58 | NokoNokoChallenge_TransitionToResult |
| 0x80122c0c | FUN_80122c0c | NokoNokoChallenge_SaveResult |
| 0x80122ce4 | FUN_80122ce4 | NokoNokoChallenge_CalcResultText |
| 0x80122e80 | FUN_80122e80 | NokoNokoChallenge_GetCoinDivisor |
