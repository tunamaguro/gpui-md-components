// Ported from Jetpack Window (https://android.googlesource.com/platform/frameworks/support)
// Source: window/window-core/src/commonMain/kotlin/androidx/window/core/layout/WindowSizeClass.kt
//         (WindowSizeClass.Companion) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ウィンドウのサイズ区分の境界
//!
//! 区分はウィンドウの幅と高さ (dp) を、各区分の下限と比べて決める。
//! 下限以上なら、その区分以上の広さがある。現在のウィンドウからの区分の判定は扱わない

use crate::Dp;

/// 幅が Medium 以上になる下限
pub const WIDTH_MEDIUM_LOWER_BOUND: Dp = Dp(600.0);

/// 幅が Expanded 以上になる下限
pub const WIDTH_EXPANDED_LOWER_BOUND: Dp = Dp(840.0);

/// 幅が Large 以上になる下限
pub const WIDTH_LARGE_LOWER_BOUND: Dp = Dp(1200.0);

/// 幅が Extra Large 以上になる下限
pub const WIDTH_EXTRA_LARGE_LOWER_BOUND: Dp = Dp(1600.0);

/// 高さが Medium 以上になる下限
pub const HEIGHT_MEDIUM_LOWER_BOUND: Dp = Dp(480.0);

/// 高さが Expanded 以上になる下限
pub const HEIGHT_EXPANDED_LOWER_BOUND: Dp = Dp(900.0);

/// 幅の区分の下限 (`BREAKPOINTS_V1`)。Compact / Medium / Expanded の順
pub const WIDTH_BREAKPOINTS_V1: [Dp; 3] = [
    Dp(0.0),
    WIDTH_MEDIUM_LOWER_BOUND,
    WIDTH_EXPANDED_LOWER_BOUND,
];

/// 幅の区分の下限 (`BREAKPOINTS_V2`)。V1 に Large / Extra Large を加えたもの
pub const WIDTH_BREAKPOINTS_V2: [Dp; 5] = [
    Dp(0.0),
    WIDTH_MEDIUM_LOWER_BOUND,
    WIDTH_EXPANDED_LOWER_BOUND,
    WIDTH_LARGE_LOWER_BOUND,
    WIDTH_EXTRA_LARGE_LOWER_BOUND,
];

/// 高さの区分の下限。Compact / Medium / Expanded の順で、V1 と V2 で同じ
pub const HEIGHT_BREAKPOINTS: [Dp; 3] = [
    Dp(0.0),
    HEIGHT_MEDIUM_LOWER_BOUND,
    HEIGHT_EXPANDED_LOWER_BOUND,
];
