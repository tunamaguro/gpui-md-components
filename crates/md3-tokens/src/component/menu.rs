// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/MenuTokens.kt
//         @ b8256a52 (tokens VERSION v0_210)
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `MenuTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use md3_color::ColorRole;

/// `ContainerColor`
pub const CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainer;
/// `ContainerElevation`
pub const CONTAINER_ELEVATION: f32 = elevation::LEVEL2;
/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::ExtraSmall;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `ListItemSelectedContainerColor`
pub const LIST_ITEM_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ListItemSelectedLabelTextColor`
pub const LIST_ITEM_SELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ListItemSelectedLeadingTrailingIconColor`
pub const LIST_ITEM_SELECTED_LEADING_TRAILING_ICON_COLOR: ColorRole =
    ColorRole::OnSecondaryContainer;
/// `MenuListItemLeadingIconColor`
pub const MENU_LIST_ITEM_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
