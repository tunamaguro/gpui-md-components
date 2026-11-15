// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt
//         (`ExposedDropdownMenuItemHorizontalPadding`)、
//         Menu.kt (`DropdownMenuVerticalPadding`、`MenuListItemContainerHeight`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! セレクト (Compose の `ExposedDropdownMenuBox` の中に置いた読み取り専用の入力欄と一覧) の一覧の値のうち、トークンに無いもの
//!
//! Compose の実装が持つ固定値から転記している。入力欄の値は [`super::text_field`] にある

use crate::Dp;

/// 一覧の内容の上下の余白。`DropdownMenuVerticalPadding`
pub const LISTBOX_VERTICAL_PADDING: Dp = Dp(8.0);
/// 一覧の項目の最小の高さ。`MenuListItemContainerHeight`
pub const OPTION_MIN_HEIGHT: Dp = Dp(48.0);
/// 一覧の項目の内容の左右の余白。`ExposedDropdownMenuItemHorizontalPadding`
pub const OPTION_HORIZONTAL_PADDING: Dp = Dp(16.0);
