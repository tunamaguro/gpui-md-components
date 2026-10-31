//! Tabs（Primary / Secondary、固定幅）。

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    divider, primary_navigation_tab as primary, secondary_navigation_tab as secondary,
};
use md3_theme::md3_tokens::{layout, state};
use md3_theme::{CornerRadius, Md3ThemeExt, TypeScale};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// タブの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// 画面の主な区分（選択の印は内容の幅で、角が丸い）。アイコンはラベルの上に置く。
    #[default]
    Primary,
    /// 区分の中のさらに細かい区分（選択の印はタブの幅いっぱい）。アイコンはラベルの左に置く。
    Secondary,
}

/// タブ 1 つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    label: SharedString,
    name: gpui_headless::AccessibleName,
    icon: Option<IconName>,
    disabled: bool,
}

impl Tab {
    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            name: gpui_headless::AccessibleName::new(label.clone()),
            label,
            icon: None,
            disabled: false,
        }
    }

    accessible_name!();

    /// アイコンを付ける。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 種類ごとのトークン。
struct TabsStyle {
    active_label: ColorRole,
    inactive_label: ColorRole,
    active_icon: ColorRole,
    inactive_icon: ColorRole,
    label_font: TypeScale,
    container: ColorRole,
    height: f32,
}

impl TabsVariant {
    fn style(self) -> TabsStyle {
        match self {
            Self::Primary => TabsStyle {
                active_label: primary::ACTIVE_LABEL_TEXT_COLOR,
                inactive_label: primary::INACTIVE_LABEL_TEXT_COLOR,
                active_icon: primary::ACTIVE_ICON_COLOR,
                inactive_icon: primary::INACTIVE_ICON_COLOR,
                label_font: primary::LABEL_TEXT_FONT,
                container: primary::CONTAINER_COLOR,
                height: primary::CONTAINER_HEIGHT,
            },
            Self::Secondary => TabsStyle {
                active_label: secondary::ACTIVE_LABEL_TEXT_COLOR,
                inactive_label: secondary::INACTIVE_LABEL_TEXT_COLOR,
                active_icon: secondary::ACTIVE_ICON_COLOR,
                inactive_icon: secondary::INACTIVE_ICON_COLOR,
                label_font: secondary::LABEL_TEXT_FONT,
                container: secondary::CONTAINER_COLOR,
                height: secondary::CONTAINER_HEIGHT,
            },
        }
    }
}

/// タブの並び（全タブが同じ幅で、並び全体の幅を分け合う）。
///
/// Tab では選択中のタブで 1 回だけ止まり、←→ で隣のタブを選ぶ（フォーカスと選択が一緒に動く）。
/// Home / End は端のタブ。タブの中身（パネル）は呼び出し側が選択に合わせて描く。
#[derive(IntoElement)]
pub struct Tabs {
    inner: gpui_headless::Tabs,
    variant: TabsVariant,
    tabs: Vec<Tab>,
}
impl Tabs {
    /// ID と選択中のタブの位置から作る。
    pub fn new(id: impl Into<ElementId>, selected: usize) -> Self {
        Self {
            inner: gpui_headless::Tabs::new(id, selected)
                .reveal_margin(crate::focus::reveal_margin())
                .group_prefix("md3"),
            variant: TabsVariant::default(),
            tabs: Vec::new(),
        }
    }
    delegate_accessible_name!(inner);

