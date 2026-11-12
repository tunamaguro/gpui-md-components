//! 選んだタブの内容をパネルに表示する、見た目を持たないタブ

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    MouseButton, ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, accesskit, div, prelude::FluentBuilder,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    roving::{self, GroupChild, RovingItem},
};

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// 見た目を持たないタブの並び
///
/// 状態 (選ばれているタブの値) は呼び出し側が持ち、[`TabList::new`] に毎回渡す。並び自身は状態を変えない。
/// タブは [`TabList::item`] で、タブでない要素は `child` で加え、加えた順に並ぶ。
/// タブは横に並べる。選ばれたタブの内容は、呼び出し側が [`TabPanel`] に入れて表示する
///
/// - Tab で到達できるタブは 1 つだけで、選ばれているタブ、最初のフォーカスできるタブの順に選ぶ。
///   タブにフォーカスがある間は、そのタブだけが Tab で到達できる。フォーカスできるタブが無ければ Tab で入らない
/// - 修飾キーなしの左右の矢印キーで前後のタブへフォーカスを移し、移った先のタブを選ぶよう求める。端では反対の端に戻る。
///   Home と End では最初と最後のタブへ移り、移った先を選ぶよう求める。
///   [`Availability::Disabled`] のタブは飛ばし、[`Availability::DisabledFocusable`] のタブにはフォーカスだけを移す。
///   移れるタブがほかに無い場合は、キー入力を祖先に渡す
/// - フォーカス中の Enter と Space・マウスのクリック・支援技術からのクリック操作で、タブを選ぶよう求める
///
/// 支援技術には role `tablist` と横向きとして公開され、タブは role `tab` と選ばれているかどうかの状態として公開される。
/// 並びの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct TabList<T: PartialEq + 'static> {
    id: ElementId,
    inner: Stateful<Div>,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    children: Vec<GroupChild<Tab<T>>>,
}

