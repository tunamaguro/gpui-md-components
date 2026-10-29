//! List（1〜3 行の項目、仮想化した LazyList）。

use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, Div, ElementId, Entity, FocusHandle, IntoElement, KeyDownEvent, RenderOnce, Role,
    ScrollStrategy, SharedString, Stateful, UniformListScrollHandle, Window, div, prelude::*, px,
    uniform_list,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::list as tokens;
use md3_theme::md3_tokens::layout;

use crate::a11y::{A11yExt, A11yNode};
use crate::control::control;
use crate::{FocusRing, ForcedState, Icon, IconName, StateLayer, StyledExt, focus_handle};

type SelectHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// 項目の先頭に置くもの。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListLeading {
    /// アイコン（24dp）。
    Icon(IconName),
    /// 頭文字のアバター（40dp の丸）。
    Avatar(SharedString),
}

/// 項目の末尾に置くもの。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListTrailing {
    /// アイコン（24dp）。
    Icon(IconName),
    /// 短い文字（件数、時刻など）。
    Text(SharedString),
}

/// リストの 1 項目。[`List`] か [`LazyList`] に入れて使う。
///
/// 行数は内容から決まる。上書き（overline）と補助テキストの両方があれば 3 行（88dp）、
/// どちらか一方なら 2 行（72dp）、見出しだけなら 1 行（56dp）。
#[derive(Clone)]
pub struct ListItem {
    headline: SharedString,
    overline: Option<SharedString>,
    supporting_text: Option<SharedString>,
    leading: Option<ListLeading>,
    trailing: Option<ListTrailing>,
    selected: bool,
    disabled: bool,
    on_select: Option<SelectHandler>,
    forced: Option<ForcedState>,
}

impl ListItem {
    /// 見出しから作る。
    pub fn new(headline: impl Into<SharedString>) -> Self {
        Self {
            headline: headline.into(),
            overline: None,
            supporting_text: None,
            leading: None,
            trailing: None,
            selected: false,
            disabled: false,
            on_select: None,
            forced: None,
        }
    }

