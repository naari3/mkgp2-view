# ビリビリランド (BiribiriLand) 詳細解析

## 概要
- **courseId**: 0xC (12)
- **種別**: チャレンジモード / ミニゲーム
- **ccVariant**: 3 (ミニゲーム用: speedCap=300, mue=0.4)
- **ゲーム内容**: 直線コース上に配置された電気障害物を避けながらゴールを目指す

## ゲームメカニクス

### ジャンプ (自動バンプ)
**プレイヤーが明示的にジャンプボタンを押す仕組みはない。** ジャンプは `KartMovement_CollisionUpdate` (0x8019b080) 内の地形段差検出による自動バンプとして発生する。

#### 自動バンプ発動条件 (KartMovement_CollisionUpdate内)
1. 2つ以上の接地センサーが接地状態
2. `kartMovement+0x2A8` (jumpHeight) <= 0.0 (現在ジャンプ中でない)
3. 地形法線とカートの走行方向法線の差が閾値を超える (段差検出)
4. `kartMovement+0x1E0` (speed?) が一定値以上

#### ジャンプ物理パラメータ
- **ジャンプ初速**: `12.5 * (1.0 - upVectorY) + 50.0` (0x806d9700, 0x806d972c 定数)
  - upVectorY = kartMovement+0x6C (カートの上方向ベクトルY成分)
  - 平坦な地面 (upVectorY ≈ 1.0) では初速 ≈ 50.0
  - 傾斜地では初速が増加
- **重力**: -1.15f/frame (0x806d9788 = 0xBF933333)
- **ジャンプ高さ格納**: `kartMovement+0x2A8`
- **着地検出**: `kartMovement+0x244` が 1 にセット

#### 位置積分 (KartMovement_PhysicsStep)
毎フレームの Y 方向位置更新:
```
y_velocity += jumpHeight + y_velocity + gravity_each_frame
position.Y += y_velocity * timeStep * posScale
```
`jumpHeight` が正の間カートは上昇し、重力 (-1.15/frame) で減速・下降。

### 障害物 (電気ライン)
9個の電気障害物がコース上に配置される。

#### 障害物配置データ (0x80357f6c)
各障害物: 36バイト (9 float) のエントリ。始点(x,y,z) + 終点(x,y,z) + 拡張データ。

| # | 始点X | 始点Y | 始点Z | 終点X | 終点Y | 終点Z | 方向 | 種別 |
|---|-------|-------|-------|-------|-------|-------|------|------|
| 0 | 113.0 | 0.0 | -6826.0 | -113.0 | 0.0 | -6826.0 | 横 | type 0 |
| 1 | 113.0 | 0.0 | -15552.0 | -113.0 | 0.0 | -15552.0 | 横 | type 1 |
| 2 | 113.0 | 0.0 | -23040.0 | -113.0 | 0.0 | -23040.0 | 横 | type 2 |
| 3 | 113.0 | 0.0 | -29280.0 | -113.0 | 0.0 | -29280.0 | 横 | type 3 |
| 4 | 2384.0 | 0.0 | -34960.0 | 2384.0 | 0.0 | -35928.0 | 縦 | type 2 |
| 5 | 2735.0 | 0.0 | -34960.0 | 2735.0 | 0.0 | -35928.0 | 縦 | type 0 (anim) |
| 6 | 4044.0 | 0.0 | -34960.0 | 4044.0 | 0.0 | -35928.0 | 縦 | type 1 |
| 7 | 5088.0 | 0.0 | -34960.0 | 5088.0 | 0.0 | -35928.0 | 縦 | type 3 (anim) |
| 8 | 6000.0 | 0.0 | -34960.0 | 6000.0 | 0.0 | -35928.0 | 縦 | type 0 (anim) |

- **前半4個 (0-3)**: Z方向に並ぶ横向き障害物 (X=-113〜113)。コース直進区間。
- **後半5個 (4-8)**: X方向に並ぶ縦向き障害物 (Z=-34960〜-35928)。カーブ/横移動区間。
- **拡張データ byte[32]**: 浮遊アニメーションの有無を制御するフラグ。

#### 障害物の浮遊アニメーション (BiribiriLand_UpdateObstacles)
- `bouncePhase` (+0x4C): 毎フレーム +1.0 加算、120.0 でリセット
- 浮遊Y偏差: `5.0 * sin(3.0 * bouncePhase) + 5.0`
- 拡張データのフラグが非ゼロの障害物のみアニメーション適用

### 障害物衝突判定 (BiribiriLand_Update内)
```
kartPos = (kartMovement+0x0C, kartMovement+0x1C, kartMovement+0x2C)
for each obstacle (i = 0..8):
    if not obstacle.isCollected:
        diff = Vec3_Subtract(kartPos, obstacle.startMin)
        dotProduct = Vec3_Dot(diff, obstacle.normal)
        if -10.0 < dotProduct < 10.0:
            if kartPos.Y < obstacle.centerY:
                → 衝突!
```

