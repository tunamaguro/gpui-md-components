// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/InteractiveComponentSize.kt
//         (`LocalMinimumInteractiveComponentSize`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! 操作できる部品の大きさ

use crate::Dp;

/// 操作できる部品が確保する、操作領域の最小の幅と高さ
///
/// 見た目がこれより小さい部品も、周囲に余白をとってこの大きさの操作領域を確保する
pub const MIN_SIZE: Dp = Dp(48.0);
