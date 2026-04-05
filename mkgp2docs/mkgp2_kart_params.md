# MKGP2 カートパラメータシステム

## 概要

MKGP2 には **4種のカートタイプ** と、キャラクターごとに **3つのCCバリアント** があり、それぞれ独自のパラメータブロックを持つ。
パラメータデータは main.dol に完全に埋め込まれている（外部ファイルからのロードなし）。

## カートタイプ

| Index | 内部名             | 説明 (EN)                                             |
|-------|--------------------|-------------------------------------------------------|
| 0     | NORMAL             | Easy control kart                                     |
| 1     | GOLD               | Strong acceleration kart                              |
| 2     | ORIGINAL           | Balanced kart                                         |
| 3     | ORIGINAL ENHANCED  | High speed, yet tricky advanced player's kart         |

カートタイプ名テーブル: `PTR_s_NORMAL_804e53d4`
カート説明文字列 (JP+EN ペア): `0x8048e9f0` (8ポインタ, 4タイプ x 2言語)

タイプ別ビジュアルモデル:
- NORMAL: `cart_<chara>.dat`
- GOLD: `gold_<chara>.dat`
- ORIGINAL / ORIGINAL ENHANCED: `sp_cart_<chara>.dat`

## CCバリアント

CCクラス (param_3) からバリアントインデックスへの対応:

| CCクラス | 値 | バリアントインデックス |
|----------|-----|----------------------|
| 50cc     | 0   | 0                    |
| 100cc    | 1   | 0 (50ccと同一)       |
| 150cc    | 2   | 1                    |
| (特殊)   | 3   | 2                    |

## パラメータ参照 (GetKartParamBlock @ 0x80090ff0)

**修正済みシグネチャ**: `GetKartParamBlock(int charId, int ccClass, int kartVariant, int ccVariant)`

注意: 旧コメントではparam2を"kartType"、param3を"ccClass"と呼んでいたが、
実際のCallerデータフローからparam2=ccClass, param3=kartVariant が正しい。
詳細は `mkgp2_kart_type_enum_analysis.md` 参照。

### プレイヤーパス (ccVariant == -1):
```
normalizedKartIdx = {NORMAL:0, GOLD:0, ORIGINAL:1, ORIGINAL_ENHANCED:2}[kartVariant]
paramBlock = *(PTR_PTR_80317854[charId] + ccClass * 0xC + normalizedKartIdx * 4)
```

- キャラクターポインタテーブル: `0x80317854` (16エントリ, キャラ1体につき1つ)
- 各キャラクターエントリ: 9ポインタ = 3ccClass * 3normalizedKartVariant (NORMAL/GOLD共有)

### 特殊モードパス (ccVariant >= 0):

> **訂正 (2026-04-03)**: 以前は「敵/AIパス」と記載していたが、ccVariant テーブルは AI 用ではなく**特殊モード用プレイヤーパラメータ**である。AI カートは `FUN_801e83c8` で独自初期化され、KartParamBlock を使用しない。AI の速度は BaseSpeed テーブル (0x803A01E8) で決定される。

```
paramBlock = PTR_DAT_80317888[ccVariant * 3 + ccClass]
```

- 特殊モードテーブル: `g_kartParamCcVariantTable` (0x80317888), 全8 ccVariant * 3 ccClass = 24エントリ
- ccClass 間のデータ差はなし (同一ブロックを指す)

#### ccVariant 別パラメータ比較

| ccVariant | 用途 | speedMax | mue | 特徴 |
|-----------|------|----------|-----|------|
| 0, 2, 4 | カート試乗/プレビュー | 110.0 | 0.400 | 基本パラメータ |
| 1 | タイムトライアル | 300.0 | 0.210 | TT専用: 速度最大+mue低下 |
| 3, 5, 6 | ミニゲーム/特殊モード | 300.0 | 0.400 | 速度増加版 |
| 7 | 特殊モード | 250.0 | 0.400 | 中間値 |

#### ccVariant の設定元

| 呼び出し元 | ccVariant | 用途 |
|-----------|-----------|------|
| RaceInit / BattleInit | -1 | プレイヤー通常レース (プレイヤーパスへ) |
| FUN_800ad910 (TestDrive_Init) | 0 | カート試乗 |
| FUN_80129938 | 1 | タイムトライアル |
| SuikaBall_Init (0x80133398) | 0 | スイカ割りミニゲーム (courseId=9) |
| FUN_80134ff0 | 3 | ミニゲーム (速い) |
| FUN_80214db8 | 6 | 特殊モード |
| FUN_80213210 | 7 | 特殊モード |

## パラメータブロック構造 (~0x1D8 = 472 bytes)

> Ghidra上に `KartParamBlock` 構造体 (472バイト) を作成済み。

### 主要パラメータ (0x00 - 0x8F)

