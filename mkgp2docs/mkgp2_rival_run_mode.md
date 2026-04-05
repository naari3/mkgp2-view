# MKGP2 Rival Run Mode (courseId=14 / 0xE)

## 概要

ライバルカート1台と同じコースで制限時間付き1周レースを行うミニゲーム。
プレイヤーが先にラップを完了すれば勝利(コイン付与)、ライバルが先着またはタイマー切れで敗北。

## MiniGame_CreateByCourseId での生成

- courseId=14 (0xE) のケース
- Alloc(0x3C) → RivalRun_Init 呼び出し
- 構造体サイズ: 60バイト (RivalRunContext)
- vtable: PTR_PTR_8048e4a0

## 関数一覧

| アドレス | 名前 | 説明 |
|----------|------|------|
| 0x80136ea0 | RivalRun_Init | 初期化。プレイヤーCarObject, ライバル, Scene3D, Camera, HUD, LakituStart, WeatherSystem生成 |
| 0x80135734 | RivalRun_Update | 毎フレーム更新。入力処理, AI更新, タイマー, ラップ判定 |
| 0x8013558c | RivalRun_Draw | レンダリング |
| 0x80136ba8 | RivalRun_Destroy | オブジェクト解放 |
| 0x8013732c | RivalRun_RivalInit | ライバルCarObject初期化 (charId=0/Mario, 0x54バイト) |
| 0x80135b7c | RivalRun_RivalAIUpdate | ライバル自動操縦 (パス追従, ステアリング, ドリフト) |
| 0x80136148 | RivalRun_RivalTactics | AI戦術 (距離ベースのアイテム使用, ラインチェンジ, 速度調整) |
| 0x80135efc | RivalRun_UpdatePathFollower | ウェイポイント追従更新 (最近接点検索, 方向ベクトル計算) |
| 0x80137588 | RivalRun_PathFollowerInit | パスフォロワー初期化 (0x2C bytes) |
| 0x801367f4 | RivalRun_CalcPathDistance | 2カート間のパス沿い符号付き距離計算 |

## vtable (PTR_PTR_8048e4a0)

| Idx | アドレス | 役割 |
|-----|----------|------|
| 0 | 0x806cf634 | RTTI/データポインタ |
| 1 | NULL | - |
| 2 | 0x80135734 | RivalRun_Update |
| 3 | 0x8013558c | RivalRun_Draw |
| 4 | 0x80136ba8 | RivalRun_Destroy |
| 5 | NULL | - |
| 6 | 0x80139a6c | CoinSystem_UpdateAll (共通) |
| 7 | 0x80139ad0 | 共通vtable関数 |
| 8-15 | ... | 共通vtable関数群 |

## RivalRunContext 構造体 (0x3C = 60 bytes)

| Offset | 型 | フィールド名 | 説明 |
|--------|-----|------------|------|
| +0x00 | ptr | vtable | 仮想関数テーブル |
| +0x04 | float | floatZero | 0.0 初期化 |
| +0x08 | ptr | playerCarObject | プレイヤーCarObject |
| +0x0C | ptr | lakituStart | ジュゲムスタート |
| +0x10 | ptr | inputManager | 入力マネージャ |
| +0x14 | ptr | scene3D | 3Dシーン |
| +0x18 | ptr | goalCamera | ゴールカメラ |
| +0x1C | ptr | camera | メインカメラ |
| +0x20 | ptr | renderTarget | レンダーターゲット |
| +0x24 | ptr | physicsHelper | 物理ヘルパー |
| +0x28 | ptr | hud | HUD |
| +0x2C | ptr | subSystem | サブシステム |
| +0x30 | ptr | rivalCtx | RivalAIContext |
| +0x34 | byte | isInitialized | 初期化済みフラグ |
| +0x35 | byte | (padding) | - |
| +0x36 | byte | isFinished | 終了フラグ |
| +0x37 | byte | isPlayerWin | プレイヤー勝利フラグ |
| +0x38 | int | finishTimer | 終了後タイマー |

## 勝敗判定ロジック (RivalRun_Update)

1. **タイマー**: FLOAT_806d132c がカウントダウン (毎フレーム FLOAT_806d7394 減算)
2. **ラップ判定**: KartMovement+0x240 < 1 で周回完了
3. 判定:
   - プレイヤーが先に完了 → isPlayerWin=1, SetCoinCount(divisor)
   - ライバルが先に完了 → isPlayerWin=0, SetCoinCount(0)
   - タイマー切れ → isPlayerWin=0, SetCoinCount(0)
4. 結果表示: TimeTrial_CalcResultText → HUD表示 → TimeTrial_SaveResult → TimeTrial_TransitionToResult

## ライバルAI制御

### 操縦 (RivalRun_RivalAIUpdate)
- コースパス追従: Vec3_Subtract/Normalize でパス方向を計算
- ステアリング: パス方向との角度差 → 比例制御
- ドリフト: 速度閾値超過+角度条件で自動ドリフト発動 (FUN_8004f584)
- 速度: FLOAT_806cf610〜806cf620 のパラメータで制御

### 戦術 (RivalRun_RivalTactics)
- プレイヤーとの距離を RivalRun_CalcPathDistance で算出
- 距離に応じてアイテム使用 (0x32=近距離, 0x3e/0x6c/0x3d=遠距離)
- ラインチェンジ: 3レーン制、相対位置に応じて切り替え
- 速度ファクター: courseDataのテーブルから距離に応じた速度係数を設定

### パス追従 (RivalRun_UpdatePathFollower)
- ウェイポイントリストから最近接点を検索
- 目標位置と方向ベクトルを更新
- 終端到達時はオーバーシュート補正
