// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tab.kt
//         (ファイル末尾の private と internal な定数)、TabRow.kt (`TabRowImpl`、`TabRowDefaults.PrimaryIndicator`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tabs/internal/_tab.scss (`md-focus-ring` の `shape`) @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! タブの値のうち、Compose のトークンに無いもの
//!
//! 寸法は Compose の実装が持つ固定値から、フォーカスリングの角丸は Material Web から転記している

use crate::{Dp, Sp};

/// アイコンとラベルの両方を持つタブの高さ。`LargeTabHeight`
pub const LARGE_TAB_HEIGHT: Dp = Dp(72.0);
/// ラベルの左右の余白。`HorizontalTextPadding`
pub const HORIZONTAL_TEXT_PADDING: Dp = Dp(16.0);
/// アイコンとラベルの両方を持つタブで、インジケーターの上端から 1 行のラベルの基準線までの距離。
/// `SingleLineTextBaselineWithIcon`
pub const SINGLE_LINE_TEXT_BASELINE_WITH_ICON: Dp = Dp(14.0);
/// アイコンとラベルの両方を持つタブで、ラベルの最初の基準線からアイコンの下端までの距離。`IconDistanceFromBaseline`
pub const ICON_DISTANCE_FROM_BASELINE: Sp = Sp(20.0);
/// Primary のインジケーターの最小の幅。`TabRowImpl` がタブの内容の幅に課す最小値
pub const PRIMARY_INDICATOR_MIN_WIDTH: Dp = Dp(24.0);
/// フォーカスリングの角丸の半径。Material Web の `md-focus-ring` の `shape`
pub const FOCUS_RING_CORNER_RADIUS: Dp = Dp(8.0);
/// 選ばれたタブで、インジケーターの上端からフォーカスリングの下端までの距離。
/// Material Web の `:host([active]) md-focus-ring` の `margin-bottom` で、インジケーターの高さに足す値
pub const FOCUS_RING_ACTIVE_BOTTOM_GAP: Dp = Dp(1.0);
