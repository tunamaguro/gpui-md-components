# SegmentedButton 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`SegmentedButton.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/OutlinedSegmentedButtonTokens.kt` @ b8256a52

## バリアント

- 単一選択（RadioGroup として振る舞う）/ 複数選択（各項目がトグル）

## 構成要素（アナトミー）

1. 枠線で区切った項目の並び
1. 選択中の項目はチェックアイコン + SecondaryContainer

## 寸法

| 項目       | 値                     | トークン                                           |
| ---------- | ---------------------- | -------------------------------------------------- |
| 高さ       | 40dp（密度で減る）     | `outlined_segmented_button::CONTAINER_HEIGHT`      |
| 枠線       | 1dp                    | `OUTLINE_WIDTH`                                    |
| 左右の余白 | 12dp                   | `layout::SEGMENTED_BUTTON_HORIZONTAL_SPACE`        |
| アイコン   | 18dp、ラベルとの間 8dp | `ICON_SIZE`, `layout::SEGMENTED_BUTTON_ICON_SPACE` |

## 色（状態別）

| 要素             | enabled            | hover | focused | pressed | disabled      |
| ---------------- | ------------------ | ----- | ------- | ------- | ------------- |
| コンテナ（選択） | SecondaryContainer | 同    | 同      | 同      | —             |
| ラベル（非選択） | OnSurface          | 同    | 同      | 同      | OnSurface 38% |
| 枠線             | Outline            | 同    | 同      | 同      | OnSurface 12% |

## 振る舞い

- 状態レイヤーとフォーカスリングは本体と同じ四隅の半径を使う。先頭・末尾は外側だけ丸め、中間は角丸なし。1 項目だけの場合は四隅を丸める。

- 単一選択: 矢印キーでフォーカスと選択が一緒に移る。Tab で止まるのは選択中の項目だけ。

- 複数選択: 矢印キーでフォーカスだけが移り、Space / Enter で反転する。

## アクセシビリティ

- 役割: RadioGroup + RadioButton（単一）、Group + Button（toggled、複数）
- 名前: グループのラベル、各項目のラベル
- 状態: checked（単一。AccessKit の `toggled`、APG Radio Group に合わせる）/ toggled（複数。pressed として出る）、位置（n / 全体）
- キー操作: 矢印で移動（端で反対側へ回る）

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。

操作とアクセシビリティ情報は `gpui-headless` が持ち、寸法と描画は `md3-components` が担う。
