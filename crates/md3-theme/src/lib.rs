//! md3 のテーマ。
//!
//! [`Theme`]（gpui の `Global`）、`cx.md3()` で参照する拡張トレイト [`Md3ThemeExt`]、
//! gpui 型への変換、OS のライト / ダーク外観への追従を提供する。
//!
//! ```ignore
//! let theme = Theme::from_seed(Argb::from_rgb(0x67, 0x50, 0xA4))
//!     .variant(SchemeVariant::TonalSpot)
//!     .contrast(Contrast::Standard)
//!     .appearance(Appearance::System)
//!     .build();
//! cx.set_md3_theme(theme);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod contrast;
mod shape;
mod typography;

use std::sync::OnceLock;

use gpui::{
    App, BoxShadow, Global, Hsla, Pixels, Rgba, SharedString, Subscription, Window,
    WindowAppearance, px,
};
use md3_color::{ColorRole, ContrastLevel, DynamicScheme, Hct};
use md3_tokens::elevation::{self, Shadow};

pub use contrast::{ContrastIssue, ContrastKind};
pub use md3_color::{self, Argb, Variant as SchemeVariant};
pub use md3_tokens::{
    self, density::Density, elevation::Elevation, shape::Shape, typography::TypeScale,
};
pub use shape::{CornerRadius, Shapes};
pub use typography::{TypeScaleStyle, TypeScaleStyles};

/// 配色の既定のシード色（MD3 のベースライン紫 `#6750A4`）。
pub const DEFAULT_SEED: Argb = Argb::from_rgb(0x67, 0x50, 0xA4);

/// カラーロールの数。
const ROLE_COUNT: usize = ColorRole::ALL.len();

/// 配色のコントラストの段階。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Contrast {
    /// 標準（MD3 の既定）。
    #[default]
    Standard,
    /// 中。
    Medium,
    /// 高。
    High,
}

impl Contrast {
    /// すべての段階。
    pub const ALL: [Contrast; 3] = [Contrast::Standard, Contrast::Medium, Contrast::High];

    fn level(self) -> ContrastLevel {
        match self {
            Contrast::Standard => ContrastLevel::STANDARD,
            Contrast::Medium => ContrastLevel::MEDIUM,
            Contrast::High => ContrastLevel::HIGH,
        }
    }
}

/// ライト / ダークの選び方。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Appearance {
    /// OS の設定に従う。
    #[default]
    System,
    /// 常にライト。
    Light,
    /// 常にダーク。
    Dark,
}

impl Appearance {
    /// すべての選び方。
    pub const ALL: [Appearance; 3] = [Appearance::System, Appearance::Light, Appearance::Dark];
}

/// OS（gpui）の外観がダークかどうか。
pub fn is_dark_appearance(appearance: WindowAppearance) -> bool {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => false,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => true,
    }
}

/// ライトまたはダークの全カラーロールの色。
#[derive(Clone, Debug, PartialEq)]
struct ColorTable([Argb; ROLE_COUNT]);

impl ColorTable {
    fn new(seed: Hct, variant: SchemeVariant, is_dark: bool, contrast: Contrast) -> Self {
        let scheme = DynamicScheme::new(seed, variant, is_dark, contrast.level());
        Self(ColorRole::ALL.map(|role| scheme.argb(role)))
    }

    fn get(&self, role: ColorRole) -> Argb {
        self.0[role.index()]
    }

    fn set(&mut self, role: ColorRole, argb: Argb) {
        self.0[role.index()] = argb;
    }
}

/// `Argb` を gpui の色に変換する。
pub fn to_hsla(argb: Argb) -> Hsla {
    Rgba {
        r: f32::from(argb.red()) / 255.0,
        g: f32::from(argb.green()) / 255.0,
        b: f32::from(argb.blue()) / 255.0,
        a: f32::from(argb.alpha()) / 255.0,
    }
    .into()
}

