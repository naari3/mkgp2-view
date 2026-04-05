# MKGP2 AI パスファインディングシステム

## 概要

AIカートの走行経路システムは以下のコンポーネントから構成される:

1. **ウェイポイント周回パス** (複数バリアント): コースごとに最大6本の走行ラインを保持
2. **EnemyRun コントローラ** (2種類): ステアリング応答の計算方式を切り替え
3. **ThinkBox マネージャ** (GP/VS/TA等): 速度制御・アイテム使用の上位ロジック
4. **チューニングテーブル**: スロットごとの速度・ステアリング・パスパターンを定義

---

## 1. クラス階層

### KartController (各AIスロットに1個)

```
KartController (FUN_801e83c8, 0xF4 bytes)
  +0x00: vtable
  +0x10: kartObj (CarObject)
  +0x14: physicsObj (PhysicsState)
  +0x18: soundObj
  +0x1C: collisionObj
  +0x20: enemyRunCtrl   ← ステアリング計算 (clEnemyRunType1 or Type2)
  +0x24: itemCtrl        ← アイテム使用ロジック
  +0x28: pathFollower    ← ウェイポイント追従
  +0x2C: driveBehavior   ← 走行状態管理 (加減速・ドリフト等)
  +0x30: steeringMgr     ← ステアリングサブシステム (11個のサブオブジェクト)
  +0x34: ??? 
  +0x78: pathSlotId
  +0x7C: pathSystemRef (DAT_806d1924)
  +0x80: kartSlotIdx
  +0x84: aiPattern (param_4 % 10)  ← チューニングテーブルの値
  +0x88: kartSlotCopy
  ...
```

### EnemyRun (ステアリング応答)

```
clEnemyRunType1 (aiVtableSelector=0, vtable=804e4c08)
  基底: FUN_801f0644 (0x198 bytes)
  ステアリング: FUN_801ec5a4 (比例方式)
    steer = (0.6 * dampingFactor * 0.01 + 0.1) * angleError

clEnemyRunType2 (aiVtableSelector=1, vtable=804e4bec)
  基底: FUN_801f0644 (0x198 bytes)
  ステアリング: FUN_801ec4b0 (微分方式)
    steer = -(0.25 * angleError - (FLOAT_806d1948 * dampingFactor * 0.01))
```

### EnemyRun 内部レイアウト (0x198 bytes)

```
  +0x00: vtable
  +0x04: pathFollowerRef (FUN_801dc320 オブジェクト)
  +0x08: kartObj
  +0x0C: ???
  +0x10: kartSlot
  +0x14: pathSlotId (8スロット管理)
  +0x18: aiPattern (0-9, チューニングテーブル値)
  ...
  +0x4C-0x54: 現在位置
  +0x64: isInitialized
  +0x7C: 現在ヘディング角
  +0x84: スムーズド角度
  +0x88: コーナリングフラグ
  +0x9D: ライバル回避中フラグ
  ...
```

---

## 2. ウェイポイントシステム

### パスデータ構造

```
PathSystem (DAT_806d1924)
  +0x00: totalWaypointCount (周回パスの総ウェイポイント数)
  +0x04: currentIndex
  +0x08: ???
  +0x0C: waypointArray → Waypoint[totalCount]
```

各ウェイポイントは 16 bytes:
```
Waypoint (0x10 bytes)
  +0x00: int  linkIndex (次のウェイポイント、-1=終端)
  +0x04: float posX
  +0x08: float posY
  +0x0C: float posZ
```

パスは循環リスト。FUN_800463cc で `(currentIndex + 1) % totalCount` に進む。

### パスバリアント (0-5)

コースごとに最大6本の走行ラインが存在する。`FUN_800abe48(courseData, variantIdx)` でバリアントを取得。

各バリアントは同じウェイポイント形式だが、ウェイポイント座標が異なる（コース幅の異なる位置を通る）。

---

## 3. パスバリアント選択

### 選択テーブル (804e36a8, 73エントリ)

FUN_801dcb18 がAIカート初期化時にパスバリアントを決定する。

