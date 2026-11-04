// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tokens/_md-comp-focus-ring.scss (values), focus/internal/_focus-ring.scss (styles),
//         tokens/versions/latest/sass/_md-sys-state-focus-indicator.scss (Version 34.0.21)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// _md-sys-state-focus-indicator.scss: Copyright 2026 Google LLC.

//! キーボードのフォーカスを示すリング
//!
//! Compose はフォーカスの表示をトークンに持たないため、Material Web から転記している。
//! Material Web の CSS px は [`Dp`] として扱う

use std::time::Duration;

use md3_color::ColorRole;

use crate::Dp;
use crate::motion::{self, CubicBezier};
use crate::shape::ShapeKey;

/// リングの色。`md.comp.focus-ring.color` (`md.sys.color.secondary`)
pub const COLOR: ColorRole = ColorRole::Secondary;

/// 表示中のリングの太さ
pub const WIDTH: Dp = Dp(3.0);

/// 表示し始めのアニメーションで最も太くなるときの太さ
pub const ACTIVE_WIDTH: Dp = Dp(8.0);

/// 部品の外側に描くときの、部品の縁からの距離
pub const OUTWARD_OFFSET: Dp = Dp(2.0);

/// 部品の内側に描くときの、部品の縁からの距離
pub const INWARD_OFFSET: Dp = Dp(0.0);

/// リングの形。部品の角丸に合わせる場合は部品側の形を使う
pub const SHAPE: ShapeKey = ShapeKey::CornerFull;

/// 表示し始めのアニメーション全体の時間。`md.sys.motion.duration-long4`
pub const DURATION: Duration = motion::DURATION_LONG4;

/// アニメーションのイージング。`md.sys.motion.easing-emphasized`
pub const EASING: CubicBezier = motion::EASING_EMPHASIZED_CUBIC_BEZIER;

/// [`DURATION`] のうち、太さを 0 から [`ACTIVE_WIDTH`] まで広げる割合。
/// 残りの時間で [`WIDTH`] まで戻す
pub const GROW_FRACTION: f32 = 0.25;

/// system のフォーカス表示のトークン (`md.sys.state.focus-indicator`)
///
/// [`WIDTH`] などのコンポーネントのトークンとは別の版で、内側の距離が異なる
pub mod indicator {
    use crate::Dp;

    /// `md.sys.state.focus-indicator.thickness`
    pub const THICKNESS: Dp = Dp(3.0);
    /// `md.sys.state.focus-indicator.outer-offset`
    pub const OUTER_OFFSET: Dp = Dp(2.0);
    /// `md.sys.state.focus-indicator.inner-offset`
    pub const INNER_OFFSET: Dp = Dp(-3.0);
}
