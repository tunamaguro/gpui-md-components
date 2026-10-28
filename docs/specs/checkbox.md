# Checkbox 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`Checkbox.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/CheckboxTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- 未チェック / チェック / 不定（Indeterminate）
- エラー

## 構成要素（アナトミー）

1. 状態レイヤー（40dp の円）
2. 18dp の箱（角丸 2dp）
3. チェックまたは横線

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 状態レイヤー | 40dp（密度で減る） | `checkbox::STATE_LAYER_SIZE` |
| 箱 | 18dp、角丸 2dp | `CONTAINER_SIZE`, `CONTAINER_SHAPE` |
| 枠線 | 2dp | `UNSELECTED_OUTLINE_WIDTH` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| 箱（選択） | Primary | 同 | 同 | 同 | OnSurface 38% |
| 枠線（非選択） | OnSurfaceVariant | OnSurface | OnSurface | OnSurface | OnSurface 38% |
| エラー | Error / OnError | 同 | 同 | 同 | — |

## 振る舞い

- 押すと次の状態（不定 → チェック → 未チェック → チェック）を `on_change` で渡す。

## アクセシビリティ

- 役割: CheckBox
- 名前: ラベル引数（必須。見える文字は呼び出し側で隣に置く）
- 状態: toggled（true / false / mixed）
- キー操作: Space（gpui の仕様で Enter でも切り替わる）

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- チェックは Material Symbols の `check` アイコン、不定の横線は箱の幅の 0.6 倍の簡略形で、Compose のパス描画・アニメーションとは違う。
- gpui の a11y API に無効状態がないため、無効はツリーに出ない（Tab で止まらないことで表す）。
