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

| 項目                   | 値                                        | トークン                                                   |
| ---------------------- | ----------------------------------------- | ---------------------------------------------------------- |
| 高さ                   | 56dp                                      | `outlined_text_field::CONTAINER_HEIGHT`                    |
| 最小幅                 | 280dp                                     | `layout::TEXT_FIELD_MIN_WIDTH`                             |
| 左右の余白             | 16dp（アイコンがあれば 12dp + 48dp の枠） | `layout::TEXT_FIELD_HORIZONTAL_SPACE`                      |
| 下線                   | 1dp、フォーカスで 2dp                     | `ACTIVE_INDICATOR_HEIGHT`, `FOCUS_ACTIVE_INDICATOR_HEIGHT` |
| 枠線                   | 1dp、フォーカスで 2dp                     | `OUTLINE_WIDTH`, `FOCUS_OUTLINE_WIDTH`                     |
| ラベルの切り欠きの余白 | 4dp                                       | `layout::OUTLINED_TEXT_FIELD_LABEL_GAP`                    |
| 補助テキストの上       | 4dp                                       | `layout::TEXT_FIELD_SUPPORTING_TOP_SPACE`                  |

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
- 1 行入力。はみ出した分はキャレットが見えるように横にスクロールする。
- 編集キー: ←→、Shift+←→、Home / End、Shift+Home / End、Backspace / Delete、Ctrl(⌘)+A / C / X / V。
- IME の変換中の文字は下線付きで表示し、確定で置き換える（`EntityInputHandler`）。
- `max_length` を指定すると「n/max」のカウンタを出す（入力は制限しない。超過時のエラー表示は呼び出し側）。

## アクセシビリティ

- キーボードでフォーカスが移ると、祖先の `ScrollScope` の表示領域へ移す（WCAG 2.2 の 2.4.11）。
  `reveal_on_focus` を共有し、マウスクリックやフォーカスが変わらないホイール操作では引き戻さない。

- 役割: TextInput

- 名前: ラベル

- 状態: value（入力文字）、placeholder

- キー操作: 通常の文字入力、IME

## デスクトップ向け調整（密度など）

- 高さは密度で変えない（Compose も TextField の高さは密度に追従しない）。

## 既知の制限・仕様からの逸脱

- 複数行入力（textarea）は未実装。
- 依存を増やせないため、書記素の境界は簡易判定（結合文字、異体字セレクタ、ZWJ、肌色修飾、地域指示子のペア）。Unicode の全規則は満たさない。
- 単語単位の移動（Ctrl+←→）、取り消し（Undo）は未実装。
- gpui の a11y API に invalid / required 状態がないため、エラーはツリーに出ない（補助テキストに理由を書くこと）。
- `init` でキーを割り当てる。wasm では OS を区別できないため Ctrl を使う。

## 既定モーション

Filled TextField（共通の FieldFrame を使う Filled Select も同様）の下線幅を FastSpatial で補間する。disabled と reduce_motion は即時変更。参照: TextField.kt L815–826 @ b8256a52。Outlined 枠線幅とラベルの移動・縮小は未対応（ラベル切り欠きと IME 入力レイアウトを合わせた変更が必要）。
