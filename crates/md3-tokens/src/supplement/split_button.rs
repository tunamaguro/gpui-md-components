// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SplitButton.kt
//         (`SplitButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! 分割ボタンの値のうち、Compose が `SplitButtonDefaults` に固定値として持つもの

use crate::Dp;

/// 前のボタンと後ろのボタンの最小の幅。`LeadingButtonMinWidth` と `TrailingButtonMinWidth`。どのサイズにも使う
pub const MIN_WIDTH: Dp = Dp(48.0);
