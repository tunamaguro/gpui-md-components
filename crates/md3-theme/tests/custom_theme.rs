//! 利用者がテーマを作って使う流れを、公開 API だけで確かめる

use std::sync::Arc;

use gpui::{Empty, TestAppContext, WindowHandle, px, size};
use md3_theme::md3_color::{Argb, ColorRole, ContrastLevel, TonalPalette, Variant};
use md3_theme::md3_tokens::component::filled_button;
use md3_theme::md3_tokens::elevation::Elevation;
use md3_theme::md3_tokens::motion::{MotionScheme, MotionSchemeKey};
use md3_theme::md3_tokens::shape::{CornerShape, CornerSize, ShapeKey, Shapes};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, Sp};
use md3_theme::{
    AppearanceMode, ColorSettings, CustomColor, Fonts, TextScale, Theme, ThemeExt, ThemeSettings,
    dp,
};

const WARNING: &str = "warning";

fn open_window(cx: &mut TestAppContext) -> WindowHandle<Empty> {
    let window = cx.add_window(|_, _| Empty);
    // アプリはウィンドウを開いたときに 1 回、外観への追従を始める
    window
        .update(cx, |_, window, cx| cx.init_md3_theme(window))
        .expect("開いたウィンドウは更新できる");
    window
}

fn current(cx: &mut TestAppContext) -> Arc<Theme> {
    cx.update(|cx| cx.md3_theme().clone())
}

#[gpui::test]
fn 既定のテーマをそのまま使える(cx: &mut TestAppContext) {
    open_window(cx);
    let theme = current(cx);

    // 部品はトークンが参照するロールを渡して色を得る
    let container = theme.color(filled_button::CONTAINER_COLOR);
    assert_eq!(container, theme.color(ColorRole::Primary));
    assert_eq!(theme.settings(), &ThemeSettings::default());
    assert!(theme.contrast_issues().is_empty());
}

#[gpui::test]
fn ブランドの色から作ったテーマを細かく調整できる(cx: &mut TestAppContext) {
    open_window(cx);
    let brand = Argb::from_rgb(0x00, 0x6a, 0x60);
    let warning = Argb::from_rgb(0xf9, 0xa8, 0x25);

    // 1. 元の色と配色の種類
    let mut color = ColorSettings {
        source: brand,
        ..ColorSettings::default()
    };
    color.variant = Variant::Vibrant;
    color.contrast = ContrastLevel::MEDIUM;
    // 2. 第三色のパレットだけを差し替える
    color.palettes.tertiary = Some(TonalPalette::from_argb(Argb::from_rgb(0x8e, 0x24, 0xaa)));
    // 3. ライトのロールを 1 つ固定する
    color
        .light
        .set(ColorRole::Surface, Argb::from_rgb(0xfa, 0xfa, 0xf7));
    // 4. アプリ独自の色を加える。キーは描画で色を引くときに使う
    color.custom_colors.insert(
        WARNING,
        CustomColor {
            color: warning,
            harmonize: true,
        },
    );

    cx.update(|cx| {
        cx.update_md3_theme(move |settings| {
            settings.color = color;
            settings.appearance = AppearanceMode::Light;
            settings.fonts = Fonts {
                brand: "Noto Serif".into(),
                plain: "Noto Sans".into(),
            };
            settings.shapes = Shapes {
                medium: CornerShape::all(CornerSize::Dp(Dp(16.0))),
                ..Shapes::default()
            };
            settings.motion = MotionScheme::Expressive;
            settings.text_scale = TextScale::new(1.25).expect("1.25 は正の有限値");
        })
    })
    .expect("Vibrant と 2026 仕様の組み合わせは解決できる");

    let theme = current(cx);

    // 色
    assert_eq!(
        theme.color(ColorRole::Surface),
        gpui::Rgba {
            r: 250.0 / 255.0,
            g: 250.0 / 255.0,
            b: 247.0 / 255.0,
            a: 1.0,
        }
        .into()
    );
    assert_ne!(
        theme.color(ColorRole::Primary),
        theme.color(ColorRole::Tertiary)
    );
    let warning_roles = theme.custom_color(WARNING).expect("設定に加えた色は引ける");
    assert_ne!(warning_roles.color, warning_roles.on_color);

    // 書体: BodyLarge は Plain の書体で 16sp、行の高さ 24sp。文字倍率 1.25 を掛けて 20px と 30px になる
    let body = theme.text_style(TypeScale::BodyLarge);
    assert_eq!(body.font_family, Some("Noto Sans".into()));
    assert_eq!(body.font_size, Some(px(20.0).into()));
    assert_eq!(body.line_height, Some(px(30.0).into()));
    assert_eq!(theme.sp(Sp(16.0)), px(20.0));
    assert_eq!(dp(Dp(16.0)), px(16.0));

    // 形: CornerMedium は差し替えた段階に、CornerFull は短辺の半分になる
    let bounds = size(px(120.0), px(40.0));
    let medium = theme.corner_radii(ShapeKey::CornerMedium, bounds);
    assert_eq!(medium.top_left, px(16.0));
    let full = theme.corner_radii(ShapeKey::CornerFull, bounds);
    assert_eq!(full.bottom_right, px(20.0));

    // 影とモーション
    assert_eq!(theme.shadow(Elevation::Level2).len(), 2);
    let spring = theme.spring(MotionSchemeKey::FastSpatial);
    let expected = MotionScheme::Expressive.spring(MotionSchemeKey::FastSpatial);
    assert_eq!(spring.stiffness, expected.stiffness);
}

#[gpui::test]
fn コントラストが足りない色も設定でき_検査の結果で分かる(
    cx: &mut TestAppContext,
) {
    open_window(cx);
    let primary = current(cx).color(ColorRole::Primary);
    let gray = Argb::from_rgb(0x80, 0x80, 0x80);
    cx.update(|cx| {
        cx.update_md3_theme(|settings| {
            settings.appearance = AppearanceMode::Light;
            settings.color.light.set(ColorRole::OnPrimary, gray);
        })
    })
    .expect("色を差し替えた設定は解決できる");

    let theme = current(cx);
    assert_eq!(theme.color(ColorRole::Primary), primary);
    assert!(
        theme.contrast_issues().iter().any(|issue| {
            issue.foreground == ColorRole::OnPrimary
                && issue.background == ColorRole::Primary
                && issue.ratio < issue.required
        }),
        "灰色の文字は主色の上で 4.5:1 に届かない"
    );
}

#[gpui::test]
fn 解決できない設定はエラーになり_今のテーマは変わらない(
    cx: &mut TestAppContext,
) {
    open_window(cx);
    let before = current(cx);
    let result = cx.update(|cx| {
        cx.update_md3_theme(|settings| {
            settings.color.variant = Variant::Cmf;
            settings.color.spec_version = md3_theme::md3_color::SpecVersion::Spec2021;
        })
    });
    assert!(result.is_err());
    assert_eq!(current(cx), before);
}
