# Card 仕様要約

- 出典: Jetpack Compose Material 3 の `Card.kt`（`Card`, `ElevatedCard`, `OutlinedCard`）とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ElevatedCardTokens.kt`, `FilledCardTokens.kt`, `OutlinedCardTokens.kt` @ b8256a52

## バリアント

- Elevated（SurfaceContainerLow、Level 1、hover で Level 2）
- Filled（SurfaceContainerHighest、hover で Level 1）
- Outlined（Surface、1dp OutlineVariant、hover で Level 1）

## 寸法

| 項目             | 値             | トークン                       |
| ---------------- | -------------- | ------------------------------ |
| 角               | Medium（12dp） | `*_card::CONTAINER_SHAPE`      |
| 枠線（Outlined） | 1dp            | `outlined_card::OUTLINE_WIDTH` |

## 色（状態別）

| 要素         | enabled  | hover              | focused       | pressed       | disabled                                                       |
| ------------ | -------- | ------------------ | ------------- | ------------- | -------------------------------------------------------------- |
| 状態レイヤー | —        | OnSurface 8%       | OnSurface 10% | OnSurface 10% | —                                                              |
| コンテナ     | 種類の色 | 同左（影が上がる） | 同左          | 同左          | 無効の色を下地に重ねた不透明な色（Compose の `compositeOver`） |

## 振る舞い

- 中身と余白は呼び出し側が子要素で決める。
- `on_click` を付けると押せるカード（Tab で止まり、Enter / Space で押す）。

## アクセシビリティ

- キーボードでフォーカスが移ると、祖先の `ScrollScope` の表示領域へ移す（WCAG 2.2 の 2.4.11）。
  `reveal_on_focus` を共有し、マウスクリックやフォーカスが変わらないホイール操作では引き戻さない。

- 押せるカード: 役割 Button、名前は `on_click` に渡すラベル

- 押せないカード: 装飾なのでツリーに出ない（中身の要素がそれぞれ出る）

## 既知の制限・仕様からの逸脱

- ドラッグの状態（Dragged）は未実装。
