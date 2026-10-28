//! NavigationDrawer（Standard / Modal）。

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Window, div,
    prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::navigation_drawer as tokens;
use md3_theme::md3_tokens::{layout, state};
use md3_theme::{CornerRadius, Elevation, Md3ThemeExt};

use super::bar::destination_builders;
use super::item::{
    ChangeHandler, Destinations, NavigationItem, destination_a11y_node, navigation_a11y_node,
};
use crate::a11y::A11yExt;
use crate::control::control;
use crate::overlays::layer::{
    DismissHandler, InitialFocus, MODAL_PRIORITY, dismiss_and_restore, modal_container,
    overlay_focus, scrim, window_layer,
};
use crate::{FocusRing, ForcedState, Icon, StateLayer, StyledExt};

/// ドロワーの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationDrawerKind {
    /// 画面の一部として左に並ぶ（幅の広いウィンドウ向け）。
    #[default]
    Standard,
    /// スクリムの上に重なる。フォーカスの扱いは Dialog と同じ（中に閉じ込め、Esc で閉じる）。
    Modal,
}

/// ナビゲーションドロワー（ウィンドウの左端）。
///
/// 項目の上に見出しなどを置ける（子要素）。キー操作は ↑↓ を使うほかは
/// [`NavigationBar`](super::NavigationBar) と同じ。Modal は開いている間だけ描く。
#[derive(IntoElement)]
pub struct NavigationDrawer {
    id: ElementId,
    label: SharedString,
    kind: NavigationDrawerKind,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    on_dismiss: Option<DismissHandler>,
    forced: Option<(usize, ForcedState)>,
    header: Vec<AnyElement>,
    inline: bool,
}

impl NavigationDrawer {
    /// ID と名前（読み上げられる）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: NavigationDrawerKind::default(),
            items: Vec::new(),
            selected: None,
            on_change: None,
            on_dismiss: None,
            forced: None,
            header: Vec::new(),
            inline: false,
        }
    }

    /// 種類を変える。
    pub fn kind(mut self, kind: NavigationDrawerKind) -> Self {
        self.kind = kind;
        self
    }

    /// Modal を閉じるとき（Esc、スクリムの押下）の処理。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// 見本用に、Modal でもスクリムなしでその場に描く（フォーカスは動かさない）。
    #[doc(hidden)]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    destination_builders!();

    /// アクセシビリティツリーに出す情報（Standard は Navigation、Modal は Dialog。項目は Tab）。
    pub fn a11y_node(&self) -> crate::A11yNode {
        match self.kind {
            NavigationDrawerKind::Standard => navigation_a11y_node(&self.label),
            NavigationDrawerKind::Modal => crate::A11yNode::new(gpui::Role::Dialog)
                .label(self.label.clone())
                .modal(!self.inline),
        }
    }
}

