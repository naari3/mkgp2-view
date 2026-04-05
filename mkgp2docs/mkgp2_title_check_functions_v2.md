# MKGP2 称号チェック関数テーブル解析 (v2: 実機メモリダンプ修正版)

## 概要

`determine_title` (0x800a70fc) がモードに応じたチェック関数テーブルを順にイテレーションし、
最初にマッチした関数が `DAT_806d13d4` (staging TitleID) と `DAT_806d13d8` (param) を書き込む。
テーブルは先頭(index 0)が最優先、末尾が最低優先。

**重要: SDA (Small Data Area) ベースアドレス**
`r13` レジスタは常に **`0x806d6d20`** を指しており、多くのグローバル変数はこのアドレスからのオフセットでアクセスされる。
(例: `g_playCount` at `0x806cf200` = `-0x7b20(r13)`)

**重要: Fixed テーブルの TitleID オフセット (+0x2D)**

determine_title の Fixed テーブルループ内で、check 関数が返す raw TitleID に **+0x2D (45)** が加算される
(PPC: `addi r5, r3, 0x2d` at 0x800a7394)。Extra 関数の TitleID にはこのオフセットは適用されない。

- **Extra 関数**: 直接 TitleID = raw 値 (TitleID 1-45: GP制覇系)
- **Fixed テーブル**: 最終 TitleID = raw 値 + 45

以下の各関数説明の TitleID は **最終値 (raw + 45 適用済み)** で記載している。

**重要: 同一称号の重複回避メカニズム**

Fixed テーブルループで check 関数がマッチしても、算出された TitleID が `g_currentTitleId` と
同じ場合はスキップされ、ループが次のエントリに進む（例外: TitleID 0x42=66 のみ重複許可）。

## テーブル構成

| モード | テーブルアドレス | エントリ数 | Extra関数 |
|--------|-----------------|-----------|-----------|
| Race/GP (0) | 0x80411c48 | 16+ (後半重複あり) | FUN_800a7600 |
| Battle (1) | 0x80411cc8 | 16 | FUN_800a7548 |
| TimeAttack (2) | 0x80411d08 | 40 | なし |

## グローバル変数の意味 (Ghidra解析による確定版)

**SDA Base (r13): 0x806d6d20**

| アドレス | SDAオフセット | シンボル名 | 旧推定名 | 用途 |
|:---|:---|:---|:---|:---|
| 0x806cf200 | -0x7b20(r13) | `g_playCount` | play_count | プレイ回数 (0=初回) |
| 0x806cf1fc | -0x7b24(r13) | `g_technicalTitleGrade` | g_technicalTitleGrade | コイン回収状況評価 (0=全回収/1=未回収/2=残存数不変) [詳細](./mkgp2_analysis_technical_flag.md) |
| 0x806d12d0 | -0x5a50(r13) | `g_selectedFrameIdx` | selected_frame_idx | フレーム選択インデックス |
| 0x806cf1f0 | -0x7b30(r13) | `g_lossCount` | loss_count | 敗北数 (0=無敗) |
| 0x806d1294 | -0x5a8c(r13) | `g_gameMode` | game_mode | 0=Race, 1=Battle, 2=TA |
| 0x806cf108 | -0x7c18(r13) | `g_courseId` | course_id | コースID (0-7) |
| 0x806d1268 | -0x5ab8(r13) | `g_isUraCourse` | (New) | 裏コースフラグ (1=裏) |
| 0x806d143c | -0x58e4(r13) | `g_battleWins` | battle_wins | バトル勝利数 |
| 0x806d13fc | -0x5924(r13) | `g_undefeatedStreak` | undefeated_streak | 無敗連続数 |
| 0x806d1444 | -0x58dc(r13) | `g_itemHitCount` | item_hit_count | アイテム被弾数 |
| 0x806d1430 | -0x58f0(r13) | `g_prevRaceScore` | prev_race_score | 前回レーススコア |
| 0x806d13e0 | -0x5940(r13) | `g_totalScore` | total_score | 累計スコア |
| 0x806cf214 | -0x7b0c(r13) | `g_isWinner` | is_winner | 勝者フラグ |
| 0x806cf218 | -0x7b08(r13) | `g_isPerfect` | is_perfect | パーフェクトフラグ |
| 0x806cf210 | -0x7b10(r13) | `g_totalCoins` | total_coins | 累計コイン枚数 |
| 0x806d1410 | -0x5910(r13) | `g_gpWinCount` | gp_win_count | GP勝利数 |
| 0x806d1414 | -0x590c(r13) | `g_gpLossCount` | gp_loss_count | GP敗北数 |
| 0x806d141c | -0x5904(r13) | `g_ghostDefeatedCount` | ghost_defeated_count | ゴースト撃破数 |
| 0x806d1418 | -0x5908(r13) | `g_specificWinCount` | specific_win_count | 特定条件勝利数 |
| 0x806d13e4 | -0x593c(r13) | `g_totalRaces` | total_races | 累計レース数 |
| 0x806d13f4 | -0x592c(r13) | `g_totalDistance` | total_distance | 累計走行距離 |
| 0x806d13f8 | -0x5928(r13) | `g_taWins` | ta_wins | TA勝利数 |
| 0x806d1400 | -0x5920(r13) | `g_taLosses` | ta_losses | TA敗北数 |
| 0x806d13e8 | -0x5938(r13) | `g_allCoursesCleared` | all_courses_cleared | 全コースクリアフラグ |
| 0x806d1438 | -0x58e8(r13) | `g_perfectCount` | perfect_count | パーフェクト回数 |
| 0x806d1440 | -0x58e0(r13) | `g_consecutiveWins` | consecutive_wins | 連勝数 |
| 0x806d1454 | -0x58cc(r13) | `g_specialWinCount` | special_win_count | 特殊勝利数 |
| 0x806d13d4 | -0x594c(r13) | `g_stagingTitleId` | - | 判定結果 TitleID |
| 0x806d13d8 | -0x5948(r13) | `g_titleParam` | - | 判定結果 Param |

