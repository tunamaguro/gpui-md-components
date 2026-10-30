# 未対応事項

実装していないこと・確認していないこと・人間の判断が要ることの一覧。片付けたら行を消し、新しく見つけたら足す。

## 確認していないこと

- 独自テーマの custom ページ: 実画面で hover / pressed / focus と操作、AT-SPI ツリーを未確認。

- 自動テストと、Linux（WSL、X11、lavapipe）の gallery での目視・マウス操作までは確認した。

- Windows / macOS の実機での見た目と操作、トラックパッドの精密スクロールは未確認
  （CI の Gallery ワークフローの Artifacts で試せる）。

- macOS / Windows での a11y ツリーの確認、全 OS での読み上げ確認は未実施（`docs/a11y-audit.md`）。

## 人間の判断・作業が要るもの

| 項目           | 内容                                                                                                                                       |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| 読み上げ確認   | `a11y.md` の手順 3（NVDA / VoiceOver / Orca で全部品）。`docs/a11y-audit.md` の「3. 読み上げ確認（未実施）」                               |
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

### レビューで見つけた不具合（2026-10-29）

Claude によるレビュー（Codex のレビューは利用上限で中断）。直したら行を消す。

| 優先 | 部品          | 内容                                                                     |
| ---- | ------------- | ------------------------------------------------------------------------ |
| P2   | Chip（Input） | 削除アイコンに役割と名前が無く、支援技術から見つけられない（WCAG 4.1.2） |
| P3   | TextField     | 貼り付けで `\r` が残る（CRLF の改行）                                    |
| P3   | SearchBar     | a11y のテストが `SearchBar` 自体を検査していない                         |

### その他

- Web の IME 候補ウィンドウは左上に表示される。gpui_web の位置更新と隠れた textarea の変更が必要で、このリポジトリの許可された依存では修正できない。
- ButtonGroup: 子の ID が同じ 2 つのグループを同じ親に置くと、押下の状態（`press_state`）を共有し、片方を押すと両方が広がる（`md3-components/src/expressive/button_group.rs`。分離前からの不具合）。
- フォーカス追従: `.hidden()` の下でフォーカスを持ったまま隠れた部品は prepaint されず、再表示と同時にフォーカスを戻すと `reveal_on_focus` が移動に気付かない（Codex レビューの P2。今の部品には該当する使い方がない）。
- Scrollbar: RTL での左右反転、タッチでのドラッグ（`docs/specs/scrollbar.md` の「制限」）。
- Snackbar: 入場の拡大は幅による近似（`docs/specs/snackbar.md`）。
- M3 Expressive: 多角形の形のモーフ（`MaterialShapes`、LoadingIndicator）、FAB メニュー、ツールバーなどは未実装。
- gpui の rev を上げるときは専用 PR にする。`gpui-notes.md` の各項目と gallery の全ページを確認し直す。
