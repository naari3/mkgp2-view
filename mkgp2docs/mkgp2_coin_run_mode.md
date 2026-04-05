# CoinRunMode (courseId=0x10) 解析

> **NOTE**: このドキュメントは旧版。最新の詳細解析は `mkgp2_coin_run_jump.md` を参照。
> CoinRunMode -> JumpDistanceMode にリネーム済み。実態はジャンプ飛距離計測ミニゲーム。

## 概要

courseId=0x10 (16) のミニゲームモード。プレイヤーがコースを自由に走行し、走行距離に応じてコインを獲得する時間制限付きゲーム。

ccVariant=7 を使用 (speedCap=250, mue=0.4)。他のミニゲーム系より低速な特殊パラメータ。

## 関数一覧

| アドレス | 名前 | 役割 |
|----------|------|------|
| 0x80213210 | CoinRunMode_Init | 初期化。CarObject, シーン, カメラ, タイマー, コインエフェクト等を構築 |
| 0x80211e6c | CoinRunMode_Update | 毎フレーム更新。カメラ, タイマーカウントダウン, HUD更新, 結果遷移 |
| 0x80210cd4 | CoinRunMode_UpdateGameplay | ゲームプレイ状態マシン。走行距離計算, コイン付与, 境界チェック |
| 0x80213a64 | CoinRunMode_InitTimerDisplay | タイマー数字表示の初期化 (3桁: 百の位, 十の位, 一の位) |
| 0x80213d58 | CoinRunMode_CopyKartMovement | KartMovement構造体(0x324バイト)のフルコピー |
| 0x80123050 | MiniGame_CreateByType | ミニゲームディスパッチャ。g_courseIdでswitch |

## コンテキスト構造体 (CoinRunModeCtx, 0x8C バイト)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| 0x00 | ptr | vtable | 仮想関数テーブル (PTR_PTR_804ecc08) |
| 0x04 | float | timeScale | 時間スケール |
| 0x08 | ptr | carObject | CarObject (0x118バイト, ccVariant=7) |
| 0x0C | ptr | lakituSignal | ジュゲムスタートシグナル (jyugemu_start.dat) |
| 0x10 | ptr | inputHandler | 入力ハンドラ (0x37Cバイト) |
| 0x14 | ptr | scene3d | 3Dシーン (0x3084バイト) |
| 0x18 | ptr | timerController | タイマーコントローラ (8バイト) |
| 0x1C | ptr | itemSlotUI | アイテムスロットUI (0x44バイト) |
| 0x20 | ptr | camera | カメラオブジェクト (0x2Cバイト) |
| 0x24 | ptr | textureData | テクスチャデータ |
| 0x28 | ptr | resultDisplay | 結果表示 (0x80バイト, HudManager) |
| 0x2C | ptr | timerDigits | タイマー数字表示 (0x2Cバイト) |
| 0x30 | ptr | cameraPath0 | カメラパス0 (0x1Cバイト) |
| 0x34 | ptr | cameraPath1 | カメラパス1 (0x1Cバイト) |
| 0x38 | ptr | cameraPath2 | カメラパス2 (0x1Cバイト) |
| 0x3C | ptr | animController | アニメーション制御 (0x98バイト) |
| 0x40 | ptr | cameraTarget | カメラターゲット制御 (0x24バイト) |
| 0x44 | byte | isInitialized | 初期化済みフラグ |
| 0x45 | byte | isGameOver | ゲームオーバーフラグ |
| 0x48 | int | waitFrames | 待機フレーム数 |
| 0x4C | int | resultWaitFrames | 結果待機フレーム数 |
| 0x50 | int | gameState | ゲーム状態 (0-5) |
| 0x54 | float | highScore | ハイスコア(距離) |
| 0x58 | float | travelDistance | 走行距離 (累積) |
| 0x5C | float | prevPosX | 前フレームX座標 |
| 0x60 | float | prevPosY | 前フレームY座標 |
| 0x64 | float | prevPosZ | 前フレームZ座標 |
| 0x68 | float | checkpointSpeed | チェックポイント通過速度 |
| 0x74 | float | elapsedTime | 経過時間 |
| 0x7C | ptr | coinDisplay | コイン表示 |
| 0x80 | int | coinMilestoneIdx | コインマイルストーンインデックス |
| 0x84 | int | earnedCoins | 獲得コイン数 |
| 0x88 | byte | isRunning | 走行中フラグ |

## ゲームプレイ状態マシン (gameState)

| 状態 | 説明 |
|------|------|
| 0 | WaitStart: カートが動き始めるのを待つ |
| 1 | Driving: 走行中。走行距離を累積し、マイルストーンでコイン付与 |
| 2 | ReachedCheckpoint: チェックポイント到達 |
| 3 | ApproachTarget: ターゲットに接近中 |
| 4 | OvershootBounce: オーバーシュート時のバウンス処理 |
| 5 | TimeUp: 時間切れ。IsCardValid()でカード確認後、結果画面へ |

## vtable (PTR_PTR_804ecc08)

| Offset | アドレス | 関数 |
|--------|----------|------|
| 0x00 | 806d03f0 | デストラクタ |
| 0x04 | 00000000 | (null) |
| 0x08 | 80211e6c | CoinRunMode_Update |
| 0x0C | 80211b10 | (別の更新関数) |
| 0x10 | 80212ea4 | (描画関数?) |

## MiniGame_CreateByType ディスパッチテーブル

| courseId | Alloc サイズ | 関数 | ccVariant | モード |
|----------|-------------|------|-----------|--------|
| 9 | 0x6C | FUN_80133398 | 3 | ミニゲーム |
| 10 | 0x48 | FUN_80129938 | 1 | タイムトライアル |
| 11 (0xB) | 0x4C | FUN_800ad910 | 0 | 試乗 |
| 12 (0xC) | 0x44 | FUN_80134ff0 | 3 | ミニゲーム |
| 13 (0xD) | 0xC4 | FUN_8012fc58 | - | (未調査) |
| 14 (0xE) | 0x3C | FUN_80136ea0 | - | (未調査) |
| 15 (0xF) | 0x48 | FUN_80214db8 | 6 | ミニゲーム |
| 16 (0x10) | 0x8C | CoinRunMode_Init | 7 | コインラン |

## コインエフェクトシステム

ctx[0x1F] にコインエフェクト管理構造体 (0xC4C バイト) を格納。

- `mario_coin.dat`: コインモデル
- `k_kira13.dat`: キラキラエフェクト
- 16個のコインスロット (ループ回数=0xf+1=16)、各0x1A*4=0x68バイト
- スロット配置: 5x3グリッド (iVar7 % 5, iVar7 / 5) でX/Z座標を DAT_803c3d28[5], DAT_803c3d3c[3] から取得

## カメラパス

3本のカメラパス (ctx+0x30, +0x34, +0x38) は DAT_806d1270 (コース向き) に応じて2セットのデータを切り替え:
- DAT_806d1270 == 0: DAT_803c3c60/3c90/3cc0 (通常)
- DAT_806d1270 != 0: DAT_803c3c78/3ca8/3cd8 (ミラー)

## ccVariant=7 の参照

ccVariant はCarObject_Init の param_10 として渡され、内部で `GetKartParamBlock(charId, ccClass, kartVariant, ccVariant)` に転送される。ccVariant != -1 の場合、通常のkartParamBlockの代わりに `g_kartParamCcVariantTable[ccVariant * 3 + ccClass]` が使用される。

ccVariant=7 のパラメータ: speed=250, mue=0.4 (他のミニゲーム系 ccV=3,5,6 は speed=300)
