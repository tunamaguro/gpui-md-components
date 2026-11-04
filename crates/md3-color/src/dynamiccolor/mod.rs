//! 元の色から色の役割ごとの色を決める動的配色
//!
//! [`DynamicScheme`] が配色を表し、[`ColorRole`] ごとに色を求める。仕様の版は
//! material-color-utilities の 2021 / 2025 / 2026 仕様に対応する

mod calc;
mod contrast_curve;
mod palettes;
mod role;
mod scheme;
mod spec;
mod spec_2021;
mod spec_2025;
mod spec_2026;
mod tone_delta_pair;
mod variant;

pub use role::ColorRole;
pub use scheme::{DynamicScheme, DynamicSchemeBuilder, SchemeError};
pub use variant::{Platform, SpecVersion, Variant};
