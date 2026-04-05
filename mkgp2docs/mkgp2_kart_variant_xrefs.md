# g_kartVariant (0x806cf118) 全クロスリファレンス解析

## 変数概要

- アドレス: `0x806cf118`
- 値: 0=NORMAL, 2=ORIGINAL, 3=ORIGINAL_ENHANCED, -1=未設定(初期値)
- SetKartVariant (0x8009bb28): 単純な書き込み。他の場所への書き込みは行わない
- CarObject+0x1C にも格納 (CarObject_Init で `param_4[7] = kartVariant`)
- playerData+0x1b9 にも格納 (FUN_801d5d98)

## 全xref一覧

### WRITE (セッター)

| Address | Function | 内容 |
|---------|----------|------|
| 0x8009bb28 | SetKartVariant | `DAT_806cf118 = param_1;` 単純代入 |
| 0x8009dc0c | FUN_8009d6cc (ゲーム初期化) | `DAT_806cf118 = 0xffffffff;` 初期化 |

### READ: 値を別の場所にコピーするだけ (差異なし)

| Address | Function | 内容 |
|---------|----------|------|
| 0x80082bbc | FUN_8008299c (ネットワークパケットビルダー) | パケットにビット埋め込み `(DAT_806cf118 << 1) & 6` |
| 0x800a1fa0 | FUN_800a1d80 (レース初期化) | `CarObject_Init(..., g_ccClass, g_kartVariant, ...)` に渡す |
| 0x800a3dd8 | FUN_800a3c38 (バトル初期化) | 同上 |
| 0x801f5bd4 | FUN_801f5b54 (リプレイデータ初期化1) | `param_1[0x10] = DAT_806cf118;` 構造体にコピー |
| 0x801f5c90, 0x801f5dd4 | FUN_801f5bf4 (リプレイデータ初期化2) | `param_1[0xd] = DAT_806cf118;` 構造体にコピー。内部でFUN_801f5b54も呼ぶ |
| 0x8021c238 | FUN_8021c070 (TA結果画面初期化) | `DAT_80679c98 = DAT_806cf118;` → `DAT_805d25a4` にコピー (ランキングデータ) |
| 0x8021d19c | FUN_8021c778 (TA結果画面更新) | パスワード生成 `FUN_8018b31c(..., DAT_806cf118, ...)` に渡す |
| 0x802482c0 | FUN_80248248 (レース後処理) | `FUN_801d5d98(&g_playerData, DAT_806cf118);` → playerData+0x1b9に格納 |

### READ: variant >= 2 (ORIGINAL/ENHANCED) で機能有効化 (2と3に差異なし)

これらの関数はすべて `DAT_806cf118 < 2` をチェックし、NORMAL(0)の場合は機能をスキップする。
ORIGINAL(2)とENHANCED(3)は同じパスを通る。

| Address | Function | ゲートされる機能 |
|---------|----------|-----------------|
| 0x800bcdc0 | FUN_800bc26c (カートエフェクト/ビジュアル更新) | ノードスケーリング有効化 + per-node scale table参照 |
| 0x800bd414, 0x800bd4d8 | FUN_800bd1bc (カートビジュアル別更新) | 同上 (2箇所で同じチェック) |
| 0x800bd9dc, 0x800bdaac | FUN_800bd900 (カートビジュアル別更新) | 同上 |
| 0x800bde70 | FUN_800bdd74 (カートビジュアル別更新) | 同上 |
| 0x800bf83c | FUN_800bf5b0 (カートビジュアル別更新) | 同上 |
| 0x800c2214 | FUN_800c2124 (カートビジュアル別更新) | ノードスケール + node_index==0xb時に特殊スケール値 |
| 0x800c232c | FUN_800c2124 (同上、2箇所目) | 同上のゲート |

### READ: ORIGINAL(2)とENHANCED(3)で異なる動作をするもの

| Address | Function | 内容 |
|---------|----------|------|
| **0x800a8004** | **FUN_800a8004 (称号チェック: ENHANCED専用)** | `DAT_806cf118 == 3` のみ: TIME_ATTACKで無敗かつスコア条件達成時に称号0x57を付与 |
| **0x800a85b0** | **FUN_800a8538 (称号チェック: ORIGINAL専用)** | `DAT_806cf118 == 2` のみ: キャラ別COIN_MILAGE範囲内なら称号0x3Fを付与 |

## ORIGINAL(2) と ENHANCED(3) で異なる動作をするもの

### 速度差 (speedCap)

> **訂正 (2026-04-03)**: 以前は称号システムのみが実動作差としていたが、速度差も存在する。

speedTablePtr チェーン構造により、ENHANCED の speedCap は ORIGINAL のカーブ最大値、ORIGINAL の speedCap は NORMAL のカーブ最大値で決まる。SpeedCurve_UpdateSpeedCap (0x8019ef90) が毎フレーム speedCap (KartMovement+0x10) を更新し、PhysicsStep が駆動力スケーリングに使用する。

動的テスト (Peach 150cc): ORIGINAL velocity=78.06, ENHANCED velocity=78.89

詳細: `mkgp2_speed_curve_dataflow.md`