---

## Race/GP テーブル優先順位 (Index 0 が最優先)

※実機メモリダンプ (0x80411c48) に基づく正しい順序

| Idx | 関数 | 最終TitleID | 称号名 | 備考 |
|:---:|:---|:---:|:---|:---|
| 0 | FUN_800a7cc8 | 153 | さまよえる | 初回プレイ |
| 1 | FUN_800a84c0 | 109-112 | 比類なき～ | バトル勝利数 |
| 2 | FUN_800a8438 | 116-117 | 不運な/ミラクル | 60秒以内 |
| 3 | FUN_800a83a4 | 119 | 衝撃的な | 特殊勝利数 |
| 4 | FUN_800a834c | 121 | 猛特訓中の | 2回目プレイ |
| 5 | FUN_800a82f4 | 123 | 正義の | 初勝利 |
| 6 | FUN_800a8298 | 125 | 技巧派 | TA未プレイ |
| 7 | FUN_800a8234 | 127 | 豪快な | GP10勝+無敗 |
| 8 | FUN_800a80cc | 129 | 百発百中 | 高スコア |
| 9 | FUN_800a805c | 131 | 進化する | バトル勝利+勝者 |
| 10 | FUN_800a7f9c | 134 | 傷だらけの | アイテムヒット7回 |
| **11** | FUN_800a7e38 | **136** | **超一流の** | `g_technicalTitleGrade == 0` |
| **12** | FUN_800a7dd4 | **138** | **噂の** | `g_technicalTitleGrade == 2` |
| 13 | FUN_800a7d20 | 140-149 | (フレーム選択系) | フレーム選択 |
| 14 | FUN_800a7c6c | 154 | サンデー | 日曜+バトル3勝 |
| 15 | FUN_800a7a7c | 171-186 | (コース系) | コース/表裏別 |
| 16 | FUN_800a8538 | 105-108 | 走行距離 | (※推定位置) |
| ... | ... | ... | ... | ... |
| **21** | FUN_800a7e0c | **137** | **パーフェクト** | `g_technicalTitleGrade == 1` |
| 22 | FUN_800a7d5c | 139 | 紙一重の | 全参加者順位 |

※ Index 23 以降は他モード用の関数や重複エントリが含まれており、Raceモードでの到達性は低いと考えられる。

---

## チェック関数詳細 (抜粋・修正)

### FUN_800a7e38 (Race[11]) — 超一流の
- **TitleID**: 136 (raw 91) = 超一流の / Top
- **条件**: `g_technicalTitleGrade == 0`
- **詳細**: レース中の特定チェックポイント到達時にフラグが 0 にセットされる。

### FUN_800a7dd4 (Race[12]) — 噂の
- **TitleID**: 138 (raw 93) = 噂の / Rumored
- **条件**: `g_technicalTitleGrade == 2` かつ `g_lossCount == 0` (無敗)
- **詳細**: 特殊演出（コイン等）を伴うサブ関数 `RankLog_UpdateTechnicalFlag` を通過した際にフラグが 2 にセットされる。

