# KartParamBlock 物理パラメータ追証レポート

## 調査対象

KartParamBlock (472 bytes) 内の以下4パラメータが、物理エンジンでどのように使用されるか追証した。

| パラメータ | Offset | 既存説明 |
|-----------|--------|---------|
| mue | +0x28 | グリップ — 路面への摩擦力 |
| gripCoefficient | +0x2C | グリップ補正 — グリップの最低保証値 |
| weight | +0x58 | 重さ — カートの重量。衝突時に影響 |
| accelGainRate | +0x60 | 加速力 — アクセルを踏んだ時の速度の上がりやすさ |

## 調査方法

- `KartMovement_PhysicsStep` (0x8019cd30) のデコンパイル + アセンブリ全文解析
- `KartMovement_Init` (0x8019e20c) の初期化処理確認
- `KartMovement_CollisionUpdate` (0x8019b080) の衝突処理確認
- 周辺関数 (FUN_8019adf4, FUN_8019c6b4, FUN_8019c15c, FUN_8019c078, FUN_8019bf80) のデコンパイル
- `CarObject_MainUpdate` (0x8004d404), `FUN_8004c320` の上位関数確認

### KartParamBlock 参照パターン

PhysicsStep では `r29` = KartMovement*, `lwz rN, 4(r29)` で KartParamBlock ポインタを取得し、`lfs fN, offset(rN)` でフィールドを読む。

---

## 1. mue (offset +0x28)

### 参照元

| 関数 | アドレス | アクセスパターン |
|------|---------|-----------------|
| KartMovement_Init | 0x8019e308 | `lfs f0, 0x28(r3)` → KartMovement+0x38 にコピー |
| KartMovement_PhysicsStep | 0x8019d318 | `lfs f1, 0x38(r29)` (KartMovement+0x38 経由で参照) |

### Init 時のコピー

```
kartMovement[0x0E] = *(kartParamBlock + 0x28)
→ KartMovement+0x38 = paramBlock->mue
```

Init 時に KartParamBlock から KartMovement+0x38 にコピーされる。PhysicsStep 内では KartParamBlock を直接読まず、KartMovement+0x38 を参照する。

### PhysicsStep 内の計算式

アセンブリ (0x8019d318 〜 0x8019d330):
```
lfs f1, 0x38(r29)      # f1 = mue (KartMovement+0x38)
lfs f0, 0x2EC(r29)     # f0 = mueOffset (KartMovement+0x2EC)
lfs f2, 0x2F8(r29)     # f2 = mueScale (KartMovement+0x2F8)
fadds f0, f1, f0       # f0 = mue + mueOffset
fmuls f0, f2, f0       # f0 = mueScale * (mue + mueOffset)
```

デコンパイル:
```c
dVar5 = (float)param_4[0xbe] * ((float)param_4[0xe] + (float)param_4[0xbb]);
// = mueScale * (mue + mueOffset)
```

この計算結果 (`effectiveMue`) は、速度の閾値として使われる:
```c
if (f26 <= effectiveMue) {
    // 速度が mue 閾値以下: 前フレームの速度ベクトルをゼロに減衰 (停止)
    prevVelocity *= 0.0;
    f30 = 1.0;   // accelTarget = 1.0 (paramBlock->damping)
    f31 = paramBlock->damping;
} else {
    // 速度が mue 閾値を超過: 速度ベクトルをスケーリング
    f30 = (effectiveMue / f26) * paramBlock->steeringScale;
    prevVelocity *= f30;

    // gripCoefficient がクランプの下限として機能
    gripFloor = paramBlock->gripCoefficient * kartMovement->mueScale2;
    if (f30 <= gripFloor) f30 = gripFloor;

    f31 = paramBlock->unused08;  // (offset 0x08 = 0.0 なので実質不使用)
}
```

### 実際の役割

**mue は「グリップ」**。現在の速度がこの閾値を超えると、速度ベクトルに減衰が掛かる（スリップ/グリップ制御）。閾値以下なら停止方向に減衰する。

- `mueOffset` (KartMovement+0x2EC) は動的に変化するパラメータ (バウンドや地形の影響)
- `mueScale` (KartMovement+0x2F8) は EffectSpeed システムで設定される乗数 (デフォルト 1.0)

**既存説明「グリップ — 路面への摩擦力」の評価**: おおむね正確。ただし物理的な摩擦力そのものではなく、速度制限の閾値として機能する点が異なる。mue が高いほどスリップしにくい（高速でもグリップを維持できる）。

---

## 2. gripCoefficient (offset +0x2C)

