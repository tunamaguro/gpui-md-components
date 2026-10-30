//! 項目を並べた選択ボタンの状態と操作。
use crate::{A11yExt, A11yNode, ControlState, control, focus_handle, step, toggled};
use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, KeyDownEvent, Pixels, Role, SharedString,
    Stateful, Window, div, prelude::*, px,
};
use std::rc::Rc;

#[derive(Clone, Debug)]
struct SegmentOption {
    label: SharedString,
    disabled: bool,
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// 見た目を持たない選択肢のグループ。
///
/// 単一選択（既定）は RadioGroup として振る舞い、矢印キーで選択が移る。Tab で止まるのは選択中の項目だけ。
/// 複数選択（[`Self::multi_select`]）は各項目がトグルボタンで、矢印キーでフォーカスだけが移る。
pub struct SegmentedButton {
    id: ElementId,
    label: SharedString,
    segments: Vec<SegmentOption>,
    selected: Vec<usize>,
    multi: bool,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}

impl SegmentedButton {
    /// ID とグループのラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            segments: Vec::new(),
            selected: Vec::new(),
            multi: false,
            on_change: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }

    /// 項目を加える。
    pub fn segment(mut self, label: impl Into<SharedString>, disabled: bool) -> Self {
        self.segments.push(SegmentOption {
            label: label.into(),
            disabled,
        });
        self
    }

    /// 選択中の項目（単一選択なら 1 つ）。
    pub fn selected(mut self, selected: impl IntoIterator<Item = usize>) -> Self {
        self.selected = selected.into_iter().collect();
        self
    }

    /// 複数選択にする。
    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// 項目が押されたときの処理。引数は押された項目の位置。
    /// 単一選択ならその項目を選択に、複数選択ならその項目の選択を反転する。
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
        let role = if self.multi {
            Role::Group
        } else {
            Role::RadioGroup
        };
        A11yNode::new(role).label(self.label.clone())
    }

    /// 項目のアクセシビリティ情報。
    pub fn segment_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let segment = self.segments.get(index)?;
        let selected = self.selected.contains(&index);
        let node = if self.multi {
            A11yNode::new(Role::Button).toggled(toggled(selected))
        } else {
            A11yNode::new(Role::RadioButton).toggled(toggled(selected))
        };
        Some(
            node.label(segment.label.clone())
                .position(index, self.segments.len())
                .disabled(segment.disabled),
        )
    }

    fn segment_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            SharedString::from(format!("{index}")),
        )
    }
}

/// 項目の要素と描画に使う状態。
pub struct SegmentItem {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 項目の位置。
    pub index: usize,
    /// 選択されているか。
    pub selected: bool,
}
impl SegmentedButton {
    /// 項目の描画を受け取り、操作を付けたグループを返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(SegmentItem, &mut Window, &mut App) -> AnyElement,
    ) -> Stateful<Div> {
        window.with_element_namespace(self.id.clone(), |window| {
            let count = self.segments.len();
            let enabled: Rc<[bool]> = self.segments.iter().map(|s| !s.disabled).collect();
            let tab_target = self
                .selected
                .iter()
                .copied()
                .find(|&i| enabled.get(i).copied().unwrap_or(false))
                .or_else(|| enabled.iter().position(|&e| e));
            let handles: Rc<[FocusHandle]> = (0..count)
                .map(|index| focus_handle(self.segment_id(index), window, cx))
                .collect();
            let mut row = div().id(self.id.clone()).a11y(self.a11y_node());
            for (index, segment) in self.segments.iter().enumerate() {
                let id = self.segment_id(index);
                let (mut base, state) = control(
                    &id,
                    segment.disabled,
                    self.multi || tab_target == Some(index),
                    window,
                    cx,
                    self.reveal_margin,
                    format!("{}-{id}", self.group_prefix).into(),
                );
                if let Some(node) = self.segment_a11y_node(index) {
                    base = base.a11y(node);
                }
                if !segment.disabled
                    && let Some(on_change) = self.on_change.clone()
                {
                    let on_click = on_change.clone();
                    base = base.on_click(move |_, window, cx| on_click(index, window, cx));
                    let handles = handles.clone();
                    let enabled = enabled.clone();
                    let multi = self.multi;
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let forward = match event.keystroke.key.as_str() {
                            "right" | "down" => true,
                            "left" | "up" => false,
                            _ => return,
                        };
                        let Some(next) = step(&enabled, index, forward) else {
                            return;
                        };
                        cx.stop_propagation();
                        if let Some(handle) = handles.get(next) {
                            window.focus(handle, cx);
                        }
                        if !multi {
                            on_change(next, window, cx);
                        }
                    });
                }
                row = row.child(window.with_element_namespace(id, |window| {
                    render(
                        SegmentItem {
                            base,
                            state,
                            index,
                            selected: self.selected.contains(&index),
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
