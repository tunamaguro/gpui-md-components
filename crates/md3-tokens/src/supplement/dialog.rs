// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AlertDialog.kt
//         (`AlertDialogDefaults` の internal と private な値、ファイル末尾の internal と private な定数)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 幕の不透明度とウィンドウの端からの余白は Material Web から転記している
// (https://github.com/material-components/material-web)
// Source: dialog/internal/_dialog.scss (`.scrim` の opacity、`dialog` の max-height と max-width)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! ダイアログの値のうち、トークンに無いもの
//!
//! Compose の実装が持つ固定値と、Compose に無く Material Web にある値を転記している。
//! Compose の値は `shouldUsePrecisionPointerComponentSizing` が無効な既定の値を使う

use crate::Dp;

/// ダイアログの最小の幅。`DialogMinWidth`
pub const MIN_WIDTH: Dp = Dp(280.0);
/// ダイアログの最大の幅。`DialogMaxWidth`
pub const MAX_WIDTH: Dp = Dp(560.0);
/// ダイアログの縁から内容までの余白。`AlertDialogDefaults.dialogPadding`
pub const PADDING: Dp = Dp(24.0);
/// アイコンの下の余白。`IconPadding`
pub const ICON_BOTTOM_PADDING: Dp = Dp(16.0);
/// 見出しの下の余白。`TitlePadding`
pub const TITLE_BOTTOM_PADDING: Dp = Dp(16.0);
/// 本文の下の余白。`AlertDialogDefaults.textPadding`
pub const TEXT_BOTTOM_PADDING: Dp = Dp(24.0);
/// 横に並ぶボタンの間隔。`ButtonsMainAxisSpacing`
pub const BUTTONS_MAIN_AXIS_SPACING: Dp = Dp(8.0);
/// 縦に並ぶボタンの間隔。`ButtonsCrossAxisSpacing`
pub const BUTTONS_CROSS_AXIS_SPACING: Dp = Dp(8.0);

/// 幕の不透明度。Material Web の `.scrim` の `opacity: 32%`
pub const SCRIM_OPACITY: f32 = 0.32;
/// ダイアログとウィンドウの端の間に空ける余白。Material Web の `calc(100% - 48px)` の片側
pub const WINDOW_MARGIN: Dp = Dp(24.0);
