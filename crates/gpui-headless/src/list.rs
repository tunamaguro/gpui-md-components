//! リストの項目、キーボード操作とスクロール。
use crate::{
    A11yExt, A11yNode, ControlState, NestedScroll, ScrollScope, control, focus_handle,
    scroll_handle,
};
use gpui::{
    AnyElement, App, Div, Edges, ElementId, Entity, FocusHandle, KeyDownEvent, Pixels, Role,
    ScrollHandle, ScrollStrategy, SharedString, Stateful, UniformListScrollHandle, Window, div,
    prelude::*, px, uniform_list,
};
use std::{ops::Range, rc::Rc};

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
/// リストの項目の名前と操作。
#[derive(Clone)]
pub struct ListItem {
    name: crate::AccessibleName,
    description: Option<SharedString>,
    selected: bool,
    disabled: bool,
    on_select: Option<Handler>,
}
impl ListItem {
    /// 読み上げる名前から作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            name: crate::AccessibleName::new(label),
            description: None,
            selected: false,
            disabled: false,
            on_select: None,
        }
    }
    /// 補足の説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    accessible_name!();

    /// 選択状態を指定する。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// 無効状態を指定する。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// 選択時の処理を指定する。
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
    fn focusable(&self) -> bool {
        self.on_select.is_some() && !self.disabled
    }
    /// 項目のアクセシビリティ情報。
    pub fn a11y_node(&self, index: usize, count: usize) -> A11yNode {
        let mut node = A11yNode::new(Role::ListItem)
            .with_name(&self.name)
            .selected(self.selected)
            .position(index, count)
            .disabled(self.disabled);
        if let Some(description) = &self.description {
            node = node.description(description.clone());
        }
        node
    }
}
/// 操作を付けた項目と描画に使う状態。
pub struct ListRow {
    /// 項目の土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 安定した項目の ID。
    pub id: ElementId,
    /// 項目の位置。
    pub index: usize,
    /// 選択されているか。
    pub selected: bool,
}
fn list_step(
    mut focusable: impl FnMut(usize) -> bool,
    count: usize,
    from: usize,
    key: &str,
) -> Option<usize> {
    let mut candidates: Box<dyn Iterator<Item = usize>> = match key {
        "down" => Box::new(from + 1..count),
        "up" => Box::new((0..from).rev()),
        "home" => Box::new(0..count),
        "end" => Box::new((0..count).rev()),
        _ => return None,
    };
    candidates.find(|&index| focusable(index))
}
fn item_id(list: &ElementId, key: &SharedString) -> ElementId {
    ElementId::NamedChild(list.clone().into(), key.clone())
}
struct RowOptions<'a> {
    id: ElementId,
    index: usize,
    count: usize,
    tab_stop: bool,
    margin: Pixels,
    prefix: &'a str,
}
fn row(
    item: ListItem,
    options: RowOptions<'_>,
    on_key: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> ListRow {
    let RowOptions {
        id,
        index,
        count,
        tab_stop,
        margin,
        prefix,
    } = options;
    let node = item.a11y_node(index, count);
    let focusable = item.focusable();
    let (base, state) = control(
        &id,
        !focusable,
        tab_stop,
        window,
        cx,
        margin,
        format!("{prefix}-{id}").into(),
    );
    let mut base = base.a11y(node);
    if let Some(on_select) = item.on_select.filter(|_| focusable) {
        base = base
            .on_click(move |_, window, cx| on_select(window, cx))
            .on_key_down(on_key);
    }
    ListRow {
        base,
        state,
        id,
        index,
        selected: item.selected,
    }
}
/// 全項目を描くリスト。
pub struct List {
    id: ElementId,
    name: crate::AccessibleName,
    items: Vec<(SharedString, ListItem)>,
    max_height: Option<Pixels>,
    content_padding: Edges<Pixels>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}
impl List {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            items: Vec::new(),
            max_height: None,
            content_padding: Edges::default(),
            reveal_margin: px(0.),
            group_prefix: "control".into(),
        }
    }
    accessible_name!();

    /// 安定したキーと項目を加える。
    pub fn item(mut self, key: impl Into<SharedString>, item: ListItem) -> Self {
        self.items.push((key.into(), item));
        self
    }
    /// スクロール領域の最大の高さ。
    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = Some(height.max(px(0.)));
        self
    }
    /// スクロールする内容の内側の余白。
    pub fn content_padding(mut self, padding: Edges<Pixels>) -> Self {
        self.content_padding = padding;
        self
    }
    /// フォーカス位置を表示する際の余白。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// 項目のグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }
    /// リストのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::List).with_name(&self.name)
    }
    /// 項目の描画を受け取り、スクロール可能なリストを組み立てる。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(ListRow, &mut Window, &mut App) -> AnyElement,
    ) -> ListBase {
        window.with_element_namespace(self.id.clone(), |window| {
            let count = self.items.len();
            let focusable: Rc<[bool]> = self
                .items
                .iter()
                .map(|(_, item)| item.focusable())
                .collect();
            let target = self
                .items
                .iter()
                .position(|(_, item)| item.selected && item.focusable())
                .or_else(|| focusable.iter().position(|&f| f));
            let handles: Rc<[FocusHandle]> = self
                .items
                .iter()
                .map(|(key, _)| focus_handle(item_id(&self.id, key), window, cx))
                .collect();
            let scroll = self.max_height.map(|_| scroll_handle(&self.id, window, cx));
            let mut base = div()
                .id(self.id.clone())
                .a11y(self.a11y_node())
                .flex()
                .flex_col();
            let padding = self.content_padding;
            let mut contents = div()
                .flex()
                .flex_col()
                .pt(padding.top)
                .pr(padding.right)
                .pb(padding.bottom)
                .pl(padding.left);
            for (index, (key, item)) in self.items.into_iter().enumerate() {
                let handles = handles.clone();
                let focusable = focusable.clone();
                let on_key = move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                    if let Some(handle) = list_step(
                        |i| focusable.get(i).copied().unwrap_or(false),
                        count,
                        index,
                        event.keystroke.key.as_str(),
                    )
                    .and_then(|i| handles.get(i))
                    {
                        cx.stop_propagation();
                        window.focus(handle, cx);
                    }
                };
                let id = item_id(&self.id, &key);
                let row = row(
                    item,
                    RowOptions {
                        id: id.clone(),
                        index,
                        count,
                        tab_stop: target == Some(index),
                        margin: self.reveal_margin,
                        prefix: &self.group_prefix,
                    },
                    on_key,
                    window,
                    cx,
                );
                contents = contents
                    .child(window.with_element_namespace(id, |window| render(row, window, cx)));
            }
            base = base.child(contents);
            if let (Some(height), Some(scroll)) = (self.max_height, &scroll) {
                let wheel = NestedScroll::new(&self.id, scroll.clone(), window, cx);
                base = base
                    .max_h(height)
                    .overflow_y_scroll()
                    .track_scroll(scroll)
                    .on_scroll(wheel.on_scroll())
                    .child(wheel.capture());
            }
            ListBase { base, scroll }
        })
    }
}
/// 組み立てたリストと、見た目のスクロールバーに渡すスクロール状態。
pub struct ListBase {
    /// リストの土台。
    pub base: Stateful<Div>,
    /// 高さを制限した場合のスクロール状態。
    pub scroll: Option<ScrollHandle>,
}
impl ListBase {
    /// 見た目を追加し、フォーカス追従の領域を付ける。
    pub fn finish(
        self,
        style: impl FnOnce(Stateful<Div>, Option<&ScrollHandle>) -> Stateful<Div>,
    ) -> AnyElement {
        let base = style(self.base, self.scroll.as_ref());
        match self.scroll {
            Some(scroll) => ScrollScope::new(scroll, base).into_any_element(),
            None => base.into_any_element(),
        }
    }
}
type ItemBuilder<T> = Rc<dyn Fn(usize, &mut App) -> (SharedString, ListItem, T)>;
/// 表示範囲だけを描く、均一な高さの項目のリスト。
pub struct LazyList<T> {
    id: ElementId,
    name: crate::AccessibleName,
    count: usize,
    scroll: UniformListScrollHandle,
    build: ItemBuilder<T>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}