| Offset | 型    | Mario (中量級) | Wario (重量級) | Toad (軽量級) | 説明                           |
|--------|-------|-----------------|---------------|--------------|--------------------------------|
| 0x00   | float | 1.0             | 1.0           | 1.0          | scale_x (常に1.0)             |
| 0x04   | float | 0.8             | 0.8           | 0.8          | 減衰/減速係数                  |
| 0x08   | float | 0.0             | 0.0           | 0.0          | (未使用?)                      |
| 0x0C   | float | 1.0             | 1.0           | 1.0          | スケール (常に1.0)             |
| 0x10   | float | 1.0             | 1.0           | 1.0          | スケール (常に1.0)             |
| 0x14   | float | 1.0             | 1.0           | 1.0          | スケール (常に1.0)             |
| 0x18   | float | 1.0             | 1.0           | 1.0          | スケール (常に1.0)             |
| 0x1C   | float | 1500.0          | 1500.0        | 1500.0       | **maxInternalSpeed** (定数)    |
| 0x20   | float | 300.0           | 300.0         | 300.0        | **accelBase** (定数)           |
| 0x24   | float | 210.0           | 210.0         | 210.0        | (全キャラ共通の定数)           |
| **0x28** | float | **0.21**      | **0.19**      | **0.285**    | **mue (摩擦/グリップ)**        |
| **0x2C** | float | **0.016**     | **0.015**     | **0.018**    | **gripCoefficient**            |
| 0x30   | float | 1.0             | 1.0           | 1.0          | (常に1.0)                      |
| 0x34   | float | 1.0             | 1.0           | 1.0          | (常に1.0)                      |
| 0x38   | float | 1.0             | 1.0           | 1.0          | (常に1.0)                      |
| 0x3C   | float | 15.0            | 15.0          | 15.0         | ステアリング角度上限           |
| 0x40   | float | 1.0             | 1.0           | 1.0          | ステアリングスケール (→ moveObj+0x3C) |
| 0x44   | float | 0.7             | 0.7           | 0.7          | ステアリング応答性             |
| 0x48   | float | 45.0            | 45.0          | 45.0         | (定数)                         |
| 0x4C   | float | 0.45            | 0.45          | 0.45         | (定数)                         |
| 0x50   | float | 1.0             | 1.0           | 1.0          | (定数)                         |
| 0x54   | float | 1.0             | 1.0           | 1.0          | (定数)                         |
| **0x58** | float | **2.0**       | **3.0**       | **1.0**      | **weight (重量)**              |
| 0x5C   | int   | 1               | 1             | 1            | (フラグ)                       |
| **0x60** | float | **0.00125**   | **0.00133**   | **0.00125**  | 加速ゲインレート               |
| **0x64** | float | **20.0**      | **18.0**      | **20.0**     | 旋回角度 (ドリフト角度)        |
| **0x68** | float | **0.0**       | **0.00125**   | **0.0**      | ドリフトバイアス               |
| 0x6C   | float | 22.0            | 22.0          | 22.0         | (定数)                         |
| **0x70** | float | **0.02**      | **0.025**     | **0.02**     | スリップ係数                   |
| **0x74** | float | **~0.000025** | **~0.00002**  | **~0.000025**| 微小スリップレート             |
| **0x78** | float | **0.1**       | **0.08**      | **0.1**      | 旋回レート / ハンドリング精度  |
| 0x7C   | float | 0.007           | 0.007         | 0.007        | (定数)                         |
| **0x80** | float | **0.005**     | **0.0025**    | **0.005**    | 復帰レート                     |
| **0x84** | float | **0.0**       | **0.0**       | **1.389**    | コーナーブースト (軽量級のみ)  |
| 0x88   | float | 1.706 (cc0) / 2.384 (cc1) | 可変 | 可変    | **ドリフト強度** (CC依存)      |
| 0x8C   | float | 2.059 (cc0) / 3.110 (cc1) | 可変 | 可変    | **ドリフト振幅** (CC依存)      |

### ホイール位置 (0x90 - 0xEF)

3floatずつ8エントリ (カート中心からの x, y, z オフセット):
- エントリ 0-3: 前後輪位置 (例: 8.0, -6.0, 0.0 / 8.0, 6.0, 0.0 / -8.0, -6.0, 0.0 / -8.0, 6.0, 0.0)
- エントリ 4-7: 複製セット (当たり判定用 vs 表示用の可能性)

### 速度テーブルポインタ (0xF0 - 0xFF)

| Offset | 型      | 値 (Mario)     | 説明                          |
|--------|---------|----------------|-------------------------------|
| 0xF0   | ptr     | 0x8038fe98     | **speedTablePtr** — 速度テーブル配列へのポインタ |
| 0xF4   | int     | 7              | 速度テーブルエントリ数        |
| 0xF8   | (pad)   | 0              |                               |
| 0xFC   | (pad)   | 0              |                               |

**+0xF0 のポインタはカートバリアントごとに異なる値を指す。** これにより、NORMAL/ORIGINAL/ENHANCED で異なる速度カーブが適用される。

#### Peach (charId=3) 150cc での speedTablePtr 比較

| バリアント | speedTablePtr | 指す先 |
|-----------|---------------|--------|
| NORMAL | 0x80394bb8 | 段階的加速カーブ |
| ORIGINAL | 0x80394d58 | 段階的加速カーブ (NORMAL より高速) |
| ENHANCED | 0x80394d90 | ORIGINALのSpeedTable (チェーン構造) |

