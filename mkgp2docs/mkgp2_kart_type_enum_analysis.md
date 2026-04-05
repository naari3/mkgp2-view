# MKGP2 カートタイプ enum 解析

## 結論: "kart variant" (カート選択種類) の単一enum

NORMAL/GOLD/ORIGINAL/ORIGINAL_ENHANCED は **カート選択種類 (kart variant)** を表す単一のenum。
「キャラクタークラス」ではない。キャラクター固有の属性としては存在しない。

### 根拠

1. **同一のグローバル変数**: `DAT_806cf118` (= `g_kartVariant`) がシステム全体で唯一のカート種類格納先
   - 初期値: `0xFFFFFFFF` (-1) at FUN_8009d6cc
   - カード読込時: card_unpack (0x801d43c4) → `FUN_8009bb28(value)` で設定
   - UI選択時: FUN_801d7b50 (kart selection confirm) → `FUN_8009bb28(value)` で設定
   - FUN_8009bb28 は `DAT_806cf118 = param_1; return;` のみ

2. **UI選択フロー** (FUN_801d7b50 @ 0x801d7b50):
   - 選択肢は 0 と 1 の2つ (0=NORMAL側, 1=ORIGINAL側)
   - 選択0 → `FUN_8009bb28(0)` → kartVariant = NORMAL
   - 選択1 + COIN_MILAGE < 閾値2 → `FUN_8009bb28(2)` → kartVariant = ORIGINAL
   - 選択1 + COIN_MILAGE >= 閾値2 → `FUN_8009bb28(3)` → kartVariant = ORIGINAL_ENHANCED

3. **LAST_KART_TAG (カード保存)**: 1ビットのみ
   - card_pack: `*(byte*)(player+0x1b9) & 1` → LAST_KART_TAG
   - card_unpack: LAST_KART_TAG == 0 → kartVariant=0 (NORMAL), != 0 → kartVariant=2 (ORIGINAL)
   - ORIGINAL vs ORIGINAL_ENHANCED の区別はカードに保存されず、COIN_MILAGE から実行時に再判定

4. **GOLD (index=1) は MKGP2 では未使用**:
   - UI選択肢にGOLDは存在しない (0と1の2択のみ)
   - card_unpack は 0 か 2 しか設定しない
   - GOLD用の文字列とパラメータは存在するが到達パスがない

## Enum定義

```
enum KartVariant {
    KART_NORMAL             = 0,  // 標準カート
    KART_GOLD               = 1,  // ゴールドカート (MKGP2未使用)
    KART_ORIGINAL           = 2,  // オリジナルカート (COIN_MILAGE 閾値1以上)
    KART_ORIGINAL_ENHANCED  = 3,  // オリジナル強化版 (COIN_MILAGE 閾値2以上)
};
```

## 格納場所

| 場所 | アドレス | 意味 |
|------|---------|------|
| グローバル | `DAT_806cf118` | 現在のkartVariant |
| CarObject+0x1C | `param_4[7]` | レース中のカートオブジェクトに格納 |
| RaceConfig+0x40 | `configPtr+0x40` | レース設定構造体 (Debug overlay で "kart type" 表示) |
| player+0x1b9 | `*(byte*)(player+0x1b9)` | カード上は1bit (0=NORMAL, 1=ORIGINAL系) |

## ccClass との関係

| 名前 | グローバル | 値域 | 意味 |
|------|-----------|------|------|
| kartVariant | `DAT_806cf118` | 0-3 | カート種類 (NORMAL/GOLD/ORIGINAL/ORIGINAL_ENHANCED) |
| g_ccClass | `g_ccClass` (0x806d12cc) | 0-2 | エンジンクラス (50cc/100cc/150cc) |

これらは**完全に独立した別概念**。DebugOverlay (0x801f4eb0) でも:
- `kart type : %02d : %s` at configPtr+0x40 → PTR_s_NORMAL テーブル
- `class : %02d : %s` at configPtr+0x4C → PTR_s_50cc テーブル

## GetKartParamBlock (0x80090ff0) のパラメータ修正

**既存のdecompilerコメントは誤り**。正しい引数マッピング:

```c
undefined* GetKartParamBlock(int charId, int ccClass, int kartVariant, int ccVariant)
```

| 引数 | 旧名 | 正しい名前 | CallerでのソースABY |
|------|------|-----------|-------------------|
| r3 | charId | charId | CarObject[5] = characterId |
| r4 | kartType | **ccClass** | CarObject[6] = g_ccClass |
| r5 | ccClass | **kartVariant** | CarObject[7] = DAT_806cf118 |
| r6 | ccVariant | ccVariant | -1 (player), >=0 (enemy) |

**修正**: CarObject_Init の引数マッピングに基づく。

```asm
; CarObject_Init entry register assignment:
; r6 = param_7 → stored to CarObject+0x18 [6]
; r7 = param_8 → stored to CarObject+0x1C [7]
;
; Caller (FUN_800a1d80 RACE init):
;   param_7 = g_ccClass     → CarObject[6]
;   param_8 = DAT_806cf118  → CarObject[7]
;
; GetKartParamBlock call in CarObject_Init:
;   r3 = CarObject[5] = charId
;   r4 = CarObject[6] = g_ccClass
;   r5 = CarObject[7] = DAT_806cf118 (kartVariant)
;   r6 = ccVariant
```

