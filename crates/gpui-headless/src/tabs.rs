//! タブの状態と操作。
use crate::{A11yExt, A11yNode, ControlState, control, focus_handle, step};
use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, KeyDownEvent, Pixels, Role, SharedString,
    Stateful, Window, div, prelude::*, px,
};
use std::rc::Rc;
type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
struct TabOption {
    label: SharedString,
    disabled: bool,
}

/// タブの並び（全タブが同じ幅で、並び全体の幅を分け合う）。
///
/// Tab では選択中のタブで 1 回だけ止まり、←→ で隣のタブを選ぶ（フォーカスと選択が一緒に動く）。
/// Home / End は端のタブ。タブの中身（パネル）は呼び出し側が選択に合わせて描く。
pub struct Tabs {
    id: ElementId,
    label: SharedString,
    reveal_margin: Pixels,
    group_prefix: SharedString,
    before: Vec<AnyElement>,
    tabs: Vec<TabOption>,
    selected: usize,
    on_change: Option<ChangeHandler>,
}

impl Tabs {
    /// ID、名前（読み上げられる）、選択中のタブの位置から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
            before: Vec::new(),
            tabs: Vec::new(),
            selected,
            on_change: None,
        }
    }

    /// タブを足す。
    pub fn tab(mut self, label: impl Into<SharedString>, disabled: bool) -> Self {
        self.tabs.push(TabOption {
            label: label.into(),
            disabled,
        });
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

    /// タブより先に描く要素を加える。
    pub fn before(mut self, element: impl IntoElement) -> Self {
        self.before.push(element.into_any_element());
        self
    }

    /// タブを選んだとき（クリック、矢印キー、Home / End）の処理。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// アクセシビリティツリーに出す情報（役割 TabList）。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::TabList).label(self.label.clone())
    }

    /// タブ `index` のアクセシビリティ情報（役割 Tab）。
    pub fn tab_a11y_node(&self, index: usize) -> Option<A11yNode> {
        self.tabs.get(index).map(|tab| {
            A11yNode::new(Role::Tab)
                .label(tab.label.clone())
                .selected(index == self.selected)
                .position(index, self.tabs.len())
                .disabled(tab.disabled)
        })
    }

    fn tab_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(self.id.clone().into(), format!("{index}").into())
    }
}

/// タブの要素と描画に使う状態。
pub struct TabItem {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// タブの位置。
    pub index: usize,
    /// 選択されているか。
    pub selected: bool,
}
impl Tabs {
    /// タブの描画を受け取り、操作を付けたグループを返す。
    pub fn build(
        mut self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(TabItem, &mut Window, &mut App) -> AnyElement,
    ) -> Stateful<Div> {
        window.with_element_namespace(self.id.clone(), |window| {
            let count = self.tabs.len();
            let enabled: Rc<[bool]> = self.tabs.iter().map(|tab| !tab.disabled).collect();
            let target = Some(self.selected)
                .filter(|&i| enabled.get(i).copied().unwrap_or(false))
                .or_else(|| enabled.iter().position(|&e| e));
            let handles: Rc<[FocusHandle]> = (0..count)
                .map(|index| focus_handle(self.tab_id(index), window, cx))
                .collect();
            let mut row = div()
                .id(self.id.clone())
                .a11y(self.a11y_node())
                .children(std::mem::take(&mut self.before));
            for (index, tab) in self.tabs.iter().enumerate() {
                let id = self.tab_id(index);
                let (mut base, state) = control(
                    &id,
                    tab.disabled,
                    target == Some(index),
                    window,
                    cx,
                    self.reveal_margin,
                    format!("{}-{id}", self.group_prefix).into(),
                );
                if let Some(node) = self.tab_a11y_node(index) {
                    base = base.a11y(node);
                }
                if !tab.disabled
                    && let Some(on_change) = self.on_change.clone()
                {
                    let on_click = on_change.clone();
                    base = base.on_click(move |_, window, cx| on_click(index, window, cx));
                    let handles = handles.clone();
                    let enabled = enabled.clone();
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let next = match event.keystroke.key.as_str() {
                            "right" => step(&enabled, index, true),
                            "left" => step(&enabled, index, false),
                            "home" => enabled.iter().position(|&e| e),
                            "end" => enabled.iter().rposition(|&e| e),
                            _ => return,
                        };
                        let Some(next) = next else {
                            return;
                        };
                        cx.stop_propagation();
                        if let Some(handle) = handles.get(next) {
                            window.focus(handle, cx);
                        }
                        on_change(next, window, cx);
                    });
                }
                row = row.child(window.with_element_namespace(id, |window| {
                    render(
                        TabItem {
                            base,
                            state,
                            index,
                            selected: index == self.selected,
                        },
                        window,
                        cx,
                    )
                }));
            }
            row
        })
    }
}