エントリ形式 (20 bytes):
```
+0x00: u8  gameMode    (0xFF=任意)
+0x01: u8  courseId    (0xFF=任意)
+0x02: u8  ccClass    (0xFF=任意)
+0x03: u8  subMode    (0xFF=任意)
+0x04: u32 padding
+0x08: int kartSlot   (0-5, 0xFF=任意)
+0x0C: int pathIdxA   (ランダム値 <= 0.5 のとき選択)
+0x10: int pathIdxB   (ランダム値 > 0.5 のとき選択)
```

テーブルは優先度の高い順に並び、最初にマッチしたエントリが使われる。

#### 選択ロジック (FUN_801dcb18)

1. テーブルをスキャンし、gameMode / courseId / ccClass / subMode / カートのlap進捗(0x23Cオフセット) にマッチするエントリを探す
2. 擬似乱数シーケンス (FUN_801eb80c) の値 vs 閾値 0.5 でpathIdxA/pathIdxBを選択

#### 擬似乱数シーケンス (FUN_801eb80c)

16個の固定値をカウンタ DAT_806d192c で巡回:
```
[0.625, 0.25, 1.0, 0.0625, 0.125, 0.75, 0.4375, 0.9375,
 0.375, 0.6875, 0.1875, 0.875, 0.5625, 0.8125, 0.3125, 0.5]
```
閾値 0.5 と比較し、pathIdxA (<=0.5) または pathIdxB (>0.5) を返す。

#### テーブル抜粋

**マリオカップ (courseId=5), 150cc (ccClass=2):**
- subMode=5: slot0→path5, slot1→1, slot2→4, slot3→2, slot4→3, slot5→1
- subMode=6: 同一マッピング

**ワイルドカードエントリ (全モード, 全コース):**
- slot0→path5/4, slot1→1/2, slot2→3/4, slot3→2/3, slot4→3/5, slot5→1/5
  (A/Bは乱数で分岐)

→ **スロットごとに異なるパスバリアントを走行する**。同じスロットでも乱数によってA/Bの2択が存在。

---

## 4. aiPattern[0-6] の役割

### チューニングテーブル上の位置

```
TuningEntry (0x5C = 92 bytes)
  [0]  +0x00: float speedFactor
  [1]  +0x04: float accelRate
  [2]  +0x08: float (未使用)
  [3]  +0x0C: float (未使用)
  [4]  +0x10: float gpAheadDistThreshold
  [5]  +0x14: float gpAheadSpeedBonusBase
  [6]  +0x18: float gpAheadSpeedBonusExtra
  [7]  +0x1C: float gpBehindDistThreshold
  [8]  +0x20: float gpBehindSpeedBonusBase
  [9]  +0x24: float gpBehindSpeedBonusExtra
  [10] +0x28: float gpSpeedTimerDuration
  [11] +0x2C: int   aiVtableSelector (0 or 1)
  [12] +0x30: float steeringDamping
  [13] +0x34: int   aiPattern[0]    ← ★
  [14] +0x38: int   aiPattern[1]
  [15] +0x3C: int   aiPattern[2]
  [16] +0x40: int   aiPattern[3]
  [17] +0x44: int   aiPattern[4]
  [18] +0x48: int   aiPattern[5]
  [19] +0x4C: int   aiPattern[6]    ← ★
  [20] +0x50: int   aiPathMode (1=reverse)
  [21] +0x54: (padding)
  [22] +0x58: (padding)
```

### aiPattern の値と消費

FUN_801e83c8 (AI初期化) で `param_4` を受け取り:
```c
kart[0x84] = param_4 % 10;  // aiPattern (0-9)
```
この値は EnemyRun コンストラクタの第6引数として渡され、`EnemyRun[6]` (offset 0x18) に格納される。

**値の範囲**: 0-9 の整数。0が最も基本的で、9が最も特殊。

### 実データ例 (ワルイージカップ courseId=7, 150cc, subMode=5)

