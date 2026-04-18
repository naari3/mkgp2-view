# MKGP2 View - 作業ガイド

Dolphinエミュレータのメモリを読み取り、MKGP2のゲーム状態をリアルタイム表示するeguiビューワー。

## 技術スタック

- Rust + egui (eframe)
- dolphin-memory crate: DolphinプロセスのMEM1をReadProcessMemoryで読む
- PPCアドレス → MEM1オフセット変換: `addr & 0x01FFFFFF`

## バイナリ

2つのバイナリを持つ:
- **mkgp2-view**: egui GUIビューワー
- **mkgp2-mcp**: stdio JSON-RPC MCPサーバー（Claude Codeからメモリ読み取り）

### ビルド

```bash
cargo build                    # 両方ビルド
cargo build --bin mkgp2-view   # viewだけ（MCP exe lockを避ける）
cargo build --bin mkgp2-mcp    # MCPだけ
cargo run --bin mkgp2-view     # view起動
```

Dolphinが起動していれば自動接続する。

### MCPサーバーのデプロイ

`.mcp.json` → `bin/run-mcp.bat` → `target/debug/mkgp2-mcp.exe` を TEMP にコピーして実行。
`target/debug/` はロックされないので、セッション中でも `cargo build --bin mkgp2-mcp` が通る。

**更新手順**: `cargo build --bin mkgp2-mcp` → Claude Code再起動（新バイナリが自動反映）

view起動中�� `cargo build` (全体) すると view の exe lock で失敗する場合は `--bin mkgp2-mcp` で MCP だけビルドする。

## MKGP2 メモリマップ

ゲーム内構造体やアドレスの詳細は dolphin リポジトリの `mkgp2docs/` を参照。
主要アドレスは `src/dolphin.rs` の `addr` モジュールに集約。

## 注意事項

- dolphin-memory crateは3年以上メンテされていないが、原理的には現行Dolphinで動作する
- PPCはビッグエンディアン。dolphin-memory crateが自動変換する
- アドレスやオフセットを追加する際は、必ず `mkgp2docs/` の確定済み情報を参照すること。推測で値を入れない

---

## ワークフロー設計

### 1. 偵察してから動く

- コードを書く前に、変更対象とその周辺を必ず Read/Grep/Glob で確認する
- 読んでいないファイルを変更しない。これは絶対のルール
- 偵察結果の報告や承認待ちは不要。把握できたらそのまま実装に入る
- 途中で想定外の構造に遭遇したら、書きかけのコードを放置してでも追加の偵察を行う
- 曖昧さはコードを読んで解消する。ユーザーに聞くのは、コードから判断できない場合の最終手段

### 2. サブエージェント戦略

- メインのコンテキストウィンドウをクリーンに保つためにサブエージェントを積極的に活用する
- リサーチ・調査・並列分析はサブエージェントに任せる
- 集中して実行するために、サブエージェント1つにつき1タスクを割り当てる

### 3. 完了前に必ず検証する

- `cargo build` が通ることを確認してからコミットする
- ビューワーが起動中だと `cargo build` がexe削除できずに失敗する。その場合はコミットだけしてユーザーに再起動を依頼する

### 4. 自律的なバグ修正

- バグレポートを受けたら、手取り足取り教えてもらわずにそのまま修正する
- ユーザーのコンテキスト切り替えをゼロにする

---

## Code Intelligence

コードナビゲーションには Grep/Read より LSP を優先する:
- `workspaceSymbol` で定義箇所を検索
- `findReferences` でコードベース全体の使用箇所を確認
- `goToDefinition` / `goToImplementation` でソースにジャンプ
- `hover` でファイルを開かずに型情報を取得

Grep は LSP が使えない場合やテキスト/パターン検索にのみ使用する。

---

## コア原則

- **シンプル第一**: すべての変更をできる限りシンプルにする。影響するコードを最小限にする。
- **手を抜かない**: 根本原因を見つける。一時的な修正は避ける。
- **影響を最小化する**: 変更は必要な箇所のみにとどめる。
- **mkgp2docsを信頼する**: アドレスやオフセットは過去の解析で確定したドキュメントに基づく。新しい値を追加する場合はGhidraで確認してからにする。
