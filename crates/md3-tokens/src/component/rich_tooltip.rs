// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/RichTooltipTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `RichTooltipTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ActionFocusLabelTextColor`
pub const ACTION_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActionHoverLabelTextColor`
pub const ACTION_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActionLabelTextColor`
pub const ACTION_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ActionLabelTextFont`
pub const ACTION_LABEL_TEXT_FONT: TypeScale = TypeScale::LabelLarge;
/// `ActionPressedLabelTextColor`
pub const ACTION_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Medium;
/// `SubheadColor`
pub const SUBHEAD_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SubheadFont`
pub const SUBHEAD_FONT: TypeScale = TypeScale::TitleSmall;
/// `SupportingTextColor`
pub const SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SupportingTextFont`
pub const SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyMedium;