#### Yoshi (charId=7) 150cc での speedTablePtr 比較

| バリアント | KartParamBlockアドレス | speedTablePtr | 指す先 |
|-----------|----------------------|---------------|--------|
| NORMAL | 0x80398820 | 0x80398778 | 前ccブロックのテーブル → カーブ最大値 76.66 |
| ORIGINAL | 0x803989F8 | 0x80398950 (= NORMAL+0x130) | NORMALのカーブ → カーブ最大値 78.06 |
| ENHANCED | 0x80398BD0 | 0x80398b28 (= ORIGINAL+0x130) | ORIGINALのカーブ → カーブ最大値 78.89 |

> **ピーチとヨッシーは同じ GOLD クラス (classIdx=1) であり、150cc の速度カーブデータは完全に同一。**
> 同クラス内では物理プロファイルも一致するため (両方 Profile A: Standard Medium)、バリアント間の速度差・加速カーブも同じパターンになる。

### 速度カーブテーブル (0x100 - 0x12F) — バリアント間で異なる

+0x100 以降に 6ペアの (距離閾値, 速度値) が格納される。

+0x00〜+0xEF の物理パラメータ部分は ORIGINAL と ENHANCED でほぼ同一 (ドリフトパラメータ +0x88/+0x8C のみ NORMAL と異なるが ORIGINAL/ENHANCED 間は一致) だが、+0x100〜 のスピードカーブは異なる。

#### Peach/Yoshi 150cc での inline 速度カーブ比較 (+0x100)

Peach と Yoshi は同クラス (GOLD, classIdx=1) のため、inline カーブデータは同一。

| 距離閾値 | NORMAL | ORIGINAL | ENHANCED |
|----------|--------|----------|----------|
| 500 | 19.51 | 22.88 | 66.67 |
| 1000 | 39.34 | 42.92 | 66.67 |
| 2000 | 51.20 | 57.27 | 66.67 |
| 3000 | 58.85 | 68.16 | 66.67 |
| 4000 | 68.85 | 75.89 | 66.67 |
| 5000 | 78.06 | 78.89 | 66.67 |

**解釈:**
- **NORMAL**: 距離に応じて段階的に加速する標準カーブ
- **ORIGINAL**: NORMAL より各段階で高い速度値。最終速度もわずかに高い
- **ENHANCED**: 全距離閾値で同一の速度値 (66.67) のフラットカーブ

#### 実効スピードカーブ (speedTablePtr curveDataPtr 経由)

speedTablePtr チェーン構造により、各バリアントが実際に使用するスピードカーブは以下の通り。
Peach と Yoshi は同クラスのため同一データ。

**NORMAL が使用するカーブ** (ORIGINAL がチェーン経由で参照):

| 距離 | 速度 |
|------|------|
| 0 | 0.0 |
| 500 | 19.51 |
| 1000 | 39.34 |
| 2000 | 51.20 |
| 3000 | 58.85 |
| 4000 | 68.85 |
| 5000 | 78.06 |

**ORIGINAL が使用するカーブ** (ENHANCED がチェーン経由で参照):

| 距離 | 速度 |
|------|------|
| 0 | 0.0 |
| 500 | 22.88 |
| 1000 | 42.92 |
| 2000 | 57.27 |
| 3000 | 68.16 |
| 4000 | 75.89 |
| 5000 | 78.89 |

#### speedTablePtr チェーン構造と speedCap への影響

> **訂正 (2026-04-03)**: 以前は「スピードカーブの違いが速度差の直接原因」としていたが、より正確なメカニズムが判明した。

各バリアントの speedTablePtr (+0xF0) は**前のバリアント**のスピードテーブルを指すチェーン構造になっている:

| Variant | speedTablePtr (Peach/Yoshi 150cc) | 参照先 | カーブ最大値 | 実測物理速度 |
|---------|----------------------------------|--------|------------|------------|
| NORMAL | 前ccブロック | 前ccブロック | 76.66 | ≈ 276 (138 km/h) |
| ORIGINAL | NORMAL+0x130 | NORMALのカーブ | 78.06 | ≈ 281 (140 km/h) |
| ENHANCED | ORIGINAL+0x130 | ORIGINALのカーブ | 78.89 | ≈ 284 (142 km/h) |

**SpeedCurve_UpdateSpeedCap (0x8019ef90)** が毎フレーム呼ばれ:
1. KartMovement+0x0C (`travelProgress`, 0→5000に漸増) をスピードカーブで参照
2. 結果を KartMovement+0x10 (`speedCap`) にキャッシュ
3. PhysicsStep (0x8019cd30) が speedCap を駆動力スケーリングに使用

結果として、各バリアントの speedCap は前バリアントのカーブ最大値で決まる。ENHANCED のフラットカーブ (66.67) 自体が speedCap になるのではなく、speedTablePtr が指す ORIGINAL のカーブ最大値 (78.89) が ENHANCED の speedCap になる。

