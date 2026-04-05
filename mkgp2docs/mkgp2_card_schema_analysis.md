# MKGP2 Card Data Structure

Ghidra 解析による 69バイト (552ビット) カードデータのビットパックスキーマ。

## 概要

- **カードサイズ**: 69 bytes (0x45)
- **ビットパック方式**: MSBファースト (ビッグエンディアン bit ordering)
- **スキーマエントリ**: 16 bytes each = `{char* name, int bitsPerElement, int elementCount, int bitOffset(runtime)}`
- **関連関数**:
  - `Card_VerifyChecksum` (0x800903d8): チェックサム検証 + VERSION取得
  - `FUN_8009060c` (card_unpack): VERSION別にスキーマを選択して展開 (v0/v1のみ)
  - `FUN_80090174`: VERSION=0 展開
  - `FUN_8008fa2c`: VERSION=1 展開
  - `card_pack` (0x801d5244): VERSION=2 書き込み
  - `bitpack_init` (0x80088bb8): ビットパックコンテキスト初期化
  - `FUN_80088880`: フィールド読み取り (名前で検索+ビット読み取り)

## チェックサムアルゴリズム

1. チェックサムスキーマ (0x803fed40, 3エントリ) でデータを初期化
2. CHECKSUM フィールド (bits 24-31 = byte 3) の値を読み取り保存
3. CHECKSUM フィールドを0に書き込み
4. 全69バイトを加算
5. `(sum & 0xFF) == 保存値` で検証
6. CHECKSUM フィールドを元の値に復元

**つまり**: byte[3] = (全バイトの合計 - byte[3]) mod 256、すなわち byte[3] を0にした状態での全バイト合計の mod 256。

## チェックサムスキーマ (0x803fed40, 3エントリ)

VERSION判定専用の小スキーマ。メインスキーマの直前に配置。

| Idx | Field | Bits/E | Count | Total | Bit Offset |
|:----|:------|:-------|:------|:------|:-----------|
| 0 | APP-CODE | 8 | 3 | 24 | 0 |
| 1 | CHECKSUM | 8 | 1 | 8 | 24 |
| 2 | VERSION | 3 | 1 | 3 | 32 |
---

## VERSION=0 スキーマ (0x803fed70, 55エントリ, 541ビット)

