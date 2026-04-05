# Kart Tuning Entry Field Analysis

## Entry Layout (92 bytes = 23 words, stride 0x5C)

**Tables**: DAT_8049d030 (RACE mode), DAT_804bf830 (non-RACE mode)

**Accessor functions**:
- `GetKartTuningEntry` (0x801de560): returns pointer to full entry (word[0])
- `GetKartTuningSpeedFactor` (0x801de4f4): returns word[0] as float
- `FUN_801de448`: returns pointer to word[4] (entry + 0x10), skipping header

### Field Map

| Word | Offset | Type | Name | Used By | Description |
|------|--------|------|------|---------|-------------|
| [0] | +0x00 | float | speedFactor | GetKartTuningSpeedFactor | Speed multiplier (NORMAL=0.6, GOLD=0.5, ORIGINAL=0.4, ENHANCED=0.3) |
| [1] | +0x04 | float | accelRate | PhysicsState_AccelInterpolate, KartPhysics_UpdateSmoothing | Per-frame steer/throttle interpolation rate |
| [2] | +0x08 | float | **(UNUSED)** | -- | No code references found. Values: NORMAL=20.0, GOLD=80.0, ORIGINAL=70.0, ENHANCED=50.0 |
| [3] | +0x0C | float | **(UNUSED)** | -- | No code references found. Values: NORMAL=50.0, GOLD=50.0, ORIGINAL=40.0, ENHANCED=30.0 |
| [4] | +0x10 | float | aheadDistThreshold | FUN_801ea6f0 (GP speed calc) | Distance threshold (squared, scaled by 10000) for speed bonus when AI is AHEAD. Default=10.0 |
| [5] | +0x14 | float | aheadSpeedBonusBase | FUN_801ea6f0 | Base speed bonus when ahead and close. Added to pfVar4[2]. NORMAL=10.0, ENHANCED=5.0 |
| [6] | +0x18 | float | aheadSpeedBonusExtra | FUN_801ea6f0 | Extra speed bonus added to [5]. Usually 0.0 |
| [7] | +0x1C | float | behindDistThreshold | FUN_801ea6f0 | Distance threshold (squared, scaled by 10000) for speed bonus when AI is BEHIND. NORMAL=30.0 |
| [8] | +0x20 | float | behindSpeedBonusBase | FUN_801ea6f0 | Base speed bonus when behind and close. Added to pfVar4[5]. Usually -5.0 |
| [9] | +0x24 | float | behindSpeedBonusExtra | FUN_801ea6f0 | Extra speed bonus added to [8]. Usually 0.0 |
| [10] | +0x28 | float | speedTimerDuration | FUN_801ea6f0, FUN_801eadc8 | Timer frames = 15 * value. Controls how long speed adjust state lasts. NORMAL=180.0 (2700f), ORIGINAL=30.0 (450f) |
| [11] | +0x2C | int | aiVtableSelector | FUN_801e83c8 | 0 or 1. Selects vtable: 0→PTR_804e4bec, 1→PTR_804e4c08 (different steering/correction behaviors) |
| [12] | +0x30 | float | steeringDamping | FUN_801ec4b0, FUN_801ec5a4 | Correction force damping factor. Clamped to [0.0, FLOAT_806da428]. Usually 0.0 |
| [13]-[19] | +0x34-+0x4C | int[7] | aiPattern[0-6] | FUN_801e83c8, EnemyRun | ラップごとの走行パターン値(0-9)。横方向オフセット量を制御。50cc等の低難度では全0。詳細は `mkgp2_ai_pathfinding.md` 参照 |
| [20] | +0x50 | int | aiPathMode | FUN_801ebc8c | If ==1: enable reverse path following (FUN_801e6c68 flag 0), else normal (flag 1) |
| [21]-[22] | +0x54-+0x58 | -- | padding | -- | Always 0 |

## GP Speed Calculation Logic (FUN_801ea6f0)

When `(&DAT_806793f4)[kartIdx * 0xe]` reaches 0, FUN_801eadc8 reinitializes it to:
- `(int)(60.0 * 0.25 * tuning[10])` = `15 * tuning[10]` frames

Each frame this counter decrements. When it reaches 0, the speed boost state resets.

### Ahead Path (`(&DAT_806793e0)[kartIdx*0xe] >= 0`):
```
scaledDist = (10.0 * 1000.0 * tuning[4])^2   // distance threshold
if (rivalDistSq >= scaledDist):
    bonus = tuning[5] + tuning[6]   // speed increase
else:
    bonus = 0.0
```

### Behind Path (`(&DAT_806793e0)[kartIdx*0xe] < 0`):
```
scaledDist = (10.0 * 1000.0 * tuning[7])^2
if (rivalDistSq >= scaledDist):
    bonus = tuning[8] + tuning[9]   // speed decrease (usually negative)
else:
    bonus = 0.0
timer reset: -(int)(60.0 * 0.25 * tuning[10])  // negative → losing state
```

## Per-Variant Comparison (Course 1, CC 0, SubMode 0)

| Field | S0 (Player) | S1 (AI1) | S2 (AI2) | S3 (AI3) |
|-------|-------------|----------|----------|----------|
| [0] speedFactor | 0.6 | 0.5 | 0.4 | 0.3 |
| [1] accelRate | 0.8 | 0.8 | 0.8 | 0.6 |
| [2] (unused) | 20.0 | 80.0 | 70.0 | 50.0 |
| [3] (unused) | 50.0 | 50.0 | 40.0 | 30.0 |
| [4] aheadDist | 10.0 | 10.0 | 10.0 | 10.0 |
| [5] aheadBonus | 10.0 | 10.0 | 10.0 | 5.0 |
| [6] aheadBonusX | 0.0 | 0.0 | 0.0 | 0.0 |
| [7] behindDist | 30.0 | 30.0 | 24.0 | 26.0 |
| [8] behindBonus | -5.0 | -5.0 | -5.0 | -5.0 |
| [9] behindBonusX | 0.0 | 0.0 | 0.0 | 0.0 |
| [10] timerDuration | 180.0 | 180.0 | 30.0 | 30.0 |
| [11] vtableSel | 0 | 1 | 0 | 1 |
