# MKGP2 スピードカーブ データフロー解析

## 概要

KartParamBlock 内の speedTablePtr (+0xF0) と speedCurve (+0x100) がコード上でどのように使われるかを追跡し、速度カーブから最終速度への完全なデータフローを解明した。

> **訂正 (2026-04-03)**: 以前は「speedCurve は物理エンジンに直接影響しない」としていたが、これは **誤り** であった。
> SpeedCurve_UpdateSpeedCap (0x8019ef90) の発見により、スピードカーブが KartMovement+0x10 (speedCap) を経由して
> PhysicsStep の駆動力スケーリングに影響することが判明した。以下のドキュメントは修正済みの正確な情報を記載している。

## KartParamBlock の速度関連フィールド

| Offset | Type | 名前 | 説明 |
|--------|------|------|------|
| +0x04 | float | damping | 加速中のブレーキ係数 (PhysicsStep内) |
| +0x08 | float | unused08 | 減速中のブレーキ係数 (PhysicsStep内) |
| +0x18 | float | scale18 | impulse → 速度変換係数 |
| +0x1C | float | maxInternalSpeed | 内部最大速度 (KartMovement+0x38にコピー) |
| +0x28 | float | mue | 摩擦係数 → 速度上限の基盤 |
| +0x2C | float | gripCoefficient | 最小速度倍率 |
| +0x30 | float | accelUpRate | 加速レート (mue → accelMue の補間速度) |
| +0x34 | float | decelDownRate | 減速レート |
| +0x38 | float | brakeRate | ブレーキレート |
| +0x50 | float | scale50 | 摩擦方向係数 |
| +0x54 | float | scale54 | 摩擦逆方向係数 |
| +0x60 | float | velocityDecay | 速度減衰率 (残速度保存率) |
| +0x88 | float | speedDistScale1 | 距離→速度変換スケール (1/val で使用) |
| +0x8C | float | speedDistScale2 | 走行距離スケール (1/val で使用) |
| +0xF0 | ptr | speedTablePtr | SpeedTable 配列へのポインタ |
| +0xF4 | int | speedTableCount | SpeedTable エントリ数 (通常 7) |
| +0x100〜+0x12F | float[12] | speedCurve | 6ペア: {distance, speed} |

## SpeedTable 構造

### SpeedTableArray (speedTablePtr が指す先)

各エントリ 0x18 (24) バイト:

| Offset | Type | 名前 | 例 (Mario NORMAL) |
|--------|------|------|-------------------|
| +0x00 | ptr | curveDataPtr | 速度カーブデータへのポインタ |
| +0x04 | int | curvePointCount | 7 |
| +0x08 | float | distThreshold | 5000.0 |
| +0x0C | float | maxSpeed | 21.0 |
| +0x10 | float | param10 | 0.1 |
| +0x14 | float | param14 | 0.3 |

### CurveData (curveDataPtr が指す先)

`curvePointCount` 個の {distance, speed} ペア (各 8 バイト):

```
Mario NORMAL (index 0):
  [0] dist=    0.0, speed= 0.000
  [1] dist=  500.0, speed= 7.228
  [2] dist= 1000.0, speed=14.456
  [3] dist= 2000.0, speed=21.685
  [4] dist= 3000.0, speed=28.913
  [5] dist= 4000.0, speed=36.141
  [6] dist= 5000.0, speed=43.370

Mario ENHANCED (index 0):
  [0] dist=    0.0, speed= 0.000
  [1] dist=  500.0, speed= 7.599
  [2] dist= 1000.0, speed=15.197
  [3] dist= 2000.0, speed=22.796
  [4] dist= 3000.0, speed=30.394
  [5] dist= 4000.0, speed=37.993
  [6] dist= 5000.0, speed=45.592
```

### SpeedTable配列内のデータ共有

7つのエントリのうち、index 0 のみがバリアント固有のカーブデータを持つ。index 1〜6 は NORMAL / ORIGINAL / ENHANCED で同じポインタ (共通データ) を共有している。

## speedTableIndex (KartMovement+0x08)

- KartMovement_Init (0x8019e20c) で **0** に初期化
- コード上で変更される箇所は確認されなかった
- **結論: speedTableIndex は常に 0 のまま**
- SpeedTable 配列に7エントリ存在するが、index 1〜6 は未使用（将来の拡張用か、別のゲームモード用の残骸）

## データフロー図 (修正版)

