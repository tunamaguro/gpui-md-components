# Button 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Button.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/{Filled,Elevated,FilledTonal,Outlined,Text,Baseline}ButtonTokens.kt` @ b8256a52

## バリアント

- Elevated / Filled（既定）/ Filled tonal / Outlined / Text
- 先頭アイコンの有無

## 構成要素（アナトミー）

1. コンテナ（Elevated は影、Outlined は枠線、Text は無し）
1. 先頭アイコン（任意、18dp）
1. ラベル（Label Large）

## 寸法

| 項目                 | 値                                          | トークン                                 |
| -------------------- | ------------------------------------------- | ---------------------------------------- |
| 高さ                 | 40dp（密度で減る）                          | `baseline_button::CONTAINER_HEIGHT`      |
| 最小幅               | 58dp                                        | `layout::BUTTON_MIN_WIDTH`               |
| 左右の余白           | 24dp（アイコン付きは左 16dp、Text は 12dp） | `baseline_button::LEADING_SPACE` ほか    |
| アイコンとラベルの間 | 8dp                                         | `baseline_button::ICON_LABEL_SPACE`      |
| 角丸                 | Full                                        | `baseline_button::CONTAINER_SHAPE_ROUND` |

## 色（状態別）

| 要素               | enabled   | hover               | focused      | pressed      | disabled             |
| ------------------ | --------- | ------------------- | ------------ | ------------ | -------------------- |
| コンテナ（Filled） | Primary   | Primary + 影 Level1 | Primary      | Primary      | OnSurface 10%        |
| ラベル（Filled）   | OnPrimary | OnPrimary           | OnPrimary    | OnPrimary    | OnSurfaceVariant 38% |
| 状態レイヤー       | —         | ラベル色 8%         | ラベル色 10% | ラベル色 10% | 無し                 |

## 振る舞い

- クリック、Enter、Space で `on_click`。
- ホバーで影が 1 段上がる（Elevated / Filled / Filled tonal）。

## アクセシビリティ

- メニューのトリガーには `.menu_expanded(open)` を指定し、has-popup と expanded を伝える（APG Menu Button）。

- 役割: Button

- 名前: 表示ラベルが既定。`.aria_label()` で上書き可能

- 状態: 無効は Tab で止まらない

- キー操作: Enter / Space

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- Text ボタンのラベル色は、トークン（OnSurfaceVariant）ではなく Compose の実装（`defaultTextButtonColors`、TODO 付きで Primary）に合わせた。
- Outlined のラベル色・枠線色は Compose の現行トークン（OnSurfaceVariant / OutlineVariant）で、m3.material.io の旧ベースライン（Primary / Outline）とは違う。
- 押下時の角丸の変形（`PressedContainerShape`、Expressive）は実装していない。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。

Button 系の役割・名前・状態と操作は `gpui_headless::Button` が担う。
