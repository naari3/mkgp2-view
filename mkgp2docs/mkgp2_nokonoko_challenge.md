# ノコノコチャレンジ (courseId=10) 解析

## 概要

courseId=10 のミニゲーム。内部名は "TimeTrial" だが、実態は**ハンマーを振りながらコース上のノコノコを全滅させる**チャレンジ。
制限時間内に全てのノコノコを倒せば成功、タイマーが0になれば失敗。

## ゲームフロー

```
NokoNokoChallenge_Init (0x80129938)
  ├── カート初期化 (ccVariant=1: TT用物理パラメータ)
  ├── NokoNoko_Init: 配置テーブルからノコノコ生成 (デフォルト15体)
  ├── MiniGame_GetCourseData → 速度乗数 + 目標討伐数
  ├── MiniGame_GetTimerValue → 制限時間 (FLOAT_806d132c)
  └── DKコインマネージャ初期化 (6スロット)

NokoNokoChallenge_Update (0x801289b8) 毎フレーム:
  ├── プレイヤー入力処理 + カート物理
  ├── 衝突判定 ×2 (カート前方 + ハンマー)
  ├── DKコイン生成 (ノコノコ撃破時)
  ├── NokoNoko_UpdateAll: 全ノコノコAI更新
  ├── 討伐数 vs 目標数: 全滅→成功
  ├── タイマー減算 (1/60秒/frame): 0到達→失敗
  ├── 自動操舵 (最寄りノコノコ方向へ補助)
  └── リザルト遷移
```

## ハンマーメカニクス

ハンマーは独立したアイテムシステムではなく、**カート物理更新の一環として衝突判定**が行われる。

### 衝突判定の流れ (NokoNokoChallenge_Update内)

毎フレーム2箇所の衝突チェック:

1. **カート前方衝突** (半径 12.0)
   - `CarObject_GetKartMovement` → `FUN_80041748` (変換行列) → 列3 (position)
   - `NokoNoko_CheckHitAndKill(12.0, kartFrontPos, &hitPos)`

2. **ハンマージョイント衝突** (半径 8.0)
   - `item_center_joint` を名前検索でジョイントノードを取得
   - ジョイントの変換行列から列3 (position) を取得
   - `NokoNoko_CheckHitAndKill(8.0, hammerPos, &hitPos)`

### ハンマー入力

ハンマー振りの入力トリガーは以下:
- `FUN_8009c1a8()` = レース開始済み判定 (DAT_806d1260)
- `FUN_8003f5e8(kartMovement, 1)` = アイテム使用可能チェック (KartMovement+0x1F0 != -1, 0x17, 0x1C)
- `FUN_8003f77c(kartMovement, 0, 0)` = アイテム使用実行 (KartMovement+0x290に使用タイマー設定)

ハンマーは使用後にKartMovement+0x290 = -1 でリセットされ、再使用可能になる。

### 衝突判定関数: NokoNoko_CheckHitAndKill (0x801888b8)

```c
// param_1 = 衝突半径, param_2 = 中心位置(xyz), param_3 = ヒット位置出力
float* NokoNoko_CheckHitAndKill(double radius, float* centerPos, float* outHitPos)
{
    // グローバル状態更新: プレイヤー位置 & ハンマー半径記録
    DAT_80677ed8 = centerPos[0];  // g_hammerPosX
    DAT_80677edc = centerPos[1];  // g_hammerPosY
    DAT_80677ee0 = centerPos[2];  // g_hammerPosZ
    FLOAT_806d168c = radius;       // g_hammerRadius

    for (noko in linked_list) {
        if (noko.state < 4) {  // 生存中
            dist = distance(centerPos, noko.pos + (0, 8.5, 0));
            if (dist < radius + 8.5) {
                // ヒット! → 押し出し + 撃破
                noko.pos = pushAway(centerPos, noko.pos, radius + 8.5);
                PlaySE(0xcb);            // 撃破SE
                SpawnEffect(2, hitPos);  // 撃破エフェクト
                noko.state = 4;          // DYING
                noko.animState = 3;      // damage model
                noko.moveSpeed = 0;
                noko.turnSpeed = 0;
            }
        }
    }
}
```

## ノコノコシステム