impl ParentElement for NavigationDrawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.header.extend(elements);
    }
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let modal = self.kind == NavigationDrawerKind::Modal && !self.inline;
        let focus = modal.then(|| {
            let target = self.selected.map(|index| {
                crate::focus_handle(
                    ElementId::NamedChild(self.id.clone().into(), format!("{index}").into()),
                    window,
                    cx,
                )
            });
            overlay_focus(
                &self.id,
                move || target.map_or(InitialFocus::Within, InitialFocus::Handle),
                window,
                cx,
            )
        });
        let dismiss = match &focus {
            Some(focus) => dismiss_and_restore(self.on_dismiss.clone(), focus),
            None => self.on_dismiss.clone(),
        };
        let destinations = Destinations::new(&self.id, &self.items, self.selected, window, cx);
        let theme = cx.md3().clone();
        let count = self.items.len();
        let pill = theme.corner(tokens::ACTIVE_INDICATOR_SHAPE);
        let items: Vec<AnyElement> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let selected = self.selected == Some(index);
                let forced = self
                    .forced
                    .and_then(|(i, state)| (i == index).then_some(state));
                let (base, control) = control(
                    &destinations.ids[index],
                    item.disabled,
                    destinations.tab_target == Some(index),
                    forced,
                    window,
                    cx,
                );
                let (icon_role, label_role) = if selected {
                    (tokens::ACTIVE_ICON_COLOR, tokens::ACTIVE_LABEL_TEXT_COLOR)
                } else {
                    (
                        tokens::INACTIVE_ICON_COLOR,
                        tokens::INACTIVE_LABEL_TEXT_COLOR,
                    )
                };
                let color = |role: ColorRole| {
                    if item.disabled {
                        theme
                            .color(ColorRole::OnSurface)
                            .opacity(state::DISABLED_CONTENT_OPACITY)
                    } else {
                        theme.color(role)
                    }
                };
                let mut element = base
                    .a11y(destination_a11y_node(item, index, count, selected))
                    .h(px(tokens::ACTIVE_INDICATOR_HEIGHT))
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(layout::NAVIGATION_DRAWER_ITEM_ICON_SPACE))
                    .pl(px(layout::NAVIGATION_DRAWER_ITEM_START_SPACE))
                    .pr(px(layout::NAVIGATION_DRAWER_ITEM_END_SPACE))
                    .corner_radius(pill)
                    .when(selected, |this| {
                        this.bg(theme.color(tokens::ACTIVE_INDICATOR_COLOR))
                    })
                    .child(
                        Icon::new(item.icon)
                            .size(px(tokens::ICON_SIZE))
                            .tint(color(icon_role)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .type_style(&theme.text_style(tokens::LABEL_TEXT_FONT))
                            .text_color(color(label_role))
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(item.label.clone()),
                    )
                    .child(
                        StateLayer::new(
                            SharedString::from(format!("{}-layer", control.group)),
                            control.group.clone(),
                            if selected {
                                tokens::ACTIVE_LABEL_TEXT_COLOR
                            } else {
                                tokens::INACTIVE_LABEL_TEXT_COLOR
                            },
                        )
                        .radius(pill)
                        .focused(control.focus_visible)
                        .disabled(item.disabled)
                        .forced(forced),
                    )
                    .child(FocusRing::new(pill).visible(control.focus_visible));
                if !item.disabled {
                    element = element.on_key_down(destinations.on_key(index, true));
                    if let Some(on_change) = self.on_change.clone() {
                        element =
                            element.on_click(move |_, window, cx| on_change(index, window, cx));
                    }
                }
                element.into_any_element()
            })
            .collect();

        let mut drawer = div()
            .id(self.id.clone())
            .a11y(node)
            .w(px(tokens::CONTAINER_WIDTH))
            .min_w(px(layout::NAVIGATION_DRAWER_MIN_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .px(px(layout::NAVIGATION_DRAWER_ITEM_HORIZONTAL_SPACE))
            .overflow_y_scroll()
            .children(self.header)
            .children(items);
        drawer = match self.kind {
            NavigationDrawerKind::Standard => drawer
                .bg(theme.color(tokens::STANDARD_CONTAINER_COLOR))
                .shadow(theme.shadows(Elevation::from_dp(tokens::STANDARD_CONTAINER_ELEVATION))),
            NavigationDrawerKind::Modal => {
                // 終端側（右）の角だけを丸める。
                let corner = match theme.corner(tokens::CONTAINER_SHAPE) {
                    CornerRadius::Fixed(radius) => radius,
                    CornerRadius::Full => px(tokens::CONTAINER_WIDTH / 2.0),
                };
                drawer
                    .bg(theme.color(tokens::MODAL_CONTAINER_COLOR))
                    .rounded_r(corner)
                    .shadow(theme.shadows(Elevation::from_dp(tokens::MODAL_CONTAINER_ELEVATION)))
            }
        };

        let Some(focus) = focus else {
            return drawer.into_any_element();
        };
        let drawer = modal_container(drawer, &focus, dismiss.clone(), cx);
        let layer = scrim(
            ElementId::NamedChild(self.id.into(), "scrim".into()),
            dismiss,
            cx,
        )
        .flex()
        .justify_start()
        .child(drawer);
        window_layer(layer, MODAL_PRIORITY, window).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Role;

    #[test]
    fn modal_drawers_are_dialogs() {
        let standard = NavigationDrawer::new("d", "メイン");
        assert_eq!(standard.a11y_node().role, Role::Navigation);
        let modal = NavigationDrawer::new("d", "メイン").kind(NavigationDrawerKind::Modal);
        assert_eq!(modal.a11y_node().role, Role::Dialog);
        assert!(modal.a11y_node().modal);
        assert!(!standard.a11y_node().modal);
    }
}
