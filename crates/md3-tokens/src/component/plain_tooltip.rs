// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/PlainTooltipTokens.kt
//         @ b8256a52 (tokens VERSION ?)
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `PlainTooltipTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::InverseSurface;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::ExtraSmall;
/// `SupportingTextColor`
pub const SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::InverseOnSurface;
/// `SupportingTextFont`
pub const SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodySmall;