### 参照元

| 関数 | アドレス | アクセスパターン |
|------|---------|-----------------|
| KartMovement_PhysicsStep | 0x8019d36c | `lfs f1, 0x2C(r3)` (r3 = paramBlock) |
| KartMovement_PhysicsStep | 0x8019d454 | `lfs f1, 0x2C(r3)` (r3 = paramBlock) |

### PhysicsStep 内の計算式

**使用箇所 1 (0x8019d364 〜 0x8019d380)**: mue 超過時のクランプ下限

```c
// mue 超過パスで速度スケール f30 が算出された後:
gripFloor = paramBlock->gripCoefficient * kartMovement->mueScale2;
if (f30 <= gripFloor) {
    f30 = gripFloor;  // f30 の最低値を gripCoefficient で保証
}
```

**使用箇所 2 (0x8019d448 〜 0x8019d468)**: 速度回復量のクランプ

```c
if (dotProduct > 0.0) {
    // 進行方向と希望方向が一致する場合、回復量を計算
    correctedAccel = clamp(
        f30 + dotProduct * paramBlock->recoveryRate,  // 回復量上乗せ
        paramBlock->gripCoefficient * kartMovement->mueScale2,  // min
        1.0                                                       // max
    );
}
```

### 実際の役割

**gripCoefficient は「速度スケールの最低保証値」**。mue 閾値を超えてスリップ状態になっても、速度スケール (f30) が gripCoefficient 未満に落ちることを防ぐ。つまり、完全にグリップを失うことはなく、最低限の旋回・制御力が保たれる。

**既存説明「グリップ補正 — グリップの最低保証値」の評価**: **正確**。まさにグリップの最低保証値として機能している。

---

## 3. weight (offset +0x58)

### 参照元

**PhysicsStep 内**: 参照なし
**KartMovement_Init 内**: コピーなし
**KartMovement_CollisionUpdate 内**: 参照なし
**KartPhysics_ApplyCollisionForce 内**: 参照なし
**CarObject_MainUpdate 内**: 参照なし
**FUN_8004c320 内**: 参照なし
**FUN_8019adf4 内**: 参照なし

### 調査結果

PhysicsStep のアセンブリ全文 (0x8019cd30 〜 0x8019e1a4) を走査し、`lwz rN, 4(r29)` で取得した KartParamBlock ポインタ (rN) に対する `lfs fM, 0x58(rN)` パターンを検索したが、**該当なし**。

PhysicsStep 内で paramBlock 経由でアクセスされるオフセット一覧:
0x04, 0x08, 0x18, 0x2C, 0x30, 0x34, 0x38, 0x50, 0x54, **0x60**, 0x64, 0x68, 0x6C, 0x70, 0x74, 0x78, 0x7C, 0x80, 0x84, 0x88, 0x8C

**offset 0x58 (weight) は含まれていない。**

KartMovement_Init でも weight は KartMovement のどのフィールドにもコピーされない (コピーされるのは mue → +0x38 と steeringScale → +0x3C のみ)。

KartMovement_CollisionUpdate (壁衝突)、周辺の地面判定関数、CarObject 上位関数でも weight への参照は見つからなかった。

### 考察

weight パラメータはキャラクター間で明確に異なる値を持つ (軽量級=1.0, 中量級=2.0, 重量級=3.0) ため、設計上の意図は明確にある。しかし、プレイヤーカートの物理エンジン (KartMovement_PhysicsStep) およびその周辺関数群では直接参照されていない。

可能性:
1. **AI カート間の衝突計算で使用** — AI は KartParamBlock を使わず独自初期化だが、カート間衝突判定の別システムで参照される可能性
2. **未使用パラメータ** — 開発中に使われなくなった可能性
3. **PhysicsState の衝突処理 (KartController 側)** で間接的に参照される経路が存在する可能性

**既存説明「重さ — カートの重量。衝突時に影響」の評価**: プレイヤーの KartMovement 物理系統では使用が確認できなかった。衝突計算での使用は未確認。説明が正確かどうかは追加調査が必要。

---

## 4. accelGainRate (offset +0x60)

### 参照元

| 関数 | アドレス | アクセスパターン |
|------|---------|-----------------|
| KartMovement_PhysicsStep | 0x8019dacc | `lfs f1, 0x60(r3)` (r3 = paramBlock) |

### PhysicsStep 内の計算式

