---
name: codex-delegate
description: 実装を Codex CLI（gpt-6.1-sol / xhigh）に任せる。Claude が計画とプロンプトを書き、nix develop の中で codex exec を走らせ、結果を検証して codex review で指摘を出させ、Codex に直させる。ユーザーが「実装は codex に」と言ったとき、まとまった実装・修正を委ねるときに使う。
---

# Codex に実装を任せる

分担と前提は `docs/agents/dev-environment.md` の「Codex との分担」。Claude は計画・プロンプト・検証・画面確認・PR、Codex は実装と修正。

## 1. プロンプトを作る

scratchpad に `prompt.md` を作る:

1. `.agents/skills/codex-delegate/templates/common.md` をコピーし、`{{COMPOSE_DIR}}` を Compose のソースの場所に置き換える
   （無ければ「not checked out; use https://github.com/androidx/androidx at the commit recorded in docs/TOKENS_SOURCE.md」）。
1. 末尾に `## Task: ...` を足す。書くもの:
   - ユーザーの症状・要望（原文の意図）と、Claude が調べた原因・対象ファイルと行
   - 設計の方針（公開 API の形、拡張点、既定値）。Claude の判断として決めておく。Codex に選ばせない
   - 一次資料の場所と、使う値の見込み（Codex に開いて確かめさせる）
   - テストと gallery の見本に求めること
     1 PR に入る量にする。互いに同じファイルを触る作業は並行させず順番に回す。

## 2. 実行する

作業ブランチで（main で走らせない）、バックグラウンドで:

```sh
nix develop -c bash -c "timeout 7200 codex exec -m gpt-6.1-sol -c model_reasoning_effort='\"xhigh\"' -s workspace-write -C . - < $SCRATCH/prompt.md" > $SCRATCH/codex.log 2>&1
```

- 完了したら `awk '/^codex$/{p=1;buf=""} p{buf=buf"\n"$0} END{print buf}' codex.log | tail -40` で最後の報告を読み、`git status` を見る。
- ログに `usage limit` が出たら、回復時刻まで待つ（ユーザーに伝える）か、残りを Claude が行って PR に明記する。
  途中で止まった場合も、作業ツリーの変更とテストは生きていることが多い。完了チェックで確かめる。

## 3. 検証する（Codex の報告をそのまま信じない）

1. 完了チェック（skill `ship-pr` の 2。`nix fmt` / `nix flake check` を含む）を Claude が自分で回す。Codex は sandbox から Nix を使えないので cargo でしか確かめていない。
1. 差分を読む: 依存の追加・`unsafe`・推測の値・規約違反・不要な変更がないか。
1. 画面で確かめる（skill `gallery-check`）。ユーザーの所見があれば、その操作を再現する。

## 4. レビューと修正

```sh
nix develop -c bash -c "timeout 3000 codex review -c model='\"gpt-6.1-sol\"' -c model_reasoning_effort='\"xhigh\"' --uncommitted" > $SCRATCH/review.txt 2>&1
```

- 指摘（[P1] / [P2] など）があれば、指摘の本文をそのまま貼った修正用プロンプト（common.md + `## Task: fix review findings`）で
  2 を回し、3 と 4 をやり直す。妥当でない指摘は理由を PR に書いて退ける。
- 指摘がなくなったら skill `ship-pr` で出す。PR の「確認」にレビューの回数と直した指摘を書く。

## 注意

- `pkill -f "codex ..."` は自分のシェルにも当たる。止めるときは `pkill -f "codex revie[w]"` のように正規表現で自分を外す。
- 長い待ちを `run_in_background` の Bash に `timeout` 付きで仕込むと、Bash ツールの上限で殺されることがある。
  時刻を待つなら `setsid nohup <script> &` で切り離す。