| Idx | Field | Bits/E | Count | Total | Bit Offset | Notes |
|:----|:------|:-------|:------|:------|:-----------|:------|
| 0 | APP-CODE | 8 | 3 | 24 | 0 | |
| 1 | CHECKSUM | 8 | 1 | 8 | 24 | |
| 2 | VERSION | 3 | 1 | 3 | 32 | |
| 3 | LOCATION-TEST | 1 | 1 | 1 | 35 | ロケテストフラグ |
| 4 | COUNTRY | 1 | 1 | 1 | 36 | isJapanese()と比較 |
| 5 | REMAIN | 6 | 1 | 6 | 37 | 残りクレジット |
| 6 | UPDATE | 1 | 1 | 1 | 43 | |
| 7 | NAME | 8 | 5 | 40 | 44 | インデックス→文字変換 |
| 8 | CHARA | 4 | 1 | 4 | 84 | キャラクターID |
| 9 | TITLE | 8 | 1 | 8 | 88 | 称号ID |
| 10 | TITLE-PARAM | 3 | 1 | 3 | 96 | |
| 11 | VOLUME | 3 | 1 | 3 | 99 | 0=50cc?, 1=100cc, 2=150cc |
| 12 | URA | 1 | 1 | 1 | 102 | ミラーモード |
| 13 | WINS | 10 | 1 | 10 | 103 | 勝利数 (max 1023) |
| 14 | LOSE | 10 | 1 | 10 | 113 | 敗北数 (max 1023) |
| 15 | GET-ITEM-LIST | 1 | 88 | 88 | 123 | アイテム/アンロックフラグ |
| 16 | LAST-GET-ITEM | 8 | 1 | 8 | 211 | |
| 17 | ITEMPACK-1 | 7 | 3 | 21 | 219 | |
| 18 | ITEMPACK-2 | 7 | 3 | 21 | 240 | |
| 19 | ITEMPACK-3 | 7 | 3 | 21 | 261 | |
| 20 | RESULT-GP-MR | 1 | 4 | 4 | 282 | マリオカップ結果 |
| 21 | RESULT-GP-DK | 1 | 4 | 4 | 286 | DKカップ結果 |
| 22 | RESULT-GP-WC | 1 | 4 | 4 | 290 | ワリオカップ結果 |
| 23 | RESULT-GP-PC | 1 | 4 | 4 | 294 | パックマンカップ結果 |
| 24 | RESULT-GP-KP | 1 | 4 | 4 | 298 | クッパカップ結果 |
| 25 | RESULT-GP-RB | 1 | 4 | 4 | 302 | RBカップ結果 |
| 26 | GHOST-MR | 1 | 4 | 4 | 306 | |
| 27 | GHOST-DK | 1 | 4 | 4 | 310 | |
| 28 | GHOST-WC | 1 | 4 | 4 | 314 | |
| 29 | GHOST-PC | 1 | 4 | 4 | 318 | |
| 30 | GHOST-KP | 1 | 4 | 4 | 322 | |
| 31 | GHOST-RB | 1 | 4 | 4 | 326 | |
| 32 | WIN-GHOST-MR | 1 | 4 | 4 | 330 | |
| 33 | WIN-GHOST-DK | 1 | 4 | 4 | 334 | |
| 34 | WIN-GHOST-WC | 1 | 4 | 4 | 338 | |
| 35 | WIN-GHOST-PC | 1 | 4 | 4 | 342 | |
| 36 | WIN-GHOST-KP | 1 | 4 | 4 | 346 | |
| 37 | WIN-GHOST-RB | 1 | 4 | 4 | 350 | |
| 38 | RECORD-TA | 19 | 6 | 114 | 354 | タイムアタック記録 |
| 39 | RECORD-STG | 3 | 6 | 18 | 468 | ステージ記録 |
| 40 | RECORD-CRS | 1 | 6 | 6 | 486 | コース記録フラグ |
| 41 | RECORD-DIR | 1 | 6 | 6 | 492 | 方向/ミラーフラグ |
| 42 | MISS-GP | 1 | 1 | 1 | 498 | |
| 43 | BATTLE-WIN-MR-TAG | 6 | 1 | 6 | 499 | |
| 44 | BATTLE-WIN-DK-TAG | 6 | 1 | 6 | 505 | |
| 45 | BATTLE-WIN-WC-TAG | 6 | 1 | 6 | 511 | |
| 46 | BATTLE-WIN-PC-TAG | 6 | 1 | 6 | 517 | |
| 47 | BATTLE-WIN-KP-TAG | 6 | 1 | 6 | 523 | |
| 48 | BATTLE-WIN-RB-TAG | 6 | 1 | 6 | 529 | |
| 49 | BATTLE-LOSE-MR-TAG | 1 | 1 | 1 | 535 | |
| 50 | BATTLE-LOSE-DK-TAG | 1 | 1 | 1 | 536 | |
| 51 | BATTLE-LOSE-WC-TAG | 1 | 1 | 1 | 537 | |
| 52 | BATTLE-LOSE-PC-TAG | 1 | 1 | 1 | 538 | |
| 53 | BATTLE-LOSE-KP-TAG | 1 | 1 | 1 | 539 | |
| 54 | BATTLE-LOSE-RB-TAG | 1 | 1 | 1 | 540 | |

---

## VERSION=1 スキーマ (0x803ff0e0, 64エントリ, 534ビット)

v0からの変更点: LOCATION-TEST削除, CAMERA-TAG追加, URA拡張(1→2bit), GET-ITEM-LIST拡張(88→100), ITEMPACK再構成(7bit×3pack→8bit×2pack), RECORD配列縮小(6→4要素), 新規フィールド多数追加。

