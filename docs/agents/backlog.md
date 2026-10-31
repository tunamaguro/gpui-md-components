# 未対応事項

実装していないこと・確認していないこと・人間の判断が要ることの一覧。片付けたら行を消し、新しく見つけたら足す。

## 確認していないこと

- フォーカス移動やクリックによる IME 変換の終了は、Windows / macOS / Web / Linux の実際の IME では未確認。Windows 以外の入力欄間の移動は blur 処理のみに依存する。
  gpui に変換を終える公開 API がないため、Windows 以外で変換中にプログラムからフォーカスを移して戻すと、次の変換の更新で文字が二重になることがある（Codex レビューの指摘。実際の IME では未確認。X11 / Wayland のマウスのクリックは gpui が変換を終えるので該当しない）。

- 独自テーマの custom ページ: 全部品の hover / pressed / focus、開いた Dialog / Select / Tooltip の画面、AT-SPI ツリーを未確認。

- 自動テストと、Linux（WSL、X11、lavapipe）の gallery での目視・マウス操作までは確認した。

- Windows / macOS の実機での見た目と操作、トラックパッドの精密スクロールは未確認
  （CI の Gallery ワークフローの Artifacts で試せる）。

- macOS / Windows での a11y ツリーの確認、全 OS での読み上げ確認は未実施（`docs/a11y-audit.md`）。

## 人間の判断・作業が要るもの

| 項目           | 内容                                                                                                                                       |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| 読み上げ確認   | `a11y.md` の手順 3（VoiceOver / Narrator / Orca で全部品）。`docs/a11y-audit.md` の「3. 読み上げ確認（未実施）」                           |
| v1.0 タグ      | 読み上げ確認で重大な指摘が無いことを確かめてから付ける。**エージェントが勝手に付けない**                                                   |
| 日本語フォント | Web 版 gallery（GitHub Pages）に添える CJK フォント（Noto Sans CJK）は大きく、初回の読み込みが重い。字を絞る（サブセット化）かどうかの判断 |

## 未対応の候補（バックログ）

### 既定モーション（Compose で既定のアニメーションがあるもの。参照は androidx@b8256a52）

| 候補                           | 状態・理由                                                                                                           |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Checkbox                       | 未対応。Checkbox.kt L596–653 のチェック経路の描画率・不定状態のモーフは、今の SVG / 横線の描画を置き換える必要がある |
| RadioButton                    | 内側の円は実装済み。DefaultEffects の色変化は未対応                                                                  |
| Slider                         | 追加なし。Compose も押下 / ドラッグで幅を直接変える（バネなし）                                                      |
| TextField                      | Filled の下線は実装済み。ラベルの移動・Outlined の枠線は切り欠きと入力レイアウトの変更が要る                         |
| Tabs                           | 未対応。各タブ内の印を、測った位置を共有する 1 本のインジケーターに移す必要がある（TabRow.kt の DefaultSpatial）     |
| NavigationBar / Rail           | 選択背景は実装済み。アイコン / ラベルの色は未対応                                                                    |
| Menu / Select                  | 未対応。アンカー起点の拡大と、閉じたあとも退場中は描き続ける仕組みが要る（Menu.kt の FastSpatial / FastEffects）     |
| Dialog                         | 追加なし。commonMain の AlertDialog.kt に共通の既定アニメーションがない                                              |
| BottomSheet / NavigationDrawer | 未対応。位置・scrim の遷移と、閉じるときの保持・フォーカス復帰の時点を合わせる必要がある                             |
| SideSheet                      | 追加なし。Compose の commonMain に対応する実装がない                                                                 |
| Tooltip                        | 未対応。gpui の tooltip の表示寿命と退場の保持を合わせる必要がある                                                   |
| ExtendedFAB                    | 追加なし。expanded 状態をまだ持っていない                                                                            |

「退場中も描き続ける」仕組みは Snackbar（`overlays/snackbar.rs`）に実装がある。Menu / Sheet / Tooltip に広げるなら共通化を検討する。

### レビューで見つけた不具合

直したら行を消す。

| 優先 | 部品      | 内容                                  |
| ---- | --------- | ------------------------------------- |
| P3   | TextField | 貼り付けで `\r` が残る（CRLF の改行） |

### その他

- Web の IME 候補ウィンドウは左上に表示される。gpui_web の位置更新と隠れた textarea の変更が必要で、このリポジトリの許可された依存では修正できない。
- ButtonGroup: 子の ID が同じ 2 つのグループを同じ親に置くと、押下の状態（`press_state`）を共有し、片方を押すと両方が広がる（`md3-components/src/expressive/button_group.rs`。分離前からの不具合）。
- フォーカス追従: `.hidden()` の下でフォーカスを持ったまま隠れた部品は prepaint されず、再表示と同時にフォーカスを戻すと `reveal_on_focus` が移動に気付かない（Codex レビューの P2。今の部品には該当する使い方がない）。
- Scrollbar: RTL での左右反転、タッチでのドラッグ（`docs/specs/scrollbar.md` の「制限」）。
- Snackbar: 入場の拡大は幅による近似（`docs/specs/snackbar.md`）。
- TextField / Select のエラーの通知: Linux はエラーになっても文字が変わらないと読み上げず（AT-SPI は名前の変更だけを通知する）、ページを開いた時点のエラーは全 OS で読み上げられ、補助テキストは description と Status で二重にツリーに出る（レビューの P3）。
- Snackbar: `.aria_label()` を付けても macOS は `value`（本文）を読み上げるので、他の OS と読み上げる文が変わる（レビューの P3）。
- Select: Windows の UI Automation の Expand / Collapse で開閉できない。Button の `.menu_expanded()` と同じ処理が要る（レビューの P3）。
- `.menu_expanded()` の Expand / Collapse は `ClickEvent::default()` を渡すので位置が (0, 0) になる。位置でメニューを開くアプリは左上に開く（レビューの P3）。
- `MenuItem::a11y_node` と `ListItem::a11y_node` は実際の描画と役割・位置が違い、使えるのは description だけ。`docs/specs/menu.md` の expanded の記述も古い（レビューの P3）。
- Chip: 削除ボタンは Button の中の Button になる。ARIA では Button の子は表現上のものなので、VoiceOver / Narrator で届くかを読み上げ確認で見る。届かなければ Group と Button 2 つに分ける（レビューの P3）。`on_remove` のない Input の削除ボタンは、操作できないのに有効に見える（レビューの P3）。
- TextField の支援技術: TextRun に位置と単語の境界がなく、拡大鏡のカーソル追従と単語単位の移動が効かない。無効の欄は macOS で値を設定できるように見えるが、要求は捨てられる。変換中の SetTextSelection は Windows 以外で文字が二重になることがある。支援技術からの値の `\r` は残る。IME の経路、`write_text_run`、`a11y_actions`、Chip の押せる範囲のクリックには自動テストがない（レビューの P3）。
- M3 Expressive: 多角形の形のモーフ（`MaterialShapes`、LoadingIndicator）、FAB メニュー、ツールバーなどは未実装。
- gpui の rev を上げるときは専用 PR にする。`gpui-notes.md` の各項目と gallery の全ページを確認し直す。