#### 最高速度比較 (Peach/Yoshi 150cc)

| Variant | speedCap | displaySpeed (x3.6) | 表示 km/h (÷2) |
|---------|---------|---------------------|-----------------|
| NORMAL | 76.66 | 275.9 | ~138 |
| ORIGINAL | 78.06 | 281.0 | ~140 |
| ENHANCED | 78.89 | 284.0 | ~142 |

#### 加速特性の比較 (Peach/Yoshi 150cc)

- **最高速**: ENHANCED (78.89) > ORIGINAL (78.06) > NORMAL (76.66)
- **加速の立ち上がり**: ENHANCED (ORIGINALカーブ使用) > NORMAL (前ccカーブ使用) > ORIGINAL (NORMALカーブ使用)
  - ORIGINALは中間域でNORMALより低い速度値のカーブを使うため、加速初期はNORMALに劣る
- **ドリフト**: NORMAL (1.705/2.059) がやや穏やか、ORIGINAL/ENHANCED (1.720/2.100) がやや強い

詳細なデータフロー: `mkgp2_speed_curve_dataflow.md` を参照。

### 速度テーブルのサブ構造 (0x130+)

+0x130 以降にはサブテーブルが続く。バリアントにより構造が異なる:

- **NORMAL/ORIGINAL**: ポインタ + count(7) + パラメータ の構造が繰り返される
- **ENHANCED**: フラット値のペアのみ (サブテーブルポインタは null)

#### 速度テーブルエントリの共通構造

各エントリ 0x18 (24) bytes:
| Offset | 型    | 例            | 説明                  |
|--------|-------|---------------|-----------------------|
| 0x00   | ptr   | (サブカーブ)  | 速度サブカーブポインタ (ENHANCED では null) |
| 0x04   | int   | 7             | サブカーブエントリ数  |
| 0x08   | float | 5000.0        | このギアの最高速度    |
| 0x0C   | float | 21.0 / 31.5   | 加速レート            |
| 0x10   | float | 0.1           | 摩擦係数              |
| 0x14   | float | 0.3           | ブレーキ係数          |

エントリ [0] はキャラクターごとに異なる加速レートを持つ (例: Mario=21.0, Toad/Wario=31.5)。

## 全キャラクター x 全カートタイプ 詳細データ (ccVariant=0)

### 可変フィールド一覧

全キャラ・全カートタイプ共通の定数フィールド:
- damping=0.8, maxInternalSpeed=1500.0, accelBase=300.0
- driftIntensity/driftAmplitudeはccVariantのみで変動 (cc0: 1.705/2.059, cc1/cc2: 2.383/3.110)

変動する10フィールドのみ以下に記載。

### Profile A: "Standard Medium" (19ブロック)
- mue_grip=0.21, gripCoeff=0.016, weight=2.0, accelGain=0.00125
- turnAngle=20, driftBias=0, slip=0.02, turnRate=0.1, recovery=0.005, cornerBoost=0
- **該当**: Mario(N/G/O), Yoshi(N/G/O), Waluigi(N/G/O), Akabei(N/G/O/OE), Tanooki(N/G/O)
- **OEとして**: Peach.OE, Wario.OE, MsPacMan.OE

### Profile B: "Slightly Slippery Medium" (3ブロック)
- mue_grip=0.20, gripCoeff=0.016, weight=2.0 (Aより若干グリップ低い)
- **該当**: Luigi(N/G), Mario.OE

### Profile C: "Heavy Luigi ORIGINAL" (1ブロック - ユニーク)
- mue_grip=0.20, gripCoeff=0.016, **weight=5.0** (ゲーム内最重量)
- **該当**: Luigi.ORIGINAL のみ

### Profile D: "Heavy-Handling" (10ブロック)
- mue_grip=0.19, gripCoeff=0.015, weight=3.0, accelGain=0.001333
- turnAngle=18, driftBias=0.001, slip=0.025, turnRate=0.08, recovery=0.0025
- **該当**: Peach(N/G/O), Toad(N/G), Wario(N/G), Luigi.OE, Yoshi.OE, DK.OE

### Profile E: "Heavy-Handling Loose" (2ブロック)
- mue_grip=0.18 (Dより低い), turnAngle=20 (Dの18より広い), driftBias=0
- **該当**: Toad.ORIGINAL, Wario.ORIGINAL

### Profile F: "Light Speed" (12ブロック)
- mue_grip=0.285, gripCoeff=0.018, weight=1.0, **cornerBoost=1.389**
- **該当**: DK(N/G/O), MsPacMan(N/G/O), Mametchi(N/G/O), Toad.OE, PacMan.OE, Tanooki.OE
- 唯一cornerBoostを持つプロファイル。最軽量・最高グリップ

### Profile G: "Drift Medium" (4ブロック)
- mue_grip=0.205, **driftBias=0.01** (Profile Dの10倍)
- **該当**: PacMan(N/G/O), Waluigi.OE

### 全データテーブル