| Idx | Field | Bits/E | Count | Total | Bit Offset | Notes |
|:----|:------|:-------|:------|:------|:-----------|:------|
| 0 | APP-CODE | 8 | 3 | 24 | 0 | |
| 1 | CHECKSUM | 8 | 1 | 8 | 24 | |
| 2 | VERSION | 3 | 1 | 3 | 32 | |
| 3 | COUNTRY | 1 | 1 | 1 | 35 | *v0のLOCATION-TESTがなくなり繰り上げ* |
| 4 | CAMERA-TAG | 1 | 1 | 1 | 36 | *新規* |
| 5 | REMAIN | 6 | 1 | 6 | 37 | |
| 6 | UPDATE | 1 | 1 | 1 | 43 | |
| 7 | NAME | 8 | 5 | 40 | 44 | |
| 8 | CHARA | 4 | 1 | 4 | 84 | |
| 9 | TITLE | 8 | 1 | 8 | 88 | |
| 10 | TITLE-PARAM | 3 | 1 | 3 | 96 | |
| 11 | VOLUME | 3 | 1 | 3 | 99 | |
| 12 | URA | **2** | 1 | **2** | 102 | *1→2 bitに拡張* |
| 13 | WINS | 10 | 1 | 10 | 104 | |
| 14 | LOSE | 10 | 1 | 10 | 114 | |
| 15 | GET-ITEM-LIST | 1 | **100** | **100** | 124 | *88→100に拡張* |
| 16 | LAST-GET-ITEM | 8 | 1 | 8 | 224 | |
| 17 | ITEMPACK-1 | **8** | 3 | **24** | 232 | *7→8 bitに拡張* |
| 18 | ITEMPACK-2 | **8** | 3 | **24** | 256 | *ITEMPACK-3は廃止* |
| 19 | RESULT-GP-MR | 1 | 4 | 4 | 280 | |
| 20 | RESULT-GP-DK | 1 | 4 | 4 | 284 | |
| 21 | RESULT-GP-WC | 1 | 4 | 4 | 288 | |
| 22 | RESULT-GP-PC | 1 | 4 | 4 | 292 | |
| 23 | RESULT-GP-KP | 1 | 4 | 4 | 296 | |
| 24 | RESULT-GP-RB | 1 | 4 | 4 | 300 | |
| 25 | GHOST-MR | 1 | 4 | 4 | 304 | |
| 26 | GHOST-DK | 1 | 4 | 4 | 308 | |
| 27 | GHOST-WC | 1 | 4 | 4 | 312 | |
| 28 | GHOST-PC | 1 | 4 | 4 | 316 | |
| 29 | GHOST-KP | 1 | 4 | 4 | 320 | |
| 30 | GHOST-RB | 1 | 4 | 4 | 324 | |
| 31 | WIN-GHOST-MR | 1 | 4 | 4 | 328 | |
| 32 | WIN-GHOST-DK | 1 | 4 | 4 | 332 | |
| 33 | WIN-GHOST-WC | 1 | 4 | 4 | 336 | |
| 34 | WIN-GHOST-PC | 1 | 4 | 4 | 340 | |
| 35 | WIN-GHOST-KP | 1 | 4 | 4 | 344 | |
| 36 | WIN-GHOST-RB | 1 | 4 | 4 | 348 | |
| 37 | RECORD-TA | 19 | **4** | **76** | 352 | *6→4要素* |
| 38 | RECORD-STG | 3 | **4** | **12** | 428 | *6→4要素* |
| 39 | RECORD-CRS | 1 | **4** | **4** | 440 | *6→4要素* |
| 40 | RECORD-DIR | 1 | **4** | **4** | 444 | *6→4要素* |
| 41 | RECORD-CLASS | 1 | 4 | 4 | 448 | *新規* |
| 42 | MISS-GP | 1 | 1 | 1 | 452 | |
| 43 | BATTLE-WIN-MR-TAG | 6 | 1 | 6 | 453 | |
| 44 | BATTLE-WIN-DK-TAG | 6 | 1 | 6 | 459 | |
| 45 | BATTLE-WIN-WC-TAG | 6 | 1 | 6 | 465 | |
| 46 | BATTLE-WIN-PC-TAG | 6 | 1 | 6 | 471 | |
| 47 | BATTLE-WIN-KP-TAG | 6 | 1 | 6 | 477 | |
| 48 | BATTLE-WIN-RB-TAG | 6 | 1 | 6 | 483 | |
| 49 | BATTLE-LOSE-MR-TAG | 1 | 1 | 1 | 489 | |
| 50 | BATTLE-LOSE-DK-TAG | 1 | 1 | 1 | 490 | |
| 51 | BATTLE-LOSE-WC-TAG | 1 | 1 | 1 | 491 | |
| 52 | BATTLE-LOSE-PC-TAG | 1 | 1 | 1 | 492 | |
| 53 | BATTLE-LOSE-KP-TAG | 1 | 1 | 1 | 493 | |
| 54 | BATTLE-LOSE-RB-TAG | 1 | 1 | 1 | 494 | |
| 55 | PLAYCOUNT-TAG | 6 | 1 | 6 | 495 | *新規: プレイ回数* |
| 56 | PLAY-SUNDAYCOUNT-TAG | 6 | 1 | 6 | 501 | *新規: 日曜プレイ回数* |
| 57 | ITEMPACK-LAST-TAG | 1 | 1 | 1 | 507 | *新規* |
| 58 | TA-RECORD-TA | 19 | 1 | 19 | 508 | *新規: TA記録* |
| 59 | TA-RECORD-STG | 3 | 1 | 3 | 527 | *新規* |
| 60 | TA-RECORD-CRS | 1 | 1 | 1 | 530 | *新規* |
| 61 | TA-RECORD-DIR | 1 | 1 | 1 | 531 | *新規* |
| 62 | TITLE-PARAM2 | 1 | 1 | 1 | 532 | *新規* |
| 63 | TA-RECORD-CLASS | 1 | 1 | 1 | 533 | *新規* |

