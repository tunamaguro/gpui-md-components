# TextField 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`TextField.kt`, `OutlinedTextField.kt`, `TextFieldDefaults.kt`）とトークン。入力処理は gpui の `examples/input.rs` を移植した。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledTextFieldTokens.kt`, `OutlinedTextFieldTokens.kt` @ b8256a52

## バリアント

- Filled / Outlined（`TextFieldVariant`）

## 構成要素（アナトミー）

1. コンテナ（Filled: SurfaceContainerHighest、上側の角だけ丸い / Outlined: 透明、枠線）
1. 下線（Filled）または枠線（Outlined。浮いたラベルの位置は枠線を切り欠く）
1. ラベル（空でフォーカスがないときは中央に Body Large、それ以外は上に Body Small）
1. 入力文字（Body Large）、placeholder、接頭辞・接尾辞
1. 先頭・末尾のアイコン（24dp、48dp の枠）
1. 補助テキスト・文字数カウンタ（Body Small）

## 寸法

| 項目                   | 値                                                                                      | トークン                                                                              |
| ---------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 高さ                   | 56dp（最小値。Outlined はラベルがあると上に行の高さの半分を空ける）                     | `outlined_text_field::CONTAINER_HEIGHT`                                               |
| 複数行の上下の余白     | Filled + ラベルは 8dp、それ以外は 16dp                                                  | `layout::TEXT_FIELD_WITH_LABEL_VERTICAL_SPACE`, `layout::TEXT_FIELD_HORIZONTAL_SPACE` |
| 最小幅                 | 280dp                                                                                   | `layout::TEXT_FIELD_MIN_WIDTH`                                                        |
| 左右の余白             | 16dp（アイコンがあれば 4dp + 48dp の枠。複数行の右は 24dp、末尾のアイコンがあれば 4dp） | `layout::TEXT_FIELD_HORIZONTAL_SPACE`                                                 |
| 下線                   | 1dp、フォーカスで 2dp                                                                   | `ACTIVE_INDICATOR_HEIGHT`, `FOCUS_ACTIVE_INDICATOR_HEIGHT`                            |
| 枠線                   | 1dp、フォーカスで 2dp                                                                   | `OUTLINE_WIDTH`, `FOCUS_OUTLINE_WIDTH`                                                |
| ラベルの切り欠きの余白 | 4dp                                                                                     | `layout::OUTLINED_TEXT_FIELD_LABEL_GAP`                                               |
| 補助テキストの上       | 4dp                                                                                     | `layout::TEXT_FIELD_SUPPORTING_TOP_SPACE`                                             |

## 色（状態別）

| 要素       | enabled                    | hover            | focused   | error                              | disabled                                 |
| ---------- | -------------------------- | ---------------- | --------- | ---------------------------------- | ---------------------------------------- |
| 下線・枠線 | OnSurfaceVariant / Outline | OnSurface        | Primary   | Error（hover は OnErrorContainer） | OnSurface 38%（Filled）/ 12%（Outlined） |
| ラベル     | OnSurfaceVariant           | OnSurfaceVariant | Primary   | Error                              | OnSurface 38%                            |
| 入力文字   | OnSurface                  | OnSurface        | OnSurface | OnSurface                          | OnSurface 38%                            |
| キャレット | Primary                    | —                | Primary   | Error                              | —                                        |
| 選択範囲   | Primary 40%                | —                | —         | —                                  | —                                        |

## 振る舞い

- 中身は `Entity<TextFieldState>` が持ち、変更は `gpui_headless::TextFieldEvent::Changed` で通知する。
- 既定は 1 行入力。はみ出した分はキャレットが見えるように横にスクロールする。
- `.multi_line(min_lines, max_lines)` で複数行入力にする（Compose の `TextFieldLineLimits.MultiLine`）。欄の幅で折り返し、高さは `min_lines` 行から `max_lines` 行の間で内容に合わせて変わる。`max_lines` を超えた分は欄の中で縦にスクロールする。上限なしは `usize::MAX`。
  カーソルが動くと、欄の中と祖先の `ScrollScope` の中でカーソルが見える位置まで送る。ホイールやスクロールバーで動かした位置は再描画で戻さない。端に達したホイールは祖先へ渡す（`NestedScroll` と同じ）。無効の欄もホイールとスクロールバーで隠れた行を読める。
