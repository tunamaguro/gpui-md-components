//! メニューを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, prelude::*,
};
use md3_components::{
    Button, ButtonVariant, IconButton, Menu, MenuGroup, MenuItem, MenuItemKind, MenuVariant,
    Tooltip,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 並び替えの基準
const SORT_KEYS: [(&str, &str, &str); 3] = [
    ("name", "Name", "A to Z"),
    ("date", "Date modified", "Newest first"),
    ("size", "Size", "Largest first"),
];

/// メニューを開くボタンを並べるページ。開いているメニューと、項目の状態を持つ
#[derive(Default)]
pub struct MenuPage {
    /// 開いているメニューの id
    expanded: Option<&'static str>,
    ruler: bool,
    grid: bool,
    sort: usize,
    /// 最後に操作した項目
    last_action: Option<SharedString>,
}

impl MenuPage {
    fn menu(&self, id: &'static str, cx: &mut Context<Self>) -> Menu {
        Menu::new(id, self.expanded == Some(id)).on_expanded_change(cx.listener(
            move |this, expanded: &bool, _, cx| {
                this.expanded = expanded.then_some(id);
                cx.notify();
            },
        ))
    }

    fn action(&self, id: &'static str, label: &'static str, cx: &mut Context<Self>) -> MenuItem {
        MenuItem::new(id, label).on_click(cx.listener(move |this, _, _, cx| {
            this.last_action = Some(label.into());
            cx.notify();
        }))
    }

    fn edit_menu(&self, id: &'static str, cx: &mut Context<Self>) -> Menu {
        self.menu(id, cx)
            .group(
                MenuGroup::new("edit")
                    .item(
                        self.action("cut", "Cut", cx)
                            .leading_icon(md3_icons::CONTENT_CUT)
                            .shortcut("Ctrl+X"),
                    )
                    .item(
                        self.action("copy", "Copy", cx)
                            .leading_icon(md3_icons::CONTENT_COPY)
                            .shortcut("Ctrl+C"),
                    )
                    .item(
                        self.action("paste", "Paste", cx)
                            .leading_icon(md3_icons::CONTENT_PASTE)
                            .shortcut("Ctrl+V")
                            .disabled(true),
                    ),
            )
            .group(
                MenuGroup::new("view")
                    .label("View")
                    .divider()
                    .item(
                        MenuItem::new("ruler", "Ruler")
                            .kind(MenuItemKind::Checkbox(self.ruler))
                            .leading_icon(md3_icons::STRAIGHTEN)
                            .selected_leading_icon(md3_icons::CHECK)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.ruler = !this.ruler;
                                cx.notify();
                            })),
                    )
                    .item(
                        MenuItem::new("grid", "Grid")
                            .kind(MenuItemKind::Checkbox(self.grid))
                            .leading_icon(md3_icons::GRID_ON)
                            .selected_leading_icon(md3_icons::CHECK)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.grid = !this.grid;
                                cx.notify();
                            })),
                    ),
            )
            .group(
                MenuGroup::new("other").item(
                    self.action("settings", "Settings", cx)
                        .trailing_icon(md3_icons::SETTINGS),
                ),
            )
    }

    fn sort_menu(&self, id: &'static str, cx: &mut Context<Self>) -> Menu {
        self.menu(id, cx)
            .variant(MenuVariant::Vibrant)
            .aria_label("Sort by")
            .group(
                MenuGroup::new("sort")
                    .label("Sort by")
                    .items(SORT_KEYS.iter().enumerate().map(
                        |(ix, (item_id, label, description))| {
                            MenuItem::new(*item_id, *label)
                                .kind(MenuItemKind::Radio(self.sort == ix))
                                .selected_leading_icon(md3_icons::CHECK)
                                .supporting_text(*description)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.sort = ix;
                                    cx.notify();
                                }))
                        },
                    )),
            )
    }
}

impl Render for MenuPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row = || div().flex().flex_wrap().items_center().gap_4();
        let last_action = match &self.last_action {
            Some(action) => format!("Last action: {action}"),
            None => "Last action: none".to_string(),
        };

        div()
            .id("menu-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(label(&theme, "standard", "Standard"))
            .child(
                row()
                    .child(
                        IconButton::new("more", md3_icons::MORE_VERT, "More options")
                            .tooltip(Tooltip::new("More options"))
                            .menu(self.edit_menu("more-menu", cx)),
                    )
                    .child(
                        Button::new("edit")
                            .variant(ButtonVariant::Outlined)
                            .icon(md3_icons::EDIT)
                            .aria_label("Edit")
                            .menu(self.edit_menu("edit-menu", cx))
                            .child(gpui::text!(id = "edit-label", "Edit")),
                    ),
            )
            .child(label(&theme, "vibrant", "Vibrant"))
            .child(
                row()
                    .child(
                        IconButton::new("sort", md3_icons::SORT, "Sort")
                            .tooltip(Tooltip::new("Sort"))
                            .menu(self.sort_menu("sort-menu", cx)),
                    )
                    .child(
                        Button::new("sort-button")
                            .variant(ButtonVariant::FilledTonal)
                            .menu(self.sort_menu("sort-button-menu", cx))
                            .child(gpui::text!(
                                id = "sort-button-label",
                                SharedString::from(format!("Sort by: {}", SORT_KEYS[self.sort].1))
                            )),
                    ),
            )
            .child(label(&theme, "last-action", last_action))
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡すこと
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
