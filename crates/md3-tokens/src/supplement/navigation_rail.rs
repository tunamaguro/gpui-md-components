// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ComponentStyles.kt
//         (`NavigationRailStyle.Default`、`NavigationRailItemStyle.Default`、`WideNavigationRailHorizontalPadding`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ナビゲーションレールの値のうち、Compose の実装が固定値として持つもの

use md3_color::ColorRole;

use crate::Dp;

/// 項目の状態レイヤーの色
///
/// Compose の項目の ripple はレールの内容の色を使い、`NavigationRailStyle.Default` はそれを OnSurface にする
pub const STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;

/// 展開したレールの、内容の左右の余白
pub const EXPANDED_HORIZONTAL_PADDING: Dp = Dp(20.0);

/// ラベルの上にアイコンを置く項目の、内容の左右の余白
pub const TOP_ICON_ITEM_HORIZONTAL_PADDING: Dp = Dp(20.0);