### GetKartParamBlock 内部ロジック (ASM検証済み)

```
; r5 (kartVariant) でnormalizedKartIdxを計算:
;   0(NORMAL),1(GOLD) → 0
;   2(ORIGINAL) → 1
;   3(ORIGINAL_ENHANCED) → 2
;
; Player path (r6 < 0):
;   table = g_kartParamPtrTable[charId]    ; r3→charId
;   index = ccClass * 3 + normalizedKartIdx ; r4→ccClass
;   result = table[index]
;
; Enemy path (r6 >= 0):
;   result = g_kartParamCcVariantTable[ccVariant * 3 + ccClass]
```

→ 実際のパラメータテーブル構造は: `paramTable[charId][ccClass * 3 + normalizedKartIdx]`
  - 各キャラクター: 3 ccClass行 x 3 kartVariant列 = 9エントリ (NORMAL/GOLDは共有)

## COIN_MILAGE 閾値テーブル

### ORIGINAL アンロック閾値 (DAT_8039e904)
| CharId | キャラ | 閾値 |
|--------|--------|------|
| 0 | MARIO | 30 |
| 1 | LUIGI | 1470 |
| 2 | WARIO | 1230 |
| 3 | PEACH | 150 |
| 4 | KOOPA | 750 |
| 5 | KINOPIO | 1350 |
| 6 | DONKEY | 270 |
| 7 | YOSHI | 630 |
| 8 | PACMAN | 990 |
| 9 | MS PACMAN | 390 |
| 10 | AKABEI (アカベイ) | 1110 |
| 11 | WALUIGI | 510 |
| 12 | MAMETCHI | 870 |

### ORIGINAL_ENHANCED アンロック閾値 (DAT_8039e938)
| CharId | キャラ | 閾値 |
|--------|--------|------|
| 0 | MARIO | 1590 |
| 1 | LUIGI | 3030 |
| 2 | WARIO | 2790 |
| 3 | PEACH | 1710 |
| 4 | KOOPA | 2310 |
| 5 | KINOPIO | 2910 |
| 6 | DONKEY | 1830 |
| 7 | YOSHI | 2190 |
| 8 | PACMAN | 2550 |
| 9 | MS PACMAN | 1950 |
| 10 | AKABEI (アカベイ) | 2670 |
| 11 | WALUIGI | 2070 |
| 12 | MAMETCHI | 2430 |

## 説明文テーブル (0x8048e9f0) について

8ポインタ (4種 x JP+EN) の文字列テーブル。コードからの直接xrefは0件。
UIリソースシステム経由で間接的に参照される模様。

| Index | JP要旨 | EN |
|-------|--------|-----|
| 0 | 操作のしやすい小型軽めカート | Easy control kart |
| 1 | 加速バランスの優れた中重量カート | Strong acceleration kart |
| 2 | バランスに優れた中重量カート | Balanced kart |
| 3 | (未確認) | (未確認) |

これらはカート選択UIで表示される説明文であり、ゲームプレイロジックには関与しない。

## キャラクターデータテーブル (g_characterDataTable @ 0x80401ce0)

stride=0x2C (44 bytes), 13エントリ (キャラクター数)。

byte@0x20 フィールド: ほぼ全キャラ=0、PEACH(3)のみ=1。
→ kartVariantとは無関係。PEACHの専用フラグ (おそらく女性キャラフラグまたは特殊演出フラグ)。

## DAT_806cf118 参照一覧 (22 xrefs)

| アドレス | 関数 | R/W | 用途 |
|---------|------|-----|------|
| 8009bb28 | FUN_8009bb28 (SetKartVariant) | W | 唯一のsetter |
| 8009dc0c | FUN_8009d6cc (GameInit) | W | 初期値 0xFFFFFFFF |
| 80082bbc | FUN_8008299c (StatusReport) | R | LIVE通信ステータスにパック |
| 800a1fa0 | FUN_800a1d80 (RaceInit) | R | CarObject_Init に渡す |
| 800a3dd8 | FUN_800a3c38 (BattleInit) | R | CarObject_Init に渡す |
| 800a8004 | FUN_800a8004 (TitleCheck) | R | == 3 (ORIGINAL_ENHANCED) でTIME_ATTACK称号判定 |
| 800a85b0 | FUN_800a8538 | R | 称号関連 |
| 800bc26c-800bde70 | 複数の3D演出関数 | R | `< 2` チェック (NORMALかGOLDなら特殊演出無効) |
| 800bf83c | FUN_800bf5b0 | R | 演出関連 |
| 800c2214,800c232c | FUN_800c2124 | R | 演出関連 |
| 801d53f0 | card_pack | R | LAST_KART_TAG書込み |
| 801d44b8 | card_unpack | R | LAST_KART_TAG読込み |
| 801f5bd4 | FUN_801f5b54 | R | UI config struct に格納 |
| 801f5c90,801f5dd4 | FUN_801f5bf4 | R | UI config struct に格納 |
| 8021c238 | FUN_8021c070 | R | リプレイ/結果画面用 |
| 8021d19c | FUN_8021c778 | R | リプレイ関連 |
| 802482c0 | FUN_80248248 | R | その他 |
