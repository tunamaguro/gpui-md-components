//! テーマの設定
//!
//! 利用者が指定する値だけを持ち、OS の外観などの環境の値は持たない。
//! テーマはこの設定と環境から毎回解決し直すため、ここに入れた上書きは外観の変更などで失われない

use std::collections::BTreeMap;

use gpui::SharedString;
use md3_color::{Argb, ColorRole, ContrastLevel, SpecVersion, TonalPalette, Variant};
use md3_tokens::density::Density;
use md3_tokens::motion::MotionScheme;
use md3_tokens::shape::Shapes;
use md3_tokens::typography::{FontFamily, Typeface};

/// 既定の元の色。MD3 のベースラインの主色 `#6750A4`
pub(crate) const DEFAULT_SOURCE: Argb = Argb::from_rgb(0x67, 0x50, 0xA4);

/// テーマの設定
///
/// [`Default`] は MD3 のベースラインの配色、OS の外観に従う明暗、tokens の既定の書体・形・モーション
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemeSettings {
    /// ライト / ダークの選び方
    pub appearance: AppearanceMode,
    /// 配色
    pub color: ColorSettings,
    /// 書体の系統ごとのフォント
    pub fonts: Fonts,
    /// 角丸の段階
    pub shapes: Shapes,
    /// モーションの方式
    pub motion: MotionScheme,
    /// 部品の密度
    pub density: Density,
    /// 文字倍率
    pub text_scale: TextScale,
}

/// ライト / ダークの選び方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AppearanceMode {
    /// OS の外観に従う
    #[default]
    System,
    /// 常にライト
    Light,
    /// 常にダーク
    Dark,
}

impl AppearanceMode {
    /// OS の外観が `system` のときに使う明暗
    pub(crate) const fn brightness(self, system: Brightness) -> Brightness {
        match self {
            Self::System => system,
            Self::Light => Brightness::Light,
            Self::Dark => Brightness::Dark,
        }
    }
}

/// 明暗
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Brightness {
    /// ライト
    #[default]
    Light,
    /// ダーク
    Dark,
}

impl Brightness {
    /// gpui のウィンドウの外観の明暗
    pub(crate) const fn from_window_appearance(appearance: gpui::WindowAppearance) -> Self {
        match appearance {
            gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => Self::Light,
            gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark => Self::Dark,
        }
    }
}

/// 文字倍率。`sp` の寸法に掛ける正の有限値で、1 が等倍
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TextScale(f32);

impl TextScale {
    /// 倍率から作る。正の有限値でなければ `None`
    pub fn new(scale: f32) -> Option<Self> {
        (scale.is_finite() && scale > 0.0).then_some(Self(scale))
    }

