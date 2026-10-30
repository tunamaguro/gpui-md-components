//! ボタンの状態と操作。

use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    App, ClickEvent, Div, ElementId, Pixels, Role, SharedString, Stateful, Window, prelude::*, px,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// 見た目を持たないボタン。
pub struct Button {
    id: ElementId,
    label: SharedString,
    toggled: Option<bool>,
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
            toggled: None,
            disabled: false,
            on_click: None,
            reveal_margin: px(0.0),
        }
    }
    /// トグルの状態を設定する。
    pub fn toggled(mut self, value: Option<bool>) -> Self {
        self.toggled = value;
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
        let node = A11yNode::new(Role::Button)
            .label(self.label.clone())
            .disabled(self.disabled);
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
        (
            base.a11y(node)
                .when_some(self.on_click.filter(|_| !self.disabled), |base, handler| {
                    base.on_click(handler)
                }),
            state,
        )
    }
}