| Slot | aiPattern[0-6] |
|------|----------------|
| 0    | 0, 1, 2, 3, 6, 1, 5 |
| 1    | 7, 8, 9, 4, 7, 2, 6 |
| 2    | 2, 3, 4, 5, 8, 3, 5 |
| 3    | 5, 6, 7, 3, 9, 0, 6 |
| 4    | 3, 4, 0, 4, 8, 2, 5 |

**注意**: マリオカップ/ヨシーカップの50ccではaiPattern[0-6]が全て0。これは難易度が低いコースでパターン分化が不要なため。

### aiPattern の消費先

aiPatternは `FUN_801e83c8` 内で `kart[0x84]` に格納されるが、**7個の配列のうちどの要素が選ばれるかはラップ/周回に依存**する計算がある:
```c
iVar5 = param_4 / 10 + (param_4 >> 0x1f);
param_1[0x21] = param_4 + (iVar5 - (iVar5 >> 0x1f)) * -10;
```
ここで `param_4` は `tuning[13 + lapIndex]` に相当する値。7要素の配列をラップごとにローテーションし、各ラップで異なるaiPatternを適用する設計。

aiPattern値(0-9) は EnemyRun オブジェクトの内部状態に格納され、KartPhysics_GroundedMovement で参照される横方向オフセット量 (`physicsState[0x51]`) の計算に間接的に影響する。

**具体的な影響**:
- FUN_801ddc5c が横方向オフセットの大きさを返す（テーブル 804e21b0 で定義）
- これに擬似乱数を掛け、`physicsState[0x51]` として保持
- GroundedMovement 内で、ウェイポイントに対する左右オフセットとして適用
- aiPattern値が大きいほど、ウェイポイントからの横方向オフセットが大きくなる傾向（ワイドライン走行）

---

## 5. aiVtableSelector の分岐

### 選択ロジック (FUN_801e83c8)

```c
puVar7 = GetKartTuningEntry(puVar7, courseId, subMode, kartSlot);
iVar5 = (int)puVar7[0xb] >> 0x1f;   // word[11] = aiVtableSelector
if ((puVar7[0xb] & 1 ^ -iVar5) + iVar5 == 1) {
    // aiVtableSelector == 1 → clEnemyRunType2
    FUN_801ec568(...)  // vtable=804e4bec
} else {
    // aiVtableSelector == 0 → clEnemyRunType1
    FUN_801ec65c(...)  // vtable=804e4c08
}
```

### clEnemyRunType1 (aiVtableSelector=0, FUN_801ec5a4)

**比例ステアリング方式**:
```
steer = (0.6 * dampingFactor * 0.01 + 0.1) * angleError
```
- dampingFactor = tuning[12] (steeringDamping), clamped to [0.0, 100.0]
- angleErrorに対して一定割合で応答
- dampingFactor=0 → steer = 0.1 * angleError (低応答)
- dampingFactor=100 → steer = 0.7 * angleError (高応答)

### clEnemyRunType2 (aiVtableSelector=1, FUN_801ec4b0)

**微分ステアリング方式**:
```
steer = -(0.25 * angleError - FLOAT_806d1948 * dampingFactor * 0.01)
```
- FLOAT_806d1948 は前フレームのステアリング蓄積値
- dampingFactorが大きいと前フレームの影響が大きくなる（慣性的挙動）
- 結果を反転 (`-`) するため、オーバーシュート補正的な動き

### 両タイプの共通点

- ライバル回避時 (`*(char *)(param_2 + 0x9d) != 0`): 結果に 0.15 を乗算（大幅に減衰）
- 両方とも `KartPhysics_GroundedMovement` の `(**(code **)(*physicsState + 0xc))(dVar9, physicsState)` から呼ばれる

---

## 6. 走行ライン計算 (KartPhysics_GroundedMovement, 0x801eed14)

### 基本フロー (毎フレーム)

1. **横方向オフセット更新**: `physicsState[0x51]` = ランダム * FUN_801ddc5c(lapProgress)
   - 一定間隔でリフレッシュ (0xA0 + random frames)

2. **ライバル回避判定**: `physicsState[0x53]` (避け方向: 0=左,1=右,2=左ライン切替,3=右ライン切替)
   - 近接ライバルの方向と距離からFUN_801ddb04で回避パターンを決定
   - パターン2/3: **別パスバリアントへの動的切替**を実行

