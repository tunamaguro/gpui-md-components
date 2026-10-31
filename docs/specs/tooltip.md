# Tooltip 仕様要約

- 出典: Jetpack Compose Material 3 の `Tooltip.kt`（`PlainTooltip`, `RichTooltip`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PlainTooltipTokens.kt`, `RichTooltipTokens.kt` @ b8256a52

## バリアント

- Plain（短い説明）/ Rich（見出し、本文、操作のボタン）

## 構成要素（アナトミー）

1. Plain: InverseSurface のコンテナ（角 ExtraSmall）、Body Small の文字（InverseOnSurface）
1. Rich: SurfaceContainer のコンテナ（角 Medium、Level 2）、見出し（Title Small）、本文（Body Medium）、Text Button

## 寸法

| 項目           | 値                 | トークン                                                                 |
| -------------- | ------------------ | ------------------------------------------------------------------------ |
| Plain の余白   | 上下 4dp、左右 8dp | `layout::PLAIN_TOOLTIP_VERTICAL_SPACE`, `PLAIN_TOOLTIP_HORIZONTAL_SPACE` |
| Plain の最大幅 | 200dp              | `layout::PLAIN_TOOLTIP_MAX_WIDTH`                                        |
| Rich の最大幅  | 320dp              | `layout::RICH_TOOLTIP_MAX_WIDTH`                                         |
| Rich の左右    | 16dp               | `layout::RICH_TOOLTIP_HORIZONTAL_SPACE`                                  |
| 最小           | 40 × 24dp          | `layout::TOOLTIP_MIN_WIDTH`, `TOOLTIP_MIN_HEIGHT`                        |

## 振る舞い

- `TooltipExt::md3_tooltip(text)` で、ポインタを重ねると Plain を出す。
- `TooltipExt::md3_focus_tooltip`（IconButton は `.tooltip(text)`）は、キーボードでフォーカスしても Plain を出す。基準の要素の上に中央揃えで置き（4dp の間隔）、入らなければ下に置く。Esc で閉じ、フォーカスが戻るとまた出る。Dialog の中でも Esc はツールチップを先に閉じる（APG の Tooltip パターン）。
- Rich は `RichTooltip::builder()` を gpui の `hoverable_tooltip` に渡す（ポインタを移して操作できる）。

## アクセシビリティ

- 役割: Tooltip
- 名前: 文字
- `md3_tooltip` と `md3_focus_tooltip` は基準の要素の説明（description）にも同じ文を入れる。

## 既知の制限・仕様からの逸脱

- gpui のツールチップはポインタを重ねたときだけ出る。ポインタで出たものは Esc で閉じられない。出る位置は gpui が決める（ポインタの近く）。
- Rich と、`md3_tooltip` だけを付けた要素はフォーカスでは出ない。
- 文字の行の位置は Compose ではベースラインで合わせるが、ここでは行の下端で近似した。
