# MKGP2 カートタスクシステム解析

## タスク文字列テーブル

"KART-EXEC" (0x802e9a74)、"KART-DRAW" (0x802e9a80)、"KART-MOTION" (0x802e9a8c) はタスク名テーブルに格納されている。ポインタテーブル 0x802e9b90 が全タスク名を指す。文字列はこのポインタテーブルへの DATA xref のみ (インデックス 13, 14, 15) で、直接のコード参照はない。

タスクシステムは文字列ベースではなくインデックスベースの登録を使うため、文字列名はデバッグ表示専用。

## clFlowKart (カート選択フロー)

- 文字列: "clFlowKart" @ 0x8039e9cc
- clFlowProcess (0x8039e9d8) を継承
- クラスデータ: 0x806cfec8 (名前ポインタ + vtable情報)
- メソッド:
  - **FUN_801d8284** (デストラクタ): スプライト、入力状態、画面輝度のクリーンアップ
  - **FUN_801d7e84** (更新/実行): カート選択UIのステートマシン
    - State 0: イントロアニメーション、スプライト生成
    - State 1: 入力処理 (左右カート選択)、FUN_801d7920 でカートモデル表示
    - State 2: トランジションアウト
    - State 4: 結果返却
  - **FUN_801d7cc4** (描画): カートモデルと選択矢印のレンダリング

このクラスは**レース前のカート選択画面**を管理する。レース中のカート物理ではない。

## レース中カートシステムアーキテクチャ

### 初期化

**FUN_8009f814** (レース初期化): マスターレース初期化関数
- `FUN_801ebc8c` (KartSystemInit) を呼んでカートシステムをセットアップ
- `FUN_8025421c` (TaskManager Init) を呼んでタスク/表示マネージャを生成

**FUN_801ebc8c** (KartSystemInit):
- 8つのカートスロットをイテレート
- `FUN_801e83c8` でカートオブジェクトを確保 (各 0xF4 bytes)
- カートポインタを `DAT_80679598[]` (グローバルカートインスタンス配列) に格納
  - ※プレイヤーの CarObject は g_playerCarObject (DAT_806d1098) に別途保存される
- `FUN_801ebf48` (KartControllerFactory) を呼んでAIコントローラを生成

### コントローラファクトリ (FUN_801ebf48 @ 0x801ebf48)

ゲームモードに基づいてポリモーフィックなAIコントローラオブジェクトを生成:

| モード | Vtable | クラス |
|--------|--------|--------|
| BATTLE (ベース) | PTR_PTR_804e4b88 | Base (AIなし) |
| 非バトル 非TA | PTR_PTR_804e4b74 | clEnemyThinkBoxVScli |
| TIME_ATTACK | PTR_PTR_804e4ba8 | clEnemyThinkBoxInterface |
| RACE / デフォルト | PTR_PTR_804e4bc8 | clEnemyThinkBoxGrandPrix |

### Vtableレイアウト (コントローラクラスごと)

各vtableはオフセット +0x08, +0x0C, +0x10 に3つの主要メソッドスロットを持つ:

**通常モード (PTR_PTR_804e4b74 @ 0x804e4b74):**
- +0x08: FUN_801e96bc (PreUpdate)
- +0x0C: FUN_801e9594 (PerKartUpdate)
- +0x10: FUN_801e93ac (AISpeedCalc)

**TIME_ATTACK (PTR_PTR_804e4ba8 @ 0x804e4ba8):**
- +0x08: FUN_801e9e74 (PreUpdate)
- +0x0C: FUN_801e9cbc (PerKartUpdate)
- +0x10: FUN_801e9a84 (AISpeedCalc)

**RACE / Grand Prix (PTR_PTR_804e4bc8 @ 0x804e4bc8):**
- +0x08: FUN_801eadc8 (PreUpdate)
- +0x0C: FUN_801eaca0 (PerKartUpdate)
- +0x10: FUN_801ea6f0 (AISpeedCalc)

### AIクラス名 (0x806cffc0 構造体より)

- clEnemyThinkBoxVScli (VSクライアント)
- clEnemyThinkBoxInterface (ベースインターフェース)
- clEnemyThinkBoxVSsvr (VSサーバー)
- clEnemyThinkBoxGrandPrix (Grand Prix / レース)

## 実行チェーン

```
レース初期化 (FUN_8009f814)
  |
  +-> KartSystemInit (FUN_801ebc8c) -- DAT_80679598[] にカートオブジェクト生成
  +-> KartControllerFactory (FUN_801ebf48) -- AIコントローラ生成 (vtableディスパッチ)
  
毎フレーム実行:
  AIコントローラ vtableメソッド PerKartUpdate (例: FUN_801e9594)
    |
    +-> AISpeedCalc (例: FUN_801e93ac) -- データテーブルから目標速度を計算
    +-> KartController_SetTargetSpeed -- カートオブジェクトに目標速度を書き込み (+0x8c, +0x90)
    +-> KartController_TimedUpdate -- タイミングラッパー
          |
          +-> KartController_Update -- メインカート更新関数
                |
                +-> PhysicsState_AccelInterpolate / PhysicsState_SetSpeedImmediate -- 加速補間
                +-> 行列変換 (位置更新)
                +-> FUN_801e5b50 (ItemEffectHandler)
                +-> デバッグオーバーレイ: "GRID %d: speed %6.2f(km/h) line %d-%d"
```

