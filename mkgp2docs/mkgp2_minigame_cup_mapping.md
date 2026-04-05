# MKGP2 ミニゲーム-カップ マッピング

## 変換ルール

**ミニゲームcourseId = GPカップcourseId + 8**

`FUN_800a0950` (0x800a0950) がエントリポイント:
- `g_courseId` が 9-16 の場合 → `FUN_80124df8` (ミニゲーム初期化) を呼ぶ
- `g_courseId` が 1-8 の場合 → 通常のレース/バトル/タイムアタック初期化

## マッピングテーブル

| GPカップ | GP courseId | MG courseId | ゲーム内正式名 | 初期化関数 |
|---|---|---|---|---|
| マリオ (MR) | 1 | 9 | 巨大スイカを運べ！ | SuikaBall_Init (0x80133398) |
| DK | 2 | 10 | ノコノコパニック！ | NokoNokoChallenge_Init (0x80129938) |
| ワリオ (WC) | 3 | 11 | バックでレース！ | ReverseChallenge_Init (0x800ad910) |
| パックマン (PC) | 4 | 12 | パックチャレンジ | BiribiriLand_Init (0x80134ff0) |
| クッパ (KP) | 5 | 13 | 最終決戦！クッパを倒せ！ | CGameCoin_Init (0x8012fc58) |
| レインボー (RB) | 6 | 14 | ロボマリオと一騎打ち！ | RivalRun_Init (0x80136ea0) |
| ヨッシー (YI) | 7 | 15 | コインコレクター！ | MiniGame_CoinChallenge_Init (0x80214db8) |
| ワルイージ (DN) | 8 | 16 | 鳥カートコンテスト！ | JumpDistanceMode_Init (0x80213210) |

## ミニゲーム概要

### 巨大スイカを運べ！ (courseId=9, マリオカップ)
スイカボールをカートで押して4つのチェックポイントゲートを通過しゴールを目指す。衝突→射出の3ゾーン物理。

### ノコノコパニック！ (courseId=10, DKカップ)
ハンマーを振りながらコース上のノコノコをすべて倒すミニゲーム。

### バックでレース！ (courseId=11, ワリオカップ)
バック走行でバナナを避けながらゴールを目指す。初期配置は180度逆向き。speedCap=110, mue=0.4。

### パックチャレンジ (courseId=12, パックマンカップ)
直線コース上の電気障害物9個を地形段差による自動バンプジャンプで避けながら進む。内部名「ビリビリランド」。

### 最終決戦！クッパを倒せ！ (courseId=13, クッパカップ)
黒い甲羅でクッパに4回当てて撃破する。クッパは炎弾(64スロット)で反撃。撃破後にコイン噴出演出。

### ロボマリオと一騎打ち！ (courseId=14, レインボーカップ)
ロボマリオ (charId=0) と1対1の制限時間付き1周レース。

### コインコレクター！ (courseId=15, ヨッシーカップ)
制限時間内に目標枚数のコインを集めるチャレンジ。speedCap=300, mue=0.4。

### 鳥カートコンテスト！ (courseId=16, ワルイージカップ)
ジャンプ台からの飛距離に応じてコインを獲得。水平距離のみ計測、70mで最大15コイン。speedCap=250, mue=0.4。

## 根拠コード

### MiniGame_CreateByCourseId (0x80123050)
g_courseId で switch して各ミニゲーム初期化関数を呼ぶ。

### MiniGame_GetCourseData (0x80122efc)
courseId < 9 なら `courseId - 1`、courseId >= 9 なら `courseId - 9` でインデックス計算。
つまりミニゲームのコースデータは courseId 9 = index 0 ... courseId 16 = index 7。

### コース名文字列テーブル (0x8040b33c 付近)
courseId 順にコース名ポインタが並ぶ。index 9-16 がチャレンジコース名:
- "MR highway Challenge", "DK jungle Challenge", "WC dcity Challenge",
- "PC land Challenge", "KP castle Challenge", "RB road Challenge",
- "YI land Challenge", "DN stadium Challenge"
