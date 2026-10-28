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

/// 操作できる要素の最小の大きさ（`LocalMinimumInteractiveComponentSize`）。
pub const MIN_INTERACTIVE_SIZE: f32 = 48.0;

/// テキストフィールドの最小幅（`TextFieldDefaults.MinWidth`）。
pub const TEXT_FIELD_MIN_WIDTH: f32 = 280.0;

/// フォーカス中の Filled テキストフィールドの下線の太さ（`FocusedIndicatorThickness`）。
pub const TEXT_FIELD_FOCUSED_INDICATOR_THICKNESS: f32 = 2.0;

/// テキストフィールドの左右の余白（`TextFieldPadding`）。
pub const TEXT_FIELD_HORIZONTAL_SPACE: f32 = 16.0;

/// 補助テキストの上の余白（`SupportingTopPadding`）。
pub const TEXT_FIELD_SUPPORTING_TOP_SPACE: f32 = 4.0;

/// 接頭辞・接尾辞と入力文字の間（`PrefixSuffixTextPadding`）。
pub const TEXT_FIELD_PREFIX_SUFFIX_SPACE: f32 = 2.0;

/// ラベル付き Filled テキストフィールドの上下の余白（`TextFieldWithLabelVerticalPadding`）。
pub const TEXT_FIELD_WITH_LABEL_VERTICAL_SPACE: f32 = 8.0;

/// Outlined テキストフィールドの枠線をラベルの左右で切り欠く幅（`OutlinedTextFieldInnerPadding`）。
pub const OUTLINED_TEXT_FIELD_LABEL_GAP: f32 = 4.0;

/// メニューとウィンドウの上下の端の最小の間隔（`MenuVerticalMargin`）。
pub const MENU_VERTICAL_MARGIN: f32 = 48.0;

/// メニューとウィンドウの左右の端の最小の間隔（`MenuHorizontalMargin`）。
pub const MENU_HORIZONTAL_MARGIN: f32 = 8.0;

/// メニュー項目の最小の高さ（`MenuListItemContainerHeight`）。
pub const MENU_ITEM_HEIGHT: f32 = 48.0;

/// メニュー項目の左右の余白とアイコンとの間（`DropdownMenuItemHorizontalPadding`）。
pub const MENU_ITEM_HORIZONTAL_SPACE: f32 = 12.0;

/// メニューの上下の余白（`DropdownMenuVerticalPadding`）。
pub const MENU_VERTICAL_SPACE: f32 = 8.0;

/// メニューの最小幅（`DropdownMenuItemDefaultMinWidth`）。
pub const MENU_MIN_WIDTH: f32 = 112.0;

/// メニューの最大幅（`DropdownMenuItemDefaultMaxWidth`）。
pub const MENU_MAX_WIDTH: f32 = 280.0;

/// ダイアログの内側の余白（`dialogPaddingValue`）。
pub const DIALOG_SPACE: f32 = 24.0;

/// ダイアログの本文の下の余白（`textPaddingValue`）。
pub const DIALOG_TEXT_BOTTOM_SPACE: f32 = 24.0;

/// ダイアログの最小幅（`DialogMinWidth`）。
pub const DIALOG_MIN_WIDTH: f32 = 280.0;

/// ダイアログの最大幅（`DialogMaxWidth`）。
pub const DIALOG_MAX_WIDTH: f32 = 560.0;

/// ダイアログのボタンの間（`ButtonsMainAxisSpacing`）。
pub const DIALOG_BUTTON_SPACE: f32 = 8.0;

/// ダイアログのアイコンの下の余白（`IconPadding`）。
pub const DIALOG_ICON_BOTTOM_SPACE: f32 = 16.0;

/// ダイアログの見出しの下の余白（`TitlePadding`）。
pub const DIALOG_TITLE_BOTTOM_SPACE: f32 = 16.0;

/// スナックバーの最大幅（`ContainerMaxWidth`）。
pub const SNACKBAR_MAX_WIDTH: f32 = 600.0;

