// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt
//         (`TooltipDefaults` とファイル末尾の internal と private な定数)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ツールチップの値のうち、トークンに無いもの
//!
//! Compose の実装が持つ固定値から転記している

use crate::Dp;

/// トリガーとツールチップの間隔。`SpacingBetweenTooltipAndAnchor`
pub const SPACING_BETWEEN_TOOLTIP_AND_ANCHOR: Dp = Dp(4.0);
/// 容器の最小の高さ。`TooltipMinHeight`
pub const MIN_HEIGHT: Dp = Dp(24.0);
/// 容器の最小の幅。`TooltipMinWidth`
pub const MIN_WIDTH: Dp = Dp(40.0);
/// Plain の容器の上下の余白。`PlainTooltipVerticalPadding`。
/// Rich で見出しを持たないときは、本文の上下の余白にも使う
pub const PLAIN_VERTICAL_PADDING: Dp = Dp(4.0);
/// Plain の容器の左右の余白。`PlainTooltipHorizontalPadding`
pub const PLAIN_HORIZONTAL_PADDING: Dp = Dp(8.0);
/// Plain の容器の最大の幅。`TooltipDefaults.plainTooltipMaxWidth`
pub const PLAIN_MAX_WIDTH: Dp = Dp(200.0);
/// Rich の容器の最大の幅。`TooltipDefaults.richTooltipMaxWidth`
pub const RICH_MAX_WIDTH: Dp = Dp(320.0);
/// Rich の容器の左右の余白。`RichTooltipHorizontalPadding`
pub const RICH_HORIZONTAL_PADDING: Dp = Dp(16.0);
/// Rich の容器の上端から見出しの最初の基準線までの距離。`HeightToSubheadFirstLine`
pub const HEIGHT_TO_SUBHEAD_FIRST_LINE: Dp = Dp(28.0);
/// Rich で見出しを持つときの、見出しの下端から本文の最初の基準線までの距離。`HeightFromSubheadToTextFirstLine`
pub const HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE: Dp = Dp(24.0);
/// Rich で見出しを持つときの、本文の下の余白。`TextBottomPadding`
pub const TEXT_BOTTOM_PADDING: Dp = Dp(16.0);