    /// 見出しの上の小さな文字。
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(text.into());
        self
    }

    /// 見出しの下の補助テキスト。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 先頭に置くもの。
    pub fn leading(mut self, leading: ListLeading) -> Self {
        self.leading = Some(leading);
        self
    }

    /// 末尾に置くもの。
    pub fn trailing(mut self, trailing: ListTrailing) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// 選択中にする（`SecondaryContainer` で塗る）。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押したとき（クリック、Enter、Space）の処理。付けると項目がフォーカスを持つ。
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// 見本用に状態を固定する。
    #[doc(hidden)]
    pub fn forced(mut self, forced: ForcedState) -> Self {
        self.forced = Some(forced);
        self
    }

    /// 行数（1〜3）。
    pub fn line_count(&self) -> usize {
        match (self.overline.is_some(), self.supporting_text.is_some()) {
            (true, true) => 3,
            (false, false) => 1,
            _ => 2,
        }
    }

    /// 矢印キーで止まれるか。
    fn focusable(&self) -> bool {
        self.on_select.is_some() && !self.disabled
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self, index: usize, count: usize) -> A11yNode {
        A11yNode::new(Role::ListItem)
            .label(self.headline.clone())
            .selected(self.selected)
            .position(index, count)
            .disabled(self.disabled)
    }

    fn render(
        self,
        id: ElementId,
        tab_stop: bool,
        (index, count): (usize, usize),
        on_key: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let node = self.a11y_node(index, count);
        let lines = self.line_count();
        let focusable = self.focusable();
        let (base, control) = control(&id, !focusable, tab_stop, self.forced, window, cx);
        let theme = cx.md3().clone();
        let disabled = |role: ColorRole, opacity: f32| theme.color(role).opacity(opacity);
        let headline_color = if self.disabled {
            disabled(
                tokens::ITEM_DISABLED_LABEL_TEXT_COLOR,
                tokens::ITEM_DISABLED_LABEL_TEXT_OPACITY,
            )
        } else if self.selected {
            theme.color(tokens::ITEM_SELECTED_LABEL_TEXT_COLOR)
        } else {
            theme.color(tokens::ITEM_LABEL_TEXT_COLOR)
        };
        let secondary_color = |role: ColorRole| {
            if self.disabled {
                disabled(
                    tokens::ITEM_DISABLED_SUPPORTING_TEXT_COLOR,
                    tokens::ITEM_DISABLED_SUPPORTING_TEXT_OPACITY,
                )
            } else {
                theme.color(role)
            }
        };
        let icon_color = |role: ColorRole| {
            if self.disabled {
                disabled(
                    tokens::ITEM_DISABLED_LEADING_ICON_COLOR,
                    tokens::ITEM_DISABLED_LEADING_ICON_OPACITY,
                )
            } else if self.selected {
                theme.color(tokens::ITEM_SELECTED_LEADING_ICON_COLOR)
            } else {
                theme.color(role)
            }
        };
        let (height, vertical) = match lines {
            1 => (
                tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_VERTICAL_SPACE,
            ),
            2 => (
                tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_VERTICAL_SPACE,
            ),
            _ => (
                tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_THREE_LINE_VERTICAL_SPACE,
            ),
        };
        let radius = if self.selected {
            theme.corner(tokens::ITEM_SELECTED_CONTAINER_SHAPE)
        } else {
            theme.corner(tokens::ITEM_CONTAINER_SHAPE)
        };

        let leading = self.leading.map(|leading| {
            let element = match leading {
                ListLeading::Icon(icon) => Icon::new(icon)
                    .size(px(tokens::ITEM_LEADING_ICON_SIZE))
                    .tint(icon_color(tokens::ITEM_LEADING_ICON_COLOR))
                    .into_any_element(),
                ListLeading::Avatar(initials) => div()
                    .size(px(tokens::ITEM_LEADING_AVATAR_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .corner_radius(theme.corner(tokens::ITEM_LEADING_AVATAR_SHAPE))
                    .bg(theme.color(tokens::ITEM_LEADING_AVATAR_COLOR))
                    .text_color(theme.color(tokens::ITEM_LEADING_AVATAR_LABEL_COLOR))
                    .type_style(&theme.text_style(tokens::ITEM_LEADING_AVATAR_LABEL_FONT))
                    .when(self.disabled, |this| {
                        this.opacity(tokens::ITEM_DISABLED_LEADING_ICON_OPACITY)
                    })
                    .child(initials)
                    .into_any_element(),
            };
            div()
                .flex_none()
                .pr(px(layout::LIST_ITEM_LEADING_END_SPACE))
                .child(element)
        });
        let trailing = self.trailing.map(|trailing| {
            let element = match trailing {
                ListTrailing::Icon(icon) => Icon::new(icon)
                    .size(px(tokens::ITEM_TRAILING_ICON_SIZE))
                    .tint(icon_color(tokens::ITEM_TRAILING_ICON_COLOR))
                    .into_any_element(),
                ListTrailing::Text(text) => div()
                    .type_style(&theme.text_style(tokens::ITEM_TRAILING_SUPPORTING_TEXT_FONT))
                    .text_color(secondary_color(tokens::ITEM_TRAILING_SUPPORTING_TEXT_COLOR))
                    .child(text)
                    .into_any_element(),
            };
            div()
                .flex_none()
                .pl(px(layout::LIST_ITEM_TRAILING_START_SPACE))
                .child(element)
        });
        let text = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .when_some(self.overline, |this, overline| {
                this.child(
                    div()
                        .type_style(&theme.text_style(tokens::ITEM_OVERLINE_FONT))
                        .text_color(secondary_color(tokens::ITEM_OVERLINE_COLOR))
                        .child(overline),
                )
            })
            .child(
                div()
                    .type_style(&theme.text_style(tokens::ITEM_LABEL_TEXT_FONT))
                    .text_color(headline_color)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(self.headline),
            )
            .when_some(self.supporting_text, |this, text| {
                this.child(
                    div()
                        .type_style(&theme.text_style(tokens::ITEM_SUPPORTING_TEXT_FONT))
                        .text_color(secondary_color(tokens::ITEM_SUPPORTING_TEXT_COLOR))
                        .when(lines < 3, |this| {
                            this.whitespace_nowrap().overflow_hidden().text_ellipsis()
                        })
                        .child(text),
                )
            });

        let mut item = base
            .debug_selector(|| format!("md3-list-item-{id}"))
            .a11y(node)
            .w_full()
            .min_h(px(height))
            .flex()
            .when(lines < 3, |this| this.items_center())
            .when(lines == 3, |this| this.items_start())
            .py(px(vertical))
            .pl(px(layout::LIST_ITEM_START_SPACE))
            .pr(px(layout::LIST_ITEM_END_SPACE))
            .corner_radius(radius)
            .bg(theme.color(if self.selected {
                tokens::ITEM_SELECTED_CONTAINER_COLOR
            } else {
                tokens::ITEM_CONTAINER_COLOR
            }))
            .children(leading)
            .child(text)
            .children(trailing);
        if let Some(on_select) = self.on_select.filter(|_| focusable) {
            item = item
                .child(
                    StateLayer::new(
                        SharedString::from(format!("{}-layer", control.group)),
                        control.group.clone(),
                        ColorRole::OnSurface,
                    )
                    .radius(radius)
                    .focused(control.focus_visible)
                    .forced(self.forced),
                )
                .child(
                    FocusRing::new(radius)
                        .inward()
                        .visible(control.focus_visible),
                )
                .on_click(move |_, window, cx| on_select(window, cx))
                .on_key_down(on_key);
        } else if self.forced.is_some() {
            item = item.child(
                FocusRing::new(radius)
                    .inward()
                    .visible(control.focus_visible),
            );
        }
        item
    }
}

/// 矢印キーで次に止まる項目（端で止まり、回らない）。`Home` / `End` は端の項目。
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

/// 項目の ID（リストの ID とデータのキーから作る）。
fn item_id(list: &ElementId, key: &SharedString) -> ElementId {
    ElementId::NamedChild(list.clone().into(), key.clone())
}

/// 全項目を描くリスト。項目が多いときは [`LazyList`] を使う。
///
/// 押せる項目（[`ListItem::on_select`]）があれば Tab ではリスト全体で 1 回だけ止まり、
/// 中は ↑↓ / Home / End で移る（選択中の項目、無ければ最初の押せる項目に止まる）。
#[derive(IntoElement)]
pub struct List {
    max_height: Option<gpui::Pixels>,
    scrollbar: bool,
    id: ElementId,
    label: SharedString,
    items: Vec<(SharedString, ListItem)>,
}

impl List {
    /// 高さを制限し、超過する項目をスクロール可能にする。
    pub fn max_height(mut self, height: gpui::Pixels) -> Self {
        self.max_height = Some(height.max(px(0.0)));
        self
    }

    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID と名前（読み上げられる）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            scrollbar: true,
            max_height: None,
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
        }
    }

    /// 項目を足す。`key` はデータの安定したキー（並び替えても変わらないもの）。
    pub fn item(mut self, key: impl Into<SharedString>, item: ListItem) -> Self {
        self.items.push((key.into(), item));
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::List).label(self.label.clone())
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.items.len();
        let focusable: Rc<[bool]> = self
            .items
            .iter()
            .map(|(_, item)| item.focusable())
            .collect();
        let tab_target = self
            .items
            .iter()
            .position(|(_, item)| item.selected && item.focusable())
            .or_else(|| focusable.iter().position(|&f| f));
        let handles: Rc<[FocusHandle]> = self
            .items
            .iter()
            .map(|(key, _)| focus_handle(item_id(&self.id, key), window, cx))
            .collect();
        // 高さを限ったリストはスクロールする。矢印・Tab で移った項目を見える範囲に入れる。
        let scroll = self
            .max_height
            .map(|_| crate::scroll::scroll_handle(&self.id, window, cx));
        let reveal = scroll
            .as_ref()
            .map(|_| crate::scroll::reveal_state(&self.id, window, cx));
        let mut list = div()
            .id(self.id.clone())
            .debug_selector(|| format!("md3-list-{}", self.id))
            .a11y(self.a11y_node())
            .flex()
            .flex_col();
        for (index, (key, item)) in self.items.into_iter().enumerate() {
            let reveal = scroll.clone().zip(reveal.clone()).map(|(scroll, reveal)| {
                crate::scroll::reveal_on_focus(scroll, handles[index].clone(), reveal)
                    .into_any_element()
            });
            let handles = handles.clone();
            let focusable = focusable.clone();
            let on_key = move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let next = list_step(
                    |i| focusable.get(i).copied().unwrap_or(false),
                    count,
                    index,
                    event.keystroke.key.as_str(),
                );
                if let Some(handle) = next.and_then(|next| handles.get(next)) {
                    cx.stop_propagation();
                    window.focus(handle, cx);
                }
            };
            list = list.child(
                item.render(
                    item_id(&self.id, &key),
                    tab_target == Some(index),
                    (index, count),
                    on_key,
                    window,
                    cx,
                )
                .children(reveal),
            );
        }
        if let (Some(height), Some(scroll)) = (self.max_height, scroll) {
            let wheel = crate::scroll::NestedScroll::new(&self.id, scroll.clone(), window, cx);
            list = list
                .max_h(height)
                .overflow_y_scroll()
                .track_scroll(&scroll)
                .on_scroll(wheel.on_scroll())
                .child(wheel.capture())
                .when(self.scrollbar, |this| {
                    this.child(crate::Scrollbar::new(
                        ElementId::NamedChild(self.id.clone().into(), "scrollbar".into()),
                        scroll,
                    ))
                });
        }
        list
    }
}

