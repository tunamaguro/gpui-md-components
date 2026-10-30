// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/{NavigationBar,NavigationRail}.kt @ b8256a52
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! NavigationBar と NavigationRail（アイコンを丸い選択の印で囲み、ラベルを下に置く項目の並び）。

use gpui::{
    AnimationExt, AnyElement, App, ElementId, IntoElement, Pixels, RenderOnce, SharedString,
    SpringAnimation, Window, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    navigation_bar, navigation_bar_vertical_item, navigation_rail_baseline_item,
    navigation_rail_collapsed, navigation_rail_color, navigation_rail_vertical_item,
};
use md3_theme::md3_tokens::{layout, state};
use md3_theme::{Md3ThemeExt, MotionToken, Theme};

use super::item::{
    ChangeHandler, Destinations, NavigationItem, destination_a11y_node, navigation_a11y_node,
};
use crate::control::control;
use crate::{FocusRing, Icon, StateLayer, StyledExt};
use gpui_headless::A11yExt;

/// 縦に積む項目の寸法と色。
struct ItemStyle {
    indicator_width: f32,
    indicator_height: f32,
    icon_size: f32,
    label_space: f32,
    top_space: f32,
    bottom_space: f32,
}

/// 項目 1 つ（選択の印、アイコン、ラベル）。
#[allow(clippy::too_many_arguments)]
fn vertical_item(
    theme: &Theme,
    id: &ElementId,
    item: &NavigationItem,
    selected: bool,
    tab_stop: bool,
    style: &ItemStyle,
    window: &mut Window,
    cx: &mut App,
) -> (gpui::Stateful<gpui::Div>, bool) {
    let (base, control) = control(id, item.disabled, tab_stop, window, cx);
    let (icon_role, label_role) = if selected {
        (
            navigation_rail_color::ITEM_ACTIVE_ICON,
            navigation_rail_color::ITEM_ACTIVE_LABEL_TEXT,
        )
    } else {
        (
            navigation_rail_color::ITEM_INACTIVE_ICON,
            navigation_rail_color::ITEM_INACTIVE_LABEL_TEXT,
        )
    };
    let disabled = |role: ColorRole| {
        if item.disabled {
            theme
                .color(ColorRole::OnSurface)
                .opacity(state::DISABLED_CONTENT_OPACITY)
        } else {
            theme.color(role)
        }
    };
    let layer_role = if selected {
        navigation_rail_color::ITEM_ACTIVE_HOVERED_STATE_LAYER
    } else {
        navigation_rail_color::ITEM_INACTIVE_HOVERED_STATE_LAYER
    };
    let pill = theme.corner(navigation_rail_baseline_item::ACTIVE_INDICATOR_SHAPE);
    let width = style.indicator_width;
    let progress = if selected { 1.0_f32 } else { 0.0 };
    let background = div()
        .absolute()
        .h_full()
        .corner_radius(pill)
        .bg(theme.color(navigation_rail_color::ITEM_ACTIVE_INDICATOR))
        .with_spring(
            "navigation-indicator-width",
            SpringAnimation::new(theme.spring(MotionToken::FastSpatial)).to(progress),
            move |element, progress| {
                element
                    .w(px(width * progress.max(0.0)))
                    .left(px(width * (1.0 - progress.max(0.0)) / 2.0))
            },
        )
        .with_spring(
            "navigation-indicator-opacity",
            SpringAnimation::new(theme.spring(MotionToken::DefaultEffects)).to(progress),
            |element, opacity| {
                element.map_element(|element| element.opacity(opacity.clamp(0.0, 1.0)))
            },
        );
    let indicator = div()
        .relative()
        .flex_none()
        .w(px(style.indicator_width))
        .h(px(style.indicator_height))
        .flex()
        .items_center()
        .justify_center()
        .corner_radius(pill)
        .child(background)
        .child(
            Icon::new(item.icon)
                .size(px(style.icon_size))
                .tint(disabled(icon_role)),
        )
        .child(
            StateLayer::new(
                SharedString::from(format!("{}-layer", control.group)),
                control.group.clone(),
                layer_role,
            )
            .radius(pill)
            .focused(control.focus_visible)
            .disabled(item.disabled),
        )
        .child(FocusRing::new(pill).visible(control.focus_visible));
    let element = base
        .flex()
        .flex_col()
        .items_center()
        .pt(px(style.top_space))
        .pb(px(style.bottom_space))
        .gap(px(style.label_space))
        .child(indicator)
        .child(
            div()
                .type_style(&theme.text_style(navigation_bar::LABEL_TEXT_FONT))
                .text_color(disabled(label_role))
                .whitespace_nowrap()
                .child(item.label.clone()),
        );
    (element, item.disabled)
}

/// 項目の並びを作る共通部分。
#[allow(clippy::too_many_arguments)]
fn destinations(
    id: &ElementId,
    items: &[NavigationItem],
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    vertical: bool,
    style: &ItemStyle,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement> {
    let destinations = Destinations::new(id, items, selected, window, cx);
    let theme = cx.md3().clone();
    let count = items.len();
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = selected == Some(index);
            let (element, disabled) = vertical_item(
                &theme,
                &destinations.ids[index],
                item,
                is_selected,
                destinations.tab_target == Some(index),
                style,
                window,
                cx,
            );
            let mut element = element.a11y(destination_a11y_node(item, index, count, is_selected));
            if !disabled {
                element = element.on_key_down(destinations.on_key(index, vertical));
                if let Some(on_change) = on_change.clone() {
                    element = element.on_click(move |_, window, cx| on_change(index, window, cx));
                }
            }
            element.into_any_element()
        })
        .collect()
}

