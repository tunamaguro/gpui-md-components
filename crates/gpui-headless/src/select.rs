//! 選択肢から 1 つを選ぶ入力の開閉と操作。

use crate::overlay::DismissHandler;
use crate::{A11yExt, A11yNode, MenuKind, control, focus_handle};
use gpui::{
    App, Div, ElementId, Entity, Pixels, Role, SharedString, Stateful, Window, prelude::*, px,
};
use std::rc::Rc;
use std::sync::Arc;

/// 押すか、Enter / Space / ↓ で開く入力。選択肢は [`crate::Menu`] で描く。
pub struct Select {
    id: ElementId,
    name: crate::AccessibleName,
    value: Option<SharedString>,
    disabled: bool,
    error: bool,
    required: bool,
    description: Option<SharedString>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}

/// Select の描画に使う状態。
pub struct SelectState {
    /// ホバーなどに使うグループ名。
    pub group: SharedString,
    /// キーボードのフォーカスを示すか。
    pub focus_visible: bool,
    /// 無効か。
    pub disabled: bool,
    /// 開いているか。
    pub open: bool,
    /// フォーカスがあるか（開いている間も含む）。
    pub focused: bool,
    /// ポインタが重なっているか。
    pub hovered: bool,
    open_state: Entity<bool>,
}

impl SelectState {
    /// 選択肢のメニューを閉じる処理。[`crate::Menu::on_dismiss`] に渡す。
    pub fn dismiss_handler(&self) -> DismissHandler {
        let open = self.open_state.clone();
        Rc::new(move |_, cx| set_open(&open, false, cx))
    }
}

fn set_open(open: &Entity<bool>, value: bool, cx: &mut App) {
    open.update(cx, |open, cx| {
        *open = value;
        cx.notify();
    });
}

impl Select {
    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// 入力の補足説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            value: None,
            disabled: false,
            error: false,
            required: false,
            description: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }

    /// 表示するラベルを既定の名前にする。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.name.set_visible(label);
        self
    }

    accessible_name!();

    /// 選ばれている値の文。
    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 入力に誤りがあることを示す。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }

    /// ホバーなどに使うグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }

    /// アクセシビリティツリーに出す情報（開閉の状態は `build` が付ける）。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(Role::ComboBox)
            .with_name(&self.name)
            .disabled(self.disabled)
            .invalid(self.error)
            .required(self.required)
            .has_popup(MenuKind::Options.popup());
        if let Some(description) = &self.description {
            node = node.description(description.clone());
        }
        node
    }

    /// 操作を付けた土台と描画用の状態を返す。
    pub fn build(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, SelectState) {
        let node = self.a11y_node();
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
        let (base, control) = control(
            &self.id,
            self.disabled,
            true,
            window,
            cx,
            self.reveal_margin,
            format!("{}-{}", self.group_prefix, self.id).into(),
        );
        let focused = !self.disabled
            && (is_open || focus_handle(self.id.clone(), window, cx).is_focused(window));
        let mut select = base
            .a11y(node)
            .aria_expanded(is_open)
            .when_some(self.value, |this, value| this.aria_value(value));
        if !self.disabled {
            let toggle = open.clone();
            let open_by_key = open.clone();
            let hover_state = hovered.clone();
            select = select
                .on_hover(move |hover, _, cx| {
                    hover_state.update(cx, |value, cx| {
                        *value = *hover;
                        cx.notify();
                    })
                })
                .on_click(move |_, _, cx| {
                    let next = !*toggle.read(cx);
                    set_open(&toggle, next, cx);
                })
                .on_key_down(move |event, _, cx| {
                    if event.keystroke.key == "down" && !*open_by_key.read(cx) {
                        set_open(&open_by_key, true, cx);
                        cx.stop_propagation();
                    }
                });
        }
        let state = SelectState {
            group: control.group,
            focus_visible: control.focus_visible,
            disabled: self.disabled,
            open: is_open,
            focused,
            hovered: !self.disabled && *hovered.read(cx),
            open_state: open,
        };
        (select, state)
    }
}
