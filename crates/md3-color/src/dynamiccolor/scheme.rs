// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/dynamic_scheme.ts (DynamicScheme)
//         typescript/scheme/scheme_content.ts (SchemeContent)
//         typescript/scheme/scheme_cmf.ts (SchemeCmf)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// scheme_content.ts: Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// scheme_cmf.ts: Copyright 2026 Google LLC. Licensed under the Apache License, Version 2.0.

use std::fmt;

use super::spec::spec;
use super::{ColorRole, Platform, SpecVersion, Variant, calc, palettes};
use crate::hct::Hct;
use crate::palettes::TonalPalette;
use crate::units::{Argb, ContrastLevel};
use crate::utils::math;

/// 配色が参照するパレットの種類
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Palette {
    Primary,
    Secondary,
    Tertiary,
    Neutral,
    NeutralVariant,
    Error,
}

/// 元の色・配色の種類・明暗・コントラストから、すべての色の役割の色を決める配色
///
/// [`DynamicScheme::builder`] で作る。色は [`DynamicScheme::argb`] などで役割ごとに求める。
/// 計算結果はキャッシュしないため、同じ役割を何度も使うときは呼び出し側で保持する
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicScheme {
    pub(crate) source_color_hct: Hct,
    /// 先頭は `source_color_hct`
    pub(crate) source_color_hcts: Vec<Hct>,
    pub(crate) variant: Variant,
    pub(crate) contrast_level: f64,
    pub(crate) is_dark: bool,
    pub(crate) platform: Platform,
    pub(crate) spec_version: SpecVersion,
    pub(crate) primary_palette: TonalPalette,
    pub(crate) secondary_palette: TonalPalette,
    pub(crate) tertiary_palette: TonalPalette,
    pub(crate) neutral_palette: TonalPalette,
    pub(crate) neutral_variant_palette: TonalPalette,
    pub(crate) error_palette: TonalPalette,
}

impl DynamicScheme {
    /// `source_color` を元の色とし、`variant` の規則で配色を作るビルダー
    ///
    /// 既定はライト、標準コントラスト、[`SpecVersion::Spec2026`]、[`Platform::Phone`]
    pub fn builder(source_color: Hct, variant: Variant) -> DynamicSchemeBuilder {
        DynamicSchemeBuilder {
            source_color_hct: source_color,
            additional_source_color_hcts: Vec::new(),
            variant,
            is_dark: false,
            contrast_level: ContrastLevel::STANDARD,
            spec_version: SpecVersion::default(),
            platform: Platform::default(),
            primary_palette: None,
            secondary_palette: None,
            tertiary_palette: None,
            neutral_palette: None,
            neutral_variant_palette: None,
            error_palette: None,
        }
    }

    /// 最初の元の色
    pub fn source_color_hct(&self) -> Hct {
        self.source_color_hct
    }

    /// 元の色の一覧。先頭は [`DynamicScheme::source_color_hct`] で、常に 1 色以上ある
    pub fn source_color_hcts(&self) -> &[Hct] {
        &self.source_color_hcts
    }

    /// 配色の種類
    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// コントラストレベル
    pub fn contrast_level(&self) -> ContrastLevel {
        ContrastLevel::new(self.contrast_level)
    }

    /// ダークなら `true`
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// 表示する機器の種類
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// 計算に使う仕様の版
    ///
    /// 指定した版に [`Variant`] が対応しないときは読み替えた版になる。
    /// [`Variant::Cmf`] は指定どおり、[`Variant::TonalSpot`]・[`Variant::Neutral`]・
    /// [`Variant::Vibrant`]・[`Variant::Expressive`] と、[`Platform::Watch`] での
    /// [`Variant::Content`] は 2026 仕様を 2025 仕様に、それ以外は常に 2021 仕様に読み替える
    pub fn spec_version(&self) -> SpecVersion {
        self.spec_version
    }

    /// 主色のパレット
    pub fn primary_palette(&self) -> &TonalPalette {
        &self.primary_palette
    }

    /// 第二色のパレット
    pub fn secondary_palette(&self) -> &TonalPalette {
        &self.secondary_palette
    }

    /// 第三色のパレット
    pub fn tertiary_palette(&self) -> &TonalPalette {
        &self.tertiary_palette
    }

    /// 中間色のパレット
    pub fn neutral_palette(&self) -> &TonalPalette {
        &self.neutral_palette
    }

