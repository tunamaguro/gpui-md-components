//! FAB を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{Button, ButtonVariant, ExtendedFab, ExtendedFabSize, Fab, FabColor, FabSize};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const SIZES: [(FabSize, &str); 4] = [
    (FabSize::Small, "Small"),
    (FabSize::Baseline, "Baseline"),
    (FabSize::Medium, "Medium"),
    (FabSize::Large, "Large"),
];

const COLORS: [(FabColor, &str); 6] = [
    (FabColor::PrimaryContainer, "Primary container"),
    (FabColor::SecondaryContainer, "Secondary container"),
    (FabColor::TertiaryContainer, "Tertiary container"),
    (FabColor::Primary, "Primary"),
    (FabColor::Secondary, "Secondary"),
    (FabColor::Tertiary, "Tertiary"),
];

const EXTENDED_SIZES: [(ExtendedFabSize, &str); 3] = [
    (ExtendedFabSize::Small, "Small"),
    (ExtendedFabSize::Medium, "Medium"),
    (ExtendedFabSize::Large, "Large"),
];

/// FAB を並べるページ。押した回数と最後に押した FAB、Extended FAB を展開しているかを持つ
pub struct FabPage {
    clicks: usize,
    last_clicked: Option<SharedString>,
    expanded: bool,
}

impl Default for FabPage {
    fn default() -> Self {
        Self {
            clicks: 0,
            last_clicked: None,
            expanded: true,
        }
    }
}

impl Render for FabPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row = |id: &'static str, name: &'static str| {
            div()
                .flex()
                .items_center()
                .gap_6()
                .child(label(&theme, id, name).w_32())
        };

        div()
            .id("fab-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(label(
                &theme,
                "clicks",
                format!(
                    "押した回数: {} / 最後に押した FAB: {}",
                    self.clicks,
                    self.last_clicked.as_deref().unwrap_or("なし")
                ),
            ))
            .child(row("sizes", "Sizes").children(SIZES.map(|(size, name)| {
                let id = SharedString::from(format!("size-{name}"));
                Fab::new(id.clone(), md3_icons::ADD, format!("Add ({name})"))
                    .size(size)
                    .on_click(record_click(id, cx))
            })))
            .child(
                row("colors", "Colors").children(COLORS.map(|(color, name)| {
                    let id = SharedString::from(format!("color-{name}"));
                    Fab::new(id.clone(), md3_icons::EDIT, format!("Edit ({name})"))
                        .color(color)
                        .on_click(record_click(id, cx))
                })),
            )
            .child(
                row("expanded", "Extended").child(
                    Button::new("toggle-expanded")
                        .variant(ButtonVariant::Outlined)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.expanded = !this.expanded;
                            cx.notify();
                        }))
                        .child(gpui::text!(
                            id = "toggle-expanded-label",
                            if self.expanded { "Collapse" } else { "Expand" }
                        )),
                ),
            )
            .children(EXTENDED_SIZES.map(|(size, name)| {
                let id = SharedString::from(format!("extended-{name}"));
                row(
                    match size {
                        ExtendedFabSize::Small => "extended-small",
                        ExtendedFabSize::Medium => "extended-medium",
                        ExtendedFabSize::Large => "extended-large",
                    },
                    name,
                )
                .child(
                    ExtendedFab::new(id.clone(), md3_icons::EDIT, "Compose")
                        .size(size)
                        .expanded(self.expanded)
                        .on_click(record_click(id, cx)),
                )
            }))
            .child(
                // ラベル付きの FAB は幅を縮めないため、ウィンドウの幅に収まるよう折り返す。
                // 行の幅を決めないと、折り返す箱は一覧を 1 行に並べた幅で測られて折り返さない
                row("extended-colors", "Extended colors").w_full().child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_wrap()
                        .gap_6()
                        .children(COLORS.map(|(color, name)| {
                            let id = SharedString::from(format!("extended-color-{name}"));
                            ExtendedFab::new(id.clone(), md3_icons::ADD, name)
                                .color(color)
                                .expanded(self.expanded)
                                .on_click(record_click(id, cx))
                        })),
                ),
            )
    }
}

/// 押すと押した回数と最後に押した FAB を更新するハンドラ。`id` を最後に押した FAB として表示する
fn record_click(
    id: SharedString,
    cx: &Context<FabPage>,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static {
    cx.listener(move |this, _, _, cx| {
        this.clicks += 1;
        this.last_clicked = Some(id.clone());
        cx.notify();
    })
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡し、テキストが変わっても同じ値を渡すこと
fn label(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    text: impl Into<SharedString>,
) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id.into(), text.into()))
}
