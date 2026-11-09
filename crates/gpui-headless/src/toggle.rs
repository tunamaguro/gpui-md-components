//! 押下状態を持つ、見た目を持たないボタンとそのグループ

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    MouseButton, ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, accesskit, div, prelude::FluentBuilder,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    roving::{self, NextItem, PreviousItem, RovingItem},
};

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// 押下状態を持つ、見た目を持たないボタン
///
/// 状態 (押されているか) は呼び出し側が持ち、[`ToggleButton::new`] に毎回渡す。部品自身は状態を変えない。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作で [`ToggleButton::on_change`] のハンドラを呼ぶ。
///
/// 支援技術には role `button` と押されているかどうかの状態 (pressed) として公開される。
/// アクセシブルな名前は、子孫に置いたテキストか `aria_label` で与えること。
/// テキストは `gpui::text!` のように id を持つ要素にする。`&str`・`String`・`SharedString` をそのまま置いても名前にならない。
/// 状態によって名前を変えないこと。名前を変える場合は、押下状態を持たない [`Button`](crate::button::Button) を使う。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct ToggleButton {
    id: ElementId,
    inner: Stateful<Div>,
    pressed: bool,
    on_change: Option<ChangeHandler>,
    provided_focus_handle: Option<FocusHandle>,
    availability: Availability,
}

