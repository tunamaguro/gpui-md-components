//! ボタンの状態と操作。

use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    App, ClickEvent, Div, ElementId, Pixels, Role, SharedString, Stateful, Window, accesskit,
    prelude::*, px,
};
use std::rc::Rc;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// 見た目を持たないボタン。
pub struct Button {
    id: ElementId,
    label: SharedString,
    description: Option<SharedString>,
    aria_label: Option<Option<SharedString>>,
    toggled: Option<bool>,
    menu_expanded: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    reveal_margin: Pixels,
    group: SharedString,
}

impl Button {
    /// ID と読み上げ用の名前から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            group: format!("control-{id}").into(),
            id,
            label: label.into(),
            description: None,
            aria_label: None,
            toggled: None,
            menu_expanded: None,
            disabled: false,
            on_click: None,
            reveal_margin: px(0.0),
        }
    }
    /// 補足の説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }
    /// トグルの状態を設定する。
    pub fn toggled(mut self, value: Option<bool>) -> Self {
        self.toggled = value;
        self
    }
    /// メニューの開閉状態を指定する。
    pub fn menu_expanded(mut self, expanded: bool) -> Self {
        self.menu_expanded = Some(expanded);
        self
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
        let mut node = A11yNode::new(Role::Button)
            .label(self.label.clone())
            .disabled(self.disabled);
        if let Some(description) = &self.description {
            node = node.description(description.clone());
        }
        if let Some(label) = &self.aria_label {
            node.label = label.clone();
        }
        if let Some(expanded) = self.menu_expanded {
            node = node.expanded(expanded).has_popup(accesskit::HasPopup::Menu);
        }
        match self.toggled {
            Some(value) => node.toggled(toggled(value)),
            None => node,
        }
    }
    pub(crate) fn id(&self) -> &ElementId {
        &self.id
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
        let mut base = base.a11y(node);
        if let Some(handler) = self.on_click.filter(|_| !self.disabled) {
            let handler: Rc<ClickHandler> = Rc::new(handler);
            if let Some(expanded) = self.menu_expanded {
                for action in [accesskit::Action::Expand, accesskit::Action::Collapse] {
                    let handler = handler.clone();
                    base = base.on_a11y_action(action, move |_, window, cx| {
                        if should_activate_menu(action, expanded, false) {
                            handler(&ClickEvent::default(), window, cx);
                        }
                    });
                }
            }
            base = base.on_click(move |event, window, cx| handler(event, window, cx));
        }
        (base, state)
    }
}

fn should_activate_menu(action: accesskit::Action, expanded: bool, disabled: bool) -> bool {
    !disabled
        && matches!(
            (action, expanded),
            (accesskit::Action::Expand, false) | (accesskit::Action::Collapse, true)
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_actions_only_activate_for_enabled_state_changes() {
        for disabled in [false, true] {
            for expanded in [false, true] {
                assert_eq!(
                    should_activate_menu(accesskit::Action::Expand, expanded, disabled),
                    !disabled && !expanded
                );
                assert_eq!(
                    should_activate_menu(accesskit::Action::Collapse, expanded, disabled),
                    !disabled && expanded
                );
                assert!(!should_activate_menu(
                    accesskit::Action::Click,
                    expanded,
                    disabled
                ));
            }
        }
    }
}
