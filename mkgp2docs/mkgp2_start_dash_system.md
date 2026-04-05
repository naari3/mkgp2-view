# MKGP2 スタートダッシュ (ロケットスタート)

## 概要

レース開始カウントダウン中にアクセルを踏み続けると発動するブースト。成功すると speedTableIndex が専用ブーストカーブ(idx=4)に切り替わり、travelProgress が最大値に跳ぶことで、通常走行より高い speedCap が即座に適用される。

## キャラ/カート依存性 (ROM追証済み)

- **キャラ間**: 全13キャラの150cc NORMALでentry 4カーブデータをバイト単位比較 → **全バイト同一**
- **バリアント間**: NORMAL/ORIGINAL/ENHANCEDのentry 4は同一curveDataPtrを共有
- **ccクラス間**: ブーストカーブのピーク速度が異なる

| ccクラス | entry 4 ピーク速度 | 通常走行(entry 0)最高速(参考) |
|---|---|---|
| 50cc | 66.67 (200/3) | 77.78 |
| 100cc | 66.67 (200/3) | 77.78 |
| 150cc | 83.33 (250/3) | 77.78 |

結論: **スタートダッシュの性能はccクラスのみに依存し、キャラ・カートバリアントによる差は一切ない。**

なお、全7スピードテーブルエントリ(entry 0〜6)のカーブデータを同cc・同バリアントのキャラ間で比較した結果、すべて同一だった。キャラ間の速度差はスピードカーブには存在しない。

## AI のロケットスタート

**AIはロケットスタート機構を持っていない。** GPモードでのAIの速度制御は KartController → AI_CalcGPTargetSpeed 経由の目標速度補間であり、CarObject_ApplyInput のスタートダッシュ判定を通らない。

GO! の前後でAIの速度は以下のように切り替わる:
- GO! 前: `GetKartTuningSpeedFactor(slot) * scale` (低速)
- GO! 後: `AI_CalcGPTargetSpeed()` → BaseSpeed + チューニング調整

スロット別の speedFactor / accelRate の差によりスタート速度に大きな差が出る:
- S0 (最速AI): speedFactor=0.60, accelRate=0.90 → 素早く高速に到達
- S4 (最遅AI): speedFactor=0.20, accelRate=0.50 → 明らかに出遅れる

「5位のAIだけロケットスタートしない」説は、S4 の低い speedFactor/accelRate による遅いスタートが原因。実際にはどのAIもロケットスタートしていない。

## カウントダウンシーケンス

**LakituStart_UpdateCountdown** (0x8003d360) がジュゲムの信号ライトアニメーションを駆動:

| g_countdownPhase | アニメフレーム | 状態 | SE |
|---|---|---|---|
| 4 | 0 | 初期化 | - |
| 3 | >= 120 | 赤1 ON | 0x45 |
| 2 | >= 180 | 赤2 ON | 0x45 |
| 1 | >= 240 | 赤3 ON | 0x45 |
| 0 | >= 300 | GO! (青3つ) | 0x46 |

フレーム301到達で `SetRaceStartFlag(1)` → `g_raceStarted` = 1。

## タイミング判定

**CarObject_ApplyInput** (0x8004e154) 内で毎フレーム判定。CarObject_ApplyInput_AI (0x8004dfe0) にも同一ロジックあり (VSモード用)。

**成否判定は GO! の瞬間 (フレーム300で g_raceStarted=1 になった瞬間) に1回だけ行われる。** ウィンドウが329まであるのは startDashReady フラグの設定範囲であり、判定自体ではない。GO! 後にアクセルを踏んでも間に合わない。

### 状態変数 (CarObject内)

| Offset | Type | Name | 説明 |
|---|---|---|---|
| +0xA8 | byte | raceStartedLocal | レース開始済みフラグ(ローカル) |
| +0xAC | float | savedTravelProgress | 15フレームごとのスナップショット |
| +0xD5 | byte | startDashReady | 成功判定フラグ |
| +0xD8 | int | startDashFrameCounter | カウントダウン中フレーム数 |

### ロジック

