//! 項目を並べた選択ボタンの状態と操作。
use crate::control::Roving;
use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    AnyElement, App, Div, ElementId, KeyDownEvent, Pixels, Role, SharedString, Stateful, Window,
    div, prelude::*, px,
};
use std::rc::Rc;

#[derive(Clone, Debug)]
/// 表示文字と操作状態を持つ選択肢。
pub struct SegmentOption {
    label: SharedString,
    aria_label: Option<Option<SharedString>>,
    disabled: bool,
}
impl SegmentOption {
    /// 表示文字と無効状態から作る。
    pub fn new(label: impl Into<SharedString>, disabled: bool) -> Self {
        Self {
            label: label.into(),
            aria_label: None,
            disabled,
        }
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// 見た目を持たない選択肢のグループ。
///
/// 単一選択（既定）は RadioGroup として振る舞い、矢印キーで選択が移る。Tab で止まるのは選択中の項目だけ。
/// 複数選択（[`Self::multi_select`]）は各項目がトグルボタンで、矢印キーでフォーカスだけが移る。
pub struct SegmentedButton {
    id: ElementId,
    label: Option<SharedString>,
    segments: Vec<SegmentOption>,
    selected: Vec<usize>,
    multi: bool,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}

impl SegmentedButton {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            segments: Vec::new(),
            selected: Vec::new(),
            multi: false,
            on_change: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 項目を加える。
    pub fn segment(mut self, label: impl Into<SharedString>, disabled: bool) -> Self {
        self.segments.push(SegmentOption {
            label: label.into(),
            aria_label: None,
            disabled,
        });
        self
    }
    /// 名前を上書きできる項目を加える。
    pub fn segment_item(mut self, item: SegmentOption) -> Self {
        self.segments.push(item);
        self
    }

    /// 項目の読み上げ名を指定する。表示文字を優先し、表示ラベルを名前に含める（WCAG 2.5.3）。
    pub fn segment_aria_label(mut self, index: usize, label: impl Into<SharedString>) -> Self {
        if let Some(item) = self.segments.get_mut(index) {
            let label = label.into();
            item.aria_label = Some((!label.is_empty()).then_some(label));
        }
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
        A11yNode::new(role).with_label(self.label.clone())
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
            node.with_label(
                segment
                    .aria_label
                    .clone()
                    .unwrap_or_else(|| Some(segment.label.clone())),
            )
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
            let roving = Roving::new(
                self.segments
                    .iter()
                    .enumerate()
                    .map(|(index, item)| (self.segment_id(index), item.disabled)),
                self.selected.iter().copied(),
                window,
                cx,
            );
            let mut row = div().id(self.id.clone()).a11y(self.a11y_node());
            for (index, segment) in self.segments.iter().enumerate() {
                let id = self.segment_id(index);
                let (mut base, state) = control(
                    &id,
                    segment.disabled,
                    self.multi || roving.is_tab_stop(index),
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
                    let roving = roving.clone();
                    let multi = self.multi;
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let forward = match event.keystroke.key.as_str() {
                            "right" | "down" => true,
                            "left" | "up" => false,
                            _ => return,
                        };
                        let Some(next) = roving.next(index, forward) else {
                            return;
                        };
                        cx.stop_propagation();
                        roving.focus(next, window, cx);
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
