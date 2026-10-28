# SegmentedButton 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`SegmentedButton.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/OutlinedSegmentedButtonTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- 単一選択（RadioGroup として振る舞う）/ 複数選択（各項目がトグル）

## 構成要素（アナトミー）

1. 枠線で区切った項目の並び
2. 選択中の項目はチェックアイコン + SecondaryContainer

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 高さ | 40dp（密度で減る） | `outlined_segmented_button::CONTAINER_HEIGHT` |
| 枠線 | 1dp | `OUTLINE_WIDTH` |
| 左右の余白 | 12dp | `layout::SEGMENTED_BUTTON_HORIZONTAL_SPACE` |
| アイコン | 18dp、ラベルとの間 8dp | `ICON_SIZE`, `layout::SEGMENTED_BUTTON_ICON_SPACE` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| コンテナ（選択） | SecondaryContainer | 同 | 同 | 同 | — |
| ラベル（非選択） | OnSurface | 同 | 同 | 同 | OnSurface 38% |
| 枠線 | Outline | 同 | 同 | 同 | OnSurface 12% |

## 振る舞い

- 単一選択: 矢印キーでフォーカスと選択が一緒に移る。Tab で止まるのは選択中の項目だけ。
- 複数選択: 矢印キーでフォーカスだけが移り、Space / Enter で反転する。

## アクセシビリティ

- 役割: RadioGroup + RadioButton（単一）、Group + Button（toggled、複数）
- 名前: グループのラベル、各項目のラベル
- 状態: selected / toggled、位置（n / 全体）
- キー操作: 矢印で移動（端で反対側へ回る）

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 項目ごとに角丸が違うため、状態レイヤーとフォーカスリングは端の項目の外側の角丸に合わせた近似。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
