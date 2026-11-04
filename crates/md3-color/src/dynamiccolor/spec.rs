//! 色の役割の定義と、仕様の版による切り替え
//!
//! 移植元の `DynamicColor` は関数を値に持つオブジェクトで、`ColorSpecDelegateImpl2025` /
//! `ColorSpecDelegateImpl2026` は `extendSpecVersion` で古い版の定義を包み、配色の版に応じて
//! 新しい版の定義に切り替える。このモジュールは役割と版から定義 ([`ColorSpec`]) を選ぶ形で、
//! 同じ切り替えを表す

use super::contrast_curve::ContrastCurve;
use super::scheme::Palette;
use super::tone_delta_pair::ToneDeltaPair;
use super::{ColorRole, DynamicScheme, SpecVersion, spec_2021, spec_2025, spec_2026};

/// ある版での 1 つの色の役割の定義 (移植元の `DynamicColor` のフィールド)
///
/// 関数はすべて配色を受け取り、その配色での値を返す
#[derive(Clone, Copy)]
pub(crate) struct ColorSpec {
    pub(crate) palette: fn(&DynamicScheme) -> Palette,
    /// `None` のときは背景のトーン (背景もなければ 50) を使う
    pub(crate) tone: Option<fn(&DynamicScheme) -> f64>,
    pub(crate) is_background: bool,
    pub(crate) chroma_multiplier: fn(&DynamicScheme) -> f64,
    pub(crate) background: fn(&DynamicScheme) -> Option<ColorRole>,
    pub(crate) second_background: fn(&DynamicScheme) -> Option<ColorRole>,
    pub(crate) contrast_curve: fn(&DynamicScheme) -> Option<ContrastCurve>,
    pub(crate) tone_delta_pair: fn(&DynamicScheme) -> Option<ToneDeltaPair>,
}

impl ColorSpec {
    /// 移植元の `DynamicColor.fromPalette` の省略時の値
    pub(crate) fn new(palette: fn(&DynamicScheme) -> Palette) -> Self {
        Self {
            palette,
            tone: None,
            is_background: false,
            chroma_multiplier: |_| 1.0,
            background: |_| None,
            second_background: |_| None,
            contrast_curve: |_| None,
            tone_delta_pair: |_| None,
        }
    }
}

/// `version` の配色で `role` に使う定義
///
/// 2026 仕様の定義があればそれを、なければ 2025 仕様、2021 仕様の順に探す
pub(crate) fn spec(role: ColorRole, version: SpecVersion) -> ColorSpec {
    if version >= SpecVersion::Spec2026
        && let Some(spec) = spec_2026::spec(role, version)
    {
        return spec;
    }
    if version >= SpecVersion::Spec2025
        && let Some(spec) = spec_2025::spec(role, version)
    {
        return spec;
    }
    spec_2021::spec(role)
}
