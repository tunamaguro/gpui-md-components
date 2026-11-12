// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Chip.kt
//         (`AssistChipDefaults`、`FilterChipDefaults`、`InputChipDefaults`、`SuggestionChipDefaults`、`ChipArrangement`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// Ported from Material Web (https://github.com/material-components/material-web)
// Source: chips/internal/_trailing-icon.scss (`.trailing.action` の `md-ripple`) @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! チップの値のうち、トークンに無いもの
//!
//! 余白は Compose の実装が持つ固定値から、削除ボタンの状態レイヤーの大きさは Material Web から転記している

use crate::Dp;

/// 容器の左右の端から内容までの余白。Assist・Filter・Suggestion の `ContentPadding`
pub const CONTENT_PADDING: Dp = Dp(8.0);
/// ラベルの左右に置く間隔。`HorizontalSpacing`
///
/// `ChipArrangement` はアイコンの有無によらずラベルの左右にこの間隔を取るため、
/// アイコンの無い側では容器の端からラベルまで [`CONTENT_PADDING`] との和だけ離れる
pub const HORIZONTAL_SPACING: Dp = Dp(8.0);
/// Input chip で、アバターを置くかアイコンの無い先頭と、末尾のアイコンの無い末尾に取る余白。
/// `InputChipDefaults.contentPadding`。ほかの場合は [`CONTENT_PADDING`] を取る
pub const INPUT_COMPACT_CONTENT_PADDING: Dp = Dp(4.0);
/// Input chip の削除ボタンで、hover とフォーカスの状態レイヤーを描く円の直径。
/// Material Web はアイコンの大きさの 4/3 倍とする
pub const REMOVE_STATE_LAYER_SIZE: Dp = Dp(24.0);
