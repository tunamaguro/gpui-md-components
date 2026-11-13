// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt
//         (ファイル末尾の internal と private な定数)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MenuDefaults.kt
//         (`MenuDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022, 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! メニューの値のうち、トークンに無いもの
//!
//! Compose の実装が持つ固定値から転記している。
//! `ComposeMaterial3Flags.isPrecisionPointerComponentSizingEnabled` が無効な既定の値を使う

use crate::Dp;

/// メニューを置くときに空ける、ウィンドウの左右の端からの余白。`MenuHorizontalMargin`
pub const HORIZONTAL_MARGIN: Dp = Dp(8.0);
/// メニューを置くときに空ける、ウィンドウの上下の端からの余白。`MenuVerticalMargin`
pub const VERTICAL_MARGIN: Dp = Dp(48.0);
/// 項目と組のラベルの最小の幅。`DropdownMenuItemDefaultMinWidth`
pub const ITEM_MIN_WIDTH: Dp = Dp(112.0);
/// 項目と組のラベルの最大の幅。`DropdownMenuItemDefaultMaxWidth`
pub const ITEM_MAX_WIDTH: Dp = Dp(280.0);
/// 項目の内容の左右の余白。`DropdownMenuItemHorizontalPadding`
pub const ITEM_HORIZONTAL_PADDING: Dp = Dp(12.0);
/// 項目の内容の上下の余白。`MenuDefaults.DropdownMenuSelectableItemContentPadding`
pub const ITEM_VERTICAL_PADDING: Dp = Dp(12.0);
/// 項目の容器の外側の左右の余白。`DropdownMenuSelectableItemPadding`
pub const ITEM_OUTER_HORIZONTAL_PADDING: Dp = Dp(4.0);
/// 補足の文を持つ項目の容器の外側の上下の余白。`DropdownMenuSelectableItemWithSupportTexPadding`
pub const ITEM_WITH_SUPPORTING_TEXT_OUTER_VERTICAL_PADDING: Dp = Dp(2.0);
/// 項目の先頭のアイコンと文の間隔。文と末尾の内容の間隔の最小値にもなる。
/// `MenuDefaults.itemHorizontalArrangement` の `spacing`
pub const ICON_TEXT_SPACING: Dp = Dp(8.0);
/// 組の内容の上下の余白。`DropdownMenuGroupVerticalPadding`
pub const GROUP_VERTICAL_PADDING: Dp = Dp(2.0);
/// 組のラベルの最小の高さ。`DropdownMenuGroupDefaultMinHeight`
pub const GROUP_LABEL_MIN_HEIGHT: Dp = Dp(32.0);
/// 組のラベルの先頭の余白。`MenuDefaults.DropdownMenuGroupLabelHorizontalPadding`
pub const GROUP_LABEL_START_PADDING: Dp = Dp(12.0);
/// 組のラベルの末尾の余白。`MenuDefaults.DropdownMenuGroupLabelHorizontalPadding`
pub const GROUP_LABEL_END_PADDING: Dp = Dp(4.0);
/// 組の中の区切り線の左右の余白。`MenuDefaults.HorizontalDividerPadding`
pub const DIVIDER_HORIZONTAL_PADDING: Dp = Dp(12.0);
/// 組の中の区切り線の上下の余白。`MenuDefaults.HorizontalDividerPadding`
pub const DIVIDER_VERTICAL_PADDING: Dp = Dp(2.0);