---

## VERSION=2 スキーマ (0x8049b6a0, 73エントリ, 484ビット)

main.dol 内の `card_pack` (0x801d5244) で使用。v1からの変更点:
- CHARA → LAST_CHARA_TAG にリネーム
- URA 削除、LAST_KART_TAG 新設 (1bit)
- TITLE 拡張 (8→10bit)
- GET-ITEM-LIST 拡張 (100→105要素)
- LAST-GET-ITEM 縮小 (8→7bit)
- COIN_MILAGE_TAG 新設 (17bit)
- ITEMPACK-2/3 削除
- RESULT-GP 完全再構成: クラス別(50/100/150) × カップ別(8杯) × 4bit
- GHOST/WIN-GHOST 8カップ化 (YI=ヨッシー, DN=ワルイージ 追加)
- BATTLE-WIN/LOSE 全削除
- RECORD-TA/STG/CRS/DIR 削除 → 単一 TA-RECORD-* に置換
- PLAY-SUNDAYCOUNT-TAG, ITEMPACK-LAST-TAG 削除
- 新設: LAST_CHARA_TAG, LAST_KART_TAG, COIN_MILAGE_TAG, TA-RECORD-CHARA, TA-RECORD-KART, LAST-CLEAR-CUP-CLASS-TAG, LAST-CLEAR-CUP-URA-TAG, LIVE-TAG, RANKIN-TAG

