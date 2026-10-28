# Switch 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Switch.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SwitchTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- アイコン無し（既定）/ つまみにアイコン（チェック / バツ）

## 構成要素（アナトミー）

1. トラック（52×32dp、非選択は枠線 2dp）
2. つまみ（非選択 16dp、選択・アイコン付き 24dp、押下 28dp）
3. 状態レイヤー（つまみの周り 40dp）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| トラック | 52×32dp | `switch::TRACK_WIDTH`, `TRACK_HEIGHT` |
| つまみ | 16 / 24 / 28dp | `UNSELECTED_HANDLE_WIDTH` ほか |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| トラック（選択） | Primary | 同 | 同 | 同 | OnSurface 12% |
| つまみ（選択） | OnPrimary | PrimaryContainer | PrimaryContainer | PrimaryContainer | Surface |
| つまみ（非選択） | Outline | OnSurfaceVariant | OnSurfaceVariant | OnSurfaceVariant | OnSurface 38% |

## 振る舞い

- 押すと反転した値を `on_change` で渡す。つまみの移動はアニメーションしない。

## アクセシビリティ

- 役割: Switch
- 名前: ラベル引数（必須）
- 状態: toggled
- キー操作: Space（gpui の仕様で Enter でも切り替わる）

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- つまみの移動・大きさの変化のアニメーションは未実装。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
