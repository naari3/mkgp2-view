# MKGP2 スイカ割りミニゲーム (courseId=9) 解析

## 概要

スイカボール(watermelon ball)を車で押してコース上のチェックポイントゲートを通過し、ゴールを目指すミニゲーム。「割る」ではなく「押して運ぶ」ゲーム。制限時間内にゴール(最終ゲート)を通過すれば成功。

## 主要関数

| アドレス | 名前 | 説明 |
|----------|------|------|
| 0x80133398 | SuikaBall_Init | ミニゲーム全体の初期化 |
| 0x801319dc | SuikaBall_Update | 毎フレーム更新 |
| 0x801313d0 | SuikaBall_Render | 描画処理 |
| 0x80124984 | SuikaBallObj_Init | スイカボールオブジェクト初期化 |
| 0x801240e4 | SuikaBallObj_UpdatePhysics | ボール物理演算 |
| 0x801240b4 | SuikaBallObj_Render | ボール3Dモデル描画 |
| 0x80124ae8 | SuikaBallObj_UpdateGameOverAnim | ゲームオーバー時のボール膨張アニメーション |

## SuikaBallScene 構造体 (0x6C bytes)

SuikaBall_Init で確保される、ミニゲーム全体の管理構造体。

| Offset | 型 | 名前 | 説明 |
|--------|-----|------|------|
| 0x00 | ptr | vtable | PTR_PTR_8048e3f8 |
| 0x04 | float | - | 未使用 (0.0) |
| 0x08 | ptr | courseData | MiniGame_GetCourseData()の戻り値 |
| 0x0C | ptr | carObject | プレイヤーCarObject |
| 0x10 | ptr | scene3dObj | Scene3D描画管理 |
| 0x14 | ptr | lodManager | LOD管理 |
| 0x18 | ptr | cameraPath | カメラパス |
| 0x1C | ptr | goalCamera | ChallengeGoalCamera |
| 0x20 | ptr | activeCamera | カメラインスタンス |
| 0x24 | ptr | renderTarget | 描画ターゲット |
| 0x28 | ptr | lakituStart | ジュゲムスタート演出 |
| 0x2C | ptr | hud | HUDオーバーレイ |
| 0x30 | byte | isStarted | レース開始フラグ |
| 0x31 | byte | bgmPlaying | BGM再生中フラグ |
| 0x32 | byte | isGameOver | ゲーム終了フラグ |
| 0x33 | byte | isGoalReached | ゴール到達フラグ |
| 0x34 | int | gameOverTimer | ゲームオーバー後のカウンタ |
| 0x3C | ptr | suikaBallObj | スイカボールオブジェクト |
| 0x40 | ptr | goalGate | ゴールゲート判定データ |
| 0x44 | ptr | npcToad1vtbl | NPC Toad vtable 1 |
| 0x48 | ptr | npcToad2vtbl | NPC Toad vtable 2 |
| 0x4C | float | speedScale | カート速度スケール (衝突で低下、徐々に回復) |
| 0x50 | float[3] | kartForwardDir | カートの前方ベクトル |
| 0x5C | ptr | npcToadArray | NPC Toad配列 (3体) |
| 0x60 | ptr | coinSpawner | コイン生成マネージャ |
| 0x64 | ptr | checkpointMgr | チェックポイント管理 |
| 0x68 | int | score | スコア (獲得コイン数) |

## SuikaBallObj 構造体 (0x7C bytes)

スイカボール物理オブジェクト。

| Offset | 型 | 名前 | 説明 |
|--------|-----|------|------|
| 0x00 | ptr | modelObj | 3Dモデル (MRO_suikaball.dat) |
| 0x04 | float | radius | 衝突判定半径 (バウンディングボックスから算出 * 0.5) |
| 0x08-0x34 | float[12] | orientMatrix | 3x4回転行列 |
| 0x38 | float | posX | X座標 |
| 0x3C | float | posY | Y座標 (高さ) |
| 0x40 | float | posZ | Z座標 |
| 0x44 | float | velX | X方向速度 |
| 0x48 | float | velY | Y方向速度 (鉛直) |
| 0x4C | float | velZ | Z方向速度 |
| 0x50 | float | spinMagnitude | 回転の強さ |
| 0x54 | float | spinAxisX | 回転軸X |
| 0x58 | float | spinAxisY | 回転軸Y |
| 0x5C | float | spinAxisZ | 回転軸Z |
| 0x60 | byte | isColliding | カートと衝突中フラグ |
| 0x61 | byte | isGoalReached | ゴール到達フラグ |
| 0x64 | float | innerCollisionZone | 内側衝突ゾーン (0.3) |
| 0x68 | float | outerCollisionZone | 外側衝突ゾーン (0.4) |
| 0x6C | float | wallBounce | 壁反射減衰 (0.8) |
| 0x70 | float | gravity | 重力加速度 (0.1) |
| 0x74 | float | airDrag | 空気抵抗減衰 (0.96) |
| 0x78 | float | groundFriction | 地面摩擦減衰 |