impl ToggleButton {
    /// `pressed` を表示するボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`ToggleButton::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, pressed: bool) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            pressed,
            on_change: None,
            provided_focus_handle: None,
            availability: Availability::Enabled,
        }
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態を反転した値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ToggleButton::new`] に渡すこと。
    /// 設定しない場合、ボタンは押されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// ボタンを無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、支援技術には disabled として公開される。
    /// [`ToggleGroup`] の項目では、矢印キーでの到達も同じように決まる。
    /// 有効か無効かだけを切り替える場合は [`ToggleButton::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`ToggleButton::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのボタンのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。
    /// 単体では `focus_handle` は Tab で到達できるように設定され、[`ToggleGroup`] の項目ではグループが決める。
    /// 指定しない場合はボタンかグループが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    // TODO: フォーカス・無効状態・クリックの扱いは [`Button`](crate::button::Button) と同じ処理を複製している。1 つにまとめたい
    fn into_element_with(
        self,
        focus_handle: FocusHandle,
        tab_stop: bool,
        window: &Window,
    ) -> impl IntoElement + use<> {
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_change = self.on_change.filter(|_| !disabled);
        let next = !self.pressed;
        self.inner
            .role(accesskit::Role::Button)
            .aria_toggled(if self.pressed {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
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
                this.track_focus(&focus_handle.clone().tab_stop(tab_stop))
            })
            .when_some(on_change, |this, on_change| {
                this.on_click({
                    let on_change = on_change.clone();
                    move |_, window, cx| on_change(next, window, cx)
                })
                // gpui は on_a11y_action が無いと、支援技術のクリックをノードの中央へのマウス操作として届け、
                // 上に重なった要素が受け取る。マウスのクリックと同じくフォーカスも移す
                .on_a11y_action(accesskit::Action::Click, {
                    let focus_handle = focus_handle.clone();
                    move |_, window, cx| {
                        window.focus(&focus_handle, cx);
                        on_change(next, window, cx);
                    }
                })
            })
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
            })
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| {
                    cx.focus_handle().tab_index(0).tab_stop(true)
                })
                .read(cx)
                .clone()
        });
        self.into_element_with(focus_handle, true, window)
    }
}

impl Styled for ToggleButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for ToggleButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for ToggleButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for ToggleButton {}

/// [`ToggleButton`] を横に並べ、左右の矢印キーで項目の間を移るグループ
///
/// 項目は [`ToggleGroup::item`] で渡した順に並ぶ。各項目の押下状態は項目ごとに独立していて、グループは状態を持たず、
/// 押されている数も制限しない。数を制限する場合は、呼び出し側が各項目の `on_change` で判断する。
///
/// - Tab で到達できる項目は 1 つだけで、最初の有効な項目にする。項目にフォーカスがある間は、その項目だけが Tab で到達できる
/// - 左右の矢印キーで前後の項目へフォーカスを移す。端では反対の端に戻る。押下状態は変えない。
///   Tab で到達できない無効な項目は飛ばす。移れる項目がほかに無い場合は、キー入力を祖先に渡す
/// - フォーカス中の項目を消すと、キー入力はどの項目にも届かなくなる。グループが内部で持つフォーカスは捨てられる。
///   [`ToggleButton::track_focus`] で渡したフォーカスはフォーカスがあるとみなされたまま残り、同じフォーカスで描き直すと戻る。
///   隣の項目へ移すには、消す項目と移す先の項目に [`ToggleButton::track_focus`] でフォーカスを渡し、項目を消す側で移すこと
///
/// 矢印キーの割り当ては [`crate::init`] が key context を付けて登録し、グループの `key_context` はその値で上書きされる。
/// 最も深い context でも成り立つ割り当て (context の指定なし、`!Foo` など) は、[`crate::init`] より後に登録したか、
/// 項目に `key_context` を付けた場合に、同じキーのグループの割り当てより優先される。
/// その action を祖先か `App::on_action` のリスナーが処理すると、矢印キーで項目の間を移れない。
/// 避けるには、その割り当てにグループの祖先の context を付ける。項目に `key_context` を付けない場合は、
/// [`crate::init`] をその割り当てより後に呼んでもよい
///
/// 支援技術には role `toolbar` として公開される。グループの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    inner: Stateful<Div>,
    items: Vec<ToggleButton>,
}

impl ToggleGroup {
    /// 項目のないグループを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、[`ToggleButton::track_focus`] を指定していない項目のフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            items: Vec::new(),
        }
    }

    /// 末尾に項目を加える。項目の `id` はグループの中で一意にすること
    pub fn item(mut self, item: ToggleButton) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える。項目の `id` はグループの中で一意にすること
    pub fn items(mut self, items: impl IntoIterator<Item = ToggleButton>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for ToggleGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handles = roving::focus_handles(
            &self.id,
            self.items
                .iter()
                .map(|item| (item.id.clone(), item.provided_focus_handle.clone())),
            window,
            cx,
        );
        let roving_items: Rc<[RovingItem]> = self
            .items
            .iter()
            .zip(&focus_handles)
            .map(|(item, focus_handle)| RovingItem {
                // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
                focusable: item.availability.is_focusable() || focus_handle.is_focused(window),
                disabled: item.availability.is_disabled(),
                focus_handle: focus_handle.clone(),
            })
            .collect();
        let tab_stop = roving::tab_stop(&roving_items, None, window);
        let move_focus = move |forward: bool, window: &mut Window, cx: &mut App| {
            match roving::neighbor(&roving_items, forward, window) {
                Some(ix) => window.focus(&roving_items[ix].focus_handle, cx),
                // 移る先が無いキー入力は、祖先や他のキー割り当てに渡す
                None => cx.propagate(),
            }
        };
        let move_focus = Rc::new(move_focus);

        let items = self.items.into_iter().zip(focus_handles).enumerate().map(
            |(ix, (item, focus_handle))| {
                item.into_element_with(focus_handle, tab_stop == Some(ix), window)
            },
        );

        self.inner
            .role(accesskit::Role::Toolbar)
            .key_context(roving::TOGGLE_GROUP_CONTEXT)
            .on_action({
                let move_focus = move_focus.clone();
                move |_: &NextItem, window, cx| move_focus(true, window, cx)
            })
            .on_action(move |_: &PreviousItem, window, cx| move_focus(false, window, cx))
            .children(items)
    }
}

impl Styled for ToggleGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for ToggleGroup {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for ToggleGroup {}

/// APG の Button パターン (content/patterns/button/button-pattern.html) の Toggle Button と、
/// Toolbar パターン (content/patterns/toolbar/toolbar-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, Element, Entity, KeyBinding, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers,
        Render, TestAppContext, VisualTestContext, actions, point, px,
    };

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    actions!(toggle_test, [OuterRight]);

    const OUTER_CONTEXT: &str = "ToggleTestOuter";

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    fn a11y_node(element: impl IntoElement) -> accesskit::Node {
        let element = element.into_element();
        let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
        element.write_a11y_info(&mut node);
        node
    }

    struct ButtonView {
        pressed: bool,
        availability: Availability,
        focus_handle: FocusHandle,
        changes: Rc<RefCell<Vec<bool>>>,
        node: Option<accesskit::Node>,
    }

    impl Render for ButtonView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let element = ToggleButton::new("mute", self.pressed)
                .on_change(move |pressed, _, _| changes.borrow_mut().push(pressed))
                .track_focus(&self.focus_handle)
                .availability(self.availability)
                .aria_label("Mute")
                .size(px(50.))
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            self.node = Some(node);
            div().size_full().child(element)
        }
    }

    fn setup_button(
        cx: &mut TestAppContext,
        pressed: bool,
        availability: Availability,
    ) -> (Entity<ButtonView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ButtonView {
            pressed,
            availability,
            focus_handle: cx.focus_handle(),
            changes: Rc::default(),
            node: None,
        })
    }

    fn button_changes(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> Vec<bool> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn button_node(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画済みであること")
    }

    #[gpui::test]
    fn role_button_と押下状態で公開される(cx: &mut TestAppContext) {
        for (pressed, toggled) in [
            (false, accesskit::Toggled::False),
            (true, accesskit::Toggled::True),
        ] {
            let (view, cx) = setup_button(cx, pressed, Availability::Enabled);
            let node = button_node(&view, cx);
            assert_eq!(node.role(), accesskit::Role::Button);
            assert_eq!(node.label(), Some("Mute"));
            assert_eq!(node.toggled(), Some(toggled), "{pressed}");
            assert!(node.supports_action(accesskit::Action::Click));
        }
    }

    #[gpui::test]
    fn enter_と_space_とクリックで反転した値が渡される(cx: &mut TestAppContext) {
        let (view, cx) = setup_button(cx, false, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "enter");
        press_key(cx, "space");
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        assert_eq!(
            button_changes(&view, cx),
            vec![true, true, true],
            "渡された状態が変わらない限り同じ値を渡すこと"
        );
        assert_eq!(
            button_node(&view, cx).toggled(),
            Some(accesskit::Toggled::False),
            "自身では状態を変えないこと"
        );
    }

    #[gpui::test]
    fn 無効なボタンは押せず_disabled_として公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup_button(cx, false, Availability::Disabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
        assert!(button_changes(&view, cx).is_empty());
        let node = button_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効でも到達できるボタンは_tab_で到達できるが押せない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_button(cx, false, Availability::DisabledFocusable);
        cx.simulate_keystrokes("tab");
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        assert!(cx.update(|window, _| focus_handle.is_focused(window)));
        press_key(cx, "enter");
        assert!(button_changes(&view, cx).is_empty());
    }

    struct GroupView {
        pressed: [bool; 3],
        availability: [Availability; 3],
        /// `false` なら項目に `handles` を渡さず、グループが内部で持つフォーカスを使う
        with_handles: bool,
        handles: [FocusHandle; 3],
        before: FocusHandle,
        after: FocusHandle,
        changes: Rc<RefCell<Vec<(usize, bool)>>>,
        /// グループの外で `OuterRight` を処理した回数
        outer_rights: Rc<RefCell<usize>>,
        group_node: Option<accesskit::Node>,
    }

    impl Render for GroupView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let group = ToggleGroup::new("formatting")
                .aria_label("Text formatting")
                .flex()
                .items((0..3).map(|ix| {
                    let changes = self.changes.clone();
                    ToggleButton::new(ix, self.pressed[ix])
                        .on_change(move |pressed, _, _| changes.borrow_mut().push((ix, pressed)))
                        .when(self.with_handles, |this| {
                            this.track_focus(&self.handles[ix])
                        })
                        .availability(self.availability[ix])
                        .size(px(20.))
                }))
                .render(window, cx)
                .into_element();
            let mut node = accesskit::Node::new(group.a11y_role().expect("role を持つこと"));
            group.write_a11y_info(&mut node);
            self.group_node = Some(node);
            let outer_rights = self.outer_rights.clone();
            div()
                .key_context(OUTER_CONTEXT)
                .on_action(move |_: &OuterRight, _, _| *outer_rights.borrow_mut() += 1)
                .flex()
                .flex_col()
                .child(Button::new("before").track_focus(&self.before).h(px(20.)))
                .child(group)
                .child(Button::new("after").track_focus(&self.after).h(px(20.)))
        }
    }

    fn setup_group(
        cx: &mut TestAppContext,
        availability: [Availability; 3],
    ) -> (Entity<GroupView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| GroupView {
            pressed: [false, true, false],
            availability,
            with_handles: true,
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            before: cx.focus_handle(),
            after: cx.focus_handle(),
            changes: Rc::default(),
            outer_rights: Rc::default(),
            group_node: None,
        })
    }

    fn update(
        view: &Entity<GroupView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut GroupView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    /// `ix` 番目の項目をクリックする
    fn click_item(cx: &mut VisualTestContext, ix: usize) {
        cx.simulate_click(point(px(10. + 20. * ix as f32), px(30.)), Modifiers::none());
    }

    const ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある項目の位置。前のボタンは `Some(-1)`、後のボタンは `Some(3)`
    fn focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Option<i32> {
        let (handles, before, after) = view.read_with(cx, |view, _| {
            (
                view.handles.clone(),
                view.before.clone(),
                view.after.clone(),
            )
        });
        cx.update(|window, _| {
            if before.is_focused(window) {
                return Some(-1);
            }
            if after.is_focused(window) {
                return Some(3);
            }
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| ix as i32)
        })
    }

    fn group_changes(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Vec<(usize, bool)> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    #[gpui::test]
    fn グループは_role_toolbar_と名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        let node = view
            .read_with(cx, |view, _| view.group_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::Toolbar);
        assert_eq!(node.label(), Some("Text formatting"));
    }

    #[gpui::test]
    fn グループの項目も押下状態で公開される(cx: &mut TestAppContext) {
        let (_, cx) = setup_group(cx, ENABLED);
        let node = cx.update(|window, cx| {
            a11y_node(ToggleButton::new("bold", true).into_element_with(
                cx.focus_handle(),
                false,
                window,
            ))
        });
        assert_eq!(node.role(), accesskit::Role::Button);
        assert_eq!(node.toggled(), Some(accesskit::Toggled::True));
    }

    #[gpui::test]
    fn tab_で最初の項目に入り次の_tab_で外に出る(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(0));
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(3));
    }

    #[gpui::test]
    fn 左右の矢印キーで移り押下状態は変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab right");
        assert_eq!(focused(&view, cx), Some(1));
        cx.simulate_keystrokes("right right");
        assert_eq!(focused(&view, cx), Some(0), "末尾の次は先頭に戻ること");
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some(2), "先頭の前は末尾に戻ること");
        assert!(group_changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 上下の矢印キーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab down up");
        assert_eq!(focused(&view, cx), Some(0));
    }

    #[gpui::test]
    fn グループの外から戻ると最初の有効な項目に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab right tab");
        assert_eq!(focused(&view, cx), Some(3));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some(0),
            "グループの外に出ると最初の項目に戻ること"
        );
    }

    #[gpui::test]
    fn 項目は_enter_と_space_で個別に押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab");
        press_key(cx, "space");
        cx.simulate_keystrokes("right");
        press_key(cx, "enter");
        assert_eq!(group_changes(&view, cx), vec![(0, true), (1, false)]);
    }

    #[gpui::test]
    fn 無効な項目は飛ばし_無効でも到達できる項目には移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(
            cx,
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::DisabledFocusable,
            ],
        );
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(1), "最初の有効な項目に入ること");
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2));
        press_key(cx, "enter");
        press_key(cx, "space");
        click_item(cx, 2);
        assert!(
            group_changes(&view, cx).is_empty(),
            "無効でも到達できる項目は押せないこと"
        );
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(1), "無効な項目を飛ばすこと");
    }

    #[gpui::test]
    fn すべて無効ならグループに_tab_で入らない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, [Availability::Disabled; 3]);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(3));
    }

    #[gpui::test]
    fn クリックした項目にフォーカスが移る(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ENABLED);
        click_item(cx, 2);
        assert_eq!(focused(&view, cx), Some(2));
        assert_eq!(group_changes(&view, cx), vec![(2, true)]);
    }

    #[gpui::test]
    fn フォーカス中に無効になった項目はフォーカスを保ち押せない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(cx, ENABLED);
        cx.simulate_keystrokes("tab tab right");
        update(&view, cx, |view| {
            view.availability[1] = Availability::Disabled
        });
        assert_eq!(focused(&view, cx), Some(1));
        press_key(cx, "enter");
        press_key(cx, "space");
        click_item(cx, 1);
        assert!(group_changes(&view, cx).is_empty());
        cx.simulate_keystrokes("right");
        assert_eq!(
            focused(&view, cx),
            Some(2),
            "無効な項目からも矢印キーで移れること"
        );
    }

    #[gpui::test]
    fn 移れる項目がほかに無ければ矢印キーを祖先に渡す(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| cx.bind_keys([KeyBinding::new("right", OuterRight, Some(OUTER_CONTEXT))]));
        let (view, cx) = setup_group(
            cx,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Disabled,
            ],
        );
        cx.simulate_keystrokes("tab tab right");
        assert_eq!(focused(&view, cx), Some(0));
        let outer_rights = view.read_with(cx, |view, _| view.outer_rights.clone());
        assert_eq!(*outer_rights.borrow(), 1);

        update(&view, cx, |view| view.availability = ENABLED);
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(1));
        assert_eq!(*outer_rights.borrow(), 1, "移れる間は渡さないこと");
    }

    #[gpui::test]
    fn グループが持つフォーカスは描画をまたいで保たれる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(cx, ENABLED);
        update(&view, cx, |view| view.with_handles = false);
        cx.simulate_keystrokes("tab tab");
        cx.simulate_keystrokes("right");
        press_key(cx, "space");
        assert_eq!(group_changes(&view, cx), vec![(1, false)]);
    }
}