```
KartParamBlock
 ├─ speedTablePtr (+0xF0) ──→ SpeedTableArray[7]
 │                                ├─ [0].curveDataPtr ──→ CurveData (バリアント固有)
 │                                ├─ [0].distThreshold = 5000.0
 │                                ├─ [1-6].curveDataPtr ──→ 共通CurveData
 │                                └─ ...
 ├─ mue (+0x28) ─────────────→ 物理速度上限の基盤
 ├─ maxInternalSpeed (+0x1C) → KartMovement+0x38 (最大速度)
 └─ damping/accelRate等 ─────→ PhysicsStep の加速・減速パラメータ

KartMovement_Init (0x8019e20c)
 ├─ paramBlockPtr (+0x04) = KartParamBlock*
 ├─ speedTableIndex (+0x08) = 0 (固定)
 ├─ travelProgress (+0x0C) = 0 → 走行中に 0〜5000 へ漸増
 ├─ speedCap (+0x10) = 0 → SpeedCurve_UpdateSpeedCap が毎フレーム更新
 ├─ speedTablePtr (+0x24) = paramBlock->speedTablePtr
 └─ speedTableCount (+0x28) = paramBlock->speedTableCount

=== 物理速度決定パス (最重要) ===

┌─────────────────────────────────────────────────────────┐
│ SpeedCurve_UpdateSpeedCap (0x8019ef90) — 毎フレーム呼出 │
│                                                         │
│ entry = speedTable[speedTableIndex]                     │
│ travelProgress = KartMovement+0x0C (0→5000に漸増)      │
│ curveData を travelProgress で補間して速度値を算出      │
│ → 結果を KartMovement+0x10 (speedCap) にキャッシュ     │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ PhysicsStep (0x8019cd30)                                │
│                                                         │
│ param_4[4] = KartMovement+0x10 (speedCap) を参照       │
│ → 駆動力のスケーリングに使用                            │
│ → speedCap の値が物理的な最高速度を決定する             │
└─────────────────────────────────────────────────────────┘

=== 表示速度パス ===

┌─────────────────────────────────────────────────────────┐
│ KartMovement_CalcMaxSpeed (0x801999e0)                   │
│                                                         │
│ entry = speedTable[speedTableIndex]                     │
│ for each point in curveData:                            │
│   if |point.dist - entry.distThreshold| < 1:            │
│     return point.speed * 3.6 (km/h)                     │
│                                                         │
│ → 実質的に curveData の最後のエントリ                    │
│   (dist == distThreshold) の speed を返す                │
└─────────────────────┬───────────────────────────────────┘
                      │
       ┌──────────────┴──────────────────────┐
       │                                     │
CarObject_Init              FUN_8004decc (毎フレーム)
(初期表示)                  (タコメーター更新)
       │                                     │
       └─────────→ Tachometer_UpdateDisplaySpeed
                                │
                         タコメーター表示速度
```

### 速度カーブの使用箇所

#### 1. 物理速度への影響 (SpeedCurve_UpdateSpeedCap)

**SpeedCurve_UpdateSpeedCap (0x8019ef90)** が毎フレーム呼ばれ、以下の処理を行う:

1. KartMovement+0x0C (`travelProgress`, 走行進捗: 0→5000に漸増) を取得
2. speedTable[speedTableIndex] のカーブデータを travelProgress で参照
3. 結果の速度値を **KartMovement+0x10 (`speedCap`)** にキャッシュ
4. PhysicsStep (0x8019cd30) が `param_4[4]` (= KartMovement+0x10) を駆動力のスケーリングに使用

これにより、**スピードカーブの最大値が物理的な最高速度を決定する**。

> **以前は「speedCurve は物理エンジンに直接影響しない」としていたが誤りであった。**
> SpeedCurve_UpdateSpeedCap を経由して speedCap にキャッシュされ、PhysicsStep の駆動力に影響する。

#### 2. タコメーター (HUD)

CalcMaxSpeed の戻り値をタコ表示に反映。

#### 3. サウンド/進行率計算 (FUN_8004baac)

現在速度 / (0.9 * maxCurveSpeed) で進行率を計算。走行距離 (KartMovement+0x0C) / distThreshold でも進行率を計算。

#### 4. エフェクト系 (FUN_8004edd4)

currentSpeed / (0.9 * maxCurveSpeed) をクランプしてエフェクト制御。

### 物理速度計算 (PhysicsStep, 0x8019cd30)

PhysicsStep は speedCurve のテーブルを直接参照しないが、**SpeedCurve_UpdateSpeedCap がキャッシュした speedCap (KartMovement+0x10) を `param_4[4]` として受け取り、駆動力のスケーリングに使用する。**

```
speedCap = param_4[4]   // = KartMovement+0x10, SpeedCurve_UpdateSpeedCap が設定
目標速度限界 = mueScale * (paramBlock->mue + mueBonus)

if 現在の力 > 目標速度限界:
    減速: dampFactor = (限界/力) * paramBlock->dampFactor
    brakeRate = paramBlock->unused08
else:
    加速: impulse リセット
    brakeRate = paramBlock->damping

accelMue の補間:
  if accelMue < 目標:
    accelMue += paramBlock->accelUpRate
  else:
    accelMue -= paramBlock->decelDownRate

加速度ベクトル *= accelMue
// speedCap が駆動力の上限スケーリングに影響
velocity += 加速度
```

