// Ported from AndroidX Compose (https://github.com/androidx/androidx)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scrollbar.kt
// @ b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6
// Copyright 2026 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! スクロールバーの既定値。

use md3_color::ColorRole;

/// 太さ（dp）。
pub const THICKNESS: f32 = 4.0;
/// つまみの色。
pub const THUMB_COLOR: ColorRole = ColorRole::Outline;
/// つまみの不透明度。
pub const THUMB_OPACITY: f32 = 0.7;
/// 最小長（dp）。
pub const THUMB_MIN_LENGTH: f32 = 24.0;
/// トラックに対する最大長の比率。
pub const THUMB_MAX_LENGTH_FRACTION: f32 = 0.9;
/// 主軸の余白（dp）。
pub const MAIN_AXIS_TRACK_INSET: f32 = 2.0;
/// 交差軸の余白（dp）。
pub const CROSS_AXIS_TRACK_INSET: f32 = 0.0;
/// 消えるまでの待機時間（ms）。
pub const FADE_DELAY_MILLIS: u64 = 400;
/// フェード時間（ms）。
pub const FADE_DURATION_MILLIS: u64 = 250;
