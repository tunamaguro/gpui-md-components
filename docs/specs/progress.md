# Progress indicator 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`ProgressIndicator.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/{Linear,Circular}ProgressIndicatorTokens.kt, ProgressIndicatorTokens.kt` @ b8256a52

## バリアント

- 線 / 円
- 確定（値あり）/ 不定（`None`）

## 構成要素（アナトミー）

1. 活性部分（Primary）
1. すき間（4dp）
1. トラック（SecondaryContainer）
1. 終端の点（線の確定のみ）

## 寸法

| 項目       | 値             | トークン                                     |
| ---------- | -------------- | -------------------------------------------- |
| 線の太さ   | 4dp            | `linear_progress_indicator::TRACK_THICKNESS` |
| 円の大きさ | 40dp、太さ 4dp | `circular_progress_indicator::SIZE`          |
| すき間     | 4dp            | `TRACK_ACTIVE_SPACE`                         |

## 色（状態別）

| 要素     | enabled            | hover | focused | pressed | disabled |
| -------- | ------------------ | ----- | ------- | ------- | -------- |
| 活性部分 | Primary            | —     | —       | —       | —        |
| トラック | SecondaryContainer | —     | —       | —       | —        |

## 振る舞い

- 不定は繰り返しアニメーションする（線 1750ms、円 6000ms 周期）。

## アクセシビリティ

- 役割: ProgressIndicator
- 名前: `.aria_label()`（任意）
- 状態: 確定なら value（0〜1）、不定なら値なし

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 不定のアニメーションは Compose（2 本の線の追いかけ、円の回転と伸縮の合成）を簡略化した。
- 波形（wavy）の表示は未実装。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。
