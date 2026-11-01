# Badge 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Badge.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/BadgeTokens.kt` @ b8256a52

## バリアント

- 点（6dp）/ 文字付き（高さ 16dp）

## 構成要素（アナトミー）

1. Error 色の丸
1. 文字（Label Small、OnError）

## 寸法

| 項目     | 値                        | トークン                                       |
| -------- | ------------------------- | ---------------------------------------------- |
| 点       | 6dp                       | `badge::SIZE`                                  |
| 文字付き | 最小の高さ 16dp、左右 4dp | `LARGE_SIZE`, `layout::BADGE_HORIZONTAL_SPACE` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| ---- | ------- | ----- | ------- | ------- | -------- |
| 背景 | Error   | —     | —       | —       | —        |
| 文字 | OnError | —     | —       | —       | —        |

## 振る舞い

- 操作できない。`BadgedBox::new(anchor, badge)` はアンカーの大きさを変えず右上に重ねる。文字付きは横 12dp・縦 14dp、点は横・縦 6dp 重なる。

## アクセシビリティ

- 役割: なし（ツリーに出ない）。件数は重ねる先の要素のラベルなどに含めること。`NavigationItem::badge` は文字または `badge_description` を項目の説明に渡す。

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- Compose の `BadgeEndRuler` / `BadgeTopRuler` による画面端からのはみ出し防止は未実装。NavigationBar では Text scale 200% の文字付きバッジがバーの上端から 2px ほどはみ出す。
- `BadgedBox` のアンカーは自身の大きさが決まる要素に限る。`.w_full()` のように親に対する割合で幅を決める要素は幅が 0 になる（Compose の `fillMaxWidth` は親の制約を受け取るので広がる）。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。
