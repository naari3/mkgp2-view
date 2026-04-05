# ReverseChallenge (courseId=0xb) - 逆走チャレンジ

## 概要

**バック走行でバナナを避けながらゴールを目指す**ミニゲーム。旧称「TestDrive（試乗モード）」は誤称であり、実態は逆走チャレンジ。`MiniGame_CreateByCourseid` (0x80123050) の `case 0xb` で生成される。

## 関数一覧

| アドレス | 関数名 | 役割 |
|----------|--------|------|
| 0x800ad910 | ReverseChallenge_Init | 初期化コンストラクタ |
| 0x800ad134 | ReverseChallenge_Update | メインUpdate (vtable[2]) |
| 0x800acfc0 | ReverseChallenge_Render | 描画 (vtable[3]) |
| 0x800acd60 | ReverseChallenge_Dtor | デストラクタ (vtable[4]) |

### 関連関数

| アドレス | 関数名 | 役割 |
|----------|--------|------|
| 0x80196d9c | BananaPositionData_Init | バナナ配置データ読み込み |
| 0x80196c5c | BananaField_CheckCollision | バナナ衝突判定 (毎フレーム) |
| 0x80196f60 | BananaField_TestHitBox | OBBベースのヒットテスト |
| 0x80196aa0 | BananaField_Render | バナナ描画 (ヒット時の吹き飛びアニメ含む) |
| 0x80124cf4 | ChallengeGoalCamera_Init | ゴール演出カメラ初期化 |
| 0x80124ae8 | ChallengeGoalCamera_Update | ゴール到達後のカメラ演出 |
| 0x8004e154 | CarObject_ApplyInput | ステアリング/アクセル/ブレーキ入力適用 |
| 0x80040748 | CarObject_PlayGoalReachedAnim | ゴール到達時のカート演出 (stateId=0x2A) |
| 0x80040608 | CarObject_PlayTimeoutAnim | タイムアウト時のカート演出 (stateId=0x2B) |

## vtable (PTR_PTR_80419e70)

| Offset | Address | 関数 |
|--------|---------|------|
| +0x00 | 0x806cf2c0 | データポインタ |
| +0x04 | 0x00000000 | null |
| +0x08 | 0x800ad134 | ReverseChallenge_Update |
| +0x0C | 0x800acfc0 | ReverseChallenge_Render |
| +0x10 | 0x800acd60 | ReverseChallenge_Dtor |

## self 構造体 (0x4C = 76 bytes)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| +0x00 | void* | vtable | PTR_PTR_80419e70 |
| +0x04 | float | brakeTimer | ブレーキ長押しタイマー (0.0で初期化) |
| +0x08 | void* | inputMgr | InputManager |
| +0x0C | void* | camera | Camera |
| +0x10 | void* | renderTarget | RenderTarget |
| +0x14 | void* | carObject | CarObject (プレイヤーカート) |
| +0x18 | void* | lakituStart | LakituStart |
| +0x1C | void* | scene3D | Scene3D |
| +0x20 | void* | goalCamera | ChallengeGoalCamera |
| +0x24 | void* | hud | HUD |
| +0x28 | float* | cameraPath | カメラパスデータ (0x30 bytes, ゴールライン定義含む) |
| +0x2C | float* | goalEdgeNormals | ゴール矩形の辺法線 (0x20 bytes, 8 floats) |
| +0x30 | void* | bananaField | BananaPositionData |
| +0x34 | float[3] | kartForwardVec | カートの前方ベクトル (毎フレーム更新) |
| +0x40 | byte | isReady | 初回フレームで1に設定 |
| +0x41 | byte | isFinished | ゴール or タイムアウトで1 |
| +0x42 | byte | reachedGoal | ゴール到達=1, タイムアウト=0 |
| +0x44 | int | finishTimer | 終了後のフレームカウンタ |
| +0x48 | byte | isFinished2 | (別フラグ) |

## バック走行の強制ロジック

### 入力処理の流れ (ReverseChallenge_Update)

1. **入力取得**: InputManager経由でステアリング(dVar15)、アクセル(dVar16)、ブレーキ(dVar14)を取得
2. **自動ブレーキ判定**: アクセル < 0.1 かつ ブレーキ < 0.1 かつ ゲーム開始済み → `TimeTrial_HandleBrakeInput` に `brakeFlag=1` を渡す
3. **ブレーキ長押し**: `brakeTimer` を毎フレーム 1/60 ずつ加算。4.0秒に達するとアクセル入力を **1.0 (後退)** に強制