/// スナックバーの左の余白（`HorizontalSpacing`）。
pub const SNACKBAR_HORIZONTAL_SPACE: f32 = 16.0;

/// スナックバーのボタン側（右）の余白（`HorizontalSpacingButtonSide`）。
pub const SNACKBAR_BUTTON_SIDE_SPACE: f32 = 8.0;

/// スナックバーの文字とボタンの間（`TextEndExtraSpacing`）。
pub const SNACKBAR_TEXT_END_SPACE: f32 = 8.0;

/// スナックバーの文字の上下の余白（`SnackbarVerticalPadding`）。
pub const SNACKBAR_VERTICAL_SPACE: f32 = 14.0;

/// 短いスナックバーの表示時間（ミリ秒、`SnackbarDuration.Short`）。
pub const SNACKBAR_SHORT_DURATION: u64 = 4000;

/// 長いスナックバーの表示時間（ミリ秒、`SnackbarDuration.Long`）。
pub const SNACKBAR_LONG_DURATION: u64 = 10000;

/// ツールチップと基準の要素の間（`SpacingBetweenTooltipAndAnchor`）。
pub const TOOLTIP_ANCHOR_SPACE: f32 = 4.0;

/// ツールチップの最小の高さ（`TooltipMinHeight`）。
pub const TOOLTIP_MIN_HEIGHT: f32 = 24.0;

/// ツールチップの最小幅（`TooltipMinWidth`）。
pub const TOOLTIP_MIN_WIDTH: f32 = 40.0;

/// Plain ツールチップの上下の余白（`PlainTooltipVerticalPadding`）。
pub const PLAIN_TOOLTIP_VERTICAL_SPACE: f32 = 4.0;

/// Plain ツールチップの左右の余白（`PlainTooltipHorizontalPadding`）。
pub const PLAIN_TOOLTIP_HORIZONTAL_SPACE: f32 = 8.0;

/// Plain ツールチップの最大幅（`plainTooltipMaxWidth`）。
pub const PLAIN_TOOLTIP_MAX_WIDTH: f32 = 200.0;

/// Rich ツールチップの最大幅（`richTooltipMaxWidth`）。
pub const RICH_TOOLTIP_MAX_WIDTH: f32 = 320.0;

/// Rich ツールチップの左右の余白（`RichTooltipHorizontalPadding`）。
pub const RICH_TOOLTIP_HORIZONTAL_SPACE: f32 = 16.0;

/// Rich ツールチップの上端から見出しの 1 行目のベースラインまで（`HeightToSubheadFirstLine`）。
pub const RICH_TOOLTIP_SUBHEAD_FIRST_LINE: f32 = 28.0;

/// Rich ツールチップの見出しから本文の 1 行目のベースラインまで（`HeightFromSubheadToTextFirstLine`）。
pub const RICH_TOOLTIP_TEXT_FIRST_LINE: f32 = 24.0;

/// Rich ツールチップの本文の下の余白（`TextBottomPadding`）。
pub const RICH_TOOLTIP_TEXT_BOTTOM_SPACE: f32 = 16.0;

/// Rich ツールチップの操作ボタンの行の最小の高さ（`ActionLabelMinHeight`）。
pub const RICH_TOOLTIP_ACTION_MIN_HEIGHT: f32 = 36.0;

/// Rich ツールチップの操作ボタンの下の余白（`ActionLabelBottomPadding`）。
pub const RICH_TOOLTIP_ACTION_BOTTOM_SPACE: f32 = 8.0;

/// ボトムシートの最大幅（`BottomSheetDefaults.SheetMaxWidth`）。
pub const BOTTOM_SHEET_MAX_WIDTH: f32 = 640.0;

/// ボトムシートのつまみの上下の余白（`DragHandleVerticalPadding`）。
pub const BOTTOM_SHEET_DRAG_HANDLE_VERTICAL_SPACE: f32 = 22.0;

/// ボトムシートの開閉の時間（ミリ秒、`BottomSheetAnimationSpec`）。
pub const BOTTOM_SHEET_ANIMATION_DURATION: u64 = 300;
