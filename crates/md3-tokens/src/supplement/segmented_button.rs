// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt
//         (`SegmentedButtonDefaults` とファイル末尾の private な定数)、Button.kt (`ButtonDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! Segmented button の値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// 項目の左右の余白。`SegmentedButtonDefaults.ContentPadding`
pub const HORIZONTAL_PADDING: Dp = Dp(12.0);
/// 項目の上下の余白。`SegmentedButtonDefaults.ContentPadding` が使う `ButtonDefaults.ContentPadding` の上下の余白
pub const VERTICAL_PADDING: Dp = Dp(8.0);
/// アイコンとラベルの間隔。`IconSpacing`
pub const ICON_SPACING: Dp = Dp(8.0);