    /// 中間色バリアントのパレット
    pub fn neutral_variant_palette(&self) -> &TonalPalette {
        &self.neutral_variant_palette
    }

    /// エラー色のパレット
    pub fn error_palette(&self) -> &TonalPalette {
        &self.error_palette
    }

    /// `role` の色
    pub fn argb(&self, role: ColorRole) -> Argb {
        self.hct(role).to_argb()
    }

    /// `role` の色を HCT で返す
    pub fn hct(&self, role: ColorRole) -> Hct {
        calc::get_hct(self, role)
    }

    /// この配色で `role` が背景に対して目標とするコントラスト比
    ///
    /// 移植元の `DynamicColor` の `background` / `secondBackground` と `contrastCurve` を、
    /// この配色のコントラストレベルで評価したもの。背景かコントラストの目標を持たない役割では空になる。
    /// 目標は配色の計算が満たそうとする値で、トーンの範囲や役割どうしのトーン差の制約によって、
    /// 計算結果の色が目標に届かない場合がある
    pub fn contrast_requirements(
        &self,
        role: ColorRole,
    ) -> impl Iterator<Item = ContrastRequirement> + use<> {
        let color = spec(role, self.spec_version);
        let ratio = (color.contrast_curve)(self).map(|curve| curve.get(self.contrast_level));
        let backgrounds = [(color.background)(self), (color.second_background)(self)];
        backgrounds
            .into_iter()
            .flatten()
            .filter_map(move |background| {
                Some(ContrastRequirement {
                    background,
                    ratio: ratio?,
                })
            })
    }

    pub(crate) fn palette(&self, palette: Palette) -> &TonalPalette {
        match palette {
            Palette::Primary => &self.primary_palette,
            Palette::Secondary => &self.secondary_palette,
            Palette::Tertiary => &self.tertiary_palette,
            Palette::Neutral => &self.neutral_palette,
            Palette::NeutralVariant => &self.neutral_variant_palette,
            Palette::Error => &self.error_palette,
        }
    }

    /// 移植元の `Object.assign({}, s, {isDark: false, contrastLevel: 0})`。パレットは元の配色のものを使う
    pub(crate) fn light_standard_contrast(&self) -> Self {
        Self {
            is_dark: false,
            contrast_level: 0.0,
            ..self.clone()
        }
    }
}

/// 色の役割が背景に対して目標とするコントラスト比。[`DynamicScheme::contrast_requirements`] が返す
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastRequirement {
    /// 背景の役割
    pub background: ColorRole,
    /// 目標のコントラスト比 (1〜21)
    pub ratio: f64,
}

/// [`DynamicScheme`] のビルダー
#[derive(Clone, Debug)]
pub struct DynamicSchemeBuilder {
    source_color_hct: Hct,
    additional_source_color_hcts: Vec<Hct>,
    variant: Variant,
    is_dark: bool,
    contrast_level: ContrastLevel,
    spec_version: SpecVersion,
    platform: Platform,
    primary_palette: Option<TonalPalette>,
    secondary_palette: Option<TonalPalette>,
    tertiary_palette: Option<TonalPalette>,
    neutral_palette: Option<TonalPalette>,
    neutral_variant_palette: Option<TonalPalette>,
    error_palette: Option<TonalPalette>,
}

impl DynamicSchemeBuilder {
    /// ダークにするか
    pub fn dark(mut self, is_dark: bool) -> Self {
        self.is_dark = is_dark;
        self
    }

    /// コントラストレベル
    pub fn contrast_level(mut self, contrast_level: ContrastLevel) -> Self {
        self.contrast_level = contrast_level;
        self
    }

    /// 仕様の版。[`Variant`] によっては読み替える ([`DynamicScheme::spec_version`])
    pub fn spec_version(mut self, spec_version: SpecVersion) -> Self {
        self.spec_version = spec_version;
        self
    }

    /// 表示する機器の種類
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// 2 色目以降の元の色を設定する。既に設定した 2 色目以降は置き換える
    ///
    /// 2 色目を使うのは [`Variant::Cmf`] と、[`Platform::Watch`] での [`Variant::Content`] だけ
    pub fn additional_source_colors(mut self, colors: impl IntoIterator<Item = Hct>) -> Self {
        self.additional_source_color_hcts = colors.into_iter().collect();
        self
    }