/// 背景のロールの上に置く文字・アイコンのロール（`Primary` なら `OnPrimary`）。
///
/// 対になる前景が決まっていないロール（前景色そのものや影など）は `None`。
pub fn content_color(role: ColorRole) -> Option<ColorRole> {
    use ColorRole::*;
    match role {
        Background => Some(OnBackground),
        Surface
        | SurfaceDim
        | SurfaceBright
        | SurfaceContainerLowest
        | SurfaceContainerLow
        | SurfaceContainer
        | SurfaceContainerHigh
        | SurfaceContainerHighest => Some(OnSurface),
        SurfaceVariant => Some(OnSurfaceVariant),
        InverseSurface => Some(InverseOnSurface),
        Primary => Some(OnPrimary),
        PrimaryContainer => Some(OnPrimaryContainer),
        Secondary => Some(OnSecondary),
        SecondaryContainer => Some(OnSecondaryContainer),
        Tertiary => Some(OnTertiary),
        TertiaryContainer => Some(OnTertiaryContainer),
        Error => Some(OnError),
        ErrorContainer => Some(OnErrorContainer),
        PrimaryFixed | PrimaryFixedDim => Some(OnPrimaryFixed),
        SecondaryFixed | SecondaryFixedDim => Some(OnSecondaryFixed),
        TertiaryFixed | TertiaryFixedDim => Some(OnTertiaryFixed),
        PrimaryPaletteKeyColor
        | SecondaryPaletteKeyColor
        | TertiaryPaletteKeyColor
        | NeutralPaletteKeyColor
        | NeutralVariantPaletteKeyColor
        | ErrorPaletteKeyColor
        | OnBackground
        | OnSurface
        | OnSurfaceVariant
        | InverseOnSurface
        | Outline
        | OutlineVariant
        | Shadow
        | Scrim
        | SurfaceTint
        | OnPrimary
        | OnPrimaryContainer
        | InversePrimary
        | OnSecondary
        | OnSecondaryContainer
        | OnTertiary
        | OnTertiaryContainer
        | OnError
        | OnErrorContainer
        | OnPrimaryFixed
        | OnPrimaryFixedVariant
        | OnSecondaryFixed
        | OnSecondaryFixedVariant
        | OnTertiaryFixed
        | OnTertiaryFixedVariant => None,
    }
}

/// md3 のテーマ。配色（ライトとダークの両方）、文字、角丸、密度をまとめて持つ。
///
/// [`Theme::from_seed`] で [`ThemeBuilder`] を作って組み立てる。
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    seed: Argb,
    variant: SchemeVariant,
    contrast: Contrast,
    appearance: Appearance,
    overrides: Vec<(ColorRole, Argb)>,
    light: ColorTable,
    dark: ColorTable,
    is_dark: bool,
    font_family: Option<SharedString>,
    type_scale: TypeScaleStyles,
    density: Density,
    shapes: Shapes,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Theme::from_seed(DEFAULT_SEED).build()
    }
}

impl Theme {
    /// シード色からテーマの組み立てを始める。
    pub fn from_seed(seed: Argb) -> ThemeBuilder {
        ThemeBuilder::new(seed)
    }

    /// 現在の設定を引き継いだ組み立てを始める（一部だけ変えたいとき）。
    pub fn to_builder(&self) -> ThemeBuilder {
        ThemeBuilder {
            seed: self.seed,
            variant: self.variant,
            contrast: self.contrast,
            appearance: self.appearance,
            overrides: self.overrides.clone(),
            font_family: self.font_family.clone(),
            type_scale: self.type_scale.clone(),
            density: self.density,
            shapes: self.shapes,
            system_is_dark: self.is_dark,
        }
    }

    /// シード色。
    pub fn seed(&self) -> Argb {
        self.seed
    }

    /// スキームのバリアント。
    pub fn variant(&self) -> SchemeVariant {
        self.variant
    }

    /// コントラストの段階。
    pub fn contrast(&self) -> Contrast {
        self.contrast
    }

    /// ライト / ダークの選び方。
    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// 現在ダークの配色を使っているか。
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// 密度。
    pub fn density(&self) -> Density {
        self.density
    }

    /// 現在の配色でのカラーロールの色。
    pub fn argb(&self, role: ColorRole) -> Argb {
        self.table(self.is_dark).get(role)
    }

    /// 現在の配色でのカラーロールの色（gpui の色）。
    pub fn color(&self, role: ColorRole) -> Hsla {
        to_hsla(self.argb(role))
    }

    /// ライトまたはダークの配色でのカラーロールの色。
    pub fn argb_for(&self, role: ColorRole, is_dark: bool) -> Argb {
        self.table(is_dark).get(role)
    }

    fn table(&self, is_dark: bool) -> &ColorTable {
        if is_dark { &self.dark } else { &self.light }
    }

    /// フォントファミリ。`None` ならシステムフォント。
    pub fn font_family(&self) -> Option<&SharedString> {
        self.font_family.as_ref()
    }

    /// タイプスケール全体。
    pub fn type_scale(&self) -> &TypeScaleStyles {
        &self.type_scale
    }

    /// タイプスケールの 1 種の文字スタイル。フォントファミリはテーマのものを補う。
    pub fn text_style(&self, scale: TypeScale) -> TypeScaleStyle {
        let mut style = self.type_scale.get(scale).clone();
        if style.font_family.is_none() {
            style.font_family = self.font_family.clone();
        }
        style
    }

    /// 角丸スケール。
    pub fn shapes(&self) -> &Shapes {
        &self.shapes
    }

    /// 角丸の段階に対応する半径。
    pub fn corner(&self, shape: Shape) -> CornerRadius {
        self.shapes.corner(shape)
    }

