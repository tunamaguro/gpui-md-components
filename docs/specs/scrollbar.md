# Scrollbar 仕様要約

- 出典: AndroidX Compose `Scrollbar.kt` / `NonInteractiveScrollbarDefaults` @ `b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6`（Apache-2.0）
- マイルストーン: M6（共通スクロール表示）、M7（アクセシビリティの確認）
- 独立した MD3 Scrollbar の仕様ページに代えて、指定された Compose 実装を数値と表示条件の根拠にする。

## バリアント

- 縦（既定）/ 横。縦は右端、横は下端。
- `Draggable`（デスクトップ・wasm 共通の既定）/ `Passive`（表示のみ）。
- `fade(false)` で常時表示。内容が収まる場合はどちらも非表示。

## 構成要素と寸法

透明なトラックと丸いつまみ。色は全操作状態で `outline`、不透明度 0.7。

| 項目 | 値 | Compose の名前 |
| --- | --- | --- |
| 太さ | 4dp | Thickness |
| 最小長 | 24dp | ThumbMinLength |
| 最大長 | トラック長の 0.9 | ThumbMaxLengthFraction |
| 主軸余白 | 両端 2dp | MainAxisTrackInset |
| 交差軸余白 | 0dp | CrossAxisTrackInset |
| 待機 | 400ms | ThumbFadeDelayMillis |
| フェード | 250ms | ThumbFadeDurationMillis |

## 振る舞い

- `ScrollbarSource` は表示領域のウィンドウ座標、軸別の最大距離・現在位置・setter を提供する。
  内容の長さは表示領域の長さ + 最大距離。正の位置を GPUI の負の offset に変換する。
- `ScrollHandle` と `UniformListScrollHandle` に実装。UniformList のドラッグは保留中の項目ジャンプを解除する。
- つまみの長さは内容比率、位置は移動可能なトラック長に比例。両端にクランプする。
- トラックが 24dp 未満なら非表示。24dp 以上でも最小長と最大比率が両立しない場合は最大比率を優先する。
- 初回表示と位置変化で表示し、ホバー・ドラッグ中は表示を保つ。静止後に線形フェード。
  GPUI の executor の時計とフレーム要求を使用し、スレッド・ブロッキング・独自の常駐タイマーは使わない。
  `reduce_motion` 時は 400ms 後に即座に隠す。フェード完了後はフレーム要求を止める。
- 左ボタンのドラッグはウィンドウの move / up リスナーで追跡し、つまみの外でも続く。
  トラックのクリックはその方向へ表示領域 1 ページ移動する。
- ホイールの処理を追加しない。`BlockMouseExceptScroll` のヒット領域により、親の既定処理と
  `NestedScroll` の端での伝播・精密ジェスチャーの占有を保つ。
- 各スクロール領域の最後の子として配置する。自身はレイアウト上のサイズを持たず、スクロール位置によらず
  供給元の表示領域に描画する。UniformList ではリストの後ろの兄弟として置く。
- gallery の根、LazyList、`List::max_height`、Dialog 本文、Menu / Select、Sheet 本文、Drawer に適用。
  各部品の `.scrollbar(false)` はバーだけを隠し、スクロールは維持する。

## アクセシビリティ

- 安定した ElementId を必須とし、Canvas の装飾として描く。role / 名前 / 値の重複ノードを作らない。
  GPUI の Canvas は a11y ノードを作らず、フォーカスハンドルも持たない。
- キー操作と読み上げはコンテナと既存の項目に任せる。List / Menu の矢印キー、Dialog / Sheet の
  フォーカス管理を維持する。バー独自の focused / disabled 状態はなく、Passive でポインタ操作を無効にする。
- Web の AccessKit 非対応は既存の GPUI 制限のまま。

## デスクトップ向け調整と制限

- 12dp の操作領域は本実装の判断であり MD3 トークンではない。見た目の太さはホバー中も 4dp。
- Compose は非操作型だが、ここではマウス操作を追加。初回にも短時間表示し、スクロール可能なことを示す。
- RTL の配置反転とタッチ専用のドラッグには未対応。GPUI ウィンドウ外への OS レベルの capture は基盤に依存する。
- OS の読み上げ、実 GPU / Web での見た目の確認結果は `docs/a11y-audit.md` に記録する。
