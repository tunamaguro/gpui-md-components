// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt
//         (ファイル末尾の private な値と `Snackbar(snackbarData)` の余白) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! スナックバーの値のうち、Compose の実装が固定値として持つもの
//!
//! 配置は `ComposeMaterial3Flags.isSnackbarStylingFixEnabled` が有効なときの `OneRowSnackbar` と
//! `NewLineButtonSnackbar` の値を転記する。無効なときの Legacy の配置は文のベースラインで位置を決めるため転記しない

use crate::Dp;

/// 容器の最大の幅。`ContainerMaxWidth`
pub const CONTAINER_MAX_WIDTH: Dp = Dp(600.0);
/// 容器の左端と文の間、文を 1 行に置くときの文と右端の間。`HorizontalSpacing`
pub const HORIZONTAL_SPACING: Dp = Dp(16.0);
/// 下の行に置いたボタンと容器の右端の間 (閉じるボタンが無いとき)。`HorizontalSpacingButtonSide`
pub const HORIZONTAL_SPACING_BUTTON_SIDE: Dp = Dp(8.0);
/// 1 行に置いたボタンと容器の右端の間 (閉じるボタンが無いとき)。`TextEndExtraSpacing`
pub const TEXT_END_EXTRA_SPACING: Dp = Dp(8.0);
/// 文の上下の余白。`SnackbarVerticalPadding`
pub const VERTICAL_PADDING: Dp = Dp(14.0);
/// 下の行に置いたボタンの下の余白。`ActionButtonBottomPadding`
pub const ACTION_BUTTON_BOTTOM_PADDING: Dp = Dp(4.0);
/// `SnackbarHost` が描くスナックバーの周りの余白。`Snackbar(snackbarData)` の `padding(12.dp)`
pub const HOST_PADDING: Dp = Dp(12.0);
