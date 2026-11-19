// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ShortNavigationBar.kt
//         (`ShortNavigationBarItemDefaults`、`TopIconIndicatorToLabelPadding`)、
//         ColorScheme.kt (`contentColorFor`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ナビゲーションバーの値のうち、Compose の実装が固定値として持つもの

use md3_color::ColorRole;

use crate::Dp;

/// 項目の状態レイヤーの色
///
/// Compose の項目の ripple はバーの内容の色を使い、容器の色 SurfaceContainer に `contentColorFor` が組み合わせる色は OnSurface になる
pub const STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;

/// 無効な項目の内容の不透明度
pub const DISABLED_CONTENT_OPACITY: f32 = super::color_scheme::DISABLED_ALPHA;

/// ラベルの上にアイコンを置く項目の、インジケーターとラベルの間隔
pub const TOP_ICON_INDICATOR_TO_LABEL_PADDING: Dp = Dp(4.0);
