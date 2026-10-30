//! gallery の各ページ。

pub mod buttons;
pub mod containment;
pub mod custom_theme;
pub mod expressive;
pub mod inputs;
pub mod navigation;
pub mod overlays;
pub mod scrollbar;
pub mod selection;
pub mod theme;

use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Role, SharedString, Window, div,
    prelude::*, px,
};
use gpui_headless::{focus_handle, is_focus_visible};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{Md3ThemeExt, Shape, TypeScale};
use md3_components::{FocusRing, Icon, IconName, StateLayer, StyledExt, Text};

/// 状態の見本の 1 行（行の名前と、Enabled / Disabled の列など）。
pub fn state_row(label: impl Into<SharedString>, cells: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_4()
        .child(div().w(px(120.0)).flex_none().child(
            Text::new(TypeScale::LabelLarge, label.into()).color(ColorRole::OnSurfaceVariant),
        ))
        .children(
            cells
                .into_iter()
                .map(|cell| div().w(px(150.0)).flex_none().child(cell)),
        )
}

/// 状態の見本の見出し行。
pub fn state_header() -> impl IntoElement {
    state_row(
        "",
        ["Enabled", "Disabled"]
            .map(|name| {
                Text::new(TypeScale::LabelMedium, name)
                    .color(ColorRole::OnSurfaceVariant)
                    .into_any_element()
            })
            .into(),
    )
}

/// 見出し付きのまとまり。
#[derive(IntoElement)]
pub struct Section {
    title: SharedString,
    body: gpui::Div,
}

impl Section {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: div().flex().flex_col().gap_3(),
        }
    }
}

impl ParentElement for Section {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Section {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Text::new(TypeScale::TitleLarge, self.title))
            .child(self.body)
    }
}

/// 選択肢を 1 行に並べる。
pub fn option_row(
    label: &str,
    options: impl IntoIterator<Item = gpui::AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_2()
        .child(div().w(px(96.0)).child(
            Text::new(TypeScale::LabelLarge, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        ))
        .children(options)
}

/// 切り替え UI の 1 つの選択肢（基礎部品を組み合わせた、gallery 専用の選択ボタン）。
pub struct Choice {
    id: SharedString,
    label: SharedString,
    selected: bool,
    icon: Option<IconName>,
}

impl Choice {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        selected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected,
            icon: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn render(
        self,
        window: &mut Window,
        cx: &mut App,
        on_select: impl Fn(&mut App) + 'static,
    ) -> gpui::AnyElement {
        let handle = focus_handle(self.id.clone(), window, cx);
        let focused = is_focus_visible(&handle, window, cx);
        let reveal = gpui_headless::reveal_on_focus(
            &self.id.clone().into(),
            handle.clone(),
            window,
            cx,
            px(
                md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_THICKNESS
                    + md3_components::theme::md3_tokens::state::FOCUS_INDICATOR_OUTER_OFFSET,
            ),
        );
        let theme = cx.md3();
        let radius = theme.corner(Shape::Full);
        let (container, content) = if self.selected {
            (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            )
        } else {
            (ColorRole::SurfaceContainerLow, ColorRole::OnSurface)
        };
        let group = self.id.clone();
        div()
            .id(self.id.clone())
            .group(group.clone())
            .relative()
            .track_focus(&handle)
            .child(reveal)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .aria_selected(self.selected)
            .h(theme.dense(40.0))
            .px_4()
            .flex()
            .items_center()
            .gap_2()
            .bg(theme.color(container))
            .text_color(theme.color(content))
            .border_1()
            .border_color(theme.color(if self.selected {
                container
            } else {
                ColorRole::OutlineVariant
            }))
            .corner_radius(radius)
            .cursor_pointer()
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(px(18.0)).color(content))
            })
            .when(self.selected && self.icon.is_none(), |this| {
                this.child(Icon::new(IconName::Check).size(px(18.0)).color(content))
            })
            .child(Text::new(TypeScale::LabelLarge, self.label))
            .child(
                StateLayer::new(format!("{group}-layer"), group, content)
                    .radius(radius)
                    .focused(focused),
            )
            .child(FocusRing::new(radius).visible(focused))
            .on_click(move |_, _, cx| on_select(cx))
            .into_any_element()
    }
}
