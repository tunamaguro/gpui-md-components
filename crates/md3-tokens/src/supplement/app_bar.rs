// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt
//         (ファイル末尾の private な値) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! トップアプリバーの値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// ナビゲーションのアイコンが無いときの、容器の左端からタイトルの領域までの最小の距離。
/// タイトルの領域はさらに左右に [`LEADING_SPACE`](crate::component::app_bar::LEADING_SPACE) の余白を持つ。
/// `TopAppBarTitleInset`
pub const TITLE_INSET: Dp = Dp(12.0);
/// Medium のトップアプリバーの、下の行の下端からタイトルの最後の基準線までの距離。`MediumTitleBottomPadding`
pub const MEDIUM_TITLE_BOTTOM_PADDING: Dp = Dp(24.0);
/// Large のトップアプリバーの、下の行の下端からタイトルの最後の基準線までの距離。`LargeTitleBottomPadding`
pub const LARGE_TITLE_BOTTOM_PADDING: Dp = Dp(28.0);
/// アイコンボタンを置く枠の大きさ。Compose の `IconButton` は容器の周りに、
/// `minimumInteractiveComponentSize` (InteractiveComponentSize.kt) の 48dp まで押せる範囲を広げる
pub const ICON_TARGET_SIZE: Dp = Dp(48.0);
