// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt
//         (ファイル末尾の private な値) と InteractiveComponentSize.kt (`minimumInteractiveComponentSize`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! トップアプリバーとボトムアプリバーの値のうち、Compose の実装が固定値として持つもの

use crate::Dp;

/// ナビゲーションのアイコンが無いときの、容器の左端からタイトルの領域までの最小の距離。
/// タイトルの領域はさらに左右に [`LEADING_SPACE`](crate::component::app_bar::LEADING_SPACE) の余白を持つ。
/// `TopAppBarTitleInset`
pub const TITLE_INSET: Dp = Dp(12.0);
/// Medium のトップアプリバーの、下の行の下端からタイトルの最後の基準線までの距離。`MediumTitleBottomPadding`
pub const MEDIUM_TITLE_BOTTOM_PADDING: Dp = Dp(24.0);
/// Large のトップアプリバーの、下の行の下端からタイトルの最後の基準線までの距離。`LargeTitleBottomPadding`
pub const LARGE_TITLE_BOTTOM_PADDING: Dp = Dp(28.0);
/// アイコンボタンを置く配置枠の大きさ。Compose の `IconButton` は `minimumInteractiveComponentSize` で、
/// 容器の周りに 48dp まで配置の大きさを取る
pub const ICON_TARGET_SIZE: Dp = Dp(48.0);
/// ボトムアプリバーの容器の左右の端から内容までの距離。`BottomAppBarHorizontalPadding`
pub const BOTTOM_HORIZONTAL_PADDING: Dp = Dp(4.0);
/// ボトムアプリバーの容器の上端から内容までの距離。下端には余白を取らない。`BottomAppBarVerticalPadding`
pub const BOTTOM_VERTICAL_PADDING: Dp = Dp(4.0);
/// ボトムアプリバーの内容の右端から FAB までの距離。容器の右端からは 16dp になる。`FABHorizontalPadding`
pub const BOTTOM_FAB_HORIZONTAL_PADDING: Dp = Dp(12.0);
/// ボトムアプリバーの内容の上端から FAB までの距離。容器の上端からは 12dp になる。`FABVerticalPadding`
pub const BOTTOM_FAB_VERTICAL_PADDING: Dp = Dp(8.0);
