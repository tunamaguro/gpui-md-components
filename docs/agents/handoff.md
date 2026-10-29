# 現状と次の作業（引き継ぎ）

2026-10-29 時点。新しく作業を始める人（エージェント）は、AGENTS.md の次にこれを読む。

## 現状

- M0〜M8 の実装はすべて main にマージ済み（`milestones.md` の各節の「状態」）。
- M8 のあと、Windows で触った所見から次を直した。
  - #10 ButtonGroup の押下の広がりを隣が縮める幅までに抑える
  - #11 CI で gallery をビルドして Artifacts に置く（動作確認用、3 日で消える）
  - #12 CI のキャッシュ（Windows / macOS で効いていなかった）
  - #13 SegmentedButton の状態レイヤーの角、入れ子スクロール、M6 / M7 の Codex レビュー指摘
  - #14 Scrollbar
  - #15 Switch などの既定モーション、Snackbar の入れ替え（Replace / Queue）
- 確認の範囲: 自動テストと、Linux（WSL、X11、lavapipe）の gallery での目視とマウス操作。
  Windows / macOS の実機、トラックパッドの精密スクロールは人間の確認待ち（CI の Artifacts で試せる）。

## 人間の判断・作業が要るもの

| 項目 | 内容 |
| --- | --- |
| 読み上げ確認 | `a11y.md` の手順 3（NVDA / VoiceOver / Orca で全部品）。`docs/a11y-audit.md` の「3. 読み上げ確認（未実施）」 |
| v1.0 タグ | M7 の完了条件。読み上げ確認の結果を見てから付ける。**エージェントが勝手に付けない** |
| Web 版の公開 | `cargo xtask web` の出力をどこに置くか。**エージェントが勝手に公開しない** |
| 日本語フォント | Web 版 gallery に添える CJK フォント（Noto Sans CJK、大きい）の扱い。配信サイズとライセンス表記の判断 |

## 未対応の候補（バックログ）

### 既定モーション（#15 の調査結果。参照は androidx@b8256a52）

| 候補 | 状態・理由 |
| --- | --- |
| Checkbox | 未対応。Checkbox.kt L596–653 のチェック経路の描画率・不定状態のモーフは、今の SVG / 横線の描画を置き換える必要がある |
| RadioButton | 内側の円は実装済み。DefaultEffects の色変化は未対応 |
| Slider | 追加なし。Compose も押下 / ドラッグで幅を直接変える（バネなし） |
| TextField | Filled の下線は実装済み。ラベルの移動・Outlined の枠線は切り欠きと入力レイアウトの変更が要る |
| Tabs | 未対応。各タブ内の印を、測った位置を共有する 1 本のインジケーターに移す必要がある（TabRow.kt の DefaultSpatial） |
| NavigationBar / Rail | 選択背景は実装済み。アイコン / ラベルの色は未対応 |
| Menu / Select | 未対応。アンカー起点の拡大と、閉じたあとも退場中は描き続ける仕組みが要る（Menu.kt の FastSpatial / FastEffects） |
| Dialog | 追加なし。commonMain の AlertDialog.kt に共通の既定アニメーションがない |
| BottomSheet / NavigationDrawer | 未対応。位置・scrim の遷移と、閉じるときの保持・フォーカス復帰の時点を合わせる必要がある |
| SideSheet | 追加なし。Compose の commonMain に対応する実装がない |
| Tooltip | 未対応。gpui の tooltip の表示寿命と退場の保持を合わせる必要がある |
| ExtendedFAB | 追加なし。expanded 状態をまだ持っていない |

「退場中も描き続ける」仕組みは Snackbar（`overlays/snackbar.rs`）に実装がある。Menu / Sheet / Tooltip に広げるなら共通化を検討する。

### その他

- Scrollbar: RTL での左右反転、タッチでのドラッグ（`docs/specs/scrollbar.md` の「制限」）。
- Snackbar: 入場の拡大は幅による近似（`docs/specs/snackbar.md`）。
- M3 Expressive: 多角形の形のモーフ（`MaterialShapes`、LoadingIndicator）、FAB メニュー、ツールバーなどは未実装。
- gpui の rev を上げるときは専用 PR にする。`gpui-notes.md` の各項目と gallery の全ページを確認し直す。
