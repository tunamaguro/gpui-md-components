// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SuggestionChipTokens.kt
//         @ b8256a52 (tokens VERSION 7_0_1)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `SuggestionChipTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 32.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Small;
/// `DisabledLabelTextColor`
pub const DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLabelTextOpacity`
pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `DraggedContainerElevation`
pub const DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `DraggedLabelTextColor`
pub const DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ElevatedContainerColor`
pub const ELEVATED_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `ElevatedContainerElevation`
pub const ELEVATED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ElevatedDisabledContainerColor`
pub const ELEVATED_DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `ElevatedDisabledContainerElevation`
pub const ELEVATED_DISABLED_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ElevatedDisabledContainerOpacity`
pub const ELEVATED_DISABLED_CONTAINER_OPACITY: f32 = 0.12;
/// `ElevatedFocusContainerElevation`
pub const ELEVATED_FOCUS_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `ElevatedHoverContainerElevation`
pub const ELEVATED_HOVER_CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `ElevatedPressedContainerElevation`
pub const ELEVATED_PRESSED_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `FlatContainerElevation`
pub const FLAT_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `FlatDisabledOutlineColor`
pub const FLAT_DISABLED_OUTLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `FlatDisabledOutlineOpacity`
pub const FLAT_DISABLED_OUTLINE_OPACITY: f32 = 0.12;
/// `FlatFocusOutlineColor`
pub const FLAT_FOCUS_OUTLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `FlatOutlineColor`
pub const FLAT_OUTLINE_COLOR: ColorRole = ColorRole::OutlineVariant;
/// `FlatOutlineWidth`
pub const FLAT_OUTLINE_WIDTH: f32 = 1.0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `FocusLabelTextColor`
pub const FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `HoverLabelTextColor`
pub const HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LabelTextColor`
pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `PressedLabelTextColor`
pub const PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `DisabledLeadingIconColor`
pub const DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `DisabledLeadingIconOpacity`
pub const DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `DraggedLeadingIconColor`
pub const DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `FocusLeadingIconColor`
pub const FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `HoverLeadingIconColor`
pub const HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `LeadingIconColor`
pub const LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `LeadingIconSize`
pub const LEADING_ICON_SIZE: f32 = 18.0;
/// `PressedLeadingIconColor`
pub const PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::Primary;
