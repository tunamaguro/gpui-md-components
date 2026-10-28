# Slider 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Slider.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SliderTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- 連続値 / 離散値（`step`）

## 構成要素（アナトミー）

1. 活性トラック（16dp、外側の角は Full、つまみ側は 2dp）
2. つまみ（4×44dp の棒、押下・フォーカスで幅 2dp）
3. 非活性トラック
4. 終端の点（4dp）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| つまみ | 4×44dp | `slider::HANDLE_WIDTH`, `HANDLE_HEIGHT` |
| トラック | 16dp | `INACTIVE_TRACK_HEIGHT` |
| つまみとトラックの間 | 6dp | `ACTIVE_HANDLE_LEADING_SPACE` |
| 内側の角 | 2dp | `layout::SLIDER_TRACK_INSIDE_CORNER` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| 活性トラック・つまみ | Primary | 同 | 同 | 同 | OnSurface 38% |
| 非活性トラック | SecondaryContainer | 同 | 同 | 同 | OnSurface 12% |

## 振る舞い

- クリックとドラッグで値が変わる。
- 矢印で 1 ステップ（連続値は範囲の 1%）、Page Up / Down で 10 ステップ、Home / End で端。

## アクセシビリティ

- 役割: Slider
- 名前: ラベル引数（必須）
- 状態: value / min / max / step
- キー操作: APG の Slider

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 値のラベル（ドラッグ中の吹き出し）、目盛り、範囲スライダー、縦向きは未実装。
- 終端の点の色は、トークン（StopIndicatorColor = SecondaryContainer、非活性トラックと同色）ではなく活性トラックの色にした。
- フォーカスリングはつまみではなくスライダー全体を囲む。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
