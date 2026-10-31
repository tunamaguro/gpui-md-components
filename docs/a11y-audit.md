# アクセシビリティ監査

手順は [`docs/agents/a11y.md`](agents/a11y.md) の「検証手順（3 段階）」。

- 対象: md-components の全部品、gpui @ `37faef81`（AccessKit 0.24 / accesskit_unix 0.22）
- 実施日: 2026-10-29

## 結果の要約

| 段階            | 状態                       | 結果                                                                                                   |
| --------------- | -------------------------- | ------------------------------------------------------------------------------------------------------ |
| 1. 自動テスト   | 実施                       | 全部品の役割・名前・状態と、キー操作・フォーカス移動をテストで確認。コントラストは全配色で基準を満たす |
| 2. ツリー確認   | Linux で実施（自動）       | 全 8 ページで名前の無い要素 0。役割・状態は想定どおり。監査中に見つかった不具合 2 件は修正済み（下記） |
| 2. ツリー確認   | macOS / Windows **未実施** | 人による確認が必要                                                                                     |
| 3. 読み上げ確認 | **未実施（全 OS）**        | VoiceOver / Narrator / Orca での確認が必要。下の表に記録する                                           |

**「重大な指摘ゼロ」（v1.0 タグの条件）は、手順 3 の読み上げ確認が終わるまで判定できない。**
自動で確認できた範囲（手順 1、Linux の手順 2）では、未修正の重大な指摘は無い。
ただし Web 版にはアクセシビリティツリーが無い（gpui の制約、下記）ことは既知の制限として残る。

## 1. 自動テスト

`cargo test --workspace` で実行する（CI で Linux / macOS / Windows）。

- **部品の a11y 情報**: 各 headless 部品が `a11y_node()`（役割・名前・状態を組み立てる純関数。要素に当てるものと同じ）を公開し、
  単体テストで役割・名前・選択 / 切り替え / 値・無効・modal・invalid・live を確認する。
- Button 系、Checkbox、Switch、Chip、Card、RadioGroup、SegmentedButton、ButtonGroup、Tabs、Navigation、Slider、TextField、SearchField、AppBar、Progress、List / LazyList の役割・名前・状態とキー操作は `crates/gpui-headless/tests/` で検証する。
  headless の分離の前後で Linux の AT-SPI ツリーを比べ、ページ切替に `Custom` ボタンが増えた以外は同じことを確認した。
- **キー操作**（`crates/md3-components/tests/` と `crates/gpui-headless/tests/`）: Button / Checkbox / RadioGroup / Slider / TextField / Menu / Dialog /
  Snackbar / Tabs / NavigationBar / List / LazyList / モーダルな Drawer で、Tab での到達、Enter / Space / 矢印 / Home / End、
  Esc で閉じてフォーカスが戻ることを確認する。
- **コントラスト**（`crates/md3-theme/tests/contrast.rs`）: シード色 22 色（色相 30° ごとの 12 色、既定の紫、黒・白・灰、
  RGB の原色と補色）× バリアント 7 × コントラスト 3 × ライト / ダーク = 924 配色で、前景と背景の 37 組が
  WCAG 2.2 の基準（文字 4.5:1、UI 部品 3:1）を満たす。不合格 0。
  - Primary / Secondary / Tertiary / Error は Text ボタン・タブ・エラー文などで文字色にもなるため、監査で面の上での基準を
    3:1 から 4.5:1 に上げた（`Theme::check_contrast` も同じ基準）。標準コントラストでも最小 4.62:1 で合格。
  - 無効状態の部品（38% の不透明度）は WCAG 2.2 の 1.4.3 / 1.4.11 の対象外。

## 2. ツリー確認

### Linux（AT-SPI、自動）

`nix develop -c tools/a11y-tree/run.sh` で、gallery の各ページを起動して AT-SPI のツリーをスクロールしながら書き出した
（専用の D-Bus セッションで動くので、デスクトップのスクリーンリーダーは要らない）。見えている部品だけがツリーに出るため、
1 ページを 9 か所で書き出して合わせた。

| ページ      | 異なる要素 | 主な役割                                                                | 名前の無い要素 |
| ----------- | ---------- | ----------------------------------------------------------------------- | -------------- |
| foundations | 31         | button、radio button（ページの切り替え）                                | 0              |
| buttons     | 54         | button、toggle button、radio button（SegmentedButton）                  | 0              |
| selection   | 77         | check box、radio button、toggle button（Chip）、slider、progress bar    | 0              |
| inputs      | 70         | entry、combo box                                                        | 0              |
| overlays    | 65         | menu / menu item、dialog、tool tip、status bar（Snackbar）              | 0              |
| navigation  | 52         | landmark（Navigation）、heading、page tab list / page tab、dialog、list | 0              |
| containment | 49         | list / list item、button（押せる Card）                                 | 0              |
| expressive  | 51         | toggle button（ToggleButton、ButtonGroup の項目）、panel（ButtonGroup） | 0              |

確認できた状態の例:

