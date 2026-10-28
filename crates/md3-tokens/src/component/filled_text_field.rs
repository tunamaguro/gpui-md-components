// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FilledTextFieldTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `FilledTextFieldTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActiveIndicatorColor`
pub const ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ActiveIndicatorHeight`
pub const ACTIVE_INDICATOR_HEIGHT: f32 = 1.0;
/// `CaretColor`
pub const CARET_COLOR: ColorRole = ColorRole::Primary;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerHighest;
/// `ContainerShape`（上側の 2 つの角だけに適用する）
pub const CONTAINER_SHAPE: Shape = Shape::ExtraSmall;
/// `DisabledActiveIndicatorColor`
pub const DISABLED_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledActiveIndicatorHeight`
pub const DISABLED_ACTIVE_INDICATOR_HEIGHT: f32 = 1.0;
/// `DisabledActiveIndicatorOpacity`
pub const DISABLED_ACTIVE_INDICATOR_OPACITY: f32 = 0.38;
/// `DisabledContainerColor`
pub const DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledContainerOpacity`
pub const DISABLED_CONTAINER_OPACITY: f32 = 0.04;
/// `DisabledInputColor`
pub const DISABLED_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledInputOpacity`
pub const DISABLED_INPUT_OPACITY: f32 = 0.38;
/// `DisabledLabelColor`
pub const DISABLED_LABEL_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLabelOpacity`
pub const DISABLED_LABEL_OPACITY: f32 = 0.38;
/// `DisabledLeadingIconColor`
pub const DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLeadingIconOpacity`
pub const DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `DisabledSupportingColor`
pub const DISABLED_SUPPORTING_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledSupportingOpacity`
pub const DISABLED_SUPPORTING_OPACITY: f32 = 0.38;
/// `DisabledTrailingIconColor`
pub const DISABLED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledTrailingIconOpacity`
pub const DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
/// `ErrorActiveIndicatorColor`
pub const ERROR_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::Error;
/// `ErrorFocusActiveIndicatorColor`
pub const ERROR_FOCUS_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::Error;
/// `ErrorFocusCaretColor`
pub const ERROR_FOCUS_CARET_COLOR: ColorRole = ColorRole::Error;
/// `ErrorFocusInputColor`
pub const ERROR_FOCUS_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ErrorFocusLabelColor`
pub const ERROR_FOCUS_LABEL_COLOR: ColorRole = ColorRole::Error;
/// `ErrorFocusLeadingIconColor`
pub const ERROR_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ErrorFocusSupportingColor`
pub const ERROR_FOCUS_SUPPORTING_COLOR: ColorRole = ColorRole::Error;
/// `ErrorFocusTrailingIconColor`
pub const ERROR_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::Error;
/// `ErrorHoverActiveIndicatorColor`
pub const ERROR_HOVER_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::OnErrorContainer;
/// `ErrorHoverInputColor`
pub const ERROR_HOVER_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ErrorHoverLabelColor`
pub const ERROR_HOVER_LABEL_COLOR: ColorRole = ColorRole::OnErrorContainer;
/// `ErrorHoverLeadingIconColor`
pub const ERROR_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ErrorHoverSupportingColor`
pub const ERROR_HOVER_SUPPORTING_COLOR: ColorRole = ColorRole::Error;
/// `ErrorHoverTrailingIconColor`
pub const ERROR_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnErrorContainer;
/// `ErrorInputColor`
pub const ERROR_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ErrorLabelColor`
pub const ERROR_LABEL_COLOR: ColorRole = ColorRole::Error;
/// `ErrorLeadingIconColor`
pub const ERROR_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ErrorSupportingColor`
pub const ERROR_SUPPORTING_COLOR: ColorRole = ColorRole::Error;
/// `ErrorTrailingIconColor`
pub const ERROR_TRAILING_ICON_COLOR: ColorRole = ColorRole::Error;
/// `FocusActiveIndicatorColor`
pub const FOCUS_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::Primary;
/// `FocusActiveIndicatorHeight`
pub const FOCUS_ACTIVE_INDICATOR_HEIGHT: f32 = 2.0;
/// `FocusInputColor`
pub const FOCUS_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `FocusLabelColor`
pub const FOCUS_LABEL_COLOR: ColorRole = ColorRole::Primary;
/// `FocusLeadingIconColor`
pub const FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `FocusSupportingColor`
pub const FOCUS_SUPPORTING_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `FocusTrailingIconColor`
pub const FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoverActiveIndicatorColor`
pub const HOVER_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::OnSurface;
/// `HoverActiveIndicatorHeight`
pub const HOVER_ACTIVE_INDICATOR_HEIGHT: f32 = 1.0;
/// `HoverInputColor`
pub const HOVER_INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `HoverLabelColor`
pub const HOVER_LABEL_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoverLeadingIconColor`
pub const HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoverSupportingColor`
pub const HOVER_SUPPORTING_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoverTrailingIconColor`
pub const HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InputColor`
pub const INPUT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InputFont`
pub const INPUT_FONT: TypeScale = TypeScale::BodyLarge;
/// `InputPlaceholderColor`
pub const INPUT_PLACEHOLDER_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InputPrefixColor`
pub const INPUT_PREFIX_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InputSuffixColor`
pub const INPUT_SUFFIX_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LabelColor`
pub const LABEL_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LabelFont`
pub const LABEL_FONT: TypeScale = TypeScale::BodyLarge;
/// `LeadingIconColor`
pub const LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LeadingIconSize`
pub const LEADING_ICON_SIZE: f32 = 24.0;
/// `SupportingColor`
pub const SUPPORTING_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SupportingFont`
pub const SUPPORTING_FONT: TypeScale = TypeScale::BodySmall;
/// `TrailingIconColor`
pub const TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `TrailingIconSize`
pub const TRAILING_ICON_SIZE: f32 = 24.0;
