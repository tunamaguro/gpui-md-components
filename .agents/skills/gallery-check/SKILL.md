---
name: gallery-check
description: gallery（見本アプリ）を Linux の X11 で起動し、xdotool で操作（クリック・ホバー・ホイール・ドラッグ・キー）してスクリーンショットを撮り、部品の見た目と動きを確かめる。部品の追加・修正の後、ユーザーの所見の再現、PR 前の目視確認に使う。
---

# gallery で見た目と動きを確かめる

Codex の sandbox は Wayland に繋がらないので、画面の確認はこの skill で Claude が行う。
背景は `docs/agents/dev-environment.md` の「画面の確認」、GPUI の癖は `docs/agents/gpui-notes.md`。

## 手順

スクリプトは `.agents/skills/gallery-check/scripts/`。撮った画像は Read で見る。出力先は scratchpad にする。

1. 起動（ビルドも行う。既存の gallery は止める）:
   `scripts/launch.sh <ページ> [ログ]`
   ページ: `foundations buttons selection inputs overlays navigation containment expressive`。
   モーションを減らす設定で見るなら `MD3_REDUCE_MOTION=1 scripts/launch.sh ...`。
2. 目的の見本まで送る: `scripts/xdo.sh scroll 1100 600 <段数>`（ページ右側の空いた所で回すと、中のリストを回さない）。
   撮ってから位置を決める。ウィンドウは 1200×900、座標はスクリーンショットと同じ。
3. 操作:
   - ホバー `scripts/xdo.sh move X Y` / クリック `click X Y` / ドラッグ `drag X1 Y1 X2 Y2`
   - キー `scripts/xdo.sh key Tab space Escape`（フォーカスリングや APG のキー操作の確認）
4. 撮る: `scripts/shot.sh <出力.png> [WxH+X+Y]`。小さな差は切り出して拡大して見る
   （`nix shell nixpkgs#imagemagick -c magick in.png -crop ... -scale 200% out.png`）。
5. 色の有無を数値で確かめたいとき（細いバーなど）:
   `magick in.png -crop 8x900+1192+0 -scale 8x30! txt:- | grep -v '#FDF7FF'` のように画素を出す。
6. 終わったら `pkill -x gallery`（`pkill -f` は自分のシェルも殺すので使わない）。

## 注意

- lavapipe（CPU 描画）は遅い。フェード・バネ・Snackbar の表示時間など時間で変わるものは、操作の直後に撮る。
  「出ていない」と判断する前に、ホバーで保持するなどして時間の影響を外して撮り直す。
- 内部の値を確かめたいときは、一時的に `eprintln!` を入れてログ（launch.sh の第 2 引数）を見る。入れたものは必ず戻す。
- 確認できるのは Linux（X11）とマウスだけ。トラックパッドの精密スクロール、Windows / macOS の見た目は
  確認していないと PR と報告に書く（CI の Gallery ワークフローの Artifacts で人間が試せる）。
- a11y ツリーを見るなら `nix develop -c tools/a11y-tree/run.sh <出力先> [ページ...]`（第 1 引数は出力先）。