type ItemBuilder = Rc<dyn Fn(usize, &mut App) -> (SharedString, ListItem)>;

/// 見えている項目だけを描くリスト（gpui の `uniform_list`）。項目の高さはすべて同じにすること。
///
/// 大きさ（高さ）は呼び出し側が指定する。キー操作は [`List`] と同じで、
/// 見えていない項目へ移るときはその項目までスクロールしてからフォーカスする。
/// Tab で止まるのは最後にフォーカスした項目で、それが見えていなければ最初に見えている押せる項目。
#[derive(IntoElement)]
pub struct LazyList {
    scrollbar: bool,
    id: ElementId,
    label: SharedString,
    count: usize,
    scroll: UniformListScrollHandle,
    build: ItemBuilder,
    base: gpui::Div,
}

/// LazyList の描画をまたいで残す状態。
struct LazyListState {
    /// Tab で止まる項目（最後にフォーカスした項目）。
    active: usize,
    /// 描いたらフォーカスを移す項目。
    pending: Option<usize>,
}

impl LazyList {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID、名前、項目数、スクロールの状態、項目を作る関数から作る。
    ///
    /// `build(index)` は項目のキーと中身を返す。キーはデータの安定したキーにする。
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        count: usize,
        scroll: &UniformListScrollHandle,
        build: impl Fn(usize, &mut App) -> (SharedString, ListItem) + 'static,
    ) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            label: label.into(),
            count,
            scroll: scroll.clone(),
            build: Rc::new(build),
            base: div(),
        }
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::List).label(self.label.clone())
    }
}

