//! NavigationBar / Rail / Drawer が共有する項目と、項目の間のキー操作。

use gpui_headless::focus_handle;
use std::rc::Rc;

use gpui::{App, ElementId, FocusHandle, KeyDownEvent, Role, SharedString, Window};

use crate::IconName;
use gpui_headless::A11yNode;
use gpui_headless::step;

/// 行き先を選んだときの処理（選んだ項目の位置を受け取る）。
pub(crate) type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// ナビゲーションの行き先 1 つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationItem {
    pub(crate) icon: IconName,
    pub(crate) label: SharedString,
    pub(crate) disabled: bool,
}

impl NavigationItem {
    /// アイコンとラベルから作る。
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            label: label.into(),
            disabled: false,
        }
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 項目の並びの状態（ID、フォーカスハンドル、Tab で止まる項目）。
pub(crate) struct Destinations {
    pub ids: Vec<ElementId>,
    pub handles: Rc<[FocusHandle]>,
    pub enabled: Rc<[bool]>,
    /// Tab で止まる項目（選択中、無ければ最初の有効な項目）。
    pub tab_target: Option<usize>,
}

impl Destinations {
    pub fn new(
        id: &ElementId,
        items: &[NavigationItem],
        selected: Option<usize>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let ids: Vec<ElementId> = (0..items.len())
            .map(|index| ElementId::NamedChild(id.clone().into(), format!("{index}").into()))
            .collect();
        let handles = ids
            .iter()
            .map(|id| focus_handle(id.clone(), window, cx))
            .collect();
        let enabled: Rc<[bool]> = items.iter().map(|item| !item.disabled).collect();
        let tab_target = selected
            .filter(|&index| enabled.get(index).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&e| e));
        Self {
            ids,
            handles,
            enabled,
            tab_target,
        }
    }

    /// 項目 `index` のキー操作。矢印キーでフォーカスだけを移し、選ぶのは Enter / Space（手動の選択）。
    /// `vertical` なら ↑↓、そうでなければ ←→ で移る。Home / End は端の有効な項目。
    pub fn on_key(
        &self,
        index: usize,
        vertical: bool,
    ) -> impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static {
        let handles = self.handles.clone();
        let enabled = self.enabled.clone();
        move |event, window, cx| {
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
        }
    }
}

/// 行き先の項目のアクセシビリティ情報（役割 Tab、選択中なら selected）。
pub(crate) fn destination_a11y_node(
    item: &NavigationItem,
    index: usize,
    count: usize,
    selected: bool,
) -> A11yNode {
    A11yNode::new(Role::Tab)
        .label(item.label.clone())
        .selected(selected)
        .position(index, count)
        .disabled(item.disabled)
}

/// 行き先の並び全体のアクセシビリティ情報（役割 Navigation）。
pub(crate) fn navigation_a11y_node(label: &SharedString) -> A11yNode {
    A11yNode::new(Role::Navigation).label(label.clone())
}