courseData が存在する場合、+0x64〜+0x78 の値は courseData の先頭24バイトで上書きされる。

## ボール物理 (SuikaBallObj_UpdatePhysics)

### 衝突 -> 前方射出メカニズム

1. カートの orient matrix (+0x0C列 = 前方ベクトル) からカート位置を取得
2. カート位置からボール位置へのベクトルを計算
3. そのベクトルの長さが `radius + param_1(=12.0)` 未満なら **衝突判定**
4. 衝突時:
   - ボール位置を「カート位置から radius 分離れた位置」に補正
   - 速度の向きがカート前方と反対方向なら、反射成分を `-2.0 * dot` 倍で反転（弾き返し）

### 衝突応答の3ゾーン

カート前方軸への射影距離に基づいて3つのゾーンがある:

| ゾーン | 条件 | 挙動 |
|--------|------|------|
| 近距離 | `dot < radius * innerZone` | カート前方方向に `2.0 * forward * kartSpeed` で射出 + Y方向 `1.0` の浮き |
| 中距離 | `< radius * (inner + outer)` | 角度付き射出: `DEG_TO_RAD * 45.0` の回転をかけた方向に `1.5 * rotated * kartSpeed` で力を加算 |
| 遠距離 | それ以上 | 完全停止: spinMag = kartSpeed, 方向反転 (dot < 0 なら -1.0), vel = 0 |

### 壁衝突

- 4方向 (前後左右にradius分) の地形衝突テストを実行
- 壁に衝突した場合、法線方向に押し戻し + 速度を法線方向にリフレクト
- リフレクト後、速度に wallBounce (0.8) を掛けて減衰

### 地面判定

- 地面レイキャスト (FUN_80035700) でY座標を取得
- ボール底面が地面より下なら、Y座標を補正 + Y速度を反転 * -wallBounce
- 地面近くにいるかどうかで摩擦モードを切替:
  - **地面上**: vel *= groundFriction (courseDataから設定)
  - **空中**: vel *= airDrag (0.96)
  - **ゴール到達後**: vel *= 0.9 (固定高減衰)

### 重力と速度制限

- 毎フレーム: velY -= gravity (0.1)
- 速度の大きさが `10.0` を超えた場合、`10.0` にクランプ
- spinMagnitude *= 0.98 (毎フレーム自然減衰)

### 回転 (見た目のスピン)

- 速度ベクトルから回転軸を計算 (velZ, 0, -velX の方向、速度ベクトルとY軸の外積に相当)
- 回転速度 = `0.01 * PI * |vel|` (速度に比例)
- `spinMagnitude > 0` の場合は衝突時の回転軸を使った回転も加算

## チェックポイントシステム

### チェックポイントマネージャ (+0x64)

| Offset | 型 | 説明 |
|--------|-----|------|
| 0x00-0x0F | ptr[4] | 4つのゲートへのポインタ |
| 0x10 | int | 現在のチェックポイントインデックス |

### ゲート通過判定

各ゲートは2点 (p1, p2) で定義される線分。ボール位置と線分の差分ベクトルを計算し、法線方向への dot product が `> 0.0` なら通過と判定。

通過判定はボール位置ベースで、順番にしか進めない (currentIndex が < 4 の場合のみチェック)。

### ゲート座標 (DAT_803579f4)

| Gate | P1 (X, Z) | P2 (X, Z) |
|------|-----------|-----------|
| 0 | (1603.9, 875.1) | (1659.2, 958.8) |
| 1 | (1166.6, 949.3) | (1142.5, 1046.0) |
| 2 | (648.9, 991.3) | (685.0, 1083.0) |
| 3 (GOAL) | (313.9, 960.2) | (260.2, 1042.6) |

Y座標は全て 0.0。

### ゲート通過時の報酬

| 状況 | コインスポーン数 | スコア加算 |
|------|----------------|-----------|
| ボールがカートに衝突 | 5個 (角度: 0, +30deg, -30deg, +90deg, -90deg) | +5 |
| 中間ゲート通過 (index < 3) | 2個 (+30deg, -30deg) | +2 |
| ゴールゲート通過 (index == 3) | 4個 (+30deg, -30deg, +90deg, -90deg) | +4 |

## コインスポーンメカニズム

衝突/ゲート通過時にコインが放出される:
1. カート→ボール方向の角度を `atan2(dz, dx)` で計算
2. 各コインに角度オフセット (0, +/-30deg, +/-90deg) を加算
3. cos/sin で方向ベクトルを生成、速度 `0.7` で射出
4. Y方向初速 `1.82`
5. コイン生成には `IsCardValid()` チェックが必要 (カード未挿入時はコイン出現しない)
6. 最大同時コイン数: 6スロット (stride 0x16C = 364 bytes)