```
Character   KartType     mue_grip  gripCoeff  weight  accelGain  turnAngle  driftBias  slipCoeff  turnRate  recovery  cornerBoost  Profile
--------------------------------------------------------------------------------------------------------------------------------------
Mario       NORMAL       0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Mario       GOLD         0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Mario       ORIGINAL     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Mario       ORIG_ENH     0.200     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        B

Luigi       NORMAL       0.200     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        B
Luigi       GOLD         0.200     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        B
Luigi       ORIGINAL     0.200     0.016      5.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        C (!)
Luigi       ORIG_ENH     0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D

Peach       NORMAL       0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Peach       GOLD         0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Peach       ORIGINAL     0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Peach       ORIG_ENH     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A

Yoshi       NORMAL       0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Yoshi       GOLD         0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Yoshi       ORIGINAL     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Yoshi       ORIG_ENH     0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D

Toad        NORMAL       0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Toad        GOLD         0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Toad        ORIGINAL     0.180     0.015      3.0     0.00133    20.0       0.000      0.025      0.080     0.0025    0.000        E
Toad        ORIG_ENH     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F

DK          NORMAL       0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
DK          GOLD         0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
DK          ORIGINAL     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
DK          ORIG_ENH     0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D

Wario       NORMAL       0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Wario       GOLD         0.190     0.015      3.0     0.00133    18.0       0.001      0.025      0.080     0.0025    0.000        D
Wario       ORIGINAL     0.180     0.015      3.0     0.00133    20.0       0.000      0.025      0.080     0.0025    0.000        E
Wario       ORIG_ENH     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A

Waluigi     NORMAL       0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Waluigi     GOLD         0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Waluigi     ORIGINAL     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Waluigi     ORIG_ENH     0.205     0.016      2.0     0.00125    20.0       0.010      0.020      0.100     0.005     0.000        G

PacMan      NORMAL       0.205     0.016      2.0     0.00125    20.0       0.010      0.020      0.100     0.005     0.000        G
PacMan      GOLD         0.205     0.016      2.0     0.00125    20.0       0.010      0.020      0.100     0.005     0.000        G
PacMan      ORIGINAL     0.205     0.016      2.0     0.00125    20.0       0.010      0.020      0.100     0.005     0.000        G
PacMan      ORIG_ENH     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F

MsPacMan    NORMAL       0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
MsPacMan    GOLD         0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
MsPacMan    ORIGINAL     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
MsPacMan    ORIG_ENH     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A

Akabei      NORMAL       0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Akabei      GOLD         0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Akabei      ORIGINAL     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Akabei      ORIG_ENH     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A

Tanooki     NORMAL       0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Tanooki     GOLD         0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Tanooki     ORIGINAL     0.210     0.016      2.0     0.00125    20.0       0.000      0.020      0.100     0.005     0.000        A
Tanooki     ORIG_ENH     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F

Mametchi    NORMAL       0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
Mametchi    GOLD         0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
Mametchi    ORIGINAL     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
Mametchi    ORIG_ENH     0.285     0.018      1.0     0.00125    20.0       0.000      0.020      0.100     0.005     1.389        F
```

### ORIG_ENHチェーンパターン

各キャラのORIG_ENHは次キャラのNORMALとポインタ共有:
```
Mario.OE      -> Luigi.NORMAL       (Profile B)
Luigi.OE      -> Peach.NORMAL       (Profile D)
Peach.OE      -> Yoshi.NORMAL       (Profile A)
Yoshi.OE      -> Toad.NORMAL        (Profile D)
Toad.OE       -> DK.NORMAL          (Profile F)
DK.OE         -> Wario.NORMAL       (Profile D)
Wario.OE      -> Waluigi.NORMAL     (Profile A)
Waluigi.OE    -> PacMan.NORMAL      (Profile G)
PacMan.OE     -> MsPacMan.NORMAL    (Profile F)
MsPacMan.OE   -> Akabei.NORMAL      (Profile A)
Akabei.OE     -> Tanooki.NORMAL     (Profile A)
Tanooki.OE    -> Mametchi.NORMAL    (Profile F)
Mametchi.OE   -> [INVALID]
```

> **注意**: このチェーンパターンは +0x00〜+0xEF の物理パラメータ部分のポインタ共有のみを示す。
> +0xF0 (speedTablePtr) 以降のスピードカーブはバリアントごとに独立したデータを持ち、
> ORIGINAL と ENHANCED で大きく異なる。詳細は「速度カーブテーブル」セクションを参照。

### カートタイプ間の差異

ほとんどのキャラはNORMAL=GOLD=ORIGINALで同一データ。例外:
- **Luigi**: ORIGINAL のみ weight=5.0 (ゲーム内最重量)
- **Toad**: ORIGINAL は grip=0.18 (低い), driftBias=0
- **Wario**: ORIGINAL は grip=0.18 (低い), driftBias=0
- **Akabei**: N=G=O=OE すべて同一 (Profile A)

## キャラクター重量クラス (NORMALカート基準)

