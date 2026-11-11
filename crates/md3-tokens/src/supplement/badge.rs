// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt
//         (ファイル末尾の internal な定数) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! バッジの値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// 内容のあるバッジの左右の余白。`BadgeWithContentHorizontalPadding`
pub const WITH_CONTENT_HORIZONTAL_PADDING: Dp = Dp(4.0);
/// 内容のあるバッジの左端を、アンカーの右端から内側へずらす量。`BadgeWithContentHorizontalOffset`
pub const WITH_CONTENT_HORIZONTAL_OFFSET: Dp = Dp(12.0);
/// 内容のあるバッジの下端を、アンカーの上端から下へずらす量。`BadgeWithContentVerticalOffset`
pub const WITH_CONTENT_VERTICAL_OFFSET: Dp = Dp(14.0);
/// 内容の無いバッジの左端をアンカーの右端から内側へ、下端をアンカーの上端から下へずらす量。`BadgeOffset`
pub const OFFSET: Dp = Dp(6.0);
