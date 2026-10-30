//! 行き先のグループの状態と操作。
use crate::{A11yExt, A11yNode, ControlState, control, focus_handle, step};
use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, KeyDownEvent, Pixels, Role, SharedString,
    Stateful, Window, div, prelude::*, px,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
struct Destination {
    label: SharedString,
    disabled: bool,
}

/// 行き先の要素と描画に使う状態。
pub struct NavigationItem {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 項目の位置。
    pub index: usize,
    /// 選択されているか。
    pub selected: bool,
}

/// 行き先を描く土台と、初期フォーカス先。
pub struct NavigationBase {
    /// アクセシビリティ情報を付けたグループ。
    pub base: Stateful<Div>,
    /// クロージャが描いた項目。
    pub items: Vec<AnyElement>,
    /// Tab で止まる項目（モーダルの初期フォーカスにも使う）。
    pub initial_focus: Option<FocusHandle>,
}

/// 見た目を持たない行き先のグループ。
pub struct Navigation {
    id: ElementId,
    label: SharedString,
    items: Vec<Destination>,
    selected: Option<usize>,
    vertical: bool,
    dialog: Option<bool>,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}
impl Navigation {
    /// ID と名前から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            selected: None,
            vertical: false,
            dialog: None,
            on_change: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }
    /// 行き先を足す。
    pub fn item(mut self, label: impl Into<SharedString>, disabled: bool) -> Self {
        self.items.push(Destination {
            label: label.into(),
            disabled,
        });
        self
    }
    /// 選択中の位置を設定する。
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    /// 縦方向の矢印キーを使う。
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    /// ダイアログとして扱い、モーダル状態を設定する。
    pub fn dialog(mut self, modal: bool) -> Self {
        self.dialog = Some(modal);
        self
    }
    /// 行き先を選んだときの処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// 項目のホバーなどに使うグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }
    /// グループのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        match self.dialog {
            Some(modal) => A11yNode::new(Role::Dialog)
                .label(self.label.clone())
                .modal(modal),
            None => A11yNode::new(Role::Navigation).label(self.label.clone()),
        }
    }
    /// 項目のアクセシビリティ情報。
    pub fn item_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let item = self.items.get(index)?;
        Some(
            A11yNode::new(Role::Tab)
                .label(item.label.clone())
                .selected(self.selected == Some(index))
                .position(index, self.items.len())
                .disabled(item.disabled),
        )
    }
    /// 項目の描画を受け取り、土台と初期フォーカス先を返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(NavigationItem, &mut Window, &mut App) -> AnyElement,
    ) -> NavigationBase {
        window.with_element_namespace(self.id.clone(), |window| {
            let ids: Vec<_> = (0..self.items.len())
                .map(|i| ElementId::NamedChild(self.id.clone().into(), format!("{i}").into()))
                .collect();
            let handles: Rc<[FocusHandle]> = ids
                .iter()
                .map(|id| focus_handle(id.clone(), window, cx))
                .collect();
            let enabled: Rc<[bool]> = self.items.iter().map(|item| !item.disabled).collect();
            let target = self
                .selected
                .filter(|&i| enabled.get(i).copied().unwrap_or(false))
                .or_else(|| enabled.iter().position(|&e| e));
            let mut items = Vec::with_capacity(self.items.len());
            for (index, item) in self.items.iter().enumerate() {
                let id = &ids[index];
                let (mut base, state) = control(
                    id,
                    item.disabled,
                    target == Some(index),
                    window,
                    cx,
                    self.reveal_margin,
                    format!("{}-{id}", self.group_prefix).into(),
                );
                if let Some(node) = self.item_a11y_node(index) {
                    base = base.a11y(node);
                }
                if !item.disabled {
                    let handles = handles.clone();
                    let enabled = enabled.clone();
                    let vertical = self.vertical;
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let next = match (event.keystroke.key.as_str(), vertical) {
                            ("down", true) | ("right", false) => step(&enabled, index, true),
                            ("up", true) | ("left", false) => step(&enabled, index, false),
                            ("home", _) => enabled.iter().position(|&e| e),
                            ("end", _) => enabled.iter().rposition(|&e| e),
                            _ => return,
                        };
                        if let Some(handle) = next.and_then(|next| handles.get(next)) {
                            cx.stop_propagation();
                            window.focus(handle, cx);
                        }
                    });
                    if let Some(on_change) = self.on_change.clone() {
                        base = base.on_click(move |_, window, cx| on_change(index, window, cx));
                    }
                }
                items.push(window.with_element_namespace(id.clone(), |window| {
                    render(
                        NavigationItem {
                            base,
                            state,
                            index,
                            selected: self.selected == Some(index),
                        },
                        window,
                        cx,
                    )
                }));
            }
            let node = self.a11y_node();
            NavigationBase {
                base: div().id(self.id).a11y(node),
                items,
                initial_focus: target.and_then(|index| handles.get(index).cloned()),
            }
        })
    }
}
