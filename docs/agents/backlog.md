# 未対応事項

実装していないこと・確認していないこと・人間の判断が要ることの一覧。片付けたら行を消し、新しく見つけたら足す。

## 確認していないこと

- フォーカス移動やクリックによる IME 変換の終了は、Windows / macOS / Web / Linux の実際の IME では未確認。Windows 以外の入力欄間の移動は blur 処理のみに依存する。
  gpui に変換を終える公開 API がないため、Windows 以外で変換中にプログラムからフォーカスを移して戻すと、次の変換の更新で文字が二重になることがある（Codex レビューの指摘。実際の IME では未確認。X11 / Wayland のマウスのクリックは gpui が変換を終えるので該当しない）。

- 独自テーマの custom ページ: 全部品の hover / pressed / focus、開いた Dialog / Select / Tooltip の画面、AT-SPI ツリーを未確認。

- 自動テストと、Linux（WSL、X11、lavapipe）の gallery での目視・マウス操作までは確認した。

- `examples/mail` は Linux（X11）の 1200×900 と 500 幅で操作を確認した。Web 版は headless Chromium（WebGL2、SwiftShader）で起動と最初の画面だけを確認し、操作は未確認。Windows / macOS、読み上げ、a11y ツリーは未確認。確認中に 1 度、長いメールを開いて戻ったあとの Tab でフォーカスリングが出ないことがあった。同じ手順では再現せず、原因は分かっていない（最初の止まり先の検索欄はリングを描かないので、見落としの可能性もある）。

- Windows / macOS の実機での見た目と操作、トラックパッドの精密スクロールは未確認
  （CI の Gallery ワークフローの Artifacts で試せる）。

- macOS / Windows での a11y ツリーの確認、全 OS での読み上げ確認は未実施（`docs/a11y-audit.md`）。

## 人間の判断・作業が要るもの

| 項目           | 内容                                                                                                                                     |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 読み上げ確認   | `a11y.md` の手順 3（VoiceOver / Narrator / Orca で全部品）。`docs/a11y-audit.md` の「3. 読み上げ確認（未実施）」                         |
| v1.0 タグ      | 読み上げ確認で重大な指摘が無いことを確かめてから付ける。**エージェントが勝手に付けない**                                                 |
| 日本語フォント | Web 版の見本（GitHub Pages）に添える CJK フォント（Noto Sans CJK）は大きく、初回の読み込みが重い。字を絞る（サブセット化）かどうかの判断 |

## 未対応の候補（バックログ）

### サンプルアプリで見つけた不足

`examples/mail` を作って分かったこと。先頭の印は優先度で、P1 は操作不能・入力の喪失・WCAG 違反、P2 は機能の不足、P3 は整理や未確認の項目。

- [P2] `NavigationItem` にバッジを渡せず、受信トレイの未読数はラベルに含めた（`examples/mail/src/main.rs`）。

- [P2] `ListItem` の末尾は文字または静的アイコンに限られ、行内のスター切り替えを置けない。詳細のメニューで切り替える（`examples/mail/src/main.rs`）。

- [P3] 内容の上に重ねる部品は、下の要素にクリックを渡さない処理（`block_mouse_except_scroll`）を部品ごとに持つ必要がある。FAB は直したが、ほかの部品やアプリが重ねる要素には同じ問題が残りうる。

- [P3] Web では変換中の Esc がダイアログを閉じる可能性がある（`gpui_web/src/events.rs` は変換中の判定より先にキーを渡す。コードを読んだだけで、ブラウザでは未確認）。

- [P3] サンプル側の改善: 検索や絞り込みで一覧から消えたメールが詳細に残る。To の入力エラーは次の Send まで消えない。Archive の行き先の画面がなく、Undo でしか戻せない。`MailView::render` が長いので、一覧・詳細・設定に分ける。

- [P3] サンプルは画面のテストがなく、`model.rs` の単体テストだけ。`gpui` の test-support を dev-dependencies に足せば `MailView` を実際のキー割り当てで動かせる（`a11y.md` が認めている。レビューの検証用コピーで動作を確認）。

- [P3] 入力欄（TextField / SearchBar）はフォーカスリングを描かず、キーボードでの位置は点滅するカーソルと下線でしか分からない。`a11y.md` の「必ず描く」との関係を整理する。

- [P3] Scaffold をクリップされる領域の中に置くと、Snackbar はクリップされず、隠れた操作も Tab で止まる（`docs/specs/scaffold.md`）。`defer_draw` のマスクで描画は切れるが、退場の完了とフォーカスの追従も別に要るので、画面の根に置く前提にした。

- [P3] `LazyList` の内容の余白: gpui の `uniform_list` の padding は表示領域を縮めて固定されるため、スクロールする contentPadding として使えない。

### Tooltip

- 基準の要素がクリップされて見えないとき、ツールチップは出ないが 1 回目の Esc を受ける（`attach` が表示の判定を見ていない）。
- `md3_focus_tooltip` は `key_context` を上書きし、独自の `FocusHandle` を使う要素ではポインタの判定が効かない見込み（コードを読んだだけ）。doc コメントに前提がない。
- フォーカスのツールチップと、ほかの要素のホバーのツールチップが同時に出る。Compose は `BasicTooltipDefaults.GlobalMutatorMutex` で同時に 1 つにする（`BasicTooltip.kt`）。

### 文字の拡大

