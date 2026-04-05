# MKGP2 ジャンプ飛距離ミニゲーム (courseId=0x10)

旧名: CoinRunMode -> **JumpDistanceMode** にリネーム済み

ジャンプ台からカートをジャンプさせ、飛距離に応じてコインが貰えるミニゲーム。
制限時間30秒以内に複数回ジャンプでき、飛距離は累積加算される。

## 関数一覧

| アドレス | 関数名 | 説明 |
|----------|--------|------|
| 0x80213210 | JumpDistanceMode_Init | 初期化 (struct=0x8C bytes) |
| 0x80211e6c | JumpDistanceMode_Update | フレーム更新 (カメラ, タイマー, HUD, リザルト) |
| 0x80210cd4 | JumpDistanceMode_UpdateGameplay | ゲームプレイ状態機械 |
| 0x80211550 | JumpDistanceMode_HandlePhysics | 物理/入力ハンドラ |
| 0x80210a50 | JumpDistanceMode_PredictLanding | 着地位置予測 (物理シミュ) |
| 0x80213d58 | JumpDistanceMode_CopyKartMovement | KartMovement複製 |
| 0x80213a64 | JumpDistanceMode_InitTimerDisplay | タイマーHUD初期化 |

## 初期化パラメータ

- **ccVariant**: 7 (speed=250, mue=0.4)
- **制限時間**: 30秒 (全ccClass/subMode共通)
- **ターゲット飛距離**: 70.0m (全ccClass/subMode共通)
- **チェックポイント平面**: コースバリアント (DAT_806d1270) により2セット

## 状態機械 (ctx+0x50)

```
State 0: WaitForLaunch
  - プレイヤー入力を処理、通常走行
  - IsAirborne==1 かつ ランプゾーン内 かつ height-ground >= 6.0 でジャンプ開始
  -> State 1

State 1: Airborne (飛距離計測中)
  - 毎フレーム飛距離を累積加算
  - コインマイルストーン判定
  - タイマー切れ -> State 0 にリセット
  - 着地平面を通過 かつ ランプゾーン内 -> State 2

State 2: Landed (着地)
  - PredictLanding でカメラターゲットを設定
  - IsAirborne==0 で着地完了 -> State 3

State 3: ApproachLandingTarget
  - カートが目標平面を通過するか監視
  -> State 4

State 4: OvershootBounce (カメラ演出)
  - カートの速度が0になるまで待機
  - 速度==0 -> State 5

State 5: TimeUp / Complete
  - ゲーム終了処理
```

## ジャンプ開始条件 (State 0 -> 1)

3つの条件すべてを満たすと飛距離計測が開始される:

1. **空中判定**: `CarObject_IsAirborne` == 1 (KartMovement+0x2bd)
2. **ランプゾーン内**: 位置が矩形領域内
   - X: [300.0, 1000.0] (DAT_803c3d54, DAT_803c3d5c)
   - Z: [-800.0, -7500.0] (DAT_803c3d58 < 0, DAT_803c3d60 < 0、ただし min/max 逆順注意)
3. **高度条件**: `kartPosY - groundHeight >= 6.0` (FLOAT_806dada8)
   - Terrain_GetGroundHeight (0x80035700) で地面高さを取得
   - ジャンプ台上の微小な浮きでは発動しない

追加条件: 速度ベクトルとランプ法線の内積 < 0 (ランプから離れる方向に移動中)

## 飛距離計測ロジック

### 距離計算式 (State 1, 2)

```
deltaX = currentPosX - prevPosX
deltaZ = currentPosZ - prevPosZ
distSq = deltaX * deltaX + deltaZ * deltaZ
dist = sqrt(distSq)  // Newton-Raphson inverse sqrt * 3回反復
travelDistance += dist * 3.6 / 100.0
```

- **3.6**: ゲーム内部単位→km/h変換係数 (CalcCurrentSpeed等と同じ)
- **100.0**: 100で割ってメートル単位に変換
- 実質: `travelDistance += sqrt(deltaXZ^2) * 0.036` (メートル/フレーム)
- Y軸（高さ）は無視。水平距離のみ計測

### 格納先