| Idx | Field | Bits/E | Count | Total | Bit Offset | Notes |
|:----|:------|:-------|:------|:------|:-----------|:------|
| 0 | APP-CODE | 8 | 3 | 24 | 0 | "MKA" |
| 1 | CHECKSUM | 8 | 1 | 8 | 24 | |
| 2 | VERSION | 3 | 1 | 3 | 32 | =2 |
| 3 | COUNTRY | 1 | 1 | 1 | 35 | |
| 4 | CAMERA-TAG | 1 | 1 | 1 | 36 | |
| 5 | REMAIN | 6 | 1 | 6 | 37 | 残りクレジット |
| 6 | UPDATE | 1 | 1 | 1 | 43 | |
| 7 | NAME | 8 | 5 | 40 | 44 | 文字テーブルインデックス |
| 8 | LAST_CHARA_TAG | 4 | 1 | 4 | 84 | *旧 CHARA* |
| 9 | LAST_KART_TAG | 1 | 1 | 1 | 88 | *新規* |
| 10 | TITLE | **10** | 1 | **10** | 89 | *8→10bit拡張* |
| 11 | TITLE-PARAM | 3 | 1 | 3 | 99 | |
| 12 | VOLUME | 3 | 1 | 3 | 102 | 0=50cc?, 1=100cc, 2=150cc |
| 13 | WINS | 10 | 1 | 10 | 105 | |
| 14 | LOSE | 10 | 1 | 10 | 115 | |
| 15 | GET-ITEM-LIST | 1 | **105** | **105** | 125 | *100→105に拡張* |
| 16 | LAST-GET-ITEM | **7** | 1 | **7** | 230 | *8→7bit縮小* |
| 17 | COIN_MILAGE_TAG | **17** | 1 | **17** | 237 | *新規: コインマイレージ* |
| 18 | ITEMPACK-1 | 7 | 3 | 21 | 254 | *ITEMPACK-2/3は廃止* |
| 19 | RESULT-GP-50-MR | 4 | 1 | 4 | 275 | *50cc マリオカップ* |
| 20 | RESULT-GP-50-DK | 4 | 1 | 4 | 279 | |
| 21 | RESULT-GP-50-WC | 4 | 1 | 4 | 283 | |
| 22 | RESULT-GP-50-PC | 4 | 1 | 4 | 287 | |
| 23 | RESULT-GP-50-KP | 4 | 1 | 4 | 291 | |
| 24 | RESULT-GP-50-RB | 4 | 1 | 4 | 295 | |
| 25 | RESULT-GP-50-YI | 4 | 1 | 4 | 299 | *新規カップ* |
| 26 | RESULT-GP-50-DN | 4 | 1 | 4 | 303 | *新規カップ* |
| 27 | RESULT-GP-100-MR | 4 | 1 | 4 | 307 | *100cc マリオカップ* |
| 28 | RESULT-GP-100-DK | 4 | 1 | 4 | 311 | |
| 29 | RESULT-GP-100-WC | 4 | 1 | 4 | 315 | |
| 30 | RESULT-GP-100-PC | 4 | 1 | 4 | 319 | |
| 31 | RESULT-GP-100-KP | 4 | 1 | 4 | 323 | |
| 32 | RESULT-GP-100-RB | 4 | 1 | 4 | 327 | |
| 33 | RESULT-GP-100-YI | 4 | 1 | 4 | 331 | |
| 34 | RESULT-GP-100-DN | 4 | 1 | 4 | 335 | |
| 35 | RESULT-GP-150-MR | 4 | 1 | 4 | 339 | *150cc マリオカップ* |
| 36 | RESULT-GP-150-DK | 4 | 1 | 4 | 343 | |
| 37 | RESULT-GP-150-WC | 4 | 1 | 4 | 347 | |
| 38 | RESULT-GP-150-PC | 4 | 1 | 4 | 351 | |
| 39 | RESULT-GP-150-KP | 4 | 1 | 4 | 355 | |
| 40 | RESULT-GP-150-RB | 4 | 1 | 4 | 359 | |
| 41 | RESULT-GP-150-YI | 4 | 1 | 4 | 363 | |
| 42 | RESULT-GP-150-DN | 4 | 1 | 4 | 367 | |
| 43 | GHOST-MR | 1 | 4 | 4 | 371 | |
| 44 | GHOST-DK | 1 | 4 | 4 | 375 | |
| 45 | GHOST-WC | 1 | 4 | 4 | 379 | |
| 46 | GHOST-PC | 1 | 4 | 4 | 383 | |
| 47 | GHOST-KP | 1 | 4 | 4 | 387 | |
| 48 | GHOST-RB | 1 | 4 | 4 | 391 | |
| 49 | GHOST-YI | 1 | 4 | 4 | 395 | *新規カップ (ヨッシー)* |
| 50 | GHOST-DN | 1 | 4 | 4 | 399 | *新規カップ (ワルイージ)* |
| 51 | WIN-GHOST-MR | 1 | 4 | 4 | 403 | |
| 52 | WIN-GHOST-DK | 1 | 4 | 4 | 407 | |
| 53 | WIN-GHOST-WC | 1 | 4 | 4 | 411 | |
| 54 | WIN-GHOST-PC | 1 | 4 | 4 | 415 | |
| 55 | WIN-GHOST-KP | 1 | 4 | 4 | 419 | |
| 56 | WIN-GHOST-RB | 1 | 4 | 4 | 423 | |
| 57 | WIN-GHOST-YI | 1 | 4 | 4 | 427 | *新規カップ (ヨッシー)* |
| 58 | WIN-GHOST-DN | 1 | 4 | 4 | 431 | *新規カップ (ワルイージ)* |
| 59 | MISS-GP | 1 | 1 | 1 | 435 | |
| 60 | PLAYCOUNT-TAG | 6 | 1 | 6 | 436 | プレイ回数 |
| 61 | TA-RECORD-TA | 19 | 1 | 19 | 442 | TAベストタイム |
| 62 | TA-RECORD-STG | 3 | 1 | 3 | 461 | TAベストステージ |
| 63 | TA-RECORD-CRS | 1 | 1 | 1 | 464 | |
| 64 | TA-RECORD-DIR | 1 | 1 | 1 | 465 | |
| 65 | TITLE-PARAM2 | 1 | 1 | 1 | 466 | |
| 66 | TA-RECORD-CLASS | 2 | 1 | 2 | 467 | |
| 67 | TA-RECORD-CHARA | 4 | 1 | 4 | 469 | *新規: TAベストキャラ* |
| 68 | TA-RECORD-KART | 2 | 1 | 2 | 473 | *新規: TAベストカート* |
| 69 | LAST-CLEAR-CUP-CLASS-TAG | 2 | 1 | 2 | 475 | *新規: 最後にクリアしたCCクラス* |
| 70 | LAST-CLEAR-CUP-URA-TAG | 1 | 1 | 1 | 477 | *新規: 最後にクリアしたのが裏か* |
| 71 | LIVE-TAG | 1 | 1 | 1 | 478 | *新規: カード有効フラグ* |
| 72 | RANKIN-TAG | 1 | 5 | 5 | 479 | *新規: ランキング関連* |

