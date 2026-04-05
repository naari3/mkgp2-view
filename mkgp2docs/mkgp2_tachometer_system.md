# MKGP2 タコメーター（スピードメーター）表示システム

## 概要

タコメーターは HUD 上のスピード表示を担当するサブシステム。**物理速度 (velocity) とは独立した「表示速度」** を計算・描画する。プレイヤーカートでのみ動作し、タイムアタックモードでは描画がスキップされる。

## データフロー全体像

```
CarObject_FrameUpdate (毎フレーム、isPlayer==1 のとき)
  │
  ├── KartMovement_CalcCurrentSpeed(kartMovement)
  │     → velocity magnitude * 3.6  (m/s → km/h)
  │     → ※ coinSpeedBonus は含まない (CalcCurrentSpeed を使用)
  │
  ├── KartMovement_CalcMaxSpeed(kartMovement)
  │     → スピードカーブの最大速度 * 3.6  (m/s → km/h)
  │     → 上限キャップ: 55.556 m/s = 200 km/h
  │
  ├── Tachometer_UpdateDisplaySpeed(currentSpeed, maxSpeed)
  │     → displaySpeed を計算、g_tachometer_displaySpeed に格納
  │
  └── Tachometer_SetCoinCount(carObject[0x32])
        → g_tachometer_coinCount, g_tachometer_coinTier を更新

Tachometer_RenderDigits (描画フレーム)
  │
  └── g_tachometer_displaySpeed を読み取り、数字スプライトとして描画
```

## 関数詳細

### Tachometer_UpdateDisplaySpeed (0x800ab220)

**引数:**
- `param_1` (f1): currentSpeed — `KartMovement_CalcCurrentSpeed` の戻り値 (km/h)
- `param_2` (f2): maxSpeed — `KartMovement_CalcMaxSpeed` の戻り値 (km/h)

**計算式:**

```
coinBonus_per_coin = (1.08 * maxSpeed - maxSpeed) / 15.0
                   = (0.08 * maxSpeed) / 15.0
                   = maxSpeed * 0.08 / 15.0

effectiveCoinCount = min(g_tachometer_coinCount, 15)

displaySpeed = currentSpeed + coinBonus_per_coin * effectiveCoinCount
```

つまり:
```
displaySpeed = currentSpeed + maxSpeed * (0.08 / 15.0) * min(coinCount, 15)
```

**コインボーナスの数値例 (maxSpeed = 200 km/h の場合):**
| コイン数 | ボーナス (km/h) | 表示速度 = 実速度 + ボーナス |
|----------|----------------|---------------------------|
| 0        | 0.0            | 実速度のまま |
| 5        | 5.33           | +5.33 km/h |
| 10       | 10.67          | +10.67 km/h |
| 15       | 16.0           | +16.0 km/h (上限) |

最大ボーナス = maxSpeed * 0.08 = **16 km/h** (maxSpeed=200 km/h 時)。

**maxDisplaySpeed の動的更新:**

`maxSpeed > 0` かつ `isInitialized` の場合、`maxSpeed + coinBonus` が現在の `g_tachometer_maxDisplaySpeed` を超えると更新される。これにより表示上限がレース中のカート性能に応じて変動する。

**クランプ処理:**
1. `currentSpeed <= 0` → `displaySpeed = 0`
2. `displaySpeed <= 0` → `displaySpeed = 0`
3. `displaySpeed >= maxDisplaySpeed` → `displaySpeed = maxDisplaySpeed`
4. それ以外 → `displaySpeed` をそのまま使用

### Tachometer_SetCoinCount (0x800ab1bc)

```
g_tachometer_coinCount = param_1
g_tachometer_coinTier =
    param_1 < 1  → 0
    param_1 < 5  → 1
    param_1 < 10 → 2
    param_1 >= 10 → 3
```

coinTier はおそらくタコメーターの視覚効果（色/光沢）に影響。

### Tachometer_RenderDigits (0x800ab324)

**タイムアタック (TIME_ATTACK) モードではスキップ。**

1. `g_tachometer_displaySpeed` を取得
2. 単位変換:
   - `DAT_806d1459 == 1` (km/h): `value = displaySpeed` のまま
   - `DAT_806d1459 != 1` (mph): `value = displaySpeed / 1.6`
3. `intValue = (int)(value * 0.5)` — 表示用整数値
4. 百の位、十の位、一の位に分解
5. 各桁をスプライトルックアップテーブルで描画

**x0.5 の意味:** 内部 displaySpeed は「内部単位系での速度」であり、表示上の km/h は半分の値になる。つまり内部 200 km/h は表示上 100 km/h として描画される。

### Tachometer_Init (0x800ab77c)

- `g_tachometer_isInitialized = 1`
- `DAT_806d145c`: ccClass==0 なら 0、それ以外なら 6（何らかのモード分け）
- `DAT_806d1459`: カード設定 `DAT_80598a84` の byte+2 から取得（デフォルト=1=km/h）
- `g_tachometer_maxDisplaySpeed = 200.0` (初期値)
- `g_tachometer_displaySpeed = 0.0`
- 初回 `UpdateDisplaySpeed(0.0, -1.0)` 呼び出し（結果は displaySpeed=0.0）
- スプライトリソースのプリロード・初期化