- `ctx+0x58`: travelDistance (float, 累積飛距離メートル)
- `ctx+0x5C`: prevPosX
- `ctx+0x60`: prevPosY
- `ctx+0x64`: prevPosZ
- `ctx+0x74`: simTime (1/60ずつ加算)

## コイン報酬テーブル

TimeTrial_GetCoinDivisor (0x80122e80) が返すテーブル参照。
courseId=16 では全ccClass/subMode共通:

| 飛距離閾値 (m) | 累積コイン数 |
|----------------|-------------|
| 1 | 2 |
| 30 | 4 |
| 40 | 6 |
| 50 | 8 |
| 60 | 10 |
| 70 | 15 |
| 0 (終端) | 0 |

テーブルアドレス: 0x8048de68 (全6バリアント同一データ)

### マイルストーン処理

```c
milestoneIdx = ctx+0x80  // 現在のマイルストーンインデックス
entry = table[milestoneIdx]  // {threshold, cumulativeCoins} の8バイトペア
if (travelDistance >= entry.threshold && entry.coins > 0) {
    totalCoins = entry.coins  // ctx+0x84
    coinDelta = totalCoins - previousTotal  // HUD差分表示用
    milestoneIdx++
}
```

- 飛距離が増えるたびに次のマイルストーンを確認
- HUD表示 (ctx+0x2C) に整数化した飛距離を設定
- 飛距離が変化するたびに SE 0x1B (コイン音) を再生

## 着地判定

### 平面通過検知

3つのチェックポイント平面 (ctx+0x0C, ctx+0x0D, ctx+0x0E) を使用:
- 平面は2点から方向ベクトルを計算 (Vec3_Subtract + Vec3_Normalize)
- 毎フレーム `Vec3_Subtract(kartPos, planePoint)` で差分を取り、法線との内積で符号を判定
- 内積 > 0 なら平面を通過

### 着地位置予測 (JumpDistanceMode_PredictLanding)

1. KartMovement を丸ごとコピー (0x324 bytes)
2. コピーに対して KartMovement_PhysicsStep を最大65535回ループ
3. 1秒経過後に重力加速 (KM+0x2d4 = 80.0) を適用
4. KM+0x2bd (airborne) == 0 になったら終了
5. 着地位置からカメラターゲットを計算:
   - トラック方向に50ユニットオフセット
   - 高さ+5ユニット
   - 壁衝突チェック (FUN_80035ae4) で反転

## カメラ制御 (HandlePhysics内)

### State 3 (空中追跡)

カートのヨー角に基づきアクセル入力を調整:
- バリアント0: yaw < 175度 → 減速(-0.5)、yaw >= 185度 → 半速(0.5)
- バリアント1: yaw > 355度 or yaw < 180度 → 半速(0.5)、5度 <= yaw → 減速(-0.5)
- ピーク速度を ctx+0x68 に記録（カメラ距離計算用）
- ピーク速度は 0.02/frame ずつ減衰、上限 0.8

### State 4 (着地後アプローチ)

- カートが目標平面を通過するか監視
- 速度ベクトルのY成分で進行方向を判定
- ピーク速度は 0.02/frame ずつ0に向かって減衰

## コンテキスト構造体レイアウト (0x8C bytes)

