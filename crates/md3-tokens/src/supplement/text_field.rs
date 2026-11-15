// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TextFieldDefaults.kt
//         (`TextFieldDefaults.MinHeight`、`TextFieldDefaults.MinWidth`)、
//         TextField.kt (`TextFieldWithLabelVerticalPadding`)、
//         OutlinedTextField.kt (`OutlinedTextFieldInnerPadding`)、
//         internal/TextFieldImpl.kt (ファイル末尾の internal な定数)、
//         InteractiveComponentSize.kt (`LocalMinimumInteractiveComponentSize` の既定)、
//         MaterialTheme.kt (`TextSelectionBackgroundOpacity`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! テキストフィールドの値のうち、トークンに無いもの
//!
//! Compose の実装が持つ固定値から転記している

use crate::Dp;

/// 入力欄の最小の高さ。`TextFieldDefaults.MinHeight`
pub const MIN_HEIGHT: Dp = Dp(56.0);
/// 入力欄の最小の幅。`TextFieldDefaults.MinWidth`
pub const MIN_WIDTH: Dp = Dp(280.0);
/// 入力欄の内容の左右の余白。`TextFieldPadding`
pub const HORIZONTAL_PADDING: Dp = Dp(16.0);
/// Filled の入力欄の、上端から浮かせたラベルまでと、文から下端までの余白。`TextFieldWithLabelVerticalPadding`
pub const FILLED_LABEL_VERTICAL_PADDING: Dp = Dp(8.0);
/// 先頭と末尾のアイコンを置く箱の大きさ。アイコンはこの中央に置く。`LocalMinimumInteractiveComponentSize` の既定
pub const ICON_BOX_SIZE: Dp = Dp(48.0);
/// Outlined の入力欄の枠線を、浮かせたラベルの左右に空ける幅。`OutlinedTextFieldInnerPadding`
pub const OUTLINED_LABEL_CUTOUT_PADDING: Dp = Dp(4.0);
/// 補足の文の上の余白。`SupportingTopPadding`
pub const SUPPORTING_TOP_PADDING: Dp = Dp(4.0);
/// 接頭辞と文の間、文と接尾辞の間の余白。`PrefixSuffixTextPadding`
pub const PREFIX_SUFFIX_PADDING: Dp = Dp(2.0);
/// 選択している範囲の背景に使う、Primary の不透明度。`TextSelectionBackgroundOpacity`
pub const SELECTION_OPACITY: f32 = 0.4;