macro_rules! destination_builders {
    () => {
        /// 行き先を足す。
        pub fn item(mut self, item: NavigationItem) -> Self {
            self.items.push(item);
            self
        }

        /// 行き先をまとめて足す。
        pub fn items(mut self, items: impl IntoIterator<Item = NavigationItem>) -> Self {
            self.items.extend(items);
            self
        }

        /// 選択中の行き先（位置）。
        pub fn selected(mut self, index: Option<usize>) -> Self {
            self.selected = index;
            self
        }

        /// 行き先を選んだとき（クリック、Enter、Space）の処理。
        pub fn on_change(
            mut self,
            handler: impl Fn(usize, &mut Window, &mut App) + 'static,
        ) -> Self {
            self.on_change = Some(std::rc::Rc::new(handler));
            self
        }
    };
}
pub(crate) use destination_builders;

/// ナビゲーションバー（ウィンドウの下端、幅の狭いウィンドウ向け。3〜5 個の行き先）。
///
/// Tab では選択中の項目で 1 回だけ止まり、←→ / Home / End でフォーカスを移して Enter / Space で選ぶ。
#[derive(IntoElement)]
pub struct NavigationBar {
    id: ElementId,
    label: SharedString,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
}

impl NavigationBar {
    /// ID と名前（読み上げられる）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            on_change: None,
        }
    }

    destination_builders!();

    /// アクセシビリティツリーに出す情報（役割 Navigation。項目は Tab）。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        navigation_a11y_node(&self.label)
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = ItemStyle {
            indicator_width: navigation_bar_vertical_item::ACTIVE_INDICATOR_WIDTH,
            indicator_height: navigation_bar_vertical_item::ACTIVE_INDICATOR_HEIGHT,
            icon_size: navigation_bar_vertical_item::ICON_SIZE,
            label_space: navigation_bar::ITEM_ACTIVE_INDICATOR_ICON_LABEL_SPACE,
            top_space: layout::NAVIGATION_BAR_INDICATOR_VERTICAL_OFFSET,
            bottom_space: 0.0,
        };
        let node = self.a11y_node();
        let items = destinations(
            &self.id,
            &self.items,
            self.selected,
            self.on_change,
            false,
            &style,
            window,
            cx,
        );
        let theme = cx.md3();
        div()
            .id(self.id)
            .a11y(node)
            .w_full()
            .h(px(navigation_bar::TALL_CONTAINER_HEIGHT))
            .flex_none()
            .flex()
            .gap(px(layout::NAVIGATION_BAR_ITEM_HORIZONTAL_SPACE))
            .px(px(layout::NAVIGATION_BAR_ITEM_HORIZONTAL_SPACE))
            .bg(theme.color(navigation_bar::CONTAINER_COLOR))
            .children(
                items
                    .into_iter()
                    .map(|item| div().flex_1().min_w_0().h_full().child(item)),
            )
    }
}

/// ナビゲーションレール（ウィンドウの左端、中くらいの幅のウィンドウ向け）。
///
/// 先頭に FAB などを置ける（[`NavigationRail::header`]）。キー操作は ↑↓ を使うほかは [`NavigationBar`] と同じ。
#[derive(IntoElement)]
pub struct NavigationRail {
    id: ElementId,
    label: SharedString,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    header: Option<AnyElement>,
}

impl NavigationRail {
    /// ID と名前（読み上げられる）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            on_change: None,
            header: None,
        }
    }

    /// 項目の上に置く要素（FAB、メニューボタンなど）。
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    destination_builders!();

    /// アクセシビリティツリーに出す情報（役割 Navigation。項目は Tab）。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        navigation_a11y_node(&self.label)
    }
}

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = ItemStyle {
            indicator_width: navigation_rail_vertical_item::ACTIVE_INDICATOR_WIDTH,
            indicator_height: navigation_rail_vertical_item::ACTIVE_INDICATOR_HEIGHT,
            icon_size: navigation_rail_baseline_item::ICON_SIZE,
            label_space: navigation_rail_vertical_item::ICON_LABEL_SPACE,
            top_space: layout::NAVIGATION_RAIL_ITEM_VERTICAL_SPACE,
            bottom_space: layout::NAVIGATION_RAIL_ITEM_VERTICAL_SPACE,
        };
        let node = self.a11y_node();
        let items = destinations(
            &self.id,
            &self.items,
            self.selected,
            self.on_change,
            true,
            &style,
            window,
            cx,
        );
        let theme = cx.md3();
        let space: Pixels = px(layout::NAVIGATION_RAIL_VERTICAL_SPACE);
        div()
            .id(self.id)
            .a11y(node)
            .w(px(navigation_rail_collapsed::NARROW_CONTAINER_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .py(space)
            .gap(space)
            .bg(theme.color(navigation_rail_collapsed::CONTAINER_COLOR))
            .when_some(self.header, |this, header| {
                this.child(
                    div()
                        .pb(px(layout::NAVIGATION_RAIL_HEADER_SPACE))
                        .child(header),
                )
            })
            .children(items.into_iter().map(|item| div().w_full().child(item)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Role;

    #[test]
    fn bars_are_navigation_landmarks() {
        let bar = NavigationBar::new("n", "メイン");
        let node = bar.a11y_node();
        assert_eq!(node.role, Role::Navigation);
        assert_eq!(node.label.as_deref(), Some("メイン"));
    }
}