### yaw角度による自動旋回 (ゴール領域内のみ)

ゴール矩形内にいる場合のみ適用:

```
yawDeg = (int)(KartMovement+0x1C8)  // yaw角度 (度)
yawDeg = yawDeg mod 360             // 360度正規化 (負の場合も考慮)

if (0 <= yawDeg < 170):     steering = 1.0   // 右旋回 → 後ろ向きへ
if (190 < yawDeg < 360):    steering = -1.0  // 左旋回 → 後ろ向きへ
if (170 <= yawDeg <= 190):  steering = (プレイヤー入力そのまま)  // 後方向き
```

- **170度〜190度が「後ろ向き」**: この範囲では自動旋回なし = プレイヤーが自由に操舵
- ゴール領域内ではアクセルも **1.0** に強制 (バック走行を継続)
- ゴール領域外ではプレイヤーの入力がそのまま使われる

### バック走行の仕組み

車が「前向き」にならないよう、ゴールゾーンで自動的にハンドルを切る。カメラはScene3Dの追従カメラで、カートの後方を映す（バック走行なので進行方向を見る形になる）。
`FLOAT_806d4f5c` (= π) がInitでGetCourseStartYaw + π として初期yaw計算に使われており、スポーン時点で180度回転した向き（=コース逆向き）で配置。

## バナナ配置・回避判定

### バナナ配置データ

表/裏でテーブルが異なる:
- **表コース**: `PTR_DAT_806cfcd0[0]` → 0x80495fc0 (8個のバナナ)
- **裏コース**: `PTR_DAT_806cfcd0[1]` → 0x80496100 (8個のバナナ)

各エントリ (16 bytes raw): `[posX:float, 0:int, posZ:float, terminatorFlag:int]`

### BananaPositionData 構造体 (0x10 bytes)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| +0x00 | void* | modelObj | バナナ3Dモデル ("it_a_01_banana") |
| +0x04 | void* | entryArray | バナナエントリ配列 |
| +0x08 | int | count | バナナ数 |
| +0x0C | byte | hasHit | 今フレームでヒットが発生したか |

### バナナエントリ (0x18 = 24 bytes each)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| +0x00 | int | state | 0=active, 1=hit(吹き飛び中), 2=消滅 |
| +0x04 | float | posX | X座標 |
| +0x08 | float | posY | Y座標 (地面高さ補正済み) |
| +0x0C | float | posZ | Z座標 |
| +0x10 | float | hitAngle | ヒット時のカートとの角度 (atan2) |
| +0x14 | float | hitTimer | ヒット後の経過時間 |

### 衝突判定 (BananaField_TestHitBox, 0x80196f60)

OBB (Oriented Bounding Box) ベースの衝突判定:
1. カートの orientMatrix から3軸 (前方, 上方, 側方) を取り出す
2. カート中心からバナナ位置へのベクトルを計算
3. 各軸への射影を計算 (Vec3_Dot)
4. キャラクター別サイズテーブル (`0x8037d9e0`, charId * 0xC) で判定サイズを取得
5. 3軸全てで射影距離がサイズ以内 → 衝突

#### キャラクター別衝突サイズ (0x8037d9e0)

| charId | 横幅 | 高さ | 奥行き |
|--------|------|------|--------|
| 0 (MARIO) | 10.0 | 10.0 | 6.0 |
| 1 (LUIGI) | 10.0 | 10.0 | 6.0 |
| 2 (PEACH) | 11.0 | 10.0 | 7.0 |
| 3 (YOSHI) | 10.0 | 10.0 | 6.0 |

### ヒット処理

バナナにヒットすると:
1. `BananaField_CheckCollision` が `hasHit=1` を設定
2. `ReverseChallenge_Update` で `hasHit` を検出
3. アイテムヒット構造体を構築 (itemId=0x32=バナナ)
4. `CarObject_ApplyItemHit` (0x8004ae28) でスピン処理 → コイン-1

## ゴール判定

### ゴール矩形テスト

2つのチェックポイントの座標をウェイポイントとして定義 (0x80328dd8):

| ウェイポイント | X | Z |
|-----------|------|------|
| WP0 (近い側) | 1045.0 | -340.0 |
| WP1 | 913.4 | 250.0 |
| WP2 | 650.0 | 330.9 |
| WP3 (遠い側) | 650.0 | -150.0 |

`goalEdgeNormals` (self+0x2C) に4辺の法線ベクトルを事前計算:
- 辺0: WP0→WP1 の法線
- 辺1: WP1→WP0 の法線 (逆向き)
- 辺2: WP2→WP3 の法線
- 辺3: WP3→WP2 の法線

