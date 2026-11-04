//! Material Design 3 の仕様に基づく配色エンジン
//!
//! HCT 色空間、トーンのパレット、動的配色、コントラスト、アプリ独自の色の計算を提供する。
//! 計算は material-color-utilities の TypeScript 版を移植したもので、動的配色は 2026 仕様を既定とする

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod blend;
pub mod contrast;
mod custom_color;
pub mod dislike;
pub mod dynamiccolor;
pub mod hct;
pub mod palettes;
pub mod temperature;
mod units;
mod utils;

pub use custom_color::CustomColorGroup;
pub use dynamiccolor::{
    ColorRole, DynamicScheme, DynamicSchemeBuilder, Platform, SchemeError, SpecVersion, Variant,
};
pub use hct::{Cam16, Hct, ViewingConditions};
pub use palettes::TonalPalette;
pub use temperature::TemperatureCache;
pub use units::{Argb, Chroma, ContrastLevel, Hue, Tone};
