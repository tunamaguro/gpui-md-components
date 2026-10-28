//! RadioButton（RadioGroup としてまとめて使う）。

use std::rc::Rc;

use gpui::{
    App, ElementId, FocusHandle, IntoElement, KeyDownEvent, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::radio_button as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Md3ThemeExt, TypeScale};

use crate::a11y::{A11yExt, A11yNode};
use crate::control::{control, step};
use crate::{FocusRing, ForcedState, StateLayer, Text, focus_handle};

/// ラジオボタンの選択肢。
#[derive(Clone, Debug)]
struct RadioOption {
    label: SharedString,
    disabled: bool,
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// ラジオボタンのグループ。選択肢ごとにラジオボタンとラベルを縦に並べる。
///
/// APG の Radio Group に従い、Tab で止まるのは選択中の項目（無ければ最初の項目）だけで、
/// 矢印キーでグループ内を移ると同時に選択する。
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    label: SharedString,
    options: Vec<RadioOption>,
    selected: Option<usize>,
    horizontal: bool,
    on_change: Option<ChangeHandler>,
    forced: Option<(usize, ForcedState)>,
}

impl RadioGroup {
    /// ID とグループのラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: Vec::new(),
            selected: None,
            horizontal: false,
            on_change: None,
            forced: None,
        }
    }

    /// 選択肢を加える。
    pub fn option(mut self, label: impl Into<SharedString>) -> Self {
        self.options.push(RadioOption {
            label: label.into(),
            disabled: false,
        });
        self
    }

    /// 無効な選択肢を加える。
    pub fn disabled_option(mut self, label: impl Into<SharedString>) -> Self {
        self.options.push(RadioOption {
            label: label.into(),
            disabled: true,
        });
        self
    }

    /// 選択中の項目。
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// 横に並べる。
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// 選択が変わったときの処理。引数は選ばれた項目の位置。
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
        A11yNode::new(Role::RadioGroup).label(self.label.clone())
    }

    /// 選択肢のアクセシビリティ情報。
    pub fn option_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let option = self.options.get(index)?;
        Some(
            A11yNode::new(Role::RadioButton)
                .label(option.label.clone())
                .selected(self.selected == Some(index))
                .position(index, self.options.len())
                .disabled(option.disabled),
        )
    }

    fn option_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            SharedString::from(format!("{index}")),
        )
    }
}

/// 選択状態ごとの円の色。
pub(crate) fn icon_color(selected: bool) -> (ColorRole, ColorRole) {
    if selected {
        (
            tokens::SELECTED_ICON_COLOR,
            tokens::SELECTED_HOVER_ICON_COLOR,
        )
    } else {
        (
            tokens::UNSELECTED_ICON_COLOR,
            tokens::UNSELECTED_HOVER_ICON_COLOR,
        )
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.options.len();
        let enabled: Rc<[bool]> = self.options.iter().map(|o| !o.disabled).collect();
        let tab_target = self
            .selected
            .filter(|&i| enabled.get(i).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&e| e));
        let handles: Rc<[FocusHandle]> = (0..count)
            .map(|index| focus_handle(self.option_id(index), window, cx))
            .collect();

        let mut group = div().id(self.id.clone()).a11y(self.a11y_node()).flex();
        group = if self.horizontal {
            group.flex_row().flex_wrap().gap_4()
        } else {
            group.flex_col()
        };
        for (index, option) in self.options.iter().enumerate() {
            let selected = self.selected == Some(index);
            let forced = self
                .forced
                .and_then(|(i, state)| (i == index).then_some(state));
            let (base, control) = control(
                &self.option_id(index),
                option.disabled,
                tab_target == Some(index),
                forced,
                window,
                cx,
            );
            let theme = cx.md3();
            let (role, hovered_role) = icon_color(selected);
            let color = if option.disabled {
                let opacity = if selected {
                    tokens::DISABLED_SELECTED_ICON_OPACITY
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_OPACITY
                };
                theme
                    .color(tokens::DISABLED_SELECTED_ICON_COLOR)
                    .opacity(opacity)
            } else {
                theme.color(role)
            };
            let hovered = theme.color(hovered_role);
            let state_size = theme.dense(tokens::STATE_LAYER_SIZE);
            let radius = CornerRadius::Full;

            let mut ring = div()
                .size(px(tokens::ICON_SIZE))
                .rounded_full()
                .border(px(layout::RADIO_BUTTON_STROKE_WIDTH))
                .border_color(color)
                .flex()
                .items_center()
                .justify_center();
            if !option.disabled {
                ring = ring.group_hover(control.group.clone(), move |s| s.border_color(hovered));
            }
            if selected {
                let mut dot = div()
                    .size(px(layout::RADIO_BUTTON_DOT_SIZE))
                    .rounded_full()
                    .bg(color);
                if !option.disabled {
                    dot = dot.group_hover(control.group.clone(), move |s| s.bg(hovered));
                }
                ring = ring.child(dot);
            }
            let radio = div()
                .relative()
                .size(state_size)
                .flex()
                .items_center()
                .justify_center()
                .child(ring)
                .child(
                    StateLayer::new(
                        SharedString::from(format!("{}-layer", control.group)),
                        control.group.clone(),
                        role,
                    )
                    .radius(radius)
                    .focused(control.focus_visible)
                    .disabled(option.disabled)
                    .forced(forced),
                )
                .child(FocusRing::new(radius).visible(control.focus_visible));

            let label_color = if option.disabled {
                theme
                    .color(ColorRole::OnSurface)
                    .opacity(tokens::DISABLED_UNSELECTED_ICON_OPACITY)
            } else {
                theme.color(ColorRole::OnSurface)
            };
            let mut item = base
                .flex()
                .items_center()
                .pr_2()
                .text_color(label_color)
                .child(radio)
                .child(Text::new(TypeScale::BodyLarge, option.label.clone()));
            if let Some(node) = self.option_a11y_node(index) {
                item = item.a11y(node);
            }
            if !option.disabled
                && let Some(on_change) = self.on_change.clone()
            {
                let on_click = on_change.clone();
                item = item.on_click(move |_, window, cx| on_click(index, window, cx));
                let handles = handles.clone();
                let enabled = enabled.clone();
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
                    on_change(next, window, cx);
                });
            }
            group = group.child(item);
        }
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_report_selection_and_position() {
        let group = RadioGroup::new("g", "サイズ")
            .option("小")
            .option("中")
            .disabled_option("大")
            .selected(Some(0));
        assert_eq!(group.a11y_node().role, Role::RadioGroup);
        let first = group.option_a11y_node(0).expect("option");
        assert_eq!(first.role, Role::RadioButton);
        assert_eq!(first.selected, Some(true));
        assert_eq!(first.position, Some((1, 3)));
        assert!(group.option_a11y_node(2).expect("option").disabled);
    }

    #[test]
    fn selected_uses_primary() {
        assert_eq!(icon_color(true).0, ColorRole::Primary);
        assert_eq!(icon_color(false).0, ColorRole::OnSurfaceVariant);
    }
}
