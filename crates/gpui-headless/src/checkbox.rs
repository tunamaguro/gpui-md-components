//! チェックボックスの状態と操作。

use crate::{A11yExt, A11yNode, ControlState, control};
use gpui::{
    App, Div, ElementId, Pixels, Role, SharedString, Stateful, Toggled, Window, prelude::*, px,
};

/// チェックの状態。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckboxState {
    /// 未チェック。
    #[default]
    Unchecked,
    /// チェック済み。
    Checked,
    /// 一部だけ選ばれている。
    Indeterminate,
}

impl CheckboxState {
    /// 押したときの次の状態。不定はチェック済みになる。
    pub fn next(self) -> Self {
        match self {
            Self::Checked => Self::Unchecked,
            Self::Unchecked | Self::Indeterminate => Self::Checked,
        }
    }
}

type ChangeHandler = Box<dyn Fn(CheckboxState, &mut Window, &mut App) + 'static>;

/// 見た目を持たないチェックボックス。
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    label: Option<SharedString>,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group: SharedString,
}

impl Checkbox {
    /// ID と状態から作る。
    pub fn new(id: impl Into<ElementId>, state: CheckboxState) -> Self {
        let id = id.into();
        Self {
            group: format!("control-{id}").into(),
            id,
            state,
            label: None,
            disabled: false,
            on_change: None,
            reveal_margin: px(0.0),
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 現在の状態を返す。
    pub fn state(&self) -> CheckboxState {
        self.state
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 状態が変わるときの処理を設定する。
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
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
        let toggled = match self.state {
            CheckboxState::Unchecked => Toggled::False,
            CheckboxState::Checked => Toggled::True,
            CheckboxState::Indeterminate => Toggled::Mixed,
        };
        let mut node = A11yNode::new(Role::CheckBox)
            .toggled(toggled)
            .disabled(self.disabled);
        node.label = self.label.clone();
        node
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
        let base = base.a11y(node).when_some(
            self.on_change.filter(|_| !self.disabled),
            |base, handler| {
                let next = self.state.next();
                base.on_click(move |_, window, cx| handler(next, window, cx))
            },
        );
        (base, state)
    }
}