- Checkbox の checked / indeterminate、RadioButton・SegmentedButton（単一選択）の checked、Tabs・NavigationBar の selected、
  トグルの IconButton・SegmentedButton（複数選択）の pressed。
- Slider の値（`value=3.0` など）、Progress の値と indeterminate。
- 無効な Checkbox / Slider / TextField / Select は enabled・focusable が外れる（read only）。
- 各オーバーレイを AT-SPI でフォーカスして Enter で開くと、Dialog・BottomSheet・モーダルな SideSheet / Drawer は
  `modal` 付きの dialog として現れ、初期フォーカスが中に入る（Dialog は入力欄、Menu / Select は最初の項目、
  Drawer は選択中の項目）。見本用の `inline()` のものには `modal` が付かない。

### 監査中に見つけて直したもの

1. **無効・modal・入力エラー・live region がツリーに出ていなかった。** gpui の要素の API（`aria_*`）に無いため。
   `a11y_synthetic_children` で要素自身の AccessKit のノード（`gpui::accesskit` で再公開されている）に直接書くようにした
   （`crates/md3-components/src/a11y.rs`）。依存は増えていない。
1. **アクセシビリティが有効なとき、gallery でダイアログを開くと gpui のデバッグ用の検査で panic した**
   （`set_focus called more than once in a single frame`）。gallery の見本のダイアログと本物のダイアログが同じ ID と
   同じ入力欄の状態を使い、1 つのフォーカスハンドルを 2 つの要素が追跡していた。gallery の ID を分けて解消。
   リリースビルドでは警告だけになる。部品を使う側も、**同じ ID・同じ `TextFieldState` を 2 か所に描かない**こと
   （[利用ガイド](guide.md) の共通ルール）。

### Linux で確認できた制約（AccessKit 側）

- **ボタン・タブ・リスト項目などの無効が伝わらない。** `accesskit_atspi_common` 0.19 は、読み取り専用を持てる役割
  （Checkbox / RadioButton / Switch / Slider / テキスト入力 / ComboBox など）でしか enabled を外さない。
  これらの部品は無効なら Tab で止まらないため操作の妨げにはならないが、ツリーを辿ると有効に見える。
- **invalid（入力エラー）は AT-SPI に出ない**（`accesskit_atspi_common` が対応していない）。エラーの理由は補助テキストで伝える。
- live region（Snackbar）は AT-SPI の Announcement イベントとして送られる（Orca での読み上げは手順 3 で確認する）。
- AccessKit の Click（スクリーンリーダーの「押す」）は、gpui では要素の中心へのクリックとして処理される。
  画面外にスクロールされた要素は押せないことがある。フォーカスしてから Enter / Space で押すのは問題ない。

### macOS / Windows

未実施。Accessibility Inspector（macOS）、Accessibility Insights for Windows で gallery の各ページを開き、上の表と同じ
役割・名前・状態が出ていることを確かめる。特に、Linux で伝わらなかった無効（ボタン等）と invalid が出るかを見る。

## Web（wasm32）

**アクセシビリティツリーが無い。** gpui は wasm32 で AccessKit を組み込まない（`crates/gpui/src/window.rs` の
`#[cfg(not(target_family = "wasm"))]`）。キャンバスに描くだけなので、スクリーンリーダーは部品を読めない。
md-components 側では対処できない（gpui の上流の対応待ち）。

キーボード操作は動く: Chromium（SwiftShader の WebGL2）で gallery を開き、ページの切り替え、TextField への入力、
Tab でのフォーカス移動とフォーカスリングを確認した。

## 3. 読み上げ確認（未実施）

チェック項目（各部品）: ① Tab で移動できる ② 役割と名前が読まれる ③ 状態が読まれる ④ Space / Enter / 矢印で操作でき、
変化後の状態が読まれる ⑤ 無効状態が伝わる。結果は ✅（問題なし）/ ⚠️（軽微）/ ❌（重大）で記入し、⚠️ / ❌ は下に詳細を書く。

| 部品                           | macOS VoiceOver | Windows Narrator | Linux Orca |
| ------------------------------ | --------------- | ---------------- | ---------- |
| Button / IconButton / FAB      | 未              | 未               | 未         |
| SegmentedButton                | 未              | 未               | 未         |
| Checkbox                       | 未              | 未               | 未         |
| RadioGroup                     | 未              | 未               | 未         |
| Switch                         | 未              | 未               | 未         |
| Chip                           | 未              | 未               | 未         |
| Slider                         | 未              | 未               | 未         |
| Progress                       | 未              | 未               | 未         |
| TextField / SearchBar          | 未              | 未               | 未         |
| Select                         | 未              | 未               | 未         |
| Menu                           | 未              | 未               | 未         |
| Dialog                         | 未              | 未               | 未         |
| Tooltip                        | 未              | 未               | 未         |
| Snackbar（表示時に読まれるか） | 未              | 未               | 未         |
| BottomSheet / SideSheet        | 未              | 未               | 未         |
| TopAppBar                      | 未              | 未               | 未         |
| NavigationBar / Rail / Drawer  | 未              | 未               | 未         |
| Tabs                           | 未              | 未               | 未         |
| List / LazyList                | 未              | 未               | 未         |
| Card                           | 未              | 未               | 未         |
| ToggleButton / ButtonGroup     | 未              | 未               | 未         |

