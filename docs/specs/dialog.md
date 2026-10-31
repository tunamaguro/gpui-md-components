# Dialog 仕様要約

- 出典: Jetpack Compose Material 3 の `AlertDialog.kt`（`AlertDialogContent`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DialogTokens.kt`, `ScrimTokens.kt` @ b8256a52

## バリアント

- 基本のダイアログ（`Dialog`）、確認を求めるダイアログ（`.alert()`、役割が AlertDialog）

## 構成要素（アナトミー）

1. スクリム（Scrim 32%）
1. コンテナ（SurfaceContainerHigh、角 ExtraLarge、Level 3）
1. アイコン（任意、Secondary、24dp）
1. 見出し（Headline Small、アイコンがあれば中央揃え）
1. 補助テキスト（Body Medium、OnSurfaceVariant）と任意の中身
1. 操作のボタン（Text Button、右揃え）

## 寸法

| 項目         | 値         | トークン                                       |
| ------------ | ---------- | ---------------------------------------------- |
| 幅           | 280〜560dp | `layout::DIALOG_MIN_WIDTH`, `DIALOG_MAX_WIDTH` |
| 内側の余白   | 24dp       | `layout::DIALOG_SPACE`                         |
| アイコンの下 | 16dp       | `layout::DIALOG_ICON_BOTTOM_SPACE`             |
| 見出しの下   | 16dp       | `layout::DIALOG_TITLE_BOTTOM_SPACE`            |
| 本文の下     | 24dp       | `layout::DIALOG_TEXT_BOTTOM_SPACE`             |
| ボタンの間   | 8dp        | `layout::DIALOG_BUTTON_SPACE`                  |

## 振る舞い

- ウィンドウの中央に出す。スクリムのクリックと Esc で `on_dismiss` を呼ぶ。
- 開くと中の最初の要素にフォーカスする。Tab / Shift+Tab は中で回る。
- 閉じると開いた要素にフォーカスを戻す。操作のボタンは handler を呼んだ後に戻す。

## アクセシビリティ

- 補助テキストを description として伝える（WCAG 1.3.1）。

- 役割: Dialog（`.alert()` で AlertDialog）

- 名前: 任意の `.headline()` が既定。`.aria_label()` で上書きできる

- キー操作: Tab がダイアログの中で回る、Esc で閉じる

## 既知の制限・仕様からの逸脱

- 全画面ダイアログは未実装。
- gpui の a11y API に modal の印がないため、後ろの要素はツリーに残る（フォーカスとマウスは届かない）。
- 開閉のアニメーションはない。
