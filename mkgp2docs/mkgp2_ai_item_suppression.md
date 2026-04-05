# MKGP2 AIアイテム使用システム

## 概要

AIカートのアイテム使用は `AI_UpdateItemUseLogic` (0x801ea2fc) で制御される。抑制テーブル、クールダウン、照準判定、確率ロールの4段ゲートを通過して初めてアイテムが使用される。

最も顕著な効果: **最終ラップではリーダーとの周回差が閾値以内の全AIがアイテムを投げなくなる。** 閾値はコースにより50〜155で、通常レースではほぼ全AIが該当する。周回遅れ等で閾値を超えた場合は抑制されない。

## アイテム使用フロー

```
AI_CalcGPTargetSpeed (0x801ea6f0)
  └── AI_UpdateItemUseLogic (0x801ea2fc)
        │
        ├── Step 1: 抑制チェック
        │   AI_CheckItemSuppression (0x801dd344)
        │   → テーブルにマッチしたら return (アイテム使用中止)
        │
        ├── Step 2: クールダウン
        │   kartController+0xF0: 使用間隔タイマー (毎フレーム-1)
        │   → 0以下になるまで待機
        │
        ├── Step 3: アイテム所持確認
        │   AI_HasItem (0x801e6d6c): kartController+0x24 (アイテムポインタ)
        │   + item+0x20 == 1 (アイテム使用可能状態)
        │   → 持っていなければ return
        │
        ├── Step 4: 照準タイマー
        │   kartController+0xEC: 保持タイマー (毎フレーム-1)
        │   → 0以下なら Step 5-6 をスキップして即使用 (アイテム抱え落ち防止)
        │
        ├── Step 5: 照準判定
        │   AI_CheckTargetInRange (0x801dd044)
        │   → アイテム種別ごとの射程・角度条件で相手との位置関係をチェック
        │   → 条件不成立なら kartController+0xF0 = 0x78 (120フレーム冷却)
        │
        ├── Step 6: 確率ロール
        │   AI_RollItemUseProbability (0x801dcf00)
        │   → レース順位に応じた確率でアイテム使用を判定
        │   → 失敗なら冷却タイマーセット
        │
        └── Step 7: アイテム使用実行
            GetCourseSectionType (0x8023d6d4) → コース区間で投擲方向決定
            AI_UseItem (0x801e6ba8) → item->vtable[4] で仮想ディスパッチ
```

## 抑制テーブル (DAT_804e2490)

### エントリ構造 (20 bytes)

| Offset | Type | 説明 |
|---|---|---|
| +0x00 (byte3) | char | gameMode (-1=any) |
| +0x00 (byte2) | char | courseId (-1=any) |
| +0x00 (byte1) | char | ccClass (-1=any) |
| +0x00 (byte0) | char | subMode (-1=any) |
| +0x04 (byte0) | char | kartIdx (-1=any) |
| +0x08 | int | remainingLaps (比較: value-1。-1=any) |
| +0x0C | int | lapDiffMin |
| +0x10 | int | lapDiffMax |

マッチしたら return 0 = **アイテム使用禁止**。マッチしなければ return 1 = 使用可。

### 最終ラップ抑制ルール (全8コース)

| courseId | コース | lapDiff上限 |
|---|---|---|
| 1 | マリオ | 128 |
| 2 | DK | 100 |
| 3 | ワリオ | 128 |
| 4 | パックマン | 155 |
| 5 | クッパ | 110 |
| 6 | レインボー | 50 |
| 7 | ヨシー | 50 |
| 8 | ワルイージ | 111 |

- **kartIdx = -1 (全AIスロット対象)**: 先頭だけでなく全AIが対象
- ccClass = -1, subMode = -1: 全ccクラス・全subMode共通
- lapDiff上限は通常レースで十分大きい (リーダーとの差が上限以内なら抑制)

### subMode別 追加抑制ルール

最終ラップ以外にも、subModeごとに追加の抑制ポイントがある:

| コース | subMode | 残りラップ | lapDiff範囲 | 備考 |
|---|---|---|---|---|
| マリオ | 0 | 5 | 0-128 | 9ラップ中4ラップ目から抑制 |
| マリオ | 1 | 0 | 0-128 | 最終ラップのみ (重複) |
| マリオ | 2 | 5 | 0-128 | 9ラップ中4ラップ目から |
| マリオ | 3 | 3 | 0-128 | |
| DK | 0 | 6 | 0-100 | |
| DK | 1 | 3 | 0-100 | |
| DK | 2 | 3 | 0-167 | |
| DK | 3 | 3 | 0-100 | |
| ワリオ | 0-3 | 3 | 0-128 | 全subMode同一 |
| パックマン | 0-3 | 3 | 0-155 | 全subMode同一 |
| クッパ | 0,2 | 3 | 0-180 | |
| クッパ | 1,3 | 2 | 0-110 | |
| レインボー | 0-3 | 1 | 100-200 | lapDiff 100以上の場合のみ |
| ヨシー | 0,2 | 2 | 50-113 | |
| ヨシー | 1,3 | 1 | 100-173 | |
| ワルイージ | 0-3 | 3 | 0-111 | 全subMode同一 |

