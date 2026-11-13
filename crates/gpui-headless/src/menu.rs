//! ボタンから開く、見た目を持たないメニュー

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ClickEvent, Div, Edges, Element, ElementId, Entity,
    FocusHandle, GlobalElementId, InspectorElementId, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, LayoutId, MouseButton, ParentElement, Pixels,
    Point, ScrollHandle, Size, Stateful, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, accesskit, canvas, div, point, prelude::FluentBuilder, px, size,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    button::Button,
    choice::CheckedState,
    navigation::{FocusNext, FocusPrevious},
    roving::{self, GroupChild, RovingItem},
    tooltip,
};

/// メニューをツールチップより奥に描くための、遅延描画の優先度
const PRIORITY: usize = tooltip::PRIORITY - 1;

type ExpandedChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// ボタンから開く、見た目を持たないメニュー
///
/// [`Menu::attach`] でボタンに付けて使う。開いているかどうかは呼び出し側が持ち、[`Menu::new`] に毎回渡す。
/// メニュー自身は状態を変えず、開閉は [`Menu::on_expanded_change`] で求める。
/// 項目は [`Menu::item`] で、項目の組は [`Menu::group`] で、項目でない要素 (区切り線など) は `child` で加え、加えた順に縦に並ぶ
///
/// ボタン
///
/// - クリック・Enter・Space で、閉じていれば開くよう、開いていれば閉じるよう求める
/// - 修飾キーなしの下矢印キーで開くよう求め、上矢印キーで開くよう求めて最後の項目にフォーカスを移す。
///   開いている間は、開くよう求めずにフォーカスだけを移す
/// - 支援技術には、メニューを開くボタンであることと、開いているかどうかを公開する
///
/// 開いている間のメニュー
///
/// - 開いたとき、最初のフォーカスできる項目 (上矢印キーで開いたときは最後の項目) にフォーカスを移す。
///   フォーカスできる項目が無ければメニューにフォーカスを移す
/// - 修飾キーなしの上下の矢印キーで前後の項目へフォーカスを移し、端では反対の端に戻る。
///   組の中の項目も、組をまたいで加えた順に移る。
///   Home と End では最初と最後の項目へ移る。[`Availability::Disabled`] の項目は飛ばす。
///   キーでフォーカスを移した項目が見えるようにスクロールする
/// - 項目にマウスが乗ると、その項目にフォーカスを移す
/// - 修飾キーなしの Escape で閉じるよう求める
/// - Tab で閉じるよう求め、フォーカスをボタンの次に Tab で到達できる要素へ移す。
///   Shift+Tab で閉じるよう求め、フォーカスをボタンへ移す
/// - メニューとボタンの外でマウスのボタンを押すと、閉じるよう求める。押した先には、そのまま押下が届く
/// - 閉じたとき、フォーカスがメニューの中にあればボタンへ戻す
/// - フォーカスのある項目が描かれなくなると、メニューにフォーカスを移す
/// - 前の 2 つは、そのフレームを描き終えるまでに別の要素へフォーカスが移されていれば行わない
/// - メニューの上では、クリックやスクロールは背後の要素に届かない
///
/// メニューはボタンの下に、左端をボタンの左端にそろえて置く。
/// ウィンドウの端から [`Menu::window_margin`] の幅の内側に収まらなければ、右端をボタンの右端にそろえ、
/// それでも収まらなければ、ボタンの中心に近い側のウィンドウの端の余白に寄せる。
/// 上下は、下に収まらなければボタンの上に、それでも収まらなければ中心をボタンの上端にそろえ、
/// それでも収まらなければ、ボタンの中心に近い側のウィンドウの端の余白に寄せる。
/// 余白を除いた幅や高さに収まらない大きさなら、ウィンドウの中央に置く。
/// メニューは、[`Menu::paint_margin`] の余白と合わせてウィンドウの大きさまでにし、超える分は縦にスクロールする
///
/// 支援技術には role `menu` として、ウィンドウの直下に公開される。名前は `aria_label` で、ボタンと同じ名前を与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct Menu {
    id: ElementId,
    inner: Stateful<Div>,
    expanded: bool,
    on_expanded_change: Option<ExpandedChangeHandler>,
    children: Vec<MenuChild>,
    window_margin: Size<Pixels>,
    paint_margin: Edges<Pixels>,
}

/// メニューの子
enum MenuChild {
    Item(MenuItem),
    Group(MenuGroup),
    Other(AnyElement),
}

