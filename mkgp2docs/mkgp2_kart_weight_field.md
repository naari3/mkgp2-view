# KartParamBlock+0x58 `weight` フィールド調査

## 結論

**`weight` フィールド (KartParamBlock+0x58) はゲーム内コードから一切参照されていない未使用パラメータである。**

## 調査方法

1. main.dol の全命令 (716,264命令) をスキャンし、offset 0x58 への `lfs`/`lfd`/`lwz` アクセスを網羅的���検出 (462件)
2. 各ヒットについて、KartParamBlockポインタ経由のアクセスかを前方/後方のレジスタトレースで判定
3. `<<PARAMBLOCK>>` マークが付いた4件を個別にデコンパイルし、全て偽陽性 (別構造体の+0x58) と確認
4. KartParamBlockを使う全ての主要関数を手動確認:
   - `KartMovement_PhysicsStep` (0x8019cd30): +0x04, +0x08, +0x18, +0x2c, +0x30, +0x34, +0x38, +0x50, +0x54, +0x60, +0x64, +0x68, +0x6c, +0x74, +0x78, +0x7c, +0x80, +0x84, +0x88, +0x8c を参照。**+0x58 不使用**
   - `KartMovement_Init` (0x8019e20c): +0x28, +0x40, +0xF0, +0xF4 のみ参照。**+0x58 不使用**
   - `KartMovement_CollisionUpdate` (0x8019b080): KartParamBlock非参照 (KartMovement自体のorientMatrix +0x58 はアクセス)
   - `FUN_8004c320` (CarObjectメインアップデートサブ): KartParamBlock非参照
   - `ItemEffect_Dispatch` (0x80050410): KartMovement+0x58 (orientMatrix) を参照、KartParamBlock非参照
   - `KartPhysics_ApplyCollisionForce` (0x801edfec): PhysicsState構造体の+0x58 を参照、KartParamBlock非参照
   - `KartController_Update` (0x801e6ed4): KartParamBlock非参照
   - `DebugOverlay_KartPhysics`: 表示しない
5. `GetKartParamBlock` (0x80090ff0) のcallerは `CarObject_Init` のみ

## データ値

| キャラ分類 | weight値 |
|-----------|---------|
| 軽量級    | 1.0     |
| 中量級    | 2.0     |
| 重量級    | 3.0     |
| Luigi     | 5.0     |

値はキャラ間で明確に異なるが、コード��参照されないため物理挙動に影響しない。

## 考察

- KartParamBlockにはweight以外にも `unused08`, `flags5C` など使われていなさそうなフィールドがある
- weightは開発途中のカート間衝突システム (重量差による吹っ飛���計算など) のために設計されたが、最終的に実装されなかったパラメータと思われる
- 実際のカート間衝突は `KartMovement_CollisionUpdate` で壁衝突と同じロジックで処理され、重量差の概念がない
- Luigiの5.0は他と比べて異常に高く、テスト中のデバッグ値が残った可能性がある

## カート間衝突メカニクス (2026-04-04 追加調査)

### 衝突処理の呼び出しフロー

```
KartController_Update (0x801e6ed4)
  └→ KartPhysics_UpdateSmoothing (0x801effa4)
       ├→ KartPhysics_GroundedMovement (0x801eed14)   // 接地時物理
       ├→ KartPhysics_ApplyCollisionForce (0x801edfec) // 衝突力を位置に適用
       ├→ PhysicsState_WallCollisionCheck (0x801ed0e8) // 壁/環境レイキャスト
       └→ PhysicsState_KartCollisionCheck (0x801ecb88) // カート間衝突判定
            └→ CollisionTest_CalcPenetration (0x80207e3c) // 貫通量計算
```

### PhysicsState_KartCollisionCheck (0x801ecb88) 詳細

カート間衝突の判定と応答を行う中核関数。

**判定フェーズ:**
1. PhysicsState+0x124 (collidedObject) が存在すれば、その相手カートとの衝突を処理
2. collidedObject がない場合、DAT_806d1358 のワールド衝突システムを使って最大4体まで走査
3. `CollisionTest_CalcPenetration` (0x80207e3c) でバウンディングボリューム間の貫通量を計算
4. 貫通量 > 0 なら衝突成立

**応答フェーズ:**
- 衝突法線を `Vec3_Subtract` + `Vec3_Normalize` で計算
- **衝突力** = 法線 * FLOAT_806da47c (**-1.6**, 固定定数)
  - PhysicsState+0xd8/+0xdc/+0xe0 に加算
- **位置補間** = 現在位置を前フレーム位置方向にスナップ (スケール 1.0)
- **衝突方向判定**: 相手カートとの角度差から前方/側方/後方ヒットを分類
  - `FUN_80040c24` (前方), `FUN_80040ca0` (側方), `FUN_80040d1c` (後方) を呼び分け

**衝突バウンディングサイズ**: 全カート共通固定値
- X=8.0, Y=10.0, Z=15.0 (FLOAT_806da474/806da444/806da478)

### PhysicsState_WallCollisionCheck (0x801ed0e8)

壁/コース境界との衝突チェック。4方向レイキャスト。

- レイ方向テーブル (DAT_803bdd28): (+-7.2, 0, +-9.6) をPhysicsState+0x13cでスケーリング
- 衝突力は貫通量に比例 (FLOAT_806da4a0/806da4a4 でスケール)

### KartPhysics_ApplyCollisionForce (0x801edfec)

フレームごとに衝突力を位置に適用し減衰させる。

- **衝突力** (+0xd8-0xe0): magnitude をクランプ後、位置に減算。0.92倍で減衰
- **ドリフト力** (+0xcc-0xd4): heading delta から生成。0.88倍で減衰

### weight 不使用の確証

今回の追加調査で、カート間衝突の全パスを完全にデコンパイルし確認:
- `PhysicsState_KartCollisionCheck`: KartParamBlock を参照しない。衝突力は固定定数
- `CollisionTest_CalcPenetration`: バウンディングサイズは collisionObject 構造体から取得。KartParamBlock 無関係
- `KartPhysics_ApplyCollisionForce`: PhysicsState のみ操作。KartParamBlock 無関係
- `PhysicsState_WallCollisionCheck`: PhysicsState+0x13c をスケールに使うが、KartParamBlock+0x58 ではない