### FUN_800a7e0c (Race[21]) — パーフェクト
- **TitleID**: 137 (raw 92) = パーフェクト / Perfect
- **条件**: `g_technicalTitleGrade == 1`
- **詳細**: 走行状態と発生エフェクトが一致する（完璧な操作）等の条件でフラグが 1 にセットされる。
- **注意**: 「超一流」「噂の」より優先順位がかなり低い（インデックス21）。

### FUN_800a7a7c (Race[15]) — コース別称号
- **TitleID**: 171-186 (raw 126-141)
- **条件**: `g_lossCount == 0` (無敗)
- **詳細**: `g_courseId` と `g_isUraCourse` (0=表, 1=裏) に基づいて算出。
  - 計算式: `(mapped_course_index * 2) + g_isUraCourse + 0x7E (126) + 45`
  - コース0表=171(高速コースの), コース0裏=172(遊園地の)
  - コース1表=173(ハイウェイの), コース1裏=174(海岸線の)
  - ... (以下略)
- **備考**: `g_isUraCourse` はアドレス `0x806d1268` (`-0x5ab8(r13)`) に位置する。

### FUN_800a7d20 (Race[13]) — フレーム選択テーブル参照
- **TitleID**: 動的 (テーブル `0x80326ad8[tier]` + 45)
- **条件**: `g_selectedFrameIdx > 0` のみ (選択値バリデーションなし)
- **アセンブリ解析** (0x800a7d20-0x800a7d58, leaf function):
  ```c
  bool CheckSelectedFrame() {
      int frameIdx = g_selectedFrameIdx;
      if (frameIdx <= 0) return false;
      g_titleParam = 0;
      g_stagingTitleId = DAT_80326ad8[frameIdx];
      return true;
  }
  ```
- **詳細**:
  `g_selectedFrameIdx` (0x806d12d0) はフレーム選択メニューで選択されたフレーム番号を保持する。
  テーブル `0x80326ad8` は全13エントリ (tier 0-12)。

| フレーム | TitleID (raw+45) | 称号名 | 選択UI |
|:---:|:---:|:---|:---:|
| 0 | 50 | (デフォルト) | vis 0 |
| 1 | 141 | チャンバラ | vis 1 |
| 2 | 148 | 極悪 | vis 10 |
| 3 | 140 | ダンシング | vis 4 |
| 4 | 142 | 艶やかな | **不可** |
| 5 | 144 | 大海原の | vis 9 |
| 6 | 146 | 戦国の | **不可** |
| **7** | **145** | **かわいい** | **vis 5** |
| 8 | 143 | 萌え系 | vis 2 |
| 9 | 147 | マジカル | vis 8 |
| 10 | 149 | 危機一髪 | vis 6 |
| 11 | 52 | (不明) | vis 3 |
| 12 | 46 | (不明) | vis 7 |

- **備考**: フレーム 4 (艶やかな) と 6 (戦国の) は UI 上で選択不可 (ビジュアル位置 11,12 = 選択範囲外)。
  フレーム 11, 12 は選択可能だが称号名は未確認。
- **重要: g_selectedFrameIdx はカードからロードされない**:
  XREF解析(WRITE箇所)の結果、`g_selectedFrameIdx` に値を書き込むのは以下の3箇所のみ:
  1. FUN_8009d07c (init): `g_selectedFrameIdx = 0`
  2. FUN_8009d6cc (init2): `g_selectedFrameIdx = 0` (その後カード読み込みを呼ぶ)
  3. FUN_801c53a0 (メニューUI): `g_selectedFrameIdx = *(param_1 + 4)` (フレーム選択UIでのボタン操作時のみ)

  カードの TITLE-PARAM フィールドはフレーム番号を保存しているが、card_unpack (0x801d43c4) は
  これを称号デコード用のパラメータとして TitleDecode() に渡すだけで、
  `g_selectedFrameIdx` には書き込まない。ゲーム起動後、プレイヤーがフレーム選択メニューで
  明示的にフレーム7を選択しない限り、値は0のままで「かわいい」の条件 `g_selectedFrameIdx > 0` は成立しない。
- **フレーム選択UIの詳細**: `mkgp2_frame_selection_ui.md` 参照

---

## progressionScore の計算 (DetermineTitle 内)

