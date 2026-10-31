//! チップの状態と操作。

use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    App, ClickEvent, Div, ElementId, Pixels, Role, SharedString, Stateful, Window, prelude::*, px,
};
use std::rc::Rc;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type RemoveHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// 見た目を持たないチップ。選択状態がある場合はチェックボックスとして扱う。
pub struct Chip {
    id: ElementId,
    name: crate::AccessibleName,
    selected: bool,
    selectable: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    on_remove: Option<RemoveHandler>,
    remove_label: Option<SharedString>,
    reveal_margin: Pixels,
    group: SharedString,
}
impl Chip {
    /// ID と読み上げ用の名前から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            group: format!("control-{id}").into(),
            id,
            name: crate::AccessibleName::new(label),
            selected: false,
            selectable: false,
            disabled: false,
            on_click: None,
            on_remove: None,
            remove_label: None,
            reveal_margin: px(0.0),
        }
    }
    accessible_name!();

    /// チェックボックスとして扱い、選択状態を指定する。
    pub fn selected(mut self, selected: Option<bool>) -> Self {
        self.selected = selected.unwrap_or(false);
        self.selectable = selected.is_some();
        self
    }
    /// 選択状態と、チェックボックスとして公開するかを指定する。
    pub fn selection(mut self, selected: bool, selectable: bool) -> Self {
        self.selected = selected;
        self.selectable = selectable;
        self
    }
    /// 選択状態を返す。
    pub fn is_selected(&self) -> bool {
        self.selected
    }
    /// 有効な削除処理を返す。
    pub fn remove_handler(&self) -> Option<RemoveHandler> {
        self.on_remove.as_ref().filter(|_| !self.disabled).cloned()
    }
    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// 押されたときの処理を設定する。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
    /// Backspace / Delete による削除処理を設定する。
    pub fn on_remove(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }
    /// 削除ボタンの読み上げる名前を指定する。
    pub fn remove_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.remove_label = (!label.is_empty()).then_some(label);
        self
    }
    /// 削除ボタンのアクセシビリティ情報。
    pub fn remove_a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Button)
            .with_label(self.remove_label.clone())
            .disabled(self.disabled)
    }
    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// ホバーなどに使うグループ名を指定する。
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = group.into();
        self
    }
    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let node = if self.selectable {
            A11yNode::new(Role::CheckBox).toggled(toggled(self.selected))
        } else {
            A11yNode::new(Role::Button)
        };
        node.with_name(&self.name).disabled(self.disabled)
    }
    /// 操作を付けた要素と描画用の状態を返す。
    pub fn build(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, ControlState) {
        let node = self.a11y_node();
        let (base, state) = control(
            &self.id,
            self.disabled,
            true,
            window,
            cx,
            self.reveal_margin,
            self.group,
        );
        let base = base
            .a11y(node)
            .when_some(self.on_click.filter(|_| !self.disabled), |base, handler| {
                base.on_click(handler)
            })
            .when_some(
                self.on_remove.filter(|_| !self.disabled),
                |base, handler| {
                    base.on_key_down(move |event, window, cx| {
                        if matches!(event.keystroke.key.as_str(), "backspace" | "delete") {
                            cx.stop_propagation();
                            handler(window, cx);
                        }
                    })
                },
            );
        (base, state)
    }
}
