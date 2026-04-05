# MKGP2 キャラクター・カート詳細パラメータ

## プレイヤブルキャラクター一覧 (13体)

| ID | 内部名 | 名前 | 重量クラス |
|----|--------|------|-----------|
| 0  | MARIO    | マリオ       | 中量級 |
| 1  | LUIGI    | ルイージ     | 中量級 |
| 2  | WARIO    | ワリオ       | 重量級 |
| 3  | PEACH    | ピーチ       | 中量級 |
| 4  | KOOPA    | クッパ       | 重量級 |
| 5  | KINOPIO  | キノピオ     | 軽量級 |
| 6  | DONKEY   | ドンキーコング | 重量級 |
| 7  | YOSHI    | ヨッシー     | 中量級 |
| 8  | PACMAN   | パックマン   | 中量級 (特殊) |
| 9  | MS PACMAN | ミズパックマン | 軽量級 |
| 10 | MONSTER  | アカベイ     | 中量級 |
| 11 | WALUIGI  | ワルイージ   | 中量級 |
| 12 | MAMETH   | まめっち     | 軽量級 |

ID 13-14 は "NO CHARA" プレースホルダ。

## カートバリアント (4種)

プレイヤーがカート選択で選ぶ形態。GOLD は MKGP1 のもので MKGP2 では選択不可。

| 内部インデックス | 内部名 | モデル | ゲーム上の扱い |
|-----------------|--------|--------|---------------|
| 0 | NORMAL | `cart_<chara>.dat` | 通常カート (初期状態) |
| 1 | GOLD | `gold_<chara>.dat` | MKGP2では未使用 |
| 2 | ORIGINAL | `sp_cart_<chara>.dat` | 専用カート (景品で入手) |
| 3 | ORIGINAL ENHANCED | `sp_cart_<chara>.dat` (同一モデル) | チューンナップカート (COIN_MILAGEで自動解放) |

- ORIGINAL と ORIGINAL ENHANCED は同じ 3D モデル (`sp_cart_`) を共有。走行性能は **speedTablePtr チェーン構造** により差別化される (物理パラメータ +0x00〜+0xEF は同一だが、speedTablePtr (+0xF0) が異なるバリアントのカーブを指すことで speedCap に差が生じ、最高速が異なる)。
- 各モデルに3段階のLODあり: `<name>.dat` (高), `<name>_mid.dat` (中), `<name>_low.dat` (低)

### キャラクタークラス割り当て (キャラ選択画面の説明文) [CONFIRMED]

**キャラ選択画面では、キャラクターごとに固有のクラス説明文が表示される。**
このクラスはカートバリアント選択 (clFlowKart) とは独立した、キャラクター固有の分類。

クラスインデックスは `g_charaClassTable` (0x8039AEA4, int[13]) に格納されている。

| charId | キャラクター | classIdx | クラス名 | 説明 (EN) |
|--------|-------------|----------|---------|-----------|
| 0 | マリオ | 2 | ORIGINAL | Balanced kart |
| 1 | ルイージ | 2 | ORIGINAL | Balanced kart |
| 2 | ワリオ | 3 | ORIGINAL_ENHANCED | High speed, yet tricky advanced player's kart |
| 3 | ピーチ | 1 | GOLD | Strong acceleration kart |
| 4 | クッパ | 3 | ORIGINAL_ENHANCED | High speed, yet tricky advanced player's kart |
| 5 | キノピオ | 0 | NORMAL | Easy control kart |
| 6 | ドンキー | 3 | ORIGINAL_ENHANCED | High speed, yet tricky advanced player's kart |
| 7 | ヨッシー | 1 | GOLD | Strong acceleration kart |
| 8 | パックマン | 2 | ORIGINAL | Balanced kart |
| 9 | ミズパックマン | 0 | NORMAL | Easy control kart |
| 10 | アカベイ | 1 | GOLD | Strong acceleration kart |
| 11 | ワルイージ | 2 | ORIGINAL | Balanced kart |
| 12 | まめっち | 0 | NORMAL | Easy control kart |

#### 表示の仕組み

