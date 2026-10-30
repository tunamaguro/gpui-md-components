// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt
//         @ b8256a52 (tokens VERSION 29.0.0)
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! `ListTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `ContainerShape`
pub const CONTAINER_SHAPE: Shape = Shape::Large;
/// `DividerBottomSpace`
pub const DIVIDER_BOTTOM_SPACE: f32 = 0.0;
/// `DividerLeadingSpace`
pub const DIVIDER_LEADING_SPACE: f32 = 16.0;
/// `DividerTopSpace`
pub const DIVIDER_TOP_SPACE: f32 = 0.0;
/// `DividerTrailingSpace`
pub const DIVIDER_TRAILING_SPACE: f32 = 16.0;
/// `FocusIndicatorColor`
pub const FOCUS_INDICATOR_COLOR: ColorRole = ColorRole::Secondary;
/// `ItemBetweenSpace`
pub const ITEM_BETWEEN_SPACE: f32 = 12.0;
/// `ItemBottomSpace`
pub const ITEM_BOTTOM_SPACE: f32 = 10.0;
/// `ItemContainerColor`
pub const ITEM_CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ItemContainerElevation`
pub const ITEM_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
/// `ItemContainerExpressiveShape`
pub const ITEM_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::ExtraSmall;
/// `ItemContainerShape`
pub const ITEM_CONTAINER_SHAPE: Shape = Shape::None;
/// `ItemDisabledContainerExpressiveShape`
pub const ITEM_DISABLED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::ExtraSmall;
/// `ItemDisabledLabelTextColor`
pub const ITEM_DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDisabledLabelTextOpacity`
pub const ITEM_DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `ItemDisabledLeadingIconColor`
pub const ITEM_DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDisabledLeadingIconOpacity`
pub const ITEM_DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `ItemDisabledOverlineColor`
pub const ITEM_DISABLED_OVERLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDisabledOverlineOpacity`
pub const ITEM_DISABLED_OVERLINE_OPACITY: f32 = 0.38;
/// `ItemDisabledStateLayerOpacity`
pub const ITEM_DISABLED_STATE_LAYER_OPACITY: f32 = 0.1;
/// `ItemDisabledSupportingTextColor`
pub const ITEM_DISABLED_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDisabledSupportingTextOpacity`
pub const ITEM_DISABLED_SUPPORTING_TEXT_OPACITY: f32 = 0.38;
/// `ItemDisabledTrailingIconColor`
pub const ITEM_DISABLED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDisabledTrailingIconOpacity`
pub const ITEM_DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
/// `ItemDraggedContainerElevation`
pub const ITEM_DRAGGED_CONTAINER_ELEVATION: f32 = elevation::LEVEL4;
/// `ItemDraggedContainerExpressiveShape`
pub const ITEM_DRAGGED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemDraggedLabelTextColor`
pub const ITEM_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemDraggedLeadingIconIconColor`
pub const ITEM_DRAGGED_LEADING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemDraggedTrailingIconIconColor`
pub const ITEM_DRAGGED_TRAILING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemFocusLabelTextColor`
pub const ITEM_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemFocusLeadingIconIconColor`
pub const ITEM_FOCUS_LEADING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemFocusTrailingIconIconColor`
pub const ITEM_FOCUS_TRAILING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemFocusedContainerExpressiveShape`
pub const ITEM_FOCUSED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemHoverLabelTextColor`
pub const ITEM_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemHoverLeadingIconIconColor`
pub const ITEM_HOVER_LEADING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemHoverTrailingIconIconColor`
pub const ITEM_HOVER_TRAILING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemHoveredContainerExpressiveShape`
pub const ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Medium;
/// `ItemLabelTextColor`
pub const ITEM_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemLabelTextFont`
pub const ITEM_LABEL_TEXT_FONT: TypeScale = TypeScale::BodyLarge;
/// `ItemLargeLeadingVideoHeight`
pub const ITEM_LARGE_LEADING_VIDEO_HEIGHT: f32 = 64.0;
/// `ItemLargeLeadingVideoWidth`
pub const ITEM_LARGE_LEADING_VIDEO_WIDTH: f32 = 114.0;
/// `ItemLeadingAvatarColor`
pub const ITEM_LEADING_AVATAR_COLOR: ColorRole = ColorRole::PrimaryContainer;
/// `ItemLeadingAvatarLabelColor`
pub const ITEM_LEADING_AVATAR_LABEL_COLOR: ColorRole = ColorRole::OnPrimaryContainer;
/// `ItemLeadingAvatarLabelFont`
pub const ITEM_LEADING_AVATAR_LABEL_FONT: TypeScale = TypeScale::TitleMedium;
/// `ItemLeadingAvatarShape`
pub const ITEM_LEADING_AVATAR_SHAPE: Shape = Shape::Full;
/// `ItemLeadingAvatarSize`
pub const ITEM_LEADING_AVATAR_SIZE: f32 = 40.0;
/// `ItemLeadingIconColor`
pub const ITEM_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemLeadingIconExpressiveSize`
pub const ITEM_LEADING_ICON_EXPRESSIVE_SIZE: f32 = 20.0;
/// `ItemLeadingIconSize`
pub const ITEM_LEADING_ICON_SIZE: f32 = 24.0;
/// `ItemLeadingImageExpressiveShape`
pub const ITEM_LEADING_IMAGE_EXPRESSIVE_SHAPE: Shape = Shape::Small;
/// `ItemLeadingImageHeight`
pub const ITEM_LEADING_IMAGE_HEIGHT: f32 = 56.0;
/// `ItemLeadingImageShape`
pub const ITEM_LEADING_IMAGE_SHAPE: Shape = Shape::None;
/// `ItemLeadingImageWidth`
pub const ITEM_LEADING_IMAGE_WIDTH: f32 = 56.0;
/// `ItemLeadingSpace`
pub const ITEM_LEADING_SPACE: f32 = 16.0;
/// `ItemLeadingVideoShape`
pub const ITEM_LEADING_VIDEO_SHAPE: Shape = Shape::Small;
/// `ItemLeadingVideoWidth`
pub const ITEM_LEADING_VIDEO_WIDTH: f32 = 100.0;
/// `ItemOneLineContainerHeight`
pub const ITEM_ONE_LINE_CONTAINER_HEIGHT: f32 = 56.0;
/// `ItemOverlineColor`
pub const ITEM_OVERLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemOverlineFont`
pub const ITEM_OVERLINE_FONT: TypeScale = TypeScale::LabelSmall;
/// `ItemPressedContainerExpressiveShape`
pub const ITEM_PRESSED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemPressedLabelTextColor`
pub const ITEM_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemPressedLeadingIconIconColor`
pub const ITEM_PRESSED_LEADING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemPressedTrailingIconIconColor`
pub const ITEM_PRESSED_TRAILING_ICON_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemSegmentedContainerColor`
pub const ITEM_SEGMENTED_CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `ItemSelectedContainerColor`
pub const ITEM_SELECTED_CONTAINER_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `ItemSelectedContainerExpressiveShape`
pub const ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedContainerShape`
pub const ITEM_SELECTED_CONTAINER_SHAPE: Shape = Shape::Large;
/// `ItemSelectedDisabledContainerColor`
pub const ITEM_SELECTED_DISABLED_CONTAINER_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledContainerExpressiveShape`
pub const ITEM_SELECTED_DISABLED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedDisabledContainerOpacity`
pub const ITEM_SELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledLabelTextColor`
pub const ITEM_SELECTED_DISABLED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledLabelTextOpacity`
pub const ITEM_SELECTED_DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledLeadingIconColor`
pub const ITEM_SELECTED_DISABLED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledLeadingIconOpacity`
pub const ITEM_SELECTED_DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledOverlineColor`
pub const ITEM_SELECTED_DISABLED_OVERLINE_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledOverlineOpacity`
pub const ITEM_SELECTED_DISABLED_OVERLINE_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledStateLayerOpacity`
pub const ITEM_SELECTED_DISABLED_STATE_LAYER_OPACITY: f32 = 0.1;
/// `ItemSelectedDisabledSupportingTextColor`
pub const ITEM_SELECTED_DISABLED_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledSupportingTextOpacity`
pub const ITEM_SELECTED_DISABLED_SUPPORTING_TEXT_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledTrailingIconColor`
pub const ITEM_SELECTED_DISABLED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledTrailingIconOpacity`
pub const ITEM_SELECTED_DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
/// `ItemSelectedDisabledTrailingSupportingTextColor`
pub const ITEM_SELECTED_DISABLED_TRAILING_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDisabledTrailingSupportingTextOpacity`
pub const ITEM_SELECTED_DISABLED_TRAILING_SUPPORTING_TEXT_OPACITY: f32 = 0.38;
/// `ItemSelectedDraggedContainerExpressiveShape`
pub const ITEM_SELECTED_DRAGGED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedDraggedLabelTextColor`
pub const ITEM_SELECTED_DRAGGED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedDraggedLeadingIconColor`
pub const ITEM_SELECTED_DRAGGED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedDraggedTrailingIconColor`
pub const ITEM_SELECTED_DRAGGED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedFocusLabelTextColor`
pub const ITEM_SELECTED_FOCUS_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedFocusLeadingIconColor`
pub const ITEM_SELECTED_FOCUS_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedFocusTrailingIconColor`
pub const ITEM_SELECTED_FOCUS_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedFocusedContainerExpressiveShape`
pub const ITEM_SELECTED_FOCUSED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedHoverLabelTextColor`
pub const ITEM_SELECTED_HOVER_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedHoverLeadingIconColor`
pub const ITEM_SELECTED_HOVER_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedHoverTrailingIconColor`
pub const ITEM_SELECTED_HOVER_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedHoveredContainerExpressiveShape`
pub const ITEM_SELECTED_HOVERED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedLabelTextColor`
pub const ITEM_SELECTED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedLeadingIconColor`
pub const ITEM_SELECTED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedOverlineColor`
pub const ITEM_SELECTED_OVERLINE_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedPressedContainerExpressiveShape`
pub const ITEM_SELECTED_PRESSED_CONTAINER_EXPRESSIVE_SHAPE: Shape = Shape::Large;
/// `ItemSelectedPressedLabelTextColor`
pub const ITEM_SELECTED_PRESSED_LABEL_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedPressedLeadingIconColor`
pub const ITEM_SELECTED_PRESSED_LEADING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedPressedTrailingIconColor`
pub const ITEM_SELECTED_PRESSED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `ItemSelectedSupportingTextColor`
pub const ITEM_SELECTED_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedTrailingIconColor`
pub const ITEM_SELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSelectedTrailingSupportingTextColor`
pub const ITEM_SELECTED_TRAILING_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSecondaryContainer;
/// `ItemSmallLeadingVideoHeight`
pub const ITEM_SMALL_LEADING_VIDEO_HEIGHT: f32 = 56.0;
/// `ItemSmallLeadingVideoWidth`
pub const ITEM_SMALL_LEADING_VIDEO_WIDTH: f32 = 100.0;
/// `ItemSupportingTextColor`
pub const ITEM_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemSupportingTextFont`
pub const ITEM_SUPPORTING_TEXT_FONT: TypeScale = TypeScale::BodyMedium;
/// `ItemThreeLineContainerHeight`
pub const ITEM_THREE_LINE_CONTAINER_HEIGHT: f32 = 88.0;
/// `ItemTopSpace`
pub const ITEM_TOP_SPACE: f32 = 10.0;
/// `ItemTrailingIconColor`
pub const ITEM_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemTrailingIconExpressiveSize`
pub const ITEM_TRAILING_ICON_EXPRESSIVE_SIZE: f32 = 20.0;
/// `ItemTrailingIconSize`
pub const ITEM_TRAILING_ICON_SIZE: f32 = 24.0;
/// `ItemTrailingSpace`
pub const ITEM_TRAILING_SPACE: f32 = 16.0;
/// `ItemTrailingSupportingTextColor`
pub const ITEM_TRAILING_SUPPORTING_TEXT_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `ItemTrailingSupportingTextFont`
pub const ITEM_TRAILING_SUPPORTING_TEXT_FONT: TypeScale = TypeScale::LabelSmall;
/// `ItemTwoLineContainerHeight`
pub const ITEM_TWO_LINE_CONTAINER_HEIGHT: f32 = 72.0;
/// `ItemUnselectedTrailingIconColor`
pub const ITEM_UNSELECTED_TRAILING_ICON_COLOR: ColorRole = ColorRole::OnSurface;
/// `SegmentedGap`
pub const SEGMENTED_GAP: f32 = 2.0;