    /// 種類を変える。
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }
    /// タブを足す。
    pub fn tab(mut self, tab: Tab) -> Self {
        self.inner = self.inner.tab_item(
            gpui_headless::TabOption::new(tab.label.clone(), tab.disabled).name(tab.name.clone()),
        );
        self.tabs.push(tab);
        self
    }
    /// タブをまとめて足す。
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab>) -> Self {
        for tab in tabs {
            self = self.tab(tab);
        }
        self
    }
    /// タブを選んだときの処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.variant.style();
        let theme = cx.md3().clone();
        // アイコンとラベルの両方を持つ Primary のタブがあれば、全タブを高くする。
        let tall =
            self.variant == TabsVariant::Primary && self.tabs.iter().any(|tab| tab.icon.is_some());
        let height = if tall {
            layout::TAB_LARGE_HEIGHT
        } else {
            style.height
        };
        let indicator_color = theme.color(primary::ACTIVE_INDICATOR_COLOR);
        let indicator_height = px(primary::ACTIVE_INDICATOR_HEIGHT);

        let container_color = theme.color(style.container);
        let divider_color = theme.color(divider::COLOR);
        let variant = self.variant;
        let tabs = self.tabs;
        let row = self
            .inner
            .before(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(divider::THICKNESS))
                    .bg(divider_color),
            )
            .build(window, cx, move |item, _, _| {
                let gpui_headless::TabItem {
                    base,
                    state: control,
                    index,
                    selected,
                } = item;
                let tab = &tabs[index];
                let color = |role: ColorRole| {
                    if tab.disabled {
                        theme
                            .color(ColorRole::OnSurface)
                            .opacity(state::DISABLED_CONTENT_OPACITY)
                    } else {
                        theme.color(role)
                    }
                };
                let label_color = color(if selected {
                    style.active_label
                } else {
                    style.inactive_label
                });
                let icon_color = color(if selected {
                    style.active_icon
                } else {
                    style.inactive_icon
                });
                let icon = tab.icon.map(|icon| {
                    Icon::new(icon)
                        .size(px(primary::ICON_SIZE))
                        .tint(icon_color)
                });
                let label = div()
                    .type_style(&theme.text_style(style.label_font))
                    .text_color(label_color)
                    .text_center()
                    .child(tab.label.clone());
                let content = match variant {
                    TabsVariant::Primary => div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .children(icon)
                        .child(label),
                    TabsVariant::Secondary => div()
                        .flex()
                        .items_center()
                        .gap(px(layout::TAB_LEADING_ICON_SPACE))
                        .children(icon)
                        .child(label),
                };
                // Primary の選択の印は内容の幅（最小 24dp）で、上側の角を丸める。
                let content = match variant {
                    TabsVariant::Primary => div()
                        .relative()
                        .h_full()
                        .min_w(px(layout::TAB_INDICATOR_MIN_WIDTH))
                        .flex()
                        .flex_col()
                        .justify_center()
                        .items_center()
                        .child(content)
                        .when(selected, |this| {
                            let radius = match primary::ACTIVE_INDICATOR_SHAPE {
                                md3_theme::md3_tokens::shape::CornerSize::Fixed(dp) => px(dp),
                                _ => px(0.0),
                            };
                            this.child(
                                div()
                                    .absolute()
                                    .bottom_0()
                                    .left_0()
                                    .right_0()
                                    .h(indicator_height)
                                    .rounded_t(radius)
                                    .bg(indicator_color),
                            )
                        }),
                    TabsVariant::Secondary => content,
                };
                let element = base
                    .flex_1()
                    .min_w_0()
                    .min_h(px(height))
                    .flex()
                    .justify_center()
                    .items_center()
                    .px(px(layout::TAB_HORIZONTAL_SPACE))
                    .child(content)
                    .when(selected && variant == TabsVariant::Secondary, |this| {
                        this.child(
                            div()
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .right_0()
                                .h(indicator_height)
                                .bg(indicator_color),
                        )
                    })
                    .child(
                        StateLayer::new(
                            SharedString::from(format!("{}-layer", control.group)),
                            control.group.clone(),
                            if selected {
                                style.active_label
                            } else {
                                style.inactive_label
                            },
                        )
                        .focused(control.focus_visible)
                        .disabled(tab.disabled),
                    )
                    .child(
                        FocusRing::new(CornerRadius::Fixed(px(0.0)))
                            .inward()
                            .visible(control.focus_visible),
                    );
                element.into_any_element()
            });
        row.relative().w_full().flex().bg(container_color)
    }
}