- **Vec3_Subtract** (0x8025e288): カート位置と障害物始点の減算で相対位置ベクトルを算出
- **Vec3_Dot** (0x8025e368): そのベクトルと障害物法線の内積で「障害物平面までの距離」を算出
- **衝突範囲**: dot が -10.0〜10.0 の範囲 (FLOAT_806d7338=10.0, FLOAT_806d733c=-10.0)
- **高さ条件**: `kartPos.Y < obstacle.centerY` → カートが障害物の高さより低い場合のみ衝突

### 障害物衝突ペナルティ (CarObject_HandleObstacleHit 0x8004ae28)
衝突時の処理:
1. **SE(0x90) 再生** (電気ショック音)
2. **`isCollected = 1`** にセット (同じ障害物との再衝突を防ぐ)
3. **CarObject_HandleObstacleHit** 呼び出し:
   - collision type = 8 → エフェクトID = 0x11C
   - カートのスピン/ダメージアニメーション実行
   - **コイン1枚消失** (CoinSystem_RemoveCoins)
   - **速度低下** (kartMovement+0x0C の travelProgress 低下)

### ゴール判定
ゴールゾーンは `self+0x34` に格納された 0x30 バイトのデータ:
- 始点: (6249.4, 0.0, -8690.3)
- 終点: (6249.4, 0.0, -8983.7)
- 法線: 始点→終点の差分ベクトル (正規化済み)

判定:
```
diff = Vec3_Subtract(kartPos, goalZone)
dotProduct = Vec3_Dot(diff, goalZone.normal)
if dotProduct > 0.0:
    → ゴール通過!
```

### タイマー
- **初期値**: `MiniGame_GetTimerValue()` の戻り値 (float に変換)
- **格納先**: `FLOAT_806d132c` (グローバル変数)
- **毎フレーム減算**: `FLOAT_806d132c -= 1/60.0` (0x3C888889)
- **タイムアップ条件**: `FLOAT_806d132c < 0.0`
- タイムアップ時:
  - `isFinished = 1`
  - `KartMovement_ResetOnTimeout` (0x80040608) 呼び出し → カート停止アニメーション
  - KartMovement+0x1F0 = 0x2B (タイムアップ状態ID)

### コイン報酬 (ゲーム終了時)
```
elapsedTime = totalTime - remainingTime
coinDivisorTable = TimeTrial_GetCoinDivisor()  // courseId, ccClass, subMode 依存
for each entry in table:
    if elapsedTime <= entry.threshold:
        coinReward = entry.coinCount
        break
SetCoinCount(coinReward)
```
- ゴール到達時 (`reachedGoal == 1`): テーブルに基づくコイン報酬
- タイムアップ時 (`reachedGoal == 0`): コイン報酬 0

### 入力処理
BiribiriLand_Update 内の入力フロー:
1. `InputMgr_GetPlayer(0)` でプレイヤー入力取得
2. vtable 経由で各入力値取得:
   - `(*inputPlayer + 0x18)()` → アクセル値 (0.0〜1.0)
   - `(*inputPlayer + 0x1C)()` → ブレーキ値 (0.0〜1.0)
   - `(*inputPlayer + 0x10)()` → ステアリング値 (-1.0〜1.0)
3. `TimeTrial_HandleBrakeInput`: ブレーキ長押し (4秒) でフルブレーキ
4. `FUN_8004e154`: 入力値をCarObjectに適用

ジャンプボタンはない。**電気障害物の回避はカートの横移動 (ステアリング) と地形段差のバンプジャンプによる。**

### ゴール後の演出 (DAT_806d1289 == 1)
ゴール/タイムアップ後:
- 入力無視、カメラが固定アニメーションに切り替え
- `KartMovement_SetSpeedScale(0.5)` で減速
- `FUN_80124ae8`: ゴールカメラ演出
- リザルト画面 → `TimeTrial_TransitionToResult`

## 関数一覧

| アドレス | 関数名 | 役割 |
|----------|--------|------|
| 0x80134ff0 | BiribiriLand_Init | 初期化 (コンストラクタ) |
| 0x8013449c | BiribiriLand_Update | 毎フレーム更新 |
| 0x801341e4 | BiribiriLand_Render | 描画 |
| 0x80134a8c | BiribiriLand_UpdateObstacles | 障害物更新 (浮遊アニメ + 最近接追跡) |
| 0x80134d68 | BiribiriLand_Destroy | デストラクタ |
| 0x8004ae28 | CarObject_HandleObstacleHit | 障害物衝突ペナルティ処理 |
| 0x80040748 | KartMovement_ResetOnGoal | ゴール到達時のカート停止 |
| 0x80040608 | KartMovement_ResetOnTimeout | タイムアップ時のカート停止 |
| 0x8004effc | CarObject_CalcSpeedRatio | 速度比率計算 (HUD用) |
| 0x8019b080 | KartMovement_CollisionUpdate | カート衝突+自動ジャンプ処理 |
| 0x8019cd30 | KartMovement_PhysicsStep | カート物理メインステップ |

## BiribiriLandContext 構造体

