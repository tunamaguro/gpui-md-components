//! 今のテーマの設定・色・書体・形・影を並べるページ

use gpui::{
    Context, Corners, Div, Hsla, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    Refineable as _, Render, Rgba, SharedString, StatefulInteractiveElement as _, Styled as _,
    Window, div, px, size,
};
use md3_theme::md3_color::{ColorRole, ContrastLevel};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{AppearanceMode, Theme, ThemeExt as _};

/// 主なロールの、背景とその上に置く文字の組 (背景, 前景)
const PAIRS: [(ColorRole, ColorRole); 12] = [
    (ColorRole::Primary, ColorRole::OnPrimary),
    (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
    (ColorRole::Secondary, ColorRole::OnSecondary),
    (
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
    ),
    (ColorRole::Tertiary, ColorRole::OnTertiary),
    (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
    (ColorRole::Error, ColorRole::OnError),
    (ColorRole::ErrorContainer, ColorRole::OnErrorContainer),
    (ColorRole::Surface, ColorRole::OnSurface),
    (ColorRole::SurfaceVariant, ColorRole::OnSurfaceVariant),
    (ColorRole::SurfaceContainerHighest, ColorRole::OnSurface),
    (ColorRole::InverseSurface, ColorRole::InverseOnSurface),
];

/// 今のテーマを並べるページ
pub struct ThemePage;

impl Render for ThemePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme();
        div()
            .id("theme-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_8()
            .child(controls(theme))
            .child(section(theme, "設定", settings(theme)))
            .child(section(theme, "主なロールの組", pairs(theme)))
            .child(section(theme, "すべてのロール", roles(theme)))
            .child(section(theme, "書体", type_scale(theme)))
            .child(section(theme, "形", shapes(theme)))
            .child(section(theme, "影", elevations(theme)))
            .child(section(theme, "コントラストの検査", contrast(theme)))
    }
}

pub(crate) fn section(theme: &Theme, title: &'static str, body: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(styled_text(theme, TypeScale::TitleLarge, title))
        .child(body)
}

pub(crate) fn styled_text(theme: &Theme, scale: TypeScale, text: impl Into<SharedString>) -> Div {
    let mut element = div();
    element.text_style().refine(&theme.text_style(scale));
    element.child(text.into())
}

fn with_corners(element: Div, radii: Corners<Pixels>) -> Div {
    element
        .rounded_tl(radii.top_left)
        .rounded_tr(radii.top_right)
        .rounded_br(radii.bottom_right)
        .rounded_bl(radii.bottom_left)
}

fn hex(color: Hsla) -> String {
    let rgba = Rgba::from(color);
    let channel = |value: f32| (value * 255.0).round() as u8;
    format!(
        "#{:02X}{:02X}{:02X}",
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b)
    )
}

/// 明暗とコントラストレベルを切り替えるボタン
fn controls(theme: &Theme) -> Div {
    let settings = theme.settings();
    let button = |id: SharedString, label: &'static str, selected: bool| {
        let (bg, fg) = if selected {
            (ColorRole::Primary, ColorRole::OnPrimary)
        } else {
            (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            )
        };
        let radii = theme.corner_radii(ShapeKey::CornerFull, size(px(120.0), px(36.0)));
        with_corners(div(), radii)
            .id(id)
            .px_4()
            .py_2()
            .cursor_pointer()
            .bg(theme.color(bg))
            .text_color(theme.color(fg))
            .child(styled_text(theme, TypeScale::LabelLarge, label))
    };
    let appearance = [
        ("System", AppearanceMode::System),
        ("Light", AppearanceMode::Light),
        ("Dark", AppearanceMode::Dark),
    ]
    .map(|(label, mode)| {
        button(
            format!("appearance-{label}").into(),
            label,
            settings.appearance == mode,
        )
        .on_click(move |_, _, cx| {
            cx.update_md3_theme(|settings| settings.appearance = mode)
                .expect("changing the appearance keeps the settings resolvable");
        })
    });
    let contrast = [
        ("Reduced", ContrastLevel::REDUCED),
        ("Standard", ContrastLevel::STANDARD),
        ("Medium", ContrastLevel::MEDIUM),
        ("High", ContrastLevel::HIGH),
    ]
    .map(|(label, level)| {
        button(
            format!("contrast-{label}").into(),
            label,
            settings.color.contrast == level,
        )
        .on_click(move |_, _, cx| {
            cx.update_md3_theme(|settings| settings.color.contrast = level)
                .expect("changing the contrast level keeps the settings resolvable");
        })
    });
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(styled_text(
            theme,
            TypeScale::HeadlineMedium,
            "既定のテーマ",
        ))
        .child(div().flex().flex_wrap().gap_2().children(appearance))
        .child(div().flex().flex_wrap().gap_2().children(contrast))
}