    /// 密度を反映した寸法（既定の寸法から密度の段階 × 4dp を引く）。
    pub fn dense(&self, default: f32) -> Pixels {
        px(self.density.apply(default))
    }

    /// 高さ（elevation）の段階に対応する影。MD3 の key / ambient の 2 層。
    pub fn shadows(&self, level: Elevation) -> Vec<BoxShadow> {
        if level == Elevation::Level0 {
            return Vec::new();
        }
        let color = self.color(ColorRole::Shadow);
        let (key, ambient) = level.shadows();
        let layer = |shadow: Shadow, opacity: f32| {
            BoxShadow::new(px(shadow.x), px(shadow.y), color.opacity(opacity))
                .blur_radius(px(shadow.blur))
                .spread_radius(px(shadow.spread))
        };
        vec![
            layer(key, elevation::KEY_SHADOW_OPACITY),
            layer(ambient, elevation::AMBIENT_SHADOW_OPACITY),
        ]
    }

    /// ライトとダークの両方の配色で、WCAG のコントラスト比を下回るロールの組を返す。
    ///
    /// 生成した配色はトーン差で基準を満たすが、[`ThemeBuilder::override_color`] で崩れることがある。
    /// 開発時の警告用。
    pub fn check_contrast(&self) -> Vec<ContrastIssue> {
        [false, true]
            .into_iter()
            .flat_map(|is_dark| {
                let table = self.table(is_dark);
                contrast::check(is_dark, |role| table.get(role))
            })
            .collect()
    }
}

/// [`Theme`] の組み立て。
#[derive(Clone, Debug)]
pub struct ThemeBuilder {
    seed: Argb,
    variant: SchemeVariant,
    contrast: Contrast,
    appearance: Appearance,
    overrides: Vec<(ColorRole, Argb)>,
    font_family: Option<SharedString>,
    type_scale: TypeScaleStyles,
    density: Density,
    shapes: Shapes,
    system_is_dark: bool,
}

impl ThemeBuilder {
    fn new(seed: Argb) -> Self {
        Self {
            seed,
            variant: SchemeVariant::TonalSpot,
            contrast: Contrast::Standard,
            appearance: Appearance::System,
            overrides: Vec::new(),
            font_family: None,
            type_scale: TypeScaleStyles::default(),
            density: Density::Default,
            shapes: Shapes::default(),
            system_is_dark: false,
        }
    }

    /// シード色を変える。
    pub fn seed(mut self, seed: Argb) -> Self {
        self.seed = seed;
        self
    }

