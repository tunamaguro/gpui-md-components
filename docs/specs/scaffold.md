# Scaffold 仕様要約

- 出典: Jetpack Compose `Scaffold.kt` @ 2a941dd11aa。m3.material.io は未参照。

## バリアント

- FAB の位置は Start / Center / End。

## 構成要素（アナトミー）

上部バー、本文、Snackbar、FAB、下部バー。

## 寸法

| 項目                   | 値   | トークン                     |
| ---------------------- | ---- | ---------------------------- |
| FAB と端・下部バーの間 | 16dp | `layout::SCAFFOLD_FAB_SPACE` |

## 色（状態別）

レイアウト自身は色を持たない。

## 振る舞い

Snackbar は FAB の直上、FAB がなければ下部バーの直上。本文の余白には FAB を含めない。必要に応じて `List::content_padding` を指定する。

## アクセシビリティ

Scaffold 自身は役割、名前、状態、キー操作を持たない。ツリーの順序は上部バー、本文、Snackbar、FAB、下部バー。Snackbar が FAB を覆わない（WCAG 2.2 2.4.11）。

## デスクトップ向け調整（密度など）

なし。

## 既知の制限・仕様からの逸脱

- Compose の EndOverlay とウィンドウインセットは対象外。Start / End は左 / 右。
- SnackbarHost は Scaffold に渡した後インライン表示になる。
- Scaffold は画面の根に置く前提。インラインの Snackbar は遅延描画なので祖先にクリップされず、クリップされる領域の中に置くと見えない所でも描かれ、操作を受ける。
- GPUI の Deferred は Snackbar を遅れて prepaint するため、Tab 順は本文、FAB、下部バー、Snackbar となり、Compose（本文、Snackbar、FAB、下部バー）と異なる。Dialog より手前に描くために Deferred が要り、`tab_index` は使わない規約なので、この順のままとする。
- gallery と実際の支援技術での確認は未実施。
