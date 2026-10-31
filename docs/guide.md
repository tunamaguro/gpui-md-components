# 利用ガイド

md3-components を GPUI のアプリで使うための手引き。部品ごとの仕様（寸法・状態・キー操作）は
[`docs/specs/`](specs/) にある。

## 1. 準備

- `gpui` は md-components と**同じ git rev** を使う（`Cargo.toml` の `[workspace.dependencies]` を参照）。
  rev が違うと gpui の型が別物になり、部品を要素として渡せない。
  自分で依存に書く代わりに、再公開の `md3_components::gpui` を使ってもよい。
  `gpui_platform` も feature `platform`（既定では無効）を付ければ `md3_components::gpui_platform` で使える。
- 必要な部品群だけを feature で選べる（`buttons` / `selection` / `inputs` / `overlays` / `navigation` / `containment` / `expressive`、既定は全部）。
  `inputs` と `navigation` は `overlays` を、`overlays` は `buttons` を自動で有効にする。

起動時に 1 回:

```rust
md3_components::init(cx); // 既定のテーマの設定と、部品のキー操作の割り当て
```

`init` はテキスト編集・メニュー・ダイアログのキー（矢印、Home / End、Esc、Tab の閉じ込めなど）を
部品の key context に割り当てる。アプリの Tab / Shift-Tab（`window.focus_next` / `focus_prev`）は
**`init` より前に**割り当てる。後から割り当てた方が優先されるため、順序が逆だとダイアログの中で Tab が外に出る。

## 2. テーマ

```rust
use md3_components::theme::{
    Appearance, Argb, Contrast, Density, MotionScheme, SchemeVariant, SetMd3ThemeExt, Theme,
};

cx.set_md3_theme(
    Theme::from_seed(Argb::from_rgb(0x00, 0x6A, 0x6A))
        .variant(SchemeVariant::TonalSpot) // Monochrome / Neutral / TonalSpot / Vibrant / Expressive / Fidelity / Content
        .contrast(Contrast::Standard)      // Standard / Medium / High
        .appearance(Appearance::System)    // System / Light / Dark
        .density(Density::Minus1)          // デスクトップ向けの密度
        .motion_scheme(MotionScheme::Standard) // Standard / Expressive（M3 Expressive の部品のバネ）
        .font_family("Noto Sans JP")       // 省略時はシステムフォント
        .build(),
);
```

- `Appearance::System` のときは `theme::follow_window_appearance(window, cx)` の返り値を保持している間、
  OS のライト / ダークに追従する。
- 部品の描画からは `cx.md3()` でテーマを参照する（`theme.color(ColorRole::Primary)` など）。
- 生成した配色はどの組み合わせでも WCAG 2.2 のコントラスト比を満たす（`crates/md3-theme/tests/contrast.rs`）。
  `ThemeBuilder::override_color` で個別の色を差し替えたときは崩れることがあるので、開発時に
  `theme.check_contrast()` で確かめる（基準を下回るロールの組を返す）。
- 高コントラストを望む利用者のために、`Contrast` を設定から選べるようにすることを勧める。

## 3. モーション（M3 Expressive）

- `expressive` の部品（`ToggleButton`、`ButtonGroup`）は、押下・選択で形（角丸）と幅がバネで変わる。
  バネはテーマのモーションスキームから取る（`Standard` は行き過ぎが少なく、`Expressive` は弾む）。
- **OS の「視差効果を減らす」設定を gpui に伝える。** gpui は OS の設定を読まないので、アプリが
  `cx.set_reduce_motion(true)` を呼ぶ。`true` のとき部品のバネとアニメーションは動かず、最終の状態で描かれる。
  設定画面に切り替えを置くことも勧める。

```rust
cx.set_reduce_motion(user_prefers_reduced_motion); // 起動時と、設定が変わったとき
```

- Web では `matchMedia("(prefers-reduced-motion: reduce)").matches` の値を wasm に渡して同じように呼ぶ
  （gallery の `xtask/web/index.html` と `start` が例）。

## 4. 部品の使い方の共通ルール

