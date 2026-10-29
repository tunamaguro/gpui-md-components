# ButtonGroup 仕様要約（M3 Expressive）

- 出典: Jetpack Compose Material 3 の実装（`ButtonGroup.kt` の `ButtonGroup`、`ButtonGroupDefaults`、`animateWidth`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ButtonGroupSmallTokens.kt`、`ConnectedButtonGroupSmallTokens.kt` @ b8256a52

## バリアント

- Standard: 項目の間を 12dp あける。
- Connected（`connected(true)`）: 項目の間を 2dp に詰め、両端の外側の角だけを丸くする。

## 構成要素（アナトミー）

1. [`ToggleButton`](toggle-button.md) の並び

## 寸法

| 項目                  | 値                                        | トークン                                                                                                    |
| --------------------- | ----------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| 高さ                  | 40dp（密度で減る）                        | `button_group_small::CONTAINER_HEIGHT`                                                                      |
| 項目の間（Standard）  | 12dp                                      | `button_group_small::BETWEEN_SPACE`                                                                         |
| 項目の間（Connected） | 2dp                                       | `connected_button_group_small::BETWEEN_SPACE`                                                               |
| 外側の角（Connected） | 丸                                        | `connected_button_group_small::CONTAINER_SHAPE`                                                             |
| 内側の角（Connected） | 8dp、押下中 4dp、選択中は高さの 50%（丸） | `INNER_CORNER_CORNER_SIZE`, `PRESSED_INNER_CORNER_CORNER_SIZE`, `SELECTED_INNER_CORNER_CORNER_SIZE_PERCENT` |
| 押された項目の広がり  | 幅の 15%                                  | Compose の `ButtonGroupDefaults.ExpandedRatio`（トークンではない）                                          |

## 色（状態別）

- 各項目は ToggleButton と同じ（[`toggle-button.md`](toggle-button.md)）。

## 振る舞い

- 項目を押すと、その項目の幅が 15% 広がり、隣の項目が縮む。グループ全体の幅は変わらない
  （押された項目の隣だけが伸び縮みし、縮むのは左右の余白の分まで。Compose の `compressionLimit` の既定と同じ）。
- 広がりは、隣が縮んで吸収できる幅の合計までに抑える（バネの行き過ぎも含む）。ラベルの長い端の項目は 15% より小さく広がり、
  隣の無い 1 項目のグループは広がらない。グループの幅からはみ出さない。
- 幅と形はテーマのモーションスキームの `FastSpatial` のバネで変わる（Compose の ButtonGroup と同じ）。
  押下を離した後も、幅が戻るまで最後に押された項目の隣が伸び縮みする。
- gpui の `App::reduce_motion` が `true` なら、幅と形は動かずに最終の状態で描く。
- 選択を 1 つに限るかは利用者が決める（各項目の `on_change` で他の選択を外す）。

## アクセシビリティ

- 役割: Group（グループのラベルを名前に持つ）+ 各項目は Button（toggled）
- 名前: グループのラベル、各項目のラベル
- 状態: 各項目の toggled、disabled
- キー操作: 各項目が Tab で止まる（無効な項目は飛ばす）。Enter / Space で反転。

## デスクトップ向け調整（密度など）

- 高さはテーマの密度で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 項目は ToggleButton だけ（Compose は任意のボタンと、あふれた項目のメニューを持てる）。あふれた項目のメニューは未実装。
- 押されていないときの幅は、ラベルの文字幅（gpui の `shape_line`）と余白・アイコンから求める。
- 1 つを選ぶ用途でも、読み上げは「ボタン（押されている）」になる。RadioGroup として読ませたいときは
  [SegmentedButton](segmented-button.md) を使う（矢印キーでの移動もそちらが持つ）。
- 縦の並び（Compose の `VerticalButtonGroupSample`）は未実装。
