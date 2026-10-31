# SearchBar 仕様要約

- 出典: Jetpack Compose Material 3 の `SearchBar.kt`（`SearchBarDefaults.InputField`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SearchBarTokens.kt` @ b8256a52

## 構成要素（アナトミー）

1. コンテナ（SurfaceContainerHigh、完全な丸）
1. 先頭のアイコン（検索。押せるアイコンに置き換えられる）
1. 入力文字（Body Large、OnSurface）と placeholder（OnSurfaceVariant）
1. 末尾の押せるアイコン（消去など）

## 寸法

| 項目     | 値                                  | トークン                                                         |
| -------- | ----------------------------------- | ---------------------------------------------------------------- |
| 高さ     | 56dp                                | `search_bar::CONTAINER_HEIGHT`                                   |
| 幅       | 360〜720dp                          | `layout::SEARCH_BAR_MIN_WIDTH`, `SEARCH_BAR_MAX_WIDTH`           |
| アイコン | 48dp の領域の中央、領域を端から 4dp | `layout::MIN_INTERACTIVE_SIZE`, `layout::SEARCH_BAR_ICON_OFFSET` |

## 振る舞い

- 入力は TextField と同じ（`gpui_headless::TextFieldState`、IME、編集のキー）。結果の一覧は呼び出し側が描く（gallery は List を使う）。

## アクセシビリティ

- キーボードでフォーカスが移ると、祖先の `ScrollScope` の表示領域へ移す（WCAG 2.2 の 2.4.11）。
  `reveal_on_focus` を共有し、マウスクリックやフォーカスが変わらないホイール操作では引き戻さない。

- 役割: SearchInput、名前は既定で placeholder（`.aria_label()` で上書き可能）、value は入力文字

## 既知の制限・仕様からの逸脱

- 結果を重ねて出す展開（SearchView: 全画面 / docked）とそのアニメーションは未実装。
