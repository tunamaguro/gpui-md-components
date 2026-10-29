---
name: ship-pr
description: 作業ツリーの変更を完了チェック（fmt / clippy / test / wasm / check-tokens）してコミットし、PR を作り、CI が緑になったら squash merge して main を更新する。1 つの作業のまとまりを出すときに使う。
---

# PR を出してマージする

運用の背景は `docs/agents/dev-environment.md` の「PR の進め方」「CI」。

## 1. ブランチ

- main にいるなら、作業の内容が分かる名前でブランチを切る（`git checkout -b <name>`）。main に直接コミットしない。
- 手元の main が古いと後で衝突するので、切る前に `git fetch && git checkout main && git pull`。

## 2. 完了チェック

```sh
nix develop -c bash -c '
  cargo fmt --all --check &&
  cargo clippy --workspace --all-targets -- -D warnings &&
  cargo test --workspace &&
  cargo build -p md3-components --target wasm32-unknown-unknown &&
  cargo xtask check-tokens'
```

fmt の差分は `nix develop -c cargo fmt --all` で直す。どれかが落ちたら直してからやり直す。

## 3. コミットと PR

- コミットメッセージは日本語。1 行目に要約、本文に変更点の箇条書き。末尾に
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`。
- `git push -u origin <branch>` のあと `gh pr create --base main`。本文に書くもの:
  - 何を・なぜ（対応するマイルストーン、所見、handoff.md の項目）
  - `## 確認`: 通したチェック、gallery で確かめたこと（skill `gallery-check`）、`codex review` の結果、
    確認していないこと（Windows / macOS 実機、トラックパッドなど）
  - 誰が実装したか（Codex / Claude）。Codex のレビューを回していなければそう書く
  - 末尾に `🤖 Generated with [Claude Code](https://claude.com/claude-code)`

## 4. CI を待ってマージ

バックグラウンドで待つ（完了の通知で戻ってくる）:

```sh
sleep 20; until s=$(gh pr checks <番号> 2>&1) && echo "$s" | grep -q 'Check (windows' && ! echo "$s" | grep -q pending; do sleep 30; done; echo "$s" | cut -f1,2
```

- 全部 pass（dependabot の skipping は無視）なら `gh pr merge <番号> --squash --delete-branch`、
  そのあと `git checkout main && git pull`。
- 落ちたら `gh run view --job <job id> --log-failed` で原因を見て直し、push し直す。
- 人間の確認を待たずにマージしてよい。ただし v1.0 のタグ付けと Web 版の公開はしない（人間に確認する）。
