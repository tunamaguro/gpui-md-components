# Snackbar 仕様要約

- 出典: Jetpack Compose Material 3 の `Snackbar.kt`（`OneRowSnackbar`）、`SnackbarHost.kt` とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SnackbarTokens.kt` @ b8256a52

## バリアント

- 文字だけ / 操作のボタン付き / 閉じるボタン付き / 長い文（2 行以上）

## 構成要素（アナトミー）

1. コンテナ（InverseSurface、角 ExtraSmall、Level 3）
1. 文字（Body Medium、InverseOnSurface）
1. 操作のボタン（Text Button、InversePrimary）
1. 閉じるボタン（IconButton、InverseOnSurface）

## 寸法

| 項目                 | 値                              | トークン                                                    |
| -------------------- | ------------------------------- | ----------------------------------------------------------- |
| 最大幅               | 600dp                           | `layout::SNACKBAR_MAX_WIDTH`                                |
| 左の余白             | 16dp                            | `layout::SNACKBAR_HORIZONTAL_SPACE`                         |
| 文字の上下           | 14dp                            | `layout::SNACKBAR_VERTICAL_SPACE`                           |
| ボタンの外側         | 8dp                             | `layout::SNACKBAR_BUTTON_SIDE_SPACE`                        |
| ウィンドウ端との間隔 | 12dp                            | `layout::SNACKBAR_OUTER_SPACE`                              |
| 表示時間             | 短い 4 秒 / 長い 10 秒 / 無期限 | `layout::SNACKBAR_SHORT_DURATION`, `SNACKBAR_LONG_DURATION` |

## 振る舞い

- `SnackbarHost` をウィンドウの根に置き、`show(message)` で出す。一度に 1 つで、新しいものが古いものを置き換える。
- 時間は操作のボタンがあれば無期限、無ければ短い（Compose と同じ既定）。

## アクセシビリティ

- 役割: Status
- 名前: 表示文字が既定。`.aria_label()` で上書き可能
- キー操作: ボタンは通常の Tab 順に入る
- 下の要素へのクリックとホバーを遮り、ホイールは下のスクロール領域に通す。

## 既知の制限・仕様からの逸脱

- AccessKit に Status・polite を渡す。実際の読み上げは未確認。Web の AccessKit 非対応は従来どおり。
- スワイプで閉じる操作は未実装。

## 既定モーションと表示の方針

- `SnackbarHost::new(...).policy(SnackbarPolicy::Replace)` が既定。新しい知らせは現在の知らせを退場させ、待機分も破棄して最新の 1 件を残す。
- `SnackbarPolicy::Queue` は Compose の順番待ち。操作付きの既定は Indefinite なので、操作か閉じるまで次の知らせは待つ。`set_policy` は以後の追加に適用する。
- M3 の同時表示は 1 件。積み重ね表示にはしない。既定を Replace にしたのは、利用者が続けて発行した知らせを即座に反映し、待機を故障と誤解しにくくするための製品側の選択（Compose の既定からの差）。
- 入場は FastEffects のフェードと FastSpatial の 0.8→1、退場は FastEffects のフェード。退場完了まで旧メッセージを保持し、その後に次を描く。退場中の操作は無効、live 通知も停止する。新しい ID の Status・polite で各表示を通知する。
- `SnackbarMessage::on_result` は ActionPerformed / Dismissed を一度だけ返す。時間切れ・閉じる・置換は Dismissed（待機中に置換された知らせも含む）。現在の知らせ・待機分ともホストの更新完了後に `App::defer` で通知し、結果ハンドラーから同じホストに次の知らせを追加できる。操作ハンドラーは操作ボタンでのみ実行し、結果通知より先に完了する。
- `reduce_motion` では即時切替。退場中に設定が有効になった場合も次の描画で即時完了する。退場完了はバネの描画値から判定し、専用の待機タイマーは使わない。4 秒 / 10 秒の表示時間はアクセシビリティ設定にかかわらず維持する。
- 固定 GPUI にコンテナ全体の graphicsLayer 相当の scale API がないため、0.8→1 は中央揃えの幅で近似する。高さ・文字・アイコンの等倍拡大ではなく、狭い幅では文が再改行される。Compose と完全に同じ拡大表示は未対応。
- 参照: `SnackbarHost.kt` L320–418（androidx@b8256a52）。M3 ガイドライン URL は取得時に JavaScript 必須で本文を取得できず、同梱の Compose ソースで振る舞いを確認した。
