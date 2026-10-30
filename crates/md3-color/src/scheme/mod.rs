// Ported from material-color-utilities: typescript/dynamiccolor/dynamic_scheme.ts (DynamicScheme, DynamicSchemePalettesDelegateImpl2021) @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3. See NOTICE.

//! 元の色から全カラーロールの色を決める配色（2021 仕様）。

mod contrast_curve;
mod role;
mod tone;
mod variant;

pub use contrast_curve::ContrastCurve;
pub use role::ColorRole;
pub use variant::Variant;

use crate::dislike;
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::temperature::TemperatureCache;
use crate::units::{Argb, Hue, Tone};
use crate::utils::math;

/// コントラストレベル。-1（低）〜 1（高）。0 が標準。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ContrastLevel(f64);

impl ContrastLevel {
    /// 低コントラスト（-1）。
    pub const REDUCED: Self = Self(-1.0);
    /// 標準（0）。
    pub const STANDARD: Self = Self(0.0);
    /// 中コントラスト（0.5）。
    pub const MEDIUM: Self = Self(0.5);
    /// 高コントラスト（1）。
    pub const HIGH: Self = Self(1.0);

    /// 値から作る。`[-1, 1]` に丸め、NaN は 0 にする。
    pub fn new(level: f64) -> Self {
        if level.is_nan() {
            Self::STANDARD
        } else {
            Self(math::clamp_double(-1.0, 1.0, level))
        }
    }

    /// 値。
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl Default for ContrastLevel {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// 元の色・配色の種類・明暗・コントラストから決まる配色。
///
/// 各役割の色は [`DynamicScheme::argb`] などで求める。キャッシュは持たず、
/// 呼ぶたびに計算する。
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicScheme {
    pub(crate) source_color_hct: Hct,
    pub(crate) variant: Variant,
    pub(crate) is_dark: bool,
    pub(crate) contrast_level: f64,
    pub(crate) primary_palette: TonalPalette,
    pub(crate) secondary_palette: TonalPalette,
    pub(crate) tertiary_palette: TonalPalette,
    pub(crate) neutral_palette: TonalPalette,
    pub(crate) neutral_variant_palette: TonalPalette,
    pub(crate) error_palette: TonalPalette,
}

impl DynamicScheme {
    /// 配色を作る。
    pub fn new(
        source_color: Hct,
        variant: Variant,
        is_dark: bool,
        contrast_level: ContrastLevel,
    ) -> Self {
        let s = source_color;
        let hue = s.hue().degrees();
        let chroma = s.chroma().value();
        let palette = TonalPalette::from_hue_and_chroma_raw;
        let (primary, secondary, tertiary, neutral, neutral_variant) = match variant {
            Variant::Content => (
                palette(hue, chroma),
                palette(hue, f64::max(chroma - 32.0, chroma * 0.5)),
                TonalPalette::from_hct(dislike::fix_if_disliked(analogous_tertiary(s))),
                palette(hue, chroma / 8.0),
                palette(hue, (chroma / 8.0) + 4.0),
            ),
            Variant::Fidelity => (
                palette(hue, chroma),
                palette(hue, f64::max(chroma - 32.0, chroma * 0.5)),
                TonalPalette::from_hct(dislike::fix_if_disliked(
                    TemperatureCache::new(s).complement(),
                )),
                palette(hue, chroma / 8.0),
                palette(hue, (chroma / 8.0) + 4.0),
            ),
            Variant::Monochrome => (
                palette(hue, 0.0),
                palette(hue, 0.0),
                palette(hue, 0.0),
                palette(hue, 0.0),
                palette(hue, 0.0),
            ),
            Variant::Neutral => (
                palette(hue, 12.0),
                palette(hue, 8.0),
                palette(hue, 16.0),
                palette(hue, 2.0),
                palette(hue, 2.0),
            ),
            Variant::TonalSpot => (
                palette(hue, 36.0),
                palette(hue, 16.0),
                palette(math::sanitize_degrees_double(hue + 60.0), 24.0),
                palette(hue, 6.0),
                palette(hue, 8.0),
            ),
            Variant::Expressive => (
                palette(math::sanitize_degrees_double(hue + 240.0), 40.0),
                palette(
                    rotated_hue_raw(
                        &s,
                        &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                        &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0],
                    ),
                    24.0,
                ),
                palette(
                    rotated_hue_raw(
                        &s,
                        &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                        &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0],
                    ),
                    32.0,
                ),
                palette(math::sanitize_degrees_double(hue + 15.0), 8.0),
                palette(math::sanitize_degrees_double(hue + 15.0), 12.0),
            ),
            Variant::Vibrant => (
                palette(hue, 200.0),
                palette(
                    rotated_hue_raw(
                        &s,
                        &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                        &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0],
                    ),
                    24.0,
                ),
                palette(
                    rotated_hue_raw(
                        &s,
                        &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                        &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0],
                    ),
                    32.0,
                ),
                palette(hue, 10.0),
                palette(hue, 12.0),
            ),
        };
        Self {
            source_color_hct: s,
            variant,
            is_dark,
            contrast_level: contrast_level.value(),
            primary_palette: primary,
            secondary_palette: secondary,
            tertiary_palette: tertiary,
            neutral_palette: neutral,
            neutral_variant_palette: neutral_variant,
            error_palette: palette(25.0, 84.0),
        }
    }

    /// 元の色。
    pub fn source_color(&self) -> Hct {
        self.source_color_hct
    }

    /// 配色の種類。
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// ダークテーマか。
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// コントラストレベル。
    pub fn contrast_level(&self) -> ContrastLevel {
        ContrastLevel(self.contrast_level)
    }

    /// primary のパレット。
    pub fn primary_palette(&self) -> &TonalPalette {
        &self.primary_palette
    }

    /// secondary のパレット。
    pub fn secondary_palette(&self) -> &TonalPalette {
        &self.secondary_palette
    }

    /// tertiary のパレット。
    pub fn tertiary_palette(&self) -> &TonalPalette {
        &self.tertiary_palette
    }

    /// neutral のパレット。
    pub fn neutral_palette(&self) -> &TonalPalette {
        &self.neutral_palette
    }

    /// neutral variant のパレット。
    pub fn neutral_variant_palette(&self) -> &TonalPalette {
        &self.neutral_variant_palette
    }

    /// error のパレット。
    pub fn error_palette(&self) -> &TonalPalette {
        &self.error_palette
    }

    /// 役割の色。
    pub fn argb(&self, role: ColorRole) -> Argb {
        self.hct(role).to_argb()
    }

    /// 役割の色を HCT で返す。
    pub fn hct(&self, role: ColorRole) -> Hct {
        tone::get_hct(self, role)
    }

    /// 役割のトーン（コントラスト制約を適用した後の値）。
    pub fn tone(&self, role: ColorRole) -> Tone {
        Tone::new(tone::get_tone(self, role))
    }
}

