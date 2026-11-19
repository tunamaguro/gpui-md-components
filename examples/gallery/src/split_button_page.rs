//! 分割ボタンを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{ButtonSize, Menu, MenuGroup, MenuItem, SplitButton, SplitButtonVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 分割ボタンを並べるページ。最後の操作と、開いているメニューを持つ
#[derive(Default)]
pub struct SplitButtonPage {
    last_action: Option<SharedString>,
    /// 開いているメニューの分割ボタンの id
    expanded: Option<&'static str>,
}

const VARIANTS: [(&str, &str, SplitButtonVariant); 4] = [
    ("filled", "Filled", SplitButtonVariant::Filled),
    ("tonal", "Filled tonal", SplitButtonVariant::FilledTonal),
    ("elevated", "Elevated", SplitButtonVariant::Elevated),
    ("outlined", "Outlined", SplitButtonVariant::Outlined),
];

const SIZES: [(&str, &str, ButtonSize); 5] = [
    ("extra-small", "XS", ButtonSize::ExtraSmall),
    ("small", "S", ButtonSize::Small),
    ("medium", "M", ButtonSize::Medium),
    ("large", "L", ButtonSize::Large),
    ("extra-large", "XL", ButtonSize::ExtraLarge),
];

impl SplitButtonPage {
    fn split_button(
        &self,
        id: &'static str,
        text: &'static str,
        cx: &mut Context<Self>,
    ) -> SplitButton {
        let action = |id: &'static str, label: &'static str, cx: &mut Context<Self>| {
            MenuItem::new(id, label).on_click(cx.listener(move |this, _, _, cx| {
                this.last_action = Some(label.into());
                cx.notify();
            }))
        };
        let menu = Menu::new(id, self.expanded == Some(id))
            .on_expanded_change(cx.listener(move |this, expanded: &bool, _, cx| {
                this.expanded = expanded.then_some(id);
                cx.notify();
            }))
            .group(
                MenuGroup::new("actions")
                    .item(action("edit", "Edit", cx))
                    .item(action("settings", "Settings", cx))
                    .item(action("send", "Send feedback", cx)),
            );
        SplitButton::new(id, "More options")
            .on_click(cx.listener(move |this, _, _, cx| {
                this.last_action = Some(text.into());
                cx.notify();
            }))
            .menu(menu)
            .child(gpui::text!(
                id = SharedString::from(format!("{id}-label")),
                text
            ))
    }
}

impl Render for SplitButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let variants = div().flex().flex_wrap().gap_4().children(
            VARIANTS.map(|(id, text, variant)| self.split_button(id, text, cx).variant(variant)),
        );
        let sizes = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_4()
            .children(SIZES.map(|(id, text, size)| {
                self.split_button(id, text, cx)
                    .size(size)
                    .icon(md3_icons::EDIT)
            }));
        let disabled = self.split_button("disabled", "Disabled", cx).disabled(true);

        div()
            .id("split-button-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(label(
                &theme,
                "last-action",
                format!(
                    "最後の操作: {}",
                    self.last_action.as_deref().unwrap_or("なし")
                ),
            ))
            .child(label(&theme, "variants-label", "Variants"))
            .child(variants)
            .child(label(&theme, "sizes-label", "Sizes with icons"))
            .child(sizes)
            .child(label(&theme, "disabled-label", "Disabled"))
            .child(div().flex().child(disabled))
    }
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