## 速度スケールシステム

カートの速度にスケール値 (scene+0x4C) を掛けることで、衝突時の減速と回復を実現。

| イベント | speedScale |
|----------|-----------|
| 初期値 | 1.0 |
| ボール衝突時 | 0.3 にリセット |
| 毎フレーム | +0.01 (最大 1.0 まで回復) |
| ゲームオーバー後 | 0.5 固定 (自動走行) / 0.0 (停止) |

## タイマーシステム

- グローバル変数 `FLOAT_806d132c` に残り時間を格納
- 初期値: `MiniGame_GetTimerValue()` から取得 (整数 → float変換)
- 毎フレーム `-1/60` 減算 (= 毎秒1.0ずつ減少)
- `0.0` に達したらゲームオーバー

## ゲームオーバー処理

1. タイマー切れ or ゴール到達で `isGameOver = 1`
2. `gameOverTimer` が 0 の時:
   - コイン数を `SetCoinCount()` で確定
   - BGM停止
   - ゴール到達なら勝利SE (0xd6) 再生、カート速度リセット
   - `TimeTrial_CalcResultText` でリザルトテキスト生成
   - `TimeTrial_SaveResult` でスコア保存
3. `gameOverTimer > 0` の間:
   - `SuikaBallObj_UpdateGameOverAnim`: ボールが毎フレーム radius += 1.0 で膨張 (最大 298.0)
   - リザルト画面完了後 `TimeTrial_TransitionToResult` で遷移

## ボール初期位置

- posX = 1950.6, posY = -195.0, posZ = 539.1

## 描画処理 (SuikaBall_Render)

1. 3Dシーン (コース、カート、LOD) 描画
2. スイカボールモデル描画
3. NPC Toad 3体: ボーン同期 (元モデル→表示モデルへ translate/rotation/scale コピー)
4. HUDオーバーレイ
5. GX immediate mode でチェックポイントゲートマーカー描画 (4つの quad strip、Y=上下オフセット付き)
6. コインスポーナー描画
7. エフェクト・天候

## 定数一覧

### SuikaBallObj 物理定数 (0x806d6e58-)

| アドレス | 値 | 用途 |
|----------|-----|------|
| 806d6e58 | 0.0 | ゼロ定数 |
| 806d6e5c | -2.0 | 反射係数 |
| 806d6e60 | 2.0 | 前方射出係数 |
| 806d6e64 | 1.0 | Y方向浮上量 / 地面判定閾値 |
| 806d6e68 | -1.0 | 方向反転 |
| 806d6e6c | 0.01745 | DEG_TO_RAD |
| 806d6e70 | 45.0 | 中距離ゾーンの回転角度 (度) |
| 806d6e74 | 1.5 | 中距離ゾーンの射出力係数 |
| 806d6e78 | 10.0 | 最大速度 |
| 806d6e7c | 0.01 | 回転速度係数 |
| 806d6e80 | PI | 回転計算用 |
| 806d6e84 | 0.9 | ゴール到達後の減衰 |
| 806d6e88 | 0.98 | spinMagnitude 自然減衰 |
| 806d6e8c | 5.0 | デフォルト radius |
| 806d6e90 | 0.3 | innerCollisionZone |
| 806d6e94 | 0.4 | outerCollisionZone |
| 806d6e98 | 0.8 | wallBounce |
| 806d6e9c | 0.1 | gravity |
| 806d6ea0 | 0.96 | airDrag |
| 806d6ea4 | 0.5 | boundingBox scale for radius |

### SuikaBall_Update 定数 (0x806d72a4-)

| アドレス | 値 | 用途 |
|----------|-----|------|
| 806d72a4 | 1.0 | speedScale 上限 |
| 806d72b0 | 0.8 | デバッグ用アクセル入力 |
| 806d72b4 | 0.1 | デッドゾーン閾値 |
| 806d72b8 | 0.0 | ゼロ定数 |
| 806d72bc | 0.5 | ゲームオーバー時 speedScale |
| 806d72c0 | 12.0 | 衝突判定の追加距離 |
| 806d72c4 | 0.3 | 衝突後 speedScale リセット値 |
| 806d72c8 | 0.01 | speedScale 回復レート |
| 806d72cc | 0.7 | コイン射出速度 |
| 806d72d0 | 1.82 | コインY方向初速 |
| 806d72d4 | +30deg (rad) | コイン角度オフセット1 |
| 806d72d8 | -30deg (rad) | コイン角度オフセット2 |
| 806d72dc | +90deg (rad) | コイン角度オフセット3 |
| 806d72e0 | -90deg (rad) | コイン角度オフセット4 |
| 806d72e4 | 1/60 | タイマー減算量/フレーム |
| 806d72e8 | 100.0 | HUD速度表示スケール |
