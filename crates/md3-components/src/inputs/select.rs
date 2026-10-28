//! Select（テキストフィールドの見た目のドロップダウン。MD3 の exposed dropdown menu）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    App, ElementId, IntoElement, RenderOnce, Role, SharedString, Window, div, prelude::*, px,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::filled_text_field as tokens;
use md3_theme::md3_tokens::layout;

use super::field::{FieldColors, FieldFrame, FieldState, TextFieldVariant, tint};
use crate::a11y::{A11yExt, A11yNode};
use crate::control::control;
use crate::overlays::{Menu, MenuItem, MenuKind};
use crate::{ForcedState, Icon, IconName, focus_handle};

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// 選択肢から 1 つを選ぶ入力。ラベル（読み上げ用の名前を兼ねる）が必須。
///
/// 押すか、Enter / Space / ↓ で選択肢のメニューを開く。開閉の状態は部品が持つ。
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    label: SharedString,
    options: Vec<SharedString>,
    selected: Option<usize>,
    variant: TextFieldVariant,
    supporting_text: Option<SharedString>,
    error: bool,
    leading_icon: Option<IconName>,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    forced: Option<ForcedState>,
}

impl Select {
    /// ID、ラベル、選択肢、選ばれている位置から作る。
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = impl Into<SharedString>>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: options.into_iter().map(Into::into).collect(),
            selected,
            variant: TextFieldVariant::default(),
            supporting_text: None,
            error: false,
            leading_icon: None,
            disabled: false,
            on_change: None,
            forced: None,
        }
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 下に出す補助テキスト。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// エラー状態にする。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// 先頭のアイコン（装飾）。
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 選んだときの処理。引数は選んだ位置。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 見本用に状態を固定する（`Hovered` と `Focused` だけを使う）。
    #[doc(hidden)]
    pub fn forced(mut self, forced: ForcedState) -> Self {
        self.forced = Some(forced);
        self
    }

    fn value(&self) -> Option<SharedString> {
        self.selected
            .and_then(|index| self.options.get(index).cloned())
    }

    /// アクセシビリティツリーに出す情報（開閉の状態は描画時に付ける）。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::ComboBox)
            .label(self.label.clone())
            .disabled(self.disabled)
            .invalid(self.error)
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let value = self.value();
        let parent = Arc::new(self.id.clone());
        let open = window.use_keyed_state(
            ElementId::NamedChild(parent.clone(), "open".into()),
            cx,
            |_, _| false,
        );
        let hovered = window.use_keyed_state(
            ElementId::NamedChild(parent.clone(), "hovered".into()),
            cx,
            |_, _| false,
        );
        let is_open = *open.read(cx) && !self.disabled;
        let (base, _) = control(&self.id, self.disabled, true, self.forced, window, cx);
        let focused = !self.disabled
            && (is_open
                || self.forced == Some(ForcedState::Focused)
                || focus_handle(self.id.clone(), window, cx).is_focused(window));
        let state = FieldState {
            focused,
            hovered: !self.disabled
                && (*hovered.read(cx) || self.forced == Some(ForcedState::Hovered)),
            error: self.error,
            disabled: self.disabled,
        };
        let theme = &cx.md3().clone();
        let colors = FieldColors::of(self.variant, state);
        let arrow = if is_open {
            IconName::ArrowDropUp
        } else {
            IconName::ArrowDropDown
        };
        let (mut container, supporting) = FieldFrame {
            variant: self.variant,
            state,
            label: Some(self.label.clone()),
            floated: value.is_some() || focused,
            leading_icon: self.leading_icon,
            trailing: Some(
                Icon::new(arrow)
                    .size(px(tokens::TRAILING_ICON_SIZE))
                    .tint(tint(theme, colors.trailing_icon))
                    .into_any_element(),
            ),
            content: div()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .children(value.clone())
                .into_any_element(),
            supporting_text: self.supporting_text.clone(),
            counter: None,
        }
        .render(cx);

        if is_open {
            let on_change = self.on_change.clone();
            let close = open.clone();
            let menu = Menu::new(
                ElementId::NamedChild(parent.clone(), "menu".into()),
                self.label.clone(),
            )
            .kind(MenuKind::Options)
            .selected(self.selected)
            .match_anchor_width()
            .items(self.options.iter().enumerate().map(|(index, option)| {
                let on_change = on_change.clone();
                MenuItem::new(option.clone()).on_select(move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(index, window, cx);
                    }
                })
            }))
            .on_dismiss(move |_, cx| {
                close.update(cx, |open, cx| {
                    *open = false;
                    cx.notify();
                })
            });
            container = container.child(menu);
        }
        // テキストフィールドと同じく、フォーカスはフォーカスリングではなく下線・枠線とラベルの色で示す。
        let _ = &theme;

        let mut select = base
            .a11y(node)
            .aria_expanded(is_open)
            .when_some(value, |this, value| this.aria_value(value))
            .min_w(px(layout::TEXT_FIELD_MIN_WIDTH))
            .flex()
            .flex_col()
            .child(container)
            .children(supporting);
        if !self.disabled {
            let toggle = open.clone();
            let open_by_key = open.clone();
            select = select
                .on_hover(move |hover, _, cx| {
                    hovered.update(cx, |value, cx| {
                        *value = *hover;
                        cx.notify();
                    })
                })
                .on_click(move |_, _, cx| {
                    toggle.update(cx, |open, cx| {
                        *open = !*open;
                        cx.notify();
                    })
                })
                .on_key_down(move |event, _, cx| {
                    if event.keystroke.key == "down" && !*open_by_key.read(cx) {
                        open_by_key.update(cx, |open, cx| {
                            *open = true;
                            cx.notify();
                        });
                        cx.stop_propagation();
                    }
                });
        }
        select
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_is_a_combobox_with_its_value() {
        let select = Select::new("s", "色", ["赤", "青"], Some(1));
        assert_eq!(select.a11y_node().role, Role::ComboBox);
        assert_eq!(select.value().as_deref(), Some("青"));
        assert_eq!(Select::new("s", "色", ["赤"], Some(3)).value(), None);
        assert!(!select.a11y_node().invalid);
        assert!(select.error(true).a11y_node().invalid);
    }
}
