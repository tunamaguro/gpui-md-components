//! 一覧から値を 1 つ選ぶ、見た目を持たないセレクト

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, Div, Edges, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, Modifiers, MouseButton, ParentElement, Pixels, RenderOnce, SharedString, Size,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, canvas, div,
    prelude::FluentBuilder,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    navigation::{FocusNext, FocusPrevious},
    popup::{self, Anchor, ExpandedChangeHandler, FocusTarget, OpenFocus},
    roving::{GroupChild, RovingItem},
};

type ChangeHandler<T> = Rc<dyn Fn(&T, &mut Window, &mut App)>;

/// PageUp と PageDown で移る項目の数
const PAGE: usize = 10;

/// 一覧から値を 1 つ選ぶ、見た目を持たないセレクト (APG の Select-Only Combobox)
///
/// この要素は選ばれている値を表示し、押すと [`Select::listbox`] で渡した一覧を開く。
/// 選ばれている値と開いているかどうかは呼び出し側が持ち、[`Select::new`] に毎回渡す。
/// セレクト自身は状態を変えず、選択は [`Select::on_change`] で、開閉は [`Select::on_expanded_change`] で求める。
/// 選ばれている値の表示は `child` で加える
///
/// 閉じている間
///
/// - クリック・Enter・Space・修飾キーなしか Alt だけの下矢印キーで開くよう求める。
///   開くと、選ばれている項目 (無ければ最初のフォーカスできる項目) にフォーカスを移す
/// - 修飾キーなしの上矢印キーと Home で開くよう求めて最初の項目に、End で開くよう求めて最後の項目にフォーカスを移す
///
/// 開いている間の一覧
///
/// - 修飾キーなしの上下の矢印キーで前後の項目へ移る。端では移らない。
///   Home と End では最初と最後の項目へ、PageUp と PageDown では 10 個前と後 (無ければ端) の項目へ移る。
///   [`Availability::Disabled`] の項目は飛ばし、数えない。キーでフォーカスを移した項目が見えるようにスクロールする
/// - 項目にマウスが乗ると、その項目にフォーカスを移す
/// - 項目のクリック・Enter・Space・支援技術からのクリック操作と、Alt+上矢印キーで、
///   フォーカスをセレクトへ移し、その項目を選ぶよう求めて閉じるよう求める
/// - Tab と Shift+Tab で、フォーカスのある項目を選ぶよう求めて閉じるよう求め、
///   フォーカスをセレクトの次と前の Tab で到達できる要素へ移す
/// - 修飾キーなしの Escape で、選ばずに閉じるよう求める
/// - 一覧とセレクトの外でマウスのボタンを押すと、閉じるよう求める。押した先には、そのまま押下が届く
/// - 閉じたとき、フォーカスが一覧の中にあればセレクトへ戻す
/// - 選ばれている項目を選んでも [`Select::on_change`] は呼ばない
///
/// 一覧の位置は [`crate::menu::Menu`] と同じ規則で決め、一覧の幅はセレクトの幅以上にする。
///
/// 支援技術には role `combobox` として、開いているかと、選ばれている項目の名前を値として公開する。
/// 名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Select<T: PartialEq + 'static> {
    id: ElementId,
    inner: Stateful<Div>,
    selected: Option<T>,
    expanded: bool,
    on_change: Option<ChangeHandler<T>>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    listbox: SelectListbox<T>,
    availability: Availability,
    provided_focus_handle: Option<FocusHandle>,
}

