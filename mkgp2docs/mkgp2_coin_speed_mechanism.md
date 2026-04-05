# MKGP2 コイン速度ボーナスの仕組み

## 概要

レース中に収集したコインはカートの速度を直接増加させる。カートのコイン数から毎フレーム速度ボーナスが計算され、位置積分と表示速度の両方に倍率として適用される。

## データフロー

```
CoinSystem_UpdateAll (0x80139898)
  case 3: kartData+0x2cc += 1   (コイン取得, カートごとのカウント)
       |
       v
CarObject_MainUpdate (0x8004d404)
  carObj+0xC8 = kartData+0x2cc  (毎フレームコイン数をキャッシュ)
       |
       v
CarObject_UpdateCoinSpeedBonus (0x8004cf64)
  rawBonus = coinCount * 0.02666667  (= 0.4/15)
  coinBonus = clamp(rawBonus, 0.0, 0.4)
  carObj+0xCC += 0.1 ずつ coinBonus に接近  (補間ランプ)
  kartMovement+0x2E0 = carObj+0xCC
       |
       v
KartMovement_PhysicsStep (0x8019cd30)
  positionScale = 10.0 + kartMovement+0x2E0   (位置積分)
       |
       v
KartMovement_CalcSpeedWithCoinBonus (0x80199b10)
  speed = |velocity| * (1.0 + kartMovement+0x2E0) * 3.6  (速度表示)
```

## 計算式

```
coinSpeedBonus = clamp(coinCount * (0.4 / 15), 0.0, 0.4)
```

| コイン数 | ボーナス | 速度倍率 |
|----------|---------|---------|
| 0        | 0.000   | 100.0%  |
| 1        | 0.027   | 102.7%  |
| 5        | 0.133   | 113.3%  |
| 10       | 0.267   | 126.7%  |
| 15       | 0.400   | 140.0% (上限) |
| 20+      | 0.400   | 140.0% (上限到達済) |

- **15枚** で最大ボーナス **+40%** に到達。
- 15枚を超えてもさらなる速度効果はない。
- ボーナスは1フレームあたり0.1ずつ目標値に接近する補間方式（即座には反映されない）。

## 主要アドレス

| アドレス        | 種別   | 説明                                           |
|----------------|--------|------------------------------------------------|
| `0x8004cf64`   | 関数   | `CarObject_UpdateCoinSpeedBonus` - 毎フレームボーナス計算 |
| `0x80199b10`   | 関数   | `KartMovement_CalcSpeedWithCoinBonus` - ボーナス込み速度計算 |
| `0x8019cd30`   | 関数   | `KartMovement_PhysicsStep` - 位置積分でボーナスを使用 |
| `0x80139898`   | 関数   | `CoinSystem_UpdateAll` - コイン衝突/ステートマシン |
| `kartData+0x2cc` | int  | カートごとのレース中コイン数                   |
| `carObj+0xC8`    | int  | キャッシュ済みコイン数 (毎フレームコピー)      |
| `carObj+0xCC`    | float | 補間済みコインボーナス (目標値に向かってランプ) |
| `carObj+0xFE`    | byte | コイン速度ボーナス有効フラグ (常に1)           |
| `kartMovement+0x2E0` | float | "mueAddition" - アクティブなコイン速度ボーナス (KartMovementオブジェクト内) |

## 定数

| アドレス       | 値       | 役割                          |
|---------------|----------|-------------------------------|
| `0x806d2784`  | 0.02667  | コイン1枚あたりの係数 (0.4/15) |
| `0x806d2788`  | 0.4      | 最大ボーナス上限              |
| `0x806d26e8`  | 0.1      | 1フレームあたりの補間ステップ |
| `0x806d973c`  | 10.0     | 位置積分のベーススケール      |
| `0x806d96d4`  | 1.0      | 速度計算のベース倍率          |
| `0x806d96dc`  | 3.6      | 内部単位→km/h変換係数         |

## 適用箇所

コインボーナス (`kartMovement+0x2E0`) は2箇所で速度に影響する:

1. **位置積分** (`KartMovement_PhysicsStep` 内):
   ```
   posScale = 10.0 + coinBonus
   position += velocity * posScale * dt
   ```
   カートがコース上で実際に速く移動するようになる。

2. **速度大きさ** (`KartMovement_CalcSpeedWithCoinBonus` 内):
   ```
   speed = |velocity| * (1.0 + coinBonus) * 3.6
   ```
   HUDスピードメーター表示に影響し、内部の速度比較にも使用される。

## 別システム (コイン非関連)

`KartMovement_SetSpeedScale` (+0x2D8) と `KartMovement_SetMueScale` (+0x2F8) は KartMovement オブジェクト内の `EffectSpeed` システムの一部で、アイテム効果 (キノコブースト、バナナ減速等) に使用される。これらはコイン速度ボーナスとは独立している。

> **注意**: これらの関数は旧ドキュメントで `PhysicsState_Set*` と命名されていたが、操作対象はKartMovement (CarObject+0x28) であり、PhysicsState (KartController+0x20) ではない。

## タコメーター (HUDスピードメーター) 追加ボーナス

コインボーナスは物理速度だけでなく、タコメーター表示にも別途ボーナスが加算される:

- `Tachometer_UpdateDisplaySpeed` (0x800ab220): `displaySpeed = currentSpeed + maxSpeed * 0.08/15 * min(coinCount, 15)`
- 最大15枚で +8% の表示ボーナス (物理ボーナスとは別枠)
- `Tachometer_SetCoinCount` (0x800ab1bc): コインティア設定 (0=0枚, 1=1-4枚, 2=5-9枚, 3=10枚以上)
- `g_tachometer_displaySpeed` (0x806d1470): 表示専用速度値

## 初回調査で見落とした理由

初回調査では `GetCoinCount`/`DAT_806d0440` (リザルト画面用のグローバル合計コインカウンタ) と速度の直接的な接続を探した。レース中のコイン数は `kartData+0x2cc` というカートごとのフィールドに格納されており、グローバルカウンタとは完全に別物だった。速度ボーナスは `CarObject_UpdateCoinSpeedBonus` 内でキャッシュ済みのカートごとコイン数から計算されるため、グローバルの `DAT_806d0440` を追っても見つからなかった。