| Offset | Type | 名前 | 説明 |
|--------|------|------|------|
| 0x00 | ptr | vtable | PTR_PTR_804ecc08 |
| 0x04 | float | unk04 | 初期値 100.0 |
| 0x08 | ptr | carObject | CarObjectポインタ |
| 0x0C | ptr | rampStartPlane | ランプ開始平面 |
| 0x0D | ptr | landingPlane | 着地判定平面 |
| 0x0E | ptr | targetPlane | 目標平面 |
| 0x14 | ptr | scene3D | 3Dシーン |
| 0x18 | ptr | lakituStart | ジュゲム開始演出 |
| 0x1C | ptr | kartModelScene | カートモデルシーン |
| 0x28 | ptr | hudManager | HUDマネージャ |
| 0x2C | ptr | distanceDisplay | 飛距離表示 (int値) |
| 0x30 | ptr | checkpointPlane1 | チェックポイント平面1 |
| 0x34 | ptr | checkpointPlane2 | チェックポイント平面2 |
| 0x38 | ptr | checkpointPlane3 | チェックポイント平面3 |
| 0x3C | ptr | signalCtrl | 信号制御 |
| 0x40 | ptr | cameraState | カメラ状態 |
| 0x44 | byte | initialized | 初期化済みフラグ |
| 0x45 | byte | beatRecord | 記録更新フラグ |
| 0x48 | int | endDelay | 終了遅延カウンタ (0x1E=30フレーム) |
| 0x4C | int | endPhase | 終了フェーズ |
| 0x50 | int | state | ゲームプレイ状態 (0-5) |
| 0x54 | float | targetDistance | ターゲット飛距離 (70.0m) |
| 0x58 | float | travelDistance | 累積飛距離 (m) |
| 0x5C | float | prevPosX | 前フレームX座標 |
| 0x60 | float | prevPosY | 前フレームY座標 |
| 0x64 | float | prevPosZ | 前フレームZ座標 |
| 0x68 | float | peakSpeed | ピーク速度 (カメラ用) |
| 0x6C | byte | unk6C | |
| 0x6D | byte | unk6D | |
| 0x70 | ptr | physicsClone | KartMovement複製 (着地予測用) |
| 0x74 | float | simTime | シミュレーション時間 |
| 0x78 | | | |
| 0x7C | ptr | coinEffectCtrl | コインエフェクト制御 |
| 0x80 | int | milestoneIdx | 現在のマイルストーンインデックス |
| 0x84 | int | totalCoins | 累積コイン数 |
| 0x88 | byte | outOfBounds | コース外フラグ |

## ゲーム終了処理

1. State 4 で速度が0になると State 5 へ遷移
2. DAT_806d1289 = 1 (ゲーム終了フラグ)
3. カード有効時: `beatRecord = true` (常に記録更新)
4. カード無効時: `beatRecord = (targetDistance < travelDistance)` (70m超えたか)
5. コイン上限: 15 (totalCoins > 15 なら 15 にクランプ)
6. SetCoinCount で最終コイン数を設定
7. 30フレーム (0x1E) 待機後にリザルト画面遷移

## 定数一覧

| アドレス | 値 | 用途 |
|----------|-----|------|
| 806dad70 | 1.0 | 1.0定数 |
| 806dad74 | 1/60 (0.0167) | フレーム時間 |
| 806dad78 | 80.0 | 重力加速値 (予測シミュ用) |
| 806dad7c | 0.0 | ゼロ定数 |
| 806dad80 | 50.0 | カメラオフセット距離 |
| 806dad84 | 5.0 | カメラ高さオフセット |
| 806dad88 | 0.1 | 入力デッドゾーン |
| 806dad8c | 0.5 | 半速アクセル |
| 806dada0 | 3.6 | 内部単位→km/h変換 |
| 806dada4 | 100.0 | 距離スケール (÷100でメートル) |
| 806dada8 | 6.0 | ジャンプ検知高度閾値 |
| 806dadac | 0.8 | ピーク速度上限 |
| 806dadb0 | 3.0 | ジャンプパッド強度 |
| 806dadb4 | 360.0 | 角度正規化 |
| 806dadb8 | 175.0 | ヨー角閾値1 |
| 806dadbc | -0.5 | 減速アクセル |
| 806dadc0 | 185.0 | ヨー角閾値2 |
| 806dadcc | 0.02 | ピーク速度減衰率 |
| 806dadd8 | 0.01745 | DEG_TO_RAD |
| 803c3d54 | 300.0 | ランプゾーン minX |
| 803c3d58 | -7500.0 | ランプゾーン minZ |
| 803c3d5c | 1000.0 | ランプゾーン maxX |
| 803c3d60 | -800.0 | ランプゾーン maxZ |
| 803c3d64 | 1900.0 | OOBゾーン minX |
| 803c3d68 | -7800.0 | OOBゾーン minZ |
| 803c3d6c | 2100.0 | OOBゾーン maxX |
| 803c3d70 | -7400.0 | OOBゾーン maxZ |