## 速度 / 加速システム

### カートオブジェクト速度フィールド

**カートオブジェクト (DAT_80679598[] に格納, 0xF4 bytes):**
- +0x8c: 目標最低速度 (KartController_SetTargetSpeed で設定)
- +0x90: 目標最高速度 (KartController_SetTargetSpeed で設定)

**カート物理サブオブジェクト (PhysicsState_AccelInterpolate の param_3):**
- +0x6c: 現在の前進速度
- +0x70: 目標前進速度
- +0x74: 現在の旋回速度
- +0x78: 目標旋回速度

### 速度設定: KartController_SetTargetSpeed (0x801e6c5c)

```c
void KartController_SetTargetSpeed(double maxSpeed, double minSpeed, int kartObj) {
    kartObj->targetMinSpeed = (float)minSpeed;  // +0x8c
    kartObj->targetMaxSpeed = (float)maxSpeed;   // +0x90
}
```

### 加速ロジック: PhysicsState_AccelInterpolate (0x801ec894)

現在速度を目標に向かって滑らかに補間する:
```c
void PhysicsState_AccelInterpolate(float targetSteer, float targetThrottle, int *self) {
    // コース別カートチューニングテーブルから加速レートを取得
    // GetKartTuningEntry(enemyParam, g_courseId, g_subMode, self->kartTypeId)
    // テーブル: DAT_8049d030 (RACE) / DAT_804bf830 (その他)
    // インデックス: (courseId-1)*0xC0 + ccClass*0x40 + subMode*8 + kartTypeId
    int tuningEntry = GetKartTuningEntry(GetEnemyParam(), g_courseId, g_subMode, self[4]);
    float accelRate = *(float*)(tuningEntry + 4);  // エントリの+0x04フィールド
    float maxDelta = FLOAT_806da444 * FLOAT_806da448 * accelRate;  // 10.0 * 1000.0 * accelRate
    
    // 前進速度の補間 (self[0x1b] = +0x6C, self[0x1c] = +0x70)
    float diff = targetSteer - self->smoothedSteer;
    if (diff > maxDelta)
        self->smoothedSteer += maxDelta;
    else if (diff < -maxDelta)
        self->smoothedSteer -= maxDelta;
    else
        self->smoothedSteer = targetSteer;
    self->targetSteer = targetSteer;
    
    // 旋回速度も同様 (self[0x1d] = +0x74, self[0x1e] = +0x78)...
}
```

加速レートはカートスロット (slotId) によって異なる:

| Slot | 説明 | accelRate | maxDelta | 意味 |
|------|------|-----------|----------|------|
| Slot 0 | Player | 0.8 | 8000 | 標準加速 |
| Slot 1 | AI1 | 0.8 | 8000 | AI第1グループ |
| Slot 2 | AI2 | 0.8 | 8000 | AI第2グループ |
| Slot 3 | AI3 | 0.6 | 6000 | AI第3グループ (加速が遅い) |

Slot 3 (AI3) は最高速度が高い代わりに加速で劣るトレードオフ設計。

### 即時速度設定: PhysicsState_SetSpeedImmediate (0x801ec880)

初期配置時に使用 (加速ランプなし):
```c
void PhysicsState_SetSpeedImmediate(double front, double turn, int kartPhys) {
    kartPhys->currentForwardSpeed = front;   // +0x6c
    kartPhys->targetForwardSpeed = front;    // +0x70
    kartPhys->currentTurnSpeed = turn;       // +0x74
    kartPhys->targetTurnSpeed = turn;        // +0x78
}
```

### BaseSpeed テーブル (AI 速度の根幹)

> **注意**: AI カートは `CarObject_Init` を使わない。`FUN_801e83c8` で独自初期化され、KartParamBlock (プレイヤー用) は AI に適用されない。AI の速度は以下の BaseSpeed テーブルで決定される。

**RACEモード**: ベーステーブル `DAT_803a01e8` (`GetBaseSpeedMax` / `GetBaseSpeedMin`)
- インデックス: `(courseId - 1) * 0x18 + ccClass * 8 + subMode`
- 各エントリ: 2つのfloat (speedMin, speedMax)
- subMode 0-3 = 表コース、subMode 4-7 = 裏コース (裏は表より高い速度上限)

**その他のモード**: ベーステーブル `DAT_803a07e8`
- インデックス: `courseId + ccClass * 8 - 1`

RACEモード速度サンプル (コース1の最初の4エントリ):
- 50cc モード0: min=145.0, max=160.0 (km/h)
- 50cc モード1: min=145.0, max=160.0
- 100cc モード0: min=155.0, max=170.0
- 100cc モード1: min=155.0, max=170.0
- 150cc モード0: min=160.0, max=170.0
- 150cc モード1: min=160.0, max=170.0

