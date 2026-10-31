# List 仕様要約

- 出典: Jetpack Compose Material 3 の `ListItem.kt` とトークン。仮想化は gpui の `uniform_list`
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt` @ b8256a52

## バリアント

- 1 行（56dp）/ 2 行（72dp）/ 3 行（88dp）。行数は内容（上書き・補助テキスト）から決まる。高さは最小値で、長い文は省略せず折り返す（Compose と同じ）。LazyList は全項目が同じ高さなので省略する
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

- `List::content_padding` は項目の内側の余白としてスクロールする。FAB を重ねる場合は末尾に必要な分を呼び出し側が指定する。`LazyList` には指定できない。

- 役割・名前・状態、キー操作、スクロールの振る舞いは `gpui-headless` が持ち、色・寸法・スクロールバーは `md3-components` が持つ。

- LazyList は内側が動ける方向のホイールを消費し、ページを同時に動かさない。すでに端にいる方向の入力だけ祖先へ渡す。開始フェーズのある精密スクロールは、一度消費したら終了・キャンセルまで消費し続ける。`max_height` を指定した List にも同じ制御を付ける。

- `on_select` を付けた項目は押せる（クリック、Enter、Space）。

- Tab ではリスト全体で 1 回だけ止まる（選択中の項目、無ければ最初の押せる項目）。↑↓ で移り、端では止まる（回らない）。Home / End。

- LazyList は見えていない項目へ移るとき、その項目までスクロールしてからフォーカスする。Tab で止まるのは最後にフォーカスした項目で、見えていなければ最初に見えている押せる項目。

## アクセシビリティ

- 上書きと補助テキストを `、` で連結し description として伝える（Compose `ListItem.kt` の mergeDescendants、WCAG 1.3.1）。

- 役割: List / ListItem（selected、位置）

- 名前: リストは任意の `.aria_label()`、項目は表示見出しが既定で `.aria_label()` で上書き可能

- `max_height` で高さを限ったときは、フォーカスが移った項目（キーボードによる矢印・Tab・初期フォーカス）は、スクロール領域の見える範囲に最小の量だけ動かして入れる（WCAG 2.2 の 2.4.11 Focus Not Obscured）。LazyList は `scroll_to_item` で同じことをする

- `ScrollScope` と共通の `reveal_on_focus` で、祖先のスクロール領域も内側から調整する。外側のリング分も含め、両軸をスクロール可能な範囲に制限する。マウスクリックでは動かさず、ホイールで離してもフォーカスが同じなら引き戻さない。

- LazyList の仮想化領域にはスコープを足さず、`scroll_to_item` を使う。

## 既知の制限・仕様からの逸脱

- 先頭の画像・動画、末尾のチェックボックスやスイッチの組み込み、並べ替え、スワイプは未実装。
- LazyList は gpui の `uniform_list` を使うため、項目の高さは全て同じ（最初の項目で測る）でなければならない。
- 選択中の項目の角（Large）は ListTokens の `ItemSelectedContainerShape` に従った。
