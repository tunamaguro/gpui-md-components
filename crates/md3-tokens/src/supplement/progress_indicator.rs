// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt
//         (ファイル末尾の internal な定数、`ProgressIndicatorDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! 進捗インジケーターの値のうち、Compose の実装が固定値として持つもの
//!
//! 時刻は 1 周期の始まりからの経過時間で表す

use std::time::Duration;

use crate::Dp;
use crate::motion::{
    CubicBezier, EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER, EASING_STANDARD_CUBIC_BEZIER,
};

/// 直線の既定の幅。`LinearIndicatorWidth`
///
/// 仕様にはあるがトークンにない値
pub const LINEAR_WIDTH: Dp = Dp(240.0);

/// 終端の点と直線の端の間隔の上限。`StopIndicatorTrailingSpace`
pub const STOP_TRAILING_SPACE_MAX: Dp = Dp(6.0);

/// 不確定の直線の 1 周期。`LinearAnimationDuration`
pub const LINEAR_INDETERMINATE_DURATION: Duration = Duration::from_millis(1750);

/// 不確定の直線で、1 本目の先端が動き始める時刻。`FirstLineHeadDelay`
pub const LINEAR_FIRST_LINE_HEAD_DELAY: Duration = Duration::from_millis(0);
/// 不確定の直線で、1 本目の先端が端から端まで動く時間。`FirstLineHeadDuration`
pub const LINEAR_FIRST_LINE_HEAD_DURATION: Duration = Duration::from_millis(1000);
/// 不確定の直線で、1 本目の末端が動き始める時刻。`FirstLineTailDelay`
pub const LINEAR_FIRST_LINE_TAIL_DELAY: Duration = Duration::from_millis(250);
/// 不確定の直線で、1 本目の末端が端から端まで動く時間。`FirstLineTailDuration`
pub const LINEAR_FIRST_LINE_TAIL_DURATION: Duration = Duration::from_millis(1000);
/// 不確定の直線で、2 本目の先端が動き始める時刻。`SecondLineHeadDelay`
pub const LINEAR_SECOND_LINE_HEAD_DELAY: Duration = Duration::from_millis(650);
/// 不確定の直線で、2 本目の先端が端から端まで動く時間。`SecondLineHeadDuration`
pub const LINEAR_SECOND_LINE_HEAD_DURATION: Duration = Duration::from_millis(850);
/// 不確定の直線で、2 本目の末端が動き始める時刻。`SecondLineTailDelay`
pub const LINEAR_SECOND_LINE_TAIL_DELAY: Duration = Duration::from_millis(900);
/// 不確定の直線で、2 本目の末端が端から端まで動く時間。`SecondLineTailDuration`
pub const LINEAR_SECOND_LINE_TAIL_DURATION: Duration = Duration::from_millis(850);
/// 不確定の直線で、線の端が動くときのイージング。`LinearIndeterminateProgressEasing`
pub const LINEAR_INDETERMINATE_EASING: CubicBezier = EASING_EMPHASIZED_ACCELERATE_CUBIC_BEZIER;

/// 不確定の円の 1 周期。`CircularAnimationProgressDuration`
pub const CIRCULAR_INDETERMINATE_DURATION: Duration = Duration::from_millis(6000);
/// 不確定の円で、1 周期の間に一定の速さで回る角度 (度)。`CircularGlobalRotationDegreesTarget`
pub const CIRCULAR_GLOBAL_ROTATION_DEGREES: f32 = 1080.0;
/// 不確定の円で、1 周期の間に追加で回る角度 (度)。`CircularAdditionalRotationDegreesTarget`
///
/// 4 回に分けて、90 度ずつ回る
pub const CIRCULAR_ADDITIONAL_ROTATION_DEGREES: f32 = 360.0;
/// 不確定の円で、追加の回転が始まる間隔。`CircularAnimationAdditionalRotationDelay`
pub const CIRCULAR_ADDITIONAL_ROTATION_INTERVAL: Duration = Duration::from_millis(1500);
/// 不確定の円で、追加の回転 1 回にかかる時間。`CircularAnimationAdditionalRotationDuration`
pub const CIRCULAR_ADDITIONAL_ROTATION_DURATION: Duration = Duration::from_millis(300);
/// 不確定の円で、弧の長さの最小 (円周に対する割合)。`CircularIndeterminateMinProgress`
pub const CIRCULAR_INDETERMINATE_MIN_PROGRESS: f32 = 0.1;
/// 不確定の円で、弧の長さの最大 (円周に対する割合)。`CircularIndeterminateMaxProgress`
pub const CIRCULAR_INDETERMINATE_MAX_PROGRESS: f32 = 0.87;
/// 不確定の円で、弧が最大から最小へ縮むときのイージング。`CircularProgressEasing`
pub const CIRCULAR_INDETERMINATE_EASING: CubicBezier = EASING_STANDARD_CUBIC_BEZIER;
