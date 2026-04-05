# MKGP2 周回ボーナス (AI_CalcLapSpeedBonus)

## 概要

AIカートの速度に**レースの残りラップ数**に応じたボーナスを加算するラバーバンド機構。レース終盤ほどAIが速くなる。

**以前「順位ボーナス」と誤認していたが、正しくは周回ボーナス。** 第2引数は `totalLaps - currentLap`(残りラップ数)であり、レース順位(1-6位)ではない。順位(PhysicsState+0x23c)は第4引数 `excludePosition` として、特定順位のAIをルールから除外する目的にのみ使われる。

## 関数シグネチャ

```c
// 0x801dd480
double AI_CalcLapSpeedBonus(
    int kartIdx,        // AIスロット (0-7)
    char remainingLaps, // totalLaps - currentLap
    int lapDifference,  // リーダーとの周回差 (FUN_801e6c94経由)
    char excludePosition // レース順位 (除外条件。PhysicsState+0x23c)
);
```

呼び出し元 (AI_CalcGPTargetSpeed, 0x801ea6f0):
```c
uVar7 = *(PhysicsState + 0x23c);  // 順位 (1-based)
uVar9 = *(PhysicsState + 0x240);  // 現在のラップ
iVar3 = FUN_801e6c94(iVar8);      // リーダーとのラップ差
AI_CalcLapSpeedBonus(kartIdx, DAT_806d1348 - uVar9, iVar3, uVar7);
```

## コース別 総ラップ数テーブル

DAT_806d1348 は RaceInit でコース設定テーブル (0x8040E7D0) からロードされる。

| courseId | コース | 50cc表 | 50cc裏 | 100cc表 | 100cc裏 | 150cc表 | 150cc裏 |
|---|---|---|---|---|---|---|---|
| 1 | マリオ | 6 | 4 | 6 | 4 | 9 | 6 |
| 2 | DK | 8 | 4 | 8 | 4 | 9 | 6 |
| 3 | ワリオ | 4 | 4 | 4 | 4 | 6 | 5 |
| 4 | パックマン | 3 | 3 | 3 | 3 | 4 | 4 |
| 5 | クッパ | 4 | 4 | 4 | 4 | 5 | 5 |
| 6 | レインボー | 2 | 2 | 2 | 2 | 3 | 3 |
| 7 | ヨシー | 3 | 2 | 3 | 2 | 4 | 3 |
| 8 | ワルイージ | 4 | 4 | 4 | 4 | 5 | 5 |

DAT_806d1340 = 6 (固定、レーサー総数) とは別の変数。

## ルールテーブル検索

RACEモードの場合、2段階のテーブル検索:

### 1. コース別テーブル (PTR_DAT_803bcbac)

ポインタ配列: `[courseIdNorm - 1 + tableVariant * 8]`

**Variant 0 (標準): 全8コース共通で1エントリのみ**

| ccClass | subMode | kartIdx | 残りラップ | lapDiff範囲 | 除外順位 | ボーナス |
|---|---|---|---|---|---|---|
| 150cc | * | * | 0 (最終ラップ) | 0-50 | なし | **+60.0** |

→ 150ccの最終ラップで、リーダーとの差が50以内なら +60.0

**Variant 1 (高難易度?): 共通+60に加え、序盤のペナルティルール**

各コースごとに subMode 別のペナルティ:
- 最初のラップ (remaining = totalLaps-1) で lapDiff が一定以上 → **-15.0 〜 -30.0**
- ペナルティの強度と閾値はコース長に依存

### 2. フォールバックテーブル (PTR_DAT_803bcc0c)

ポインタ配列: `[subMode + (courseIdNorm - 1) * 8 + tableVariant * 0x40]`

コース × subMode × variant ごとに独立したテーブル。各テーブルは25-35エントリ。

### ルールエントリ構造 (0x14 bytes)

| Offset | Type | 説明 |
|---|---|---|
| +0x00 | char | ccClass (-1=any, 0=50cc, 1=100cc, 2=150cc) |
| +0x01 | char | subMode (-1=any) |
| +0x02 | char | kartIdx (-1=any, 0-4=特定スロット) |
| +0x03 | char | remainingLaps (比較: value-1。-1=any) |
| +0x04 | int | lapDiffMin |
| +0x08 | int | lapDiffMax |
| +0x0C | char | excludePosition (除外順位。value-1で比較。-1=none) |
| +0x10 | float | bonus (速度加算値) |

終端マーカー: [0] = 0x9C (-100)

## フォールバックルールの構造 (マリオカップ 150cc subMode 0 の例)

複数のレイヤーで構成される:

### Layer A: 残りラップ基本ボーナス (150cc固定)

| 残りラップ | lapDiff範囲 | ボーナス | 説明 |
|---|---|---|---|
| 1 | 0-128 | +20.0 | レース終盤 |
| 2 | 0-128 | +10.0 | |
| 3 | 0-128 | +10.0 | |
| 8 (序盤) | 50-128 | -15.0 | 序盤で大差ならペナルティ |