| Offset | 型 | フィールド名 | 説明 |
|--------|-----|-------------|------|
| +0x00 | void* | vtable | BiribiriLand vtable (PTR_PTR_8048e46c) |
| +0x04 | float | brakeTimer | ブレーキ長押しタイマー |
| +0x08 | void* | carObject | プレイヤーCarObject |
| +0x0C | void* | lakituStart | ジュゲムスタート演出 |
| +0x10 | void* | inputManager | InputManager |
| +0x14 | void* | scene3D | Scene3D |
| +0x18 | void* | goalCamera | ゴールカメラ |
| +0x1C | void* | cameraBuffer | カメラバッファ |
| +0x20 | void* | renderTarget | レンダーターゲット |
| +0x24 | void* | hudManager | HUD管理 |
| +0x28 | byte | isInitialized | 初期化済みフラグ |
| +0x29 | byte | padding | 未使用 |
| +0x2A | byte | isFinished | ゲーム終了フラグ |
| +0x2B | byte | reachedGoal | ゴール到達フラグ |
| +0x2C | int | finishFrameCount | 終了後フレームカウンタ |
| +0x30 | void* | obstacleArray | 障害物配列 (0x214 bytes, 9 slots) |
| +0x34 | void* | goalZone | ゴールゾーン定義 (0x30 bytes) |

## 障害物スロット構造体 (BiribiriObstacleSlot, 0x38 bytes)

| Offset | 型 | フィールド名 | 説明 |
|--------|-----|-------------|------|
| +0x00 | void* | modelPtr | 3Dモデルポインタ |
| +0x1C | float | startX | 始点X |
| +0x20 | float | startY | 始点Y (+ 浮遊偏差) |
| +0x24 | float | startZ | 始点Z |
| +0x28 | float | endX | 終点X |
| +0x2C | float | endY | 終点Y (+ 浮遊偏差) |
| +0x30 | float | endZ | 終点Z |
| +0x34 | float | centerX | 中心X (= avg of start/end) |
| +0x38 | float | centerY | 中心Y (+ 5.8 offset) |
| +0x3C | float | centerZ | 中心Z |
| +0x40 | float | normalX | 法線X (-dirZ) |
| +0x44 | float | normalY | 法線Y (dirY) |
| +0x48 | float | normalZ | 法線Z (dirX) |
| +0x4C | float | bouncePhase | 浮遊アニメーション位相 (0〜120) |
| +0x50 | byte | isHit | 衝突済みフラグ (再衝突防止) |

## モデルファイル (PTR_s_obj_red_dat_8048e430)
0. obj_red.dat
1. obj_blue.dat
2. obj_yellow.dat
3. obj_pink.dat
4. obj_purple.dat
5. obj_UFO.dat
6. packland_biribiri.dat (コース地形モデル)

モデル0-5 はスケール 12.5 (FLOAT_806d7368) で表示。
モデル6 (packland_biribiri) は別設定。

## 定数テーブル

| アドレス | 値 | 用途 |
|----------|-----|------|
| 0x806d7324 | 0.8 | 不明 |
| 0x806d7328 | 1.0 | ブレーキ入力時のaccel設定値 |
| 0x806d732c | 0.1 | アクセル/ブレーキ入力閾値 |
| 0x806d7330 | 0.0 | ゼロ定数 |
| 0x806d7334 | 0.5 | 中心座標算出の半分係数 / ゴール後の速度スケール |
| 0x806d7338 | 10.0 | 障害物衝突判定距離 (+方向) |
| 0x806d733c | -10.0 | 障害物衝突判定距離 (-方向) |
| 0x806d7340 | 1/60 | タイマー減算量 (1秒=60フレーム) |
| 0x806d7344 | 100.0 | 速度表示倍率 |
| 0x806d7350 | -1.0 | 最近接障害物距離の初期値 |
| 0x806d7354 | 120.0 | 浮遊アニメーション周期 |
| 0x806d7358 | 5.0 | 浮遊アニメーション振幅 |
| 0x806d735c | 3.0 | 浮遊アニメーション角速度係数 |
| 0x806d7360 | 5.8 | 中心点Y方向オフセット |
| 0x806d7364 | 1000.0 | カメラ距離パラメータ |
| 0x806d7368 | 12.5 | 障害物モデルスケール |
| 0x806d736c | 6249.4 | ゴールゾーン始点X |
| 0x806d7370 | -8690.3 | ゴールゾーン始点Z |
| 0x806d7374 | -8983.7 | ゴールゾーン終点Z |
| 0x806d132c | (dynamic) | 残りタイマー値 |

## ジャンプ関連定数 (KartMovement_CollisionUpdate / PhysicsStep)

| アドレス | 値 | 用途 |
|----------|-----|------|
| 0x806d9700 | 12.5 | ジャンプ初速係数 |
| 0x806d972c | 50.0 | ジャンプ初速ベース |
| 0x806d9788 | -1.15 | 重力 (毎フレーム加算) |

## 接近SE
- **SE 0x90**: 障害物衝突時の電気ショック音
- **SE 0x91**: 未通過の最近接障害物に対する接近警告音 (3Dポジショニング付き)
