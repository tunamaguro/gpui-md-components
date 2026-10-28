# Badge 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Badge.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/BadgeTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- 点（6dp）/ 文字付き（高さ 16dp）

## 構成要素（アナトミー）

1. Error 色の丸
2. 文字（Label Small、OnError）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 点 | 6dp | `badge::SIZE` |
| 文字付き | 高さ 16dp、左右 4dp | `LARGE_SIZE`, `layout::BADGE_HORIZONTAL_SPACE` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| 背景 | Error | — | — | — | — |
| 文字 | OnError | — | — | — | — |

## 振る舞い

- 操作できない。重ねる位置は呼び出し側が決める。

## アクセシビリティ

- 役割: なし（ツリーに出ない）。件数は重ねる先の要素のラベルに含めること。

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- Compose の `BadgedBox`（アンカーへの自動配置）は未実装。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
