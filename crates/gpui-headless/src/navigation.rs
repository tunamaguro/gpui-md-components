//! 行き先のグループの状態と操作。
use crate::control::Roving;
use crate::{A11yExt, A11yNode, ControlState, control};
use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, KeyDownEvent, Pixels, Role, SharedString,
    Stateful, Window, div, prelude::*, px,
};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// 表示文字と操作状態を持つ選択肢。
#[derive(Clone, Debug)]
pub struct NavigationOption {
    name: crate::AccessibleName,
    disabled: bool,
}
impl NavigationOption {
    /// 表示文字と無効状態から作る。
    pub fn new(label: impl Into<SharedString>, disabled: bool) -> Self {
        Self {
            name: crate::AccessibleName::new(label),
            disabled,
        }
    }
    accessible_name!();
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
    name: crate::AccessibleName,
    items: Vec<NavigationOption>,
    selected: Option<usize>,
    vertical: bool,
    dialog: Option<bool>,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}
impl Navigation {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            items: Vec::new(),
            selected: None,
            vertical: false,
            dialog: None,
            on_change: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }
    accessible_name!();

    /// 行き先を足す。
    pub fn item(mut self, label: impl Into<SharedString>, disabled: bool) -> Self {
        self.items.push(NavigationOption {
            name: crate::AccessibleName::new(label),
            disabled,
        });
        self
    }
    /// 名前を上書きできる項目を加える。
    pub fn destination(mut self, item: NavigationOption) -> Self {
        self.items.push(item);
        self
    }

    /// 項目の読み上げ名を指定する。表示文字を優先し、表示ラベルを名前に含める（WCAG 2.5.3）。
    pub fn item_aria_label(mut self, index: usize, label: impl Into<SharedString>) -> Self {
        if let Some(item) = self.items.get_mut(index) {
            item.name.set_aria_label(label);
        }
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
                .with_name(&self.name)
                .modal(modal),
            None => A11yNode::new(Role::Navigation).with_name(&self.name),
        }
    }
    /// 項目のアクセシビリティ情報。
    pub fn item_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let item = self.items.get(index)?;
        Some(
            A11yNode::new(Role::Tab)
                .with_name(&item.name)
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
            let roving = Roving::new(
                ids.iter()
                    .cloned()
                    .zip(self.items.iter().map(|item| item.disabled)),
                self.selected,
                window,
                cx,
            );
            let mut items = Vec::with_capacity(self.items.len());
            for (index, item) in self.items.iter().enumerate() {
                let id = &ids[index];
                let (mut base, state) = control(
                    id,
                    item.disabled,
                    roving.is_tab_stop(index),
                    window,
                    cx,
                    self.reveal_margin,
                    format!("{}-{id}", self.group_prefix).into(),
                );
                if let Some(node) = self.item_a11y_node(index) {
                    base = base.a11y(node);
                }
                if !item.disabled {
                    let roving = roving.clone();
                    let vertical = self.vertical;
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let next = match (event.keystroke.key.as_str(), vertical) {
                            ("down", true) | ("right", false) => roving.next(index, true),
                            ("up", true) | ("left", false) => roving.next(index, false),
                            ("home", _) => roving.first(),
                            ("end", _) => roving.last(),
                            _ => return,
                        };
                        if let Some(next) = next {
                            cx.stop_propagation();
                            roving.focus(next, window, cx);
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
                initial_focus: roving.initial_focus(),
            }
        })
    }
}