1. `CharaSelect_SetupCharacterDisplay` (0x801c3470) がキャラ選択カーソル移動時に呼ばれる
2. `g_charaClassTable[charId]` から classIdx (0-3) を取得
3. `GetTextureByIdAndLang(classIdx + 0x95, lang)` で説明文テクスチャを取得
4. GetTextureByIdAndLang は `PTR_DAT_8048e548[id * 2 + lang]` を参照
   - textureId 0x95-0x98 が classIdx 0-3 に対応
   - 0x8048e548 + 0x95 * 8 = 0x8048E9F0 = 説明文ポインタテーブル
5. 同時にスター評価 (速度/加速/ハンドリング) も `g_charaStatTable` (0x8049AA58) から読み込まれる
   - レイアウト: `[frameIdx * 0x27 + charId * 3]` = {speed, accel, handling}
   - 各バイト = 5 - 表示スター数 (0=5つ星, 5=0つ星)

#### 関連コード

- `clFlowChara_Update` (0x801c3d48): キャラ選択画面のメインループ
- `CharaSelect_SetupCharacterDisplay` (0x801c3470): キャラ表示セットアップ
- `FrameSelection_Init` (0x801c4ed8): 初期化時にも同テーブルを参照
- `clFlowChara_Destructor` (0x801c4ca4): デストラクタ
- `GetTextureByIdAndLang` (0x8013c2d4): ローカライズテクスチャ/テキスト取得

### アンロックメッセージ
- ORIGINAL 入手時: "専用カートをゲット！" / "Get %s original kart!"
- ORIGINAL ENHANCED 入手時: "%sの専用カートを チューンナップしました！" / "Get %s power up parts!"

### ヨッシー (charId=7) 150cc の具体例

ヨッシーは GOLD クラス (classIdx=1) に属し、ピーチ (charId=3) およびアカベイ (charId=10) と同クラス。
同クラス内のキャラクターは物理パラメータと速度カーブが完全に一致する。

#### KartParamBlock アドレス (150cc)

| バリアント | アドレス |
|-----------|---------|
| NORMAL | 0x80398820 |
| ORIGINAL | 0x803989F8 |
| ENHANCED | 0x80398BD0 |

#### 物理パラメータ (Profile A: Standard Medium)

35フィールド中33が3バリアント共通:
- mue=0.21, gripCoeff=0.016, weight=2.0, accelGainRate=0.00125
- turnAngle=20.0, driftBias=0.0, slipCoeff=0.02, turnRate=0.1
- recoveryRate=0.005, cornerBoost=0.0

差異があるのは**2フィールドのみ**:

| フィールド | NORMAL | ORIGINAL/ENHANCED |
|-----------|--------|-------------------|
| driftIntensity (+0x88) | 1.705 | 1.720 |
| driftAmplitude (+0x8C) | 2.059 | 2.100 |

#### speedTablePtr チェーンと最高速度

| Variant | speedTablePtr | 参照先カーブ | speedCap | 表示 km/h |
|---------|--------------|-------------|---------|-----------|
| NORMAL | 0x80398778 | 前ccブロック | 76.66 | ~138 |
| ORIGINAL | 0x80398950 | NORMALのカーブ | 78.06 | ~140 |
| ENHANCED | 0x80398b28 | ORIGINALのカーブ | 78.89 | ~142 |

#### 加速特性

- **最高速**: ENHANCED > ORIGINAL > NORMAL
- **加速の立ち上がり**: ENHANCED > NORMAL > ORIGINAL (ORIGINALは中間域でNORMALカーブを使うため初期加速で劣る)
- **ドリフト**: NORMAL がやや穏やか、ORIGINAL/ENHANCED がやや強い

詳細データ: `mkgp2_kart_params.md` の「速度カーブテーブル」セクション参照。

## オブジェクト階層

