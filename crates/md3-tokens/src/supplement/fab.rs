// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButton.kt
//         (`FloatingActionButtonDefaults` とファイル末尾の private な定数) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! FAB の値のうち、Compose の実装が固定値として持つもの

use crate::Dp;
use crate::shape::ShapeKey;
use crate::typography::TypeScale;

/// Medium の FAB と Extended FAB の形。`FloatingActionButtonDefaults.mediumShape` と `mediumExtendedFabShape`
///
/// Compose は TODO 付きで、トークンの代わりに `ShapeDefaults.LargeIncreased` を使う
pub const MEDIUM_CONTAINER_SHAPE: ShapeKey = ShapeKey::CornerLargeIncreased;

/// Large の FAB と Extended FAB のアイコンの大きさ。`FloatingActionButtonDefaults.LargeIconSize`
///
/// Compose は TODO 付きで、`FabLargeTokens.IconSize` (32dp) と異なる値を使う。
/// Compose の FAB はアイコンの大きさを決めず、サンプルがこの値をアイコンに指定する
pub const LARGE_ICON_SIZE: Dp = Dp(36.0);

/// Medium の Extended FAB のアイコンとラベルの間隔。`MediumExtendedFabIconPadding`
///
/// Compose は TODO 付きで、`ExtendedFabMediumTokens.IconLabelSpace` (16dp) と異なる値を使う
pub const MEDIUM_EXTENDED_ICON_LABEL_SPACE: Dp = Dp(12.0);
/// Large の Extended FAB のアイコンとラベルの間隔。`LargeExtendedFabIconPadding`
///
/// Compose は TODO 付きで、`ExtendedFabLargeTokens.IconLabelSpace` (20dp) と異なる値を使う
pub const LARGE_EXTENDED_ICON_LABEL_SPACE: Dp = Dp(16.0);

/// Small の Extended FAB のラベルの書体。`SmallExtendedFabTextStyle`
pub const SMALL_EXTENDED_LABEL_FONT: TypeScale = TypeScale::TitleMedium;
/// Medium の Extended FAB のラベルの書体。`MediumExtendedFabTextStyle`
pub const MEDIUM_EXTENDED_LABEL_FONT: TypeScale = TypeScale::TitleLarge;
/// Large の Extended FAB のラベルの書体。`LargeExtendedFabTextStyle`
pub const LARGE_EXTENDED_LABEL_FONT: TypeScale = TypeScale::HeadlineSmall;
