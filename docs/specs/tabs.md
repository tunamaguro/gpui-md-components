# Tabs 仕様要約

- 出典: Jetpack Compose Material 3 の `Tab.kt`, `TabRow.kt`（`PrimaryTabRow`, `SecondaryTabRow`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PrimaryNavigationTabTokens.kt`, `SecondaryNavigationTabTokens.kt`, `DividerTokens.kt` @ b8256a52

## バリアント

- Primary（印は内容の幅、最小 24dp、上の角 3dp。アイコンはラベルの上）
- Secondary（印はタブの幅いっぱい。アイコンはラベルの左）

## 寸法

| 項目         | 値                                                | トークン                                                               |
| ------------ | ------------------------------------------------- | ---------------------------------------------------------------------- |
| 高さ         | 48dp（Primary でアイコンとラベルの両方なら 72dp） | `primary_navigation_tab::CONTAINER_HEIGHT`, `layout::TAB_LARGE_HEIGHT` |
| ラベルの左右 | 16dp                                              | `layout::TAB_HORIZONTAL_SPACE`                                         |
| 印の太さ     | 3dp                                               | `primary_navigation_tab::ACTIVE_INDICATOR_HEIGHT`                      |
| 下の区切り線 | 1dp、OutlineVariant                               | `divider::THICKNESS`, `divider::COLOR`                                 |

## 色（状態別）

| 要素   | 選択中                            | 選択外           | hover / focused / pressed  | disabled      |
| ------ | --------------------------------- | ---------------- | -------------------------- | ------------- |
| ラベル | Primary（Secondary は OnSurface） | OnSurfaceVariant | 状態レイヤー（ラベルの色） | OnSurface 38% |
| 印     | Primary                           | —                | —                          | —             |

## 振る舞い

- 全タブが並び全体の幅を分け合う（固定の並び）。
- Tab では選択中のタブで 1 回だけ止まる。←→ で隣の有効なタブを選ぶ（フォーカスと選択が一緒に動く。端で回る）。Home / End。
- パネルの切り替えは呼び出し側（`on_change`）。

## アクセシビリティ

- 役割: TabList / Tab（selected、位置）

## 既知の制限・仕様からの逸脱

- スクロールするタブの行（ScrollableTabRow）と印のアニメーションは未実装。
- Compose の `Tab` は既定で選択外も選択中と同じ色だが、トークン（`INACTIVE_LABEL_TEXT_COLOR`）の値を使った。

操作とアクセシビリティ情報は `gpui-headless` が持ち、寸法と描画は `md3-components` が担う。
