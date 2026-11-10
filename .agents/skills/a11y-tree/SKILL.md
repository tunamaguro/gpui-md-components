---
name: a11y-tree
description: 実機 (Linux の AT-SPI) で、アプリが支援技術に公開する a11y ツリー (role・名前・状態・親子関係) を書き出し、支援技術からの操作 (click・フォーカスの要求) を送って確かめるときに使う。gpui のテスト用 platform では確かめられない、テストの 3 段目の検証に使う
---

# 実機の a11y ツリーの確認

gpui のテスト用 platform は a11y を初期化しないため、`#[gpui::test]` では要素が書いた a11y ノードの属性までしか確かめられない。
親子関係、子の内容から求まる名前、AccessKit のアダプタが OS に渡す状態、支援技術からの操作の配送は、この手順で確かめる

## 手順

コマンドはリポジトリのルートで実行する。Linux (WSLg を含む) のデスクトップのセッションバスが必要

1. AT-SPI のバスを用意する。アプリより先に実行すること。アプリは起動時に AT-SPI のバスが無いと、後から用意しても登録しない

   ```sh
   nix develop .#a11y -c .agents/skills/a11y-tree/scripts/start-bus.sh
   ```

1. 確かめるアプリを default の devShell で起動する。gallery の場合は `WAYLAND_DISPLAY= cargo run -p gallery` をバックグラウンドで動かす

1. ツリーを書き出す。各行の先頭が子の番号の列 (パス)

   ```sh
   nix develop .#a11y -c python3 -I .agents/skills/a11y-tree/scripts/dump.py gallery
   ```

1. 操作を送る。`<操作>` は dump の `actions` に出る名前か `focus`

   ```sh
   nix develop .#a11y -c python3 -I .agents/skills/a11y-tree/scripts/act.py gallery <パス> <操作>
   ```

   送った後にもう一度書き出し、表示とツリーが変わったことを確かめる

1. 終わったらアプリを止め、支援技術の状態を戻す。有効なままだと、以降に debug ビルドで起動したアプリも a11y ツリーを組み立て、a11y が有効なときだけの検査 (重複した id など) で panic することがある

   ```sh
   nix develop .#a11y -c .agents/skills/a11y-tree/scripts/stop-bus.sh
   ```

## 注意

- `nix develop` の出力をパイプに通さないこと。nix の crashpad_handler がパイプを持ち続け、受け手が終わらない。ファイルにリダイレクトする
- `focused` はウィンドウが前面にあるときだけ付く
- AT-SPI のアダプタ (accesskit_atspi_common 0.19.1) は、disabled を role によって変えて渡す
  - checkbox・radio button・slider などは `read-only` になり、`sensitive` が外れる
  - button などは `sensitive` のまま。手掛かりは Click 操作が無いことと、`Availability::Disabled` では `focusable` が無いことだけ
- AccessKit のアダプタは invalid を OS に渡さない
- role が無い要素と id が無い要素はツリーに出ない。`&str`・`String`・`SharedString` をそのまま子にしたテキストも出ず、親の名前にもならない
- 子孫のテキストから名前を求めるのは Button、CheckBox、Link、MenuItem 系、RadioButton などに限られる。Switch や Tab の名前は `aria_label` で与える
- 支援技術の click は、部品が `on_a11y_action` で受けない限り、ノードの中央へのマウス操作として届く。覆われた部品には届かず、その位置の別の要素が押される
