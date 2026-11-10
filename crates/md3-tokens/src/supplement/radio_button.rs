// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tokens/versions/v0_192/_md-comp-radio-button.scss @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// 不透明度は Compose の StateTokens.kt を参照する (generated/state.rs に出典がある)
//
// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt
//         (`RadioButtonDotSize`、`RadioStrokeWidth`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ラジオボタンの値のうち、Compose のトークンに無いもの
//!
//! 状態レイヤーの色は Material Web から、円の寸法は Compose の実装が持つ固定値から転記している。
//! 状態レイヤーの不透明度は [`state`](crate::state) のものを使う

use md3_color::ColorRole;

use crate::Dp;

/// 未選択で hover 中の状態レイヤーの色。`md.comp.radio-button.unselected.hover.state-layer.color`
pub const UNSELECTED_HOVER_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
/// 未選択でフォーカス中の状態レイヤーの色。`md.comp.radio-button.unselected.focus.state-layer.color`
pub const UNSELECTED_FOCUS_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
/// 選択中で hover 中の状態レイヤーの色。`md.comp.radio-button.selected.hover.state-layer.color`
pub const SELECTED_HOVER_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
/// 選択中でフォーカス中の状態レイヤーの色。`md.comp.radio-button.selected.focus.state-layer.color`
pub const SELECTED_FOCUS_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;

/// 選択中に描く内側の円の、線の中心で測った直径。`RadioButtonDotSize`
///
/// Compose は半径から線の太さの半分を引いて塗るため、塗られる円の直径はこの値から [`STROKE_WIDTH`] を引いた大きさになる
pub const DOT_SIZE: Dp = Dp(12.0);
/// 外側の円の線の太さ。`RadioStrokeWidth`
pub const STROKE_WIDTH: Dp = Dp(2.0);
