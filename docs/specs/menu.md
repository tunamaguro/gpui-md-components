# Menu 仕様要約

- 出典: Jetpack Compose Material 3 の `Menu.kt`（`DropdownMenu`, `DropdownMenuItem`, `MenuDefaults`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/MenuTokens.kt`, `ListTokens.kt` @ b8256a52

## バリアント

- 操作のメニュー（`Menu`）、Select の選択肢（内部の `MenuKind::Options`）

## 構成要素（アナトミー）

1. コンテナ（SurfaceContainer、角 ExtraSmall、Level 2）
1. 項目: 先頭アイコン、ラベル（Label Large）、末尾アイコンまたは文字（ショートカット表示など）

## 寸法

| 項目                 | 値                  | トークン                                                 |
| -------------------- | ------------------- | -------------------------------------------------------- |
| 幅                   | 112〜280dp          | `layout::MENU_MIN_WIDTH`, `MENU_MAX_WIDTH`               |
| 項目の高さ           | 48dp                | `layout::MENU_ITEM_HEIGHT`                               |
| 項目の左右           | 12dp                | `layout::MENU_ITEM_HORIZONTAL_SPACE`                     |
| 上下の余白           | 8dp                 | `layout::MENU_VERTICAL_SPACE`                            |
| ウィンドウ端との間隔 | 上下 48dp、左右 8dp | `layout::MENU_VERTICAL_MARGIN`, `MENU_HORIZONTAL_MARGIN` |

## 色（状態別）

| 要素         | enabled          | hover        | focused       | pressed       | disabled      |
| ------------ | ---------------- | ------------ | ------------- | ------------- | ------------- |
| 項目の文字   | OnSurface        | OnSurface    | OnSurface     | OnSurface     | OnSurface 38% |
| アイコン     | OnSurfaceVariant | —            | —             | —             | OnSurface 38% |
| 状態レイヤー | —                | OnSurface 8% | OnSurface 10% | OnSurface 10% | —             |

## 振る舞い

- スクロール可能なメニューでは、領域が動ける方向のホイールを消費する。端から始まる入力だけ祖先へ渡し、開始フェーズのある精密スクロールは終了まで消費を保持する。

- 基準の要素の真下に、ウィンドウからはみ出さないように出す。高すぎる場合はスクロールする。

- 開くと最初の有効な項目にフォーカスする。閉じると開いた要素にフォーカスを戻す。

- 項目を選ぶと `on_select` を呼んでから閉じる。

## アクセシビリティ

- 役割: Menu / MenuItem
- 名前: メニューのラベル、項目のラベル
- 状態: 無効な項目は Tab・矢印で止まらない
- キー操作: ↑↓ で移動（無効を飛ばし、端で回る）、Home / End、Enter / Space で選ぶ、Esc / Tab で閉じる

## 既知の制限・仕様からの逸脱

- サブメニュー、区切り線、項目の文字の頭文字での移動は未実装。
- 開いた要素の expanded 状態は呼び出し側で付ける（`aria_expanded`）。
