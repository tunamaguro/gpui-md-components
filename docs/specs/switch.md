# Switch 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Switch.kt`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SwitchTokens.kt` @ b8256a52

## バリアント

- アイコン無し（既定）/ つまみにアイコン（チェック / バツ）

## 構成要素（アナトミー）

1. トラック（52×32dp、非選択は枠線 2dp）
1. つまみ（非選択 16dp、選択・アイコン付き 24dp、押下 28dp）
1. 状態レイヤー（つまみの周り 40dp）

## 寸法

| 項目     | 値             | トークン                              |
| -------- | -------------- | ------------------------------------- |
| トラック | 52×32dp        | `switch::TRACK_WIDTH`, `TRACK_HEIGHT` |
| つまみ   | 16 / 24 / 28dp | `UNSELECTED_HANDLE_WIDTH` ほか        |

## 色（状態別）

| 要素             | enabled   | hover            | focused          | pressed          | disabled      |
| ---------------- | --------- | ---------------- | ---------------- | ---------------- | ------------- |
| トラック（選択） | Primary   | 同               | 同               | 同               | OnSurface 12% |
| つまみ（選択）   | OnPrimary | PrimaryContainer | PrimaryContainer | PrimaryContainer | Surface       |
| つまみ（非選択） | Outline   | OnSurfaceVariant | OnSurfaceVariant | OnSurfaceVariant | OnSurface 38% |

## 振る舞い

- 押すと反転した値を `on_change` で渡す。つまみの移動は後述の既定モーションに従う。
- Space / Enter の押下中にフォーカスを失ったら押下を取り消す。移動先でキーを離し、元の要素にキー解放が届かない場合も、つまみは通常の寸法に戻る。フォーカス喪失の購読は ID に結び付いた状態で保持し、要素が消えたら解除する。

## アクセシビリティ

- 役割: Switch
- 名前: `.aria_label()`（任意）
- 状態: toggled
- キー操作: Space（gpui の仕様で Enter でも切り替わる）

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。

## 既定モーション

選択変更のつまみ位置・寸法は Theme::spring(FastSpatial)。押下中は Compose ThumbNode と同じく位置・寸法を snap し、離した後にバネで戻す（マウス、Space / Enter）。トラック・枠線・つまみ・アイコンの色は FastEffects。ID を保って途中の反転にも追従する。reduce_motion は GPUI が最終値に即時変更する。参照: Switch.kt L162–201、L280–326 @ b8256a52。色補間は GPUI の Hsla 成分で行い、Compose の色空間補間とは差がある。

補足: 固定 Compose の Switch.kt の色は即時変更であり、FastEffects の色補間は Compose からの追加（色の変化も滑らかにするための判断）。バネの数値は既存 MotionScheme トークンを再利用した。

状態と操作は `gpui_headless::Switch` が担う。
