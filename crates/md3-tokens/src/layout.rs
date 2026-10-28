//! コンポーネントの配置の値のうち、Compose の tokens ではなく部品の実装（`*Defaults` など）にあるもの。
//!
//! 出典は `docs/TOKENS_SOURCE.md`。単位は dp。

/// ボタンの最小幅（`ButtonDefaults.MinWidth`）。
pub const BUTTON_MIN_WIDTH: f32 = 58.0;

/// ボタンのアイコンの大きさ（`ButtonDefaults.IconSize`）。
pub const BUTTON_ICON_SIZE: f32 = 18.0;

/// アイコン付きボタンの左の余白（`ButtonWithIconStartpadding`）。
pub const BUTTON_WITH_ICON_LEADING_SPACE: f32 = 16.0;

/// Text ボタンの左右の余白（`TextButtonHorizontalPadding`）。
pub const TEXT_BUTTON_HORIZONTAL_SPACE: f32 = 12.0;

/// アイコン付き Text ボタンの右の余白（`TextButtonWithIconHorizontalEndPadding`）。
pub const TEXT_BUTTON_WITH_ICON_TRAILING_SPACE: f32 = 16.0;

/// Extended FAB のアイコンの左の余白（`ExtendedFabStartIconPadding`）。
pub const EXTENDED_FAB_START_ICON_SPACE: f32 = 16.0;

/// Extended FAB のアイコンとラベルの間（`ExtendedFabEndIconPadding`）。
pub const EXTENDED_FAB_END_ICON_SPACE: f32 = 12.0;

/// Extended FAB のラベルの左右の余白（`ExtendedFabTextPadding`）。
pub const EXTENDED_FAB_TEXT_SPACE: f32 = 20.0;

/// Extended FAB の最小幅（`ExtendedFabMinimumWidth`）。
pub const EXTENDED_FAB_MIN_WIDTH: f32 = 80.0;

/// チップの左右の余白（`AssistChipDefaults.ContentPadding`）。
pub const CHIP_CONTENT_SPACE: f32 = 8.0;

/// チップのラベルの左右の余白（`HorizontalElementsPadding`）。
pub const CHIP_LABEL_SPACE: f32 = 8.0;

/// SegmentedButton の項目の左右の余白（`SegmentedButtonDefaults.ContentPadding`）。
pub const SEGMENTED_BUTTON_HORIZONTAL_SPACE: f32 = 12.0;

/// SegmentedButton のアイコンとラベルの間（`IconSpacing`）。
pub const SEGMENTED_BUTTON_ICON_SPACE: f32 = 8.0;

/// チェックボックスの線の太さ（`CheckboxDefaults.StrokeWidth`）。
pub const CHECKBOX_STROKE_WIDTH: f32 = 2.0;

/// ラジオボタンの中の点の大きさ（`RadioButtonDotSize`）。
pub const RADIO_BUTTON_DOT_SIZE: f32 = 12.0;

/// ラジオボタンの円の線の太さ（`RadioStrokeWidth`）。
pub const RADIO_BUTTON_STROKE_WIDTH: f32 = 2.0;

/// スライダーのトラックの、つまみ側の角の半径（`TrackInsideCornerSize`）。
pub const SLIDER_TRACK_INSIDE_CORNER: f32 = 2.0;

/// 文字付きバッジの左右の余白（`BadgeWithContentHorizontalPadding`）。
pub const BADGE_HORIZONTAL_SPACE: f32 = 4.0;

/// 線の不定アニメーションの 1 周の時間（ミリ秒、`LinearAnimationDuration`）。
pub const LINEAR_PROGRESS_ANIMATION_DURATION: u64 = 1750;

/// 円の不定アニメーションの 1 周の時間（ミリ秒、`CircularAnimationProgressDuration`）。
pub const CIRCULAR_PROGRESS_ANIMATION_DURATION: u64 = 6000;

/// 円の不定アニメーションの弧の最小の長さ（1 周に対する割合、`CircularIndeterminateMinProgress`）。
pub const CIRCULAR_PROGRESS_MIN_SWEEP: f32 = 0.1;

/// 円の不定アニメーションの弧の最大の長さ（1 周に対する割合、`CircularIndeterminateMaxProgress`）。
pub const CIRCULAR_PROGRESS_MAX_SWEEP: f32 = 0.87;
