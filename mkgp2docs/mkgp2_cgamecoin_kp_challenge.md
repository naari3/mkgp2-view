# CGameCoin - KPクッパ城チャレンジ (courseId=0xD)

## 概要

KP(クッパ)カップの城ステージを使ったミニゲーム。プレイヤーが黒い甲羅を投げてクッパ(またはメカクッパ)に当て、コインを獲得する。ソースファイル名: `CGameCoin.hpp`

## ゲーム内容

- ステージ: KP_castle_challenge (クッパ城チャレンジ専用マップ)
- 制限時間つきで甲羅を投げ続ける
- カメラは5段階に切り替え可能 (上下入力)
- ボスにヒットするとコインが噴出 (3種類のコイン: kind 0, 1, 2)
- ボス撃破(4回ヒット)で追加のコイン噴出演出
- charId==4(クッパ)選択時はメカクッパがボスに差し替わる

## 構造体 CGameCoin (0xC4 = 196バイト)

| Offset | 型 | 名前 | 説明 |
|--------|------|------|------|
| 0x00 | ptr | vtable | PTR_PTR_8048e3a0 |
| 0x04 | float | baseFloat | 初期化用float |
| 0x08 | ptr | pTimer | タイマー |
| 0x0C | ptr | pCarObject | CarObject |
| 0x10 | ptr | pKartModel | カートモデル |
| 0x14 | ptr | pSoundVolumePan | サウンド |
| 0x18 | byte[48] | orientMatrix | 方向行列 |
| 0x48 | ptr | pBossKoopa | ボスクッパ制御 |
| 0x4C | ptr | pShell | 甲羅オブジェクト |
| 0x50 | ptr | pKartDisplay | カート表示 |
| 0x54 | ptr | pScene3D | Scene3D (コース) |
| 0x58 | ptr | pChallengeGoalCamera | ゴールカメラ |
| 0x5C | ptr | pInputManager | 入力管理 |
| 0x68 | ptr | pHUDTimer | HUDタイマー表示 |
| 0x6C | ptr | pHitDetector | 当たり判定 |
| 0x70 | ptr | pFireballManager | 炎弾管理 |
| 0x78 | ptr | pDefeatDemo | 撃破演出 |
| 0x7C | byte | isInitialized | 初期化済みフラグ |
| 0x7D | byte | isSoundPlaying | BGM再生中 |
| 0x7E | byte | isGameOver | ゲーム終了 |
| 0x7F | byte | isKoopaDefeated | クッパ撃破済み |
| 0x80 | int | resultFrameCounter | リザルト後フレームカウンタ |
| 0x84 | ptr | pCourseData | コースデータ |
| 0x88 | int | currentCameraIdx | 現在のカメラ位置 (0-4, 初期値=2) |
| 0x8C | int | targetCameraIdx | 目標カメラ位置 |
| 0x90 | float | cameraTransitionTimer | カメラ遷移タイマー |
| 0x94 | float | shellHitTimer | 甲羅ヒット演出タイマー |
| 0x98 | float | koopaHitTimer | クッパヒット演出タイマー |
| 0x9C | int | countdownValue | カウントダウン |
| 0xA0 | float | cameraYOffset | カメラY軸オフセット |
| 0xA4-0xB4 | ptr*4 | skyDome/cloudJoints | スカイドーム/雲ジョイント |
| 0xB8 | float | skyDomeRotation | スカイドーム回転角 |
| 0xBC | float | skyCloudRotation | 雲回転角 |
| 0xC0 | ptr | pCoinParticleMgr | コインパーティクル管理 |

## リネーム結果

### 関数リネーム

| アドレス | 旧名 | 新名 | 役割 |
|----------|------|------|------|
| 0x8012fc58 | FUN_8012fc58 | CGameCoin_Init | 初期化 |
| 0x8012c538 | FUN_8012c538 | CGameCoin_Update | vtable: メインループ |
| 0x8012c2cc | FUN_8012c2cc | CGameCoin_Draw | vtable: 描画 |
| 0x8012f8b4 | FUN_8012f8b4 | CGameCoin_Destroy | vtable: デストラクタ |
| 0x80130a88 | FUN_80130a88 | CGameCoin_InitFields | フィールド初期化 |
| 0x8013068c | FUN_8013068c | BossKoopa_Init | ボスクッパモデル初期化 |
| 0x8012ec44 | FUN_8012ec44 | BossKoopa_UpdateAnim | ボスアニメ管理 |
| 0x8012f45c | FUN_8012f45c | Shell_UpdateState | 甲羅状態更新 |
| 0x80130d2c | FUN_80130d2c | CoinParticle_Update | コインパーティクル更新 |
| 0x80130b5c | FUN_80130b5c | CoinParticle_Draw | コインパーティクル描画 |
| 0x80130fdc | FUN_80130fdc | CoinParticleManager_Destroy | パーティクル破棄 |
| 0x801303e4 | FUN_801303e4 | CGameCoin_InitDefeatDemo | 撃破演出初期化 |
| 0x8012e69c | FUN_8012e69c | DefeatDemo_Update | 撃破演出更新 |
| 0x8013105c | FUN_8013105c | CGameCoin_ProcessInput | 入力処理 |
| 0x8012f07c | FUN_8012f07c | KoopaFireball_Update | クッパ炎弾更新 |

## リソース

### 3Dモデル
- `it_a_03_black_shell_dat` - 黒い甲羅 (投射物)
- `mario_coin_dat` - マリオコイン (報酬)
- `k_kira13_dat` - キラキラエフェクト
- `boss_koopa_tamb/frame/damage_l/damage_r_dat` - 通常クッパ
- `boss_meka_koopa_*_dat` - メカクッパ (charId==4用)
- `koopa_fire_dat` - クッパの炎
- `koopadead_dat` - クッパ撃破演出
- `challenge_demo_KP_end_dat` - 撃破後デモ

### ジョイント
- `KP_castle_zyougan01-03_joint` - 溶岩ジョイント
- `KP_castle_kp01_skydome_joint` - スカイドーム
- `KP_castle_kp03_skycloud_joint` - 雲
- `kpch_maguma_fluid_parts` - 溶岩流体パーツ

## vtable (0x8048e3a0)

| Offset | アドレス | 関数 |
|--------|----------|------|
| 0x08 | 0x8012c538 | CGameCoin_Update |
| 0x0C | 0x8012c2cc | CGameCoin_Draw |
| 0x10 | 0x8012f8b4 | CGameCoin_Destroy |

## CoinParticleManager vtable (0x8048e3c0)

| Offset | アドレス | 関数 |
|--------|----------|------|
| 0x28 | 0x80130fdc | CoinParticleManager_Destroy |
| 0x2C | 0x80130d2c | CoinParticle_Update |
| 0x30 | 0x80130b5c | CoinParticle_Draw |

## カメラ位置テーブル (DAT_80356bcc)

5段階カメラY位置: 12.0, 6.0, 0.0, -8.0, -16.0

## コイン種類

- kind 0: 通常コイン - 速度スケール FLOAT_806d719c
- kind 1: 中コイン - 速度スケール FLOAT_806d71a0 (X/Z), FLOAT_806d71a4 (Y)
- kind 2: 大コイン - 速度スケール FLOAT_806d719c (X/Z), FLOAT_806d71a8 (Y), パーティクル数10に設定

## ボス撃破演出

- ヒット4回でボス撃破
- 200フレーム後に大量のコインが6方向から噴出 (各方向5発)
- charId!=4(非クッパ)の場合、撃破SE 0x13B を再生
