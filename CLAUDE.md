# MKGP2 View - 作業ガイド

Dolphinエミュレータのメモリを読み取り、MKGP2のゲーム状態をリアルタイム表示するeguiビューワー。

## 技術スタック

- Rust + egui (eframe)
- dolphin-memory crate: DolphinプロセスのMEM1をReadProcessMemoryで読む
- PPCアドレス → MEM1オフセット変換: `addr & 0x01FFFFFF`

## MKGP2 メモリマップ

ゲーム内構造体やアドレスの詳細は dolphin リポジトリの `mkgp2docs/` を参照。
主要アドレスは `src/dolphin.rs` の `addr` モジュールに集約。

## 注意事項

- dolphin-memory crateは3年以上メンテされていないが、原理的には現行Dolphinで動作する
- PPCはビッグエンディアン。dolphin-memory crateが自動変換する
- ビルドは `cargo build` / `cargo run`
