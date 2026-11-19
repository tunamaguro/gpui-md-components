// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt
//         (ファイル末尾の private な定数、`LoadingIndicatorImpl` の `morphAnimationSpec`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ローディングインジケーターの値のうち、Compose の実装が固定値として持つもの

use std::time::Duration;

use crate::motion::Spring;

/// 不確定のときに全体が 1 回転する時間。`GlobalRotationDurationMillis`
pub const GLOBAL_ROTATION_DURATION: Duration = Duration::from_millis(4666);

/// 不確定のときに、次の形へ変わり始めるまでの間隔。`MorphIntervalMillis`
pub const MORPH_INTERVAL: Duration = Duration::from_millis(650);

/// 不確定のときに形が変わるバネ。`morphAnimationSpec`
pub const MORPH_SPRING: Spring = Spring {
    damping_ratio: 0.6,
    stiffness: 200.0,
};

/// 形が変わるバネが止まったとみなす、目標との差。`morphAnimationSpec` の `visibilityThreshold`
///
/// 形の変化の進み (0〜1) で表す
pub const MORPH_VISIBILITY_THRESHOLD: f32 = 0.1;