### 概要
DetermineTitle (0x800a70fc) が TitleID 判定前に progressionScore を計算し、
DetermineNameId (FUN_800a8a28) に渡す。

### 計算式
```
progressionScore = ghost_clears + course_clears_omote*4 + course_clears_ura*4 + g_playerWinsCurrent
```

### ghost_clears (FUN_801d5b8c)
- ループ: ccClass=0..1 (50cc, 100cc のみ), course=1..8, ghostIdx=0..7
- 各ゴースト撃破: +1
- 最大: 2 × 8 × 8 = 128

### course_clears (FUN_801d59f8)
- ループ: course=1..8, k=0 (表), k=1 (裏)
- 各通過: +4
- **FUN_801d59f8 は 150cc のバイト値のみを参照** (オフセット 0x1ca 固定)
- 表: バイト値 ≥ 4 で通過
- 裏: バイト値 ≥ 9 で通過

### player 構造体のコースクリアバイト配列

| オフセット | CC クラス | 内容 |
|:---|:---|:---|
| 0x1ba..0x1c1 | 50cc (ccClass=0) | byte[8], 各コースの進行度 |
| 0x1c2..0x1c9 | 100cc (ccClass=1) | byte[8], 各コースの進行度 |
| 0x1ca..0x1d1 | 150cc (ccClass=2) | byte[8], 各コースの進行度 |

計算式: `player + 0x1ba + ccClass * 8 + courseIdx` (courseIdx = 0..7)

### 二重カウント問題

card_unpack (0x801d43c4) は RESULT-GP の 4bit ニブルを `local_44[3] & 0xf` で取り出し、
**そのまま** バイト値として格納する:
```c
*(byte *)(puVar14 + 0xdd) = local_44[3] & 0xf;
// puVar14 + 0xdd = player のバイトオフセット 0x1ba (0xdd * sizeof(undefined2))
```

RESULT-GP-150-XX = 0xF (全4ラウンドクリア) の場合:
- バイト値 = 15
- 表チェック: 15 ≥ 4 → **通過** (+4)
- 裏チェック: 15 ≥ 9 → **通過** (+4)
- 1カップあたり **+8** (二重カウント)

RESULT-GP-150-XX = 0x7 (3ラウンドクリア相当) の場合:
- バイト値 = 7
- 表チェック: 7 ≥ 4 → **通過** (+4)
- 裏チェック: 7 < 9 → **不通過**
- 1カップあたり **+4** (正常)

バイト値の閾値と対応する意味:

| 閾値 | 意味 | 使用関数 |
|:---:|:---|:---|
| ≥ 1..4 | ラウンド1~4クリア (ゴースト0~3撃破) | FUN_801d5b8c (ghostIdx 0-3) |
| ≥ 4 | 表クリア | FUN_801d59f8 (k=0) |
| ≥ 5 | 表完全制覇 | FUN_801d5c0c (param_4=0) |
| ≥ 6..9 | ゴースト4~7撃破 | FUN_801d5b8c (ghostIdx 4-7) |
| ≥ 9 | 裏クリア | FUN_801d59f8 (k=1) |
| ≥ 10 | 裏完全制覇 | FUN_801d5c0c (param_4≠0) |

---

## NameID 決定ロジック (determine_name_id / FUN_800a8a28)

### 概要
DetermineTitle から TitleID 決定**前**に呼ばれ、プレイヤーの進行度スコアに基づいて
NameID (称号の名詞部分: ルーキー、レーサー、キング等) を決定する。

### 判定優先順位

#### 1. 裏全クリア + 高スコア → 最強伝説
- **条件**: 裏クリア数 == 8 AND score >= 150
- **NameID**: 37 (最強伝説 / Legendary)

#### 2. 表全クリア → マリカ鬼神 [バグあり]
- **条件**: 表クリア数 == 8
- **NameID**: score ≤ 128 → 36 (マリカ鬼神 / MK Lord)
- **バグ**: 閾値チェック順序の問題で、NameID 29-35 (獅子王～マリカ魔王) は到達不可能。

#### 3. メイン本体: スコアベース判定
- ≥ 96: 28 (韋駄天)
- 92-95: 27 (暴走列車)
- ... (以下略) ...
- 0-3: 0 (ルーキー) / 1 (アタッカー)

### 性別分岐
character_id が 3 または 9 の場合（女性キャラ）、3つのスコア帯で名詞が変わる:
- 48-51: ヒーロー → ヒロイン
- 52-55: プリンス → プリンセス
- 56-59: キング → クイーン