    /// スキームのバリアントを変える（既定は `TonalSpot`）。
    pub fn variant(mut self, variant: SchemeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// コントラストの段階を変える。
    pub fn contrast(mut self, contrast: Contrast) -> Self {
        self.contrast = contrast;
        self
    }

    /// ライト / ダークの選び方を変える（既定は OS に従う）。
    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// フォントファミリを指定する。未指定ならシステムフォント。
    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// タイプスケールを書き換える。
    ///
    /// ```ignore
    /// builder.type_scale(|s| s.body_large = s.body_large.clone().size(px(15.)))
    /// ```
    pub fn type_scale(mut self, f: impl FnOnce(&mut TypeScaleStyles)) -> Self {
        f(&mut self.type_scale);
        self
    }

    /// カラーロールの色を上書きする（ライト・ダークの両方）。
    pub fn override_color(mut self, role: ColorRole, argb: Argb) -> Self {
        self.overrides.retain(|(r, _)| *r != role);
        self.overrides.push((role, argb));
        self
    }

    /// すべての色の上書きを取り消す。
    pub fn clear_color_overrides(mut self) -> Self {
        self.overrides.clear();
        self
    }

    /// 密度を変える。
    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    /// 角丸スケールを書き換える。
    pub fn shapes(mut self, f: impl FnOnce(&mut Shapes)) -> Self {
        f(&mut self.shapes);
        self
    }

    /// `Appearance::System` のときに使う OS の外観。[`Md3ThemeExt::set_md3_theme`] が設定するので、
    /// 通常は呼ばなくてよい。
    pub fn system_is_dark(mut self, is_dark: bool) -> Self {
        self.system_is_dark = is_dark;
        self
    }

    /// テーマを作る。ライトとダークの配色をここで計算する。
    pub fn build(self) -> Theme {
        let seed = Hct::from_argb(self.seed);
        let mut light = ColorTable::new(seed, self.variant, false, self.contrast);
        let mut dark = ColorTable::new(seed, self.variant, true, self.contrast);
        for &(role, argb) in &self.overrides {
            light.set(role, argb);
            dark.set(role, argb);
        }
        let is_dark = match self.appearance {
            Appearance::System => self.system_is_dark,
            Appearance::Light => false,
            Appearance::Dark => true,
        };
        Theme {
            seed: self.seed,
            variant: self.variant,
            contrast: self.contrast,
            appearance: self.appearance,
            overrides: self.overrides,
            light,
            dark,
            is_dark,
            font_family: self.font_family,
            type_scale: self.type_scale,
            density: self.density,
            shapes: self.shapes,
        }
    }
}

/// テーマを `cx.md3()` で参照するための拡張トレイト。
pub trait Md3ThemeExt {
    /// 現在のテーマ。未設定なら既定のテーマ。
    fn md3(&self) -> &Theme;
}

impl Md3ThemeExt for App {
    fn md3(&self) -> &Theme {
        static DEFAULT: OnceLock<Theme> = OnceLock::new();
        self.try_global::<Theme>()
            .unwrap_or_else(|| DEFAULT.get_or_init(Theme::default))
    }
}

/// テーマを設定する拡張トレイト。
pub trait SetMd3ThemeExt {
    /// テーマを設定して全ウィンドウを再描画する。
    ///
    /// `Appearance::System` なら OS の外観に合わせ、それ以外なら OS のウィンドウ枠の外観も合わせる（macOS）。
    fn set_md3_theme(&mut self, theme: Theme);
}

impl SetMd3ThemeExt for App {
    fn set_md3_theme(&mut self, theme: Theme) {
        let forced = match theme.appearance {
            Appearance::System => None,
            Appearance::Light => Some(WindowAppearance::Light),
            Appearance::Dark => Some(WindowAppearance::Dark),
        };
        self.set_window_appearance(forced);
        let theme = if theme.appearance == Appearance::System {
            let is_dark = is_dark_appearance(self.window_appearance());
            Theme { is_dark, ..theme }
        } else {
            theme
        };
        self.set_global(theme);
        self.refresh_windows();
    }
}

/// 既定のテーマを設定する（まだ設定されていなければ）。アプリの起動時に呼ぶ。
pub fn init(cx: &mut App) {
    if !cx.has_global::<Theme>() {
        cx.set_md3_theme(Theme::default());
    }
}

/// ウィンドウの外観（OS のライト / ダーク）の変化をテーマに反映する。
///
/// `Appearance::System` のテーマだけが影響を受ける。返り値を保持している間だけ追従する
/// （ウィンドウと同じ寿命でよければ `detach` する）。
pub fn follow_window_appearance(window: &Window, _cx: &mut App) -> Subscription {
    window.observe_window_appearance(|window, cx| {
        let is_dark = is_dark_appearance(window.appearance());
        let theme = cx.md3();
        if theme.appearance == Appearance::System && theme.is_dark != is_dark {
            let theme = Theme {
                is_dark,
                ..theme.clone()
            };
            cx.set_global(theme);
            cx.refresh_windows();
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_passes_contrast_check() {
        for variant in SchemeVariant::ALL {
            for contrast in Contrast::ALL {
                let theme = Theme::from_seed(DEFAULT_SEED)
                    .variant(variant)
                    .contrast(contrast)
                    .build();
                let issues = theme.check_contrast();
                assert!(issues.is_empty(), "{variant:?} {contrast:?}: {issues:#?}");
            }
        }
    }

    #[test]
    fn override_breaks_contrast_and_is_reported() {
        let theme = Theme::from_seed(DEFAULT_SEED)
            .override_color(ColorRole::OnPrimary, Argb::from_rgb(0x70, 0x60, 0xB0))
            .build();
        let issues = theme.check_contrast();
        assert!(
            issues
                .iter()
                .any(|issue| issue.foreground == ColorRole::OnPrimary
                    && issue.background == ColorRole::Primary
                    && !issue.is_dark)
        );
    }

    #[test]
    fn appearance_selects_table() {
        let light = Theme::from_seed(DEFAULT_SEED)
            .appearance(Appearance::Light)
            .build();
        let dark = light.to_builder().appearance(Appearance::Dark).build();
        assert!(!light.is_dark());
        assert!(dark.is_dark());
        let seed = Hct::from_argb(DEFAULT_SEED);
        for (theme, is_dark) in [(&light, false), (&dark, true)] {
            let scheme = DynamicScheme::new(
                seed,
                SchemeVariant::TonalSpot,
                is_dark,
                ContrastLevel::STANDARD,
            );
            assert_eq!(
                theme.argb(ColorRole::Primary),
                scheme.argb(ColorRole::Primary)
            );
        }
    }

    #[test]
    fn hsla_conversion_round_trips_white() {
        let white = to_hsla(Argb::new(0xffffffff));
        assert_eq!(white.l, 1.0);
        assert_eq!(white.a, 1.0);
    }

    #[test]
    fn shadows_follow_elevation() {
        let theme = Theme::default();
        assert!(theme.shadows(Elevation::Level0).is_empty());
        let shadows = theme.shadows(Elevation::Level3);
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[1].blur_radius, px(8.0));
    }
}
