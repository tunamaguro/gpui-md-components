// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ProgressIndicatorTokens.kt @ b8256a52 (tokens VERSION v0_4_0). See NOTICE.

//! `ProgressIndicatorTokens` の転記。値の出典は `docs/TOKENS_SOURCE.md`。

use crate::shape::Shape;
use md3_color::ColorRole;

/// `ActiveIndicatorColor`
pub const ACTIVE_INDICATOR_COLOR: ColorRole = ColorRole::Primary;
/// `ActiveShape`
pub const ACTIVE_SHAPE: Shape = Shape::Full;
/// `StopColor`
pub const STOP_COLOR: ColorRole = ColorRole::Primary;
/// `StopShape`
pub const STOP_SHAPE: Shape = Shape::Full;
/// `TrackColor`
pub const TRACK_COLOR: ColorRole = ColorRole::SecondaryContainer;
/// `TrackShape`
pub const TRACK_SHAPE: Shape = Shape::Full;
