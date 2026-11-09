// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tokens/versions/v0_192/_md-comp-switch.scss @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// 不透明度は Compose の StateTokens.kt を参照する (generated/state.rs に出典がある)

//! スイッチの値のうち、Compose のトークンに無く Material Web にあるもの
//!
//! 状態レイヤーの不透明度は [`state`](crate::state) のものを使う

use md3_color::ColorRole;

/// 未選択で hover 中の状態レイヤーの色。`md.comp.switch.unselected.hover.state-layer.color`
pub const UNSELECTED_HOVER_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
/// 未選択でフォーカス中の状態レイヤーの色。`md.comp.switch.unselected.focus.state-layer.color`
pub const UNSELECTED_FOCUS_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
/// 選択中で hover 中の状態レイヤーの色。`md.comp.switch.selected.hover.state-layer.color`
pub const SELECTED_HOVER_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
/// 選択中でフォーカス中の状態レイヤーの色。`md.comp.switch.selected.focus.state-layer.color`
pub const SELECTED_FOCUS_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
