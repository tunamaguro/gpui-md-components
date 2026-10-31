//! テーマの切り替えと基礎（色・文字・高さ・アイコン）のページ。

use gpui::{App, IntoElement, Window, div, prelude::*, px};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{
    self, Appearance, Argb, Contrast, Density, Elevation, Md3ThemeExt, MotionScheme, SchemeVariant,
    SetMd3ThemeExt, Shape, Theme, TypeScale,
};
use md3_components::{Icon, IconName, Surface, Text};

use super::{Choice, Section, option_row};

/// 切り替え UI で選べるシード色（見本用の任意の色）。
const SEEDS: [(&str, Argb); 5] = [
    ("Purple", theme::DEFAULT_SEED),
    ("Blue", Argb::from_rgb(0x00, 0x61, 0xA4)),
    ("Green", Argb::from_rgb(0x38, 0x6A, 0x20)),
    ("Red", Argb::from_rgb(0xB3, 0x26, 0x1E)),
    ("Orange", Argb::from_rgb(0xE8, 0x75, 0x1A)),
];

/// テーマの切り替え UI。
pub fn controls(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let current = cx.md3().clone();
    let seed_options = SEEDS.map(|(name, seed)| {
        let option = Choice::new(format!("seed-{name}"), name, current.seed() == seed);
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().seed(seed).build();
            cx.set_md3_theme(theme);
        })
    });
    let variant_options = SchemeVariant::ALL.map(|variant| {
        let name = format!("{variant:?}");
        let option = Choice::new(
            format!("variant-{name}"),
            name,
            current.variant() == variant,
        );
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().variant(variant).build();
            cx.set_md3_theme(theme);
        })
    });
    let contrast_options = Contrast::ALL.map(|contrast| {
        let name = format!("{contrast:?}");
        let option = Choice::new(
            format!("contrast-{name}"),
            name,
            current.contrast() == contrast,
        );
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().contrast(contrast).build();
            cx.set_md3_theme(theme);
        })
    });
    let appearance_options = Appearance::ALL.map(|appearance| {
        let name = format!("{appearance:?}");
        let option = Choice::new(
            format!("appearance-{name}"),
            name,
            current.appearance() == appearance,
        )
        .icon(match appearance {
            Appearance::System => IconName::Settings,
            Appearance::Light => IconName::LightMode,
            Appearance::Dark => IconName::DarkMode,
        });
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().appearance(appearance).build();
            cx.set_md3_theme(theme);
        })
    });
    let density_options = Density::ALL.map(|density| {
        let name = format!("{density:?}");
        let option = Choice::new(
            format!("density-{name}"),
            name,
            current.density() == density,
        );
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().density(density).build();
            cx.set_md3_theme(theme);
        })
    });
    let text_scale_options = [1.0, 1.5, 2.0].map(|scale: f32| {
        let name = format!("{:.0}%", scale * 100.0);
        let option = Choice::new(
            format!("text-scale-{name}"),
            name,
            current.text_scale() == scale,
        );
        option.render(window, cx, move |cx| {
            let theme = cx.md3().to_builder().text_scale(scale).build();
            cx.set_md3_theme(theme);
        })
    });
    let mut motion_options: Vec<_> = MotionScheme::ALL
        .map(|scheme| {
            let name = format!("{scheme:?}");
            let option = Choice::new(
                format!("motion-{name}"),
                name,
                current.motion_scheme() == scheme,
            );
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().motion_scheme(scheme).build();
                cx.set_md3_theme(theme);
            })
        })
        .into();
    // gpui の `App::reduce_motion`（OS の「視差効果を減らす」相当）。部品のバネとアニメーションが止まる。
    let reduce_motion = cx.reduce_motion();
    motion_options.push(
        Choice::new("motion-reduce", "Reduce motion", reduce_motion).render(
            window,
            cx,
            move |cx| cx.set_reduce_motion(!reduce_motion),
        ),
    );

    let issues = current.check_contrast();
    let contrast_report = if issues.is_empty() {
        "check_contrast: no issues".to_owned()
    } else {
        format!("check_contrast: {} issue(s)", issues.len())
    };

    Section::new("Theme")
        .child(option_row("Seed", seed_options))
        .child(option_row("Variant", variant_options))
        .child(option_row("Contrast", contrast_options))
        .child(option_row("Appearance", appearance_options))
        .child(option_row("Density", density_options))
        .child(option_row("Text scale", text_scale_options))
        .child(option_row("Motion", motion_options))
        .child(Text::new(TypeScale::BodySmall, contrast_report).color(ColorRole::OnSurfaceVariant))
}

/// 基礎のページ。
pub fn foundations(theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(Section::new("Color roles").child(swatches(theme)))
        .child(Section::new("Type scale").child(type_scale()))
        .child(Section::new("Elevation").child(elevations()))
        .child(Section::new("Icons").child(icons()))
}

/// 全カラーロールの見本。
fn swatches(theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_2()
        .children(ColorRole::ALL.map(|role| {
            let argb = theme.argb(role);
            let content = theme::content_color(role)
                .map(|content| theme.color(content))
                .unwrap_or_else(|| {
                    // 対になる前景色が無いロールは、明るさで白黒を選ぶ。
                    if argb.lstar().value() > 50.0 {
                        gpui::black()
                    } else {
                        gpui::white()
                    }
                });
            div()
                .w(px(168.0))
                .h(px(72.0))
                .p_2()
                .flex()
                .flex_col()
                .justify_between()
                .bg(theme.color(role))
                .text_color(content)
                .border_1()
                .border_color(theme.color(ColorRole::OutlineVariant))
                .rounded(px(8.0))
                .child(Text::new(TypeScale::LabelMedium, role.name()))
                .child(Text::new(
                    TypeScale::LabelSmall,
                    format!("#{:06X}", argb.to_u32() & 0x00FF_FFFF),
                ))
        }))
}

/// タイプスケール 15 種の見本。
fn type_scale() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(TypeScale::ALL.map(|scale| Text::new(scale, format!("{scale:?} あいう Aa"))))
}

/// 高さ 6 段階の見本。
fn elevations() -> impl IntoElement {
    let levels = [
        Elevation::Level0,
        Elevation::Level1,
        Elevation::Level2,
        Elevation::Level3,
        Elevation::Level4,
        Elevation::Level5,
    ];
    div()
        .flex()
        .flex_wrap()
        .gap_6()
        .p_2()
        .children(levels.map(|level| {
            Surface::new(ColorRole::SurfaceContainerLow)
                .elevation(level)
                .shape(Shape::Medium)
                .w(px(112.0))
                .h(px(72.0))
                .flex()
                .items_center()
                .justify_center()
                .child(Text::new(TypeScale::LabelLarge, format!("{level:?}")))
        }))
}

/// アイコンの見本。
fn icons() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_4()
        .children(IconName::ALL.map(Icon::new))
}