### 指摘

（読み上げ確認で見つかったものをここに書く）

## Scrollbar（2026-10-29）

- Canvas による装飾で、アクセシビリティの role / フォーカスを追加しない。スクロール領域の既存の意味を維持する。
- 自動確認: `tests/a11y_scrollbar.rs`（Tab 順への非介入、領域外ドラッグ・解除、横のページ移動、Passive）。
  幾何計算と reduce_motion による即時非表示はモジュール内の純関数テスト。
- gallery: Containment の Scrollbar 見本と根のスクロール。ホバー・ドラッグ中は表示を保持する。
- 手動確認: Windows / macOS / Linux のスクリーンリーダー、実 GPU の表示、Web のマウス操作は未検証。
  GPUI のテスト環境では AccessKit ツリーを取得できず、Web は AccessKit 非対応。

## 既定モーション / Snackbar の追加監査（2026-10-29）

- 対象: Switch / RadioGroup、Snackbar / Filled TextField、NavigationBar / Rail。
- すべて GPUI の with_spring を通し、reduce_motion では最終値へ即時変更。Snackbar は退場途中で設定を有効にした場合も待たずに進む。表示時間のタイマーは通知内容を読む時間なので維持する。
- Snackbar は表示ごとの安定 ID、Status、polite を維持。退場中は live を停止し、アクションと閉じるボタンを無効化。Replace の破棄は ActionPerformed と区別して一度だけ通知する。
- `tests/a11y_snackbar.rs`: 連続 Replace（待機分の破棄を含む）、退場後の次表示、古いタイマーの無効化、Queue の表示時間と無期限、キーボード操作結果、reduce_motion の途中変更を検証。
- レビュー修正: `on_result` は現在の知らせ・置換された待機分ともホストの更新後に通知する。`tests/a11y_snackbar.rs` に、結果ハンドラーが同じ `Entity<SnackbarHost>` に次の知らせを追加し、一度だけ通知される回帰テストを追加した。
- `tests/a11y_checkbox.rs`: アクティブなウィンドウで Switch の Space / Enter を押したまま次の要素へ移動し、キー解放が届く前に押下寸法から通常寸法へ戻ること、キャンセルで選択が変わらないことを検証（`reduce_motion` 有効、デバッグビルドのつまみ境界を取得）。
- 実 OS の目視・読み上げ（Orca / VoiceOver / Narrator）と Web の実機操作は未確認。従来の Web AccessKit 非対応は解消していない。

検証結果: `cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`、`cargo build -p md3-components --target wasm32-unknown-unknown`、`cargo xtask check-tokens` は通過（1597 定数一致）。`cargo run -p gallery` はビルド後、sandbox から Wayland compositor への接続が Operation not permitted となり起動できなかったため、実画面の目視確認は未実施。

## キーボードフォーカスのスクロール追従（2026-10-30）

- WCAG 2.2 の [2.4.11 Focus Not Obscured (Minimum)](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html) に対応するため、
  `gpui-headless` の `ScrollScope` と `reveal_on_focus` で、キーボードで移った要素を表示領域へ入れる。フォーカスリングの外周も計算に含める。
- `tests/navigation.rs`: ページ内の Button の Tab / Shift+Tab / 循環、Card・TextField、ページと高さ制限付き List の入れ子、
  マウスクリック時の非スクロール、ホイールで離した後の非追従、Menu の矢印移動と背後のページの非スクロールを検証する。
  入力前にテスト用の次フレーム配信を予約し、入力後は `run_until_parked()` だけを使う（手動 refresh / draw は足さない）。
- `scroll.rs` の純関数テストで両軸の最小移動、最大オフセットでの制限、大きな項目の先頭合わせ、3dp + 2dp のリング余白を確認する。
- gallery の根と独自の選択ボタンにも適用。Containment の見本には縦・横、Card、入力欄、高さ制限付き List と disabled を追加した。
- Linux の gallery の Tab の追従は人間が実操作で確認した。実 OS の読み上げと Web の操作は未確認。
- 回帰テストの有効性: 次フレームの予約を旧実装の `window.refresh()` に一時的に戻すと、
  `page_reveals_tab_shift_tab_and_wrapping` が画面外の bounds を検出して失敗することも確認した（実装は復元済み）。
- 完了チェック: `cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings`、
  `cargo clippy -p md3-components --no-default-features -- -D warnings`、`cargo test --workspace`（379 件成功）、
  `cargo build -p md3-components --target wasm32-unknown-unknown`、`cargo xtask check-tokens`（1597 定数一致）が通過。

独自テーマの `custom` ページは今回追加したため、上の 8 ページの実測には含まれない。画面と AT-SPI ツリー、読み上げは未確認。