struct LazyState {
    active: usize,
    pending: Option<usize>,
}
impl<T: 'static> LazyList<T> {
    /// ID、項目数、スクロール状態と項目の生成関数から作る。
    pub fn new(
        id: impl Into<ElementId>,
        count: usize,
        scroll: &UniformListScrollHandle,
        build: impl Fn(usize, &mut App) -> (SharedString, ListItem, T) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            count,
            scroll: scroll.clone(),
            build: Rc::new(build),
            reveal_margin: px(0.),
            group_prefix: "control".into(),
        }
    }
    accessible_name!();

    /// フォーカス位置を表示する際の余白。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// 項目のグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }
    /// リストのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::List).with_name(&self.name)
    }
    /// 項目の描画を受け取り、仮想リストを組み立てる。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        render: impl Fn(ListRow, T, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Stateful<Div> {
        window.with_element_namespace(self.id.clone(), |window| {
            let state: Entity<LazyState> = window.use_keyed_state(
                ElementId::NamedChild(self.id.clone().into(), "state".into()),
                cx,
                |_, _| LazyState {
                    active: 0,
                    pending: None,
                },
            );
            let list_id = self.id.clone();
            let count = self.count;
            let build = self.build.clone();
            let scroll = self.scroll.clone();
            let margin = self.reveal_margin;
            let prefix = self.group_prefix.clone();
            let wheel = NestedScroll::new(
                &self.id,
                self.scroll.0.borrow().base_handle.clone(),
                window,
                cx,
            );
            let items = uniform_list(
                ElementId::NamedChild(self.id.clone().into(), "items".into()),
                count,
                move |range: Range<usize>, window, cx| {
                    let (active, pending) = {
                        let state = state.read(cx);
                        (state.active, state.pending)
                    };
                    let built: Vec<_> = range.map(|index| (index, build(index, cx))).collect();
                    let target = if built
                        .iter()
                        .any(|(i, (_, item, _))| *i == active && item.focusable())
                    {
                        Some(active)
                    } else {
                        built
                            .iter()
                            .find(|(_, (_, item, _))| item.focusable())
                            .map(|(i, _)| *i)
                    };
                    built
                        .into_iter()
                        .map(|(index, (key, item, data))| {
                            let id = item_id(&list_id, &key);
                            if pending == Some(index) {
                                let handle = focus_handle(id.clone(), window, cx);
                                window.focus(&handle, cx);
                                state.update(cx, |state, _| {
                                    state.pending = None;
                                    state.active = index;
                                });
                            }
                            let on_key = {
                                let state = state.clone();
                                let build = build.clone();
                                let scroll = scroll.clone();
                                move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                                    if let Some(next) = list_step(
                                        |i| build(i, cx).1.focusable(),
                                        count,
                                        index,
                                        event.keystroke.key.as_str(),
                                    ) {
                                        cx.stop_propagation();
                                        scroll.scroll_to_item(next, ScrollStrategy::Nearest);
                                        state.update(cx, |state, _| state.pending = Some(next));
                                        window.refresh();
                                    }
                                }
                            };
                            let activate = state.clone();
                            let row = row(
                                item,
                                RowOptions {
                                    id: id.clone(),
                                    index,
                                    count,
                                    tab_stop: target == Some(index),
                                    margin,
                                    prefix: &prefix,
                                },
                                on_key,
                                window,
                                cx,
                            );
                            let child = window
                                .with_element_namespace(id, |window| render(row, data, window, cx));
                            div()
                                .w_full()
                                .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                    activate.update(cx, |state, _| state.active = index)
                                })
                                .child(child)
                        })
                        .collect()
                },
            )
            .track_scroll(&self.scroll)
            .on_scroll(wheel.on_scroll())
            .size_full();
            div()
                .id(self.id.clone())
                .a11y(self.a11y_node())
                .flex()
                .flex_col()
                .child(items)
                .child(wheel.capture())
        })
    }
}
