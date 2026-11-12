// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Card.kt
//         (`CardDefaults`)、ColorScheme.kt (`contentColorFor`、`DisabledAlpha`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! カードの値のうち、Compose の実装が固定値として持つもの

use md3_color::ColorRole;

/// 内容の色
///
/// `CardDefaults` は容器の色に `contentColorFor` が組み合わせる色を使う。
/// Filled・Elevated・Outlined の容器の色 (SurfaceContainerHighest・SurfaceContainerLow・Surface) はどれも OnSurface になる
pub const CONTENT_COLOR: ColorRole = ColorRole::OnSurface;

/// 無効なときの内容の不透明度。`DisabledAlpha`
pub const DISABLED_CONTENT_OPACITY: f32 = 0.38;
