# Chip 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Chip.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/{Assist,Filter,Input,Suggestion}ChipTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- Assist / Filter / Input / Suggestion
- Flat（枠線）/ Elevated（影）

## 構成要素（アナトミー）

1. コンテナ（32dp、角丸 Small）
2. 先頭アイコン（18dp、Filter の選択中はチェック）
3. ラベル（Label Large）
4. 末尾の削除アイコン（Input）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 高さ | 32dp（密度で減る） | `assist_chip::CONTAINER_HEIGHT` |
| 左右の余白 | 8dp + ラベルの左右 8dp | `layout::CHIP_*` |
| 枠線 | 1dp | `FLAT_OUTLINE_WIDTH` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| 枠線（Flat） | OutlineVariant | 同 | 同 | 同 | OnSurface 12% |
| コンテナ（Filter 選択） | SecondaryContainer | 同 | 同 | 同 | OnSurface 12% |
| ラベル | OnSurface / OnSurfaceVariant | 同 | 同 | 同 | OnSurface 38% |

## 振る舞い

- Filter は `on_click` で呼び出し側が選択を反転する。
- Input は Backspace / Delete か削除アイコンで `on_remove`。

## アクセシビリティ

- 役割: Filter は CheckBox（toggled）、他は Button
- 名前: ラベル
- キー操作: Enter / Space、Input は Backspace / Delete で削除

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- Input chip の先頭のアバターは未対応。
- 削除アイコンは独立したフォーカス位置を持たない（チップ自体のキー操作で削除する）。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。