    /// 主色のパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn primary_palette(mut self, palette: TonalPalette) -> Self {
        self.primary_palette = Some(palette);
        self
    }

    /// 第二色のパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn secondary_palette(mut self, palette: TonalPalette) -> Self {
        self.secondary_palette = Some(palette);
        self
    }

    /// 第三色のパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn tertiary_palette(mut self, palette: TonalPalette) -> Self {
        self.tertiary_palette = Some(palette);
        self
    }

    /// 中間色のパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn neutral_palette(mut self, palette: TonalPalette) -> Self {
        self.neutral_palette = Some(palette);
        self
    }

    /// 中間色バリアントのパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn neutral_variant_palette(mut self, palette: TonalPalette) -> Self {
        self.neutral_variant_palette = Some(palette);
        self
    }

    /// エラー色のパレットを指定する。指定しなければ [`Variant`] の規則で決まる
    pub fn error_palette(mut self, palette: TonalPalette) -> Self {
        self.error_palette = Some(palette);
        self
    }

    /// 配色を作る
    ///
    /// 結果は移植元で [`Variant`] に対応する `Scheme*` クラスを使ったときと同じになる。
    /// [`Variant::Cmf`] に 2026 仕様以外を指定したときは [`SchemeError::CmfRequiresSpec2026`]
    pub fn build(self) -> Result<DynamicScheme, SchemeError> {
        let Self {
            source_color_hct: source,
            additional_source_color_hcts,
            variant,
            is_dark,
            contrast_level,
            spec_version,
            platform,
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette,
        } = self;
        let second_source = additional_source_color_hcts.first().copied();
        let contrast_level = contrast_level.value();
        let spec_version = maybe_fallback_spec_version(spec_version, variant, platform);

        let palettes = match variant {
            Variant::Cmf => {
                if spec_version != SpecVersion::Spec2026 {
                    return Err(SchemeError::CmfRequiresSpec2026);
                }
                cmf_palettes(source, second_source)
            }
            _ => {
                let mut palettes =
                    palettes::for_variant(spec_version, variant, &source, is_dark, platform);
                // SchemeContent は腕時計で元の色が 2 色以上あるとき、2 色目から第三色を作る
                if variant == Variant::Content
                    && platform == Platform::Watch
                    && let Some(second) = second_source
                    && source.argb_raw() != second.argb_raw()
                {
                    palettes.tertiary = TonalPalette::from_hue_and_chroma_raw(
                        second.hue_raw(),
                        second.chroma_raw(),
                    );
                }
                palettes
            }
        };

        let mut source_color_hcts = Vec::with_capacity(1 + additional_source_color_hcts.len());
        source_color_hcts.push(source);
        source_color_hcts.extend(additional_source_color_hcts);
        Ok(DynamicScheme {
            source_color_hct: source,
            source_color_hcts,
            variant,
            contrast_level,
            is_dark,
            platform,
            spec_version,
            primary_palette: primary_palette.unwrap_or(palettes.primary),
            secondary_palette: secondary_palette.unwrap_or(palettes.secondary),
            tertiary_palette: tertiary_palette.unwrap_or(palettes.tertiary),
            neutral_palette: neutral_palette.unwrap_or(palettes.neutral),
            neutral_variant_palette: neutral_variant_palette.unwrap_or(palettes.neutral_variant),
            error_palette: error_palette.unwrap_or(palettes.error),
        })
    }
}

/// [`DynamicSchemeBuilder::build`] の失敗
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemeError {
    /// [`Variant::Cmf`] に 2026 仕様以外を指定した
    CmfRequiresSpec2026,
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CmfRequiresSpec2026 => f.write_str("the CMF variant requires spec version 2026"),
        }
    }
}

impl std::error::Error for SchemeError {}

/// 配色の 6 つのパレット
pub(crate) struct Palettes {
    pub(crate) primary: TonalPalette,
    pub(crate) secondary: TonalPalette,
    pub(crate) tertiary: TonalPalette,
    pub(crate) neutral: TonalPalette,
    pub(crate) neutral_variant: TonalPalette,
    pub(crate) error: TonalPalette,
}

fn maybe_fallback_spec_version(
    spec_version: SpecVersion,
    variant: Variant,
    platform: Platform,
) -> SpecVersion {
    match variant {
        Variant::Cmf => spec_version,
        Variant::Expressive | Variant::Vibrant | Variant::TonalSpot | Variant::Neutral => {
            spec_version.min(SpecVersion::Spec2025)
        }
        Variant::Content if platform == Platform::Watch => spec_version.min(SpecVersion::Spec2025),
        _ => SpecVersion::Spec2021,
    }
}

