//! Menu（ドロップダウンメニュー）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnchoredPositionMode, App, Bounds, Edges, ElementId, Entity, FocusHandle, IntoElement, Pixels,
    RenderOnce, Role, SharedString, Window, anchored, canvas, deferred, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{list, menu as tokens};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt, TypeScale};

use super::layer::{
    DismissHandler, InitialFocus, MENU_PRIORITY, dismiss_and_restore, overlay_focus,
};
use crate::a11y::{A11yExt, A11yNode};
use crate::control::{control, step};
use crate::keys::{Dismiss, MENU_CONTEXT};
use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt, focus_handle};

/// メニューの項目。
pub struct MenuItem {
    label: SharedString,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    trailing_text: Option<SharedString>,
    disabled: bool,
    on_select: Option<DismissHandler>,
}

impl MenuItem {
    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            leading_icon: None,
            trailing_icon: None,
            trailing_text: None,
            disabled: false,
            on_select: None,
        }
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

/// メニューの使われ方。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuKind {
    /// 操作の一覧（`menu` / `menuitem`）。
    Actions,
    /// 値の選択肢（`listbox` / `option`）。Select が使う。
    Options,
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
    label: SharedString,
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

    /// ID とラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            on_dismiss: None,
            kind: MenuKind::Actions,
            match_anchor_width: false,
            inline: false,
        }
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

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = match self.kind {
            MenuKind::Actions => Role::Menu,
            MenuKind::Options => Role::ListBox,
        };
        A11yNode::new(role).label(self.label.clone())
    }

    /// 項目のアクセシビリティツリーに出す情報。
    pub fn item_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let item = self.items.get(index)?;
        let node = match self.kind {
            MenuKind::Actions => A11yNode::new(Role::MenuItem),
            MenuKind::Options => {
                A11yNode::new(Role::ListBoxOption).selected(self.selected == Some(index))
            }
        };
        Some(
            node.label(item.label.clone())
                .position(index, self.items.len())
                .disabled(item.disabled),
        )
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let parent = Arc::new(self.id.clone());
        let item_ids: Vec<ElementId> = (0..self.items.len())
            .map(|index| ElementId::NamedChild(parent.clone(), format!("item-{index}").into()))
            .collect();
        let enabled: Vec<bool> = self.items.iter().map(|item| !item.disabled).collect();
        // control() と同じキーで取り出すので、同じフォーカスハンドルになる。
        let handles: Vec<Option<FocusHandle>> = item_ids
            .iter()
            .zip(&enabled)
            .map(|(id, &enabled)| enabled.then(|| focus_handle(id.clone(), window, cx)))
            .collect();
        let initial = self
            .selected
            .filter(|&index| enabled.get(index).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&enabled| enabled))
            .and_then(|index| handles.get(index).cloned().flatten());
        let focus = (!self.inline).then(|| {
            overlay_focus(
                &self.id,
                move || match initial {
                    Some(handle) => InitialFocus::Handle(handle),
                    None => InitialFocus::Within,
                },
                window,
                cx,
            )
        });
        let anchor_bounds: Entity<Option<Bounds<Pixels>>> = window.use_keyed_state(
            ElementId::NamedChild(parent.clone(), "anchor".into()),
            cx,
            |_, _| None,
        );
        let close = focus
            .as_ref()
            .and_then(|focus| dismiss_and_restore(self.on_dismiss.clone(), focus));

        let node = self.a11y_node();
        let item_nodes: Vec<Option<A11yNode>> = (0..self.items.len())
            .map(|index| self.item_a11y_node(index))
            .collect();
        let theme = &cx.md3().clone();
        let radius = theme.corner(tokens::CONTAINER_SHAPE);
        let label_style = theme.text_style(TypeScale::LabelLarge);
        let trailing_style = theme.text_style(list::ITEM_TRAILING_SUPPORTING_TEXT_FONT);
        let viewport = window.viewport_size();
        let margin_y = px(layout::MENU_VERTICAL_MARGIN);
        let item_space = px(layout::MENU_ITEM_HORIZONTAL_SPACE);

        // 開いたメニューは画面に収まらなければスクロールする。矢印で移った項目を見える範囲に入れる。
        let scroll = crate::scroll::scroll_handle(&self.id, window, cx);
        let reveal = focus
            .is_some()
            .then(|| crate::scroll::reveal_state(&self.id, window, cx));
        let mut rows = Vec::with_capacity(self.items.len());
        for (index, (item, id)) in self.items.into_iter().zip(item_ids).enumerate() {
            let (base, control) = control(&id, item.disabled, false, window, cx);
            let selected = self.kind == MenuKind::Options && self.selected == Some(index);
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
            let mut row = base
                .when_some(item_nodes[index].clone(), |this, node| this.a11y(node))
                .w_full()
                .h(theme.dense(layout::MENU_ITEM_HEIGHT))
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
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .text_ellipsis()
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
                );
            if let (Some(reveal), Some(handle)) = (&reveal, &handles[index]) {
                row = row.child(crate::scroll::reveal_on_focus(
                    scroll.clone(),
                    handle.clone(),
                    reveal.clone(),
                ));
            }
            if !item.disabled {
                let on_select = item.on_select;
                let close = close.clone();
                row = row.on_click(move |_, window, cx| {
                    if let Some(on_select) = &on_select {
                        on_select(window, cx);
                    }
                    if let Some(close) = &close {
                        close(window, cx);
                    }
                });
            }
            rows.push(row);
        }

        let mut container = div()
            .id(self.id.clone())
            .a11y(node)
            .flex()
            .flex_col()
            .py(px(layout::MENU_VERTICAL_SPACE))
            .min_w(px(layout::MENU_MIN_WIDTH))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .corner_radius(radius)
            .shadow(theme.shadows(Elevation::from_dp(tokens::CONTAINER_ELEVATION)))
            .children(rows);
        container = match (self.match_anchor_width, *anchor_bounds.read(cx)) {
            (true, Some(bounds)) => container.w(bounds.size.width),
            _ => container.max_w(px(layout::MENU_MAX_WIDTH)),
        };

        let Some(focus) = focus else {
            return container.into_any_element();
        };
        let handle = focus.read(cx).handle.clone();
        let on_outside = self.on_dismiss.clone();
        let anchor_for_outside = anchor_bounds.clone();
        let wheel = crate::scroll::NestedScroll::new(&self.id, scroll.clone(), window, cx);
        container = container
            .key_context(MENU_CONTEXT)
            .track_focus(&handle)
            .max_h(viewport.height - margin_y * 2.0)
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .on_scroll(wheel.on_scroll())
            .child(wheel.capture())
            .when(self.scrollbar, |this| {
                this.child(crate::Scrollbar::new(
                    ElementId::NamedChild(self.id.clone().into(), "scrollbar".into()),
                    scroll.clone(),
                ))
            })
            .occlude()
            .on_key_down(move |event, window, cx| {
                let focused = handles
                    .iter()
                    .position(|handle| handle.as_ref().is_some_and(|h| h.is_focused(window)));
                let target = match (event.keystroke.key.as_str(), focused) {
                    ("down", Some(from)) => step(&enabled, from, true),
                    ("up", Some(from)) => step(&enabled, from, false),
                    ("down" | "home", None) | ("home", Some(_)) => {
                        enabled.iter().position(|&enabled| enabled)
                    }
                    ("up" | "end", None) | ("end", Some(_)) => {
                        enabled.iter().rposition(|&enabled| enabled)
                    }
                    _ => return,
                };
                if let Some(handle) = target.and_then(|index| handles.get(index).cloned().flatten())
                {
                    window.focus(&handle, cx);
                }
                cx.stop_propagation();
            })
            .when_some(close, |this, close| {
                this.on_action(move |_: &Dismiss, window, cx| close(window, cx))
            })
            .when_some(on_outside, |this, on_outside| {
                // 外側の押下で閉じる。押した先にフォーカスが移るので、フォーカスは戻さない。
                // 基準の要素の上の押下は無視する（基準の要素の on_click が閉じる）。
                this.on_mouse_down_out(move |event, window, cx| {
                    let on_anchor = anchor_for_outside
                        .read(cx)
                        .is_some_and(|bounds| bounds.contains(&event.position));
                    if !on_anchor {
                        on_outside(window, cx);
                    }
                })
            });

        let edges = Edges {
            top: margin_y,
            bottom: margin_y,
            left: px(layout::MENU_HORIZONTAL_MARGIN),
            right: px(layout::MENU_HORIZONTAL_MARGIN),
        };
        // 親（基準の要素と同じ大きさ）を覆う層の下端に、メニューを重ね描きする。
        div()
            .absolute()
            .inset_0()
            .child(
                canvas(
                    move |bounds, _, cx| {
                        anchor_bounds.update(cx, |stored, _| *stored = Some(bounds))
                    },
                    |_, _, _, _| {},
                )
                .size_full(),
            )
            .child(
                div().absolute().top_full().left_0().child(
                    deferred(
                        anchored()
                            .position_mode(AnchoredPositionMode::Window)
                            .snap_to_window_with_margin(edges)
                            .child(container),
                    )
                    .with_priority(MENU_PRIORITY),
                ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_items_are_menuitems_in_a_set() {
        let menu = Menu::new("m", "編集")
            .item(MenuItem::new("切り取り"))
            .item(MenuItem::new("コピー").disabled(true));
        assert_eq!(menu.a11y_node().role, Role::Menu);
        let item = menu.item_a11y_node(1).expect("item");
        assert_eq!(item.role, Role::MenuItem);
        assert_eq!(item.position, Some((2, 2)));
        assert!(item.disabled);
        assert!(menu.item_a11y_node(2).is_none());
    }

    #[test]
    fn option_menus_report_selection() {
        let menu = Menu::new("m", "色")
            .kind(MenuKind::Options)
            .selected(Some(0))
            .item(MenuItem::new("赤"))
            .item(MenuItem::new("青"));
        assert_eq!(menu.a11y_node().role, Role::ListBox);
        assert_eq!(menu.item_a11y_node(0).and_then(|n| n.selected), Some(true));
        assert_eq!(menu.item_a11y_node(1).and_then(|n| n.selected), Some(false));
    }
}
