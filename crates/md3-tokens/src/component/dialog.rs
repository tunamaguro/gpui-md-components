// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DialogTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `DialogTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

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
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerHigh;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL3;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::ExtraLarge;
/// `HeadlineColor`
pub const HEADLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `HeadlineFont`
pub const HEADLINE_FONT: TypeScale = TypeScale::HeadlineSmall;
/// `SupportingTextColor`
pub const SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `SupportingTextFont`
pub const SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyMedium;
/// `IconColor`
pub const ICON_COLOR: ColorRole = ColorRole::Secondary;
/// `IconSize`
pub const ICON_SIZE: f32 = 24.0;