fn settings(theme: &Theme) -> Div {
    let settings = theme.settings();
    let color = &settings.color;
    let source = md3_theme::md3_color::Argb::to_u32(color.source);
    let rows: [(&str, String); 10] = [
        ("元の色", format!("#{:06X}", source & 0x00ff_ffff)),
        ("配色の種類", format!("{:?}", color.variant)),
        ("仕様の版", format!("{:?}", color.spec_version)),
        ("コントラストレベル", format!("{}", color.contrast.value())),
        ("明暗", format!("{:?}", settings.appearance)),
        ("Brand のフォント", settings.fonts.brand.to_string()),
        ("Plain のフォント", settings.fonts.plain.to_string()),
        ("モーション", format!("{:?}", settings.motion)),
        ("密度", format!("{:?}", settings.density)),
        ("文字倍率", format!("{}", settings.text_scale.get())),
    ];
    div()
        .flex()
        .flex_col()
        .gap_1()
        .children(rows.into_iter().map(|(label, value)| {
            div()
                .flex()
                .gap_4()
                .child(
                    styled_text(theme, TypeScale::BodyMedium, label)
                        .w(px(200.0))
                        .text_color(theme.color(ColorRole::OnSurfaceVariant)),
                )
                .child(styled_text(theme, TypeScale::BodyMedium, value))
        }))
}

fn pairs(theme: &Theme) -> Div {
    div()
        .flex()
        .flex_wrap()
        .gap_3()
        .children(PAIRS.map(|(background, foreground)| {
            div()
                .w(px(240.0))
                .p_4()
                .rounded(px(12.0))
                .border_1()
                .border_color(theme.color(ColorRole::OutlineVariant))
                .bg(theme.color(background))
                .text_color(theme.color(foreground))
                .flex()
                .flex_col()
                .gap_1()
                .child(styled_text(theme, TypeScale::TitleSmall, background.name()))
                .child(styled_text(
                    theme,
                    TypeScale::BodySmall,
                    format!("on: {}", foreground.name()),
                ))
                .child(styled_text(
                    theme,
                    TypeScale::LabelSmall,
                    format!(
                        "{} / {}",
                        hex(theme.color(background)),
                        hex(theme.color(foreground))
                    ),
                ))
        }))
}

fn roles(theme: &Theme) -> Div {
    div()
        .flex()
        .flex_wrap()
        .gap_2()
        .children(ColorRole::ALL.map(|role| {
            let color = theme.color(role);
            div()
                .w(px(260.0))
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_none()
                        .size(px(32.0))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(theme.color(ColorRole::Outline))
                        .bg(color),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(styled_text(theme, TypeScale::LabelMedium, role.name()))
                        .child(
                            styled_text(theme, TypeScale::LabelSmall, hex(color))
                                .text_color(theme.color(ColorRole::OnSurfaceVariant)),
                        ),
                )
        }))
}

fn type_scale(theme: &Theme) -> Div {
    use TypeScale::*;
    let scales = [
        DisplayLarge,
        DisplayMedium,
        DisplaySmall,
        HeadlineLarge,
        HeadlineMedium,
        HeadlineSmall,
        TitleLarge,
        TitleMedium,
        TitleSmall,
        BodyLarge,
        BodyMedium,
        BodySmall,
        LabelLarge,
        LabelMedium,
        LabelSmall,
    ];
    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(scales.iter().map(|scale| {
            let style = scale.style();
            div()
                .flex()
                .items_baseline()
                .gap_4()
                .child(
                    styled_text(
                        theme,
                        TypeScale::LabelSmall,
                        format!("{scale:?} ({}sp / {}sp)", style.size.0, style.line_height.0),
                    )
                    .w(px(260.0))
                    .text_color(theme.color(ColorRole::OnSurfaceVariant)),
                )
                .child(styled_text(theme, *scale, "Material Design 3 のテーマ"))
        }))
}

fn shapes(theme: &Theme) -> Div {
    let bounds = size(px(120.0), px(72.0));
    div()
        .flex()
        .flex_wrap()
        .gap_3()
        .children(ShapeKey::ALL.map(|key| {
            let shape = with_corners(div(), theme.corner_radii(key, bounds))
                .w(bounds.width)
                .h(bounds.height)
                .bg(theme.color(ColorRole::PrimaryContainer));
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(shape)
                .child(styled_text(
                    theme,
                    TypeScale::LabelSmall,
                    format!("{key:?}"),
                ))
        }))
}

fn elevations(theme: &Theme) -> Div {
    div()
        .flex()
        .flex_wrap()
        .gap_6()
        .p_2()
        .children((0..=5u8).map(|level| {
            div()
                .w(px(120.0))
                .h(px(72.0))
                .rounded(px(12.0))
                .bg(theme.color(ColorRole::SurfaceContainerLow))
                .shadow(theme.shadow(f32::from(level)))
                .flex()
                .items_center()
                .justify_center()
                .child(styled_text(
                    theme,
                    TypeScale::LabelLarge,
                    format!("Level {level}"),
                ))
        }))
}

fn contrast(theme: &Theme) -> Div {
    let issues = theme.contrast_issues();
    let summary = if issues.is_empty() {
        "文字として重ねる組はすべて 4.5:1 以上".to_owned()
    } else {
        format!("{} 組が 4.5:1 に届かない", issues.len())
    };
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(styled_text(theme, TypeScale::BodyMedium, summary))
        .children(issues.iter().map(|issue| {
            styled_text(
                theme,
                TypeScale::BodySmall,
                format!(
                    "{} on {}: {:.2} (必要 {:.2})",
                    issue.foreground.name(),
                    issue.background.name(),
                    issue.ratio,
                    issue.required
                ),
            )
            .text_color(theme.color(ColorRole::Error))
        }))
}