| 重量クラス | Mue (0x28) | Weight (0x58) | cornerBoost | キャラクター                             |
|------------|------------|---------------|-------------|------------------------------------------|
| 軽量級 (F) | 0.285      | 1.0           | 1.389       | DK, MsPacMan, Mametchi                   |
| 中量級A    | 0.21       | 2.0           | 0           | Mario, Yoshi, Waluigi, Akabei, Tanooki   |
| 中量級B    | 0.20       | 2.0           | 0           | Luigi (グリップ若干低い)                 |
| 中量級C    | 0.205      | 2.0           | 0           | PacMan (driftBias=0.01)                  |
| 重量級 (D) | 0.19       | 3.0           | 0           | Peach, Toad, Wario                       |

注意: 直感に反し、DK/MsPacMan/Mametchiが最軽量(weight=1.0)で最高グリップ、Peach/Toad/Warioが最重量級(weight=3.0)。

## カートタイプ説明文字列

ポインタテーブル: `0x8048e9f0` (先頭8エントリがカート説明、後続8エントリはカードエラーメッセージ)

| Index | タイプ             | JP                                     | EN                                                    |
|-------|--------------------|-----------------------------------------|-------------------------------------------------------|
| 0     | NORMAL             | とにかく走りやすい初心者向けカート       | Easy control kart                                     |
| 1     | GOLD               | 加速バツグンの初級者向けカート           | Strong acceleration kart                              |
| 2     | ORIGINAL           | バランスに優れた中級者向けカート         | Balanced kart                                         |
| 3     | ORIGINAL ENHANCED  | 速いけどクセのある上級者向けカート       | High speed,yet tricky advanced player's kart          |

### カートアンロックメッセージ (0x8048e840 周辺)

| JP                           | EN                          |
|------------------------------|-----------------------------|
| 専用カートをゲット！          | Get %s original kart!       |
| %sの専用カートを              | Get %s power up parts!      |
| チューンナップしました！      |                             |

## キャラクターID と内部名

テーブル: `0x80401ce0` (stride 0x2C = 44 bytes, 15エントリ)

| ID | 表示名      | 内部名    | 備考             |
|----|-------------|-----------|------------------|
| 0  | MARIO       | mario     |                  |
| 1  | LUIGI       | luigi     |                  |
| 2  | WARIO       | wario     |                  |
| 3  | PEACH       | peach     |                  |
| 4  | KOOPA       | koopa     | クッパ (Bowser)  |
| 5  | KINOPIO     | kinopio   | キノピオ (Toad)  |
| 6  | DONKEY      | donkey    | DK               |
| 7  | YOSHI       | yoshi     |                  |
| 8  | PACMAN      | pacman    |                  |
| 9  | MS PACMAN   | mspacman  |                  |
| 10 | MONSTER     | monster   | Akabei           |
| 11 | WALUIGI     | waluigi   |                  |
| 12 | MAMETH      | mametch   | まめっち (Tamagotchi) |
| 13 | NO CHARA    | -         | プレースホルダー |
| 14 | NO CHARA    | -         | プレースホルダー |

## カートモデルファイル

### モデルテーブル

ベースアドレス: `0x80401118` (string pointer の flat array)
レイアウト: 13キャラ x 3 LOD x 3モデルタイプ = 117エントリ

各キャラクターにつき3つのLOD:
- `<name>.dat` -- 高品質 (近距離)
- `<name>_mid.dat` -- 中品質
- `<name>_low.dat` -- 低品質 (遠距離)

### タイプ別モデルパターン

| タイプ             | ファイル名パターン       | テーブル内位置            |
|--------------------|--------------------------|---------------------------|
| NORMAL (0)         | `cart_<chara>.dat`       | idx 0-38                  |
| GOLD (1)           | `gold_<chara>.dat`       | idx 39-77                 |
| ORIGINAL (2)       | `sp_cart_<chara>.dat`    | idx 78-116                |
| ORIGINAL ENHANCED (3) | `sp_cart_<chara>.dat` | 同上 (ORIGINALと共有)    |

ORIGINAL と ORIGINAL ENHANCED は同じ `sp_cart_` モデルを使用する。
違いはビジュアルではなくゲームプレイパラメータ (速度/ハンドリング)。

### 全モデルファイル一覧

**NORMAL (cart_)**:
```
cart_mario.dat     cart_luigi.dat     cart_wario.dat     cart_peach.dat
cart_koopa.dat     cart_kinopio.dat   cart_donkey.dat    cart_yoshi.dat
cart_pacman.dat    cart_mspacman.dat  cart_monster.dat   cart_waluigi.dat
cart_mametch.dat
```

**GOLD (gold_)**:
```
gold_mario.dat     gold_luigi.dat     gold_wario.dat     gold_peach.dat
gold_koopa.dat     gold_kinopio.dat   gold_donkey.dat    gold_yoshi.dat
gold_pacman.dat    gold_mspacman.dat  gold_monster.dat   gold_waluigi.dat
gold_mametch.dat
```

**ORIGINAL / ORIGINAL ENHANCED (sp_cart_)**:
```
sp_cart_mario.dat     sp_cart_luigi.dat     sp_cart_wario.dat     sp_cart_peach.dat
sp_cart_koopa.dat     sp_cart_kinopio.dat   sp_cart_donkey.dat    sp_cart_yoshi.dat
sp_cart_pacman.dat    sp_cart_mspacman.dat  sp_cart_monster.dat   sp_cart_waluigi.dat
sp_cart_mametch.dat
```

