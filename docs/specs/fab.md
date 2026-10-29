# FAB / Extended FAB 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`FloatingActionButton.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/Fab{Baseline,Small,Large,PrimaryContainer,SecondaryContainer}Tokens.kt, ExtendedFabPrimaryTokens.kt` @ b8256a52

## バリアント

- 大きさ: Small（40dp）/ Regular（56dp）/ Large（96dp）
- 色: PrimaryContainer / SecondaryContainer
- Extended FAB（ラベル付き、アイコン任意）

## 構成要素（アナトミー）

1. コンテナ（影 Level3）
1. アイコン
1. ラベル（Extended のみ）

## 寸法

| 項目            | 値                                        | トークン                                                    |
| --------------- | ----------------------------------------- | ----------------------------------------------------------- |
| 大きさ          | 40 / 56 / 96dp                            | `fab_small` / `fab_baseline` / `fab_large::CONTAINER_WIDTH` |
| 角丸            | Medium / Large / ExtraLarge               | `*::CONTAINER_SHAPE`                                        |
| Extended の高さ | 56dp                                      | `extended_fab_primary::CONTAINER_HEIGHT`                    |
| Extended の余白 | アイコン左 16dp、ラベル左右 20dp、間 12dp | `layout::EXTENDED_FAB_*`                                    |

## 色（状態別）

| 要素     | enabled                   | hover    | focused | pressed | disabled |
| -------- | ------------------------- | -------- | ------- | ------- | -------- |
| コンテナ | PrimaryContainer + Level3 | + Level4 | Level3  | Level3  | —        |
| アイコン | OnPrimaryContainer        | 同       | 同      | 同      | —        |

## 振る舞い

- MD3 の FAB に無効状態はない。

## アクセシビリティ

- 役割: Button
- 名前: ラベル引数（必須）
- キー操作: Enter / Space

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- Tertiary / Surface の色の FAB はトークンを転記していないため未対応。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。