impl<T: PartialEq + 'static> Select<T> {
    /// `selected` の値を持つ項目が選ばれ、`expanded` なら一覧を開いた状態で表示するセレクトを作る。
    /// `selected` が `None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Select::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, selected: Option<T>, expanded: bool) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            selected,
            expanded,
            on_change: None,
            on_expanded_change: None,
            listbox: SelectListbox::new(),
            availability: Availability::Enabled,
            provided_focus_handle: None,
        }
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 開くか閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、開くよう求められたら `true` を、閉じるよう求められたら `false` を渡す。
    /// 項目を選ぶよう求めるときは、[`Select::on_change`] の後に呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと。
    /// 設定しない場合、セレクトは押されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(handler));
        self
    }

    /// 開いたときに表示する一覧を設定する
    pub fn listbox(mut self, listbox: SelectListbox<T>) -> Self {
        self.listbox = listbox;
        self
    }

    /// セレクトを無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なセレクトは押されても開くよう求めず、マウスの左ボタンの押下を親に伝えず、支援技術には disabled として公開される。
    /// 開いている一覧は閉じられる
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// セレクトを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なセレクトは Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Select::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのセレクトのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合はセレクトが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for Select<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| {
                    cx.focus_handle().tab_index(0).tab_stop(true)
                })
                .read(cx)
                .clone()
        });
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let expanded = self.expanded;
        let listbox = self.listbox;
        let margin = listbox.paint_margin;
        let request = self.on_expanded_change;

        // 一覧の状態と項目のフォーカスは、同じ親に並ぶ別のセレクトと分けるため、セレクトの id の下に置く
        let (state, focus_handles) = window.with_element_namespace(self.id.clone(), |window| {
            let state = popup::use_state("listbox".into(), window, cx);
            let focus_handles: Vec<FocusHandle> = listbox
                .children
                .iter()
                .filter_map(|child| match child {
                    GroupChild::Item(option) => Some(option.focus_handle(window, cx)),
                    GroupChild::Other(_) => None,
                })
                .collect();
            (state, focus_handles)
        });

        let mut values = Vec::new();
        let mut labels = Vec::new();
        let mut items = Vec::new();
        let mut focus_handles = focus_handles.into_iter();
        let children: Vec<GroupChild<Stateful<Div>>> = listbox
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(option) => {
                    items.push(RovingItem {
                        focus_handle: focus_handles
                            .next()
                            .expect("項目ごとにフォーカスがあること"),
                        availability: option.availability,
                    });
                    values.push(option.value);
                    labels.push(option.label);
                    GroupChild::Item(option.inner)
                }
                GroupChild::Other(element) => GroupChild::Other(element),
            })
            .collect();
        let values: Rc<[T]> = values.into();
        let items: Rc<[RovingItem]> = items.into();
        // 項目ごとの、最後に描いたときの範囲
        let item_bounds: Rc<[Rc<Cell<Bounds<Pixels>>>]> =
            items.iter().map(|_| Rc::default()).collect();
        let selected_ix = values
            .iter()
            .position(|value| Some(value) == self.selected.as_ref());
        let first = items
            .iter()
            .position(|item| item.availability.is_focusable());
        let last = items
            .iter()
            .rposition(|item| item.availability.is_focusable());
        let default_target = selected_ix
            .filter(|&ix| items[ix].availability.is_focusable())
            .or(first);
        let target = move |open_focus: OpenFocus| match open_focus {
            OpenFocus::Default => default_target,
            OpenFocus::First => first,
            OpenFocus::Last => last,
        };
        let focus_target = {
            let items = items.clone();
            let item_bounds = item_bounds.clone();
            move |ix: usize| -> FocusTarget {
                (items[ix].focus_handle.clone(), item_bounds[ix].clone())
            }
        };
        // 選ばれている項目と無効な項目は、選ぶよう求めない
        let commit = {
            let items = items.clone();
            let on_change = self.on_change.clone();
            Rc::new(move |ix: usize, window: &mut Window, cx: &mut App| {
                if Some(ix) != selected_ix
                    && !items[ix].availability.is_disabled()
                    && let Some(on_change) = &on_change
                {
                    on_change(&values[ix], window, cx);
                }
            })
        };

        let popup = expanded.then(|| {
            let mut ix = 0;
            let elements: Vec<AnyElement> = children
                .into_iter()
                .map(|child| match child {
                    GroupChild::Item(inner) => {
                        let commit = commit.clone();
                        let request = request.clone();
                        let trigger_focus = focus_handle.clone();
                        let option_ix = ix;
                        let element = option_element(
                            inner,
                            &items[ix],
                            labels[ix].clone(),
                            selected_ix == Some(ix),
                            item_bounds[ix].clone(),
                            move |window: &mut Window, cx: &mut App| {
                                window.focus(&trigger_focus, cx);
                                commit(option_ix, window, cx);
                                if let Some(close) = &request {
                                    close(&false, window, cx);
                                }
                            },
                            window,
                        );
                        ix += 1;
                        element.into_any_element()
                    }
                    GroupChild::Other(element) => element,
                })
                .collect();
            let scroll_handle = state.read(cx).scroll_handle.clone();
            let trigger_width = state.read(cx).trigger_bounds.size.width;
            let key_items = items.clone();
            let key_bounds = item_bounds.clone();
            let key_commit = commit.clone();
            let key_request = request.clone();
            let key_trigger_focus = focus_handle.clone();
            let mut listbox_element = listbox
                .inner
                .role(accesskit::Role::ListBox)
                .min_w(trigger_width)
                .on_key_down(move |event, window, cx| {
                    let current = key_items
                        .iter()
                        .position(|item| item.focus_handle.is_focused(window));
                    let close = |window: &mut Window, cx: &mut App| {
                        if let Some(close) = &key_request {
                            close(&false, window, cx);
                        }
                    };
                    if event.keystroke.modifiers == Modifiers::alt() && event.keystroke.key == "up"
                    {
                        window.focus(&key_trigger_focus, cx);
                        if let Some(ix) = current {
                            key_commit(ix, window, cx);
                        }
                        close(window, cx);
                        cx.stop_propagation();
                        return;
                    }
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let step = |forward: bool, count: usize| match current {
                        Some(ix) => step(&key_items, ix, forward, count),
                        None => default_target,
                    };
                    let target = match event.keystroke.key.as_str() {
                        "down" => step(true, 1),
                        "up" => step(false, 1),
                        "pagedown" => step(true, PAGE),
                        "pageup" => step(false, PAGE),
                        "home" => first,
                        "end" => last,
                        "escape" => {
                            close(window, cx);
                            cx.stop_propagation();
                            return;
                        }
                        _ => return,
                    };
                    if let Some(ix) = target {
                        window.focus(&key_items[ix].focus_handle, cx);
                        popup::scroll_into_view(&scroll_handle, margin, key_bounds[ix].get());
                    }
                    cx.stop_propagation();
                });
            for forward in [true, false] {
                let items = items.clone();
                let commit = commit.clone();
                let request = request.clone();
                let trigger_focus = focus_handle.clone();
                let tab = move |window: &mut Window, cx: &mut App| {
                    let current = items
                        .iter()
                        .position(|item| item.focus_handle.is_focused(window));
                    window.focus(&trigger_focus, cx);
                    if forward {
                        window.focus_next(cx);
                    } else {
                        window.focus_prev(cx);
                    }
                    if let Some(ix) = current {
                        commit(ix, window, cx);
                    }
                    if let Some(close) = &request {
                        close(&false, window, cx);
                    }
                };
                listbox_element = if forward {
                    listbox_element.on_action(move |_: &FocusNext, window, cx| tab(window, cx))
                } else {
                    listbox_element.on_action(move |_: &FocusPrevious, window, cx| tab(window, cx))
                };
            }
            popup::container(
                listbox_element.children(elements),
                &state,
                margin,
                request.clone(),
                window,
                cx,
            )
        });

        let open_focus = state.read(cx).open_focus;
        let anchor = Anchor {
            state: state.clone(),
            expanded,
            trigger_focus: focus_handle.clone(),
            window_margin: listbox.window_margin,
            paint_margin: margin,
            popup,
            open_target: target(open_focus).map(&focus_target),
        };
        let value = selected_ix.map(|ix| labels[ix].clone());
        self.inner
            .role(accesskit::Role::ComboBox)
            .aria_expanded(expanded)
            .when_some(value, |this, value| this.aria_value(value))
            .when(disabled, |this| {
                let focus_handle = focus_handle.clone();
                // 押下を親に伝えず、親の on_click を呼ばせない。
                // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if focusable {
                        window.focus(&focus_handle, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .when(focusable, |this| {
                this.track_focus(&focus_handle.clone().tab_stop(true))
            })
            .when_some(request.filter(|_| !disabled), |this, request| {
                let click_request = request.clone();
                let click_state = state.clone();
                this.on_click(move |_, window, cx| {
                    click_state.update(cx, |state, _| state.open_focus = OpenFocus::Default);
                    click_request(&!expanded, window, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let modifiers = &event.keystroke.modifiers;
                    let open_focus = match event.keystroke.key.as_str() {
                        "down" if !modifiers.modified() || *modifiers == Modifiers::alt() => {
                            OpenFocus::Default
                        }
                        "up" | "home" if !modifiers.modified() => OpenFocus::First,
                        "end" if !modifiers.modified() => OpenFocus::Last,
                        _ => return,
                    };
                    if expanded {
                        if let Some((focus_handle, bounds)) = target(open_focus).map(&focus_target)
                        {
                            window.focus(&focus_handle, cx);
                            popup::scroll_into_view(
                                &state.read(cx).scroll_handle,
                                margin,
                                bounds.get(),
                            );
                        }
                    } else {
                        state.update(cx, |state, _| state.open_focus = open_focus);
                        request(&true, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .child(anchor)
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
                node.set_has_popup(accesskit::HasPopup::Listbox);
            })
    }
}

/// `from` の項目から `forward` の向きに、フォーカスできる項目を `count` 個まで進んだ位置を返す。
/// 端では止まり、1 つも進めなければ `None`
fn step(items: &[RovingItem], from: usize, forward: bool, count: usize) -> Option<usize> {
    let focusable = |ix: &usize| items[*ix].availability.is_focusable();
    if forward {
        (from + 1..items.len()).filter(focusable).take(count).last()
    } else {
        (0..from).rev().filter(focusable).take(count).last()
    }
}

/// `inner` を、一覧の中の項目の要素にする
///
/// `label` を名前にする。描くたびに項目の範囲を `bounds` に記録する。`on_select` は項目が操作されたときに呼ぶ。無効な項目では呼ばない
fn option_element<S>(
    inner: Stateful<Div>,
    item: &RovingItem,
    label: SharedString,
    selected: bool,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    on_select: S,
    window: &Window,
) -> impl IntoElement + use<S>
where
    S: Fn(&mut Window, &mut App) + 'static,
{
    let focus_handle = item.focus_handle.clone();
    // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    let focusable = item.availability.is_focusable() || focus_handle.is_focused(window);
    let disabled = item.availability.is_disabled();
    inner
        .role(accesskit::Role::ListBoxOption)
        .aria_label(label)
        .aria_selected(selected)
        .when(disabled, |this| {
            let focus_handle = focus_handle.clone();
            // 押下を親に伝えず、一覧の外の押下として扱わせない。
            // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if focusable {
                    window.focus(&focus_handle, cx);
                }
                cx.stop_propagation();
            })
        })
        .when(focusable, |this| {
            let hover_focus = focus_handle.clone();
            this.track_focus(&focus_handle.tab_stop(false))
                .on_hover(move |hovered, window, cx| {
                    if *hovered {
                        window.focus(&hover_focus, cx);
                    }
                })
        })
        .when(!disabled, |this| {
            this.on_click(move |_, window, cx| on_select(window, cx))
        })
        .child(
            canvas(
                move |item_bounds, _, _| bounds.set(item_bounds),
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
        .a11y_info(move |node: &mut accesskit::Node| {
            if disabled {
                node.set_disabled();
            }
        })
}

impl<T: PartialEq + 'static> Styled for Select<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T: PartialEq + 'static> ParentElement for Select<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<T: PartialEq + 'static> InteractiveElement for Select<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T: PartialEq + 'static> StatefulInteractiveElement for Select<T> {}

/// [`Select`] を開いたときに表示する一覧
///
/// [`Select::listbox`] に渡して使う。項目は [`SelectListbox::option`] で、項目でない要素 (区切り線など) は `child` で加え、
/// 加えた順に縦に並ぶ。
/// 支援技術には role `listbox` として、ウィンドウの直下に公開される。名前は `aria_label` で、セレクトと同じ名前を与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct SelectListbox<T> {
    inner: Stateful<Div>,
    children: Vec<GroupChild<SelectOption<T>>>,
    window_margin: Size<Pixels>,
    paint_margin: Edges<Pixels>,
}

impl<T> SelectListbox<T> {
    /// 空の一覧を作る
    pub fn new() -> Self {
        Self {
            inner: div().id("listbox"),
            children: Vec::new(),
            window_margin: Size::default(),
            paint_margin: Edges::default(),
        }
    }

    /// 末尾に項目を加える
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.children.push(GroupChild::Item(option));
        self
    }

    /// 末尾に `options` の項目を順に加える
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption<T>>) -> Self {
        self.children
            .extend(options.into_iter().map(GroupChild::Item));
        self
    }

    /// 一覧を置くときに空ける、ウィンドウの端からの余白を設定する。既定は 0
    ///
    /// [`crate::menu::Menu::window_margin`] と同じ
    pub fn window_margin(mut self, margin: Size<Pixels>) -> Self {
        self.window_margin = margin;
        self
    }

    /// 一覧の周りに、`margin` の幅の透明な余白を取る。既定は 0
    ///
    /// [`crate::menu::Menu::paint_margin`] と同じ
    pub fn paint_margin(mut self, margin: Edges<Pixels>) -> Self {
        self.paint_margin = margin;
        self
    }
}

impl<T> Default for SelectListbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Styled for SelectListbox<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T> ParentElement for SelectListbox<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl<T> InteractiveElement for SelectListbox<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T> StatefulInteractiveElement for SelectListbox<T> {}

/// [`SelectListbox`] の項目
///
/// [`SelectListbox::option`] に渡して使う。項目が表す値が、セレクトの選ばれている値と等しいときに選ばれた状態になる。
/// 支援技術には role `option` と選ばれているかどうかの状態として公開する。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける。
/// マウスが乗ったときにフォーカスを移すため、`on_hover` は設定しないこと
pub struct SelectOption<T> {
    id: ElementId,
    inner: Stateful<Div>,
    value: T,
    label: SharedString,
    availability: Availability,
    provided_focus_handle: Option<FocusHandle>,
}

impl<T> SelectOption<T> {
    /// `value` を表し、`label` を名前とする項目を作る
    ///
    /// `label` は支援技術に項目の名前と、選ばれたときのセレクトの値として公開する。子の内容は名前にならない。
    /// `id` と `value` は一覧の中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`SelectOption::track_focus`] を指定しない場合、一覧を閉じるとフォーカスの状態は破棄される
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value,
            label: label.into(),
            availability: Availability::Enabled,
            provided_focus_handle: None,
        }
    }

    /// 項目を無効にするか、無効でも矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な項目は操作されても選ぶよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は矢印キーでも到達できない。到達できるようにしたい場合は [`SelectOption::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab では到達できないよう設定される。
    /// 指定しない場合は項目が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        })
    }
}