### 追加パーツファイル (テーブル末尾 idx 117+)

```
mario_handle.dat          -- ハンドル
mario_item_handle.dat     -- アイテム用ハンドル
mario_tamb_cen.dat        -- タンバリン (中央)
mario_item_pa_tamb.dat    -- パーカッションタンバリン
mario_item_goo_tamb.dat   -- グータンバリン
mario_tamb_l.dat          -- タンバリン (左)
```

### デモ用カートモデル (別テーブル: 0x8039aef4)

```
kart_demo_mario.dat    kart_demo_luigi.dat    kart_demo_pacman.dat
kart_demo_waluigi.dat  kart_demo_peach.dat    kart_demo_yoshi.dat
kart_demo_monster.dat  kart_demo_mametch.dat  kart_demo_mspacman.dat
kart_demo_kinopio.dat  kart_demo_wario.dat    kart_demo_donkey.dat
kart_demo_koopa.dat
```

### 共通パーツ

- `cart_tire.dat` (0x80311784) -- タイヤモデル (全カート共通)
- `sp_sel_kart_1.tpl` (0x80334140) -- カート選択UI用テクスチャ

## 主要関数 (Ghidra)

| アドレス    | 名前                           | 用途                                             |
|-------------|--------------------------------|--------------------------------------------------|
| 0x80090ff0  | GetKartParamBlock              | (charId, kartType, ccVar) でパラメータブロック選択 |
| 0x8004e618  | CarObject_Init                 | カーオブジェクト生成、全サブ初期化呼び出し       |
| 0x8019e20c  | KartMovement_Init              | 移動オブジェクト初期化 (0x324 bytes)             |
| 0x801998f8  | DebugOverlay_KartPhysics       | 実行時に mue_scale/spd_scale を表示              |
| 0x801999e0  | KartMovement_CalcMaxSpeed      | 速度テーブルから最高速度を計算                   |
| 0x80199a84  | KartMovement_CalcCurrentSpeed  | 速度ベクトルから現在速度を計算                   |
| 0x8019b080  | KartMovement_CollisionUpdate   | 地面/壁衝突 + 物理更新メイン                     |
| 0x801f4eb0  | DebugOverlay_RaceConfig        | カートタイプ、ステージ等のデバッグオーバーレイ   |
| 0x8009c84c  | GetCharacterName               | インデックス (0-14) からキャラクター名文字列を返す |

## 主要データテーブル

| アドレス    | 説明                                                   |
|-------------|--------------------------------------------------------|
| 0x80317854  | キャラクター別カートパラメータポインタテーブル (16キャラ)|
| 0x80317888  | 特殊モード用カートパラメータポインタテーブル (g_kartParamCcVariantTable, 8 ccVariant * 3 ccClass) |
| 0x80401ce0  | キャラクターデータテーブル (15キャラ, stride 0x2C)     |
| 0x804e53d4  | カートタイプ名文字列ポインタテーブル (NORMAL, GOLD...) |
| 0x804e53c8  | CCクラス名文字列ポインタテーブル (50cc, 100cc, 150cc)  |
| 0x8048e9f0  | カート説明文字列ポインタ (JP+EN * 4タイプ)             |

## 移動オブジェクトレイアウト (0x324 bytes)

移動オブジェクト (KartMovement_Init で生成) の主要オフセット:

| Offset | 型    | 説明                                           |
|--------|-------|------------------------------------------------|
| 0x00   | int   | キャラクターID                                 |
| 0x04   | ptr   | → カートパラメータブロック                     |
| 0x08   | int   | 現在の速度テーブルインデックス (ギア)          |
| 0x0C   | float | 現在の加速目標値                               |
| 0x24   | ptr   | → 速度テーブル配列 (パラメータブロック +0xF0 由来) |
| 0x38   | float | パラメータブロックからのmue (→ moveObj+0x38)   |
| 0x40-0x48 | float[3] | 位置 (x, y, z)                            |
| 0x58-0x94 | float[16] | 姿勢行列 (4x4)                           |
| 0x17C-0x184 | float[3] | 速度ベクトル (vx, vy, vz)              |
| 0x1C4  | float | ヨー角速度                                     |
| 0x1C8  | float | 累積ヨー                                       |
| 0x2D8  | float | **speedScale** (EffectSpeed駆動力倍率)         |
| 0x2E0  | float | **coinSpeedBonus** (コイン速度ボーナス)        |
| 0x2F0  | float | **mue_scale** (デバッグオーバーレイ表示用)     |
| 0x2F4  | float | **spd_scale** (デバッグオーバーレイ表示用)     |
| 0x2F8  | float | **mueScale** (EffectSpeed最高速度スケール)     |
| 0x300  | float | シ���ュレーションタイムステップ / スケール係数  |

## turnAngle / driftBias / slipCoefficient 物理エンジン使用解析

