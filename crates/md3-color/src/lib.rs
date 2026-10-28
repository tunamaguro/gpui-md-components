//! Material Design 3 の仕様に基づく配色エンジン。
//!
//! HCT / CAM16 色空間、`TonalPalette`、`DynamicScheme` を純 Rust で提供する。
//! material-color-utilities（TypeScript 版）の 2021 仕様（MD3 ベースライン）を移植したもの。
//! gpui に依存せず、panic しない。

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod hct;
mod units;
mod utils;

pub use hct::{Cam16, Hct, ViewingConditions};
pub use units::{Argb, Chroma, Hue, Tone};
pub mod contrast;
pub mod dislike;
pub mod palettes;
pub mod scheme;
pub mod temperature;

pub use palettes::{CorePalette, TonalPalette};
pub use scheme::{ColorRole, ContrastCurve, ContrastLevel, DynamicScheme, Variant};
pub use temperature::TemperatureCache;
