# Sheet（BottomSheet / SideSheet）仕様要約

- 出典: Jetpack Compose Material 3 の `ModalBottomSheet.kt`, `SheetDefaults`。サイドシートは Compose に無いため material-web のトークンと MDC Android のカタログの余白を使った。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SheetBottomTokens.kt`, `ScrimTokens.kt` @ b8256a52、material-web `_md-comp-sheet-side.scss` @ 86cb0d48

## バリアント

- BottomSheet（モーダル）
- SideSheet: Standard（画面の一部として並べる）/ Modal（スクリムの上に重ねる）

## 構成要素（アナトミー）

1. スクリム（モーダルのみ、Scrim 32%）
1. コンテナ（BottomSheet: SurfaceContainerLow、上の角 ExtraLarge、Level 1 / SideSheet Standard: Surface、左の区切り線 / Modal: SurfaceContainerLow、左の角 Large、Level 1）
1. ドラッグハンドル（BottomSheet、32 × 4dp、OnSurfaceVariant）
1. 見出し（SideSheet、Title Large）と閉じるボタン

## 寸法

| 項目                 | 値                    | トークン                                                               |
| -------------------- | --------------------- | ---------------------------------------------------------------------- |
| BottomSheet の最大幅 | 640dp                 | `layout::BOTTOM_SHEET_MAX_WIDTH`                                       |
| ハンドルの上下       | 22dp                  | `layout::BOTTOM_SHEET_DRAG_HANDLE_VERTICAL_SPACE`                      |
| SideSheet の幅       | 256dp                 | `side_sheet` のトークン                                                |
| SideSheet の余白     | 16dp、見出しの下 24dp | `layout::SIDE_SHEET_CONTENT_SPACE`, `SIDE_SHEET_HEADLINE_BOTTOM_SPACE` |

## 振る舞い

- モーダルなシートは Dialog と同じく、開くと中にフォーカスを移し、Tab は中で回り、Esc とスクリムのクリックで閉じ、閉じると元にフォーカスを戻す。
- `BottomSheet::handle_label` を付けるとハンドルが閉じるボタンになる（キーボードでも閉じられる）。

## アクセシビリティ

- 役割: BottomSheet と Modal の SideSheet は Dialog、Standard の SideSheet は Complementary
- 名前: 任意の `.aria_label()`。SideSheet は `.headline()` が既定の名前

## 既知の制限・仕様からの逸脱

- ドラッグで開閉・高さを変える操作、部分的に開いた状態（half expanded）は未実装。
- 開閉のアニメーションはない。
- gpui の a11y API に modal の印がない（`dialog.md` と同じ）。