合計: 484ビット (60.5バイト)。69バイト中 残り68ビット (8.5バイト) は未使用パディング。

---

## サンプルカードデータのパース結果 (v2スキーマ)

ピーチで「あいうえお」、150cc ドンキーカップ1ラウンドクリア、マリオ専用車解放済み。

```
4D 4B 41 F9 4E 24 24 34 44 54 63 38 09 00 00 04
00 00 02 00 00 00 00 00 00 00 00 00 01 20 00 A3
FF FF E0 00 00 00 00 00 00 00 00 20 00 00 00 00
00 00 00 00 00 00 00 40 00 00 00 12 00 00 00 00
00 00 00 00 00
```

| Field | Value | 検証 |
|:------|:------|:-----|
| APP-CODE | [0x4D,0x4B,0x41] | "MKA" ✓ |
| CHECKSUM | 0xF9 (249) | 全byte(checksum=0)合計 mod 256 = 0xF9 ✓ |
| VERSION | 2 | ✓ |
| COUNTRY | 0 | |
| CAMERA-TAG | 1 | |
| REMAIN | 49 (0x31) | 残りクレジット |
| UPDATE | 0 | |
| NAME | [0x42,0x43,0x44,0x45,0x46] | →"あいうえお" ✓ |
| LAST_CHARA_TAG | 3 | ピーチ ✓ |
| LAST_KART_TAG | 0 | |
| TITLE | 448 | 10bit称号ID |
| TITLE-PARAM | 2 | |
| VOLUME | 2 | 150cc ✓ |
| WINS | 0 | GP未完走のため ✓ |
| LOSE | 0 | ✓ |
| GET-ITEM-LIST | bit 0, 25 がON | bit 0 = マリオ専用車? |
| LAST-GET-ITEM | 36 | |
| COIN_MILAGE_TAG | 40 | 獲得コイン数? |
| ITEMPACK-1 | [127, 127, 127] | 7bit全ON=デフォルト? |
| RESULT-GP-150-DK | **1** | **DKカップ150cc ラウンド1クリア ✓** |
| RESULT-GP-* (他) | 全て0 | 他カップ未プレイ ✓ |
| GHOST/WIN-GHOST | 全て[0,0,0,0] | |
| MISS-GP | 0 | |
| PLAYCOUNT-TAG | 1 | 1プレイ ✓ |
| TA-RECORD-* | 全て0 | TAモード未プレイ |
| LAST-CLEAR-CUP-CLASS-TAG | 2 | 150cc ✓ |
| LAST-CLEAR-CUP-URA-TAG | 0 | 裏ではない ✓ |
| LIVE-TAG | 1 | カード有効 |
| RANKIN-TAG | [0,0,0,0,0] | |