### 称号システムでの差異

### FUN_800a8004 -- ENHANCED(3)専用称号

```c
if ((DAT_806cf118 == 3) && (g_gameMode == TIME_ATTACK) && (g_lossCount == 0) &&
    (g_scoreThresholdBase + 5 <= g_totalScore)) {
    g_stagingTitleId = 0x57;  // Title ID 87
    g_titleParam = 0;
    return 1;
}
```

条件: ENHANCEDカート + タイムアタック + 無敗 + スコア条件 → 称号87

### FUN_800a8538 -- ORIGINAL(2)専用称号

```c
if (iVar3 < 0) {  // 汎用マイルストーン範囲外
    if (DAT_806cf118 == 2) {
        // キャラ別COIN_MILAGE範囲テーブル (DAT_80326a64)
        if (coinMilage >= charMin && coinMilage <= charMax) {
            g_stagingTitleId = 0x3F;  // Title ID 63
            g_titleParam = 0;
            return 1;
        }
    }
}
```

キャラ別COIN_MILAGE範囲 (DAT_80326a64):
| charId | キャラ | min | max |
|--------|--------|-----|-----|
| 0 | Mario | 30 | 70 |
| 1 | DK | 1470 | 1510 |
| 2 | Wario | 1230 | 1270 |
| 3 | Pac-Man | 150 | 190 |
| 4 | Bowser | 750 | 790 |
| 5 | Toad | 1350 | 1390 |
| 6 | Luigi | 270 | 310 |
| 7 | Waluigi | 630 | 670 |
| 8 | Ms. Pac-Man | 990 | 1030 |
| 9 | Ms. Pac-Man | 390 | 430 |
| 10 | Akabei (アカベイ) | 1110 | 1150 |
| 11 | Waluigi | 510 | 550 |
| 12 | Mametchi | 870 | 920 |

## カートパラメータブロックの実データ比較

GetKartParamBlock (0x80090ff0) はkartVariantを正規化してインデックス化:
- variant 0 (NORMAL) → normalizedKartIdx = 0
- variant 2 (ORIGINAL) → normalizedKartIdx = 1  
- variant 3 (ENHANCED) → normalizedKartIdx = 2

**物理パラメータ (+0x00〜+0xEF) は全キャラ・全ccクラスで ORIGINAL(idx=1) と ENHANCED(idx=2) が完全同一。**

検証済み:
- Mario (charId=0): 0x80390118 vs 0x803902F0 → 物理パラメータ部分は同一
- DK (charId=1): 0x8038D808 vs 0x8038D9E0 → 同上
- King Boo (charId=12): 0x8038EC90 vs 0x8038EE68 → 同上

ccClass=1でも同一 (Mario: 0x8038F368 vs 0x8038F540 → 同上)

**ただし +0xF0 (speedTablePtr) 以降のスピードカーブは ORIGINAL と ENHANCED で異なる。**
- ENHANCED: フラットカーブ (全距離閾値で同一速度値) → 加速がほぼ一瞬で完了
- ORIGINAL: 段階的に加速するカーブ (NORMAL より高速だが段階的)
- 詳細: `mkgp2docs/mkgp2_kart_params.md` の「速度カーブテーブル」セクション

## 結論

**ORIGINAL と ENHANCED の差異は 速度差 (speedCap)、称号システム の2点に存在する。**

- 物理パラメータ (+0x00〜+0xEF): 完全同一 (異なるアドレスだが同一データ)
- **speedTablePtr (+0xF0) チェーン構造**: ENHANCED の speedTablePtr は ORIGINAL のスピードテーブルを指し、ORIGINAL は NORMAL を指す。結果として speedCap が異なり、最高速度に差が出る
- **speedCap → 物理速度への影響**: SpeedCurve_UpdateSpeedCap (0x8019ef90) が KartMovement+0x10 (speedCap) を毎フレーム更新 → PhysicsStep が駆動力スケーリングに使用
- ビジュアルエフェクト: 両方とも `variant >= 2` で有効化、差異なし
- 称号: ORIGINAL(2)はキャラ別COIN_MILAGE窓で称号63、ENHANCED(3)はTAスコア条件で称号87
- 動的テスト (Peach 150cc): NORMAL ≈ 276 (138 km/h), ORIGINAL velocity=78.06 / ≈ 281 (140 km/h), ENHANCED velocity=78.89 / ≈ 284 (142 km/h)

ゲーム上は「ORIGINALカートで一定のCOIN_MILAGEを溜めるとENHANCEDにアップグレードされる」仕組みで、
ENHANCEDは最高速度のわずかな向上に加え、追加の称号獲得機会を提供する。

> **以前の誤り (修正履歴)**:
> - 「ORIGINAL/ENHANCEDのパラメータブロックはバイト同一」→ 最初の240バイト (+0x00〜+0xEF) は同一だが、speedTablePtr (+0xF0) が異なる。
> - 「称号システムのみが実動作差」→ 誤り。speedCap の違いにより最高速度にも差がある。
> - 「speedCurve は物理エンジンに影響しない」→ 誤り。SpeedCurve_UpdateSpeedCap 経由で PhysicsStep に影響する。