fn analogous_tertiary(source: Hct) -> Hct {
    TemperatureCache::new(source)
        .analogous(3, 6)
        .get(2)
        .copied()
        .unwrap_or(source)
}

/// 元の色の色相が属する区間に応じた値を返す。どの区間にも入らなければ元の色相。
fn piecewise_hue(source_color_hct: &Hct, hue_breakpoints: &[f64], hues: &[f64]) -> f64 {
    let size = usize::min(hue_breakpoints.len().saturating_sub(1), hues.len());
    let source_hue = source_color_hct.hue().degrees();
    for i in 0..size {
        let (Some(&start), Some(&end), Some(&hue)) = (
            hue_breakpoints.get(i),
            hue_breakpoints.get(i + 1),
            hues.get(i),
        ) else {
            break;
        };
        if source_hue >= start && source_hue < end {
            return math::sanitize_degrees_double(hue);
        }
    }
    source_hue
}

fn rotated_hue_raw(source_color_hct: &Hct, hue_breakpoints: &[f64], rotations: &[f64]) -> f64 {
    let size = usize::min(hue_breakpoints.len().saturating_sub(1), rotations.len());
    let rotation = if size == 0 {
        0.0
    } else {
        piecewise_hue(source_color_hct, hue_breakpoints, rotations)
    };
    math::sanitize_degrees_double(source_color_hct.hue().degrees() + rotation)
}

/// 元の色の色相を、区間ごとの回転量だけ回した色相。
///
/// `hue_breakpoints[i]` 以上 `hue_breakpoints[i + 1]` 未満なら `rotations[i]` 度回す。
pub fn rotated_hue(source_color: &Hct, hue_breakpoints: &[f64], rotations: &[f64]) -> Hue {
    Hue::new(rotated_hue_raw(source_color, hue_breakpoints, rotations))
}
