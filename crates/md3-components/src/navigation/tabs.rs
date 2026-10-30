//! Tabs（Primary / Secondary、固定幅）。

use gpui_headless::focus_handle;
use std::rc::Rc;

use gpui::{
    App, ElementId, FocusHandle, IntoElement, KeyDownEvent, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    divider, primary_navigation_tab as primary, secondary_navigation_tab as secondary,
};
use md3_theme::md3_tokens::{layout, state};
use md3_theme::{CornerRadius, Md3ThemeExt, TypeScale};

use super::item::ChangeHandler;
use crate::control::control;
use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};
use gpui_headless::step;
use gpui_headless::{A11yExt, A11yNode};

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
    icon: Option<IconName>,
    disabled: bool,
}

impl Tab {
    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

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
    id: ElementId,
    label: SharedString,
    variant: TabsVariant,
    tabs: Vec<Tab>,
    selected: usize,
    on_change: Option<ChangeHandler>,
}

impl Tabs {
    /// ID、名前（読み上げられる）、選択中のタブの位置から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: TabsVariant::default(),
            tabs: Vec::new(),
            selected,
            on_change: None,
        }
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }

    /// タブを足す。
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// タブをまとめて足す。
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    /// タブを選んだとき（クリック、矢印キー、Home / End）の処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// アクセシビリティツリーに出す情報（役割 TabList）。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::TabList).label(self.label.clone())
    }

    /// タブ `index` のアクセシビリティ情報（役割 Tab）。
    pub fn tab_a11y_node(&self, index: usize) -> Option<A11yNode> {
        self.tabs.get(index).map(|tab| {
            A11yNode::new(Role::Tab)
                .label(tab.label.clone())
                .selected(index == self.selected)
                .position(index, self.tabs.len())
                .disabled(tab.disabled)
        })
    }

    fn tab_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(self.id.clone().into(), format!("{index}").into())
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.variant.style();
        let theme = cx.md3().clone();
        let count = self.tabs.len();
        let enabled: Rc<[bool]> = self.tabs.iter().map(|tab| !tab.disabled).collect();
        let tab_target = Some(self.selected)
            .filter(|&index| enabled.get(index).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&e| e));
        let handles: Rc<[FocusHandle]> = (0..count)
            .map(|index| focus_handle(self.tab_id(index), window, cx))
            .collect();
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

        let mut row = div()
            .id(self.id.clone())
            .a11y(self.a11y_node())
            .relative()
            .w_full()
            .flex()
            .bg(theme.color(style.container))
            // 並び全体の下の区切り線（Compose の TabRow の `HorizontalDivider`）。選択の印をその上に描く。
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(divider::THICKNESS))
                    .bg(theme.color(divider::COLOR)),
            );
        for (index, tab) in self.tabs.iter().enumerate() {
            let selected = index == self.selected;
            let (base, control) = control(
                &self.tab_id(index),
                tab.disabled,
                tab_target == Some(index),
                window,
                cx,
            );
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
                .whitespace_nowrap()
                .child(tab.label.clone());
            let content = match self.variant {
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
            let content = match self.variant {
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
            let mut element = base
                .a11y(
                    self.tab_a11y_node(index)
                        .unwrap_or_else(|| A11yNode::new(Role::Tab)),
                )
                .flex_1()
                .min_w_0()
                .h(px(height))
                .flex()
                .justify_center()
                .items_center()
                .px(px(layout::TAB_HORIZONTAL_SPACE))
                .child(content)
                .when(selected && self.variant == TabsVariant::Secondary, |this| {
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
            if !tab.disabled
                && let Some(on_change) = self.on_change.clone()
            {
                let on_click = on_change.clone();
                element = element.on_click(move |_, window, cx| on_click(index, window, cx));
                let handles = handles.clone();
                let enabled = enabled.clone();
                element = element.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let next = match event.keystroke.key.as_str() {
                        "right" => step(&enabled, index, true),
                        "left" => step(&enabled, index, false),
                        "home" => enabled.iter().position(|&e| e),
                        "end" => enabled.iter().rposition(|&e| e),
                        _ => return,
                    };
                    let Some(next) = next else { return };
                    cx.stop_propagation();
                    if let Some(handle) = handles.get(next) {
                        window.focus(handle, cx);
                    }
                    on_change(next, window, cx);
                });
            }
            row = row.child(element);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_are_a_tab_list() {
        let tabs = Tabs::new("t", "表示", 1)
            .tab(Tab::new("写真"))
            .tab(Tab::new("アルバム"));
        assert_eq!(tabs.a11y_node().role, Role::TabList);
        let tab = tabs.tab_a11y_node(1).expect("tab");
        assert_eq!(tab.role, Role::Tab);
        assert_eq!(tab.selected, Some(true));
        assert_eq!(tab.position, Some((2, 2)));
        assert_eq!(tabs.tab_a11y_node(0).and_then(|t| t.selected), Some(false));
    }
}
