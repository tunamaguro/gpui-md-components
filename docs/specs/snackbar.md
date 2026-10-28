# Snackbar 仕様要約

- 出典: Jetpack Compose Material 3 の `Snackbar.kt`（`OneRowSnackbar`）、`SnackbarHost.kt` とトークン
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SnackbarTokens.kt` @ b8256a52
- マイルストーン: M5

## バリアント

- 文字だけ / 操作のボタン付き / 閉じるボタン付き / 長い文（2 行以上）

## 構成要素（アナトミー）

1. コンテナ（InverseSurface、角 ExtraSmall、Level 3）
2. 文字（Body Medium、InverseOnSurface）
3. 操作のボタン（Text Button、InversePrimary）
4. 閉じるボタン（IconButton、InverseOnSurface）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 最大幅 | 600dp | `layout::SNACKBAR_MAX_WIDTH` |
| 左の余白 | 16dp | `layout::SNACKBAR_HORIZONTAL_SPACE` |
| 文字の上下 | 14dp | `layout::SNACKBAR_VERTICAL_SPACE` |
| ボタンの外側 | 8dp | `layout::SNACKBAR_BUTTON_SIDE_SPACE` |
| ウィンドウ端との間隔 | 12dp | `layout::SNACKBAR_OUTER_SPACE` |
| 表示時間 | 短い 4 秒 / 長い 10 秒 / 無期限 | `layout::SNACKBAR_SHORT_DURATION`, `SNACKBAR_LONG_DURATION` |

## 振る舞い

- `SnackbarHost` をウィンドウの根に置き、`show(message)` で出す。一度に 1 つで、新しいものが古いものを置き換える。
- 時間は操作のボタンがあれば無期限、無ければ短い（Compose と同じ既定）。

## アクセシビリティ

- 役割: Status
- 名前: 文字
- キー操作: ボタンは通常の Tab 順に入る

## 既知の制限・仕様からの逸脱

- gpui の a11y API に live region（polite / assertive）がない。役割 Status で出すが、表示時に読み上げられるかはスクリーンリーダー次第（M7 で確認）。
- 表示待ちの列（キュー）、出入りのアニメーション、スワイプで閉じる操作は未実装。
