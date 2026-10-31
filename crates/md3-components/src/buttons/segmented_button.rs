//! SegmentedButton（単一選択・複数選択）。

use gpui::{
    App, Corners, ElementId, IntoElement, RenderOnce, SharedString, Window, prelude::*, px,
};
use md3_theme::md3_tokens::component::outlined_segmented_button as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// SegmentedButton の 1 項目。
#[derive(Clone, Debug)]
pub struct Segment {
    label: SharedString,
    name: gpui_headless::AccessibleName,
    icon: Option<IconName>,
    disabled: bool,
}

impl Segment {
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

    /// アイコンを付ける（選択中はチェックに置き換わる）。
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

/// 2〜5 個の選択肢を並べたボタン。
///
/// 単一選択（既定）は RadioGroup として振る舞い、矢印キーで選択が移る。Tab で止まるのは選択中の項目だけ。
/// 複数選択（[`Self::multi_select`]）は各項目がトグルボタンで、矢印キーでフォーカスだけが移る。
#[derive(IntoElement)]
pub struct SegmentedButton {
    inner: gpui_headless::SegmentedButton,
    segments: Vec<Segment>,
}
impl SegmentedButton {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            inner: gpui_headless::SegmentedButton::new(id)
                .reveal_margin(crate::focus::reveal_margin())
                .group_prefix("md3"),
            segments: Vec::new(),
        }
    }
    delegate_accessible_name!(inner);

    /// 項目を加える。
    pub fn segment(mut self, segment: Segment) -> Self {
        self.inner = self.inner.segment_item(
            gpui_headless::SegmentOption::new(segment.label.clone(), segment.disabled)
                .name(segment.name.clone()),
        );
        self.segments.push(segment);
        self
    }
    /// 選択中の項目（単一選択なら 1 つ）。
    pub fn selected(mut self, selected: impl IntoIterator<Item = usize>) -> Self {
        self.inner = self.inner.selected(selected);
        self
    }
    /// 複数選択にする。
    pub fn multi_select(mut self, multi: bool) -> Self {
        self.inner = self.inner.multi_select(multi);
        self
    }
    /// 項目が押されたときの処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }
}

impl RenderOnce for SegmentedButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.segments.len();
        let segments = self.segments;
        self.inner
            .build(window, cx, move |item, _window, cx| {
                let gpui_headless::SegmentItem {
                    base,
                    state: control,
                    index,
                    selected,
                } = item;
                let segment = &segments[index];
                let theme = cx.md3();
                let full = theme.dense(tokens::CONTAINER_HEIGHT) / 2.0;
                let (left, right) = (index == 0, index + 1 == count);
                let radius_of = |first: bool, last: bool| {
                    (
                        if first { full } else { px(0.0) },
                        if last { full } else { px(0.0) },
                    )
                };
                let (start_radius, end_radius) = radius_of(left, right);
                let content_role = if selected {
                    tokens::SELECTED_LABEL_TEXT_COLOR
                } else {
                    tokens::UNSELECTED_LABEL_TEXT_COLOR
                };
                let content = if segment.disabled {
                    theme
                        .color(tokens::DISABLED_LABEL_TEXT_COLOR)
                        .opacity(tokens::DISABLED_LABEL_TEXT_OPACITY)
                } else {
                    theme.color(content_role)
                };
                let outline = if segment.disabled {
                    theme
                        .color(tokens::DISABLED_OUTLINE_COLOR)
                        .opacity(tokens::DISABLED_OUTLINE_OPACITY)
                } else {
                    theme.color(tokens::OUTLINE_COLOR)
                };
                let icon = if selected {
                    Some(IconName::Check)
                } else {
                    segment.icon
                };
                let corners = Corners {
                    top_left: start_radius,
                    bottom_left: start_radius,
                    top_right: end_radius,
                    bottom_right: end_radius,
                };

                let item = base
                    .min_h(theme.dense(tokens::CONTAINER_HEIGHT))
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(layout::SEGMENTED_BUTTON_ICON_SPACE))
                    .px(px(layout::SEGMENTED_BUTTON_HORIZONTAL_SPACE))
                    .rounded_tl(start_radius)
                    .rounded_bl(start_radius)
                    .rounded_tr(end_radius)
                    .rounded_br(end_radius)
                    .border_y(px(tokens::OUTLINE_WIDTH))
                    .border_r(px(tokens::OUTLINE_WIDTH))
                    .when(left, |this| this.border_l(px(tokens::OUTLINE_WIDTH)))
                    .border_color(outline)
                    .text_color(content)
                    .type_style(&theme.text_style(tokens::LABEL_TEXT_FONT))
                    .when(selected && !segment.disabled, |this| {
                        this.bg(theme.color(tokens::SELECTED_CONTAINER_COLOR))
                    })
                    .when_some(icon, |this, icon| {
                        this.child(Icon::new(icon).size(px(tokens::ICON_SIZE)).tint(content))
                    })
                    .child(segment.label.clone())
                    .child(
                        StateLayer::new(
                            SharedString::from(format!("{}-layer", control.group)),
                            control.group.clone(),
                            content_role,
                        )
                        .corners(corners)
                        .focused(control.focus_visible)
                        .disabled(segment.disabled),
                    )
                    .child(
                        FocusRing::new(CornerRadius::Fixed(px(0.0)))
                            .corners(corners)
                            .visible(control.focus_visible),
                    );
                item.into_any_element()
            })
            .flex()
            .flex_none()
    }
}
