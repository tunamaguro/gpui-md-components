# List 仕様要約

- m3.material.io の本文は読んでいない。以下は Compose とローカルの APG を参照した。
- 出典: Jetpack Compose Material 3 の `ListItem.kt` とトークン。仮想化は gpui の `uniform_list`
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt` @ b8256a52

## バリアント

- 1 行（56dp）/ 2 行（72dp）/ 3 行（88dp）。行数は内容（上書き・補助テキスト）から決まる。高さは最小値で、長い文は省略せず折り返す（Compose と同じ）。LazyList は全項目が同じ高さなので省略する
- `List`（全項目を描く）/ `LazyList`（見えている項目だけを描く。高さが揃った項目向け）

## 構成要素（アナトミー）

1. 先頭: アイコン（24dp、OnSurfaceVariant）またはアバター（40dp、PrimaryContainer）
1. 上書き（Label Small）、見出し（Body Large、OnSurface）、補助テキスト（Body Medium、OnSurfaceVariant）
1. 末尾: アイコンまたは短い文字（Label Small）。その後ろに `ListAction` のアイコンボタンを 1 つ併置できる
1. 選択中は SecondaryContainer で塗る（角 Large）

## 寸法

| 項目                   | 値                 | トークン                                                      |
| ---------------------- | ------------------ | ------------------------------------------------------------- |
| 高さ                   | 56 / 72 / 88dp     | `list::ITEM_*_CONTAINER_HEIGHT`                               |
| 上下                   | 8dp（3 行は 12dp） | `layout::LIST_ITEM_VERTICAL_SPACE`, `…THREE_LINE…`            |
| 左右                   | 16dp               | `layout::LIST_ITEM_START_SPACE`, `END_SPACE`                  |
| 先頭・末尾の要素との間 | 16dp               | `layout::LIST_ITEM_LEADING_END_SPACE`, `TRAILING_START_SPACE` |

## 振る舞い

- `ListItem::action(ListAction)` は行の ID の `NamedChild(..., "action")` を持つ Standard IconButton を 1 つ置く。再指定すると置き換える。`ListTrailing` の文字・静的アイコンと併用でき、操作の手前は `layout::LIST_ITEM_TRAILING_START_SPACE`（16dp）。ボタンは既存の IconButton の寸法・色・状態レイヤー・フォーカスリングを使う。

- `ListAction::new(icon).aria_label(name)` でアイコンと名前を指定する。`.selected(bool)` でトグルになり、`.on_click()` で呼び出し側が状態を更新する。`.disabled(true)` または行の無効状態で操作できず、Tab でも止まらない。

- Compose `ListItem.kt` @ b8256a52 の `trailingContent: @Composable (() -> Unit)?`（L145、L264）は任意の要素を受け取る。本 API はそのうちアイコンボタン 1 つに対応する。

- ボタンとその手前の余白は `gpui_headless::list_action_slot()` で囲み、ポインタ入力を行に渡さない。ボタンのクリックでは行の `on_select` を呼ばず、行の hover / pressed の状態レイヤーも出さない。ホイールはリストへ通す。

- `List::content_padding` は項目の内側の余白としてスクロールする。FAB を重ねる場合は末尾に必要な分を呼び出し側が指定する。`LazyList` には指定できない。

- 役割・名前・状態、キー操作、スクロールの振る舞いは `gpui-headless` が持ち、色・寸法・スクロールバーは `md3-components` が持つ。

- LazyList は内側が動ける方向のホイールを消費し、ページを同時に動かさない。すでに端にいる方向の入力だけ祖先へ渡す。開始フェーズのある精密スクロールは、一度消費したら終了・キャンセルまで消費し続ける。`max_height` を指定した List にも同じ制御を付ける。

- `on_select` を付けた項目は押せる（クリック、Enter、Space）。

- 押せる行の Tab の対象は、フォーカスを含む行、選択中の行、最初の押せる行の順に優先する。行内のボタンも同じ行だけを Tab の対象にし、行 → ボタン → リストの外と進む。Shift+Tab は逆順。ボタンが無い・無効なら飛ばす。押せない行の有効なボタンは、それ自体が Tab の止まり場所になる。

- 押せる行とそのボタンから ↑↓ / Home / End で押せる行へ移る。端では止まる（回らない）。矢印で移った行から Tab はその行のボタン、もう一度 Tab で外へ出る。行からの Shift+Tab はリストの前へ出る。Enter / Space はフォーカス中の行またはボタンだけを実行する。

- LazyList は見えていない項目へ移るとき、その項目までスクロールしてからフォーカスする。Tab の対象は、見えている押せる行のうちフォーカスを含む行を優先し、無ければ最後にフォーカスした行、最初に見えている押せる行の順に選ぶ。行内ボタンのクリックでもその行を記憶する。

## アクセシビリティ

- 上書きと補助テキストを `、` で連結し description として伝える（Compose `ListItem.kt` の mergeDescendants、WCAG 1.3.1）。

- 役割: List / ListItem（selected、位置）。行内の操作は Button（名前、任意の toggled、disabled）。

- APG の [Listbox パターン](../references/aria-practices/content/patterns/listbox/listbox-pattern.html) の About This Pattern は、option 内のボタン・リンク・チェックボックスを操作できないため Grid パターンを案内している。本リストは List / ListItem であり listbox / option ではない。Grid の役割やセルの移動モデルも実装していない。行内の操作を含む構造の実際の読み上げは未確認。

- 名前: リストは任意の `.aria_label()`、項目は表示見出しが既定で `.aria_label()` で上書き可能

- `max_height` で高さを限ったときは、フォーカスが移った項目（キーボードによる矢印・Tab・初期フォーカス）は、スクロール領域の見える範囲に最小の量だけ動かして入れる（WCAG 2.2 の 2.4.11 Focus Not Obscured）。LazyList は `scroll_to_item` で同じことをする

- `ScrollScope` と共通の `reveal_on_focus` で、祖先のスクロール領域も内側から調整する。外側のリング分も含め、両軸をスクロール可能な範囲に制限する。マウスクリックでは動かさず、ホイールで離してもフォーカスが同じなら引き戻さない。

- LazyList の仮想化領域にはスコープを足さず、`scroll_to_item` を使う。

## 既知の制限・仕様からの逸脱

- LazyList は描かれている行だけが Tab の順序に入る。押せない行の `ListAction` は、スクロールして行が見えるまでキーボードで届かないので、押せる行に付ける（WCAG 2.1.1。押せる行は矢印で移れる）。
- `ListAction` の操作で行が消える・操作が無効になるときのフォーカスの移動先は呼び出し側が決める。部品は移さない。
- 先頭の画像・動画、末尾のチェックボックスやスイッチの組み込み、並べ替え、スワイプは未実装。
- LazyList は gpui の `uniform_list` を使うため、項目の高さは全て同じ（最初の項目で測る）でなければならない。
- 選択中の項目の角（Large）は ListTokens の `ItemSelectedContainerShape` に従った。
