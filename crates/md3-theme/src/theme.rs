//! 解決済みのテーマと gpui 型への変換

use std::collections::BTreeMap;

use gpui::{
    BoxShadow, Corners, FontWeight, Hsla, Pixels, Rgba, Size, SpringConfig, TextStyleRefinement,
    point, px,
};
use md3_color::{Argb, ColorRole, CustomColorGroup, DynamicScheme, Hct, SchemeError, contrast};
use md3_tokens::motion::MotionSchemeKey;
use md3_tokens::shadow;
use md3_tokens::shape::{CornerSize, ShapeKey};
use md3_tokens::typography::TypeScale;
use md3_tokens::{Dp, Sp};

use crate::settings::{Brightness, ThemeSettings};

/// 色を gpui の色に変換する
pub(crate) fn to_hsla(argb: Argb) -> Hsla {
    Rgba {
        r: f32::from(argb.red()) / 255.0,
        g: f32::from(argb.green()) / 255.0,
        b: f32::from(argb.blue()) / 255.0,
        a: f32::from(argb.alpha()) / 255.0,
    }
    .into()
}

/// dp の寸法を gpui の論理ピクセルにする
///
/// 1dp は 1 論理ピクセルで、ウィンドウのスケールは gpui が掛ける。密度は掛けない
pub fn dp(value: Dp) -> Pixels {
    px(value.0)
}

/// 配色の目標に届かないコントラスト
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastIssue {
    /// 前景のロール
    pub foreground: ColorRole,
    /// 背景のロール
    pub background: ColorRole,
    /// テーマの色でのコントラスト比
    pub ratio: f64,
    /// 配色の目標のコントラスト比
    pub required: f64,
}

/// アプリ独自の色の 4 つの役割。トーンの決まり方は [`CustomColorGroup`] と同じ
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomColorRoles {
    /// 色
    pub color: Hsla,
    /// 色の上の文字・アイコン
    pub on_color: Hsla,
    /// コンテナ
    pub container: Hsla,
    /// コンテナの上の文字・アイコン
    pub on_container: Hsla,
}

/// 設定と OS の外観から解決したテーマ
///
/// 作った後は変わらない。設定は [`ThemeExt::update_md3_theme`](crate::ThemeExt::update_md3_theme) で変え、テーマはそのたびに作り直される
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    settings: ThemeSettings,
    brightness: Brightness,
    colors: [Argb; ColorRole::ALL.len()],
    custom_colors: BTreeMap<&'static str, CustomColorRoles>,
    contrast_issues: Vec<ContrastIssue>,
}

impl Theme {
    /// OS の外観が `system` のときの、`settings` のテーマ
    ///
    /// 配色の設定から配色を作れないとき (2026 仕様以外の [`Variant::Cmf`](md3_color::Variant::Cmf)) は
    /// その [`SchemeError`]
    pub(crate) fn resolve(
        settings: ThemeSettings,
        system: Brightness,
    ) -> Result<Self, SchemeError> {
        let brightness = settings.appearance.brightness(system);
        let color = &settings.color;
        let mut builder = DynamicScheme::builder(Hct::from_argb(color.source), color.variant)
            .dark(brightness == Brightness::Dark)
            .contrast_level(color.contrast)
            .spec_version(color.spec_version)
            .additional_source_colors(color.additional_sources.iter().copied().map(Hct::from_argb));
        let palettes = &color.palettes;
        if let Some(palette) = palettes.primary {
            builder = builder.primary_palette(palette);
        }
        if let Some(palette) = palettes.secondary {
            builder = builder.secondary_palette(palette);
        }
        if let Some(palette) = palettes.tertiary {
            builder = builder.tertiary_palette(palette);
        }
        if let Some(palette) = palettes.neutral {
            builder = builder.neutral_palette(palette);
        }
        if let Some(palette) = palettes.neutral_variant {
            builder = builder.neutral_variant_palette(palette);
        }
        if let Some(palette) = palettes.error {
            builder = builder.error_palette(palette);
        }
        let scheme = builder.build()?;

        let generated = ColorRole::ALL.map(|role| scheme.argb(role));
        let overrides = color.overrides(brightness);
        let colors =
            ColorRole::ALL.map(|role| overrides.get(role).unwrap_or(generated[role.index()]));

        // 比は不透明度を無視したトーンで求める。ロールの色は不透明なので結果は変わらない
        let contrast_issues = ColorRole::ALL
            .into_iter()
            .flat_map(|foreground| {
                scheme
                    .contrast_requirements(foreground)
                    .map(move |requirement| (foreground, requirement))
            })
            .filter_map(|(foreground, requirement)| {
                let background = requirement.background;
                let ratio = contrast::ratio_of_tones(
                    colors[foreground.index()].lstar(),
                    colors[background.index()].lstar(),
                );
                (ratio < requirement.ratio).then_some(ContrastIssue {
                    foreground,
                    background,
                    ratio,
                    required: requirement.ratio,
                })
            })
            .collect();

        let custom_colors = color
            .custom_colors
            .iter()
            .map(|(key, custom)| {
                let group = CustomColorGroup::new(
                    color.source,
                    custom.color,
                    custom.harmonize,
                    brightness == Brightness::Dark,
                );
                let roles = CustomColorRoles {
                    color: to_hsla(group.color),
                    on_color: to_hsla(group.on_color),
                    container: to_hsla(group.container),
                    on_container: to_hsla(group.on_container),
                };
                (*key, roles)
            })
            .collect();

        Ok(Self {
            settings,
            brightness,
            colors,
            custom_colors,
            contrast_issues,
        })
    }

