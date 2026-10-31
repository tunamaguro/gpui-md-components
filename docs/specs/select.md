# Select（Exposed dropdown menu）仕様要約

- 出典: Jetpack Compose Material 3 の `ExposedDropdownMenu.kt`。見た目は TextField（読み取り専用）とメニューの組み合わせ。
- トークン出典: TextField と Menu のトークン（`text-field.md`, `menu.md` を参照）

## バリアント

- Filled / Outlined（`TextFieldVariant`）

## 構成要素（アナトミー)

1. 読み取り専用のテキストフィールド（選んだ項目を表示）
1. 末尾の ▼ / ▲ アイコン
1. 選択肢のメニュー（フィールドと同じ幅）

## 寸法・色

- TextField と Menu に同じ。

## 振る舞い

- ポップアップのスクロールは Menu と共通で、動ける間はホイールを消費し、ページを同時に動かさない。

- クリック、Enter / Space、↓ で開く。開くと選択中（無ければ先頭）の項目にフォーカスする。

- 項目を選ぶと `on_change(index)` を呼んで閉じ、フィールドにフォーカスを戻す。選択の値は呼び出し側が持つ。

- Esc / Tab / 外側のクリックで閉じる。

## アクセシビリティ

- 補助テキストを description と Status ノードにし、エラー時は invalid と live（polite）を付ける。`.required(true)` は必須状態を伝え、見える印はラベルに付ける（WCAG 1.3.1、3.3.1、4.1.3）。ComboBox に has-popup と expanded を付ける。

- 役割: ComboBox（選択肢は ListBox / ListBoxOption）

- 名前: 任意の `.label()` が既定。`.aria_label()` で上書きできる

- 状態: expanded、value（選択中の項目）、選択肢の selected

- キー操作: Enter / Space / ↓ で開く、↑↓ / Home / End で移動、Enter で選ぶ、Esc で閉じる

## 既知の制限・仕様からの逸脱

- 文字入力で絞り込む編集可能な形（editable）は未実装。
- フィールドには TextField のようなキャレットがないため、フォーカスは下線・枠線の色だけで示す（フォーカスリングは描かない。TextField と同じ）。
