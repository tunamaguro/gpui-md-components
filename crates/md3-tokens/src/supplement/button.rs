// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt
//         (`ButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ボタンの値のうち、Compose が `ButtonDefaults` に固定値として持つもの
//!
//! 精密なポインタ向けの値 (`shouldUsePrecisionPointerComponentSizing`) は Compose でも既定で使われないため転記しない

use md3_color::ColorRole;

use crate::Dp;
use crate::typography::TypeScale;

/// 最小の幅。`ButtonDefaults.MinWidth`。どのサイズにも使う
pub const MIN_WIDTH: Dp = Dp(58.0);

/// Text ボタンの内容の色。`defaultTextButtonColors` の `contentColor`
///
/// `TextButtonTokens.LabelColor` (OnSurfaceVariant) とは異なる
pub const TEXT_BUTTON_CONTENT_COLOR: ColorRole = ColorRole::Primary;

/// Extra small の左右の余白。`ButtonDefaults.ExtraSmallContentPadding`
///
/// Compose は TODO 付きで、`ButtonXSmallTokens` の余白 (16dp) と異なる値を使っている
pub const EXTRA_SMALL_HORIZONTAL_PADDING: Dp = Dp(12.0);
/// Extra small の上下の余白。`ButtonDefaults.ExtraSmallContentPadding`
pub const EXTRA_SMALL_VERTICAL_PADDING: Dp = Dp(6.0);
/// Extra small のアイコンとラベルの間隔。`ButtonDefaults.ExtraSmallIconSpacing`
///
/// `ButtonXSmallTokens` の間隔 (8dp) と異なる
pub const EXTRA_SMALL_ICON_SPACING: Dp = Dp(4.0);
/// Small の上下の余白。`smallVerticalPadding`
pub const SMALL_VERTICAL_PADDING: Dp = Dp(10.0);
/// Medium の上下の余白。`MediumVerticalPadding`
pub const MEDIUM_VERTICAL_PADDING: Dp = Dp(16.0);
/// Large の上下の余白。`LargeVerticalPadding`
pub const LARGE_VERTICAL_PADDING: Dp = Dp(32.0);
/// Extra large の上下の余白。`ButtonDefaults.ExtraLargeContentPadding`
pub const EXTRA_LARGE_VERTICAL_PADDING: Dp = Dp(48.0);

/// Extra small と Small のラベルの書体。`ButtonDefaults.textStyleFor`
pub const SMALL_LABEL_TEXT: TypeScale = TypeScale::LabelLarge;
/// Medium のラベルの書体。`ButtonDefaults.textStyleFor`
pub const MEDIUM_LABEL_TEXT: TypeScale = TypeScale::TitleMedium;
/// Large のラベルの書体。`ButtonDefaults.textStyleFor`
pub const LARGE_LABEL_TEXT: TypeScale = TypeScale::HeadlineSmall;
/// Extra large のラベルの書体。`ButtonDefaults.textStyleFor`
pub const EXTRA_LARGE_LABEL_TEXT: TypeScale = TypeScale::HeadlineLarge;