毎フレーム、カート位置をこの4辺の半平面テストで判定。全4辺の内側にいればゴール矩形内と判定。

### ゴールライン通過判定

cameraPath (self+0x28) のデータを使用:
1. カート位置とcameraPath始点のクロス積を計算
2. その結果とcameraPath+0x24 (法線) の内積 > 0 で通過と判定
3. 通過時: `reachedGoal=1`, `isFinished=1`, `CarObject_PlayGoalReachedAnim`

### タイムアウト判定

- `FLOAT_806d132c` にMiniGame_GetTimerValueの秒数を格納
- 毎フレーム `1/60` ずつ減算
- 0以下になると: `isFinished=1`, `CarObject_PlayTimeoutAnim`

## 結果処理

### コイン報酬

ゴール到達時のみ、経過時間に基づいてコインを付与:
1. `elapsedTime = totalTime - remainingTime`
2. `TimeTrial_GetCoinDivisor` でコース/cc/表裏に応じた報酬テーブルを取得
3. テーブルの閾値を走査: `elapsedTime <= threshold[i]` を満たす最初のエントリの報酬値を使用
4. `SetCoinCount` で報酬設定

### 結果表示

- ゴール到達 (`reachedGoal=1`): `TimeTrial_CalcResultText(self, 1, 0)` → 成功テキスト + 結果保存
- タイムアウト (`reachedGoal=0`): `TimeTrial_CalcResultText(self, 0, 0)` → 失敗テキスト
- 表示後 `DAT_806d1289=1` でゴール演出カメラモードに移行

### 結果保存 (TimeTrial_SaveResult)

- ゴール到達 + カード有効: 成功として保存、キャラ別効果音再生
- タイムアウト: 失敗として保存
- courseId=0xE の場合のみ追加処理 (キャラ別サウンド)

## カメラシステム

- 通常時: Scene3Dの追従カメラ (カートの後方を映す)
- ゴール領域内: カメラの前方ベクトルをカートに追従させつつスムージング (FLOAT_806d4f5c=π使用)
- ゴール演出: `ChallengeGoalCamera_Update` でカメラを徐々に引き、speedScale=0.5で減速

## 定数まとめ

| アドレス | 値 | 用途 |
|----------|-----|------|
| FLOAT_806d4f48 | 0.8 | DAT_806d1290有効時のアクセル値 |
| FLOAT_806d4f4c | 1.0 | ブレーキ長押し/ゴール内のアクセル強制値 |
| FLOAT_806d4f50 | 0.1 | 入力デッドゾーン |
| FLOAT_806d4f54 | 0.0 | ゼロ定数 |
| FLOAT_806d4f58 | -1.0 | 左旋回ステアリング値 |
| FLOAT_806d4f5c | π | yaw計算、カメラスムージング |
| FLOAT_806d4f60 | 0.5 | ゴール演出時のspeedScale |
| FLOAT_806d4f64 | 1/60 | タイマー減算量 (1フレーム) |
| FLOAT_806d4f68 | 100.0 | 速度→HUD変換係数 |
| 0x168 | 360 | yaw正規化の除算値 (度) |
| 0xAA | 170 | 自動右旋回の上限角度 |
| 0xBE | 190 | 自動左旋回の下限角度 |

## Ghidraリネーム結果

| アドレス | 旧名 | 新名 |
|----------|-------|------|
| 0x800ad910 | TestDrive_Init | ReverseChallenge_Init |
| 0x800ad134 | FUN_800ad134 | ReverseChallenge_Update |
| 0x800acfc0 | FUN_800acfc0 | ReverseChallenge_Render |
| 0x800acd60 | FUN_800acd60 | ReverseChallenge_Dtor |
| 0x80196c5c | FUN_80196c5c | BananaField_CheckCollision |
| 0x80196aa0 | FUN_80196aa0 | BananaField_Render |
| 0x80196f60 | FUN_80196f60 | BananaField_TestHitBox |
| 0x80124ae8 | SuikaBallObj_UpdateGameOverAnim | ChallengeGoalCamera_Update |
| 0x8004e154 | FUN_8004e154 | CarObject_ApplyInput |
| 0x80040748 | FUN_80040748 | CarObject_PlayGoalReachedAnim |
| 0x80040608 | FUN_80040608 | CarObject_PlayTimeoutAnim |
| 0x8004ae28 | FUN_8004ae28 | CarObject_ApplyItemHit |
| 0x8025e368 | (既存) | Vec3_Dot |