impl<T: PartialEq + 'static> TabList<T> {
    /// `selected` の値を持つタブを選ばれた状態で表示する並びを作る。`None` ならどのタブも選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            selected,
            on_change: None,
            children: Vec::new(),
        }
    }

    /// タブを選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められたタブの値を渡す。選ばれているタブと無効なタブでは呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`TabList::new`] に渡すこと。
    /// 設定しない場合、タブは操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾にタブを加える
    pub fn item(mut self, item: Tab<T>) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` のタブを順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = Tab<T>>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for TabList<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut values = Vec::new();
        let mut items = Vec::new();
        let children: Vec<GroupChild<Stateful<Div>>> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(tab) => {
                    items.push(RovingItem {
                        // タブの id は並びの中でだけ一意なので、並びの id で分ける
                        focus_handle: window.with_element_namespace(self.id.clone(), |window| {
                            tab.focus_handle(window, cx)
                        }),
                        availability: tab.availability,
                    });
                    values.push(tab.value);
                    GroupChild::Item(tab.inner)
                }
                GroupChild::Other(element) => GroupChild::Other(element),
            })
            .collect();
        let values: Rc<[T]> = values.into();
        let items: Rc<[RovingItem]> = items.into();
        let selected_ix = values
            .iter()
            .position(|value| Some(value) == self.selected.as_ref());
        let tab_stop = roving::tab_stop(&items, selected_ix, window);

        let select = self.on_change.map(|on_change| {
            let items = items.clone();
            Rc::new(move |ix: usize, window: &mut Window, cx: &mut App| {
                if !items[ix].availability.is_disabled() && Some(ix) != selected_ix {
                    on_change(&values[ix], window, cx);
                }
            })
        });

        let mut ix = 0;
        let elements: Vec<AnyElement> = children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(inner) => {
                    let select = select.clone().map(|select| {
                        move |window: &mut Window, cx: &mut App| select(ix, window, cx)
                    });
                    let element = tab_element(
                        inner,
                        &items[ix],
                        tab_stop == Some(ix),
                        selected_ix == Some(ix),
                        select,
                        window,
                    );
                    ix += 1;
                    element.into_any_element()
                }
                GroupChild::Other(element) => element,
            })
            .collect();

        self.inner
            .role(accesskit::Role::TabList)
            .aria_orientation(accesskit::Orientation::Horizontal)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let current = items
                    .iter()
                    .position(|item| item.focus_handle.is_focused(window));
                let target = match event.keystroke.key.as_str() {
                    "right" => roving::neighbor(&items, true, window),
                    "left" => roving::neighbor(&items, false, window),
                    // どのタブにもフォーカスが無い場合は、矢印キーと同じく扱わない
                    "home" => current.and_then(|_| {
                        items
                            .iter()
                            .position(|item| item.availability.is_focusable())
                    }),
                    "end" => current.and_then(|_| {
                        items
                            .iter()
                            .rposition(|item| item.availability.is_focusable())
                    }),
                    _ => return,
                };
                let Some(ix) = target.filter(|&ix| Some(ix) != current) else {
                    return;
                };
                window.focus(&items[ix].focus_handle, cx);
                if let Some(select) = &select {
                    select(ix, window, cx);
                }
                cx.stop_propagation();
            })
            .children(elements)
    }
}

// TODO: フォーカス・無効状態・クリックの扱いは [`Radio`](crate::radio::Radio) の項目と同じ処理を複製している。1 つにまとめたい
/// `inner` を、並びの中の位置に応じたタブにする
///
/// `on_select` はタブが操作されたときに呼ぶ。無効なタブでは呼ばない
fn tab_element<S>(
    inner: Stateful<Div>,
    item: &RovingItem,
    tab_stop: bool,
    selected: bool,
    on_select: Option<S>,
    window: &Window,
) -> impl IntoElement + use<S>
where
    S: Fn(&mut Window, &mut App) + 'static,
{
    let focus_handle = item.focus_handle.clone();
    // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    let focusable = item.availability.is_focusable() || focus_handle.is_focused(window);
    let disabled = item.availability.is_disabled();
    let on_select = on_select.filter(|_| !disabled);
    inner
        .role(accesskit::Role::Tab)
        .aria_selected(selected)
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
            this.track_focus(&focus_handle.tab_stop(tab_stop))
        })
        .when_some(on_select, |this, on_select| {
            this.on_click(move |_, window, cx| on_select(window, cx))
        })
        .a11y_info(move |node: &mut accesskit::Node| {
            if disabled {
                node.set_disabled();
            }
        })
}

impl<T: PartialEq + 'static> Styled for TabList<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T: PartialEq + 'static> ParentElement for TabList<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl<T: PartialEq + 'static> InteractiveElement for TabList<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T: PartialEq + 'static> StatefulInteractiveElement for TabList<T> {}

/// [`TabList`] のタブ
///
/// [`TabList::item`] に渡して使う。タブが表す値が、並びの選ばれている値と等しいときに選ばれた状態になる。
/// アクセシブルな名前は `aria_label` で与えること。子の内容は名前にならない。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct Tab<T> {
    id: ElementId,
    inner: Stateful<Div>,
    value: T,
    availability: Availability,
    provided_focus_handle: Option<FocusHandle>,
}

impl<T> Tab<T> {
    /// `value` を表すタブを作る
    ///
    /// `id` と `value` は並びの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値のタブが複数あると、そのうち最初のタブだけが選ばれた状態になる。
    /// [`Tab::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value,
            availability: Availability::Enabled,
            provided_focus_handle: None,
        }
    }

    /// タブを無効にするか、無効でも Tab や矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なタブは操作されても選ぶよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Tab::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// タブを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なタブは操作されても選ぶよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// Tab や矢印キーでも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも到達できるようにしたい場合は [`Tab::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのタブのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab で到達できるかは並びが決める。
    /// 指定しない場合はタブが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle().tab_index(0))
                .read(cx)
                .clone()
        })
    }
}

