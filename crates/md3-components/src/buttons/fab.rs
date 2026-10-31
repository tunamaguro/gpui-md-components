//! FAB（Floating action button）と Extended FAB。

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{
    extended_fab_primary, fab_baseline, fab_large, fab_primary_container, fab_secondary_container,
    fab_small,
};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt, Shape};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// FAB の大きさ。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabSize {
    /// 40dp。
    Small,
    /// 56dp。
    #[default]
    Regular,
    /// 96dp。
    Large,
}

impl FabSize {
    /// すべての大きさ。
    pub const ALL: [FabSize; 3] = [FabSize::Small, FabSize::Regular, FabSize::Large];

    fn container(self) -> f32 {
        match self {
            FabSize::Small => fab_small::CONTAINER_WIDTH,
            FabSize::Regular => fab_baseline::CONTAINER_WIDTH,
            FabSize::Large => fab_large::CONTAINER_WIDTH,
        }
    }

    fn icon(self) -> f32 {
        match self {
            FabSize::Small => fab_small::ICON_SIZE,
            FabSize::Regular => fab_baseline::ICON_SIZE,
            FabSize::Large => fab_large::ICON_SIZE,
        }
    }

    fn shape(self) -> Shape {
        match self {
            FabSize::Small => fab_small::CONTAINER_SHAPE,
            FabSize::Regular => fab_baseline::CONTAINER_SHAPE,
            FabSize::Large => fab_large::CONTAINER_SHAPE,
        }
    }
}

/// FAB の色。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FabColor {
    /// `PrimaryContainer`（既定）。
    #[default]
    PrimaryContainer,
    /// `SecondaryContainer`。
    SecondaryContainer,
}

impl FabColor {
    /// すべての色。
    pub const ALL: [FabColor; 2] = [FabColor::PrimaryContainer, FabColor::SecondaryContainer];

    /// 背景と前景のロール。
    pub(crate) fn roles(self) -> (ColorRole, ColorRole) {
        match self {
            FabColor::PrimaryContainer => (
                fab_primary_container::CONTAINER_COLOR,
                fab_primary_container::ICON_COLOR,
            ),
            FabColor::SecondaryContainer => (
                fab_secondary_container::CONTAINER_COLOR,
                fab_secondary_container::ICON_COLOR,
            ),
        }
    }
}

/// FAB の共通部分。
struct FabParts {
    inner: gpui_headless::Button,
    color: FabColor,
}

impl FabParts {
    fn new(id: ElementId, label: SharedString) -> Self {
        Self {
            inner: gpui_headless::Button::new(id.clone(), label)
                .group(format!("md3-{id}"))
                .reveal_margin(crate::focus::reveal_margin()),
            color: FabColor::default(),
        }
    }

    /// 背景・影・状態レイヤーを付けた土台と前景色のロールを返す。
    fn render(
        self,
        shape: Shape,
        window: &mut Window,
        cx: &mut App,
        content: impl FnOnce(gpui::Stateful<gpui::Div>, ColorRole) -> gpui::Stateful<gpui::Div>,
    ) -> gpui::Stateful<gpui::Div> {
        let (base, control) = self.inner.build(window, cx);
        let theme = cx.md3();
        let (container, content_role) = self.color.roles();
        let radius = theme.corner(shape);
        let resting = theme.shadows(Elevation::from_dp(
            fab_primary_container::CONTAINER_ELEVATION,
        ));
        let hovered = theme.shadows(Elevation::from_dp(
            fab_primary_container::HOVERED_CONTAINER_ELEVATION,
        ));
        // 内容の上に浮くので、下の要素にクリックを渡さない。ホイールは下のスクロール領域へ通す。
        let fab = base
            .block_mouse_except_scroll()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.color(container))
            .text_color(theme.color(content_role))
            .corner_radius(radius);
        let fab = fab
            .shadow(resting)
            .hover(move |s| s.shadow(hovered.clone()));
        content(fab, content_role)
            .child(
                StateLayer::new(
                    SharedString::from(format!("{}-layer", control.group)),
                    control.group.clone(),
                    content_role,
                )
                .radius(radius)
                .focused(control.focus_visible),
            )
            .child(FocusRing::new(radius).visible(control.focus_visible))
    }
}

/// FAB。画面の主要な操作を 1 つだけ置く。
///
/// MD3 の FAB には無効状態がないため、`disabled` は持たない。
#[derive(IntoElement)]
pub struct Fab {
    parts: FabParts,
    icon: IconName,
    size: FabSize,
}

impl Fab {
    /// ID とアイコンから作る。
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            parts: FabParts::new(id.into(), SharedString::default()),
            icon,
            size: FabSize::default(),
        }
    }

    delegate_accessible_name!(parts.inner);

    /// 大きさを変える。
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    /// 色を変える。
    pub fn color(mut self, color: FabColor) -> Self {
        self.parts.color = color;
        self
    }

    /// 押されたときの処理。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.parts.inner = self.parts.inner.on_click(handler);
        self
    }
}

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = self.size;
        let icon = self.icon;
        self.parts.render(size.shape(), window, cx, |fab, content| {
            fab.size(px(size.container()))
                .child(Icon::new(icon).size(px(size.icon())).color(content))
        })
    }
}

/// ラベル付きの FAB。
#[derive(IntoElement)]
pub struct ExtendedFab {
    parts: FabParts,
    icon: Option<IconName>,
    label: SharedString,
}

impl ExtendedFab {
    /// ID とラベルから作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            parts: FabParts::new(id.into(), label.clone()),
            icon: None,
            label,
        }
    }

    delegate_accessible_name!(parts.inner);

    /// ラベルの前にアイコンを置く。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 色を変える。
    pub fn color(mut self, color: FabColor) -> Self {
        self.parts.color = color;
        self
    }

    /// 押されたときの処理。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.parts.inner = self.parts.inner.on_click(handler);
        self
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let text_style = cx.md3().text_style(extended_fab_primary::LABEL_TEXT_FONT);
        let icon = self.icon;
        let label = self.label;
        self.parts.render(
            extended_fab_primary::CONTAINER_SHAPE,
            window,
            cx,
            |fab, content| {
                let (start, end) = if icon.is_some() {
                    (
                        layout::EXTENDED_FAB_START_ICON_SPACE,
                        layout::EXTENDED_FAB_TEXT_SPACE,
                    )
                } else {
                    (
                        layout::EXTENDED_FAB_TEXT_SPACE,
                        layout::EXTENDED_FAB_TEXT_SPACE,
                    )
                };
                fab.min_h(px(extended_fab_primary::CONTAINER_HEIGHT))
                    .min_w(px(layout::EXTENDED_FAB_MIN_WIDTH))
                    .pl(px(start))
                    .pr(px(end))
                    .gap(px(layout::EXTENDED_FAB_END_ICON_SPACE))
                    .type_style(&text_style)
                    .when_some(icon, |this, icon| {
                        this.child(
                            Icon::new(icon)
                                .size(px(extended_fab_primary::ICON_SIZE))
                                .color(content),
                        )
                    })
                    .child(label)
            },
        )
    }
}
