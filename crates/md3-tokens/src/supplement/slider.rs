// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt
//         (ファイル末尾の internal な定数) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! スライダーの値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// トラックの、つまみに向いた側の角丸の半径。`TrackInsideCornerSize`
pub const TRACK_INSIDE_CORNER_SIZE: Dp = Dp(2.0);
