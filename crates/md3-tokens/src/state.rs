//! 状態レイヤーの不透明度と、無効状態の不透明度。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

/// ホバー時の状態レイヤーの不透明度。
pub const HOVER_STATE_LAYER_OPACITY: f32 = 0.08;

/// フォーカス時の状態レイヤーの不透明度。
pub const FOCUS_STATE_LAYER_OPACITY: f32 = 0.1;

/// 押下時の状態レイヤーの不透明度。
pub const PRESSED_STATE_LAYER_OPACITY: f32 = 0.1;

/// ドラッグ時の状態レイヤーの不透明度。
pub const DRAGGED_STATE_LAYER_OPACITY: f32 = 0.16;

/// 無効状態の内容（文字・アイコン）の不透明度。
///
/// コンテナの不透明度はコンポーネントごとに異なるため、各コンポーネントのトークンに置く。
pub const DISABLED_CONTENT_OPACITY: f32 = 0.38;

/// フォーカスインジケーター（フォーカスリング）の太さ（dp）。
pub const FOCUS_INDICATOR_THICKNESS: f32 = 3.0;

/// フォーカスインジケーターと要素の外側の間隔（dp）。
pub const FOCUS_INDICATOR_OUTER_OFFSET: f32 = 2.0;

/// フォーカスインジケーターの色のロール。
pub const FOCUS_INDICATOR_COLOR: md3_color::ColorRole = md3_color::ColorRole::Secondary;
