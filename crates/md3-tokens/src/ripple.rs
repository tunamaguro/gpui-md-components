// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tokens/_md-comp-ripple.scss (values), ripple/internal/ripple.ts (constants),
//         ripple/internal/_ripple.scss (transitions), internal/motion/animation.ts (EASING)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// animation.ts: Copyright 2021 Google LLC.
// 不透明度は Compose の StateTokens.kt を参照する (generated/state.rs に出典がある)

//! 押下と hover を示すリップル
//!
//! マウスとキーボードで操作する環境に合わせ、Material Web の `md-ripple` の値を転記している。
//! Material Web の CSS px は [`Dp`] として扱う
//!
//! 押下の波紋は、直径が部品の長辺の [`INITIAL_ORIGIN_SCALE`] 倍の円を、押した位置から部品の中心へ動かしながら、
//! 直径が (部品の対角線の長さ + [`PADDING`] + ぼかしの幅) になるまで [`PRESS_GROW`] かけて広げる。
//! ぼかしの幅は 長辺 × [`SOFT_EDGE_CONTAINER_RATIO`] と [`SOFT_EDGE_MINIMUM_SIZE`] の大きい方。
//! 塗りは広げる前の円に対して決め、中心から 半径の 65% と (半径 - 70) の大きい方までを塗り、
//! そこから縁へ透明にする。広げるときは塗りも同じ倍率で拡大する

use std::time::Duration;

use md3_color::ColorRole;

use crate::Dp;
use crate::motion::CubicBezier;
use crate::state;

/// hover の色。`md.comp.ripple.hover-color` (`md.sys.color.on-surface`)
pub const HOVER_COLOR: ColorRole = ColorRole::OnSurface;

/// hover の不透明度。`md.sys.state.hover-state-layer-opacity`
pub const HOVER_OPACITY: f32 = state::HOVER_STATE_LAYER_OPACITY;

/// 押下の色。`md.comp.ripple.pressed-color` (`md.sys.color.on-surface`)
pub const PRESSED_COLOR: ColorRole = ColorRole::OnSurface;

/// 押下の不透明度。`md.sys.state.pressed-state-layer-opacity`
///
/// Compose の `StateTokens.PressedStateLayerOpacity` (0.1、tokens VERSION v0_210) を使う。
/// Material Web の `_md-sys-state.scss` (v0_192) の値は 0.12 で、数値は Compose を優先している
pub const PRESSED_OPACITY: f32 = state::PRESSED_STATE_LAYER_OPACITY;

/// 波紋が広がる時間 (`PRESS_GROW_MS`)
pub const PRESS_GROW: Duration = Duration::from_millis(450);

/// 波紋が広がるときのイージング (`EASING.STANDARD`)
pub const PRESS_GROW_EASING: CubicBezier = CubicBezier::new(0.2, 0.0, 0.0, 1.0);

/// 波紋が広がり始めてから消し始めるまでの最短の時間 (`MINIMUM_PRESS_MS`)
///
/// 押下を離した時点で広がり始めてからこの時間が過ぎていればすぐに消し始め、
/// 過ぎていなければ残りの時間だけ待つ
pub const MINIMUM_PRESS: Duration = Duration::from_millis(225);

/// タッチで押してから押下として扱うまでの待ち時間 (`TOUCH_DELAY_MS`)。マウスには適用しない
pub const TOUCH_DELAY: Duration = Duration::from_millis(150);

/// 波紋の初めの直径の、部品の長辺に対する比 (`INITIAL_ORIGIN_SCALE`)。CSS では小数点以下を切り捨てる
pub const INITIAL_ORIGIN_SCALE: f32 = 0.2;

/// 部品の対角線の長さに加える、波紋の最終的な大きさの余白 (`PADDING`)
pub const PADDING: Dp = Dp(10.0);

/// 波紋の縁のぼかしの最小の幅 (`SOFT_EDGE_MINIMUM_SIZE`)
pub const SOFT_EDGE_MINIMUM_SIZE: Dp = Dp(75.0);

/// 波紋の縁のぼかしの幅の、部品の長辺に対する比 (`SOFT_EDGE_CONTAINER_RATIO`)
pub const SOFT_EDGE_CONTAINER_RATIO: f32 = 0.35;

/// hover の表示と色の切り替えにかける時間。線形に変化する
pub const HOVER_TRANSITION: Duration = Duration::from_millis(15);

/// 押下の波紋を表示するときの不透明度の変化の時間。線形に変化する
pub const PRESSED_FADE_IN: Duration = Duration::from_millis(105);

/// 押下の波紋を消すときの不透明度の変化の時間。線形に変化する
pub const PRESSED_FADE_OUT: Duration = Duration::from_millis(375);
