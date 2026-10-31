# 開発環境・CI・作業の進め方

## 開発環境（Nix）

- `nix develop`（または direnv。`.envrc` は `use flake` だけにする。ほかの設定を足さない）。
- ツールチェーンは `rust-toolchain.toml`（上流 zed の固定 rev と同じ版）。CI もこの版を使う。
- devshell（`nix/modules/devshells.nix`）が用意するもの:
  - `MD3_WEB_FONTS`（`cargo xtask web` が wasm 版に添えるフォント。Linux / Darwin）
  - wasm-bindgen CLI（Cargo.lock と同じ版）
  - Linux: `LD_LIBRARY_PATH`（wayland / vulkan / xkbcommon）、mesa lavapipe（GPU が無い環境の CPU 描画）、
    フォント（`FONTCONFIG_FILE`）
- コマンドは `nix develop -c <cmd>` で実行する。外で gallery を起動するとフォントが見つからず panic することがある。
- `Cargo.toml` の profile や toolchain の固定は変えない。
- `nix build`（crane）のソースは `nix/build.nix` の `lib.fileset` で決まる。Cargo のソース以外は含まれないので、
  `include_bytes!` / `include_str!` で埋め込むファイルを新しい場所に足したら、そこにも足す
  （今は `crates/md3-icons/assets` と `xtask/web`）。足し忘れると cargo では通っても `nix build` だけ落ちる。

## 完了チェック（全 PR）

```sh
nix fmt                     # rustfmt / mdformat / taplo / nixfmt（treefmt）で整形する
nix flake check             # 整形の確認、clippy（-D warnings）、test、release ビルド、cargo audit / deny
nix develop -c bash -c '
  cargo build -p md3-components --target wasm32-unknown-unknown &&
  cargo xtask check-tokens'  # flake check に入っていないもの
```

- `nix flake check` は crane のビルドなので時間がかかる。作業中の確認は `nix develop -c cargo clippy ...` / `cargo test -p <crate>` など cargo を直接使ってよい。
- `nix flake check` は git が追跡するファイルだけを見る。新しいファイルは `git add` してから回す（未追跡だとビルドが落ちる）。
- `nix fmt` は Markdown も整形する（番号付きリストは `1.` にそろう）。整形の差分も PR に含める。

## CI（GitHub Actions）

| ワークフロー               | ジョブ                                   | 内容                                                                                                                                                 |
| -------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ci.yml`                   | Format / tokens                          | fmt と `xtask check-tokens`（ubuntu）                                                                                                                |
|                            | Check (ubuntu / macos-15 / windows-2025) | clippy と test                                                                                                                                       |
|                            | Build (wasm32)                           | md3-components と gallery の wasm ビルド                                                                                                             |
|                            | CI                                       | 上の全ジョブの集約。ブランチ保護の必須チェックにはこれだけを指定する想定（2026-10-29 時点で main は未保護）                                          |
| `gallery.yml`              | Desktop (3 OS) / Web                     | gallery のリリースビルドを Artifacts に置く（動作確認用、保存 3 日）。main への push では Web 版を GitHub Pages に公開する（Deploy to GitHub Pages） |
| `dependabot-automerge.yml` |                                          | Dependabot の PR の自動マージ                                                                                                                        |

- 「Pin toolchain」の手順で `RUSTUP_TOOLCHAIN` を `rust-toolchain.toml` の版に固定している（消さない。消すと毎回ツールチェーンを落とし直す）。
- キャッシュは `Swatinem/rust-cache`（`moonrepo/setup-rust` は `cache: false`）。ジョブごとに `shared-key` を分け、
  保存は main だけ（`save-if`）。PR は main のキャッシュを読む。Windows / macOS でもヒットを確認済み（2026-10-29）。
- 外部 action は commit SHA で固定する。

## PR の進め方

- 1 つの作業のまとまり（部品 1 つ、修正のまとまり）= 1 ブランチ = 1 PR。main に直接 push しない。
- レビューが終わり、CI が緑になったら squash merge する（人間の確認は待たない運用）。ただし v1.0 のタグ付けは人間に確認する（Web 版は main へのマージで自動的に GitHub Pages に出る）。
- 手順は skill `ship-pr`。

## Codex との分担

計画とレビュー・確認は Claude、実装は Codex に任せる運用をしている（skill `codex-delegate`）。

- モデルとコマンドは skill `codex-delegate`。
- Codex の sandbox からは Nix daemon に繋がらない。**`nix develop -c` の中で codex を起動する**と、
  Codex が cargo を直接実行できる。プロンプトにも「nix develop を呼ばず cargo を直接使う」と書く。
- sandbox からは Wayland に繋がらず、Codex は gallery を目視できない。画面の確認は Claude が行う（skill `gallery-check`）。
- 実装後は Codex にレビューさせ、指摘を Codex に直させて再レビューする。**レビューの完了を待ってからマージする**（skill `ship-pr`）。
- Codex には使用量の上限がある。上限に達したら、回復時刻まで待つか、残りを Claude が行い PR に明記する。ユーザーが待たないと言ったら Claude が実装する。レビューはサブエージェントに任せる。

## gallery のページを追加する

`gallery/src/main.rs` の `Page` と `PAGES` に追加し、render の `match` とテスト内の全 variant の `match` を更新する。
ビルド後のページキーの一覧は `MD3_GALLERY_LIST_PAGES=1 target/debug/gallery` で確認できる。

## 画面の確認

手順とスクリプトは skill `gallery-check`。