### 実効的な抑制開始ラップ

150cc表コースでの抑制開始 (最も早いsubModeの例):

| コース | 総ラップ | 最早抑制 (残りラップ) | 抑制開始ラップ |
|---|---|---|---|
| マリオ | 9 | 残5 (sm0,2) | 4ラップ目 |
| DK | 9 | 残6 (sm0) | 3ラップ目 |
| ワリオ | 6 | 残3 (全sm) | 3ラップ目 |
| パックマン | 4 | 残3 (全sm) | 1ラップ目! |
| クッパ | 5 | 残3 (sm0,2) | 2ラップ目 |
| レインボー | 3 | 残1 (lapDiff 100+) | 条件付き |
| ヨシー | 4 | 残2 (sm0,2) | 2ラップ目 |
| ワルイージ | 5 | 残3 (全sm) | 2ラップ目 |

パックマンカップ (4ラップ) は sm0-3 で残3ラップから抑制 = **1ラップ目からアイテム抑制**。

## 照準判定 (AI_CheckTargetInRange, 0x801dd044)

アイテム種別ごとに射程・角度条件が異なる。テーブル: `DAT_804e27f0` (12 bytes/entry)

| エントリ | 方向 | 射程 | 角度 | 対象アイテム |
|---|---|---|---|---|
| 0 | 全方位 | 無限 | - | (要検証: item+0x1C=0 のアイテム) |
| 1-11 | 前方 | 50.0 | 90° | (要検証: item+0x1C=1-11 のアイテム) |

- 方向: -1=全方位(常にマッチ), 0=前方, 1=後方
- 角度: 自分→相手の方向角と自yawの差が閾値以内か (度→ラジアン変換)
- 距離: FUN_800d3184 (Newton-Raphson逆平方根) で3D距離計算

## 確率ロール (AI_RollItemUseProbability, 0x801dcf00)

テーブル: `DAT_804e2f64` (34エントリ, 16 bytes/entry)

レース順位に応じた確率でアイテム使用を決定。乱数が確率以下なら使用。

### GPモード (gameMode=2) の確率

| 順位 | 確率 |
|---|---|
| 1位 | 70% |
| 2位 | 70% |
| 3位 | 50% |
| 4位 | 20% |

### RACEモード subMode 4 の確率

| 順位 | 確率 |
|---|---|
| 1位 | 100% |
| 2位 | 70% |
| 3位 | 70% |
| 4位 | 40% |
| 5位 | 30% |
| 6位 | 40% |

→ **上位AIほどアイテムを使う確率が高い。** 下位AIは控えめ。

## コースセクション別投擲方向 (GetCourseSectionType, 0x8023d6d4)

コースノードテーブル (`DAT_8041cbad + index * 0x14`) からセクション種別を読む。

| セクション種別 | 投擲先 | 説明 |
|---|---|---|
| 1-4 | 自分のPhysicsState | 前方投擲 (方向依存) |
| 0, 5-8+ | ライバル/プレイヤー位置 | 追尾投擲 (ターゲット指定) |

## クールダウンタイマー

| オフセット | 名前 | 説明 |
|---|---|---|
| kartController+0xF0 | useIntervalTimer | 使用間隔タイマー。毎フレーム-1。0以下で使用可能。照準成功・確率失敗時に 0x78 (120f = 2秒) にリセット |
| kartController+0xEC | targetingTimer | 保持タイマー。毎フレーム-1。0以下で照準・確率判定をスキップして強制使用 (抱え落ち防止) |

## 関数一覧

| Address | Name | 説明 |
|---|---|---|
| 0x801ea2fc | AI_UpdateItemUseLogic | アイテム使用メインロジック |
| 0x801dd344 | AI_CheckItemSuppression | 抑制テーブル検索 |
| 0x801e6d6c | AI_HasItem | アイテム所持+使用可能判定 |
| 0x801dd044 | AI_CheckTargetInRange | 射程・角度条件判定 |
| 0x801dcf00 | AI_RollItemUseProbability | 順位別確率判定 |
| 0x801e6ba8 | AI_UseItem | アイテム使用実行 (vtable dispatch) |
| 0x8023d6d4 | GetCourseSectionType | コースセクション種別取得 |
| 0x801e6d30 | AI_GetYaw | AI yaw角取得 (PhysicsState+0x7C) |
| 0x801e6da0 | AI_GetItemType | アイテム種別取得 (item+0x1C) |

## グローバル変数

| Address | Name | 説明 |
|---|---|---|
| 0x804e2490 | g_itemSuppressionTable | 抑制ルールテーブル (40エントリ × 20bytes) |
| 0x806cfefc | g_itemSuppressionCount | テーブルエントリ数 (ランタイム設定) |
| 0x804e27f0 | g_itemTargetingTable | 射程・角度テーブル (12 bytes/entry) |
| 0x804e2f64 | g_itemUseProbTable | 順位別使用確率テーブル (34エントリ, 16 bytes/entry) |
| 0x806da2ac | FLOAT_deg2rad | PI/180 (度→ラジアン変換定数) |
