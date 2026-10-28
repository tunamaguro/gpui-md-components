// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SliderTokens.kt
//         @ b8256a52 (tokens VERSION ?)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `SliderTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActiveContainerOpacity`
pub const ACTIVE_CONTAINER_OPACITY: f32 = 1.0;
/// `ActiveHandleHeight`
pub const ACTIVE_HANDLE_HEIGHT: f32 = 44.0;
/// `ActiveHandleLeadingSpace`
pub const ACTIVE_HANDLE_LEADING_SPACE: f32 = 6.0;
/// `ActiveHandlePadding`
pub const ACTIVE_HANDLE_PADDING: f32 = 6.0;
/// `ActiveHandleShape`
pub const ACTIVE_HANDLE_SHAPE: Shape = Shape::Full;
/// `ActiveHandleTrailingSpace`
pub const ACTIVE_HANDLE_TRAILING_SPACE: f32 = 6.0;
/// `ActiveHandleWidth`
pub const ACTIVE_HANDLE_WIDTH: f32 = 4.0;
/// `ActiveTrackColor`
pub const ACTIVE_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveTrackHeight`
pub const ACTIVE_TRACK_HEIGHT: f32 = 16.0;
/// `ActiveTrackShape`
pub const ACTIVE_TRACK_SHAPE: Shape = Shape::Full;
/// `ActiveTrackShapeLeading`
pub const ACTIVE_TRACK_SHAPE_LEADING: Shape = Shape::Full;
/// `DisabledActiveTrackColor`
pub const DISABLED_ACTIVE_TRACK_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledActiveTrackOpacity`
pub const DISABLED_ACTIVE_TRACK_OPACITY: f32 = 0.38;
/// `DisabledHandleColor`
pub const DISABLED_HANDLE_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledHandleOpacity`
pub const DISABLED_HANDLE_OPACITY: f32 = 0.38;
/// `DisabledHandleWidth`
pub const DISABLED_HANDLE_WIDTH: f32 = 4.0;
/// `DisabledInactiveTrackColor`
pub const DISABLED_INACTIVE_TRACK_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledInactiveTrackOpacity`
pub const DISABLED_INACTIVE_TRACK_OPACITY: f32 = 0.12;
/// `DisabledStopColor`
pub const DISABLED_STOP_COLOR: ColorRole = ColorRole::OnSurface;
/// `FocusActiveTrackColor`
pub const FOCUS_ACTIVE_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `FocusHandleWidth`
pub const FOCUS_HANDLE_WIDTH: f32 = 2.0;
/// `FocusInactiveTrackColor`
pub const FOCUS_INACTIVE_TRACK_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `FocusStopColor`
pub const FOCUS_STOP_COLOR: ColorRole = ColorRole::Primary;
/// `HandleColor`
pub const HANDLE_COLOR: ColorRole = ColorRole::Primary;
/// `HandleHeight`
pub const HANDLE_HEIGHT: f32 = 44.0;
/// `HandleShape`
pub const HANDLE_SHAPE: Shape = Shape::Full;
/// `HandleWidth`
pub const HANDLE_WIDTH: f32 = 4.0;
/// `HoverHandleColor`
pub const HOVER_HANDLE_COLOR: ColorRole = ColorRole::Primary;
/// `HoverHandleWidth`
pub const HOVER_HANDLE_WIDTH: f32 = 4.0;
/// `HoverStopColor`
pub const HOVER_STOP_COLOR: ColorRole = ColorRole::Primary;
/// `InactiveContainerOpacity`
pub const INACTIVE_CONTAINER_OPACITY: f32 = 1.0;
/// `InactiveTrackColor`
pub const INACTIVE_TRACK_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `InactiveTrackHeight`
pub const INACTIVE_TRACK_HEIGHT: f32 = 16.0;
/// `InactiveTrackShape`
pub const INACTIVE_TRACK_SHAPE: Shape = Shape::Full;
/// `LabelContainerColor`
pub const LABEL_CONTAINER_COLOR: ColorRole = ColorRole::Primary;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `PressedActiveTrackColor`
pub const PRESSED_ACTIVE_TRACK_COLOR: ColorRole = ColorRole::Primary;
/// `PressedHandleColor`
pub const PRESSED_HANDLE_COLOR: ColorRole = ColorRole::Primary;
/// `PressedHandleWidth`
pub const PRESSED_HANDLE_WIDTH: f32 = 2.0;
/// `PressedInactiveTrackColor`
pub const PRESSED_INACTIVE_TRACK_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `PressedStopColor`
pub const PRESSED_STOP_COLOR: ColorRole = ColorRole::Primary;
/// `SliderActiveHandleColor`
pub const SLIDER_ACTIVE_HANDLE_COLOR: ColorRole = ColorRole::Primary;
/// `StopIndicatorColor`
pub const STOP_INDICATOR_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `StopIndicatorColorSelected`
pub const STOP_INDICATOR_COLOR_SELECTED: ColorRole = ColorRole::SecondaryContainer;
/// `StopIndicatorShape`
pub const STOP_INDICATOR_SHAPE: Shape = Shape::Full;
/// `StopIndicatorSize`
pub const STOP_INDICATOR_SIZE: f32 = 4.0;
/// `StopIndicatorTrailingSpace`
pub const STOP_INDICATOR_TRAILING_SPACE: f32 = 6.0;
/// `ValueIndicatorActiveBottomSpace`
pub const VALUE_INDICATOR_ACTIVE_BOTTOM_SPACE: f32 = 12.0;
/// `ValueIndicatorContainerColor`
pub const VALUE_INDICATOR_CONTAINER_COLOR: ColorRole = ColorRole::InverseSurface;
/// `ValueIndicatorLabelTextColor`
pub const VALUE_INDICATOR_LABEL_TEXT_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `ValueIndicatorLabelTextFont`
pub const VALUE_INDICATOR_LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