impl<T> Styled for SelectOption<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T> ParentElement for SelectOption<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<T> InteractiveElement for SelectOption<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T> StatefulInteractiveElement for SelectOption<T> {}

/// APG の Select-Only Combobox の例 (content/patterns/combobox/examples/combobox-select-only.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, Element, Entity, Hsla, KeyDownEvent, KeyUpEvent, Keystroke, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::{button::Button, test_window::add_test_window};

    const OPTIONS: [&str; 4] = ["apple", "banana", "cherry", "durian"];
    const LISTBOX_COLOR: fn() -> Hsla = gpui::red;

    /// 左上が (100, 100) で 150x20 のセレクトと、幅 100 で高さ 20 の項目を 4 つ並べた一覧。
    /// Tab の順では、セレクトの前と後にフォーカスできる要素を置く
    struct SelectView {
        selected: Option<&'static str>,
        expanded: bool,
        /// `on_expanded_change` に渡された値
        requests: Rc<RefCell<Vec<bool>>>,
        /// `on_change` に渡された値
        changes: Rc<RefCell<Vec<&'static str>>>,
        availability: [Availability; 4],
        select_availability: Availability,
        trigger_focus: FocusHandle,
        option_focus: [FocusHandle; 4],
        before_focus: FocusHandle,
        after_focus: FocusHandle,
        node: Option<accesskit::Node>,
    }

    impl Render for SelectView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let requests = self.requests.clone();
            let changes = self.changes.clone();
            let select = Select::new("select", self.selected, self.expanded)
                .on_change(cx.listener(move |this, value: &&'static str, _, cx| {
                    changes.borrow_mut().push(*value);
                    this.selected = Some(*value);
                    cx.notify();
                }))
                .on_expanded_change(cx.listener(move |this, expanded, _, cx| {
                    requests.borrow_mut().push(*expanded);
                    this.expanded = *expanded;
                    cx.notify();
                }))
                .listbox(
                    SelectListbox::new()
                        .aria_label("Fruit")
                        .bg(LISTBOX_COLOR())
                        .options(OPTIONS.iter().enumerate().map(|(ix, name)| {
                            SelectOption::new(*name, *name, *name)
                                .availability(self.availability[ix])
                                .track_focus(&self.option_focus[ix])
                                .w(px(100.))
                                .h(px(20.))
                        })),
                )
                .track_focus(&self.trigger_focus)
                .availability(self.select_availability)
                .aria_label("Fruit")
                .absolute()
                .left(px(100.))
                .top(px(100.))
                .w(px(150.))
                .h(px(20.))
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(select.a11y_role().expect("role を持つこと"));
            select.write_a11y_info(&mut node);
            self.node = Some(node);
            div()
                .size_full()
                .child(
                    Button::new("before")
                        .track_focus(&self.before_focus)
                        .size(px(10.)),
                )
                .child(select)
                .child(
                    Button::new("after")
                        .track_focus(&self.after_focus)
                        .absolute()
                        .left(px(400.))
                        .top(px(0.))
                        .size(px(10.)),
                )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<SelectView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| SelectView {
            selected: Some("banana"),
            expanded: false,
            requests: Rc::default(),
            changes: Rc::default(),
            availability: [Availability::Enabled; 4],
            select_availability: Availability::Enabled,
            trigger_focus: cx.focus_handle(),
            option_focus: std::array::from_fn(|_| cx.focus_handle()),
            before_focus: cx.focus_handle(),
            after_focus: cx.focus_handle(),
            node: None,
        })
    }

    fn update(
        view: &Entity<SelectView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut SelectView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn expanded(view: &Entity<SelectView>, cx: &mut VisualTestContext) -> bool {
        view.read_with(cx, |view, _| view.expanded)
    }

    fn changes(view: &Entity<SelectView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    /// フォーカスのある要素の名前
    fn focused(view: &Entity<SelectView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let (trigger, options, before, after) = view.read_with(cx, |view, _| {
            (
                view.trigger_focus.clone(),
                view.option_focus.clone(),
                view.before_focus.clone(),
                view.after_focus.clone(),
            )
        });
        cx.update(|window, _| {
            [("select", &trigger), ("before", &before), ("after", &after)]
                .into_iter()
                .chain(OPTIONS.iter().copied().zip(options.iter()))
                .find(|(_, handle)| handle.is_focused(window))
                .map(|(name, _)| name)
        })
    }

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
        cx.simulate_click(point(px(x), px(y)), Modifiers::none());
    }

    /// セレクトを Tab でフォーカスしてから、`key` で開く
    fn open_with(cx: &mut VisualTestContext, key: &'static str) {
        cx.simulate_keystrokes("tab tab");
        press_key(cx, key);
    }

    #[gpui::test]
    fn クリックで開き選ばれている項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        assert!(expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("banana"));
    }

    #[gpui::test]
    fn 選ばれている項目が無ければ最初の項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.selected = None);
        click(cx, 110., 110.);
        assert_eq!(focused(&view, cx), Some("apple"));
    }

    #[gpui::test]
    fn enter_と_space_と下矢印キーと_alt_下矢印キーで開き選ばれている項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        for key in ["enter", "space", "down", "alt-down"] {
            let (view, cx) = setup(cx);
            open_with(cx, key);
            assert!(expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some("banana"), "{key}");
        }
    }

    #[gpui::test]
    fn 上矢印キーと_home_で開き最初の項目に_end_で最後の項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        for (key, expected) in [("up", "apple"), ("home", "apple"), ("end", "durian")] {
            let (view, cx) = setup(cx);
            open_with(cx, key);
            assert!(expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some(expected), "{key}");
        }
    }

    #[gpui::test]
    fn 上下の矢印キーで項目の間を移り端では止まる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        let mut path = Vec::new();
        for key in ["down", "down", "down", "up", "up", "up", "up"] {
            press_key(cx, key);
            path.push(focused(&view, cx));
        }
        assert_eq!(
            path,
            [
                Some("cherry"),
                Some("durian"),
                Some("durian"),
                Some("cherry"),
                Some("banana"),
                Some("apple"),
                Some("apple"),
            ]
        );
    }

    #[gpui::test]
    fn 修飾キーのある矢印キーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        for key in ["ctrl-down", "shift-up", "ctrl-end"] {
            press_key(cx, key);
            assert_eq!(focused(&view, cx), Some("banana"), "{key}");
        }
    }

    #[gpui::test]
    fn home_end_pageup_pagedown_で移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        let mut path = Vec::new();
        for key in ["end", "home", "pagedown", "pageup"] {
            press_key(cx, key);
            path.push(focused(&view, cx));
        }
        assert_eq!(
            path,
            [Some("durian"), Some("apple"), Some("durian"), Some("apple")],
            "10 個先が無ければ端へ移ること"
        );
    }

    #[gpui::test]
    fn 無効な項目は飛ばし無効でも到達できる項目には移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.availability[2] = Availability::Disabled;
            view.availability[3] = Availability::DisabledFocusable;
        });
        open_with(cx, "down");
        press_key(cx, "down");
        assert_eq!(focused(&view, cx), Some("durian"));
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty(), "無効な項目は選ばないこと");
        press_key(cx, "up");
        assert_eq!(focused(&view, cx), Some("banana"));
    }

    #[gpui::test]
    fn enter_と_space_で項目を選んで閉じセレクトにフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        for key in ["enter", "space"] {
            let (view, cx) = setup(cx);
            open_with(cx, "down");
            press_key(cx, "down");
            press_key(cx, key);
            assert_eq!(changes(&view, cx), ["cherry"], "{key}");
            assert!(!expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some("select"), "{key}");
        }
    }

    #[gpui::test]
    fn 項目のクリックで選んで閉じる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        // 一覧はセレクトの下に置かれ、4 つ目の項目は y が 180 から 200 の範囲になる
        click(cx, 110., 190.);
        assert_eq!(changes(&view, cx), ["durian"]);
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("select"));
    }

    #[gpui::test]
    fn 選ばれている項目を選んでも_on_change_を呼ばず閉じる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty());
        assert!(!expanded(&view, cx));
    }

    #[gpui::test]
    fn alt_上矢印キーで項目を選んで閉じる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        press_key(cx, "up");
        press_key(cx, "alt-up");
        assert_eq!(changes(&view, cx), ["apple"]);
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("select"));
    }

    #[gpui::test]
    fn escape_で選ばずに閉じセレクトにフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        press_key(cx, "down");
        press_key(cx, "escape");
        assert!(changes(&view, cx).is_empty());
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("select"));
    }

    #[gpui::test]
    fn tab_と_shift_tab_で項目を選んで閉じ前後の要素へ移る(
        cx: &mut TestAppContext,
    ) {
        for (key, expected) in [("tab", "after"), ("shift-tab", "before")] {
            let (view, cx) = setup(cx);
            open_with(cx, "down");
            press_key(cx, "down");
            cx.simulate_keystrokes(key);
            assert_eq!(changes(&view, cx), ["cherry"], "{key}");
            assert!(!expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some(expected), "{key}");
        }
    }

    #[gpui::test]
    fn 外での押下で閉じる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        click(cx, 300., 300.);
        assert!(!expanded(&view, cx));
        assert!(changes(&view, cx).is_empty());
        let requests = view.read_with(cx, |view, _| view.requests.borrow().clone());
        assert_eq!(requests, [true, false]);
    }

    #[gpui::test]
    fn セレクトのクリックで閉じる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        click(cx, 110., 110.);
        assert!(!expanded(&view, cx));
        let requests = view.read_with(cx, |view, _| view.requests.borrow().clone());
        assert_eq!(requests, [true, false], "閉じるよう 1 回だけ求めること");
    }

    #[gpui::test]
    fn 一覧の幅はセレクトの幅以上にする(cx: &mut TestAppContext) {
        let (_view, cx) = setup(cx);
        click(cx, 110., 110.);
        let bounds = cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.background == LISTBOX_COLOR().into())
                .map(|quad| {
                    (
                        quad.bounds.origin.x.0 / scale,
                        quad.bounds.origin.y.0 / scale,
                        quad.bounds.size.width.0 / scale,
                    )
                })
        });
        assert_eq!(bounds, Some((100., 120., 150.)));
    }

    #[gpui::test]
    fn 無効なセレクトは開かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.select_availability = Availability::DisabledFocusable
        });
        click(cx, 110., 110.);
        assert!(!expanded(&view, cx));
        press_key(cx, "down");
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("select"));
    }

    #[gpui::test]
    fn combobox_として開閉と選ばれている項目の名前を公開する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let node = view.read_with(cx, |view, _| view.node.clone().unwrap());
        assert_eq!(node.role(), accesskit::Role::ComboBox);
        assert_eq!(node.label(), Some("Fruit"));
        assert_eq!(node.value(), Some("banana"));
        assert_eq!(node.is_expanded(), Some(false));
        assert_eq!(node.has_popup(), Some(accesskit::HasPopup::Listbox));
        assert!(node.supports_action(accesskit::Action::Click));

        click(cx, 110., 110.);
        let node = view.read_with(cx, |view, _| view.node.clone().unwrap());
        assert_eq!(node.is_expanded(), Some(true));

        update(&view, cx, |view| view.selected = None);
        let node = view.read_with(cx, |view, _| view.node.clone().unwrap());
        assert_eq!(node.value(), None);
    }

    fn option_node(
        cx: &mut VisualTestContext,
        availability: Availability,
        selected: bool,
    ) -> accesskit::Node {
        cx.update(|window, cx| {
            let item = RovingItem {
                focus_handle: cx.focus_handle(),
                availability,
            };
            let element = option_element(
                div().id("option"),
                &item,
                "Apple".into(),
                selected,
                Rc::default(),
                |_, _| {},
                window,
            )
            .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        })
    }

    #[gpui::test]
    fn 項目は_option_として名前と選ばれているかを公開する(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        let node = option_node(cx, Availability::Enabled, true);
        assert_eq!(node.role(), accesskit::Role::ListBoxOption);
        assert_eq!(node.label(), Some("Apple"));
        assert_eq!(node.is_selected(), Some(true));
        assert!(!node.is_disabled());
        assert!(node.supports_action(accesskit::Action::Click));

        let node = option_node(cx, Availability::DisabledFocusable, false);
        assert_eq!(node.is_selected(), Some(false));
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
    }
}