- **操作できる部品には ID を指定する。** `Button::new("save", "保存")` の第 1 引数が `ElementId`、
  第 2 引数が見た目のラベル兼スクリーンリーダーが読む既定の名前。名前は API 上任意で、表示文字を優先し `.aria_label()` で指定・上書きできる。表示ラベルがある場合、名前にその文字を含める（WCAG 2.5.3）。実行時に必要な名前はアプリの責任で付ける（[名前の規約](agents/a11y.md#%E5%90%8D%E5%89%8D)）。
- **入力欄と見出しは任意。** `TextField::new(id, &state).label("名前")`、`Select::new(id, options, selected).label("色")`、`Dialog::new(id).headline("確認")` の表示文字は既定の名前にもなる。SideSheet も `.headline()` を使う。
- **ID はフレームをまたいで安定させる。** リストの項目はデータのキーから ID を作る（index にしない）。
  ID が変わるとフォーカスと状態（開閉・押下など）が失われ、読み上げ位置も飛ぶ。
- **同じ ID・同じ `TextFieldState` を同時に 2 か所に描かない。** フォーカスハンドルを 2 つの要素が共有し、
  アクセシビリティが有効なときに gpui のデバッグビルドの検査で panic する（リリースビルドでは警告）。
- 部品は値を持たない。選択・入力値・開閉は利用者が持ち、`selected(..)` などで渡して `on_change` などで受け取る。
  例外は TextField（`gpui_headless::TextFieldState` を `Entity` で持つ）と、フォーカスや一時的な表示状態（部品が内部で保持）。
  Checkbox の値には `gpui_headless::CheckboxState` を使う。
- 無効な部品（`.disabled(true)`）は Tab で止まらず、操作できない。
- 見本用の `inline()` は gallery 向けで、アプリでは使わない（`#[doc(hidden)]`）。
- gallery には各バリアントの操作できる見本と disabled、該当すれば selected / error の見本を並べる。hover / focused / pressed はマウスやキーボードで実際に操作して確かめる。

## 5. キーボード操作

全部品の割り当ては [`docs/agents/a11y.md`](agents/a11y.md) の表にある。要点:

| 部品                                                 | 操作                                                                                   |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------- |
| ボタン類・Checkbox・Switch・Chip・Card・ToggleButton | Tab で移動、Enter / Space で押す                                                       |
| ButtonGroup                                          | 各項目が Tab で止まり、Enter / Space で切り替える                                      |
| RadioGroup・SegmentedButton・Tabs                    | Tab でグループに入り、矢印で移って選ぶ（Home / End）                                   |
| NavigationBar / Rail / Drawer                        | 矢印でフォーカスだけを移し、Enter / Space で選ぶ                                       |
| List / LazyList                                      | 矢印で移る（端で止まる）、Enter / Space で選ぶ                                         |
| Slider                                               | 矢印で 1 段、Home / End で端                                                           |
| Menu・Select                                         | 開くと最初の項目にフォーカス、矢印で移動、Enter で選ぶ、Esc で閉じて元に戻る           |
| Dialog・モーダルな Sheet / Drawer                    | 開くと中にフォーカスし、Tab は中に閉じ込める。Esc で閉じ、開いた要素にフォーカスを戻す |

## 6. Web（wasm32）

- ブラウザでは gpui がシステムフォントを読めない。起動時に `cx.text_system().add_fonts(..)` でフォントの
  データを渡す。日本語などを表示するなら、その字を持つフォントも渡す
  （gpui の既定の候補に入っている「Noto Sans」などを選ぶと、テーマでフォント名を指定しなくてよい）。
  gallery の実装（`gallery/src/main.rs` の `start` と `xtask/web/index.html`）が例になる。
- 描画は WebGPU、使えなければ WebGL2。`gpui_platform::application_with_web_backend` で指定できる。
- 共有メモリ（atomics）を使わない通常のビルドなら、配信に COOP / COEP ヘッダーは要らない。
- 編集のショートカットは OS を区別できないため Ctrl（macOS のブラウザでも Ctrl）。
- `cargo xtask web` は `cargo metadata` からビルド成果物の場所を取得するため、`CARGO_TARGET_DIR` や Cargo 設定の `build.target-dir` を指定しても wasm を見つけられる。配信用の出力先は引き続きワークスペース直下の `target/web/`。`nix develop` の `MD3_WEB_FONTS` は Linux・Darwin の両方で設定する。

## 7. スクロール

### 入れ子のスクロール

LazyList・`max_height` を付けた List・Menu（Select のポップアップを含む）・NavigationDrawer・
Dialog / BottomSheet / SideSheet の本文は、内側が動ける方向の
ホイールを消費する。端を越える差分は内側の端まで移動して消費し、すでに端にいる方向の
次の入力を外側へ渡す。開始フェーズのある精密スクロールは、いったん内側が消費したら
終了・キャンセルまで保持する。開始フェーズのない入力はイベントごとに端を判定する。
利用側で追加する任意の GPUI スクロール領域には自動では適用されない（外側も一緒に動く）。

### キーボードフォーカスを見える位置へ移す

`gpui-headless` を依存に追加し、アプリのスクロール領域を、同じ `ScrollHandle` を渡した `ScrollScope` で包みます。
Button・Checkbox などの操作部品、押せる Card、TextField / SearchBar は、Tab / Shift+Tab や矢印キーで
フォーカスが移ると、自動的に見える位置までスクロールします（[WCAG 2.2 の 2.4.11](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html)）。

```rust,ignore
let scroll = gpui::ScrollHandle::new(); // ビューまたは use_keyed_state で保持する
let page = gpui_headless::ScrollScope::new(scroll.clone(),
    gpui::div().id("page").h(gpui::px(240.0))
        .overflow_y_scroll().track_scroll(&scroll)
        .child(content)
        .child(md3_components::Scrollbar::new("page-scrollbar", scroll.clone())));
```

独自のフォーカス可能要素には `reveal_on_focus` を子として付けます。ID はその要素と同じものを使い、
親を `.relative()` にします。役割・名前・状態とキー操作も要素側で用意します。

```rust,ignore
let focus = gpui_headless::focus_handle(id.clone(), window, cx);
let margin = gpui::px(md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_THICKNESS
    + md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_OUTER_OFFSET);
let reveal = gpui_headless::reveal_on_focus(&id, focus.clone(), window, cx, margin);
let element = gpui::div().id(id).relative().track_focus(&focus)
    .role(gpui::Role::Button).aria_label("保存")
    .child(label).child(reveal).on_click(save);
```

List の `max_height`、Drawer、Menu / Select、Dialog / Sheet の本文にはスコープが付いています。
入れ子でも内側から順に両軸を調整します。外側のフォーカスリング（太さ 3dp + 間隔 2dp）も含めますが、
スクロール限界までに制限します。要素が領域より大きければ先頭を合わせます。端にリングを置く場合は、本文に余白も確保してください。
マウスクリックではスクロールせず、同じ要素にフォーカスしたままホイールで離しても引き戻しません。
遅延描画するメニューは自身の領域だけを調整し、背後のページを動かしません。LazyList は従来どおり `scroll_to_item` を使います。

### スクロールバー

スクロール領域を `ScrollHandle` で追跡し、最後の子に `Scrollbar` を足します。
位置の供給元を表示領域として描くため、バーのための高さや幅を確保する必要はありません。

```rust,ignore
let scroll = gpui::ScrollHandle::new(); // ビューまたは use_keyed_state で保持する
let area = gpui::div().id("content").h(gpui::px(240.0))
    .overflow_y_scroll().track_scroll(&scroll)
    .child(content)
    .child(md3_components::Scrollbar::new("content-scrollbar", scroll.clone()));
```

既定は縦・ドラッグ可能・静止後にフェードです。`.axis(gpui::Axis::Horizontal)` で横、
`.interaction(gpui_headless::ScrollbarInteraction::Passive)` で表示のみ、`.fade(false)` で常時表示にできます。
独自の仮想リストには `gpui_headless::ScrollbarSource` を実装します。位置は先頭を 0 とした正の距離です。
キーボード操作とアクセシビリティはコンテナ側で用意します。バー自体は Tab 順に入りません。

LazyList、Menu / Select、Dialog、BottomSheet / SideSheet、NavigationDrawer は既定でバーを表示します。
通常の List は `.max_height(px(240.0))` でスクロール可能にすると表示します。
各部品の `.scrollbar(false)` で表示を止められます。gallery の Containment ページに
縦の操作型・表示のみ・横・内容が収まる場合の見本があります。

## 8. 既定のモーションと Snackbar

Switch、RadioGroup の内側の円、NavigationBar / Rail の選択背景、Filled TextField / Select の下線は既存の Theme::spring を使う。Standard / Expressive の切替にも従う。OS 設定はアプリから `cx.set_reduce_motion(true)` で反映する。`reduce_motion` が true のとき、これらのバネはすべて即時に最終値となり、Snackbar の退場待ちもなくなる。

Snackbar は最新の知らせへの Replace が既定。順番を待たせるには `cx.new(|_| SnackbarHost::new("閉じる").policy(SnackbarPolicy::Queue))` を使う。Queue の操作付きメッセージは、既定では操作か閉じるまで次に進まない。`SnackbarMessage::on_result` で操作と破棄を区別できる。gallery の Overlays には Queue / Replace のスイッチを設け、静的見本には「静的見本」の文言を付けた。通常の Selection / Navigation / Inputs の操作サンプルで状態間のモーションを確認できる。

`on_result` は待機分の破棄も含め、ホストの更新が完了してから一度だけ呼ばれる。結果ハンドラー内で同じ `Entity<SnackbarHost>` を更新して次の知らせを追加できる。操作ボタンのハンドラーは結果通知より先に完了する。Overlays の「結果から次の知らせ」→「確認」でこの使い方を試せる。

Switch は Space / Enter を押したまま Tab で移動した場合も、フォーカス喪失時につまみの押下を取り消す。キーを別の要素で離しても押下寸法は残らない。Selection の操作サンプルで確認でき、`reduce_motion` では通常の寸法へ即時に戻る。

実装していない既定モーションの一覧は [`docs/agents/backlog.md`](agents/backlog.md) にある。

## 既知の制限

### アクセシビリティ（GPUI / AccessKit 側の制約）

- **Web にはアクセシビリティツリーが無い。** gpui は wasm32 で AccessKit を組み込まない（`gpui/src/window.rs` の
  `#[cfg(not(target_family = "wasm"))]`）。Web ではスクリーンリーダーが部品を読めない。キーボード操作と
  フォーカスリングは動く。
- 無効状態はツリーに出している（AccessKit の `disabled`）が、Linux（AT-SPI）では AccessKit がボタン・タブ・リスト項目など
  一部の役割で無効を伝えない（`accesskit_atspi_common` が読み取り専用を持てる役割だけ `enabled` を外す）。
  Checkbox・RadioButton・Switch・Slider・TextField・Select では伝わる。
- Snackbar は live region（polite）として出すが、表示時に読み上げられるかはスクリーンリーダー次第。
- 文字の拡大はテーマの `.text_scale(..)` で設定する。OS の文字サイズは自動では反映されないので、アプリが読み取って渡す。
- `md3_tooltip` のツールチップはポインタを重ねたときだけ出る。`md3_focus_tooltip`（IconButton の `.tooltip()`）はキーボードフォーカスでも出し、Esc で閉じる。どちらも同じ文を基準の要素の説明（description）に入れる。
- TextField / Select は `.required(true)` で必須状態を伝える。見える必須表示はラベルに付ける。補助テキストは description に入り、エラー時は invalid と Status の live region で伝える。
- Menu を開く Button / IconButton には `.menu_expanded(open)` を渡す。支援技術の開閉操作も状態に応じてクリックを実行する。Select のポップアップは Listbox として伝える。ListItem、MenuItem、Dialog の補足も description で伝える。項目の情報は `ListItem::a11y_node(index, count)` または `Menu::item_a11y_node(index)` で取得できる。
- 静的な `Text` をツリーに出すには `.id()` に一意で安定した ID を指定する。省略時は従来どおりツリーに出ない。
- Dialog・モーダルな Sheet / Drawer には modal の印を付けるが、後ろの要素はツリーに残る（フォーカスとポインタは届かない）。
- gpui はフォーカスされた要素で Enter と Space の両方をクリックとして扱うため、Checkbox / Switch は Enter でも切り替わる。

### 表示・入力

- テキストフィールドの書記素の境界は簡易判定（結合文字・絵文字の ZWJ 列・国旗は扱う。依存を増やさないため
  Unicode の全規則ではない）。
- Web ではフォントをアプリが渡す必要がある（上記）。渡したフォントに無い字は描けない（豆腐になる）。
- Web の IME の候補ウィンドウは画面の左上に出る（gpui_web が位置の更新に対応していない）。
- M3 Expressive は ToggleButton・ButtonGroup と、バネ・角丸の形の変化だけ。多角形の形のモーフ（`MaterialShapes`、
  LoadingIndicator）、Expressive の大きさの段階・FAB メニュー・ツールバーなどは未実装。
- gpui は OS の「視差効果を減らす」設定を読まない。アプリが `cx.set_reduce_motion(..)` で渡す（上記）。
- モバイル（iOS / Android）は対象外。
- フォントは同梱しない。既定はシステムフォント。

### 環境

- Linux の描画は Vulkan（wgpu）。GPU ドライバが無い環境では mesa の lavapipe（CPU 描画）で動く（遅い）。
- Web はブラウザの WebGPU / WebGL2 に依存する。ソフトウェア描画（SwiftShader）の WebGPU では
  描画デバイスが失われることがあり、その場合は `?renderer=webgl`（gallery）で WebGL2 を使う。

## 見た目を持たない操作部品

独自の見た目を作る場合、`gpui-headless` の `TextField`、`SearchField`、`AppBar`、`Progress`、`Scrollbar`、`List`、`LazyList`、`Button`、`Checkbox`、`Switch`、`Chip`、`Card`、`RadioGroup`、`SegmentedButton`、`ButtonGroup`、`Tabs`、`Navigation`、`Slider` を使えます。
`build` が返す要素に見た目を付け、描画用の状態からフォーカスやホバーを参照します。
フォーカスリングを外側に描く場合は、その外周までの余白を `reveal_margin` に渡します。
Button の `toggled` は通常のボタンとトグルを区別し、Chip の `selected` は Filter 相当の状態を表します。
Card は装飾用と押せるものを `CardBase` で区別し、渡した子要素と寸法を保ちます。
項目のある部品は `build` のクロージャへ操作を付けた項目と描画用の状態を渡します。
`Navigation` は項目の配列と初期フォーカス先も返すため、レールやモーダルのオーバーレイに組み込めます。
Slider の描画は `fraction` と `dragging` を使います。

独自の見た目を作る手順は [headless 利用ガイド](headless-guide.md) を参照してください。