impl gpui::Styled for LazyList {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for LazyList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state: Entity<LazyListState> = window.use_keyed_state(
            ElementId::NamedChild(self.id.clone().into(), "state".into()),
            cx,
            |_, _| LazyListState {
                active: 0,
                pending: None,
            },
        );
        let list_id = self.id.clone();
        let count = self.count;
        let build = self.build.clone();
        let scroll = self.scroll.clone();
        let wheel = crate::scroll::NestedScroll::new(
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
                let built: Vec<_> = range
                    .clone()
                    .map(|index| (index, build(index, cx)))
                    .collect();
                let active_is_visible = built
                    .iter()
                    .any(|(index, (_, item))| *index == active && item.focusable());
                let tab_target = if active_is_visible {
                    Some(active)
                } else {
                    built
                        .iter()
                        .find(|(_, (_, item))| item.focusable())
                        .map(|(index, _)| *index)
                };
                built
                    .into_iter()
                    .map(|(index, (key, item))| {
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
                                let next = list_step(
                                    |i| build(i, cx).1.focusable(),
                                    count,
                                    index,
                                    event.keystroke.key.as_str(),
                                );
                                if let Some(next) = next {
                                    cx.stop_propagation();
                                    scroll.scroll_to_item(next, ScrollStrategy::Nearest);
                                    state.update(cx, |state, _| state.pending = Some(next));
                                    window.refresh();
                                }
                            }
                        };
                        let activate = state.clone();
                        div()
                            .w_full()
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                                activate.update(cx, |state, _| state.active = index)
                            })
                            .child(item.render(
                                id,
                                tab_target == Some(index),
                                (index, count),
                                on_key,
                                window,
                                cx,
                            ))
                    })
                    .collect()
            },
        )
        .track_scroll(&self.scroll)
        .on_scroll(wheel.on_scroll())
        .size_full();
        let node = self.a11y_node();
        let mut base = self.base;
        let mut container = div();
        *container.style() = base.style().clone();
        container
            .id(self.id.clone())
            .a11y(node)
            .flex()
            .flex_col()
            .child(items)
            .child(wheel.capture())
            .when(self.scrollbar, |this| {
                this.child(crate::Scrollbar::new(
                    ElementId::NamedChild(self.id.clone().into(), "scrollbar".into()),
                    self.scroll.clone(),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_count_follows_the_content() {
        assert_eq!(ListItem::new("a").line_count(), 1);
        assert_eq!(ListItem::new("a").supporting_text("b").line_count(), 2);
        assert_eq!(ListItem::new("a").overline("o").line_count(), 2);
        assert_eq!(
            ListItem::new("a")
                .overline("o")
                .supporting_text("b")
                .line_count(),
            3
        );
    }

    #[test]
    fn arrows_stop_at_the_ends_and_skip_items_that_cannot_focus() {
        let focusable = [true, false, true, true];
        let f = |i: usize| focusable[i];
        assert_eq!(list_step(f, 4, 0, "down"), Some(2));
        assert_eq!(list_step(f, 4, 3, "down"), None);
        assert_eq!(list_step(f, 4, 2, "up"), Some(0));
        assert_eq!(list_step(f, 4, 0, "up"), None);
        assert_eq!(list_step(f, 4, 2, "home"), Some(0));
        assert_eq!(list_step(f, 4, 0, "end"), Some(3));
        assert_eq!(list_step(f, 4, 0, "left"), None);
    }

    #[test]
    fn items_are_list_items_with_their_position() {
        let node = ListItem::new("受信トレイ").selected(true).a11y_node(1, 3);
        assert_eq!(node.role, Role::ListItem);
        assert_eq!(node.selected, Some(true));
        assert_eq!(node.position, Some((2, 3)));
    }
}