3. **ウェイポイントターゲット取得**:
   - `physicsState[99]` が現在のパスバリアントインデックス (or -1=メインパス)
   - `physicsState[100]` が前回のパスバリアント
   - `physicsState[0x65]` がブレンド係数 (0→1, 0.004/frame で遷移)
   - バリアント切替時は FUN_800d5e20 で新旧ウェイポイント間を線形補間

4. **ステアリング角計算**:
   - 現在位置→次ウェイポイントへの方向ベクトルを計算
   - 横方向オフセット (`physicsState[0x51]`) をウェイポイント位置に適用
   - ライバル回避オフセット (`physicsState[0x57]`) を加算
   - 現在ヘディングとの角度差を計算

5. **EnemyRun vtable経由のステアリング補正**:
   - Type1/Type2 のどちらかで角度補正量を計算
   - ライバル回避中は補正を 0.15 倍に減衰

6. **速度連動スムージング**:
   - 高速時ほど急旋回を抑制 (turnRateLimit)
   - 加速度に基づくドリフト補正を適用

7. **ウェイポイント通過判定**:
   - ウェイポイントへの距離が FLOAT_806da508 未満、または
   - ウェイポイントを通り過ぎた (overshoot) 場合に次のウェイポイントへ進む

8. **位置更新**:
   - ヘディング方向に速度分の変位を加算
   - Y成分は地面追従のため別処理

---

## 7. コース分岐でのルート選択

### 静的選択 (初期化時)

FUN_801dc320 → FUN_801dcb18 でテーブル 804e36a8 を参照し、パスバリアント (0-5) を決定。
- スロットごとに異なるバリアントが割り当てられる
- 擬似乱数でA/Bの2択が存在するが、シーケンスは決定的

### 動的選択 (走行中)

KartPhysics_GroundedMovement 内のライバル回避ロジック:
- `physicsState[0x53]` が 2 or 3 のとき、**別パスバリアントへ切替**
- 切替先は自身の左右にいるライバルの位置に基づき、空いている側のバリアントを選択
- 切替はブレンド係数 `physicsState[0x65]` で 0.004/frame (約250フレーム = 4秒) かけてスムーズに遷移
- `physicsState[0x58]` に新しいパスバリアントIDが設定される

### パスバリアントのブレンド

バリアント切替時:
```
blendedWaypoint = lerp(newVariantWaypoint, oldVariantWaypoint, blendFactor)
```
これにより走行ラインが急激に変化せず、滑らかに遷移する。

---

## 8. スロット・パターン別の走行ライン変化

### speedFactor による速度差

| Slot | speedFactor (典型値) | 役割 |
|------|---------------------|------|
| 0    | 0.60 (最高)         | プレイヤー基準 |
| 1    | 0.50                | 第1 AI |
| 2    | 0.40                | 第2 AI |
| 3    | 0.30                | 第3 AI |
| 4    | 0.25                | 第4 AI |

### パスバリアントの意味

- **path 0**: コース中央寄りの安全なライン
- **path 1-5**: コース幅の異なる位置を通るバリエーション
- スロットごとに異なるバリアントが割り当てられるため、AI同士が重ならない

### aiPattern による横方向オフセット

- 値 0-9 で横方向のオフセット量が変化
- 小さい値: ウェイポイントに忠実な走行
- 大きい値: ウェイポイントからのオフセットが大きい（ワイドな走行ライン）
- 7要素の配列はラップごとにローテーションされ、周回ごとに走行ラインが変化

---

## 関数アドレス一覧