impl<T> Styled for Tab<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T> ParentElement for Tab<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<T> InteractiveElement for Tab<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T> StatefulInteractiveElement for Tab<T> {}

/// 選ばれたタブの内容を入れる、見た目を持たないパネル
///
/// 支援技術には role `tabpanel` として公開される。名前は `aria_label` で、選ばれたタブと同じ名前を与えること。
/// パネル自身はフォーカスを持たない。パネルの中に Tab で到達できる要素が無い場合と、最初の内容がフォーカスできない場合は、
/// `tab_stop(true)` にしたフォーカスを `track_focus` で渡し、Tab でパネルに到達できるようにすること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct TabPanel {
    inner: Stateful<Div>,
}

impl TabPanel {
    /// 中身の無いパネルを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            inner: div().id(id),
        }
    }
}

impl RenderOnce for TabPanel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.inner.role(accesskit::Role::TabPanel)
    }
}

impl Styled for TabPanel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for TabPanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for TabPanel {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for TabPanel {}

/// APG の Tabs パターン (content/patterns/tabs/tabs-pattern.html) と
/// Keyboard Interface の roving tabindex (content/practices/keyboard-interface/keyboard-interface-practice.html)
/// で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::test_window::add_test_window;

    const VALUES: [&str; 3] = ["apple", "pear", "plum"];

    struct ListView {
        selected: Option<&'static str>,
        availability: [Availability; 3],
        /// `false` なら求められた値を `selected` に反映しない
        accept: bool,
        /// `false` なら `on_change` を設定しない
        handler: bool,
        /// 最初のタブの後に、タブでない要素を置く
        divider: bool,
        handles: [FocusHandle; 3],
        /// 並びの後に置くパネルのフォーカス。パネルはフォーカスできる子を持たないため、Tab で到達できるようにする
        panel: FocusHandle,
        changes: Rc<RefCell<Vec<&'static str>>>,
        /// 並びの親に届いたキー
        propagated: Rc<RefCell<Vec<String>>>,
        list_node: Option<accesskit::Node>,
    }