| オブジェクト | サイズ | 格納先 | 内容 |
|------------|--------|--------|------|
| KartController | 0xF4 | DAT_80679598[slot] | レース中のカート管理。CarObject/PhysicsStateへのポインタを保持 |
| CarObject | 900 (0x384) | KartController+0x10 | 3Dモデル、ボーン、エフェクト、KartMovementへのポインタ |
| KartMovement | 0x324 | CarObject+0x28 | 物理シミュレーション: travelProgress (+0x0C), speedCap (+0x10), 速度、加速、コインボーナス、EffectSpeed |
| PhysicsState | 0x198 | KartController+0x20 | ステアリング/スロットル入力、位置、衝突、向き |

**PhysicsState と KartMovement は別オブジェクト。** PhysicsState はステアリング入力やスムージング (+0x6C~+0x78)、位置 (+0x4C~+0x60)、向き (+0x7C~+0x84) を管理する。KartMovement は物理シミュレーション (travelProgress +0x0C、speedCap +0x10、速度ベクトル +0x17C、speedScale +0x2D8、coinSpeedBonus +0x2E0、mueScale +0x2F8) を管理する。旧ドキュメントで `PhysicsState+0x2D8` 等と記載されていたフィールドは KartMovement のものである。

**プレイヤーCarObjectの格納先**: プレイヤーの CarObject は `DAT_80679598` の KartController 配列には含まれず、`g_playerCarObject` (DAT_806d1098) に別途保存されている。DAT_80679598[0-4] は AI 用 KartController のみ。

## AI 速度決定メカニズム

AI カートはプレイヤーとは異なる速度決定システムを使用する。

### 初期化の違い

- **プレイヤー**: `CarObject_Init` (0x8004e618) → `KartMovement_Init` → `GetKartParamBlock` で KartParamBlock を取得
- **AI**: `FUN_801e83c8` で独自初期化。KartParamBlock (プレイヤー用) は AI には適用されない

### BaseSpeed テーブル (0x803A01E8)

AI の目標速度は **BaseSpeed テーブル** から取得される。`GetBaseSpeedMax` / `GetBaseSpeedMin` でアクセスする。

- **テーブルアドレス**: `0x803A01E8` (RACE モード)
- **インデックス式**: `(courseId - 1) * 0x18 + ccClass * 8 + subMode`
- **各エントリ**: 2つの float (speedMin, speedMax)

### 表/裏コースの識別

| グローバル変数 | アドレス | 値域 | 説明 |
|---------------|---------|------|------|
| **g_isUraCourse** | 0x806d1268 | 0=表, 1=裏 | 表/裏フラグ |
| **g_subMode** | 0x806d1298 | 0-7 | コース選択位置 + (裏なら +4) |
| **g_courseId** | 0x806cf108 | 1-8 | カップID。表/裏で同一値 |

- **表コース**: subMode = 0-3
- **裏コース**: subMode = 4-7

### 裏コースで AI が速い理由

BaseSpeed テーブルの subMode 別エントリにある。裏コース (subMode 4-7) は表 (0-3) より高い速度上限を持つ。

**Mario Cup 150cc の例**:

| subMode | 表/裏 | speedMax | speedMin |
|---------|-------|----------|----------|
| 0 (表1) | 表 | 250 | 270 |
| 1 (表2) | 表 | 265 | 275 |
| 2 (表3) | 表 | 270 | 280 |
| 3 (表4) | 表 | 275 | 280 |
| 4 (裏1) | 裏 | 275 | 280 |
| 5 (裏2) | 裏 | 275 | 280 |
| 6 (裏3) | 裏 | 280 | 285 |
| 7 (裏4) | 裏 | 280 | 285 |

### AI 速度の計算フロー

1. `GetBaseSpeedMax/Min(courseId, subMode)` でベース速度を取得
2. チューニングテーブルの `speedFactor` を乗じる (スロット別係数)
3. GP ラバーバンド (aheadSpeedBonus / behindSpeedBonus) を加算
4. `KartController_SetTargetSpeed` で目標速度として設定

### チューニングテーブルとの関係

チューニングテーブルの `speedFactor` はベース速度に対する係数。ベース速度が上がれば AI の実効速度も上がる。チューニングテーブル自体の subMode 間差異は主に `aiPattern` (走行パターン) のみ。

