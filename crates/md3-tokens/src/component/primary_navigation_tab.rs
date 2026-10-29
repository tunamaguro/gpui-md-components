// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PrimaryNavigationTabTokens.kt
//         @ b8256a52 (tokens VERSION v0_162)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `PrimaryNavigationTabTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::{CornerSize, Shape};
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActiveIndicatorColor`
pub const ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveIndicatorHeight`
pub const ACTIVE_INDICATOR_HEIGHT: f32 = 3.0;
/// `ActiveIndicatorShape`
pub const ACTIVE_INDICATOR_SHAPE: CornerSize = CornerSize::Fixed(3.0);
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 48.0;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::None;
/// `ActiveFocusIconColor`
pub const ACTIVE_FOCUS_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveHoverIconColor`
pub const ACTIVE_HOVER_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveIconColor`
pub const ACTIVE_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `ActivePressedIconColor`
pub const ACTIVE_PRESSED_ICON_COLOR: ColorRole = ColorRole::Primary;
/// `IconAndLabelTextContainerHeight`
pub const ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT: f32 = 64.0;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
/// `InactiveFocusIconColor`
pub const INACTIVE_FOCUS_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveHoverIconColor`
pub const INACTIVE_HOVER_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveIconColor`
pub const INACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InactivePressedIconColor`
pub const INACTIVE_PRESSED_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ActiveFocusLabelTextColor`
pub const ACTIVE_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveHoverLabelTextColor`
pub const ACTIVE_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveLabelTextColor`
pub const ACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActivePressedLabelTextColor`
pub const ACTIVE_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `InactiveFocusLabelTextColor`
pub const INACTIVE_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveHoverLabelTextColor`
pub const INACTIVE_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `InactiveLabelTextColor`
pub const INACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `InactivePressedLabelTextColor`
pub const INACTIVE_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::TitleSmall;