    impl Render for ListView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut tabs = VALUES.iter().enumerate().map(|(ix, value)| {
                Tab::new(*value, *value)
                    .availability(self.availability[ix])
                    .track_focus(&self.handles[ix])
                    .aria_label(*value)
                    .w(px(50.))
                    .h(px(20.))
            });
            let list = TabList::new("fruit", self.selected)
                .when(self.handler, |this| {
                    this.on_change(cx.listener(|this, value: &&'static str, _, cx| {
                        this.changes.borrow_mut().push(*value);
                        if this.accept {
                            this.selected = Some(*value);
                            cx.notify();
                        }
                    }))
                })
                .aria_label("Fruit")
                .flex()
                .items(tabs.next())
                .when(self.divider, |this| this.child(div().w(px(50.))))
                .items(tabs)
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(list.a11y_role().expect("role を持つこと"));
            list.write_a11y_info(&mut node);
            self.list_node = Some(node);
            let propagated = self.propagated.clone();
            div()
                .size_full()
                .on_key_down(move |event, _, _| {
                    propagated.borrow_mut().push(event.keystroke.key.clone())
                })
                .child(list)
                .child(
                    TabPanel::new("panel")
                        .aria_label("Panel")
                        .track_focus(&self.panel)
                        .h(px(20.)),
                )
        }
    }

    fn setup<'a>(
        cx: &'a mut TestAppContext,
        selected: Option<&'static str>,
        availability: [Availability; 3],
    ) -> (Entity<ListView>, &'a mut VisualTestContext) {
        add_test_window(cx, |_, cx| ListView {
            selected,
            availability,
            accept: true,
            handler: true,
            divider: false,
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            panel: cx.focus_handle().tab_stop(true),
            changes: Rc::default(),
            propagated: Rc::default(),
            list_node: None,
        })
    }

    const ALL_ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのあるタブの値
    fn focused(view: &Entity<ListView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| {
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| VALUES[ix])
        })
    }

    fn is_panel_focused(view: &Entity<ListView>, cx: &mut VisualTestContext) -> bool {
        let panel = view.read_with(cx, |view, _| view.panel.clone());
        cx.update(|window, _| panel.is_focused(window))
    }

    fn changes(view: &Entity<ListView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn update(view: &Entity<ListView>, cx: &mut VisualTestContext, f: impl FnOnce(&mut ListView)) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    /// 左から `column` 番目 (0 始まり) の幅 50px の列をクリックする
    fn click_column(cx: &mut VisualTestContext, column: usize) {
        cx.simulate_click(
            point(px(25. + 50. * column as f32), px(10.)),
            Modifiers::none(),
        );
    }

    /// タブだけを描画したときの a11y ノード
    fn tab_node(
        cx: &mut VisualTestContext,
        availability: Availability,
        selected: bool,
        has_handler: bool,
    ) -> accesskit::Node {
        cx.update(|window, cx| {
            let item = RovingItem {
                focus_handle: cx.focus_handle(),
                availability,
            };
            let element = tab_element(
                div().id("tab").aria_label("Apple"),
                &item,
                true,
                selected,
                has_handler.then_some(|_: &mut Window, _: &mut App| {}),
                window,
            )
            .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        })
    }

    #[gpui::test]
    fn 並びは_role_tablist_と横向きと名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        let node = view
            .read_with(cx, |view, _| view.list_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::TabList);
        assert_eq!(node.orientation(), Some(accesskit::Orientation::Horizontal));
        assert_eq!(node.label(), Some("Fruit"));
    }

    #[gpui::test]
    fn タブは_role_tab_と選ばれているかどうかで公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        for selected in [false, true] {
            let node = tab_node(cx, Availability::Enabled, selected, true);
            assert_eq!(node.role(), accesskit::Role::Tab);
            assert_eq!(node.label(), Some("Apple"));
            assert_eq!(node.is_selected(), Some(selected));
            assert!(!node.is_disabled());
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(node.supports_action(accesskit::Action::Focus));
        }
    }

    #[gpui::test]
    fn ハンドラを持たない並びのタブはクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        let node = tab_node(cx, Availability::Enabled, false, false);
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効なタブは_disabled_として公開され操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        let node = tab_node(cx, Availability::Disabled, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));

        let node = tab_node(cx, Availability::DisabledFocusable, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(
            node.supports_action(accesskit::Action::Focus),
            "無効でも到達できるタブはフォーカス操作を公開すること"
        );
    }

    #[gpui::test]
    fn パネルは_role_tabpanel_と名前で公開される(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        let node = cx.update(|window, cx| {
            let element = TabPanel::new("panel")
                .aria_label("Apple")
                .render(window, cx)
                .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        });
        assert_eq!(node.role(), accesskit::Role::TabPanel);
        assert_eq!(node.label(), Some("Apple"));
    }

    #[gpui::test]
    fn tab_で選ばれているタブに入り_もう一度の_tab_でパネルへ移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("pear"), ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("tab");
        assert!(
            is_panel_focused(&view, cx),
            "並びの中の Tab stop は 1 つだけで、Tab で到達できるようにしたパネルへ移ること"
        );
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("pear"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn どれも選ばれていなければ_tab_で最初のフォーカスできるタブに入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            None,
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("pear"));
        assert!(
            changes(&view, cx).is_empty(),
            "Tab で入っただけでは選ぶよう求めないこと"
        );
    }

    #[gpui::test]
    fn 右と左の矢印キーで隣のタブへ移り選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        let propagated = view.read_with(cx, |view, _| view.propagated.clone());
        propagated.borrow_mut().clear();
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert_eq!(changes(&view, cx), vec!["pear", "apple"]);
        assert!(
            propagated.borrow().is_empty(),
            "移った場合はキー入力を祖先に渡さないこと"
        );
    }

    #[gpui::test]
    fn 矢印キーの移動は端で反対の端に戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab left");
        assert_eq!(focused(&view, cx), Some("plum"));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert_eq!(changes(&view, cx), vec!["plum", "apple"]);
    }

    #[gpui::test]
    fn 上下の矢印キーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab down up");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn home_と_end_で端のフォーカスできるタブへ移り選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            Some("pear"),
            [
                Availability::Enabled,
                Availability::Enabled,
                Availability::Disabled,
            ],
        );
        cx.simulate_keystrokes("tab end");
        assert_eq!(
            focused(&view, cx),
            Some("pear"),
            "無効な最後のタブは飛ばすこと"
        );
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert_eq!(changes(&view, cx), vec!["apple"]);
    }

    #[gpui::test]
    fn 矢印キーの移動は無効なタブを飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("plum"));
        assert_eq!(changes(&view, cx), vec!["plum"]);
    }

    #[gpui::test]
    fn 無効でも到達できるタブにはフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Enabled,
                Availability::DisabledFocusable,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("pear"));
        press_key(cx, "space");
        press_key(cx, "enter");
        click_column(cx, 1);
        assert!(changes(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), Some("pear"));
    }

    #[gpui::test]
    fn 移れるタブがほかに無ければキー入力を祖先に渡す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Disabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        let propagated = view.read_with(cx, |view, _| view.propagated.clone());
        propagated.borrow_mut().clear();
        cx.simulate_keystrokes("right left home end");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(changes(&view, cx).is_empty());
        assert_eq!(*propagated.borrow(), vec!["right", "left", "home", "end"]);
    }

    #[gpui::test]
    fn 修飾キー付きのキーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab shift-right ctrl-right alt-left ctrl-end");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn space_と_enter_で選ばれていないタブを選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        update(&view, cx, |view| view.accept = false);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        press_key(cx, "enter");
        assert_eq!(changes(&view, cx), vec!["apple", "apple"]);

        update(&view, cx, |view| view.selected = Some("apple"));
        press_key(cx, "space");
        press_key(cx, "enter");
        assert_eq!(
            changes(&view, cx),
            vec!["apple", "apple"],
            "選ばれているタブでは求めないこと"
        );
    }

    #[gpui::test]
    fn クリックで選ぶよう求めフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        click_column(cx, 2);
        assert_eq!(changes(&view, cx), vec!["plum"]);
        assert_eq!(focused(&view, cx), Some("plum"));
        click_column(cx, 2);
        assert_eq!(
            changes(&view, cx),
            vec!["plum"],
            "選ばれているタブでは求めないこと"
        );
    }

    #[gpui::test]
    fn 無効なタブはクリックしても選ばれずフォーカスされない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        click_column(cx, 1);
        assert!(changes(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn ハンドラが無くても矢印キーでフォーカスは移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.handler = false);
        cx.simulate_keystrokes("tab right end");
        assert_eq!(focused(&view, cx), Some("plum"));
    }

    #[gpui::test]
    fn 自身では状態を変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.accept = false);
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("tab");
        assert!(
            is_panel_focused(&view, cx),
            "値が反映されなくても Tab で並びの外へ出ること"
        );
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some("apple"),
            "Tab で入る先は渡された値のタブであること"
        );
        assert_eq!(changes(&view, cx), vec!["pear"]);
    }

    #[gpui::test]
    fn タブでない子は加えた順に並び矢印キーの移動に含まれない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.divider = true);
        click_column(cx, 1);
        assert!(
            changes(&view, cx).is_empty(),
            "最初のタブの次の列はタブでないこと"
        );
        click_column(cx, 2);
        assert_eq!(changes(&view, cx), vec!["pear"]);
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some("apple"));
    }
}