```
毎フレーム:
  if g_countdownActive == 1:
    startDashFrameCounter++

  // 15フレームごとにスナップショットリセット
  if startDashFrameCounter % 15 == 0:
    savedTravelProgress = kartMovement->travelProgress
    startDashReady = false

  // タイミングウィンドウ: フレーム 241 〜 329
  if 240 < startDashFrameCounter < 330:
    if kartMovement->travelProgress > savedTravelProgress:
      startDashReady = true

  // GOシグナルの瞬間
  if wasNotStarted && raceStarted:
    if startDashReady:
      KartMovement_SetTravelProgress(10000.0)  // → 5000.0にクランプ
      SpeedBoost_Apply(1.0, driftController, 4) // ブーストカーブに切替
      SE再生(0x54)
    else:
      KartMovement_SetTravelProgress(0.0)       // 通常の0スタート
```

### タイミングウィンドウ

- **有効フレーム**: 241 〜 329 (89フレーム)
- GOシグナル = フレーム300。つまり **GO の約1秒前 〜 GO の約0.5秒後**
- 15フレーム周期のスナップショットリセットがあるため、直前の15フレーム窓内でアクセルを踏んでいる必要がある

## ブースト適用メカニズム

### 1. travelProgress ジャンプ

`KartMovement_SetTravelProgress` (0x8019a87c) で travelProgress = 10000.0 に設定。SpeedCurve の distThreshold = 5000.0 でクランプされるため、実質カーブ最大値に到達。

### 2. speedTableIndex 切替

`SpeedBoost_Apply` (0x80056244) で speedTableIndex を 4 に変更。entry 4 はフラットなブーストカーブ(全区間で同一速度)を持ち、通常走行 (entry 0) のカーブ最大値より高い speedCap を返す。

### 3. 減衰

speedTableIndex は他のブーストイベント(ドリフトブースト等)で上書きされるか、減速時に通常カーブに戻る。SpeedCurve_UpdateSpeedCap が毎フレーム travelProgress を自然減衰させる。

## スピードテーブル構造

各 KartParamBlock は7エントリのスピードテーブルを持つ。各エントリは 0x18 バイト:

| Offset | Type | 説明 |
|---|---|---|
| +0x00 | ptr | curveDataPtr (7点のカーブ) |
| +0x04 | int | curvePointCount (常に7) |
| +0x08 | float | distThreshold (常に5000.0) |
| +0x0C | float | maxSpeed (常に21.0) |
| +0x10 | float | param10 (常に0.1) |
| +0x14 | float | param14 (常に0.3) |

| Index | 用途 | カーブ形状 |
|---|---|---|
| 0 | 通常走行 | 漸増カーブ (0→77.78) |
| 1-3 | ブースト(予備) | フラット (83.33@150cc) |
| 4 | スタートダッシュ | フラット (83.33@150cc) |
| 5 | ドリフトブースト | フラット (83.33@150cc) |
| 6 | アドバンストブースト | フラット (83.33@150cc) |

entry 0 のみバリアント間で異なるカーブデータを持つ(NORMALとORIGINALで異なる最高速)。entry 1-6 は全バリアントで共有。

## 関数一覧

| Address | Name | 説明 |
|---|---|---|
| 0x8004e154 | CarObject_ApplyInput | プレイヤー入力 + スタートダッシュ判定 |
| 0x8004dfe0 | CarObject_ApplyInput_AI | AI用入力(同一判定ロジック) |
| 0x8003d360 | LakituStart_UpdateCountdown | カウントダウンアニメ + フェーズ遷移 |
| 0x8003db40 | LakituStart_Init | カウントダウン初期化 |
| 0x8019a87c | KartMovement_SetTravelProgress | travelProgress直接設定 |
| 0x80056244 | SpeedBoost_Apply | speedTableIndex切替 |
| 0x8004f450 | CarObject_ApplyDriftBoost | ドリフトブースト適用 |
| 0x8004f858 | CarObject_HandleItemEffect | アイテムエフェクトハンドラ |
| 0x8009c188 | SetRaceStartFlag | g_raceStarted 設定 |
| 0x8009c1a8 | IsRaceStarted | g_raceStarted 取得 |

## グローバル変数

| Address | Name | Type | 説明 |
|---|---|---|---|
| 0x806d12e0 | g_countdownActive | byte | カウントダウン中 (1=active) |
| 0x806d1260 | g_raceStarted | byte | レース開始済み (1=GO後) |
| 0x806d1284 | g_countdownPhase | int | フェーズ (4=init 〜 0=GO) |

## 定数

| Address | Value | 用途 |
|---|---|---|
| 0x806d27d8 | 10000.0 | 成功時 travelProgress 目標値 |
| 0x806d26ec | 0.0 | 失敗時 travelProgress |
| 0x806d28a0 | 0.6 | ブースト時 KartMovement+0x1D8 |
