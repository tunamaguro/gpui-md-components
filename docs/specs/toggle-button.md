# ToggleButton 仕様要約（M3 Expressive）

- 出典: Jetpack Compose Material 3 の実装（`ToggleButton.kt` の `ToggleButton`、`ToggleButtonDefaults`、`Button.kt` の `contentPaddingFor`）とトークン。m3.material.io の本文は参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ButtonSmallTokens.kt`、`FilledButtonTokens.kt`、`ExpressiveMotionTokens.kt`、`StandardMotionTokens.kt` @ b8256a52

## バリアント

- 既定（Filled の配色）のみ。Compose の Elevated / Tonal / Outlined の ToggleButton、大きさ（XSmall〜XLarge）は未実装で、Small（高さ 40dp）だけ。

## 構成要素（アナトミー）

1. コンテナ（形が変わる）
1. アイコン（任意）
1. ラベル

## 寸法

| 項目         | 値                     | トークン                                        |
| ------------ | ---------------------- | ----------------------------------------------- |
| 高さ         | 40dp（密度で減る）     | `button_small::CONTAINER_HEIGHT`                |
| 左右の余白   | 16dp                   | `button_small::LEADING_SPACE`, `TRAILING_SPACE` |
| アイコン     | 20dp、ラベルとの間 8dp | `button_small::ICON_SIZE`, `ICON_LABEL_SPACE`   |
| 最小の幅     | 58dp                   | `layout::BUTTON_MIN_WIDTH`                      |
| 形（非選択） | 丸（高さの半分）       | `button_small::CONTAINER_SHAPE_ROUND`           |
| 形（選択中） | 角丸 12dp              | `button_small::SELECTED_CONTAINER_SHAPE_SQUARE` |
| 形（押下中） | 角丸 8dp               | `button_small::PRESSED_CONTAINER_SHAPE`         |

## 色（状態別）

| 要素               | enabled          | hover             | focused | pressed | disabled      |
| ------------------ | ---------------- | ----------------- | ------- | ------- | ------------- |
| コンテナ（非選択） | SurfaceContainer | 同                | 同      | 同      | OnSurface 10% |
| ラベル（非選択）   | OnSurfaceVariant | 同 + 状態レイヤー | 同      | 同      | OnSurface 38% |
| コンテナ（選択中） | Primary          | 同                | 同      | 同      | OnSurface 10% |
| ラベル（選択中）   | OnPrimary        | 同 + 状態レイヤー | 同      | 同      | OnSurface 38% |

Compose の `ToggleButtonDefaults.colors()` と同じく、ラベルには `FilledButtonTokens.(Un)selectedPressedLabelTextColor` を使う。

## 振る舞い

- 押すと選択が反転する（`on_change` に反転後の値を渡す。状態は利用者が持つ）。
- 形は「押下中 > 選択中 > 非選択」の順で目標が決まり、テーマのモーションスキームの `FastSpatial` のバネで変わる
  （Compose の `shapeByInteraction` と同じ）。Expressive のスキームでは目標を少し行き過ぎて戻る。
- 短いクリックやキーボードでの押下でも形の変化が見えるよう、押下の形への変化が 75% 進むまで押下を保つ
  （Compose の ButtonGroup の `waitUntil { pressedAnimatable.value > 0.75f }`）。
- gpui の `App::reduce_motion` が `true` なら、バネは動かずに最終の形で描き、押下を保つ待ちもしない。

## アクセシビリティ

- 役割: Button（toggled）。Toggle の IconButton と同じ。Compose は Checkbox の役割だが、押すボタンとして読ませる。
- 名前: ラベル
- 状態: toggled（pressed）、disabled
- キー操作: Tab で止まる。Enter / Space で反転。

## デスクトップ向け調整（密度など）

- 高さはテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。形の丸は密度を反映した高さの半分。

## 既知の制限・仕様からの逸脱

- 大きさの段階（XSmall / Medium / Large / XLarge）と、Elevated / Tonal / Outlined の配色は未実装。
- 形の変化は角丸の半径の変化で表す（Compose の `RoundedCornerShape` の角の大きさのアニメーションと同じ）。
  多角形どうしのモーフ（`MaterialShapes`、`LoadingIndicator`）は未実装。