/// `SchemeCmf` のパレット
fn cmf_palettes(source: Hct, second_source: Option<Hct>) -> Palettes {
    let secondary_source = second_source.unwrap_or(source);
    let hue = source.hue_raw();
    let chroma = source.chroma_raw();
    let palette = TonalPalette::from_hue_and_chroma_raw;
    Palettes {
        primary: palette(hue, chroma),
        secondary: palette(hue, chroma * 0.5),
        tertiary: if source.argb_raw() == secondary_source.argb_raw() {
            palette(hue, chroma * 0.75)
        } else {
            palette(secondary_source.hue_raw(), secondary_source.chroma_raw())
        },
        neutral: palette(hue, chroma * 0.2),
        neutral_variant: palette(hue, chroma * 0.2),
        error: palette(
            cmf_error_hue(hue, secondary_source.hue_raw()),
            math::max_js(chroma, 50.0),
        ),
    }
}

fn cmf_error_hue(primary_hue: f64, tertiary_hue: f64) -> f64 {
    if primary_hue <= 8.0 {
        if tertiary_hue <= 24.0 {
            28.0
        } else if tertiary_hue <= 32.0 {
            16.0
        } else {
            20.0
        }
    } else if primary_hue <= 16.0 {
        if tertiary_hue <= 24.0 {
            32.0
        } else if tertiary_hue <= 32.0 {
            20.0
        } else {
            24.0
        }
    } else if primary_hue <= 20.0 {
        if tertiary_hue <= 28.0 {
            32.0
        } else if tertiary_hue <= 32.0 {
            24.0
        } else {
            28.0
        }
    } else if primary_hue <= 28.0 {
        if tertiary_hue <= 24.0 { 32.0 } else { 16.0 }
    } else if primary_hue <= 32.0 {
        if tertiary_hue <= 20.0 {
            24.0
        } else if tertiary_hue <= 28.0 {
            16.0
        } else {
            20.0
        }
    } else if primary_hue <= 40.0 {
        if tertiary_hue > 20.0 && tertiary_hue <= 28.0 {
            16.0
        } else {
            24.0
        }
    } else if primary_hue <= 152.0 {
        if tertiary_hue > 24.0 && tertiary_hue <= 36.0 {
            20.0
        } else {
            32.0
        }
    } else if primary_hue <= 272.0 {
        if tertiary_hue > 20.0 && tertiary_hue <= 28.0 {
            16.0
        } else {
            24.0
        }
    } else if tertiary_hue > 12.0 && tertiary_hue <= 28.0 {
        32.0
    } else {
        16.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirements(
        version: SpecVersion,
        contrast_level: ContrastLevel,
        role: ColorRole,
    ) -> Result<Vec<ContrastRequirement>, SchemeError> {
        let scheme =
            DynamicScheme::builder(Hct::from_argb(Argb::new(0xff67_50a4)), Variant::TonalSpot)
                .spec_version(version)
                .contrast_level(contrast_level)
                .build()?;
        Ok(scheme.contrast_requirements(role).collect())
    }

    fn on_primary(ratio: f64) -> Vec<ContrastRequirement> {
        vec![ContrastRequirement {
            background: ColorRole::Primary,
            ratio,
        }]
    }

    #[test]
    fn コントラストの目標は版とコントラストレベルで変わる() -> Result<(), SchemeError> {
        use SpecVersion::{Spec2021, Spec2026};
        // color_spec_2026.ts の onPrimary は background が primary、contrastCurve が getCurve(6) = (6, 6, 7, 11)
        let standard = requirements(Spec2026, ContrastLevel::STANDARD, ColorRole::OnPrimary)?;
        assert_eq!(standard, on_primary(6.0));
        let high = requirements(Spec2026, ContrastLevel::HIGH, ColorRole::OnPrimary)?;
        assert_eq!(high, on_primary(11.0));
        // color_spec_2021.ts の onPrimary は contrastCurve が (4.5, 7, 11, 21)
        let spec_2021 = requirements(Spec2021, ContrastLevel::STANDARD, ColorRole::OnPrimary)?;
        assert_eq!(spec_2021, on_primary(7.0));
        Ok(())
    }

    #[test]
    fn 背景を持たない役割は目標を持たない() -> Result<(), SchemeError> {
        let surface = requirements(
            SpecVersion::Spec2026,
            ContrastLevel::STANDARD,
            ColorRole::Surface,
        )?;
        assert!(surface.is_empty());
        Ok(())
    }
}