    /// 解決に使った設定
    pub fn settings(&self) -> &ThemeSettings {
        &self.settings
    }

    /// 使っている明暗
    pub(crate) fn brightness(&self) -> Brightness {
        self.brightness
    }

    /// `role` の色
    pub fn color(&self, role: ColorRole) -> Hsla {
        to_hsla(self.colors[role.index()])
    }

    /// キーが `key` のアプリ独自の色の役割。設定の [`custom_colors`](crate::ColorSettings::custom_colors) にそのキーがなければ `None`
    pub fn custom_color(&self, key: &str) -> Option<CustomColorRoles> {
        self.custom_colors.get(key).copied()
    }

    /// テーマの色でコントラスト比が配色の目標に届かない前景と背景の組
    ///
    /// 設定した色は目標に届かなくてもそのまま使われ、ここで報告するだけにとどまる。
    /// 目標は配色の計算が満たそうとする値のため、差し替えのない配色でも高コントラストなどでは届かない組がある。
    /// 対象は今の明暗だけで、アプリ独自の色は含まない
    pub fn contrast_issues(&self) -> &[ContrastIssue] {
        &self.contrast_issues
    }

    /// sp の寸法に文字倍率を掛けて、gpui の論理ピクセルにする
    pub fn sp(&self, value: Sp) -> Pixels {
        px(value.0 * self.settings.text_scale.get())
    }

    /// `scale` の書体。フォント、文字倍率を掛けた大きさと行の高さ、太さを持つ
    ///
    /// gpui の文字の描画は字間を指定できないため、[`TypeStyle::tracking`](md3_tokens::typography::TypeStyle::tracking)
    /// は反映しない
    pub fn text_style(&self, scale: TypeScale) -> TextStyleRefinement {
        let style = scale.style();
        TextStyleRefinement {
            font_family: Some(self.settings.fonts.get(style.font).clone()),
            font_size: Some(self.sp(style.size).into()),
            line_height: Some(self.sp(style.line_height).into()),
            font_weight: Some(FontWeight(f32::from(style.weight.0))),
            ..TextStyleRefinement::default()
        }
    }

    /// 大きさが `size` の要素を `key` の形にするときの角の半径
    ///
    /// 割合の角は `size` の短辺から求める。start / end の角は、左から右へ書く向きとして左右に当てる
    pub fn corner_radii(&self, key: ShapeKey, size: Size<Pixels>) -> Corners<Pixels> {
        let shape = key.resolve(&self.settings.shapes);
        let radius = |corner: CornerSize| match corner {
            CornerSize::Dp(value) => dp(value),
            CornerSize::Percent(percent) => size.width.min(size.height) * (percent / 100.0),
        };
        Corners {
            top_left: radius(shape.top_start),
            top_right: radius(shape.top_end),
            bottom_right: radius(shape.bottom_end),
            bottom_left: radius(shape.bottom_start),
        }
    }

    /// 高さの段階 `level` の影。key shadow、ambient shadow の順
    ///
    /// `level` の扱いは [`shadow::shadow`] と同じ
    pub fn shadow(&self, level: f32) -> Vec<BoxShadow> {
        let elevation = shadow::shadow(level);
        let color = self.color(shadow::SHADOW_COLOR);
        [elevation.key, elevation.ambient]
            .into_iter()
            .map(|shadow| BoxShadow {
                color: color.opacity(shadow.opacity),
                offset: point(px(0.0), dp(shadow.y)),
                blur_radius: dp(shadow.blur),
                spread_radius: dp(shadow.spread),
                inset: false,
            })
            .collect()
    }