### Layer B: 最終ラップ スロット別ボーナス (cc不問)

最終ラップ (remaining=0) で、特定順位を除外してボーナス:

| 除外順位 | ボーナス | 意味 |
|---|---|---|
| 1位以外 | +30.0 | 1位のAI以外に +30 |
| 2位以外 | +30.0 | 2位のAI以外に +30 |
| 3位以外 | +25.0 | 3位のAI以外に +25 |
| 4位以外 | +20.0 | 4位のAI以外に +20 |

→ ルールは上から順にマッチするため、実質的にAIの順位ごとに最終ラップボーナスが異なる。

### Layer C: 距離+順位条件ボーナス

特定の lapDiff 範囲で、順位除外付きの追加ボーナス:
- lapDiff 20-35, 5位除外: +0 / lapDiff 20-35, 一般: +20
- lapDiff 60-75, 一般: +20

### Layer D: 中盤ラップの順位別ペナルティ

残りラップ数が多いほど、下位AIにペナルティ:
- remaining=2: 5位は -10
- remaining=3: 4位は -10, 5位は -20
- remaining=4+: 3-5位に -10 〜 -75 の累進ペナルティ

## subMode 別の難易度ティア

subMode 0-7 で難易度が異なる (偶数=表コース系、奇数=裏コース系):

| subMode | lapDiffMax | スロット別ボーナス | 特徴 |
|---|---|---|---|
| 0 (表/易) | 短い (128) | k0=+30, k1=+30, k2=+25, k3=+20 | 控えめ |
| 1 (裏/易) | 長い (180) | k1=+30, k2=+25, k3=+10 | 3スロットのみ |
| 2 (表/中) | 短い | k1=+50, k2=+40, k3=+35, k4=+20 | 攻撃的 |
| 3 (裏/中) | 長い | k1=+40, k2=+35, k3=+20 | |
| 4 (表/高) | 短い | k1=+50, k2=+40+ | 最多エントリ、段階的 |
| 5 (裏/高) | 長い | k1=+40, k2=+35, k3=+20 | 二重距離窓 |
| 6 (表/最高) | 短い | k1=+50, k2=+40, 序盤+10 | 序盤でもボーナス |
| 7 (裏/最高) | 長い | k1=+40, k2=+35, k3=+20 | 二重距離窓 |

## マッチング優先度

テーブルは上から順にスキャン、最初にマッチしたルールが適用される:
1. コース別テーブル (variant=tableVariant のコース固有ルール)
2. マッチしなければフォールバック (subMode × course × variant のルール)
3. どちらもマッチしなければ **ボーナス = 0.0** (FLOAT_806da2a4)

## tableVariant の決定

```c
tableVariant = DAT_806d18b8;  // ゲーム進行状況で変化
if (DAT_806d12c0 < 1) {
    tableVariant = 0;          // デフォルト
}
```

variant 0 が通常のルール、variant 1 が高難易度 (序盤ペナルティ追加)。

## コース設定テーブル (0x8040E7D0)

各コース × cc × ura ごとに 0x0C バイトの設定エントリ:

| Offset | Type | 変数 | 説明 |
|---|---|---|---|
| +0x00 | char | DAT_806d1348 | 総ラップ数 |
| +0x04 | float | FLOAT_806d132c | (未調査) |
| +0x08 | float | FLOAT_806d1330 | (未調査) |

インデックス: `courseId * 0x48 + ccClass * 0x18 + isUraCourse * 0x0C`

## 関数一覧

| Address | Name | 説明 |
|---|---|---|
| 0x801dd480 | AI_CalcLapSpeedBonus | 周回ボーナス計算 |
| 0x801ea6f0 | AI_CalcGPTargetSpeed | GP AI目標速度 (呼び出し元) |
| 0x801e93ac | AI_CalcVSTargetSpeed | VS AI目標速度 (呼び出し元) |
| 0x801e6c94 | AI_GetLapDifference | ラップ差取得ラッパー |

## グローバル変数

| Address | Name | 説明 |
|---|---|---|
| 0x806d1348 | g_totalLaps | 現レースの総ラップ数 |
| 0x806d1340 | g_totalRacers | レーサー総数 (常に6) |
| 0x806da2a4 | FLOAT_defaultBonus | デフォルトボーナス (0.0) |
| 0x806d18b8 | g_tableVariant | ルールテーブルバリアント |

## ルールテーブルポインタ

| Address | 説明 |
|---|---|
| 0x803bcbac | コース別テーブルポインタ配列 [course*variant] |
| 0x803bcc0c | フォールバックテーブルポインタ配列 [subMode + course*8 + variant*0x40] |

## 以前の誤認について

前セッションで「順位ボーナス: 1位+60, 2位+20, 3-4位+10, 5位+0」と報告していたが、これは
第2引数 `DAT_806d1348 - PhysicsState+0x240` を「レーサー数 - 順位」と誤解したもの。
実際は「総ラップ数 - 現在ラップ = 残りラップ数」であり、ボーナスはレース進行度に基づく。