### エンティティ構造体 (0x40 = 64バイト)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| 0x00 | void* | nextPtr | リンクリスト次要素 |
| 0x04 | int | pathIndex | パステーブルインデックス (0-6) |
| 0x08 | int | waypointIndex | 現在のウェイポイントインデックス |
| 0x0C | int | state | 行動ステート (下記参照) |
| 0x10 | int | animState | アニメーションモデル選択 |
| 0x14 | float | animTimer | アニメーション経過時間 (0〜60でラップ) |
| 0x18 | int | waypointCountdown | ウェイポイント残フレーム (-1=即次へ) |
| 0x1C | float | posX | X座標 |
| 0x20 | float | posY | Y座標 |
| 0x24 | float | posZ | Z座標 |
| 0x28 | float | yaw | 向き (ラジアン、0〜2π) |
| 0x2C | float | moveSpeed | 移動速度 |
| 0x30 | float | turnSpeed | 旋回速度 |
| 0x34 | int | prevAnimState | ブレンド元アニメ状態 (-1=なし) |
| 0x38 | float | blendFactor | アニメブレンド係数 (0→1) |
| 0x3C | float | prevAnimTimer | ブレンド元アニメ時刻 |

### state 値

| 値 | 名前 | 説明 |
|----|------|------|
| 0 | TAMB | タンバリン/待機 |
| 1 | STUNNED | 衝突で停止 (0x78=120フレーム) |
| 2 | WALK | 通常歩行 |
| 3 | RUN | 走行 |
| 4 | DYING | 撃破演出中 (飛び上がって落下) |
| 5 | DEAD | 完全消滅 |

### animState → モデル対応

| 値 | モデル | ファイル |
|----|--------|----------|
| 0 | タンバリン | nokonoko_tamb.dat |
| 1 | (stunned/停止) | nokonoko_tamb.dat (同じ) |
| 2 | 歩行/移動 | nokonoko_work.dat |
| 3 | ダメージ | nokonoko_damage.dat |

`nokonoko_run.dat` (DAT_80677ed0) はロードされるが、animState=2のwork.datが走行にも使用される。

### AI行動パターン (NokoNoko_Update 0x80187d2c)

1. **パス追従**: パステーブル (PTR_DAT_80495d28) に従ってウェイポイントを巡回
   - 各ウェイポイント: {state, animState, countdown, moveSpeed, turnSpeed}
   - countdown == 0 → 次ウェイポイントへ遷移
   - ウェイポイント列の終端 (state==-1) → index 0 に戻る (ループ)
2. **移動計算**: `pos += speedMultiplier * moveSpeed * (sin(yaw), cos(yaw))` + `yaw += turnSpeed`
3. **ノコノコ同士の衝突**: 距離 < 20.0 かつ進行方向dot > 0.4 → stunned (120F停止)
4. **プレイヤーとの衝突** (ハンマー範囲内): state=2 or 3 のとき → stunned (同上)
5. **コース壁衝突**: 4面のAABBチェック → stunned
6. **DYING演出** (state==4): sin/cos方向に飛び散り + Y軸落下 (落下速度 0.05/frame)

### 配置テーブル (DAT_80495c28)

15体のノコノコ + ターミネータ (flag=2)。各エントリ 16バイト:
```
struct NokoNokoSpawn {
    float x;      // X座標
    float y;      // Y座標 (常に0)
    float z;      // Z座標
    int   flags;  // 1=有効, 2=終端
};
```

### パステーブル (PTR_DAT_80495d28)

7本のパス (index 0-6)。各パスはウェイポイント配列:
```
struct Waypoint {
    int   state;          // ノコノコに設定するstate
    int   animState;      // アニメーション状態
    int   countdown;      // このWPでの滞在フレーム
    float moveSpeed;      // 移動速度
    float turnSpeed;      // 旋回速度 (正=左旋回、負=右旋回)
};
// 終端: state == -1
```

パス割り当ては初期化時のランダムテーブル (DAT_80495994) から。

## 自動操舵

NokoNokoChallenge_Update 内で、最寄りのノコノコ方向にカートを誘導する処理:

1. `NokoNoko_FindNearest` (0x801887bc) で最寄り生存ノコノコの位置を取得
2. コース上のノコノコへの方向ベクトルを計算
3. カートの向き (forward) と比較
4. `KartMovement+0x294` にステアリング補助値を設定:
   - 正面方向: 0.0 (補助なし)
   - 斜め方向: -1.0 (軽い補助)
   - 反対方向: 1.0 (強い補助)

## タイマーと勝敗判定