    /// 倍率
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl Default for TextScale {
    fn default() -> Self {
        Self(1.0)
    }
}

/// 書体の系統ごとのフォントのファミリー名
///
/// [`Default`] は tokens の既定の系統 ([`Typeface::family`]) を、gpui のシステムの UI フォント
/// (`.SystemUIFont`) に当てる
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Fonts {
    /// [`Typeface::Brand`] のフォント
    pub brand: SharedString,
    /// [`Typeface::Plain`] のフォント
    pub plain: SharedString,
}

impl Fonts {
    /// `typeface` のフォント
    pub(crate) fn get(&self, typeface: Typeface) -> &SharedString {
        match typeface {
            Typeface::Brand => &self.brand,
            Typeface::Plain => &self.plain,
        }
    }
}

impl Default for Fonts {
    fn default() -> Self {
        let family = |typeface: Typeface| match typeface.family() {
            FontFamily::SansSerif => SharedString::new_static(".SystemUIFont"),
        };
        Self {
            brand: family(Typeface::Brand),
            plain: family(Typeface::Plain),
        }
    }
}

/// 配色の設定
///
/// 色は次の順に重ねて決まる。どの段も省略できる
///
/// 1. [`source`](Self::source) と [`variant`](Self::variant) などから作る動的配色
/// 2. [`palettes`](Self::palettes) によるパレットの差し替え。トーンは動的配色の規則で決まるため、
///    明暗やコントラストレベルを変えても役割どうしの関係が保たれる
/// 3. [`light`](Self::light) / [`dark`](Self::dark) によるロールごとの色の差し替え。
///    指定した色はそのまま使われ、コントラストレベルに追従しない
///
/// [`custom_colors`](Self::custom_colors) で、MD3 の役割にないアプリ独自の色も加えられる
#[derive(Clone, Debug, PartialEq)]
pub struct ColorSettings {
    /// 元の色
    pub source: Argb,
    /// 2 色目以降の元の色。[`Variant::Cmf`] などの複数の色を使う配色で使う
    pub additional_sources: Vec<Argb>,
    /// 配色の種類
    pub variant: Variant,
    /// 配色の仕様の版
    pub spec_version: SpecVersion,
    /// コントラストレベル
    pub contrast: ContrastLevel,
    /// パレットの差し替え
    pub palettes: PaletteOverrides,
    /// ライトでのロールごとの色の差し替え
    pub light: RoleOverrides,
    /// ダークでのロールごとの色の差し替え
    pub dark: RoleOverrides,
    /// アプリ独自の色。テーマではキーで [`Theme::custom_color`](crate::Theme::custom_color) から引く
    ///
    /// キーは `pub const WARNING: &str = "warning";` のように定数にしておくと、書き間違いをコンパイル時に防げる
    pub custom_colors: BTreeMap<&'static str, CustomColor>,
}

impl ColorSettings {
    /// `brightness` でのロールごとの色の差し替え
    pub(crate) fn overrides(&self, brightness: Brightness) -> &RoleOverrides {
        match brightness {
            Brightness::Light => &self.light,
            Brightness::Dark => &self.dark,
        }
    }
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            source: DEFAULT_SOURCE,
            additional_sources: Vec::new(),
            variant: Variant::TonalSpot,
            spec_version: SpecVersion::default(),
            contrast: ContrastLevel::STANDARD,
            palettes: PaletteOverrides::default(),
            light: RoleOverrides::default(),
            dark: RoleOverrides::default(),
            custom_colors: BTreeMap::new(),
        }
    }
}

/// パレットの差し替え。`None` のパレットは配色の種類の規則で作る
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaletteOverrides {
    /// 主色
    pub primary: Option<TonalPalette>,
    /// 第二色
    pub secondary: Option<TonalPalette>,
    /// 第三色
    pub tertiary: Option<TonalPalette>,
    /// 中間色
    pub neutral: Option<TonalPalette>,
    /// 中間色バリアント
    pub neutral_variant: Option<TonalPalette>,
    /// エラー色
    pub error: Option<TonalPalette>,
}

/// ロールごとの色の差し替え
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoleOverrides([Option<Argb>; ColorRole::ALL.len()]);

impl RoleOverrides {
    /// `role` の色を `color` にする
    ///
    /// ロールの色は不透明として扱い、`color` のアルファは無視する
    pub fn set(&mut self, role: ColorRole, color: Argb) {
        self.0[role.index()] = Some(Argb::new(color.to_u32() | 0xff00_0000));
    }

    /// `role` の差し替えをやめ、配色の色に戻す
    pub fn clear(&mut self, role: ColorRole) {
        self.0[role.index()] = None;
    }

    /// `role` に指定した色。指定していなければ `None`
    pub(crate) fn get(&self, role: ColorRole) -> Option<Argb> {
        self.0[role.index()]
    }
}

impl Default for RoleOverrides {
    fn default() -> Self {
        Self([None; ColorRole::ALL.len()])
    }
}

/// アプリ独自の色 (Material Theme Builder の custom color)
///
/// テーマでは [`CustomColorRoles`](crate::CustomColorRoles) の 4 つの役割になる
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomColor {
    /// 元の色
    pub color: Argb,
    /// `true` なら色相を配色の元の色に近づけて調和させる
    pub harmonize: bool,
}
