---
name: codex-delegate
description: 実装を Codex CLI に任せる（exec は gpt-6.1-terra、review は gpt-6.1-sol、どちらも xhigh）。Claude が計画とプロンプトを書き、nix develop の中で codex exec を走らせ、結果を検証して codex review で指摘を出させ、Codex に直させる。ユーザーが「実装は codex に」と言ったとき、まとまった実装・修正を委ねるときに使う。
---

# Codex に実装を任せる

分担は `docs/agents/dev-environment.md` の「Codex との分担」。Claude は計画・プロンプト・検証・画面確認・PR、Codex は実装と修正。

## 1. プロンプトを作る

scratchpad に `prompt.md` を作る:

1. `.agents/skills/codex-delegate/templates/common.md` をコピーする。
1. 末尾に `## Task: ...` を足す。書くもの:
   - ユーザーの症状・要望（原文の意図）と、Claude が調べた原因・対象ファイルと行
   - 設計の方針（公開 API の形、拡張点、既定値）。Claude の判断として決めておく。Codex に選ばせない
   - 一次資料の場所と、使う値の見込み（Codex に開いて確かめさせる）
   - テストと gallery の見本に求めること

1 PR に入る量にする。互いに同じファイルを触る作業は並行させず順番に回す。

アクセシビリティの問題を直すときは、2 段階に分ける:

1. テストだけを足すプロンプトで回す（修正は書かせない）。Claude が手元でテストを実行し、その問題が原因で失敗することを確かめる。
1. 失敗を確かめてから、修正のプロンプトを回す。テストを弱めさせない。

## 2. 実行する

作業ブランチで（main で走らせない）。**Bash ツールの `run_in_background` に、このコマンドをそのまま渡す。**
`&` や `nohup` で切り離すと、Codex が動いている間に完了の通知が来てしまう。

```sh
nix develop -c bash -c "timeout 3000 codex exec -m gpt-6.1-terra -c model_reasoning_effort='\"xhigh\"' -s workspace-write -C . - < $SCRATCH/prompt.md" > $SCRATCH/codex.log 2>&1
```

- Bash ツールの `timeout` は、コマンドの `timeout` より長くする。短いと途中で殺される。
- 完了したら `awk '/^codex$/{p=1;buf=""} p{buf=buf"\n"$0} END{print buf}' codex.log | tail -40` で最後の報告を読み、`git status` を見る。
- ログに `usage limit` が出たら、回復時刻まで待つ（ユーザーに伝える）か、残りを Claude が行って PR に明記する。
  途中で止まった場合も、作業ツリーの変更とテストは生きていることが多い。完了チェックで確かめる。

## 3. 検証する（Codex の報告をそのまま信じない）

1. 完了チェック（`docs/agents/dev-environment.md`）を Claude が自分で回す。Codex は sandbox から Nix を使えないので cargo でしか確かめていない。
1. 差分を読む: 依存の追加・`unsafe`・推測の値・規約違反・不要な変更がないか。
1. テストが問題を検知できるか確かめる: 修正を外す、または部分的に壊して、該当するテストが落ちることを見る。
1. 画面で確かめる（skill `gallery-check`）。ユーザーの所見があれば、その操作を再現する。

## 4. レビューと修正

```sh
nix develop -c bash -c "timeout 3000 codex review -c model='\"gpt-6.1-sol\"' -c model_reasoning_effort='\"xhigh\"' --uncommitted" > $SCRATCH/review.txt 2>&1
```

コミット済みなら `--uncommitted` の代わりに `--base main` を使う。実行のしかたは 2 と同じ。

- **完了を待つ。** 待つ条件と、使用量の制限で使えないときの代わり（サブエージェント）は skill `ship-pr` の 3。
- [P1] と大きい [P2] は、指摘の本文をそのまま貼った修正用プロンプト（common.md + `## Task: fix review findings`）で 2 を回し、3 と 4 をやり直す。
- 小さい指摘は直さず報告する。妥当でない指摘は理由を PR に書いて退ける。公開 API で直せない指摘は `docs/agents/backlog.md` に制限として残す。
- 指摘がなくなったら skill `ship-pr` の続きへ進む。PR の「確認」にレビューの回数と直した指摘を書く。

## 注意

- `pkill -f "codex ..."` は自分のシェルにも当たる。止めるときは `pkill -f "codex revie[w]"` のように正規表現で自分を外す。
- 時刻まで待つときは `setsid nohup <script> &` で切り離す（完了の通知は来ないので、終わりは別の手段で見る）。
