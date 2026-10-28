// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationBarTokens.kt
//         @ b8256a52 (tokens VERSION v0_11_0)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `NavigationBarTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `ContainerHeight`
pub const CONTAINER_HEIGHT: f32 = 64.0;
/// `ItemActiveIconColor`
pub const ITEM_ACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemActiveIndicatorColor`
pub const ITEM_ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ItemActiveIndicatorIconLabelSpace`
pub const ITEM_ACTIVE_INDICATOR_ICON_LABEL_SPACE: f32 = 4.0;
/// `ItemActiveIndicatorShape`
pub const ITEM_ACTIVE_INDICATOR_SHAPE: Shape = Shape::Full;
/// `ItemActiveLabelTextColor`
pub const ITEM_ACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::Secondary;
/// `ItemBetweenSpace`
pub const ITEM_BETWEEN_SPACE: f32 = 0.0;
/// `ItemInactiveIconColor`
pub const ITEM_INACTIVE_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemInactiveLabelTextColor`
pub const ITEM_INACTIVE_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `NavShape`
pub const NAV_SHAPE: Shape = Shape::None;
/// `TallContainerHeight`
pub const TALL_CONTAINER_HEIGHT: f32 = 80.0;
/// `LabelTextFont`
pub const LABEL_TEXT_FONT: TypeScale = TypeScale::LabelMedium;