impl Menu {
    /// `expanded` なら開いた状態で表示するメニューを作る
    ///
    /// `id` は描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, expanded: bool) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            expanded,
            on_expanded_change: None,
            children: Vec::new(),
            window_margin: Size::default(),
            paint_margin: Edges::default(),
        }
    }

    /// 開くか閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、開くよう求められたら `true` を、閉じるよう求められたら `false` を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Menu::new`] に渡すこと。
    /// 設定しない場合、ボタンは押されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Rc::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: MenuItem) -> Self {
        self.children.push(MenuChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.children.extend(items.into_iter().map(MenuChild::Item));
        self
    }

    /// 末尾に項目の組を加える
    pub fn group(mut self, group: MenuGroup) -> Self {
        self.children.push(MenuChild::Group(group));
        self
    }

    /// メニューを置くときに空ける、ウィンドウの端からの余白を設定する。既定は 0
    ///
    /// `margin.width` は左右の端からの、`margin.height` は上下の端からの余白にする
    pub fn window_margin(mut self, margin: Size<Pixels>) -> Self {
        self.window_margin = margin;
        self
    }

    /// メニューの周りに、`margin` の幅の透明な余白を取る。既定は 0
    ///
    /// メニューの子は余白の上にも描け、影などメニューの外に描くものに使う。
    /// 縦にスクロールするときは、項目も余白の上に描かれる。
    /// 余白はメニューを置く位置の計算に含めず、ボタンやウィンドウの端と重なってよい。
    /// メニューの子が描かれていない余白の上の操作は背後の要素に届き、押下はメニューの外の押下として扱う。
    /// 縦にスクロールするメニューは、余白の上のスクロールの操作でもスクロールする
    pub fn paint_margin(mut self, margin: Edges<Pixels>) -> Self {
        self.paint_margin = margin;
        self
    }

    /// `trigger` をこのメニューを開くボタンにして返す
    ///
    /// `trigger` の子の最後に、レイアウトに影響しない要素を加える。
    /// `trigger` に [`Button::on_click`] のハンドラがあれば、開閉を求める前に呼ぶ。
    /// 無効な `trigger` は押されても開閉を求めないが、開いているメニューは閉じられる
    pub fn attach(self, mut trigger: Button) -> Button {
        trigger.menu = Some(self);
        trigger
    }

    /// `trigger` のボタンに、メニューを開閉する操作と、開いたメニューを加える
    ///
    /// `trigger_focus` は `trigger` のフォーカスにすること
    pub(crate) fn attach_to_trigger(
        self,
        trigger: Stateful<Div>,
        trigger_focus: &FocusHandle,
        trigger_disabled: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            expanded: false,
            focus_last_on_open: false,
            menu_focus: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            trigger_bounds: Bounds::default(),
        });
        let expanded = self.expanded;
        let margin = self.paint_margin;
        let request = self.on_expanded_change;

        let mut items = Vec::new();
        // 項目ごとの、最後に描いたときの範囲
        let mut item_bounds = Vec::new();
        let mut elements = Vec::new();
        if expanded {
            let mut add_item = |item: MenuItem, window: &mut Window, cx: &mut App| {
                // 項目の id はメニューの中でだけ一意なので、メニューの id で分ける
                let focus_handle = window.with_element_namespace(self.id.clone(), |window| {
                    item.focus_handle(window, cx)
                });
                let bounds = Rc::new(Cell::new(Bounds::default()));
                items.push(RovingItem {
                    focus_handle: focus_handle.clone(),
                    availability: item.availability,
                });
                item_bounds.push(bounds.clone());
                item_element(item, &focus_handle, bounds, request.clone(), window)
                    .into_any_element()
            };
            for child in self.children {
                elements.push(match child {
                    MenuChild::Item(item) => add_item(item, window, cx),
                    MenuChild::Group(group) => {
                        let children: Vec<AnyElement> = group
                            .children
                            .into_iter()
                            .map(|child| match child {
                                GroupChild::Item(item) => add_item(item, window, cx),
                                GroupChild::Other(element) => element,
                            })
                            .collect();
                        group
                            .inner
                            .role(accesskit::Role::Group)
                            .children(children)
                            .into_any_element()
                    }
                    MenuChild::Other(element) => element,
                });
            }
        }
        let items: Rc<[RovingItem]> = items.into();
        let item_bounds: Rc<[Rc<Cell<Bounds<Pixels>>>]> = item_bounds.into();
        let first = items
            .iter()
            .position(|item| item.availability.is_focusable());
        let last = items
            .iter()
            .rposition(|item| item.availability.is_focusable());

        let menu = expanded.then(|| {
            let (menu_focus, scroll_handle) = {
                let state = state.read(cx);
                (state.menu_focus.clone(), state.scroll_handle.clone())
            };
            let viewport = window.viewport_size();
            let items = items.clone();
            let item_bounds = item_bounds.clone();
            let close = request.clone();
            let next_close = request.clone();
            let previous_close = request.clone();
            let next_focus = trigger_focus.clone();
            let previous_focus = trigger_focus.clone();
            let outside_state = state.clone();
            let outside_request = request.clone();
            let close_outside = Rc::new(
                move |position: Point<Pixels>, window: &mut Window, cx: &mut App| {
                    // ボタンの上の押下は、ボタンのクリックで閉じるよう求める
                    if outside_state.read(cx).trigger_bounds.contains(&position) {
                        return;
                    }
                    if let Some(close) = &outside_request {
                        close(&false, window, cx);
                    }
                },
            );
            let menu_close_outside = close_outside.clone();
            let menu_scroll_handle = scroll_handle.clone();
            let key_scroll_handle = scroll_handle.clone();
            let menu = self
                .inner
                .role(accesskit::Role::Menu)
                .track_focus(&menu_focus)
                .occlude()
                .on_mouse_down_out(move |event, window, cx| {
                    // スクロールで切り取られた部分もメニューの範囲に入るため、スクロールの範囲の外の押下はスクロールの要素で扱う
                    if menu_scroll_handle.bounds().contains(&event.position) {
                        menu_close_outside(event.position, window, cx);
                    }
                })
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    let current = items
                        .iter()
                        .position(|item| item.focus_handle.is_focused(window));
                    let target = match event.keystroke.key.as_str() {
                        "down" if current.is_some() => roving::neighbor(&items, true, window),
                        "up" if current.is_some() => roving::neighbor(&items, false, window),
                        "down" | "home" => first,
                        "up" | "end" => last,
                        "escape" => {
                            if let Some(close) = &close {
                                close(&false, window, cx);
                            }
                            cx.stop_propagation();
                            return;
                        }
                        _ => return,
                    };
                    if let Some(ix) = target {
                        window.focus(&items[ix].focus_handle, cx);
                        scroll_into_view(&key_scroll_handle, margin, item_bounds[ix].get());
                    }
                    cx.stop_propagation();
                })
                .on_action(move |_: &FocusNext, window, cx| {
                    window.focus(&next_focus, cx);
                    window.focus_next(cx);
                    if let Some(close) = &next_close {
                        close(&false, window, cx);
                    }
                })
                .on_action(move |_: &FocusPrevious, window, cx| {
                    window.focus(&previous_focus, cx);
                    if let Some(close) = &previous_close {
                        close(&false, window, cx);
                    }
                })
                .children(elements);
            // 余白の上にもメニューの子を描けるよう、余白を含めた範囲でスクロールし、切り取る
            div()
                .id("scroll")
                .track_scroll(&scroll_handle)
                .overflow_y_scroll()
                .max_w(viewport.width)
                .max_h(viewport.height)
                .pt(margin.top)
                .pr(margin.right)
                .pb(margin.bottom)
                .pl(margin.left)
                .on_mouse_down_out(move |event, window, cx| {
                    close_outside(event.position, window, cx)
                })
                .child(menu)
                .into_any_element()
        });

        let edge = move |last_edge: bool| {
            let ix = if last_edge { last } else { first }?;
            Some((items[ix].focus_handle.clone(), item_bounds[ix].clone()))
        };
        let anchor = Anchor {
            state: state.clone(),
            expanded,
            trigger_focus: trigger_focus.clone(),
            window_margin: self.window_margin,
            paint_margin: self.paint_margin,
            menu,
            first: edge(false),
            last: edge(true),
        };
        trigger
            .aria_expanded(expanded)
            .when_some(request.filter(|_| !trigger_disabled), |this, request| {
                let click_request = request.clone();
                let click_state = state.clone();
                this.on_click(move |_, window, cx| {
                    click_state.update(cx, |state, _| state.focus_last_on_open = false);
                    click_request(&!expanded, window, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let last_edge = match event.keystroke.key.as_str() {
                        "down" => false,
                        "up" => true,
                        _ => return,
                    };
                    if event.keystroke.modifiers.modified() {
                        return;
                    }
                    if expanded {
                        if let Some((focus_handle, bounds)) = edge(last_edge) {
                            window.focus(&focus_handle, cx);
                            scroll_into_view(&state.read(cx).scroll_handle, margin, bounds.get());
                        }
                    } else {
                        state.update(cx, |state, _| state.focus_last_on_open = last_edge);
                        request(&true, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .child(anchor)
    }
}

/// `item` を、メニューの中の項目の要素にする
///
/// `focus_handle` は項目のフォーカスにする。描くたびに項目の範囲を `bounds` に記録する。
/// `on_expanded_change` は項目の操作で閉じるよう求めるときに呼ぶ
fn item_element(
    item: MenuItem,
    focus_handle: &FocusHandle,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    window: &Window,
) -> impl IntoElement + use<> {
    let focus_handle = focus_handle.clone();
    // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    let focusable = item.availability.is_focusable() || focus_handle.is_focused(window);
    let disabled = item.availability.is_disabled();
    let kind = item.kind;
    let on_click = item.on_click.filter(|_| !disabled);
    item.inner
        .role(match kind {
            MenuItemKind::Action => accesskit::Role::MenuItem,
            MenuItemKind::Checkbox(_) => accesskit::Role::MenuItemCheckBox,
            MenuItemKind::Radio(_) => accesskit::Role::MenuItemRadio,
        })
        .map(|this| match kind {
            MenuItemKind::Action => this,
            MenuItemKind::Checkbox(state) => this.aria_toggled(match state {
                CheckedState::Unchecked => accesskit::Toggled::False,
                CheckedState::Checked => accesskit::Toggled::True,
                CheckedState::Mixed => accesskit::Toggled::Mixed,
            }),
            MenuItemKind::Radio(checked) => this.aria_toggled(checked.into()),
        })
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
            let hover_focus = focus_handle.clone();
            this.track_focus(&focus_handle.tab_stop(false))
                .on_hover(move |hovered, window, cx| {
                    if *hovered {
                        window.focus(&hover_focus, cx);
                    }
                })
        })
        .when(!disabled, |this| {
            this.on_click(move |event, window, cx| {
                if let Some(on_click) = &on_click {
                    on_click(event, window, cx);
                }
                let enter = matches!(
                    event,
                    ClickEvent::Keyboard(KeyboardClickEvent {
                        button: KeyboardButton::Enter,
                        ..
                    })
                );
                // APG では、切り替える項目は Enter でだけ閉じ、Space では開いたまま切り替える
                if (kind == MenuItemKind::Action || enter)
                    && let Some(on_expanded_change) = &on_expanded_change
                {
                    on_expanded_change(&false, window, cx);
                }
            })
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

/// 最後に描いたときの範囲が `item` の項目が、スクロールの範囲から `margin` の余白を除いた範囲に入るよう、
/// `scroll_handle` のメニューを最小限だけスクロールする
fn scroll_into_view(scroll_handle: &ScrollHandle, margin: Edges<Pixels>, item: Bounds<Pixels>) {
    let viewport = scroll_handle.bounds();
    let top = viewport.top() + margin.top;
    let bottom = viewport.bottom() - margin.bottom;
    let mut offset = scroll_handle.offset();
    if item.top() < top {
        offset.y += top - item.top();
    } else if item.bottom() > bottom {
        offset.y -= item.bottom() - bottom;
    }
    scroll_handle.set_offset(offset);
}

impl Styled for Menu {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Menu {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(MenuChild::Other));
    }
}

impl InteractiveElement for Menu {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Menu {}

/// [`Menu`] の項目をまとめる組
///
/// [`Menu::group`] に渡して使う。項目は [`MenuGroup::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。
/// 組の中の項目の `id` も、メニューの中で一意にすること。
/// 支援技術には role `group` として公開される。名前は `aria_label` で与えられる。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct MenuGroup {
    inner: Stateful<Div>,
    children: Vec<GroupChild<MenuItem>>,
}

impl MenuGroup {
    /// 空の組を作る
    ///
    /// `id` はメニューの中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            inner: div().id(id),
            children: Vec::new(),
        }
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: MenuItem) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl Styled for MenuGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for MenuGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl InteractiveElement for MenuGroup {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for MenuGroup {}

/// [`Menu`] の項目
///
/// [`Menu::item`] に渡して使う。アクセシブルな名前は、子の内容か `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける。
/// マウスが乗ったときにフォーカスを移すため、`on_hover` は設定しないこと
///
/// フォーカス中の Enter と Space・マウスのクリック・支援技術からのクリック操作で、[`MenuItem::on_click`] のハンドラを呼ぶ。
/// 続けて、[`MenuItemKind::Action`] の項目では閉じるよう求め、切り替える項目では Enter のときだけ閉じるよう求める
pub struct MenuItem {
    id: ElementId,
    inner: Stateful<Div>,
    kind: MenuItemKind,
    availability: Availability,
    on_click: Option<ClickHandler>,
    provided_focus_handle: Option<FocusHandle>,
}

/// メニューの項目の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuItemKind {
    /// 操作を実行する項目
    #[default]
    Action,
    /// オンとオフを切り替える項目。値はチェックの状態
    Checkbox(CheckedState),
    /// 同じ組の中から 1 つを選ぶ項目。値は選ばれているかどうか
    Radio(bool),
}

impl MenuItem {
    /// 項目を作る
    ///
    /// `id` はメニューの中で一意にし、描画のたびに同じ値を渡すこと。
    /// [`MenuItem::track_focus`] を指定しない場合、メニューを閉じるとフォーカスの状態は破棄される
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            kind: MenuItemKind::default(),
            availability: Availability::Enabled,
            on_click: None,
            provided_focus_handle: None,
        }
    }

    /// 項目の種類を設定する。既定は [`MenuItemKind::Action`]
    ///
    /// 支援技術には、種類に応じて role `menuitem`、`menuitemcheckbox`、`menuitemradio` として公開し、
    /// 切り替える項目はその状態を公開する
    pub fn kind(mut self, kind: MenuItemKind) -> Self {
        self.kind = kind;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 切り替える項目の状態を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`MenuItem::kind`] に渡すこと
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 項目を無効にするか、無効でも矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な項目は操作されてもハンドラを呼ばず閉じるよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// APG のメニューに合わせ、無効な項目も矢印キーとマウスでフォーカスできる
    /// ([`Availability::DisabledFocusable`])。到達できなくするには [`MenuItem::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::DisabledFocusable
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

impl Styled for MenuItem {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for MenuItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for MenuItem {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for MenuItem {}

/// フレームをまたいで持つメニューの状態
struct State {
    /// 最後に描いたフレームで開いていたか
    expanded: bool,
    /// 次に開いたとき、最初でなく最後の項目にフォーカスを移すか
    focus_last_on_open: bool,
    /// メニューのフォーカス。フォーカスがメニューの中にあるかの判定に使う
    menu_focus: FocusHandle,
    scroll_handle: ScrollHandle,
    /// 最後に描いたボタンの範囲
    trigger_bounds: Bounds<Pixels>,
}

/// ボタンの子に置き、ボタンの範囲を記録し、開いたメニューを配置して、開閉に合わせてフォーカスを移す要素
struct Anchor {
    state: Entity<State>,
    expanded: bool,
    trigger_focus: FocusHandle,
    window_margin: Size<Pixels>,
    paint_margin: Edges<Pixels>,
    menu: Option<AnyElement>,
    /// 開いたときにフォーカスを移す、最初と最後のフォーカスできる項目のフォーカスと最後に描いたときの範囲
    first: Option<(FocusHandle, Rc<Cell<Bounds<Pixels>>>)>,
    last: Option<(FocusHandle, Rc<Cell<Bounds<Pixels>>>)>,
}

impl IntoElement for Anchor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Anchor {
    /// ボタンと同じ範囲を占める要素
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut area = div().absolute().inset_0().into_any_element();
        let layout_id = area.request_layout(window, cx);
        (layout_id, area)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        area: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.state
            .update(cx, |state, _| state.trigger_bounds = bounds);
        area.prepaint(window, cx);
        let Some(mut menu) = self.menu.take() else {
            return;
        };
        // ボタンの幅に縛られないよう、ウィンドウのルートと同じく独立に大きさを決める
        let menu_size = menu.layout_as_root(
            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
            window,
            cx,
        );
        let margin = self.paint_margin;
        let origin = position(
            bounds,
            size(
                menu_size.width - margin.left - margin.right,
                menu_size.height - margin.top - margin.bottom,
            ),
            window.viewport_size(),
            self.window_margin,
        );
        window.defer_draw(
            menu,
            point(origin.x - margin.left, origin.y - margin.top),
            PRIORITY,
            None,
        );
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        area: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        area.paint(window, cx);
        // uniform_list などは測るために描かない要素もレイアウトするため、描いたときだけ呼ばれる paint で開閉を判定する
        let (was_expanded, focus_last) = self.state.update(cx, |state, _| {
            (
                std::mem::replace(&mut state.expanded, self.expanded),
                std::mem::take(&mut state.focus_last_on_open),
            )
        });
        let (menu_focus, scroll_handle) = {
            let state = self.state.read(cx);
            (state.menu_focus.clone(), state.scroll_handle.clone())
        };
        if self.expanded && !was_expanded {
            let target = if focus_last { &self.last } else { &self.first };
            let (focus_handle, bounds) = match target {
                Some((focus_handle, bounds)) => (focus_handle.clone(), Some(bounds.clone())),
                None => (menu_focus, None),
            };
            let margin = self.paint_margin;
            // 描画中のフォーカスの移動は再描画されないため、描き終えてから移す。項目の範囲も描き終えると決まる
            window.defer(cx, move |window, cx| {
                window.focus(&focus_handle, cx);
                if let Some(bounds) = bounds {
                    scroll_into_view(&scroll_handle, margin, bounds.get());
                }
            });
        } else if was_expanded && menu_focus.contains_focused(window, cx) {
            let expanded = self.expanded;
            let trigger_focus = self.trigger_focus.clone();
            let focused = window.focused(cx);
            // contains_focused は前のフレームで判定するため、描き終えてから確かめる。
            // それまでに別のメニューを開くなどでフォーカスが移されたときは移さない
            window.defer(cx, move |window, cx| {
                if window.focused(cx) != focused {
                    return;
                }
                if !expanded {
                    window.focus(&trigger_focus, cx);
                } else if !menu_focus.contains_focused(window, cx) {
                    // フォーカスのある項目が描かれなくなったときは、キーを受け続けられるようメニューに移す
                    window.focus(&menu_focus, cx);
                }
            });
        }
    }
}

/// `trigger` の範囲のボタンに対し、大きさ `menu` のメニューを置く左上の位置を返す
///
/// 位置の候補と選び方は、Compose Material3 の `MenuAnchorPosition.Below` と `DropdownMenuPositionProvider`
/// (androidx の compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt と
/// internal/MenuPosition.kt) を左から右へ書く向きについて移したもの
fn position(
    trigger: Bounds<Pixels>,
    menu: Size<Pixels>,
    viewport: Size<Pixels>,
    margin: Size<Pixels>,
) -> Point<Pixels> {
    // 余白の内側に収まる最初の候補を選ぶ。どれも収まらなければ、余白を除いた空きより大きいときは中央に置き、
    // そうでなければ最後の候補を余白の内側に寄せる
    let place = |candidates: &[Pixels], size: Pixels, space: Pixels, margin: Pixels| {
        candidates
            .iter()
            .copied()
            .find(|&candidate| candidate >= margin && candidate + size <= space - margin)
            .unwrap_or_else(|| {
                if size >= space - margin * 2. {
                    (space - size) / 2.
                } else {
                    let last = candidates[candidates.len() - 1];
                    last.max(margin).min(space - margin - size)
                }
            })
    };
    let center = trigger.center();
    let x = place(
        &[
            trigger.left(),
            trigger.right() - menu.width,
            if center.x < viewport.width / 2. {
                px(0.)
            } else {
                viewport.width - menu.width
            },
        ],
        menu.width,
        viewport.width,
        margin.width,
    );
    let y = place(
        &[
            trigger.bottom(),
            trigger.top() - menu.height,
            trigger.top() - menu.height / 2.,
            if center.y < viewport.height / 2. {
                px(0.)
            } else {
                viewport.height - menu.height
            },
        ],
        menu.height,
        viewport.height,
        margin.height,
    );
    point(x, y)
}

/// APG の Menu Button パターン (content/patterns/menu-button/menu-button-pattern.html) と
/// Menu パターン (content/patterns/menubar/menubar-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, Entity, Hsla, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render, RenderOnce,
        TestAppContext, VisualTestContext,
    };

    use super::*;
    use crate::test_window::add_test_window;

    const ITEMS: [&str; 4] = ["cut", "copy", "bold", "align"];
    const MENU_COLOR: fn() -> Hsla = gpui::red;
    const FIRST_ITEM_COLOR: fn() -> Hsla = gpui::green;
    const LAST_ITEM_COLOR: fn() -> Hsla = gpui::blue;

    /// 左上が `trigger_origin` で 40x20 のボタンと、幅 100 で高さ 20 の項目を 4 つ並べたメニュー。
    /// 2 つ目の項目の後に高さ 4 の区切りを置き、後ろの 2 つの項目は組にまとめるため、メニューは 100x84 になる。
    /// Tab の順では、ボタンの前と後にフォーカスできる要素を置く。
    /// (200, 200) から 50x50 の範囲に、クリックを数える要素を置く。
    /// `sibling` なら、同じ id のメニューを持つ 10x10 の別のボタンを (300, 0) に置く。
    /// そのメニューは `sibling_expanded` なら開き、幅 50 で高さ 20 の項目を 1 つ持つ。
    /// `sibling_first` なら、別のボタンを先に描く。`hide_cut` なら最初の項目を描かない
    struct MenuView {
        expanded: bool,
        /// `on_expanded_change` に渡された値
        requests: Rc<RefCell<Vec<bool>>>,
        /// 操作された項目
        clicks: Rc<RefCell<Vec<&'static str>>>,
        behind_clicks: Rc<RefCell<usize>>,
        availability: [Availability; 4],
        trigger_disabled: bool,
        trigger_origin: Point<Pixels>,
        sibling: bool,
        sibling_expanded: bool,
        sibling_first: bool,
        hide_cut: bool,
        window_margin: Size<Pixels>,
        paint_margin: Edges<Pixels>,
        trigger_focus: FocusHandle,
        item_focus: [FocusHandle; 4],
        sibling_item_focus: FocusHandle,
        before_focus: FocusHandle,
        after_focus: FocusHandle,
        trigger_node: Option<accesskit::Node>,
    }

    impl Render for MenuView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let requests = self.requests.clone();
            let menu = Menu::new("menu", self.expanded)
                .on_expanded_change(cx.listener(move |this, expanded, _, cx| {
                    requests.borrow_mut().push(*expanded);
                    this.expanded = *expanded;
                    cx.notify();
                }))
                .window_margin(self.window_margin)
                .paint_margin(self.paint_margin)
                .aria_label("Actions")
                .bg(MENU_COLOR())
                .items(
                    ITEMS
                        .iter()
                        .enumerate()
                        .take(2)
                        .filter(|(ix, _)| !(self.hide_cut && *ix == 0))
                        .map(|(ix, name)| self.item(ix, name)),
                )
                .child(div().h(px(4.)))
                .group(
                    MenuGroup::new("format").aria_label("Format").items(
                        ITEMS
                            .iter()
                            .enumerate()
                            .skip(2)
                            .map(|(ix, name)| self.item(ix, name)),
                    ),
                );
            let trigger = menu
                .attach(
                    Button::new("trigger")
                        .track_focus(&self.trigger_focus)
                        .disabled(self.trigger_disabled)
                        .aria_label("Actions")
                        .absolute()
                        .left(self.trigger_origin.x)
                        .top(self.trigger_origin.y)
                        .w(px(40.))
                        .h(px(20.)),
                )
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(trigger.a11y_role().expect("role を持つこと"));
            trigger.write_a11y_info(&mut node);
            self.trigger_node = Some(node);
            // 部品の render は親の要素の中で呼ばれ、兄弟は同じ id の経路で描かれるため、ボタンと同じ経路で描く
            let sibling = self.sibling.then(|| {
                Menu::new("menu", self.sibling_expanded)
                    .item(
                        MenuItem::new("sibling-item")
                            .track_focus(&self.sibling_item_focus)
                            .w(px(50.))
                            .h(px(20.)),
                    )
                    .attach(
                        Button::new("sibling")
                            .absolute()
                            .left(px(300.))
                            .top(px(0.))
                            .size(px(10.)),
                    )
                    .render(window, cx)
                    .into_element()
            });
            let (sibling_before, sibling_after) = if self.sibling_first {
                (sibling, None)
            } else {
                (None, sibling)
            };
            let behind_clicks = self.behind_clicks.clone();
            div()
                .size_full()
                .child(
                    Button::new("before")
                        .track_focus(&self.before_focus)
                        .size(px(10.)),
                )
                .children(sibling_before)
                .child(trigger)
                .child(
                    Button::new("after")
                        .track_focus(&self.after_focus)
                        .absolute()
                        .left(px(400.))
                        .top(px(0.))
                        .size(px(10.)),
                )
                .children(sibling_after)
                .child(
                    div()
                        .id("behind")
                        .absolute()
                        .left(px(200.))
                        .top(px(200.))
                        .size(px(50.))
                        .on_click(move |_, _, _| *behind_clicks.borrow_mut() += 1),
                )
        }
    }

    impl MenuView {
        fn item(&self, ix: usize, name: &'static str) -> MenuItem {
            let clicks = self.clicks.clone();
            let item = MenuItem::new(name)
                .kind(match ix {
                    2 => MenuItemKind::Checkbox(CheckedState::Unchecked),
                    3 => MenuItemKind::Radio(false),
                    _ => MenuItemKind::Action,
                })
                .availability(self.availability[ix])
                .track_focus(&self.item_focus[ix])
                .on_click(move |_, _, _| clicks.borrow_mut().push(name))
                .w(px(100.))
                .h(px(20.));
            match ix {
                0 => item.bg(FIRST_ITEM_COLOR()),
                3 => item.bg(LAST_ITEM_COLOR()),
                _ => item,
            }
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<MenuView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| MenuView {
            expanded: false,
            requests: Rc::default(),
            clicks: Rc::default(),
            behind_clicks: Rc::default(),
            availability: [Availability::Enabled; 4],
            trigger_disabled: false,
            trigger_origin: point(px(100.), px(100.)),
            sibling: false,
            sibling_expanded: false,
            sibling_first: false,
            hide_cut: false,
            window_margin: Size::default(),
            paint_margin: Edges::default(),
            trigger_focus: cx.focus_handle(),
            item_focus: std::array::from_fn(|_| cx.focus_handle()),
            sibling_item_focus: cx.focus_handle(),
            before_focus: cx.focus_handle(),
            after_focus: cx.focus_handle(),
            trigger_node: None,
        })
    }

    fn update(view: &Entity<MenuView>, cx: &mut VisualTestContext, f: impl FnOnce(&mut MenuView)) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn expanded(view: &Entity<MenuView>, cx: &mut VisualTestContext) -> bool {
        view.read_with(cx, |view, _| view.expanded)
    }

    fn requests(view: &Entity<MenuView>, cx: &mut VisualTestContext) -> Vec<bool> {
        view.read_with(cx, |view, _| view.requests.borrow().clone())
    }

    fn clicks(view: &Entity<MenuView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.clicks.borrow().clone())
    }

    /// フォーカスのある要素の名前
    fn focused(view: &Entity<MenuView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let (trigger, items, before, after) = view.read_with(cx, |view, _| {
            (
                view.trigger_focus.clone(),
                view.item_focus.clone(),
                view.before_focus.clone(),
                view.after_focus.clone(),
            )
        });
        cx.update(|window, _| {
            [
                ("trigger", &trigger),
                ("before", &before),
                ("after", &after),
            ]
            .into_iter()
            .chain(ITEMS.iter().copied().zip(items.iter()))
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

    /// `color` で塗った矩形の、切り取られた後に見える範囲の x, y, 幅, 高さ。描いていなければ `None`
    fn quad_bounds(cx: &mut VisualTestContext, color: Hsla) -> Option<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.background == color.into())
                .map(|quad| {
                    let b = quad.bounds.intersect(&quad.content_mask.bounds);
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
        })
    }

    /// ボタンを Tab でフォーカスしてから、`key` で開く
    fn open_with(cx: &mut VisualTestContext, key: &'static str) {
        cx.simulate_keystrokes("tab tab");
        press_key(cx, key);
    }

    #[gpui::test]
    fn クリックで開き最初の項目にフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        assert!(expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("cut"));
    }

    #[gpui::test]
    fn enter_と_space_と下矢印キーで開き最初の項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        for key in ["enter", "space", "down"] {
            let (view, cx) = setup(cx);
            open_with(cx, key);
            assert!(expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some("cut"), "{key}");
        }
    }

    #[gpui::test]
    fn 同じ親に同じ_id_のメニューがあっても開閉の状態を分ける(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.sibling = true);
        click(cx, 110., 110.);
        assert!(expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("cut"));
    }

    #[gpui::test]
    fn 閉じるメニューと開くメニューを同時に切り替えると開いた方にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        for sibling_first in [false, true] {
            let (view, cx) = setup(cx);
            update(&view, cx, |view| {
                view.sibling = true;
                view.sibling_first = sibling_first;
            });
            click(cx, 110., 110.);
            assert_eq!(focused(&view, cx), Some("cut"), "{sibling_first}");
            update(&view, cx, |view| {
                view.expanded = false;
                view.sibling_expanded = true;
            });
            let sibling_item = view.read_with(cx, |view, _| view.sibling_item_focus.clone());
            assert!(
                cx.update(|window, _| sibling_item.is_focused(window)),
                "{sibling_first}"
            );
        }
    }

    #[gpui::test]
    fn 上矢印キーで開き最後の項目にフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "up");
        assert!(expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("align"));
    }

    #[gpui::test]
    fn 上下の矢印キーで項目の間を移り端では反対の端に戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        let mut path = Vec::new();
        for _ in 0..4 {
            cx.simulate_keystrokes("down");
            path.push(focused(&view, cx));
        }
        assert_eq!(
            path,
            [Some("copy"), Some("bold"), Some("align"), Some("cut")],
            "区切りを飛ばして組の中へ移り、最後の次は最初に戻ること"
        );
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("align"));
    }

    #[gpui::test]
    fn home_と_end_で最初と最後の項目に移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        cx.simulate_keystrokes("end");
        assert_eq!(focused(&view, cx), Some("align"));
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&view, cx), Some("cut"));
    }

    #[gpui::test]
    fn 無効な項目は飛ばし_無効でも到達できる項目は操作できない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.availability[0] = Availability::Disabled;
            view.availability[1] = Availability::DisabledFocusable;
        });
        open_with(cx, "down");
        assert_eq!(
            focused(&view, cx),
            Some("copy"),
            "到達できない項目を飛ばして開くこと"
        );
        press_key(cx, "enter");
        click(cx, 150., 130.);
        assert_eq!(clicks(&view, cx), Vec::<&str>::new());
        assert!(expanded(&view, cx), "無効な項目の操作では閉じないこと");
        cx.simulate_keystrokes("end down");
        assert_eq!(focused(&view, cx), Some("copy"));
    }

    #[gpui::test]
    fn escape_で閉じてボタンにフォーカスを戻す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        cx.simulate_keystrokes("shift-escape");
        assert!(expanded(&view, cx), "修飾キー付きでは閉じないこと");
        cx.simulate_keystrokes("escape");
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("trigger"));
        assert_eq!(requests(&view, cx), [true, false]);
    }

    #[gpui::test]
    fn 項目を操作すると閉じてボタンにフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        for key in ["enter", "space"] {
            let (view, cx) = setup(cx);
            open_with(cx, "down");
            press_key(cx, key);
            assert_eq!(clicks(&view, cx), ["cut"], "{key}");
            assert!(!expanded(&view, cx), "{key}");
            assert_eq!(focused(&view, cx), Some("trigger"), "{key}");
            assert_eq!(requests(&view, cx), [true, false], "{key}");
        }

        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        // 2 つ目の項目は y 140 から 160
        click(cx, 150., 150.);
        assert_eq!(clicks(&view, cx), ["copy"]);
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("trigger"));
        assert_eq!(requests(&view, cx), [true, false]);
    }

    #[gpui::test]
    fn 切り替える項目は_space_とクリックでは開いたまま_enter_で閉じる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        cx.simulate_keystrokes("down down");
        press_key(cx, "space");
        assert!(expanded(&view, cx));
        cx.simulate_keystrokes("down");
        press_key(cx, "space");
        assert!(expanded(&view, cx));
        // 3 つ目の項目は区切りの後の y 164 から 184
        click(cx, 150., 170.);
        assert!(expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("bold"));
        press_key(cx, "enter");
        assert_eq!(clicks(&view, cx), ["bold", "align", "bold", "bold"]);
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("trigger"));
        assert_eq!(requests(&view, cx), [true, false]);
    }

    #[gpui::test]
    fn tab_で閉じてボタンの次の要素へ_shift_tab_で閉じてボタンへ移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        cx.simulate_keystrokes("tab");
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("after"));
        assert_eq!(requests(&view, cx), [true, false]);

        cx.simulate_keystrokes("shift-tab");
        press_key(cx, "down");
        cx.simulate_keystrokes("shift-tab");
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("trigger"));
        assert_eq!(requests(&view, cx), [true, false, true, false]);
    }

    #[gpui::test]
    fn 外でマウスのボタンを押すと閉じ_押下はそのまま届く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        click(cx, 220., 220.);
        assert!(!expanded(&view, cx));
        let behind_clicks = view.read_with(cx, |view, _| *view.behind_clicks.borrow());
        assert_eq!(behind_clicks, 1);
        assert_eq!(requests(&view, cx), [true, false]);
        assert_eq!(
            focused(&view, cx),
            Some("trigger"),
            "フォーカスできない要素を押した場合はボタンに戻ること"
        );

        press_key(cx, "down");
        click(cx, 405., 5.);
        assert!(!expanded(&view, cx));
        assert_eq!(
            focused(&view, cx),
            Some("after"),
            "押した要素がフォーカスを受け取ること"
        );
        assert_eq!(requests(&view, cx), [true, false, true, false]);
    }

    /// 余白は四方 10 で、メニューは (100, 120) から 100x84。
    /// 余白の上端の帯 (y 110 から 120) はボタン (y 100 から 120) に重なる
    #[gpui::test]
    fn 余白の上の押下は背後に届きメニューの外として扱う(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.paint_margin = Edges::all(px(10.));
        });
        click(cx, 110., 110.);
        assert!(expanded(&view, cx));
        click(cx, 110., 115.);
        assert!(!expanded(&view, cx), "余白に重なるボタンを押すと閉じること");
        assert_eq!(requests(&view, cx), [true, false]);

        click(cx, 110., 110.);
        // 背後の要素 (200, 200) から 50x50 と、右下の余白が重なる位置
        click(cx, 205., 208.);
        assert!(!expanded(&view, cx));
        let behind_clicks = view.read_with(cx, |view, _| *view.behind_clicks.borrow());
        assert_eq!(behind_clicks, 1);
    }

    /// 余白は四方 10 で、ボタンを (10, -10) に置くと、メニューは (10, 10) から、余白は (0, 0) から始まり、
    /// (0, 0) から 10x10 の before ボタンに重なる
    #[gpui::test]
    fn 余白の上で押した要素にフォーカスが移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.paint_margin = Edges::all(px(10.));
            view.trigger_origin = point(px(10.), px(-10.));
        });
        click(cx, 20., 5.);
        assert!(expanded(&view, cx));
        click(cx, 5., 5.);
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("before"));
    }

    /// 高さ 60 のウィンドウで余白を四方 10 にすると、余白と合わせた高さが 60 になる。
    /// メニューは y 0 から、スクロールの範囲は y -10 から 50 に置かれ、組の中の最初の項目 (y 44 から 64) は下の余白の上に一部が描かれる
    #[gpui::test]
    fn 余白の上に描かれた項目は押せ切り取られた部分はメニューの外として扱う(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_resize(size(px(500.), px(60.)));
        update(&view, cx, |view| {
            view.paint_margin = Edges::all(px(10.));
            view.trigger_origin = point(px(100.), px(20.));
        });
        click(cx, 110., 30.);
        click(cx, 150., 46.);
        assert_eq!(clicks(&view, cx), ["bold"]);
        assert!(
            expanded(&view, cx),
            "切り替える項目のクリックでは閉じないこと"
        );
        click(cx, 150., 55.);
        assert!(!expanded(&view, cx));
        assert_eq!(requests(&view, cx), [true, false]);
    }

    /// 高さ 60 のウィンドウで余白を四方 10 にすると、スクロールの範囲は y -10 から 50 で、余白を除いた範囲は y 0 から 40
    #[gpui::test]
    fn キーでフォーカスした項目は余白を除いた範囲に見せる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_resize(size(px(500.), px(60.)));
        update(&view, cx, |view| {
            view.paint_margin = Edges::all(px(10.));
            view.trigger_origin = point(px(100.), px(20.));
        });
        click(cx, 110., 30.);
        cx.simulate_keystrokes("end");
        assert_eq!(
            quad_bounds(cx, LAST_ITEM_COLOR()),
            Some((100.0, 20.0, 100.0, 20.0))
        );
        cx.simulate_keystrokes("home");
        assert_eq!(
            quad_bounds(cx, FIRST_ITEM_COLOR()),
            Some((100.0, 0.0, 100.0, 20.0))
        );
    }

    #[gpui::test]
    fn フォーカスのある項目が消えるとメニューにフォーカスを移しキー操作を続けられる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        assert_eq!(focused(&view, cx), Some("cut"));
        update(&view, cx, |view| view.hide_cut = true);
        assert_eq!(focused(&view, cx), None, "メニューにフォーカスがあること");
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("copy"));

        update(&view, cx, |view| view.hide_cut = false);
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("cut"));
        update(&view, cx, |view| view.hide_cut = true);
        // メニューにフォーカスがある間に閉じても、ボタンに戻す
        cx.simulate_keystrokes("escape");
        assert!(!expanded(&view, cx));
        assert_eq!(focused(&view, cx), Some("trigger"));
    }

    #[gpui::test]
    fn 開いている間にボタンをクリックすると閉じフォーカスはボタンに残る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        click(cx, 110., 110.);
        assert!(!expanded(&view, cx));
        assert_eq!(requests(&view, cx), [true, false]);
        assert_eq!(focused(&view, cx), Some("trigger"));
    }

    #[gpui::test]
    fn メニューの上のクリックは背後に届かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.trigger_origin = point(px(200.), px(180.))
        });
        click(cx, 210., 190.);
        // 区切りは y 240 から 244 で、背後の要素の上にある
        click(cx, 220., 242.);
        assert!(expanded(&view, cx));
        let behind_clicks = view.read_with(cx, |view, _| *view.behind_clicks.borrow());
        assert_eq!(behind_clicks, 0);
    }

    #[gpui::test]
    fn 項目にマウスが乗るとその項目にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 110., 110.);
        cx.simulate_mouse_move(point(px(150.), px(175.)), None, Modifiers::none());
        assert_eq!(focused(&view, cx), Some("bold"));
    }

    #[gpui::test]
    fn 開いている間はボタンの矢印キーで項目にフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with(cx, "down");
        let trigger_focus = view.read_with(cx, |view, _| view.trigger_focus.clone());
        cx.update(|window, cx| window.focus(&trigger_focus, cx));
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("align"));
        assert_eq!(requests(&view, cx), [true]);
    }

    #[gpui::test]
    fn 無効なボタンでは開かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.trigger_disabled = true);
        click(cx, 110., 110.);
        let trigger_focus = view.read_with(cx, |view, _| view.trigger_focus.clone());
        cx.update(|window, cx| window.focus(&trigger_focus, cx));
        cx.simulate_keystrokes("down");
        press_key(cx, "enter");
        assert_eq!(requests(&view, cx), Vec::<bool>::new());
    }

    #[gpui::test]
    fn ボタンの下に左端をそろえて置く(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx);
        assert_eq!(quad_bounds(cx, MENU_COLOR()), None);
        click(cx, 110., 110.);
        assert_eq!(
            quad_bounds(cx, MENU_COLOR()),
            Some((100.0, 120.0, 100.0, 84.0))
        );
    }

    /// 四方 10 の余白は位置の計算に含めないため、メニューは (100, 120) から 100x84 になる。
    /// メニューの下端 204 からウィンドウの端の余白 50 を空けると高さ 260 のウィンドウに収まるが、余白の下端 214 で判定すると収まらない
    #[gpui::test]
    fn 余白を除いたメニューをボタンに合わせて置く(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| {
            view.paint_margin = Edges::all(px(10.));
            view.window_margin = size(px(0.), px(50.));
        });
        cx.simulate_resize(size(px(500.), px(260.)));
        click(cx, 110., 110.);
        assert_eq!(
            quad_bounds(cx, MENU_COLOR()),
            Some((100.0, 120.0, 100.0, 84.0))
        );
    }

    /// 高さ 60 のウィンドウでは、高さ 84 のメニューを 60 にして中央に置き、超える分はスクロールする。
    /// 組の中の最後の項目は y 64 から始まり、ウィンドウの外にあるため描かれない
    #[gpui::test]
    fn 収まらない高さはスクロールしフォーカスした項目を見せる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_resize(size(px(500.), px(60.)));
        update(&view, cx, |view| {
            view.trigger_origin = point(px(100.), px(20.))
        });
        click(cx, 110., 30.);
        assert_eq!(
            quad_bounds(cx, MENU_COLOR()),
            Some((100.0, 0.0, 100.0, 60.0)),
            "ウィンドウの高さまでにすること"
        );
        assert_eq!(quad_bounds(cx, LAST_ITEM_COLOR()), None);
        cx.simulate_keystrokes("end");
        assert_eq!(
            quad_bounds(cx, LAST_ITEM_COLOR()),
            Some((100.0, 40.0, 100.0, 20.0)),
            "最後の項目が見えるよう、下端をそろえるだけスクロールすること"
        );
        assert_eq!(
            quad_bounds(cx, FIRST_ITEM_COLOR()),
            None,
            "最初の項目は見えなくなること"
        );
        cx.simulate_keystrokes("home");
        assert_eq!(
            quad_bounds(cx, FIRST_ITEM_COLOR()),
            Some((100.0, 0.0, 100.0, 20.0)),
            "最初の項目が見えるよう戻すこと"
        );
        // 3 つ目の項目は y 44 から 64 で、下端が 4 だけ見えない
        cx.simulate_keystrokes("down down");
        assert_eq!(
            quad_bounds(cx, FIRST_ITEM_COLOR()),
            Some((100.0, 0.0, 100.0, 16.0)),
            "見えない分だけスクロールし、最初の項目の上端 4 が切り取られること"
        );
    }

    #[gpui::test]
    fn ボタンはメニューを開くことと開いているかを公開する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        let node = view
            .read_with(cx, |view, _| view.trigger_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::Button);
        assert_eq!(node.has_popup(), Some(accesskit::HasPopup::Menu));
        assert_eq!(node.is_expanded(), Some(false));
        click(cx, 110., 110.);
        let node = view
            .read_with(cx, |view, _| view.trigger_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.is_expanded(), Some(true));
    }

    /// 項目だけを描画したときの a11y ノード
    fn item_node(cx: &mut VisualTestContext, item: MenuItem) -> accesskit::Node {
        cx.update(|window, cx| {
            let element = item_element(
                item.aria_label("Cut"),
                &cx.focus_handle(),
                Rc::default(),
                None,
                window,
            )
            .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        })
    }

    #[gpui::test]
    fn 項目は種類に応じた_role_と切り替えの状態で公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        let cases = [
            (MenuItemKind::Action, accesskit::Role::MenuItem, None),
            (
                MenuItemKind::Checkbox(CheckedState::Mixed),
                accesskit::Role::MenuItemCheckBox,
                Some(accesskit::Toggled::Mixed),
            ),
            (
                MenuItemKind::Radio(true),
                accesskit::Role::MenuItemRadio,
                Some(accesskit::Toggled::True),
            ),
        ];
        for (kind, role, toggled) in cases {
            let node = item_node(cx, MenuItem::new("item").kind(kind));
            assert_eq!(node.role(), role, "{kind:?}");
            assert_eq!(node.toggled(), toggled, "{kind:?}");
            assert_eq!(node.label(), Some("Cut"), "{kind:?}");
            assert!(node.supports_action(accesskit::Action::Click), "{kind:?}");
            assert!(node.supports_action(accesskit::Action::Focus), "{kind:?}");
        }
    }

    #[gpui::test]
    fn 無効な項目は_disabled_として公開され既定ではフォーカス操作だけを公開する(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        let node = item_node(cx, MenuItem::new("item").disabled(true));
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));

        let node = item_node(
            cx,
            MenuItem::new("item").availability(Availability::Disabled),
        );
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Focus));
    }

    fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    /// Compose の `MenuAnchorPosition.Below` の候補を、500x400 のウィンドウ、左右 8 と上下 48 の余白、
    /// 100x84 のメニューについて評価した位置
    #[test]
    fn 位置は余白の内側に収まる最初の候補を選ぶ() {
        let viewport = size(px(500.), px(400.));
        let margin = size(px(8.), px(48.));
        let menu = size(px(100.), px(84.));
        let cases = [
            // ボタンの下に左端をそろえる
            (bounds(100., 100., 40., 20.), (100., 120.)),
            // 右に収まらなければ右端をそろえる。下に収まらなければ上に置く
            (bounds(420., 300., 40., 20.), (360., 216.)),
            // 左右どちらにそろえても収まらなければ、ボタンの中心に近い左の余白に寄せる
            (bounds(0., 100., 95., 20.), (8., 120.)),
            // 上下どちらにも収まらなければ、中心をボタンの上端にそろえる
            (bounds(100., 100., 40., 220.), (100., 58.)),
            // それでも収まらなければ、ボタンの中心に近い下の余白に寄せる
            (bounds(100., 60., 40., 300.), (100., 268.)),
        ];
        for (trigger, (x, y)) in cases {
            assert_eq!(
                position(trigger, menu, viewport, margin),
                point(px(x), px(y)),
                "{trigger:?}"
            );
        }
    }

    #[test]
    fn 余白を除いた空きより大きいメニューは中央に置く() {
        let viewport = size(px(500.), px(400.));
        let margin = size(px(8.), px(48.));
        assert_eq!(
            position(
                bounds(0., 0., 40., 20.),
                size(px(490.), px(310.)),
                viewport,
                margin
            ),
            point(px(5.), px(45.))
        );
    }
}
