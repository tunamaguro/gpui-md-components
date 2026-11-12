// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Chip.kt
//         (`AssistChipDefaults`、`FilterChipDefaults`、`SuggestionChipDefaults`、`ChipArrangement`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! チップの値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// 容器の左右の端から内容までの余白。Assist・Filter・Suggestion の `ContentPadding`
pub const CONTENT_PADDING: Dp = Dp(8.0);
/// ラベルの左右に置く間隔。`HorizontalSpacing`
///
/// `ChipArrangement` はアイコンの有無によらずラベルの左右にこの間隔を取るため、
/// アイコンの無い側では容器の端からラベルまで [`CONTENT_PADDING`] との和だけ離れる
pub const HORIZONTAL_SPACING: Dp = Dp(8.0);
