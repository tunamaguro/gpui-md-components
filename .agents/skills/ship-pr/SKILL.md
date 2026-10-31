---
name: ship-pr
description: 作業ツリーの変更を完了チェックし、Codex かサブエージェントのレビューの完了を待ち、コミットして PR を作り、CI が緑になったら squash merge して main を更新する。1 つの作業のまとまりを出すときに使う。
---

# PR を出してマージする

CI の中身は `docs/agents/dev-environment.md` の「CI」。

## 1. ブランチ

- main にいるなら、`git fetch && git checkout main && git pull` のあと、作業の内容が分かる名前でブランチを切る。main に直接コミットしない。

## 2. 完了チェック

`docs/agents/dev-environment.md` の「完了チェック」を回す。`nix fmt` の整形の差分もコミットに含める。
どれかが落ちたら直してからやり直す（落ちた derivation のログは `nix log <drv>`）。

## 3. レビューの完了を待つ

**レビューが終わるまでマージしない。完了の報告もしない。** ユーザーに「マージして」と言われたときにレビューが動いていても、終わるまで待つ。先に進めるのは次のどちらかのときだけ。

- ユーザーがレビューを待たないと明示した。
- 使用量の制限で Codex もサブエージェントも使えない。PR にそう書き、ユーザーに伝える。

手順:

1. Codex のレビューを回す（skill `codex-delegate` の 4）。
1. Codex が使用量の制限で使えないときは、サブエージェント（Agent ツールの `general-purpose`）にレビューさせる。差分、見てほしい点、`AGENTS.md` の規約を渡す。作業ツリーを変えさせず、検証用のコピーと `CARGO_TARGET_DIR` は scratchpad に置かせる。測定で確かめるよう頼むと、読むだけでは見つからない退行が出る。
1. 指摘を扱う（skill `codex-delegate` の 4）。

**レビューは、マージする内容そのものに対して行う。** レビューのあとでコードを変えたら、1 行でも回し直す。Claude が直した分も同じ。指摘を直した分だけに絞ったレビューでよい。「再レビューしていない」と PR に書いても、この条件は満たさない。コメントと文書だけの変更なら回し直さなくてよい。

レビューは完了チェックや CI と並行してよい。報告は全部が終わってからにする。

## 4. コミットと PR

- コミットメッセージは日本語。1 行目に要約、本文に変更点の箇条書き。末尾に
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`。
- `git push -u origin <branch>` のあと `gh pr create --base main`。本文に書くもの:
  - 何を・なぜ（ユーザーの所見、`docs/agents/backlog.md` の項目など。片付けた項目は backlog.md から消す）
  - `## 確認`: 通したチェック、gallery で確かめたこと（skill `gallery-check`）、レビューの回数と結果、
    確認していないこと（Windows / macOS 実機、トラックパッドなど）
  - 誰が実装し、誰がレビューしたか
  - 末尾に `🤖 Generated with [Claude Code](https://claude.com/claude-code)`

## 5. CI を待ってマージ

バックグラウンドで待つ（完了の通知で戻ってくる）:

```sh
sleep 20; until s=$(gh pr checks <番号> 2>&1) && echo "$s" | grep -q 'Check (windows' && ! echo "$s" | grep -q pending; do sleep 30; done; echo "$s" | cut -f1,2
```

- 全部 pass（dependabot の skipping は無視）で、3 のレビューも終わっていれば `gh pr merge <番号> --squash --delete-branch`、
  そのあと `git checkout main && git pull`。
- 落ちたら `gh run view --job <job id> --log-failed` で原因を見て直し、push し直す。
- 人間の確認は待たずにマージしてよい。ただし、ユーザーが試す・確認すると言った PR は、マージの指示があるまで止める。
- v1.0 のタグ付けはしない（人間に確認する）。main にマージすると Web 版の見本が GitHub Pages に公開される。
