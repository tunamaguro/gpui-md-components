//! アイコンボタンのバリアントと状態を並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, prelude::*,
};
use md3_components::{ButtonShape, ButtonSize, IconButton, IconButtonVariant, IconButtonWidth};
use md3_icons::{IconStyle, StyledIcon};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const SIZES: [(ButtonSize, &str); 5] = [
    (ButtonSize::ExtraSmall, "Extra small"),
    (ButtonSize::Small, "Small"),
    (ButtonSize::Medium, "Medium"),
    (ButtonSize::Large, "Large"),
    (ButtonSize::ExtraLarge, "Extra large"),
];

const VARIANTS: [(IconButtonVariant, &str); 4] = [
    (IconButtonVariant::Filled, "Filled"),
    (IconButtonVariant::FilledTonal, "Filled tonal"),
    (IconButtonVariant::Outlined, "Outlined"),
    (IconButtonVariant::Standard, "Standard"),
];

const STYLES: [(IconStyle, &str); 3] = [
    (IconStyle::Outlined, "outlined"),
    (IconStyle::Rounded, "rounded"),
    (IconStyle::Sharp, "sharp"),
];

const WIDTHS: [(IconButtonWidth, &str); 3] = [
    (IconButtonWidth::Narrow, "narrow"),
    (IconButtonWidth::Uniform, "uniform"),
    (IconButtonWidth::Wide, "wide"),
];

/// アイコンボタンを並べるページ。押した回数と最後に押したボタンを持つ
#[derive(Default)]
pub struct IconButtonPage {
    clicks: usize,
    last_clicked: Option<SharedString>,
}

impl Render for IconButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let rows = VARIANTS.map(|(variant, name)| {
            let button = |suffix: &str, icon: md3_icons::Icon, label: &str| {
                icon_button(format!("{name}-{suffix}"), icon, label, cx).variant(variant)
            };
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(label(&theme, name, name).w_32())
                .child(button("add", md3_icons::ADD, "Add"))
                .child(button("edit", md3_icons::EDIT, "Edit"))
                .child(button("square", md3_icons::SETTINGS, "Settings").shape(ButtonShape::Square))
                .child(button("disabled", md3_icons::DELETE, "Delete").disabled(true))
                .child(
                    button("disabled-square", md3_icons::DELETE, "Delete")
                        .shape(ButtonShape::Square)
                        .disabled(true),
                )
        });
        div()
            .id("icon-button-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(label(
                &theme,
                "clicks",
                format!(
                    "押した回数: {} / 最後に押したボタン: {}",
                    self.clicks,
                    self.last_clicked.as_deref().unwrap_or("なし")
                ),
            ))
            .children(rows)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, "styles", "Styles").w_32())
                    .children(STYLES.into_iter().flat_map(|(style, style_name)| {
                        [false, true].map(|filled| {
                            let fill = if filled { "filled" } else { "unfilled" };
                            icon_button(
                                format!("style-{style_name}-{fill}"),
                                StyledIcon::new(md3_icons::FAVORITE)
                                    .style(style)
                                    .filled(filled),
                                format!("Favorite ({style_name}, {fill})"),
                                cx,
                            )
                        })
                    })),
            )
            .children(SIZES.map(|(size, name)| {
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, name, name).w_32())
                    .children(WIDTHS.map(|(width, width_name)| {
                        icon_button(
                            format!("size-{name}-{width_name}"),
                            md3_icons::FAVORITE,
                            format!("Favorite ({name}, {width_name})"),
                            cx,
                        )
                        .variant(IconButtonVariant::FilledTonal)
                        .size(size)
                        .width(width)
                    }))
                    .child(
                        icon_button(
                            format!("size-{name}-square"),
                            md3_icons::SEARCH,
                            format!("Search ({name}, square)"),
                            cx,
                        )
                        .variant(IconButtonVariant::Outlined)
                        .size(size)
                        .shape(ButtonShape::Square),
                    )
            }))
    }
}

/// 押すと押した回数と最後に押したボタンを更新するアイコンボタン。`id` を最後に押したボタンとして表示する
fn icon_button(
    id: impl Into<SharedString>,
    icon: impl Into<StyledIcon>,
    label: impl Into<SharedString>,
    cx: &Context<IconButtonPage>,
) -> IconButton {
    let id = id.into();
    IconButton::new(id.clone(), icon, label).on_click(cx.listener(move |this, _, _, cx| {
        this.clicks += 1;
        this.last_clicked = Some(id.clone());
        cx.notify();
    }))
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡し、テキストが変わっても同じ値を渡すこと
fn label(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    text: impl Into<gpui::SharedString>,
) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id.into(), text.into()))
}
