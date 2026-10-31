# NavigationBar / NavigationRail / NavigationDrawer / NavigationSuite 仕様要約

- 出典: Jetpack Compose Material 3 の `NavigationBar.kt`, `NavigationRail.kt`, `NavigationDrawer.kt` とトークン。幅のクラスは androidx.window の `WindowSizeClass.kt`
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationBarTokens.kt`, `NavigationBarVerticalItemTokens.kt`, `NavigationRail*Tokens.kt`, `NavigationDrawerTokens.kt` @ b8256a52

## バリアント

- NavigationBar（下端、Compact 向け）
- NavigationRail（左端、Medium 向け。先頭に FAB などを置ける）
- NavigationDrawer: Standard（左に並ぶ）/ Modal（スクリムの上）
- NavigationSuite: 幅のクラスで上の 3 つを切り替える（Compact → Bar、Medium → Rail、Expanded 以上 → Drawer）

## 構成要素（アナトミー）

1. コンテナ（Bar: SurfaceContainer / Rail: Surface / Drawer: Standard は Surface、Modal は SurfaceContainerLow、Level 1、右の角 Large）
1. 選択の印（SecondaryContainer。Bar と Rail は 56 × 32dp の丸、Drawer は高さ 56dp の丸い帯）
1. アイコン（24dp。選択中は OnSecondaryContainer、それ以外は OnSurfaceVariant）
1. ラベル（長いラベルは折り返す。Bar と Rail は Label Medium、選択中は Secondary / Drawer は Label Large、選択中は OnSecondaryContainer）

## 寸法

| 項目                  | 値                                                                    | トークン                                                                    |
| --------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Bar の高さ            | 最小 80dp                                                             | `navigation_bar::TALL_CONTAINER_HEIGHT`                                     |
| Bar の項目の間        | 8dp                                                                   | `layout::NAVIGATION_BAR_ITEM_HORIZONTAL_SPACE`                              |
| Bar の印の上          | 12dp                                                                  | `layout::NAVIGATION_BAR_INDICATOR_VERTICAL_OFFSET`                          |
| Rail の幅             | 80dp                                                                  | `navigation_rail_collapsed::NARROW_CONTAINER_WIDTH`                         |
| Rail の上下・項目の間 | 4dp                                                                   | `layout::NAVIGATION_RAIL_VERTICAL_SPACE`                                    |
| Drawer の幅           | 360dp（最小 240dp）                                                   | `navigation_drawer::CONTAINER_WIDTH`, `layout::NAVIGATION_DRAWER_MIN_WIDTH` |
| Drawer の項目         | 最小の高さ 56dp、外側 12dp、内側 左 16 / 右 24dp、アイコンとの間 12dp | `layout::NAVIGATION_DRAWER_ITEM_*`                                          |
| 幅のクラスの境界      | 600 / 840 / 1200 / 1600dp                                             | `layout::WINDOW_WIDTH_*`                                                    |

## 振る舞い

- Drawer のホイールは領域内で動けるときに消費し、端から始まる入力だけ祖先へ渡す。精密スクロールの消費はジェスチャー終了まで保持する。

- Tab では選択中の項目（無ければ最初の有効な項目）で 1 回だけ止まる。

- 矢印キー（Bar は ←→、Rail と Drawer は ↑↓）と Home / End でフォーカスだけを移し、Enter / Space / クリックで選ぶ（手動の選択）。

- Modal の Drawer は Dialog と同じく、開くと有効な選択項目（無効・削除済みなら最初の有効項目、有効項目がなければ内部）にフォーカスし、Tab は中で回り、Esc とスクリムで閉じ、閉じると元へ戻す。

- NavigationSuite は既定でウィンドウの幅（`WindowSizeClass::of`）から形を選ぶ。`.layout()` で固定できる。

## アクセシビリティ

- 役割: Navigation（Modal の Drawer は Dialog）。項目は Tab（selected、位置）

- 名前: 並びは任意の `.aria_label()`、項目は表示ラベルが既定で `.aria_label()` で上書き可能

- Drawer の項目がスクロールするときは、フォーカスが移った項目（キーボードによる矢印・Tab・初期フォーカス）は、スクロール領域の見える範囲に最小の量だけ動かして入れる（WCAG 2.2 の 2.4.11 Focus Not Obscured）

- `ScrollScope` と共通の `reveal_on_focus` で、祖先のスクロール領域も内側から調整する。外側のリング分も含め、両軸をスクロール可能な範囲に制限する。マウスクリックでは動かさず、ホイールで離してもフォーカスが同じなら引き戻さない。

## 既知の制限・仕様からの逸脱

- バッジ、選択中のアイコンの差し替え（塗りのアイコン）は未実装。
- Expanded の幅で Compose の `NavigationSuiteScaffold` は既定で Rail を選ぶが、このライブラリの方針（compact / medium / expanded 以上で切り替え）に合わせて Drawer にした。
- Rail の展開（expanded rail）、Drawer の見出し（セクション）、ドラッグで開閉する操作、開閉のアニメーションは未実装。

## 既定モーション

NavigationBar / NavigationRail の選択背景をアイコンと独立させ、幅を FastSpatial、透明度を DefaultEffects で補間する。アイコンやヒット領域の幅は変えない。reduce_motion は即時変更。参照: NavigationBar.kt / NavigationRail.kt L233–247、indicatorWidth の算出 @ b8256a52。

操作とアクセシビリティ情報は `gpui-headless` が持ち、寸法と描画は `md3-components` が担う。
