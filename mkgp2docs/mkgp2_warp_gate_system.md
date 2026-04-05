# MKGP2 ワープゲート＆ダッシュゾーン

## 概要

ワープゲートは**ワープゾーン**(入口→出口テレポート)と**ダッシュゾーン**(自動操縦+加速)の2段構成。speedScale (KartMovement+0x2D8) は使わず、velocity を直接書き換える独自の速度制御を行う。

## キャラ/カート依存性 (コード追証済み)

**一切なし。** CarObject_ProcessWarpAndDash (0x8004d1a8) はカートの位置情報 (KartMovement+0x88〜0x90) のみ参照し、KartParamBlock・charId・kartVariant・ccClass には一切アクセスしない。全定数がハードコード:

| パラメータ | 値 | 説明 |
|---|---|---|
| 初速 | 5.0 | ダッシュ開始時の速度マグニチュード |
| 速度上限 | 100.0 | ダッシュ中の速度クランプ |
| 持続時間 | 5.0秒 | remainingTime 初期値 (1/60 per frame で減算) |
| yaw補間レート | 0.05/frame | 目標角度への回転速度 |

## 対象コース

| courseId | カップ | ワープ | ダッシュ |
|---|---|---|---|
| 0-5 | マリオ/DK/ワリオ/パックマン/クッパ/レインボー | なし | なし |
| 6 | ヨシー | あり | あり |
| 7 | ワルイージ | あり (表/裏別データ) | あり (表/裏別データ) |
| 8 | (courseId 8) | あり (表/裏別データ) | あり (表/裏別データ) |

## 処理フロー

```
CarObject_MainUpdate (0x8004d404)
  └── CarObject_ProcessWarpAndDash (0x8004d1a8)
        │
        ├── Phase 1: 入口検出
        │   WarpZone_CheckEntry → XZ平面の凸四角形ヒットテスト
        │   └── WarpAutoRun_OnEnter → remainingTime=5.0, state=active
        │       └── [プレイヤーのみ] カメラエフェクト(type=6, scale=1.2) + SE(0x12, 0x55, 0x30)
        │
        ├── Phase 2: ダッシュ自動操縦 (毎フレーム)
        │   DashZone_ProcessAutoRun
        │   ├── remainingTime -= 1/60 ... 0以下で state=exit
        │   ├── yaw を targetYaw に 0.05/frame で補間 (最短経路)
        │   └── WarpAutoRun_GetParam → sin/cos で速度ベクトル生成 (mag ≤ 100.0)
        │   結果:
        │   ├── velocity (KartMovement+0x17C-0x184) をゼロクリア後に上書き
        │   ├── prevVelocity, angularVel もゼロクリア (慣性除去)
        │   └── yaw (+0x1C8) に直接セット
        │
        └── Phase 3: テレポート
            WarpZone_FindContaining → 出口ゾーンに入ったか判定
            └── WarpZone_CalcExitPosition → 入口座標をエッジ射影して出口座標算出
                ├── KartMovement_SetPosition → pos, orientMtx, prevPos を一括更新
                ├── KartMovement_RotateByYaw → 向き行列・速度にY軸回転適用
                ├── Camera_RotateByYaw → カメラ同期回転
                └── RenderObj_ToggleLapSegment → ラップセグメント切替
```

## Phase 1: ワープゾーン入口検出

**WarpZone_CheckEntry** (0x800a981c): カート位置 (orientMatrixCopy +0x88/+0x8c/+0x90) が凸四角形ゾーン内にあるか、4辺の半平面テストで判定。

進入時:
- **WarpAutoRun_OnEnter** (0x800a97fc) が remainingTime=5.0 をセット、現在の velocity と yaw をコンテキストに保存
- プレイヤーのみ: カメラエフェクト + SE 3種
- CarObject+0xF4 = 1 (同一フレーム内のSE重複抑制)

## Phase 2: ダッシュ自動操縦

**DashZone_ProcessAutoRun** (0x800a947c):
- remainingTime を 1/60 ずつ減算 → 0以下で exit
- currentYaw を targetYaw に 0.05/frame で補間 ([0, 360) 正規化、±180度の wrap-around 処理)
- **WarpAutoRun_GetParam** (0x800a965c) が角度 → ラジアン変換 → sin/cos で XZ 速度ベクトル生成。累積してマグニチュード 100.0 でクランプ

ダッシュ中は通常の物理エンジンを完全にバイパス:
- velocity, prevVelocity, angularVel をゼロクリアしてからダッシュ速度で上書き
- KartBody_SetWarpTransitFlag で描画にもワープ中を通知

## Phase 3: テレポート