### RESULT-GP フィールドの解釈

4bit値。各カップは4ラウンドで構成されるため、各bitが1ラウンドのクリア状態を表す:
- bit 0 = ラウンド1クリア
- bit 1 = ラウンド2クリア
- bit 2 = ラウンド3クリア
- bit 3 = ラウンド4クリア (全クリア)

RESULT-GP-150-DK = 1 (0b0001) = ラウンド1のみクリア → ユーザー情報と一致。

## キャラクターID一覧

FUN_80044d50 (キャラクターセットアップ関数) のポインタテーブル PTR_s_chara_mario_r_hand_null_joint_803fec24 (13エントリ) から確定:

| ID | Internal | Character (JP) | Character (EN) |
|:---|:---------|:---------------|:---------------|
| 0 | mario | マリオ | Mario |
| 1 | luigi | ルイージ | Luigi |
| 2 | wario | ワリオ | Wario |
| 3 | peach | ピーチ | Peach |
| 4 | koopa | クッパ | Bowser |
| 5 | kinopio | キノピオ | Toad |
| 6 | donkey | ドンキーコング | Donkey Kong |
| 7 | yoshi | ヨッシー | Yoshi |
| 8 | pacman | パックマン | Pac-Man |
| 9 | mspacman | ミズ・パックマン | Ms. Pac-Man |
| 10 | monster | アカベイ | Blinky |
| 11 | waluigi | ワルイージ | Waluigi |
| 12 | mametch | まめっち | Mametchi |

**注**: LAST_CHARA_TAG は4bit (0-15) だが有効値は0-12 (13キャラ)。card_pack でも `0xc < iVar22` で範囲チェック。

## カップ一覧

RESULT-GP / GHOST / WIN-GHOST サフィックスから確定 (8カップ)。
**注**: GHOST/WIN-GHOSTのサフィックスはキャラクターではなくカップ (コース) を表す。

| Index | Code | courseId | Cup (JP) | Cup (EN) | 備考 |
|:------|:-----|:--------|:---------|:---------|:-----|
| 0 | MR | 1 | マリオカップ | Mario Cup | |
| 1 | DK | 2 | DKカップ | DK Cup | サンプルカード検証済み |
| 2 | WC | 3 | ワリオカップ | Wario Cup | 確定: TitleID 17 で検証 |
| 3 | PC | 4 | パックマンカップ | Pac-Man Cup | |
| 4 | KP | 5 | クッパカップ | Bowser Cup | KP = クッパ (KuPpa) |
| 5 | RB | 6 | レインボーカップ | Rainbow Cup | |
| 6 | YI | 7 | ヨッシーカップ | Yoshi Cup | v2で追加 |
| 7 | DN | 8 | ワルイージカップ | Waluigi Cup | 確定: TitleID 15 で検証。DN = 開発初期コード名の名残 |

## 名前エンコーディング

NAME フィールドの各バイトは文字テーブルのインデックス。`FUN_801f8d58` でワイド文字に変換される。
- 0x42 → あ, 0x43 → い, 0x44 → う, 0x45 → え, 0x46 → お (連番)
- テーブル全体の構成は FUN_801f8d58 の解析が必要

## ビットパック読み取りアルゴリズム

`FUN_80088880` のロジック:
1. スキーマテーブルをフィールド名で線形検索
2. `startBit = bitOffset + (elementIndex + 1) * bitsPerElement`
3. startBit から逆順に1ビットずつ読み取り:
   - byte index = `bit / 8`
   - bit-in-byte = `7 - (bit % 8)` (MSBファースト)
4. 結果をビッグエンディアンで u32 に組み立て
5. `(1 << bitsPerElement) - 1` でマスク