### タイマー
- `FLOAT_806d132c`: 残り時間 (float秒)
- 初期値: `MiniGame_GetTimerValue()` (ccClass/subMode依存)
- 毎フレーム `FLOAT_806d132c -= 1/60` (FLOAT_806d70a0 = 0.016667)
- 0到達 → ゲームオーバー

### 全滅判定
```c
defeatedCount = NokoNoko_CountDefeated();  // state >= 4 の数
totalRequired = self->totalNokoNokoCount;  // self+0x40 (MiniGame_GetCourseDataから)
if (defeatedCount >= totalRequired) {
    NokoNoko_StopAllAlive();  // 残存ノコノコ停止
    self->allClearedFlag = 1; // self+0x36
    self->countdownFrames = 30; // 30F後にリザルト
    KartMovement+0x290 = 0;    // アイテム使用停止
    DAT_806d1289 = 1;          // ゲーム完了
}
```

### リザルト
- 成功 (self+0x36 == 1): `NokoNokoChallenge_CalcResultText(self, 1, 0)` → 勝利テキスト
- 失敗 (self+0x36 == 0): `NokoNokoChallenge_CalcResultText(self, 0, 0)` → 敗北テキスト
- DKコイン計算: `defeatedCount / coinDivisor` → SetCoinCount

## DKコイン (ノコノコ撃破ボーナス)

ノコノコを倒すと DKコインが出現 (カード有効時のみ):
- `DebugPrintf("Start to DKCoin : %s - %d", "CGameCoin.hpp", 0x386)`
- SE 0xab を再生
- DKコインマネージャ (self+0x44) に6スロット、0x16Cバイト/スロット
- 空きスロットを探して、撃破位置+飛散ベクトルを設定
- 毎フレーム `FUN_8011fafc` で8個のサブスプライトを更新

## 定数

| アドレス | 値 | 用途 |
|----------|------|------|
| FLOAT_806d7078 | 12.0 | カート前方の衝突判定半径 |
| FLOAT_806d7094 | 8.0 | ハンマーの衝突判定半径 |
| FLOAT_806d70a0 | 1/60 | タイマー減算値/frame |
| FLOAT_806d70a4 | 100.0 | 速度表示スケール |
| FLOAT_806d932c | 20.0 | ノコノコ同士の衝突距離 |
| FLOAT_806d933c | 8.5 | ノコノコの衝突半径 |
| FLOAT_806d9344 | 0.05 | DYING時の落下加速 |
| FLOAT_806cfbd8 | (可変) | ノコノコ速度乗数 (MiniGame_GetCourseDataから) |

## Ghidra リネーム一覧

| アドレス | 旧名 | 新名 |
|----------|------|------|
| 0x801289b8 | TimeTrial_Update | NokoNokoChallenge_Update |
| 0x80129938 | TimeTrial_Init | NokoNokoChallenge_Init |
| 0x80122bb0 | TimeTrial_HandleBrakeInput | NokoNokoChallenge_HandleBrakeInput |
| 0x80122ce4 | TimeTrial_CalcResultText | NokoNokoChallenge_CalcResultText |
| 0x80122e80 | TimeTrial_GetCoinDivisor | NokoNokoChallenge_GetCoinDivisor |
| 0x80122c0c | TimeTrial_SaveResult | NokoNokoChallenge_SaveResult |
| 0x80122e58 | TimeTrial_TransitionToResult | NokoNokoChallenge_TransitionToResult |
| 0x80187d2c | FUN_80187d2c | NokoNoko_Update |
| 0x801884e8 | FUN_801884e8 | NokoNoko_InitEntity |
| 0x801888b8 | FUN_801888b8 | NokoNoko_CheckHitAndKill |
| 0x80188ae4 | FUN_80188ae4 | NokoNoko_CountDefeated |
| 0x80188b24 | FUN_80188b24 | NokoNoko_Render |
| 0x80188d0c | FUN_80188d0c | NokoNoko_StopAllAlive |
| 0x80188d5c | FUN_80188d5c | NokoNoko_UpdateAll |
| 0x801887bc | FUN_801887bc | NokoNoko_FindNearest |
| 0x80186928 | FUN_80186928 | LookupSin |
| 0x801868ec | FUN_801868ec | LookupCos |
| 0x800387b8 | FUN_800387b8 | Clamp_Int |
| 0x8005f5cc | FUN_8005f5cc | IsSpawnTableTerminator |
