# md-components — Claude Code 向け作業指示

GPUI（Zed の UI フレームワーク）上に Material Design 3（ベースライン）を移植するワークスペース。
対象プラットフォームは Windows / macOS / Linux / Web（wasm32）。

## 最初に読むもの

1. `docs/agents/overview.md` — 目的、選定理由、スコープ（何をやらないか）
2. `docs/agents/architecture.md` — crate 構成、依存ルール、Rust の設計規約
3. `docs/agents/milestones.md` — M0〜M8 の成果物と完了条件。**作業は必ずどれかのマイルストーンに紐づける**
4. `docs/agents/resources.md` — 仕様・トークン値・アルゴリズムの「正」の場所とライセンス
5. `docs/agents/a11y.md` — アクセシビリティの実装規約と検証手順
6. `docs/agents/component-workflow.md` — コンポーネント1つを実装する手順（毎回これに従う）

## 絶対に守るルール

- **依存の追加は禁止。** 使ってよい外部 crate は `gpui`（上流 Zed リポジトリの git rev 固定）と、`gallery` / `examples` のバイナリだけ `gpui_platform` と `wasm-bindgen`（wasm32 ターゲットのみ、エントリポイント用）。dev-dependencies も原則なし。どうしても必要だと判断したら実装を止めて人間に相談する。
- `build.rs` は作らない。コード生成が必要なら `xtask`（std のみ）で生成し、生成物をコミットする。
- 全 crate に `#![forbid(unsafe_code)]`。`md3-color` では `unwrap` / `expect` / `panic!` 禁止（`clippy::unwrap_used`, `clippy::expect_used` を deny）。
- 描画経路で `std::thread`、ブロッキング、`Mutex` を使わない（wasm32 で動かなくなる）。
- トークン値・寸法を推測で書かない。必ず `docs/agents/resources.md` の一次資料から転記し、出典を `docs/TOKENS_SOURCE.md` に追記する。
- 移植・転記した Apache 2.0 のコードやデータには、ファイル先頭に出典コメント（移植元・コミット・著作権表示）を書く。`NOTICE` ファイルは置かない。
- 公開 crate 名・型名に "Material" の語を使わない（Google の商標）。`md3` を使う。
- フォントは同梱しない。既定はシステムフォント、利用者が `Theme` で指定する。
- 操作できる要素は `ElementId` を必須引数にし、a11y の役割・名前・状態を必ず付ける（`docs/agents/a11y.md`）。

## 完了の定義（全 PR 共通）

- `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` が通る
- `cargo build -p md3-components --target wasm32-unknown-unknown` が通る
- 新規コンポーネントは gallery に全状態（enabled / hover / focused / pressed / disabled、該当すれば selected / error）の見本がある
- 公開 API に doc コメントがある（`#![deny(missing_docs)]`）

## よく使うコマンド

```sh
cargo run -p gallery                     # デスクトップでギャラリー起動
cargo xtask web                          # wasm 版ギャラリーをビルド（M7 で整備）
cargo test -p md3-color                  # 配色エンジンのテスト（GPU 不要）
cargo xtask check-tokens                 # TOKENS_SOURCE.md と定数の突き合わせ（M2 で整備）
```

（`xtask` のサブコマンドは M0 で作る。存在しなければ M0 のタスクとして作成すること。）

## 不明点があるとき

GPUI の API 名・シグネチャは版で変わる。推測で書かず、固定している rev の `crates/gpui/src` と `crates/gpui/examples` を grep して確認する。
このドキュメントのコード例の GPUI API 名（`.role()` `.aria_label()` など）も「要確認」扱い。