**WarpZone_FindContaining** (0x800a919c) がダッシュ走行中に出口ゾーンへの到達を検出。
**WarpZone_CalcExitPosition** (0x800a9084) が入口位置をエッジベクトルで射影し、出口座標と出口角度を計算。

テレポート実行:
1. `KartMovement_SetPosition` (0x8019a4a0): 位置一括更新
2. `KartMovement_RotateByYaw` (0x80199bbc): 向き・速度にY軸回転
3. `Camera_RotateByYaw` (0x80063f30): カメラ同期
4. `RenderObj_ToggleLapSegment` (0x8003f364): ラップセグメント切替

## WarpAutoRunContext (0x24 bytes, CarObject+0x54)

| Offset | Type | Name | 説明 |
|---|---|---|---|
| +0x00 | int | state | 0=idle, 1=active, 2=exit |
| +0x04 | int | dashZoneId | ダッシュゾーンインデックス (-1=none) |
| +0x08 | float | remainingTime | 残り時間 (初期5.0, -1/60/frame) |
| +0x0C | float | targetYaw | 目標yaw (degrees) |
| +0x10 | float | currentYaw | 現在yaw (degrees) |
| +0x14 | float | velocityX | 速度X |
| +0x18 | float | velocityY | 速度Y |
| +0x1C | float | velocityZ | 速度Z |
| +0x20 | byte | isBattleMode | バトルモードフラグ |

## プレイヤー vs AI

| 機能 | プレイヤー | AI |
|---|---|---|
| ワープゾーン (テレポート) | あり | あり |
| ダッシュゾーン (加速, 5秒, 速度~100) | あり | **なし** |
| カメラエフェクト + SE | あり | なし |

**ダッシュゾーン加速はプレイヤー専用。** CarObject_ProcessWarpAndDash は CarObject_FrameUpdate 経由でプレイヤーの CarObject に対してのみ呼ばれる (RaceMode_FrameUpdate で確認済み)。

AI カートは KartController_Update (0x801e6ed0) の別パスでテレポートのみ実行される。ダッシュ自動操縦・カメラエフェクト・SEはない。

ヨシー・ワルイージカップではプレイヤーがワープゲートで大きな速度アドバンテージを得る。

## 関数一覧

| Address | Name | 説明 |
|---|---|---|
| 0x8004d1a8 | CarObject_ProcessWarpAndDash | 毎フレーム処理メイン |
| 0x800a981c | WarpZone_CheckEntry | 入口ゾーン判定 |
| 0x800a97fc | WarpAutoRun_OnEnter | ダッシュ開始 |
| 0x800a947c | DashZone_ProcessAutoRun | ダッシュ自動操縦 |
| 0x800a965c | WarpAutoRun_GetParam | 速度ベクトル計算 |
| 0x800a919c | WarpZone_FindContaining | 出口ゾーン判定 |
| 0x800a9084 | WarpZone_CalcExitPosition | 出口座標計算 |
| 0x8019a4a0 | KartMovement_SetPosition | 位置テレポート |
| 0x80199bbc | KartMovement_RotateByYaw | 向き回転 |
| 0x80063f30 | Camera_RotateByYaw | カメラ回転 |
| 0x800a8d34 | WarpDashMgr_Init | マネージャ初期化 |
| 0x800a9394 | WarpDashMgr_GetOrCreate | マネージャ取得/作成 |
| 0x800a9378 | WarpDashMgr_GetInstance | マネージャ取得 |
| 0x800a9450 | WarpAutoRun_Init | コンテキスト初期化 |
| 0x8005771c | KartBody_SetWarpTransitFlag | ワープ中描画フラグ |
| 0x8008025c | CameraEffect_Apply | カメラエフェクト |
| 0x8003f364 | RenderObj_ToggleLapSegment | ラップセグメント切替 |

## グローバル変数

| Address | Name | 説明 |
|---|---|---|
| 0x806cf238 | g_warpDashMgr[0] | ノーマル用 WarpDashManager |
| 0x806cf23c | g_warpDashMgr[1] | バトル用 WarpDashManager |

## 定数

| Address | Value | 用途 |
|---|---|---|
| 0x806d4e1c | 5.0 | ワープ初期速度 / remainingTime |
| 0x806d4dec | 1/60 | remainingTime 減算レート |
| 0x806d4dfc | 0.05 | yaw 補間レート |
| 0x806d4e18 | 100.0 | 速度クランプ |
| 0x806d4df0 | 360.0 | 角度正規化 |
| 0x806d4e00 | PI/180 | 度→ラジアン |
| 0x806d2798 | 1.2 | カメラエフェクトスケール |
