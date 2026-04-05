# ビリビリランド (BiribiriLand) ミニゲーム解析

## 概要
- **courseId**: 0xC (12)
- **種別**: チャレンジモード / ミニゲーム
- **ccVariant**: 3 (ミニゲーム用: speedCap=300, mue=0.4)
- **内容**: 直線上の電気障害物をジャンプ/操舵で避けながらゴールを目指すミニゲーム
- **詳細解析**: `mkgp2_biribiri_land_detail.md` を参照

## 関数一覧

| アドレス | 関数名 | 役割 |
|----------|--------|------|
| 0x80134ff0 | BiribiriLand_Init | 初期化 (コンストラクタ) |
| 0x8013449c | BiribiriLand_Update | 毎フレーム更新 |
| 0x801341e4 | BiribiriLand_Render | 描画 |
| 0x80134a8c | BiribiriLand_UpdateObstacles | 障害物更新 (浮遊アニメ + 最近接追跡) |
| 0x80134d68 | BiribiriLand_Destroy | デストラクタ |

## vtable (PTR_PTR_8048e46c)
- [0]: typeinfo ptr (0x806cf604)
- [1]: NULL
- [2]: BiribiriLand_Update (0x8013449c)
- [3]: BiribiriLand_Render (0x801341e4)

基底vtable: PTR_PTR_8048e144 (全ミニゲーム共通)

## BiribiriLandContext 構造体 (0x44 bytes = Alloc(0x44) in MiniGame_CreateByCourseId)

| Offset | 型 | フィールド名 | 説明 |
|--------|-----|-------------|------|
| +0x00 | void* | vtable | BiribiriLand vtable |
| +0x04 | float | defaultFloatVal | 初期値 0.0 |
| +0x08 | void* | carObject | プレイヤーCarObject |
| +0x0C | void* | lakituStart | ジュゲムスタート演出 |
| +0x10 | void* | courseMesh | コースメッシュ (InputManager) |
| +0x14 | void* | scene3D | Scene3D インスタンス |
| +0x18 | void* | goalCamera | ゴールカメラ (cam_challenge_goal) |
| +0x1C | void* | renderBuffer | カメラバッファ |
| +0x20 | void* | textureBuffer | レンダーターゲット |
| +0x24 | void* | hudManager | HUD管理オブジェクト |
| +0x28 | byte | isInitialized | 初期化済みフラグ |
| +0x29 | byte | flag29 | 不明 |
| +0x2A | byte | isFinished | ゲーム終了フラグ |
| +0x2B | byte | reachedGoal | ゴール到達フラグ |
| +0x2C | int | finishFrameCount | 終了後フレームカウンタ |
| +0x30 | void* | collectObjArray | 回収オブジェクト配列 (0x214 bytes) |
| +0x34 | void* | goalZone | ゴールゾーン定義 (0x30 bytes) |
| +0x38 | float | floatE | 不明 (初期値 0.0) |
| +0x3C | float | floatF | 不明 (初期値 0.0) |
| +0x40 | float | float10 | 不明 (初期値 0.0) |

## 回収オブジェクト配列 (0x214 bytes)

### ヘッダ (7 model pointers)
| Offset | 内容 |
|--------|------|
| +0x00 | obj_red.dat モデル |
| +0x04 | obj_blue.dat モデル |
| +0x08 | obj_yellow.dat モデル |
| +0x0C | obj_pink.dat モデル |
| +0x10 | obj_purple.dat モデル |
| +0x14 | obj_UFO.dat モデル |
| +0x18 | land_biribiri.dat モデル (UFO型、サイズスケール異なる) |

### スロット配列 (9 slots x 0x38 bytes each, starting at +0x1C)

BiribiriCollectSlot 構造体:

| Offset | 型 | フィールド名 | 説明 |
|--------|-----|-------------|------|
| +0x00 | float | startMinX | 配置データ始点X |
| +0x04 | float | startMinY | 配置データ始点Y |
| +0x08 | float | startMinZ | 配置データ始点Z |
| +0x0C | float | startMaxX | 配置データ終点X |
| +0x10 | float | startMaxY | 配置データ終点Y |
| +0x14 | float | startMaxZ | 配置データ終点Z |
| +0x18 | float | centerX | 中心X (= 0.5 * (startMin + startMax)) |
| +0x1C | float | centerY | 中心Y (+ yOffset) |
| +0x20 | float | centerZ | 中心Z |
| +0x24 | float | normalX | 法線X (正規化済み方向ベクトル) |
| +0x28 | float | normalY | 法線Y |
| +0x2C | float | normalZ | 法線Z |
| +0x30 | float | bouncePhase | 浮遊アニメーション位相 |
| +0x34 | byte | isCollected | 回収済みフラグ |

## 初期化フロー (BiribiriLand_Init)

1. 基底クラス初期化 (GameMode_BaseInit)
2. vtable設定 (基底→派生の順)
3. メンバ変数ゼロクリア
4. 描画/サウンド/リソースマネージャ初期化
5. コースデータ/カメラ/レンダーターゲット取得
6. 開始位置取得 → CarObject_Init (charId, ccClass=1, ccVariant=3)
7. ジュゲムスタート演出 (jyugemu_start.dat)
8. Scene3D + ゴールカメラ (cam_challenge_goal)
9. HUD初期化 (overlay 6, 0x11, 0x12, 0xE)
10. タイマー設定 (MiniGame_GetTimerValue → FLOAT_806d132c)
11. エフェクトリソースプリロード (1, 9, 10)
12. 7種オブジェクトモデル読み込み + 9箇所配置データ設定
13. ゴールゾーン定義 (2点 + 法線)
14. 天気システム初期化 (WeatherSystem_Init)

## ゲームプレイメカニクス (BiribiriLand_Update)

- カート入力処理 → 物理更新 → CarObject_FrameUpdate
- 9個のオブジェクトとの距離判定 → 回収時SE(0x90)再生 + isCollected=1
- ゴールゾーン通過 → reachedGoal=1, isFinished=1
- タイマーカウントダウン (FLOAT_806d132c) → 0到達でタイムアップ (isFinished=1)
- 終了処理: 残タイムからコイン報酬算出 (テーブル参照 → SetCoinCount)

## ccVariant=3 の影響

CarObject_Init に渡された ccVariant=3 は、GetKartParamBlock (0x80090ff0) 内で:
- `g_kartParamCcVariantTable[ccVariant * 3 + ccClass]` でパラメータブロック取得
- ccVariant=3, ccClass=1 → テーブルインデックス 10
- パラメータ: speedCap=300, mue=0.4 (高グリップ・高速のミニゲーム用設定)

## 呼び出し元

MiniGame_CreateByCourseId (0x80123050):
- g_courseId によるswitch文
- courseId=0xC → Alloc(0x44) → BiribiriLand_Init

## 配置データ

DAT_80357f6c: 9スロット x 9 words (float[6] = startMin/Max XYZ + float[3] = 追加データ)
PTR_s_obj_red_dat_8048e430: 7つのモデルファイル名ポインタ配列