詳細データ: `mkgp2_kart_tuning_table.md` 参照。

## パラメータシステム

カートのパラメータは**2つのテーブル**で構成される:

1. **カートパラメータブロック** (GetKartParamBlock) — grip, weight, drift 等の物理特性
2. **コースチューニングテーブル** (GetKartTuningEntry) — speedFactor, accelRate 等の速度特性

### カートバリアントとキャラクタークラスの関係

同じ enum 名 (NORMAL/GOLD/ORIGINAL/ORIGINAL_ENHANCED) が**2つの異なる概念**に使い回されている:

| 概念 | 格納先 | 用途 |
|------|--------|------|
| **カートバリアント** (kartVariant) | `g_kartVariant` (0x806cf118) | カート選択で選ぶ形態。パラメータ参照に使用 |
| **キャラクタークラス** (classIdx) | `g_charaClassTable` (0x8039AEA4) | キャラ選択画面の説明文表示に使用 |

両者は別テーブルに格納されており、直接の対応関係はない。

### パラメータブロック (GetKartParamBlock)

各キャラ **9ブロック** = 3 ccClass × 3 normalizedKartVariant。
参照: `charParamTable[charId] + ccClass * 12 + normalizedKartIdx * 4`
正規化: NORMAL/GOLD→0, ORIGINAL→1, ENHANCED→2

**ORIGINAL と ENHANCED は物理パラメータ (+0x00〜+0xEF) が全キャラ全CCで完全同一。**
NORMAL との違いは **driftIntensity (+0x88) と driftAmplitude (+0x8C) のみ**。

しかし **+0xF0 (speedTablePtr) 以降のスピードカーブは ORIGINAL と ENHANCED で異なる**:
- **ORIGINAL**: 距離に応じた段階的加速カーブ (NORMAL より高速)
- **ENHANCED**: 全距離閾値でフラット最高速 = 加速がほぼ一瞬で完了

> **ORIGINAL vs ENHANCED の差分 (2026-04-03 再修正)**:
> - **最高速度**: speedTablePtr チェーン構造により、ENHANCED の speedCap は ORIGINAL のカーブ最大値、ORIGINAL の speedCap は NORMAL のカーブ最大値になる。SpeedCurve_UpdateSpeedCap (0x8019ef90) が speedCap を毎フレーム更新し、PhysicsStep が駆動力スケーリングに使用する。動的テスト (Peach 150cc): NORMAL ≈ 276 (138 km/h), ORIGINAL ≈ 281 (140 km/h), ENHANCED ≈ 284 (142 km/h)
> - **称号システム**: ENHANCED(3) 専用の `CheckTitle_EnhancedKartTA` (0x800a8004) — TA無敗+スコア条件で称号ID 87、ORIGINAL(2) 専用の `CheckTitle_OriginalKartCoinMilage` (0x800a8538) — キャラ別COIN_MILAGE範囲で称号ID 63
> - **ビジュアルエフェクト**: 両方とも `variant >= 2` で有効化、差異なし
> 
> 以前「走行物理・速度に差はない」としていたが誤りであった。speedCap が異なるため最高速に差がある。
> 詳細: `mkgp2docs/mkgp2_speed_curve_dataflow.md`、`mkgp2docs/mkgp2_kart_params.md` の「速度カーブテーブル」セクション

#### パラメータフィールドの意味

| フィールド | オフセット | 説明 |
|-----------|----------|------|
| **grip** (mue) | +0x28 | タイヤのグリップ/摩擦係数。高いほど滑りにくく、コーナリングが安定する |
| **gripCoefficient** | +0x2C | グリップの速度依存係数。高速時のグリップ低下に影響 |
| **weight** | +0x58 | カートの重量。衝突時の押し合い、加速の鈍さに影響。重いほど押し負けないが加速で不利 |
| **accelGainRate** | +0x60 | 加速の基礎ゲイン。高いほどスロットル入力に対する加速応答が良い |
| **turnAngle** | +0x64 | 最大旋回角度 (度)。大きいほど急カーブに対応しやすい |
| **driftBias** | +0x68 | ドリフト時の横方向バイアス。0以外だとドリフト中に片方向に流れやすくなる |
| **turnRate** | +0x78 | ステアリング応答速度。高いほどハンドル操作が即座に反映される |
| **cornerBoost** | +0x84 | コーナー脱出時のブースト量。軽量級のみ非ゼロ (1.389) |
| **driftIntensity** | +0x88 | ドリフトの強さ。専用カート/チューンナップ時はキャラ別にチューニングされる |
| **driftAmplitude** | +0x8C | ドリフトの振幅。大きいほどドリフト中の横移動が大きい |