- 隠れた行がある間はスクロールバーを出す。Compose の TextField はバーを持たず、MD3 の Text fields の Accessibility タブは本文を読めなかったので、下の「アクセシビリティ」の WCAG の項に従う。
  入力の右の余白の中央に置く。余白は隠れた行の有無によらず同じなので、バーが出ても折り返しは変わらない。縦の範囲は入力の行で、トラックのクリックは入力の高さの分だけ動く。
- 複数行の配置（Compose の `TextFieldMeasurePolicy`）: 入力とラベルは上に寄せ、先頭・末尾のアイコンは縦の中央に置く。空でフォーカスがないときも入力の行を確保するので、フォーカスで高さは変わらない（折り返す placeholder を除く）。
- 改行は LF で持つ。貼り付けや `set_text` の CRLF / CR は LF にそろえる。1 行の欄では、入力された改行は空白にし、改行だけの入力は無視する。`set_text` / `with_text` は欄の種類を知らないので LF を残し、1 行の欄では空白として表示する。
- 編集キー: ←→、Shift+←→、Home / End、Shift+Home / End（表示行の先頭・末尾）、Backspace / Delete、Ctrl(⌘)+A / C / X / V。
- 複数行だけのキー: Enter / Shift+Enter（改行。変換中は何もしない）、↑↓、Shift+↑↓、PageUp / PageDown、Shift+PageUp / PageDown（見えている行数）、Ctrl+Home / End、Ctrl+Shift+Home / End（macOS は ⌘+↑↓、⌘+Shift+↑↓）。↑↓ は x を保ち、先頭より上は文字列の先頭、末尾より下は文字列の末尾へ移す（Compose の `jumpByLinesOffset`）。1 行の欄ではこれらのキーは親の要素へ渡る。
- Tab は文字を入れずフォーカスを移す（WCAG 2.1.2）。
- IME の変換中の文字は下線付きで表示し、確定で置き換える（`EntityInputHandler`）。
- 入力欄からフォーカスが移ると変換範囲を解除し、文字を残す。ウィンドウの非アクティブ化だけでは解除しない。Windows では入力欄の内外をクリックしても文字を残して変換を終了し、欄内ならクリック位置にキャレットを移す。Linux のクリックは gpui の処理に委ねる。macOS / Web は未確認。
- 参照: Compose foundation `TextFieldDecoratorModifier.kt` の `onFocusChange` @ b8256a52、および [Apple `NSTextInputClient.unmarkText()`](<https://developer.apple.com/documentation/appkit/nstextinputclient/unmarktext()>)。
- Windows の変換終了処理中は一時的に入力を受け付けず、範囲を指定しない文字の置換を無視する。この間の通常の文字入力や Backspace / Delete / 貼り付けなどは反映されないことがある。Backspace / Delete は削除前に選択範囲を変える場合があり、切り取りはコピー後の削除が無視される場合がある。
- フォーカス中は描画後に入力位置を IME に伝え、その位置や欄の範囲が変われば更新する。変換中は変換範囲の先頭、それ以外は選択の末端を使う。欄からはみ出す位置は欄の中に収め、空欄では入力の左上を返す。複数行で範囲が複数の表示行にまたがるときは、開始行の右端までを返す。
- `max_length` を指定すると「n/max」のカウンタを出す（入力は制限しない。超過時のエラー表示は呼び出し側）。

## アクセシビリティ

- 補助テキストを description にし、Status ノードとしても出す。エラー時は invalid と live（polite）を付ける（WCAG 1.3.1、3.3.1、4.1.3）。`.required(true)` は必須状態を伝え、見える印はラベルに付ける。

- キーボードでフォーカスが移ると、祖先の `ScrollScope` の表示領域へ移す（WCAG 2.2 の 2.4.11）。
  `reveal_on_focus` を共有し、マウスクリックやフォーカスが変わらないホイール操作では引き戻さない。

- 複数行のスクロールバー（Compose に無いので WCAG 2.2 で決めた）:

  - 1.4.11: 隠れた行があることを示す印はバーだけなので、フェードさせず、つまみは Outline の不透明度 1.0 にする。トークンの 0.7 は既定の配色の Filled で 2.28:1（ライト）/ 2.68:1（ダーク）と 3:1 に届かない。1.0 なら 3.51:1 / 3.87:1。
  - 2.5.8: 操作領域は幅 24dp で、文字に重ねない。トラックが 24dp 未満なら描かないので、高さも 24dp 以上ある。
  - 2.5.7: トラックのクリックでページ移動できるので、ドラッグなしで操作できる。
  - 2.1.1: バーはフォーカスを持たず、キーボードでは ↑↓ / PageUp / PageDown でカーソルと一緒に動く。APG に textbox のパターンはない。

- 役割: TextInput（複数行は MultilineTextInput）

- 名前: 任意の `.label()` が既定。`.aria_label()` で上書きできる

- 状態: value（入力文字）、placeholder

- キー操作: 通常の文字入力、IME

- 支援技術: 文字とカーソルを段落ごとの TextRun で渡し（値は末尾の改行を含む）、SetValue / ReplaceSelectedText / SetTextSelection を受ける（WCAG 4.1.2。Compose の `setText` など）。無効のときは受けない。OS から届く操作は AccessKit のアダプタ次第で、Linux（AT-SPI）は EditableText がないので SetTextSelection だけが届く。ReplaceSelectedText を送るアダプタは今はない

## デスクトップ向け調整（密度など）

- 高さは密度で変えない（Compose も TextField の高さは密度に追従しない）。

## 既知の制限・仕様からの逸脱

- 選択のドラッグは欄の外やスクロールバーの上でも続くが、複数行で欄の外に出ても時間で送り続けない（マウスが動いた分だけ送る）。
- 複数行の支援技術の行は段落単位で、折り返しの位置を見ない。
- Text scale 100% の高さ 1 行の欄（`.multi_line(1, 1)`）はトラックが最小長の 24dp に届かず、スクロールバーを描かない（`docs/specs/scrollbar.md`）。
- 複数行の右の余白は 24dp で、Compose の 16dp より広い。スクロールバーの操作領域を文字に重ねないため（WCAG 2.5.8）で、隠れた行の有無によらず同じ幅にして折り返しを変えない。末尾のアイコンがある欄の余白は 1 行の欄と同じ 4dp で、操作領域は装飾のアイコンの枠に 20dp かかる。
- ラベルのある欄の placeholder はフォーカス中だけ出るので、折り返すとフォーカスで高さが変わる。
- Web の IME 候補ウィンドウは gpui_web の制限により左上に表示される。
- 依存を増やせないため、書記素の境界は簡易判定（結合文字、異体字セレクタ、ZWJ、肌色修飾、地域指示子のペア）。Unicode の全規則は満たさない。
- 単語単位の移動（Ctrl+←→）、取り消し（Undo）は未実装。
- Linux の AT-SPI には invalid が出ない（`docs/a11y-audit.md`）。
- `init` でキーを割り当てる。wasm では OS を区別できないため Ctrl を使う。

## 既定モーション

Filled TextField（共通の FieldFrame を使う Filled Select も同様）の下線幅を FastSpatial で補間する。disabled と reduce_motion は即時変更。参照: TextField.kt L815–826 @ b8256a52。Outlined 枠線幅とラベルの移動・縮小は未対応（ラベル切り欠きと IME 入力レイアウトを合わせた変更が必要）。