| アドレス | 名称 | 説明 |
|----------|------|------|
| 0x801e83c8 | AI_KartInit | AIカートの初期化、EnemyRunタイプ選択 |
| 0x801ebc8c | AI_SystemInit | 全AIスロットの初期化ループ |
| 0x801ebf48 | ThinkBox_Create | ThinkBoxマネージャ生成 |
| 0x801eadc8 | AI_UpdatePerFrameState | フレーム毎のAI状態更新 |
| 0x801eaca0 | AI_PerKartUpdate | スロット別速度設定・アイテム |
| 0x801ea6f0 | AI_CalcGPTargetSpeed | GP AI目標速度計算 |
| 0x801eed14 | KartPhysics_GroundedMovement | ★主要: 地上走行・パス追従 |
| 0x801effa4 | KartPhysics_UpdateSmoothing | 物理スムージング・行列更新 |
| 0x801e6ed4 | KartController_Update | KartControllerのフレーム更新 |
| 0x801e8140 | KartController_TimedUpdate | タイミング計測付きUpdate |
| 0x801ec568 | EnemyRunType2_Init | clEnemyRunType2 コンストラクタ |
| 0x801ec65c | EnemyRunType1_Init | clEnemyRunType1 コンストラクタ |
| 0x801f0644 | EnemyRun_BaseInit | EnemyRun 基底初期化 |
| 0x801ec4b0 | EnemyRunType2_Steer | 微分ステアリング計算 |
| 0x801ec5a4 | EnemyRunType1_Steer | 比例ステアリング計算 |
| 0x801ee7f0 | AI_CalcRivalAvoidance | ライバル回避ステアリング |
| 0x801dc320 | PathFollower_Init | パスフォロワー初期化 |
| 0x801dcb18 | PathVariant_Select | パスバリアント選択テーブル参照 |
| 0x801dc514 | PathData_GetByContext | コース/モード別パスデータ取得 |
| 0x800abe48 | CourseData_GetPath | パスバリアントのポインタ取得 |
| 0x80046680 | Waypoint_FindNearest | 最近接ウェイポイント検索 |
| 0x800463cc | Waypoint_Advance | 次ウェイポイントへ進む |
| 0x80046334 | Path_GetTotalCount | 総ウェイポイント数 |
| 0x801dfa40 | SteeringMgr_Init | ステアリングサブシステム初期化 |
| 0x801de560 | GetKartTuningEntry | チューニングテーブルアクセス |
| 0x801de448 | GetKartTuningGPSpeedParams | GP速度パラメータ取得 |
| 0x801de4d4 | GetKartTuningSpeedFactor | 速度係数取得 |
| 0x801ddc5c | LateralOffset_GetMagnitude | 横方向オフセット量テーブル参照 |
| 0x801eb80c | PseudoRandom_Next | 擬似乱数値取得 |
| 0x801ddb04 | RivalAvoidance_CalcPattern | ライバル回避パターン計算 |
| 0x801de848 | DriveBehavior_HandleWpType | ウェイポイントタイプ別処理 |

## グローバルデータ

| アドレス | 名称 | 説明 |
|----------|------|------|
| 0x806d1924 | g_pathSystem | パスシステムオブジェクトポインタ |
| 0x80679598 | g_kartControllerArray[8] | AIカートコントローラ配列 |
| 0x806793d8 | g_aiSlotData[8] | AIスロットデータ (stride 0x38) |
| 0x804e36a8 | g_pathSelectionTable[73] | パスバリアント選択テーブル |
| 0x8049d030 | g_kartTuningTable | RACEモードチューニングテーブル |
| 0x804bf830 | g_kartTuningTableNonRace | 非RACEモードチューニングテーブル |
| 0x804e21b0 | g_lateralOffsetTable | 横方向オフセット量テーブル |
| 0x806d192c | g_pseudoRandomCounter | 擬似乱数カウンタ |
| 0x8049bc24 | g_waypointTypeTable | ウェイポイントタイプ定義 (stride 0x20) |

## 未解明事項

- aiPattern 7要素のうち、具体的にどのインデックスがどのラップに対応するかの正確なマッピング
- ウェイポイントタイプテーブル (0x8049bc24) の全タイプの意味（3-12の一部は解析済み: 3,4,7=カーブ, 5,6=速度, 8,9=フラグ, 10,11=特殊移動, 12=トリガー）
- FUN_801ddc5c のオフセットテーブル (804e21b0) の全エントリとaiPatternとの正確な対応関係
- 非RACEモード (BATTLE等) でのパスファインディングの差異