## 物理コインボーナスとの関係

### 二重にかかるか？ → **YES、部分的に二重**

**物理系（CarObject_UpdateCoinSpeedBonus → KartMovement+0x2E0）:**
```
coinSpeedBonus = clamp(coinCount * (0.4/15), 0.0, 0.4)
```
- KartMovement+0x2E0 に格納
- 物理速度の position integration に `posScale = 10.0 + coinSpeedBonus` として適用
- 速度表示時に `speed *= (1.0 + coinSpeedBonus)` として適用 (CalcSpeedWithCoinBonus)

**タコメーター表示系（Tachometer_UpdateDisplaySpeed）:**
```
displaySpeed = currentSpeed + maxSpeed * 0.08/15 * min(coinCount, 15)
```
- `currentSpeed` は `CalcCurrentSpeed` の戻り値
- **CalcCurrentSpeed は coinSpeedBonus を含まない** (velocity magnitude * 3.6 のみ)
- つまりタコメーター入力の `currentSpeed` は物理ボーナス適用前の「素の速度」

**結論:**
- **物理的なコインボーナス** (最大+40%) は velocity ベクトルの大きさに影響する（position integration 経由）
- velocity 自体がコインで加速されるので、`CalcCurrentSpeed` の出力にもコインの**間接的な影響**がある
- その上にタコメーター固有の「表示ボーナス」(最大+8%のmaxSpeed) が加算される
- `CalcSpeedWithCoinBonus` はタコメーターには使われていない（サウンド/エフェクト用途）

したがって:
1. コインが多い → 物理加速 → velocity 増加 → CalcCurrentSpeed の出力が高い
2. さらにタコメーター独自のコインボーナス (+maxSpeed*0.08/15*coins) が加算
3. **二重効果**だが、物理ボーナスは velocity 経由の間接効果、タコメーターボーナスは直接加算

## maxDisplaySpeed (200.0) の役割

**スケーリング用ではなく、表示上限 (キャップ) として機能。**

- 初期値: 200.0 (km/h, 内部単位)
- `displaySpeed >= maxDisplaySpeed` のとき、`displaySpeed = maxDisplaySpeed` にクランプ
- ただし **動的更新あり**: `maxSpeed + coinBonus > maxDisplaySpeed` なら `maxDisplaySpeed` を更新
  - これにより、高性能カートやコインボーナスで上限が押し上げられる
  - 200 km/h を超える表示速度も理論上は可能

## km/h と mph の切り替え

- **切り替え変数**: `DAT_806d1459` (1バイト)
- **ソース**: カード設定データ `DAT_80598a84._2_1_` (カード構造体のbyte offset +2)
- **デフォルト値**: 1 (km/h)
- **判定**: `DAT_806d1459 == 1` → km/h表示、それ以外 → mph表示
- **mph 変換**: `displaySpeed / 1.6` してから `* 0.5` で整数化
- **km/h 変換**: そのまま `* 0.5` で整数化

ゲームの設定画面（カード設定）で変更可能と推定される。リソーステーブル（DAT_80411e20）はkm/hとmphで別のスプライトセットを持つ。

## グローバル変数一覧

| アドレス | 名前 | 型 | 説明 |
|----------|------|-----|------|
| 0x806d1458 | g_tachometer_isInitialized | byte | 初期化フラグ |
| 0x806d1459 | DAT_806d1459 | byte | 単位フラグ (1=km/h, 0=mph) |
| 0x806d145c | DAT_806d145c | int | ccClass由来モード値 |
| 0x806d1460 | g_tachometer_coinTier | int | コインティア (0-3) |
| 0x806d1464 | g_tachometer_coinCount | int | タコメーター用コイン数 |
| 0x806d1468 | FLOAT_806d1468 | float | X位置ベース |
| 0x806d146c | FLOAT_806d146c | float | Y位置ベース |
| 0x806d1470 | g_tachometer_displaySpeed | float | 表示速度 |
| 0x806d1474 | FLOAT_806d1474 | float | スクロール/オフセット値 |
| 0x806cf25c | FLOAT_806cf25c | float | スケール値 (初期1.0) |

## 定数一覧

| アドレス | 値 | 名前/用途 |
|----------|-----|----------|
| 0x806d4e60 | 1.08f | coinBonus 係数 (1.0 + 0.08) |
| 0x806d4e64 | 15.0f | 最大有効コイン数 |
| 0x806d4e68 | 0.0f | ゼロ定数 |
| 0x806d4e48 | 0.5f | 表示変換倍率 |
| 0x806d4e4c | 1.6f | km/h → mph 変換係数 |
| 0x806d4e78 | 200.0f | maxDisplaySpeed 初期値 |
| 0x806d4e7c | 1.0f | スケール初期値 |
| 0x806d4e80 | -1.0f | Init時のダミーmaxSpeed |
| 0x806d96dc | 3.6f | m/s → km/h 変換 (CalcCurrentSpeed内) |
| 0x806d96d8 | 55.556f | 速度上限 (200/3.6 m/s) |