アセンブリ (0x8019dac4 〜 0x8019daec):
```
lwz r3, 4(r29)         # r3 = paramBlock
lfs f0, 0x188(r29)     # f0 = prevVelocityX
lfs f1, 0x60(r3)       # f1 = accelGainRate
fmuls f0, f0, f1       # f0 = prevVelocityX * accelGainRate
stfs f0, 0x188(r29)    # prevVelocityX = prevVelocityX * accelGainRate
lfs f0, 0x18C(r29)     # f0 = prevVelocityY
fmuls f0, f0, f1       # f0 = prevVelocityY * accelGainRate
stfs f0, 0x18C(r29)    # prevVelocityY = prevVelocityY * accelGainRate
lfs f0, 0x190(r29)     # f0 = prevVelocityZ
fmuls f0, f0, f1       # f0 = prevVelocityZ * accelGainRate
stfs f0, 0x190(r29)    # prevVelocityZ = prevVelocityZ * accelGainRate
```

デコンパイル:
```c
// velocity を prevVelocity にコピーした直後:
param_4[0x62] = velocity.x;   // prevVelocity.x = velocity.x
param_4[99]   = velocity.y;   // prevVelocity.y = velocity.y
param_4[100]  = velocity.z;   // prevVelocity.z = velocity.z

// accelGainRate で減衰:
fVar12 = *(float *)(param_4[1] + 0x60);  // accelGainRate
param_4[0x62] *= fVar12;  // prevVelocity.x *= accelGainRate
param_4[99]   *= fVar12;  // prevVelocity.y *= accelGainRate
param_4[100]  *= fVar12;  // prevVelocity.z *= accelGainRate
```

### PhysicsStep 内での prevVelocity の使われ方

prevVelocity (KartMovement+0x188〜0x190) は、次フレームの PhysicsStep 冒頭で「前フレームの残存速度」として参照される。mue 閾値との比較や、加速/減速の計算に使われる。

accelGainRate は **速度のフレーム間持続率（慣性）** として機能する:
- accelGainRate = 0.00125 (Mario) → prevVelocity は毎フレーム 0.125% だけ残存
- この値が大きいほど前フレームの速度を多く引き継ぐ → 加速が効きやすい

ただし、名前と値のスケールから判断すると、これは「速度の減衰率」ではなく「前フレーム速度の寄与度」。PhysicsStep の先頭では prevVelocity.magnitude が velocity.magnitude と比較され、速度の変化率を算出するベースラインとして使われる。

### 実際の役割

**accelGainRate は「前フレーム速度の残存率」**。毎フレーム velocity → prevVelocity にコピーした後、prevVelocity にこの係数を掛けることで、次フレームの「基準速度」を減衰させる。この値が大きいほど、前フレームの速度が次フレームに影響し、加速の立ち上がりが速くなる。

**既存説明「加速力 — アクセルを踏んだ時の速度の上がりやすさ」の評価**: 方向性は合っている。ただし「加速力」そのものではなく、前フレーム速度の残存率（慣性ダンピング）として間接的に加速特性に影響する。値が大きいほど速度変化が累積しやすく、結果として加速が効きやすい。

---

## まとめ

| パラメータ | 物理的役割 | 既存説明の正確さ |
|-----------|-----------|-----------------|
| **mue** (+0x28) | グリップ。超過するとスリップ減衰 | おおむね正確。「摩擦力」ではなく「速度閾値」が正しい |
| **gripCoefficient** (+0x2C) | スリップ時の速度スケール最低保証値 | **正確** |
| **weight** (+0x58) | KartMovement_PhysicsStep では未使用 | 確認不能。物理ステップでの参照なし |
| **accelGainRate** (+0x60) | 前フレーム速度の残存率（慣性ダンピング） | 方向性は合うが「加速力」より「慣性」が正確 |

### 各パラメータの計算フロー図

```
[PhysicsStep 毎フレーム]

1. prevVelocity *= damping      (KartMovement+0x1BC)
2. velocity = driveForce + ...  (ステアリング、地形、コイン等)
3. effectiveMue = mueScale * (mue + mueOffset)
4. if |accelerationVector| > effectiveMue:
       velocityScale = (effectiveMue / |accelerationVector|) * steeringScale
       velocityScale = max(velocityScale, gripCoefficient * mueScale2)  ← gripCoeff
       prevVelocity *= velocityScale
   else:
       prevVelocity = 0  (停止方向)
5. accelTarget → mue 依存で段階的に変化 (scale30, scale34, scale38)
6. velocity += acceleration
7. velocity → prevVelocity にコピー
8. prevVelocity *= accelGainRate  ← accelGainRate
9. position += velocity * (posScale + coinSpeedBonus) * simTimeStep
```
