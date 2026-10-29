# Divider 仕様要約

- 出典: Jetpack Compose Material 3 の `Divider.kt`（`HorizontalDivider`, `VerticalDivider`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DividerTokens.kt` @ b8256a52

## バリアント

- 横線 / 縦線。始端・終端を空ける（inset）

## 寸法・色

| 項目 | 値             | トークン             |
| ---- | -------------- | -------------------- |
| 太さ | 1dp            | `divider::THICKNESS` |
| 色   | OutlineVariant | `divider::COLOR`     |

## アクセシビリティ

- 装飾なのでツリーに出ない。

## 既知の制限・仕様からの逸脱

- inset の量は呼び出し側が渡す（リストの項目に合わせるなら 16dp など）。
