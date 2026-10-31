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

use super::item::{ChangeHandler, NavigationItem, navigation};
use crate::{FocusRing, Icon, StateLayer, StyledExt};

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
    base: gpui::Stateful<gpui::Div>,
    control: gpui_headless::ControlState,
    item: &NavigationItem,
    selected: bool,
    style: &ItemStyle,
) -> gpui::Stateful<gpui::Div> {
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
    base.flex()
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
        )
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
    label: Option<SharedString>,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
}

impl NavigationBar {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            items: Vec::new(),
            selected: None,
            on_change: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    destination_builders!();
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
        let nav = navigation(
            self.id,
            self.label,
            &self.items,
            self.selected,
            self.on_change,
            false,
        );
        let theme = cx.md3().clone();
        let nav = nav.build(window, cx, |item, _, _| {
            let source = &self.items[item.index];
            vertical_item(&theme, item.base, item.state, source, item.selected, &style)
                .into_any_element()
        });
        let items = nav.items;
        let theme = cx.md3();
        nav.base
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
    label: Option<SharedString>,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    header: Option<AnyElement>,
}

impl NavigationRail {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            items: Vec::new(),
            selected: None,
            on_change: None,
            header: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 項目の上に置く要素（FAB、メニューボタンなど）。
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    destination_builders!();
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
        let nav = navigation(
            self.id,
            self.label,
            &self.items,
            self.selected,
            self.on_change,
            true,
        );
        let theme = cx.md3().clone();
        let nav = nav.build(window, cx, |item, _, _| {
            let source = &self.items[item.index];
            vertical_item(&theme, item.base, item.state, source, item.selected, &style)
                .into_any_element()
        });
        let items = nav.items;
        let theme = cx.md3();
        let space: Pixels = px(layout::NAVIGATION_RAIL_VERTICAL_SPACE);
        nav.base
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
