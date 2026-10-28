# IconButton 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`IconButton.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/{Standard,Filled,FilledTonal,Outlined,Small}IconButtonTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- Standard（既定）/ Filled / Filled tonal / Outlined
- 通常 / トグル（選択・非選択）

## 構成要素（アナトミー）

1. コンテナ（Standard は無し、Outlined は枠線）
2. アイコン（24dp）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 大きさ | 40dp（密度で減る） | `small_icon_button::CONTAINER_HEIGHT` |
| アイコン | 24dp | `small_icon_button::ICON_SIZE` |
| 枠線 | 1dp | `small_icon_button::OUTLINED_OUTLINE_WIDTH` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| アイコン（Standard） | OnSurfaceVariant | 同 | 同 | 同 | OnSurface 38% |
| コンテナ（Filled、選択） | Primary | 同 | 同 | 同 | OnSurface 10% |
| コンテナ（Filled、非選択） | SurfaceContainer | 同 | 同 | 同 | OnSurface 10% |

## 振る舞い

- トグルは呼び出し側が選択状態を持ち、`on_click` で反転する。

## アクセシビリティ

- 役割: Button
- 名前: ラベル引数（必須）
- 状態: トグルなら toggled
- キー操作: Enter / Space

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 選択時の角丸の変化（Expressive）は実装していない。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
