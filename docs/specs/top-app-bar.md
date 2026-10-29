# TopAppBar 仕様要約

- 出典: Jetpack Compose Material 3 の `AppBar.kt`（`TopAppBar`, `CenterAlignedTopAppBar`, `MediumTopAppBar`, `LargeTopAppBar`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/AppBarTokens.kt`, `AppBarSmallTokens.kt`, `AppBarMediumTokens.kt`, `AppBarLargeTokens.kt` @ b8256a52

## バリアント

- Small / CenterAligned（64dp）、Medium（112dp）、Large（152dp）

## 構成要素（アナトミー）

1. コンテナ（Surface。スクロール後は SurfaceContainer）
1. ナビゲーションのアイコン（IconButton、OnSurface）
1. タイトル（Small: Title Large、Medium: Headline Small、Large: Headline Medium）
1. 操作のアイコン（IconButton、OnSurfaceVariant）

## 寸法

| 項目                             | 値                  | トークン                                                   |
| -------------------------------- | ------------------- | ---------------------------------------------------------- |
| 高さ                             | 64 / 112 / 152dp    | `app_bar_small/medium/large::CONTAINER_HEIGHT`             |
| 左右の余白                       | 4dp                 | `layout::TOP_APP_BAR_HORIZONTAL_SPACE`                     |
| アイコンが無いときのタイトルの前 | 12dp（端から 16dp） | `layout::TOP_APP_BAR_TITLE_INSET`                          |
| Medium / Large のタイトルの下    | 24 / 28dp           | `layout::MEDIUM_TOP_APP_BAR_TITLE_BOTTOM_SPACE`, `LARGE_…` |

## 振る舞い

- CenterAligned は先頭と末尾のアイコン群の幅（密度と間隔を含む）の大きい方を左右に確保する。タイトルはその範囲で中央揃えし、長い場合は省略記号で切る。

- `.scrolled(true)` で色を SurfaceContainer に変える（スクロールの監視は呼び出し側）。

## アクセシビリティ

- 役割: Header（名前はタイトル）。タイトルは Heading
- 操作はアイコンボタン（各ボタンのラベル）

## 既知の制限・仕様からの逸脱

- スクロールに合わせて縮む・色が徐々に変わるアニメーションは未実装。
- 2 行のタイトル、副題（subtitle）は未実装。
