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
use super::item::{ChangeHandler, NavigationItem, navigation};
use crate::overlays::layer::scrim;
use crate::{FocusRing, Icon, StateLayer, StyledExt};
use gpui_headless::overlay::{
    DismissHandler, InitialFocus, MODAL_PRIORITY, dismiss_and_restore, modal_container,
    overlay_focus, window_layer,
};

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
    scrollbar: bool,
    id: ElementId,
    name: gpui_headless::AccessibleName,
    kind: NavigationDrawerKind,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    on_dismiss: Option<DismissHandler>,
    header: Vec<AnyElement>,
}

impl NavigationDrawer {
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
            name: gpui_headless::AccessibleName::default(),
            kind: NavigationDrawerKind::default(),
            items: Vec::new(),
            selected: None,
            on_change: None,
            on_dismiss: None,
            header: Vec::new(),
        }
    }

    accessible_name!();

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

    destination_builders!();
}

impl ParentElement for NavigationDrawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.header.extend(elements);
    }
}

impl NavigationDrawer {
    /// 支援技術にダイアログとして出すか。`Some(true)` ならモーダル。Standard は `None`。
    fn dialog_mode(&self) -> Option<bool> {
        (self.kind == NavigationDrawerKind::Modal).then_some(true)
    }
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let modal = self.kind == NavigationDrawerKind::Modal;
        let dialog = self.dialog_mode();
        let theme = cx.md3().clone();
        let pill = theme.corner(tokens::ACTIVE_INDICATOR_SHAPE);
        // 項目が多ければスクロールする。矢印・Tab で移った項目を見える範囲に入れる。
        let scroll = gpui_headless::scroll_handle(&self.id, window, cx);
        let mut nav = navigation(
            self.id.clone(),
            self.name.resolve(),
            &self.items,
            self.selected,
            self.on_change,
            true,
        );
        if let Some(modal) = dialog {
            nav = nav.dialog(modal);
        }
        let nav = nav.build(window, cx, |item, _, _| {
            let source = &self.items[item.index];
            let selected = item.selected;
            let base = item.base;
            let control = item.state;
            let (icon_role, label_role) = if selected {
                (tokens::ACTIVE_ICON_COLOR, tokens::ACTIVE_LABEL_TEXT_COLOR)
            } else {
                (
                    tokens::INACTIVE_ICON_COLOR,
                    tokens::INACTIVE_LABEL_TEXT_COLOR,
                )
            };
            let color = |role: ColorRole| {
                if source.disabled {
                    theme
                        .color(ColorRole::OnSurface)
                        .opacity(state::DISABLED_CONTENT_OPACITY)
                } else {
                    theme.color(role)
                }
            };
            let element = base
                .min_h(px(tokens::ACTIVE_INDICATOR_HEIGHT))
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
                    Icon::new(source.icon)
                        .size(px(tokens::ICON_SIZE))
                        .tint(color(icon_role)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .type_style(&theme.text_style(tokens::LABEL_TEXT_FONT))
                        .text_color(color(label_role))
                        .child(source.label.clone()),
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
                    .disabled(source.disabled),
                )
                .child(FocusRing::new(pill).visible(control.focus_visible));
            element.into_any_element()
        });
        let focus = modal.then(|| {
            let target = nav.initial_focus.clone();
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
        let items = nav.items;

        let wheel = gpui_headless::NestedScroll::new(&self.id, scroll.clone(), window, cx);
        let mut drawer = nav
            .base
            .w(px(tokens::CONTAINER_WIDTH))
            .min_w(px(layout::NAVIGATION_DRAWER_MIN_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .px(px(layout::NAVIGATION_DRAWER_ITEM_HORIZONTAL_SPACE))
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
            return gpui_headless::ScrollScope::new(scroll, drawer).into_any_element();
        };
        let drawer = gpui_headless::ScrollScope::new(
            scroll,
            modal_container(drawer, &focus, dismiss.clone(), cx),
        );
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

    #[test]
    fn modal_drawers_are_dialogs() {
        let standard = NavigationDrawer::new("d").aria_label("メイン");
        assert_eq!(standard.dialog_mode(), None);
        let modal = NavigationDrawer::new("d")
            .aria_label("メイン")
            .kind(NavigationDrawerKind::Modal);
        assert_eq!(modal.dialog_mode(), Some(true));
    }
}