### 解析対象
- `KartMovement_PhysicsStep` (0x8019cd30) 内のヨー角補正ブロック (0x8019daf0-0x8019dc48)

### 前提: 角度差の計算
PhysicsStep の終盤で、走行方向ベクトル (velocity, +0x17C) と駆動力方向ベクトル (driveForce, +0x170) の
XZ平面射影をそれぞれ正規化し、差分→asin→rad2deg で角度差(度)を求める:

```
velDirXZ = normalize(velocity.xz)
forceDirXZ = normalize(driveForce.xz)
crossSum = cross(velDirXZ, forceDirXZ).x + .y + .z
angleDiff = rad2deg(asin(clamp(crossSum, -1.0, 1.0)))
angleDiff = clamp(angleDiff, -paramBlock->param6C, paramBlock->param6C)
```

この `angleDiff` は「カートの進行方向と駆動力方向のずれ」を度で表す。

### turnAngle (paramBlock+0x64) — ヨー角補正閾値

**場所**: 0x8019db90 (`lfs f0, 0x64(r3)`)

**使い方**: `abs(angleDiff)` と `turnAngle` を比較し、3つの補正パスを分岐する。

```
if abs(angleDiff) <= turnAngle:
    → 微修正パス (小さなずれ)
else:
    → スリップ/ドリフト補正パス (大きなずれ)
```

**判定**: 現在の説明「ステアリング角度 — ハンドルの最大切れ角」は **不正確**。
正確には「ヨー角補正の閾値(度) — この値以下の方向ずれは微修正、超過するとスリップ/ドリフト補正が適用される」。
値は18〜20度。

### driftBias (paramBlock+0x68) — 直進時の微細ヨー補正ゲイン

**場所**: 0x8019dc34 (`lfs f2, 0x68(r3)`)

**使い方**: `abs(angleDiff) <= turnAngle` かつ `yawAngularVelocity == 0` (ハンドルニュートラル) の場合のみ:

```
if abs(angleDiff) <= turnAngle && yawAngularVelocity == 0.0:
    yaw += angleDiff * driftBias
```

ハンドルを切っていない時に、進行方向のずれを自動的に吸収する。
値が大きいほどずれを速く修正する = ドリフトに入りやすくなる。

**判定**: 現在の説明「ドリフト傾向 — ドリフトへの入りやすさ」は **概ね正確だが補足が必要**。
正確には「直進ニュートラル時の自動ヨー補正率 — 値が大きいほど方向ずれを速く修正する」。
Profile G (PacMan系) の 0.01 が最大値で、Profile A/B/C/E/F は 0.0 (自動補正なし)。
driftBias=0 のキャラはニュートラル時にずれが残存するため、結果的にドリフトに入りにくい。
driftBias=0.01 のキャラは微小なずれをすぐ解消し、ハンドル操作に対してよりレスポンシブに動く。

### slipCoefficient (paramBlock+0x70) — ドリフト時のヨー補正ゲイン

**場所**: 0x8019dbcc (`lfs f3, 0x70(r3)`)

**使い方**: `abs(angleDiff) > turnAngle` かつ **ドリフト中** (KartMovement+0x2FC == 1 or +0x2FD == 1) の場合:

```
if abs(angleDiff) > turnAngle && (isDrifting || isArcadeSlide):
    correction = angleDiff * slipCoeff
               + 0.5 * angleDiff * yawInput * paramBlock->microSlipRate(+0x74)
    yaw -= correction
```

第1項 (`angleDiff * slipCoeff`) が方向ずれに比例した基本補正、
第2項 (`0.5 * angleDiff * yawInput * microSlipRate`) がプレイヤーの操舵入力に応じた追加補正。

**非ドリフト時** (`abs(angleDiff) > turnAngle` かつ通常走行):
```
correction = angleDiff * paramBlock->param7C(+0x7C) * (paramBlock->turnRate(+0x78) + abs(steerAngle))
yaw -= correction
```
こちらでは slipCoefficient は使われず、turnRate/param7C で補正される。

**判定**: 現在の説明「滑りやすさ — 横方向へのスリップのしやすさ」は **不正確**。
正確には「ドリフト時のヨー角補正ゲイン — 値が大きいほどドリフト中にカートの向きが速く進行方向に戻る」。
slipCoeff が大きい = スリップ(ずれ)を速く解消する = ドリフトが制御しやすい。
「滑りやすさ」とは逆の効果。値は 0.02〜0.025。

### 関連パラメータ (参考)

| Offset | 名前           | 使用箇所                          | 説明                                    |
|--------|----------------|-----------------------------------|-----------------------------------------|
| +0x6C  | param6C        | angleDiff の clamp 上限           | 角度差の最大値(度)                      |
| +0x74  | microSlipRate  | ドリフト時の操舵追加補正ゲイン    | yawInput に乗じるドリフト操舵ゲイン     |
| +0x78  | turnRate       | 非ドリフト時の基本補正率          | 非ドリフト大ずれ時の基本補正            |
| +0x7C  | param7C        | 非ドリフト時の補正スケーラー      | turnRate と操舵角の合計に乗じる係数     |