#### キャラクタークラスと重量クラスの対応

キャラ選択画面のクラス分類 (classIdx) と物理パラメータの重量クラスはほぼ1:1で対応する。

| classIdx | クラス名 | 重量クラス | キャラクター | grip | weight | turnAngle | cornerBoost | 備考 |
|----------|---------|-----------|-------------|------|--------|-----------|-------------|------|
| **0** | **NORMAL** | **軽量級** | キノピオ | 0.285 | 1.0 | 20.0 | **1.389** | |
| **0** | **NORMAL** | **軽量級** | ミズパックマン | 0.285 | 1.0 | 20.0 | **1.389** | |
| **0** | **NORMAL** | **軽量級** | まめっち | 0.285 | 1.0 | 20.0 | **1.389** | |
| **1** | **GOLD** | **中量級** | ピーチ | 0.210 | 2.0 | 20.0 | 0.0 | |
| **1** | **GOLD** | **中量級** | ヨッシー | 0.210 | 2.0 | 20.0 | 0.0 | |
| **1** | **GOLD** | **中量級** | アカベイ | 0.210 | 2.0 | 20.0 | 0.0 | |
| **2** | **ORIGINAL** | **中量級** | マリオ | 0.210 | 2.0 | 20.0 | 0.0 | |
| **2** | **ORIGINAL** | **中量級** | ワルイージ | 0.210 | 2.0 | 20.0 | 0.0 | |
| **2** | **ORIGINAL** | **中量級** | ルイージ | 0.200 | 2.0 | 20.0 | 0.0 | grip微減 |
| **2** | **ORIGINAL** | **中量級** | パックマン | 0.205 | 2.0 | 20.0 | 0.0 | driftBias=0.01 |
| **3** | **ORIG_ENH** | **重量級** | ワリオ | 0.190 | 3.0 | 18.0 | 0.0 | |
| **3** | **ORIG_ENH** | **重量級** | クッパ | 0.190 | 3.0 | 18.0 | 0.0 | |
| **3** | **ORIG_ENH** | **重量級** | ドンキーコング | 0.190 | 3.0 | 18.0 | 0.0 | |

**まとめ:**
- classIdx 0 (NORMAL) = 軽量級 → 全3体がパラメータ完全一致
- classIdx 1 (GOLD) = 中量級 → 全3体がパラメータ完全一致
- classIdx 2 (ORIGINAL) = 中量級 → マリオ/ワルイージは一致。ルイージ (grip 0.20) とパックマン (grip 0.205, driftBias) に微差
- classIdx 3 (ORIG_ENH) = 重量級 → 全3体がパラメータ完全一致

重量級のみ追加特性あり: accelGain=0.00133 (他 0.00125), driftBias=0.001, turnRate=0.08 (他 0.1)

#### ドリフト特性 (NORMAL vs ORIGINAL/ENHANCED)

NORMALのドリフト値は全キャラ共通。ORIGINAL/ENHANCEDはキャラ別チューニング:

| キャラクター | NORMAL dI / dA | ORIGINAL・ENHANCED dI / dA |
|-------------|----------------|---------------------------|
| マリオ | 1.705 / 2.059 | **2.380 / 3.110** |
| ルイージ | 1.705 / 2.059 | **2.550 / 4.040** |
| ワリオ | 1.705 / 2.059 | **2.000 / 2.100** |
| ピーチ | 1.705 / 2.059 | **2.590 / 2.590** |
| クッパ | 1.705 / 2.059 | **1.720 / 2.100** |
| キノピオ | 1.705 / 2.059 | **2.200 / 2.200** |
| ドンキー | 1.705 / 2.059 | **2.660 / 2.660** |
| ヨッシー | 1.705 / 2.059 | **1.720 / 2.100** |
| パックマン | 1.705 / 2.059 | **2.610 / 3.110** |
| ミズパックマン | 1.705 / 2.059 | **3.410 / 3.410** |
| アカベイ | 1.705 / 2.059 | **2.270 / 1.580** |
| ワルイージ | 1.705 / 2.059 | **3.240 / 5.200** |
| まめっち | 1.705 / 2.059 | **1.760 / 2.600** |

### コースチューニングテーブル (GetKartTuningEntry)

**このテーブルはカートスロット番号 (0-7) でインデックスされ、カートバリアント選択 (NORMAL/ORIGINAL/ENHANCED) とは無関係。**
プレイヤーは常に Slot 0 のエントリを使用する。ORIGINAL と ENHANCED の差別化はこのテーブルでは行われない。

テーブル: `DAT_8049d030` (RACE) / `DAT_804bf830` (その他)
インデックス: `(courseId-1)*0xC0 + ccClass*0x40 + subMode*8 + kartSlot`
各エントリ: 0x5C (92) bytes

#### エントリ構造 (0x5C = 92 bytes, 23 words)

| Word | Offset | 型 | 参照元 | 説明 |
|------|--------|-----|--------|------|
| [0] | +0x00 | float | `GetKartTuningSpeedFactor` | **speedFactor** — AI目標速度の係数 |
| [1] | +0x04 | float | `PhysicsState_AccelInterpolate`, `KartPhysics_UpdateSmoothing` | **accelRate** — 入力スムージングの最大変化率。`maxDelta = 10.0 * 1000.0 * accelRate` |
| [2] | +0x08 | float | **コード未参照** | (未使用。データは存在するがリリース版では読まれない。カット機能の残骸?) |
| [3] | +0x0C | float | **コード未参照** | (同上) |
| [4] | +0x10 | float | `AI_CalcGPTargetSpeed` | **aheadDistThreshold** — 先行時のライバル距離閾値 (実際の比較値は `10000 * value`) |
| [5] | +0x14 | float | `AI_CalcGPTargetSpeed` | **aheadSpeedBonus** — 先行時の速度ボーナスベース |
| [6] | +0x18 | float | `AI_CalcGPTargetSpeed` | **aheadSpeedBonusExtra** — 先行時追加ボーナス ([5] に加算) |
| [7] | +0x1C | float | `AI_CalcGPTargetSpeed` | **behindDistThreshold** — 後方時のライバル距離閾値 |
| [8] | +0x20 | float | `AI_CalcGPTargetSpeed` | **behindSpeedBonus** — 後方時の速度ボーナスベース (負=減速) |
| [9] | +0x24 | float | `AI_CalcGPTargetSpeed` | **behindSpeedBonusExtra** — 後方時追加ボーナス ([8] に加算) |
| [10] | +0x28 | float | `AI_UpdatePerFrameState` | **speedAdjustTimer** — 速度調整タイマー (フレーム数 = `15 * value`) |
| [11] | +0x2C | int | `FUN_801e83c8` (カート初期化) | **aiVtableSelector** — bit 0 で AI行動パターンの vtable 分岐 |
| [12] | +0x30 | float | `FUN_801ec4b0`, `FUN_801ec5a4` | **steeringDamping** — 加減速/ステアリング力計算の係数 |
| [13-19] | +0x34~+0x4C | — | — | 未使用 (0 埋め) |
| [20] | +0x50 | int | `FUN_801ebc8c` (カートシステム初期化) | **aiPathModeFlag** — ==1 で初期化モード分岐 |
| [21-22] | +0x54~+0x58 | — | — | 未使用 (0 埋め) |

