//! SegmentedButton（単一選択・複数選択）。

use std::rc::Rc;

use gpui::{
    App, Corners, ElementId, FocusHandle, IntoElement, KeyDownEvent, RenderOnce, Role,
    SharedString, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::component::outlined_segmented_button as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt};

use crate::a11y::{A11yExt, A11yNode, toggled};
use crate::control::{control, step};
use crate::{FocusRing, ForcedState, Icon, IconName, StateLayer, StyledExt, focus_handle};

/// SegmentedButton の 1 項目。
#[derive(Clone, Debug)]
pub struct Segment {
    label: SharedString,
    icon: Option<IconName>,
    disabled: bool,
}

impl Segment {
    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

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

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// 2〜5 個の選択肢を並べたボタン。
///
/// 単一選択（既定）は RadioGroup として振る舞い、矢印キーで選択が移る。Tab で止まるのは選択中の項目だけ。
/// 複数選択（[`SegmentedButton::multi_select`]）は各項目がトグルボタンで、矢印キーでフォーカスだけが移る。
#[derive(IntoElement)]
pub struct SegmentedButton {
    id: ElementId,
    label: SharedString,
    segments: Vec<Segment>,
    selected: Vec<usize>,
    multi: bool,
    on_change: Option<ChangeHandler>,
    forced: Option<(usize, ForcedState)>,
}

impl SegmentedButton {
    /// ID とグループのラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            segments: Vec::new(),
            selected: Vec::new(),
            multi: false,
            on_change: None,
            forced: None,
        }
    }

    /// 項目を加える。
    pub fn segment(mut self, segment: Segment) -> Self {
        self.segments.push(segment);
        self
    }

    /// 選択中の項目（単一選択なら 1 つ）。
    pub fn selected(mut self, selected: impl IntoIterator<Item = usize>) -> Self {
        self.selected = selected.into_iter().collect();
        self
    }

    /// 複数選択にする。
    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// 項目が押されたときの処理。引数は押された項目の位置。
    /// 単一選択ならその項目を選択に、複数選択ならその項目の選択を反転する。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 見本用に 1 項目の状態を固定する。
    #[doc(hidden)]
    pub fn forced(mut self, index: usize, forced: ForcedState) -> Self {
        self.forced = Some((index, forced));
        self
    }

    /// グループのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = if self.multi {
            Role::Group
        } else {
            Role::RadioGroup
        };
        A11yNode::new(role).label(self.label.clone())
    }

    /// 項目のアクセシビリティ情報。
    pub fn segment_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let segment = self.segments.get(index)?;
        let selected = self.selected.contains(&index);
        let node = if self.multi {
            A11yNode::new(Role::Button).toggled(toggled(selected))
        } else {
            A11yNode::new(Role::RadioButton).selected(selected)
        };
        Some(
            node.label(segment.label.clone())
                .position(index, self.segments.len())
                .disabled(segment.disabled),
        )
    }

    fn segment_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            SharedString::from(format!("{index}")),
        )
    }
}

impl RenderOnce for SegmentedButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.segments.len();
        let enabled: Rc<[bool]> = self.segments.iter().map(|s| !s.disabled).collect();
        // Tab で止まる項目: 単一選択なら選択中の項目、無ければ最初の有効な項目。
        let tab_target = self
            .selected
            .iter()
            .copied()
            .find(|&i| enabled.get(i).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&e| e));
        let handles: Rc<[FocusHandle]> = (0..count)
            .map(|index| focus_handle(self.segment_id(index), window, cx))
            .collect();
        let group_node = self.a11y_node();

        let mut row = div()
            .id(self.id.clone())
            .a11y(group_node)
            .flex()
            .flex_none();
        for (index, segment) in self.segments.iter().enumerate() {
            let node = self.segment_a11y_node(index);
            let selected = self.selected.contains(&index);
            let forced = self
                .forced
                .and_then(|(i, state)| (i == index).then_some(state));
            let tab_stop = self.multi || tab_target == Some(index);
            let (base, control) = control(
                &self.segment_id(index),
                segment.disabled,
                tab_stop,
                forced,
                window,
                cx,
            );
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

            let mut item = base
                .h(theme.dense(tokens::CONTAINER_HEIGHT))
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
                    .disabled(segment.disabled)
                    .forced(forced),
                )
                .child(
                    FocusRing::new(CornerRadius::Fixed(px(0.0)))
                        .corners(corners)
                        .visible(control.focus_visible),
                );
            if let Some(node) = node {
                item = item.a11y(node);
            }
            if !segment.disabled
                && let Some(on_change) = self.on_change.clone()
            {
                let on_click = on_change.clone();
                item = item.on_click(move |_, window, cx| on_click(index, window, cx));
                let handles = handles.clone();
                let enabled = enabled.clone();
                let multi = self.multi;
                item = item.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let forward = match event.keystroke.key.as_str() {
                        "right" | "down" => true,
                        "left" | "up" => false,
                        _ => return,
                    };
                    let Some(next) = step(&enabled, index, forward) else {
                        return;
                    };
                    cx.stop_propagation();
                    if let Some(handle) = handles.get(next) {
                        window.focus(handle, cx);
                    }
                    if !multi {
                        on_change(next, window, cx);
                    }
                });
            }
            row = row.child(item);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_select_is_a_radio_group() {
        let button = SegmentedButton::new("s", "表示")
            .segment(Segment::new("日"))
            .segment(Segment::new("週"))
            .selected([1]);
        assert_eq!(button.a11y_node().role, Role::RadioGroup);
        let node = button.segment_a11y_node(1).expect("segment");
        assert_eq!(node.role, Role::RadioButton);
        assert_eq!(node.selected, Some(true));
        assert_eq!(node.position, Some((2, 2)));
    }
}
