# 独自テーマの作り方

`gpui-headless` は役割・名前・状態、キーボード操作、フォーカス管理、値の計算と入力・開閉の状態を持ちます。色、寸法、角、フォント、アイコン、影、モーションはテーマ側の責務です。依存は同じ rev の `gpui` と `gpui-headless` だけです。動く例は [custom_theme.rs](../examples/gallery/src/stories/custom_theme.rs)（gallery の Custom ページ）にあります。

起動時に `gpui_headless::init(cx)` で Tab / Shift-Tab によるフォーカス移動と部品内のキー操作を登録します。Tab の動作を変更する場合はその後で割り当てます。モーションを使うテーマでは `cx.reduce_motion()` に従ってください。

## 単体の部品

`build(window, cx)` は操作付きの土台と描画用の状態を返します。例の `frame` は枠・色を設定し、`focus_visible` と `disabled` を反映します。

```rust
let (base, state) = Button::new("custom-button-disabled", "無効")
    .disabled(true)
    .build(window, cx);
frame(base, state.focus_visible, state.disabled).child("無効")
```

ホバー・押下は土台の `.hover()` / `.active()`、子は `state.group` を使う `.group_hover()` / `.group_active()` で描けます。状態を更新するコールバックは利用者が指定します。

`Slider::build` が返す土台はポインター操作の領域なので、余白は土台の外側に置いてください。

## 項目とオーバーレイ

`RadioGroup`、`Tabs`、`List`、`Menu` の `build` は項目ごとのクロージャに土台と選択・フォーカス状態を渡します。List は `ListBase::finish`、Menu は `MenuPopup::place` のクロージャで入れ物を仕上げます。Select の開閉状態と `dismiss_handler` を Menu に渡します。Dialog は `overlay::modal_container`、`scrim`、`window_layer` と閉じる際のフォーカス復帰を組み合わせます。

```rust
let popup = menu.build(window, cx, |row, _, _| {
    frame(row.base, row.state.focus_visible, row.state.disabled)
        .child(format!("{}{}", if row.selected { "■ " } else { "" }, labels[row.index]))
        .into_any_element()
});
let popup = popup.place(|base| base.flex().flex_col());
```

List / LazyList の行内にボタンを置く場合は、`list_action_slot().child(button)` で囲み、`Button::tab_stop(row.action_tab_stop)` を渡します。ID は `ElementId::NamedChild(row.id.into(), "action".into())` など行から派生させ、一意な名前も付けます。入れ物はポインタ入力を行に渡さず、ホイールを通します。押せる行では Tab で行 → 操作 → 外、Shift+Tab で逆順、行と操作から ↑↓ / Home / End で押せる行へ移動します。押せない行の操作は個別の Tab 停止位置になります（LazyList では描かれている行だけが対象なので、操作は押せる行に付けます）。`row.state.disabled` は行が押せない場合も `true` なので、操作の無効状態は項目のデータから別に渡してください。

通知のビューは `SnackbarOwner` を実装して `SnackbarQueue` を保持し、`show` で表示時間付きの `SnackbarEntry` を追加します。描画前に `prepare`、`front` で現在の内容を取得し、退場後に `finish_exit` を呼びます。Button、Dialog、MenuItem、ListItem は `.description()` で補足説明を指定でき、各ノードに反映されます。Tooltip は `DescribedTooltip` で同じ説明を読み上げ対象にも付けます。

## テキストとスクロール

`TextFieldState` を Entity で保持し、`TextField::build` が返す `TextFieldControl::input` に `TextPaint`（文字・プレースホルダー・キャレット・選択範囲の色とキャレット幅）を渡します。`focus_visible` はキーボードフォーカスの表示に使います。スクロール領域には `ScrollScope` と必要に応じ `NestedScroll` を使い、`Scrollbar` には表示領域の `ScrollHandle`（複数行の入力欄の中は `TextFieldControl::scroll` の値）、テーマ側の寸法と描画を渡します。詳細は [利用ガイドのスクロールバー](guide.md) と [Scrollbar の仕様](specs/scrollbar.md) を参照してください。

入力要素はレイアウトを状態に書き戻すため、1 つの `TextFieldState` は必ず 1 つの入力要素だけで表示してください。

TextField / Select の表示ラベルは `.label()`、補足説明は `.description()` で指定できます。`.aria_label()` は設定の順序にかかわらずラベルを上書きします。補助テキストを状態としてもツリーに出す場合は `supporting_status(&id, text, error)` を使い、返された ID とノードを表示要素に付けます。

## アクセシビリティ

操作できる部品に一意な `ElementId` を渡してください。名前は API 上任意です。表示文字を既定とし、必要なら `.aria_label()` で指定・上書きします。実行時に必要な名前はアプリ側で付けてください。表示ラベルがある場合は名前にその文字を含めてください（WCAG 2.5.3）。headless が役割・状態とキー操作を付けますが、テーマは `focus_visible` を明瞭に表示し、文字と UI のコントラスト、十分な操作対象サイズを確保する必要があります。WCAG 2.2 と [a11y の規約](agents/a11y.md) に沿って実際の画面、キーボードと支援技術で確認してください。