    /// 設定のモーションの方式で `key` に当たるバネ
    pub fn spring(&self, key: MotionSchemeKey) -> SpringConfig {
        let spring = self.settings.motion.spring(key);
        // Compose の spring は質量 1 で、減衰比 ζ と剛性 k から減衰係数 c = 2ζ√k になる
        SpringConfig::new(
            spring.stiffness,
            2.0 * spring.damping_ratio * spring.stiffness.sqrt(),
            1.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use gpui::size;
    use md3_color::{ContrastLevel, TonalPalette, Variant};
    use md3_tokens::motion::MotionScheme;
    use md3_tokens::shape::{CornerShape, Shapes};

    use super::*;
    use crate::settings::{AppearanceMode, ColorSettings, CustomColor, DEFAULT_SOURCE, TextScale};

    fn argb(theme: &Theme, role: ColorRole) -> Argb {
        theme.colors[role.index()]
    }

    fn resolve(settings: ThemeSettings) -> Theme {
        Theme::resolve(settings, Brightness::Light).expect("テストの設定は解決できる")
    }

    fn scheme(settings: &ColorSettings, dark: bool) -> DynamicScheme {
        DynamicScheme::builder(Hct::from_argb(settings.source), settings.variant)
            .dark(dark)
            .contrast_level(settings.contrast)
            .build()
            .expect("テストの配色は作れる")
    }

    #[test]
    fn 既定の設定はベースラインの元の色の動的配色になる() {
        let theme = resolve(ThemeSettings::default());
        let expected = scheme(&ColorSettings::default(), false);
        for role in ColorRole::ALL {
            assert_eq!(argb(&theme, role), expected.argb(role), "{role:?}");
        }
        assert!(theme.contrast_issues().is_empty());
    }

    #[test]
    fn 明暗は明示指定を_os_の外観より優先する() {
        let surface_on_dark_system = |appearance| {
            let settings = ThemeSettings {
                appearance,
                ..ThemeSettings::default()
            };
            let theme = Theme::resolve(settings, Brightness::Dark).expect("既定の配色は解決できる");
            argb(&theme, ColorRole::Surface)
        };
        let surface = |dark| scheme(&ColorSettings::default(), dark).argb(ColorRole::Surface);
        assert_eq!(
            surface_on_dark_system(AppearanceMode::System),
            surface(true)
        );
        assert_eq!(
            surface_on_dark_system(AppearanceMode::Light),
            surface(false)
        );
    }

    #[test]
    fn ロールの差し替えは指定した明暗にだけ効く() {
        let mut settings = ThemeSettings::default();
        let red = Argb::from_rgb(0xff, 0, 0);
        settings.color.light.set(ColorRole::Primary, red);
        let light = resolve(settings.clone());
        settings.appearance = AppearanceMode::Dark;
        let dark = resolve(settings);
        assert_eq!(argb(&light, ColorRole::Primary), red);
        assert_eq!(
            argb(&dark, ColorRole::Primary),
            scheme(&ColorSettings::default(), true).argb(ColorRole::Primary)
        );
        // 明暗を変えて作り直しても、ライトの差し替えは設定に残る
        assert_eq!(
            dark.settings().color.light.get(ColorRole::Primary),
            Some(red)
        );
    }

    #[test]
    fn パレットの差し替えは役割のトーンを動的配色の規則で決める() {
        let mut settings = ThemeSettings::default();
        let palette = TonalPalette::from_argb(Argb::from_rgb(0x00, 0x6a, 0x60));
        settings.color.palettes.primary = Some(palette);
        let theme = resolve(settings);
        let expected = DynamicScheme::builder(Hct::from_argb(DEFAULT_SOURCE), Variant::TonalSpot)
            .primary_palette(palette)
            .build()
            .expect("TonalSpot の配色は作れる");
        assert_eq!(
            argb(&theme, ColorRole::Primary),
            expected.argb(ColorRole::Primary)
        );
        assert_ne!(
            argb(&theme, ColorRole::Primary),
            scheme(&ColorSettings::default(), false).argb(ColorRole::Primary)
        );
    }

    #[test]
    fn 目標に届かない色も使ったうえで報告する() {
        let mut settings = ThemeSettings::default();
        settings.color.contrast = ContrastLevel::HIGH;
        let primary = argb(&resolve(settings.clone()), ColorRole::Primary);
        settings.color.light.set(ColorRole::OnPrimary, primary);
        let theme = resolve(settings);
        assert_eq!(argb(&theme, ColorRole::OnPrimary), primary);
        let issue = theme
            .contrast_issues()
            .iter()
            .find(|issue| {
                issue.foreground == ColorRole::OnPrimary && issue.background == ColorRole::Primary
            })
            .expect("前景を背景と同じ色にした組は報告される");
        assert_eq!(issue.ratio, 1.0);
        // color_spec_2026.ts の onPrimary の contrastCurve は getCurve(6) で、高コントラストでは 11
        assert_eq!(issue.required, 11.0);
    }

    #[test]
    fn 差し替えた色が目標を満たしていれば報告しない() {
        let mut settings = ThemeSettings::default();
        settings
            .color
            .light
            .set(ColorRole::OnPrimary, Argb::from_rgb(0xff, 0xff, 0xff));
        assert!(resolve(settings).contrast_issues().is_empty());
    }

    #[test]
    fn 二色目の元の色がない_cmf_は_2026_仕様以外で失敗する() {
        let mut settings = ThemeSettings::default();
        settings.color.variant = Variant::Cmf;
        settings.color.spec_version = md3_color::SpecVersion::Spec2021;
        assert_eq!(
            Theme::resolve(settings, Brightness::Light),
            Err(SchemeError::CmfRequiresSpec2026)
        );
    }

    #[test]
    fn 独自の色は今の明暗の役割になる() {
        let green = Argb::from_rgb(0x00, 0x80, 0x00);
        let mut settings = ThemeSettings {
            appearance: AppearanceMode::Dark,
            ..ThemeSettings::default()
        };
        let custom = CustomColor {
            color: green,
            harmonize: true,
        };
        settings.color.custom_colors.insert("success", custom);
        let roles = resolve(settings)
            .custom_color("success")
            .expect("加えた色は引ける");
        let group = CustomColorGroup::new(DEFAULT_SOURCE, green, true, true);
        assert_eq!(roles.color, to_hsla(group.color));
        assert_eq!(roles.on_color, to_hsla(group.on_color));
        assert_eq!(roles.container, to_hsla(group.container));
        assert_eq!(roles.on_container, to_hsla(group.on_container));
    }

    #[test]
    fn 加えていないキーの色は引けない() {
        assert_eq!(
            resolve(ThemeSettings::default()).custom_color("success"),
            None
        );
    }

    #[test]
    fn 差し替えた色は不透明になる() {
        let mut settings = ThemeSettings::default();
        settings
            .color
            .light
            .set(ColorRole::OnPrimary, Argb::new(0x00ff_ffff));
        let theme = resolve(settings);
        assert_eq!(argb(&theme, ColorRole::OnPrimary), Argb::new(0xffff_ffff));
    }

    #[test]
    fn 文字倍率は_sp_の寸法にだけ掛かる() {
        let settings = ThemeSettings {
            text_scale: TextScale::new(2.0).expect("2 は正の有限値"),
            ..ThemeSettings::default()
        };
        let theme = resolve(settings);
        let style = theme.text_style(TypeScale::BodyLarge);
        // TypeScaleTokens.kt の BodyLargeSize は 16sp、BodyLargeLineHeight は 24sp
        assert_eq!(style.font_size, Some(px(32.0).into()));
        assert_eq!(style.line_height, Some(px(48.0).into()));
        assert_eq!(dp(Dp(16.0)), px(16.0));
        assert_eq!(TextScale::new(0.0), None);
    }

    #[test]
    fn 角の半径は割合を短辺から求め_start_を左に当てる() {
        let large = CornerShape {
            top_start: CornerSize::Dp(Dp(4.0)),
            top_end: CornerSize::Percent(50.0),
            bottom_end: CornerSize::ZERO,
            bottom_start: CornerSize::Dp(Dp(8.0)),
        };
        let settings = ThemeSettings {
            shapes: Shapes {
                large,
                ..Shapes::default()
            },
            ..ThemeSettings::default()
        };
        let theme = resolve(settings);
        let bounds = size(px(100.0), px(40.0));
        assert_eq!(
            theme.corner_radii(ShapeKey::CornerLarge, bounds),
            Corners {
                top_left: px(4.0),
                top_right: px(20.0),
                bottom_right: px(0.0),
                bottom_left: px(8.0),
            }
        );
    }

    #[test]
    fn 影は影のロールの色に不透明度を掛ける() {
        let theme = resolve(ThemeSettings::default());
        let shadows = theme.shadow(1.0);
        let color = theme.color(ColorRole::Shadow);
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[0].color, color.opacity(shadow::KEY_OPACITY));
        assert_eq!(shadows[1].color, color.opacity(shadow::AMBIENT_OPACITY));
        // _elevation.scss の段階 1 は key が (y 1, blur 2)、ambient が (y 1, blur 3, spread 1)
        assert_eq!(shadows[0].offset, point(px(0.0), px(1.0)));
        assert_eq!(shadows[0].blur_radius, px(2.0));
        assert_eq!(shadows[1].spread_radius, px(1.0));
    }

    #[test]
    fn バネは減衰比と剛性を保つ() {
        let settings = ThemeSettings {
            motion: MotionScheme::Expressive,
            ..ThemeSettings::default()
        };
        let theme = resolve(settings);
        let key = MotionSchemeKey::DefaultSpatial;
        let expected = MotionScheme::Expressive.spring(key);
        let (_, damping_ratio) = theme.spring(key).canonical();
        assert!((damping_ratio - expected.damping_ratio).abs() < 1e-6);
        assert_eq!(theme.spring(key).stiffness, expected.stiffness);
    }
}
