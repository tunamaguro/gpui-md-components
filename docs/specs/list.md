# List 仕様要約

- 出典: Jetpack Compose Material 3 の `ListItem.kt` とトークン。仮想化は gpui の `uniform_list`
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt` @ b8256a52

## バリアント

- 1 行（56dp）/ 2 行（72dp）/ 3 行（88dp）。行数は内容（上書き・補助テキスト）から決まる
- `List`（全項目を描く）/ `LazyList`（見えている項目だけを描く。高さが揃った項目向け）

## 構成要素（アナトミー）

1. 先頭: アイコン（24dp、OnSurfaceVariant）またはアバター（40dp、PrimaryContainer）
1. 上書き（Label Small）、見出し（Body Large、OnSurface）、補助テキスト（Body Medium、OnSurfaceVariant）
1. 末尾: アイコンまたは短い文字（Label Small）
1. 選択中は SecondaryContainer で塗る（角 Large）

## 寸法

| 項目                   | 値                 | トークン                                                      |
| ---------------------- | ------------------ | ------------------------------------------------------------- |
| 高さ                   | 56 / 72 / 88dp     | `list::ITEM_*_CONTAINER_HEIGHT`                               |
| 上下                   | 8dp（3 行は 12dp） | `layout::LIST_ITEM_VERTICAL_SPACE`, `…THREE_LINE…`            |
| 左右                   | 16dp               | `layout::LIST_ITEM_START_SPACE`, `END_SPACE`                  |
| 先頭・末尾の要素との間 | 16dp               | `layout::LIST_ITEM_LEADING_END_SPACE`, `TRAILING_START_SPACE` |

## 振る舞い

- LazyList は内側が動ける方向のホイールを消費し、ページを同時に動かさない。すでに端にいる方向の入力だけ祖先へ渡す。開始フェーズのある精密スクロールは、一度消費したら終了・キャンセルまで消費し続ける。List 自体はスクロール領域を持たない。

- `on_select` を付けた項目は押せる（クリック、Enter、Space）。

- Tab ではリスト全体で 1 回だけ止まる（選択中の項目、無ければ最初の押せる項目）。↑↓ で移り、端では止まる（回らない）。Home / End。

- LazyList は見えていない項目へ移るとき、その項目までスクロールしてからフォーカスする。Tab で止まるのは最後にフォーカスした項目で、見えていなければ最初に見えている押せる項目。

## アクセシビリティ

- 役割: List / ListItem（selected、位置）
- 名前: リストのラベル、項目の見出し

## 既知の制限・仕様からの逸脱

- 先頭の画像・動画、末尾のチェックボックスやスイッチの組み込み、並べ替え、スワイプは未実装。
- LazyList は gpui の `uniform_list` を使うため、項目の高さは全て同じ（最初の項目で測る）でなければならない。
- 選択中の項目の角（Large）は ListTokens の `ItemSelectedContainerShape` に従った。