- ListItem の行数は内容の有無で決まり、折り返しを見ていない。Compose は補助テキストが複数行なら 3 行として扱う（`ListItem.kt`）。
- Menu の末尾の文字が長いと、本文の幅がほとんどなくなる。
- 200% で文字が容器に接する部品がある（Button の上下の余白、Rich Tooltip の上の余白）。リストのアバターの頭文字は 40dp の円からはみ出すことがある。
- Outlined の浮いたラベルが長いと枠の右をはみ出す。
- `text_scale` は 0、負数、NaN を検証していない。
- Filled の入力欄は 150% 以上で上下の余白がなくなる。Compose は `TextFieldWithLabelVerticalPadding` も足す。
- NavigationBar のラベルは幅が足りないと単語の途中で折り返す（幅 500、200% の `Settings` が `Setting` / `s` になる。`examples/mail` で確認）。
- NavigationBar はラベルの長さが違うと、200% で項目の高さが揃わない。ラベルの折り返しのテストがない（`rail_keeps_its_width_with_a_long_label` は幅だけを見る）。

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

### その他

- TextField の複数行（レビューの P3）: ZWJ の直後の改行が前の文字と結合し、Backspace で前の文字ごと消える（`boundaries`）。選択の改行の幅 `LINE_BREAK_SELECTION_WIDTH` は見た目の値だが headless にある。taffy の測定は丸める前の幅で折り返すので、端数の幅で prepaint の行数が 1 行増えることがある（コードを読んだだけ）。Web で変換中の上下キーは選択を動かす（コードを読んだだけ）。`reveal_in_scopes`、選択の改行の幅、段落の間引き、幅からのキャレット分の差し引きには自動テストがない。`tab_leaves_the_multi_line_field` は移動先を確かめていない。
- Web の IME 候補ウィンドウは左上に表示される。gpui_web の位置更新と隠れた textarea の変更が必要で、このリポジトリの許可された依存では修正できない。
- ButtonGroup: 子の ID が同じ 2 つのグループを同じ親に置くと、押下の状態（`press_state`）を共有し、片方を押すと両方が広がる（`md3-components/src/expressive/button_group.rs`。分離前からの不具合）。
- フォーカス追従: `.hidden()` の下でフォーカスを持ったまま隠れた部品は prepaint されず、再表示と同時にフォーカスを戻すと `reveal_on_focus` が移動に気付かない（Codex レビューの P2。今の部品には該当する使い方がない）。
- Scrollbar: RTL での左右反転、タッチでのドラッグ（`docs/specs/scrollbar.md` の「制限」）。
- TextField の選択ドラッグ（レビューの P3）: ウィンドウ全体の移動のリスナーを paint の `is_selecting` で登録するので、押下の次の描画より前の移動を取りこぼす。ドラッグ中に欄を無効にすると `is_selecting` が残り、欄の外のドラッグで選択が変わる。リスナーを常に登録し、イベントの時点で判定すれば両方直る（`event.dragging()` でなければ `is_selecting` を下げる）。
- 複数行の TextField のスクロールバーのテスト（レビューの P3）: 余白の左端の境界、末尾のアイコンと接尾辞がある欄の位置、1 行の欄を外へドラッグする場合、バーの上で離す場合の自動テストがない。`a11y_scrollbar.rs` は右端から 4px しか見ない。
- 複数行の TextField のスクロールバー（レビューの P3）: バーの上で欄の hover が切れる（`BlockMouseExceptScroll`）。端数の倍率で高さが丸められると、隠れた行がなくても 1 px 未満の `max` でバーが出る可能性がある（コードを読んだだけ）。
- Scrollbar の WCAG: 既定の操作領域は幅 12dp で 2.5.8 の 24dp に届かない。既定のつまみ（Outline 0.7）は背景によって 3:1 に届かず（既定の配色の Surface で 2.58:1）、`.scrollbar_fade(false)` で常時表示にして唯一の印として使うなら 1.4.11 に当たる。複数行の TextField だけ `docs/specs/text-field.md` の値に直した。
- Snackbar: 入場の拡大は幅による近似（`docs/specs/snackbar.md`）。
- TextField / Select のエラーの通知: Linux はエラーになっても文字が変わらないと読み上げず（AT-SPI は名前の変更だけを通知する）、ページを開いた時点のエラーは全 OS で読み上げられ、補助テキストは description と Status で二重にツリーに出る（レビューの P3）。
- Snackbar: `.aria_label()` を付けても macOS は `value`（本文）を読み上げるので、他の OS と読み上げる文が変わる（レビューの P3）。
- Select: Windows の UI Automation の Expand / Collapse で開閉できない。Button の `.menu_expanded()` と同じ処理が要る（レビューの P3）。
- `.menu_expanded()` の Expand / Collapse は `ClickEvent::default()` を渡すので位置が (0, 0) になる。位置でメニューを開くアプリは左上に開く（レビューの P3）。
- Chip: 削除ボタンは Button の中の Button になる。ARIA では Button の子は表現上のものなので、VoiceOver / Narrator で届くかを読み上げ確認で見る。届かなければ Group と Button 2 つに分ける（レビューの P3）。`on_remove` のない Input の削除ボタンは、操作できないのに有効に見える（レビューの P3）。
- TextField の支援技術: TextRun に位置と単語の境界がなく、拡大鏡のカーソル追従と単語単位の移動が効かない。無効の欄は macOS で値を設定できるように見えるが、要求は捨てられる。変換中の SetTextSelection は Windows 以外で文字が二重になることがある。IME の経路、`write_text_runs`、`a11y_actions`、Chip の押せる範囲のクリックには自動テストがない（レビューの P3）。
- M3 Expressive: 多角形の形のモーフ（`MaterialShapes`、LoadingIndicator）、FAB メニュー、ツールバーなどは未実装。
- gpui の rev を上げるときは専用 PR にする。`gpui-notes.md` の各項目と gallery の全ページを確認し直す。
