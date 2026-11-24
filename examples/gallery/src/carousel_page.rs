//! カルーセルを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{Carousel, CarouselVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 項目の数
const ITEM_COUNT: usize = 10;
/// 項目を塗る色と、その上の文字の色。順に繰り返す
const COLORS: [(ColorRole, ColorRole); 3] = [
    (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
    (
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
    ),
    (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
];

/// カルーセルを並べるページ
#[derive(Default)]
pub struct CarouselPage {
    /// 最後に押した項目
    clicked: Option<(SharedString, usize)>,
}

impl Render for CarouselPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let clicked = match &self.clicked {
            Some((name, index)) => format!("Clicked: {name} item {}", index + 1),
            None => "Clicked: none".to_string(),
        };
        let carousel = |name: &'static str, variant: CarouselVariant, cx: &mut Context<Self>| {
            let theme = theme.clone();
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(label(&theme, name))
                .child(
                    Carousel::new(name, ITEM_COUNT, variant)
                        .item_spacing(px(8.))
                        .aria_label(name)
                        .item(move |item, container, _, _| {
                            let (background, foreground) = COLORS[item.index % COLORS.len()];
                            let mut text = div();
                            text.text_style()
                                .refine(&theme.text_style(TypeScale::TitleMedium));
                            container
                                .aria_label(format!("Item {}", item.index + 1))
                                .bg(theme.color(background))
                                .text_color(theme.color(foreground))
                                .p_4()
                                .child(text.child(format!("{}", item.index + 1)))
                        })
                        .on_click(cx.listener(move |this, index: &usize, _, cx| {
                            this.clicked = Some((name.into(), *index));
                            cx.notify();
                        }))
                        .w(px(360.))
                        .h(px(200.))
                        .flex_none(),
                )
        };

        div()
            .id("carousel-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child({
                let mut status = div();
                status
                    .text_style()
                    .refine(&theme.text_style(TypeScale::BodyMedium));
                status.child(clicked)
            })
            .child(carousel(
                "Multi-browse",
                CarouselVariant::MultiBrowse {
                    preferred_item_width: px(186.),
                },
                cx,
            ))
            .child(carousel(
                "Uncontained",
                CarouselVariant::Uncontained {
                    item_width: px(150.),
                },
                cx,
            ))
            .child(carousel(
                "Centered hero",
                CarouselVariant::CenteredHero {
                    preferred_item_width: None,
                },
                cx,
            ))
    }
}

fn label(theme: &Theme, text: &'static str) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = text, text))
}
