//! Menu（ドロップダウンメニュー）。

use std::rc::Rc;

use gpui::{
    App, Edges, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{list, menu as tokens};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt, TypeScale};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};
use gpui_headless::MenuKind;
use gpui_headless::overlay::DismissHandler;

/// メニューの項目。
pub struct MenuItem {
    label: SharedString,
    aria_label: Option<Option<SharedString>>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    trailing_text: Option<SharedString>,
    disabled: bool,
    on_select: Option<DismissHandler>,
}

impl MenuItem {
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        let mut node = gpui_headless::A11yNode::new(gpui::Role::MenuItem)
            .with_label(
                self.aria_label
                    .clone()
                    .unwrap_or_else(|| Some(self.label.clone())),
            )
            .disabled(self.disabled);
        if let Some(text) = &self.trailing_text {
            node = node.description(text.clone());
        }
        node
    }

    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            aria_label: None,
            leading_icon: None,
            trailing_icon: None,
            trailing_text: None,
            disabled: false,
            on_select: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 先頭のアイコン。
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// 末尾のアイコン。
    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// 末尾の文字（キーボードショートカットなど）。
    pub fn trailing_text(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 選ばれたときの処理。この後にメニューの `on_dismiss` が呼ばれる。
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

/// ドロップダウンメニュー。開いている間だけ描く。
///
/// 基準の要素（ボタンなど）と同じ `relative` な親の子として置くと、基準の要素の下に出る。
/// 基準の要素の `on_click` では開閉を切り替えること（基準の要素の上の押下ではメニューは閉じない）。
///
/// 開くと最初の項目にフォーカスし、上下の矢印・Home・End で項目を移る。Enter / Space で選び、
/// Esc・Tab・外側の押下で閉じる。キーボードで閉じたときは、開く前の要素にフォーカスを戻す。
#[derive(IntoElement)]
pub struct Menu {
    scrollbar: bool,
    id: ElementId,
    label: Option<SharedString>,
    items: Vec<MenuItem>,
    selected: Option<usize>,
    on_dismiss: Option<DismissHandler>,
    kind: MenuKind,
    match_anchor_width: bool,
    inline: bool,
}

impl Menu {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            label: None,
            items: Vec::new(),
            selected: None,
            on_dismiss: None,
            kind: MenuKind::Actions,
            match_anchor_width: false,
            inline: false,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 項目を足す。
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// 項目をまとめて足す。
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// 閉じるときの処理（項目を選んだとき、Esc、Tab、外側の押下）。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Select が使う（選択肢のメニュー）。
    #[cfg_attr(not(feature = "inputs"), allow(dead_code))]
    pub(crate) fn kind(mut self, kind: MenuKind) -> Self {
        self.kind = kind;
        self
    }

    /// Select が使う（選択肢のメニュー）。
    #[cfg_attr(not(feature = "inputs"), allow(dead_code))]
    pub(crate) fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// Select が使う（選択肢のメニュー）。
    #[cfg_attr(not(feature = "inputs"), allow(dead_code))]
    pub(crate) fn match_anchor_width(mut self) -> Self {
        self.match_anchor_width = true;
        self
    }

    /// 見本用に、その場に描く（フォーカスは動かさない）。
    #[doc(hidden)]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selector = format!("md3-menu-{}", self.id);
        let theme = &cx.md3().clone();
        let label_style = theme.text_style(TypeScale::LabelLarge);
        let trailing_style = theme.text_style(list::ITEM_TRAILING_SUPPORTING_TEXT_FONT);
        let item_space = px(layout::MENU_ITEM_HORIZONTAL_SPACE);
        let margin_y = px(layout::MENU_VERTICAL_MARGIN);
        let margin_x = px(layout::MENU_HORIZONTAL_MARGIN);

        let mut menu = gpui_headless::Menu::new(self.id.clone())
            .kind(self.kind)
            .selected(self.selected)
            .match_anchor_width(self.match_anchor_width)
            .inline(self.inline)
            .window_margin(Edges {
                top: margin_y,
                bottom: margin_y,
                left: margin_x,
                right: margin_x,
            })
            .reveal_margin(crate::focus::reveal_margin())
            .group_prefix("md3");
        if let Some(on_dismiss) = self.on_dismiss {
            menu = menu.on_dismiss(move |window, cx| on_dismiss(window, cx));
        }
        if let Some(label) = self.label {
            menu = menu.aria_label(label);
        }
        let mut looks = Vec::with_capacity(self.items.len());
        for item in self.items {
            let mut row = gpui_headless::MenuItem::new(item.label.clone()).disabled(item.disabled);
            if let Some(label) = item.aria_label.clone() {
                row = row.aria_label(label.unwrap_or_default());
            }
            if let Some(on_select) = item.on_select.clone() {
                row = row.on_select(move |window, cx| on_select(window, cx));
            }
            menu = menu.item(row);
            looks.push(item);
        }

        let mut looks = looks.into_iter();
        let popup = menu.build(window, cx, |row, _, _| {
            let Some(item) = looks.next() else {
                return row.base.into_any_element();
            };
            let description = item.a11y_node().description;
            let (base, control, id, selected) = (row.base, row.state, row.id, row.selected);
            let base = base.when_some(description, |base, description| {
                base.aria_description(description)
            });
            let (text, leading, trailing) = if item.disabled {
                (
                    theme
                        .color(list::ITEM_DISABLED_LABEL_TEXT_COLOR)
                        .opacity(list::ITEM_DISABLED_LABEL_TEXT_OPACITY),
                    theme
                        .color(list::ITEM_DISABLED_LEADING_ICON_COLOR)
                        .opacity(list::ITEM_DISABLED_LEADING_ICON_OPACITY),
                    theme
                        .color(list::ITEM_DISABLED_TRAILING_ICON_COLOR)
                        .opacity(list::ITEM_DISABLED_TRAILING_ICON_OPACITY),
                )
            } else if selected {
                let icon = theme.color(tokens::LIST_ITEM_SELECTED_LEADING_TRAILING_ICON_COLOR);
                (
                    theme.color(tokens::LIST_ITEM_SELECTED_LABEL_TEXT_COLOR),
                    icon,
                    icon,
                )
            } else {
                (
                    theme.color(list::ITEM_LABEL_TEXT_COLOR),
                    theme.color(list::ITEM_LEADING_ICON_COLOR),
                    theme.color(list::ITEM_TRAILING_ICON_COLOR),
                )
            };
            let has_trailing = item.trailing_icon.is_some() || item.trailing_text.is_some();
            base.debug_selector(|| format!("md3-menu-item-{id}"))
                .w_full()
                .min_h(theme.dense(layout::MENU_ITEM_HEIGHT))
                .px(item_space)
                .flex()
                .items_center()
                .text_color(text)
                .type_style(&label_style)
                .when(selected, |this| {
                    this.bg(theme.color(tokens::LIST_ITEM_SELECTED_CONTAINER_COLOR))
                })
                .when_some(item.leading_icon, |this, icon| {
                    this.child(
                        Icon::new(icon)
                            .size(px(list::ITEM_LEADING_ICON_SIZE))
                            .tint(leading),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .when(item.leading_icon.is_some(), |this| this.pl(item_space))
                        .when(has_trailing, |this| this.pr(item_space))
                        .child(item.label),
                )
                .when_some(item.trailing_text, |this, text| {
                    this.child(
                        div()
                            .flex_none()
                            .type_style(&trailing_style)
                            .text_color(if item.disabled {
                                trailing
                            } else {
                                theme.color(list::ITEM_TRAILING_SUPPORTING_TEXT_COLOR)
                            })
                            .child(text),
                    )
                })
                .when_some(item.trailing_icon, |this, icon| {
                    this.child(
                        Icon::new(icon)
                            .size(px(list::ITEM_TRAILING_ICON_SIZE))
                            .tint(trailing),
                    )
                })
                .child(
                    StateLayer::new(
                        ElementId::from(SharedString::from(format!("{}-layer", control.group))),
                        control.group.clone(),
                        ColorRole::OnSurface,
                    )
                    .focused(control.focus_visible)
                    .disabled(item.disabled),
                )
                .child(
                    FocusRing::new(md3_theme::CornerRadius::Fixed(px(0.0)))
                        .inward()
                        .visible(control.focus_visible),
                )
                .into_any_element()
        });
        let scrollbar = (self.scrollbar && popup.is_floating()).then(|| popup.scroll.clone());
        let width = popup.anchor_width;
        let id = self.id;
        popup.place(|container| {
            let container = container
                .debug_selector(|| selector)
                .flex()
                .flex_col()
                .py(px(layout::MENU_VERTICAL_SPACE))
                .min_w(px(layout::MENU_MIN_WIDTH))
                .bg(theme.color(tokens::CONTAINER_COLOR))
                .corner_radius(theme.corner(tokens::CONTAINER_SHAPE))
                .shadow(theme.shadows(Elevation::from_dp(tokens::CONTAINER_ELEVATION)))
                .when_some(scrollbar, |this, scroll| {
                    this.child(crate::Scrollbar::new(
                        ElementId::NamedChild(id.into(), "scrollbar".into()),
                        scroll,
                    ))
                });
            match width {
                Some(width) => container.w(width),
                None => container.max_w(px(layout::MENU_MAX_WIDTH)),
            }
        })
    }
}