### 位置積分 (PhysicsStep末尾)

```
posScale = FLOAT_806d973c + coinSpeedBonus   // 10.0 + coinBonus
deltaPos = velocity * FLOAT_806d9780 * (1 + FLOAT_806d9790) * posScale
position += deltaPos
```

## speedTablePtr チェーン構造

各バリアントの KartParamBlock+0xF0 (speedTablePtr) は**前のバリアント**のスピードテーブルを指す:

**ピーチ (charId=3) 150cc:**

| Variant | speedTablePtr | 参照先 | カーブ最大値 | 実測物理速度 |
|---------|--------------|--------|------------|------------|
| NORMAL | 0x803949e0 | 前ccブロックのテーブル | (未計測) | (未計測) |
| ORIGINAL | 0x80394bb8 | NORMAL+0x130 | 78.06 | 78.06 |
| ENHANCED | 0x80394d90 | ORIGINAL+0x130 | 78.89 | 78.89 |

**ヨッシー (charId=7) 150cc:**

| Variant | speedTablePtr | 参照先 | カーブ最大値 | 実測物理速度 |
|---------|--------------|--------|------------|------------|
| NORMAL | 0x80398778 | 前ccブロックのテーブル | 76.66 | (未計測) |
| ORIGINAL | 0x80398950 | NORMAL+0x130 | 78.06 | 78.06 |
| ENHANCED | 0x80398b28 | ORIGINAL+0x130 | 78.89 | 78.89 |

ピーチとヨッシーは同じ GOLD クラス (classIdx=1) であり、カーブ最大値が一致する。

### 全13キャラ共通パターン

- ENHANCED の speedCurve 自体は全キャラでフラットカーブ (66.67) だが、speedTablePtr は ORIGINAL のスピードテーブルを指す
- ORIGINAL の speedTablePtr は NORMAL のスピードテーブルを指す
- 結果として、各バリアントの speedCap は**前バリアントのスピードカーブの最大値**になる

つまり ENHANCED の実効 speedCap は ORIGINAL のカーブ最大値 (78.89) であり、ENHANCED 自身のフラットカーブ (66.67) は SpeedCurve_UpdateSpeedCap が参照する先ではない。

## NORMAL vs ORIGINAL vs ENHANCED の速度差

### 動的テスト結果 (ピーチ 150cc, フラット路面)

| バリアント | velocity (内部) | displaySpeed | 表示 (km/h) |
|-----------|----------------|-------------|------------|
| NORMAL | - | ≈ 276 | 138 |
| ORIGINAL | 78.06 | ≈ 281 | 140 |
| ENHANCED | 78.89 | ≈ 284 | 142 |

### 速度差の原因

速度差は **speedTablePtr チェーン** による speedCap の違いに起因する:
- ORIGINAL の speedCap = NORMAL のカーブ最大値 (78.06)
- ENHANCED の speedCap = ORIGINAL のカーブ最大値 (78.89)
- speedCap が PhysicsStep の駆動力上限を決定するため、物理速度に差が出る

> **以前の誤りの訂正**:
> - 「ORIGINAL vs ENHANCED の走行物理は同一」→ **誤り**。speedCap が異なるため最高速が異なる。
> - 「ENHANCEDはフラットカーブなので加速が一瞬で完了する」→ 部分的に正しいが、speedCapの決定メカニズムの理解が不足していた。ENHANCED自身のフラットカーブ (66.67) は speedCap に直接影響せず、speedTablePtr が指す ORIGINAL のカーブの最大値 (78.89) が speedCap になる。

## KartMovement フィールド名更新

| オフセット | 旧名 | 新名 | 説明 |
|-----------|------|------|------|
| +0x0C | accelTarget | `travelProgress` | 走行進捗 (0→5000に漸増) |
| +0x10 | (名前なし) | `speedCap` | 速度キャップ (SpeedCurve_UpdateSpeedCap が算出) |

## プレイヤーCarObjectの発見

プレイヤーの CarObject は `DAT_80679598` の KartController 配列に含まれていない。
`g_playerCarObject` (DAT_806d1098) に別途保存されている。
DAT_80679598[0-4] は AI 用の KartController のみ。

## まとめ

1. **speedCurve は SpeedCurve_UpdateSpeedCap (0x8019ef90) 経由で物理速度に影響する** — KartMovement+0x10 (speedCap) にキャッシュされ、PhysicsStep の駆動力スケーリングに使用される
2. **speedTablePtr チェーン構造** により、各バリアントの speedCap は前バリアントのカーブ最大値で決定される
3. **speedTableIndex は常に 0** — 7エントリの speedTable は拡張用に用意されているが未使用
4. **NORMAL / ORIGINAL / ENHANCED の最高速差は speedCap の違いに起因する** — チェーン構造により ENHANCED > ORIGINAL > NORMAL の順に高速