[4]~[10] は `GetKartTuningGPSpeedParams` (0x801de448) 経由でアクセスされる (エントリ+0x10を返す)。
GP AI速度計算ロジック: 先行時は `rivalDist² >= (10000 * [4])²` で `[5]+[6]` を加算、後方時は `rivalDist² >= (10000 * [7])²` で `[8]+[9]` を加算 (負値=減速)。

#### スロット別チューニング値 (コース1, 50cc, subMode 0)

> **注意**: このテーブルのインデックスはカートバリアントではなく**カートスロット番号 (0-7)**。
> `FUN_801ebc8c` のループインデックスが `PhysicsState+0x10` に格納され、以降の参照で使われる。
> Slot 0 = プレイヤー、Slot 1-7 = AI。

| フィールド | Slot 0 (Player) | Slot 1 (AI) | Slot 2 (AI) | Slot 3 (AI) | Slot 4 (AI) |
|-----------|----------------|------------|------------|------------|------------|
| [0] speedFactor | **0.6** | 0.5 | 0.4 | 0.3 | 0.2 |
| [1] accelRate | **0.8** | 0.8 | 0.8 | 0.6 | 0.5 |
| [2] (未参照) | 20.0 | 80.0 | 70.0 | 50.0 | 10.0 |
| [3] (未参照) | 50.0 | 50.0 | 40.0 | 30.0 | 25.0 |
| [4] aheadDistThresh | 10.0 | 10.0 | 10.0 | 10.0 | 10.0 |
| [5] aheadSpeedBonus | 10.0 | 10.0 | 10.0 | 5.0 | 5.0 |
| [7] behindDistThresh | 30.0 | 30.0 | 24.0 | 26.0 | 28.0 |
| [8] behindSpeedBonus | -5.0 | -5.0 | -5.0 | -5.0 | -5.0 |
| [10] speedAdjustTimer | 180.0 | 180.0 | 30.0 | 30.0 | 30.0 |
| [11] aiVtableSelector | 0 | 1 | 0 | 1 | 0 |
| [12] steeringDamping | 0.0 | 0.0 | 0.0 | 0.0 | 5.0 |

Slot 5-7 はセンチネル値 (0xFFFFFFFF) で未使用。

プレイヤー (Slot 0) は最も高い speedFactor・accelRate を持つ。AIスロットが大きいほどパラメータが低下する。
全コース・全CCの完全なデータは `mkgp2_kart_tuning_table.md` を参照。

## 全パラメータ共通定数

全キャラ・全カートタイプで変わらないフィールド:

| フィールド | 値 | 説明 |
|-----------|-----|------|
| maxInternalSpeed (+0x1C) | 1500.0 | 内部最高速度 |
| accelBase (+0x20) | 300.0 | 加速ベース値 |
| damping (+0x04) | 0.8 | 減衰係数 |
| steeringAngleLimit (+0x3C) | 15.0 | ステアリング角度上限 |
| driftIntensity (+0x88) | CC依存 | ccVariant で変化 (キャラ非依存) |
| driftAmplitude (+0x8C) | CC依存 | 同上 |

## データ参照

- キャラクターデータテーブル: `0x80401ce0` (15エントリ, stride 0x2C)
- カートパラメータポインタテーブル: `0x80317854` (16エントリ)
- GetCharacterName: `0x8009c84c`
- GetKartParamBlock: `0x80090ff0`
- カートタイプ名テーブル: `PTR_s_NORMAL_804e53d4`
- カート説明文テーブル: `0x8048e9f0` (JP+EN ペア×4タイプ, GetTextureByIdAndLangのID 0x95-0x98)
- キャラクタークラステーブル: `0x8039AEA4` (int[13], charId→classIdx 0-3)
- キャラスター評価テーブル: `0x8049AA58` ([frameIdx*0x27 + charId*3], 3 bytes per char)
- ローカライズテキストベーステーブル: `0x8048e548` (GetTextureByIdAndLangが参照)
- CharaSelect_SetupCharacterDisplay: `0x801c3470`
- clFlowChara_Update: `0x801c3d48`
- モデルファイルテーブル: `0x80401118` (NORMAL 0-38, GOLD 39-77, ORIGINAL 78-116)
