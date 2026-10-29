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
- `nix fmt` は Markdown も整形する（番号付きリストは `1.` にそろう）。整形の差分も PR に含める。

## CI（GitHub Actions）

| ワークフロー               | ジョブ                                   | 内容                                                                                                        |
| -------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `ci.yml`                   | Format / tokens                          | fmt と `xtask check-tokens`（ubuntu）                                                                       |
|                            | Check (ubuntu / macos-15 / windows-2025) | clippy と test                                                                                              |
|                            | Build (wasm32)                           | md3-components と gallery の wasm ビルド                                                                    |
|                            | CI                                       | 上の全ジョブの集約。ブランチ保護の必須チェックにはこれだけを指定する想定（2026-10-29 時点で main は未保護） |
| `gallery.yml`              | Desktop (3 OS) / Web                     | gallery のリリースビルドを Artifacts に置く（動作確認用、保存 3 日）                                        |
| `dependabot-automerge.yml` |                                          | Dependabot の PR の自動マージ                                                                               |

- 「Pin toolchain」の手順で `RUSTUP_TOOLCHAIN` を `rust-toolchain.toml` の版に固定している（消さない。消すと毎回ツールチェーンを落とし直す）。
- キャッシュは `Swatinem/rust-cache`（`moonrepo/setup-rust` は `cache: false`）。ジョブごとに `shared-key` を分け、
  保存は main だけ（`save-if`）。PR は main のキャッシュを読む。Windows / macOS でもヒットを確認済み（2026-10-29）。
- 外部 action は commit SHA で固定する。

## PR の進め方

- 1 つの作業のまとまり（部品 1 つ、修正のまとまり）= 1 ブランチ = 1 PR。main に直接 push しない。
- CI が緑になったら squash merge する（人間の確認は待たない運用）。ただし v1.0 のタグ付けと Web 版の公開は人間に確認する。
- コミットの末尾: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`。
  PR 本文の末尾: `🤖 Generated with [Claude Code](https://claude.com/claude-code)`。
- 手順は skill `ship-pr`（`.agents/skills/ship-pr/`）。

## Codex との分担

計画とレビュー・確認は Claude、実装は Codex に任せる運用をしている（skill `codex-delegate`）。

- モデルは `gpt-6.1-sol`、reasoning effort は `xhigh`:
  `codex exec -m gpt-6.1-sol -c model_reasoning_effort='"xhigh"' -s workspace-write -C . -`
  （review は `-m` を取らないので `-c model='"gpt-6.1-sol"'`）。
- Codex の sandbox からは Nix daemon に繋がらない。**`nix develop -c` の中で codex を起動する**と、
  Codex が cargo を直接実行できる。プロンプトにも「nix develop を呼ばず cargo を直接使う」と書く。
- sandbox からは Wayland に繋がらず、Codex は gallery を目視できない。画面の確認は Claude が行う（skill `gallery-check`）。
- 実装後は `codex review --uncommitted` でレビューさせ、指摘を Codex に直させて再レビューする。
- Codex には使用量の上限がある。上限に達したら、回復時刻まで待つか、残りを Claude が行い PR に明記する。

## 画面の確認（Linux / WSL）

- gallery は X11 で起動して xdotool で操作し、xwd + ImageMagick で撮る。手順とスクリプトは skill `gallery-check`。
- 道具は nixpkgs から一時的に取る: `nix shell nixpkgs#xdotool nixpkgs#xwd nixpkgs#imagemagick`。
- lavapipe（CPU 描画）は遅いので、フェードなど時間で変わる表示は操作の直後に撮る。
- a11y ツリーは `nix develop -c tools/a11y-tree/run.sh <出力先> [ページ...]`（第 1 引数は出力先）。
- `pkill -f` はコマンド行に同じ文字列を含む自分のシェルも殺す。gallery は `pkill -x gallery` で止める。