#### 表/裏コースの識別

- **g_isUraCourse** (0x806d1268): 0=表, 1=裏
- **g_subMode** (0x806d1298): コース選択位置 + (裏なら +4)。表: 0-3, 裏: 4-7
- **g_courseId** (0x806cf108): カップID (1-8)。表/裏で同一値

### AI目標速度計算

GetKartTuningSpeedFactor (0x801de4d4) がカートごとの目標速度係数を取得 (GetKartTuningEntry と同じテーブルのエントリ+0x00を返す):
- **RACE**: `DAT_8049d030`, インデックス `(courseId-1)*0xC0 + ccClass*0x40 + subMode*8 + kartIndex`
- **その他**: `DAT_804bf830`, インデックス `courseId + ccClass*8 - 1`

この値に `FLOAT_806da3e0 * FLOAT_806da3e4` (グローバル速度スケーリング) を乗じてから KartController_SetTargetSpeed に渡す。

### ラバーバンド

FUN_801e93ac/FUN_801e9a84 (AISpeedCalc) が順位依存の調整を追加:
- `AI_CalcDistanceSpeedBonus(kartIdx, currentCheckpoint, leaderCheckpoint)` -- 距離ベースの速度ボーナス
- `AI_CalcLapSpeedBonus(kartIdx, elapsedLaps, kartLineIdx, kartCheckpoint)` -- ラップベースの速度ボーナス
- グローバル倍率適用前のベース速度に加算
- 結果は `DAT_8067940c[kartIdx * 0xe]` 配列に格納

## 0x801e5000-0x801e7000 範囲の関数

| アドレス | サイズ | 説明 |
|----------|--------|------|
| 0x801e5454 | 672 | 不明 |
| 0x801e5724 | 684 | 不明 |
| 0x801e5a00 | 24 | 小ユーティリティ |
| 0x801e5a84 | 204 | 不明 |
| 0x801e5b50 | 2628 | **ItemEffectHandler**: カートへのアクティブアイテム効果管理 (ステータスタイマー, 視覚効果) |
| 0x801e6594 | 36 | 小ゲッター/チェック |
| 0x801e65b8 | 184 | 不明 |
| 0x801e6670 | 68 | 不明 |
| 0x801e66b4 | 512 | **CollisionResponse**: カート間接近/反発物理 |
| 0x801e68b4 | 600 | **ItemHitHandler**: カートへのアイテムヒットイベント処理 |
| 0x801e6b0c | 44 | カートプロパティセッター |
| 0x801e6b38 | 44 | カートプロパティセッター |
| 0x801e6b64 | 68 | カートプロパティセッター |
| 0x801e6ba8 | 104 | **SetKartTarget**: カートターゲット/追従オブジェクト設定 |
| 0x801e6c10 | 68 | カート状態セッター |
| 0x801e6c54 | 8 | カートフラグセッター |
| 0x801e6c5c | 12 | **KartController_SetTargetSpeed**: 目標最低/最高速度設定 (+0x8c/+0x90) |
| 0x801e6c68 | 44 | カート初期化セッター |
| 0x801e6c94 | 52 | カートゲッター (サブフィールド返却) |
| 0x801e6cc8 | 52 | カートゲッター |
| 0x801e6cfc | 52 | カートゲッター |
| 0x801e6d30 | 52 | カートゲッター |
| 0x801e6d64 | 8 | カートゲッター (直接フィールド) |
| 0x801e6d6c | 52 | カートゲッター (boolチェック) |
| 0x801e6da0 | 52 | カートゲッター |
| 0x801e6dd4 | 8 | カートゲッター (bool) |
| 0x801e6ddc | 52 | カートゲッター |
| 0x801e6e10 | 8 | **GetKartData**: カート内部データポインタを返す |
| 0x801e6e18 | 8 | **GetKartId**: カートIDフィールドを返す |
| 0x801e6e20 | 44 | カート状態初期化 |
| 0x801e6e4c | 136 | カートモードセッター (パラメータベース) |
| 0x801e6ed4 | 4716 | **KartController_Update**: メイン毎フレームカート更新 (変換, 物理, デバッグオーバーレイ) |

## グローバルデータ

- `DAT_80679598[]`: 8個のカートオブジェクトポインタ配列 (カートスロット配列)
  - ※プレイヤーの CarObject は g_playerCarObject (DAT_806d1098) に別途保存される
- `DAT_806793d8[]`: 8個のAIコンテキスト構造体配列 (各 0x38 ワード = 0xE0 bytes)
- `DAT_806d1920`: 最大フレームカウンタ (AI速度ランプアップフェーズを制限)
- `DAT_806d1924`: コースレースデータポインタ
- `g_gameMode`: RACE, TIME_ATTACK, BATTLE
- `g_courseId` (0x806cf108): 1-8 (カップID。表/裏で同一値)
- `g_ccClass`: 0=50cc, 1=100cc, 2=150cc
- `g_subMode` (0x806d1298): コース選択位置 + (裏なら +4)。表: 0-3, 裏: 4-7
- `g_isUraCourse` (0x806d1268): 0=表, 1=裏
