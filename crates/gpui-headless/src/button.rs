//! 見た目を持たないボタン

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, Entity, FocusHandle, InteractiveElement,
    Interactivity, IntoElement, MouseButton, MouseDownEvent, ParentElement, RenderOnce, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

use crate::{Availability, accessibility::A11yInfoExt};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 見た目を持たないボタン
///
/// 支援技術には role `button` として公開される。アクセシブルな名前は、子の内容か `aria_label` で与えること。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作で [`Button::on_click`] のハンドラを呼ぶ。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    inner: Stateful<Div>,
    on_click: Option<ClickHandler>,
    /// [`Button::track_focus`] で渡されたもの、または内部で最初に引いたもの
    focus_handle: Option<FocusHandle>,
    availability: Availability,
    /// フォーカスを得てから最後に押されたのがキーボードかどうか。[`Button::is_focus_visible`] を呼んだ場合だけ持つ
    keyboard_focus: Option<Entity<bool>>,
}

impl Button {
    /// ボタンを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Button::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            on_click: None,
            focus_handle: None,
            availability: Availability::Enabled,
            keyboard_focus: None,
        }
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 設定しない場合、ボタンは押されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// ボタンを無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Button::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Button::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのボタンのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合はボタンが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// フォーカスがあることを表示すべきかどうか
    ///
    /// ボタンがフォーカスを持ち、最後に押されたのがマウスボタンではなくキーだった場合に `true` を返す。
    /// マウスの移動では変わらないため、Tab で移った後にマウスを動かしても `true` のまま。
    /// クリックでフォーカスを得た場合は、キーが押されるまで `false` になる。
    /// 描画のたびに、このボタンを描画する前に呼ぶこと
    pub fn is_focus_visible(&mut self, window: &mut Window, cx: &mut App) -> bool {
        let focused = self.focus_handle(window, cx).is_focused(window);
        let keyboard_focus = self
            .keyboard_focus
            .get_or_insert_with(|| window.use_keyed_state(self.id.clone(), cx, |_, _| false))
            .clone();
        // gpui の last_input_was_keyboard はマウスの移動でも false になるため、
        // キーで true になったことを覚えておき、マウスボタンの押下 (render で登録するリスナー) で戻す
        let visible = focused && (window.last_input_was_keyboard() || *keyboard_focus.read(cx));
        keyboard_focus.update(cx, |keyboard_focus, _| *keyboard_focus = visible);
        visible
    }

    fn focus_handle(&mut self, window: &mut Window, cx: &mut App) -> FocusHandle {
        // 部品は型ごとの id の下で描画されるため、is_focus_visible と render で keyed state を引くと
        // 別のハンドルになる。最初に引いたものを使い続ける
        self.focus_handle
            .get_or_insert_with(|| {
                window
                    .use_keyed_state(self.id.clone(), cx, |_, cx| {
                        cx.focus_handle().tab_index(0).tab_stop(true)
                    })
                    .read(cx)
                    .clone()
            })
            .clone()
    }
}

impl RenderOnce for Button {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.focus_handle(window, cx);
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_click = self.on_click.filter(|_| !disabled);
        self.inner
            .role(accesskit::Role::Button)
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
                this.track_focus(&focus_handle.tab_stop(true))
            })
            .when_some(on_click, |this, on_click| this.on_click(on_click))
            .when_some(self.keyboard_focus, |this, keyboard_focus| {
                let clear = move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| {
                    keyboard_focus.update(cx, |keyboard_focus, cx| {
                        if *keyboard_focus {
                            *keyboard_focus = false;
                            cx.notify();
                        }
                    });
                };
                this.capture_any_mouse_down(clear.clone())
                    .on_mouse_down_out(clear)
            })
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
            })
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Button {}

/// APG の Button パターン (content/patterns/button/button-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::test_window::add_test_window;

    struct ButtonView {
        has_handler: bool,
        availability: Availability,
        focus_handle: FocusHandle,
        clicks: Rc<Cell<usize>>,
        parent_clicks: Rc<Cell<usize>>,
        node: Option<accesskit::Node>,
        focus_visible: bool,
    }

    impl Render for ButtonView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            let parent_clicks = self.parent_clicks.clone();
            let mut button = Button::new("button")
                .when(self.has_handler, |this| {
                    this.on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                })
                .track_focus(&self.focus_handle)
                .availability(self.availability)
                .aria_label("Print Page")
                .size(px(50.));
            self.focus_visible = button.is_focus_visible(window, cx);
            let element = button.render(window, cx).into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            self.node = Some(node);
            div()
                .id("parent")
                .size_full()
                .on_click(move |_, _, _| parent_clicks.set(parent_clicks.get() + 1))
                .child(element)
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        availability: Availability,
    ) -> (Entity<ButtonView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ButtonView {
            has_handler: true,
            availability,
            focus_handle: cx.focus_handle(),
            clicks: Rc::default(),
            parent_clicks: Rc::default(),
            node: None,
            focus_visible: false,
        })
    }

    fn clicks(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> usize {
        view.read_with(cx, |view, _| view.clicks.get())
    }

    fn parent_clicks(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> usize {
        view.read_with(cx, |view, _| view.parent_clicks.get())
    }

    fn a11y_node(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画済みであること")
    }

    fn is_focused(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> bool {
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, _| focus_handle.is_focused(window))
    }

    fn is_focus_visible(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> bool {
        view.read_with(cx, |view, _| view.focus_visible)
    }

    fn set_availability(
        view: &Entity<ButtonView>,
        availability: Availability,
        cx: &mut VisualTestContext,
    ) {
        view.update(cx, |view, cx| {
            view.availability = availability;
            cx.notify();
        });
        cx.run_until_parked();
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

    fn click(cx: &mut VisualTestContext) {
        cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    }

    #[gpui::test]
    fn role_button_として公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        assert_eq!(a11y_node(&view, cx).role(), accesskit::Role::Button);
    }

    #[gpui::test]
    fn aria_label_がアクセシブルな名前になる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        assert_eq!(a11y_node(&view, cx).label(), Some("Print Page"));
    }

    #[gpui::test]
    fn tab_で到達できる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        assert!(is_focused(&view, cx), "Tab でボタンにフォーカスが移ること");
    }

    #[gpui::test]
    fn enter_で押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "enter");
        assert_eq!(clicks(&view, cx), 1);
    }

    #[gpui::test]
    fn space_で押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        assert_eq!(clicks(&view, cx), 1);
    }

    #[gpui::test]
    fn マウスのクリックで押せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        click(cx);
        assert_eq!(clicks(&view, cx), 1);
    }

    #[gpui::test]
    fn 押した後もフォーカスはボタンに残る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "enter");
        press_key(cx, "space");
        assert_eq!(clicks(&view, cx), 2);
        assert!(is_focused(&view, cx), "押した後もフォーカスが残ること");
    }

    #[gpui::test]
    fn 有効なボタンは支援技術にクリックとフォーカスの操作を公開する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Enabled);
        let node = a11y_node(&view, cx);
        assert!(!node.is_disabled());
        assert!(node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn ハンドラを持たないボタンはクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Enabled);
        view.update(cx, |view, cx| {
            view.has_handler = false;
            cx.notify();
        });
        cx.run_until_parked();
        let node = a11y_node(&view, cx);
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効なボタンは_tab_で到達できず押せない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Disabled);
        cx.simulate_keystrokes("tab");
        assert!(!is_focused(&view, cx), "Tab でボタンに到達しないこと");
        click(cx);
        assert_eq!(clicks(&view, cx), 0);
        assert_eq!(
            parent_clicks(&view, cx),
            0,
            "親にもクリックが伝わらないこと"
        );
        assert!(!is_focused(&view, cx), "クリックでもフォーカスされないこと");
    }

    #[gpui::test]
    fn 無効なボタンは_disabled_として公開され操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Disabled);
        let node = a11y_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn フォーカス中に無効になってもフォーカスを保ち押せない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        set_availability(&view, Availability::Disabled, cx);
        assert!(is_focused(&view, cx), "無効になってもフォーカスが残ること");

        press_key(cx, "enter");
        press_key(cx, "space");
        click(cx);
        assert_eq!(clicks(&view, cx), 0);
        assert_eq!(
            parent_clicks(&view, cx),
            0,
            "親にもクリックが伝わらないこと"
        );

        let node = a11y_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
    }

    #[gpui::test]
    fn 無効でも到達できるボタンは_tab_で到達できるが押せない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::DisabledFocusable);
        cx.simulate_keystrokes("tab");
        assert!(is_focused(&view, cx), "Tab でボタンに到達すること");

        press_key(cx, "enter");
        press_key(cx, "space");
        click(cx);
        assert_eq!(clicks(&view, cx), 0);
        assert_eq!(
            parent_clicks(&view, cx),
            0,
            "親にもクリックが伝わらないこと"
        );
    }

    #[gpui::test]
    fn 無効でも到達できるボタンはクリックでフォーカスされる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::DisabledFocusable);
        click(cx);
        assert!(
            is_focused(&view, cx),
            "クリックでボタンにフォーカスが移ること"
        );
        assert_eq!(parent_clicks(&view, cx), 0, "親にクリックが伝わらないこと");
    }

    #[gpui::test]
    fn 無効でも到達できるボタンは_disabled_として公開されフォーカス操作だけを公開する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::DisabledFocusable);
        let node = a11y_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    /// 内部のフォーカスを使うボタンを、別の部品の中で包んで描画する
    #[derive(IntoElement)]
    struct WrappedButton {
        focus_visible: Rc<Cell<bool>>,
    }

    impl RenderOnce for WrappedButton {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let mut button = Button::new("button").size(px(50.));
            self.focus_visible.set(button.is_focus_visible(window, cx));
            button
        }
    }

    struct WrapperView {
        focus_visible: Rc<Cell<bool>>,
    }

    impl Render for WrapperView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            WrappedButton {
                focus_visible: self.focus_visible.clone(),
            }
        }
    }

    #[gpui::test]
    fn 部品の中で内部のフォーカスを使ってもフォーカスの表示を判定できる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_test_window(cx, |_, _| WrapperView {
            focus_visible: Rc::default(),
        });
        cx.simulate_keystrokes("tab");
        assert!(
            view.read_with(cx, |view, _| view.focus_visible.get()),
            "Tab で移ったフォーカスを表示すること"
        );
    }

    #[gpui::test]
    fn 部品の中で内部のフォーカスを使ってもマウスボタンを押すとフォーカスの表示が消える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_test_window(cx, |_, _| WrapperView {
            focus_visible: Rc::default(),
        });
        cx.simulate_keystrokes("tab");
        cx.simulate_mouse_down(
            point(px(100.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert!(
            !view.read_with(cx, |view, _| view.focus_visible.get()),
            "マウスボタンを押すとフォーカスを表示しないこと"
        );
    }

    #[gpui::test]
    fn tab_で移ったフォーカスはマウスを動かしても表示し続ける(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        assert!(
            is_focus_visible(&view, cx),
            "Tab で移ったフォーカスを表示すること"
        );
        cx.simulate_mouse_move(point(px(30.), px(30.)), None, Modifiers::none());
        assert!(
            is_focus_visible(&view, cx),
            "マウスを動かしてもフォーカスを表示し続けること"
        );
    }

    #[gpui::test]
    fn マウスボタンを押すとフォーカスの表示が消える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        cx.simulate_mouse_move(point(px(100.), px(100.)), None, Modifiers::none());
        cx.simulate_mouse_down(
            point(px(100.), px(100.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert!(
            is_focused(&view, cx),
            "ボタンの外を押してもフォーカスは残ること"
        );
        assert!(
            !is_focus_visible(&view, cx),
            "マウスボタンを押すとフォーカスを表示しないこと"
        );
    }

    #[gpui::test]
    fn クリックで得たフォーカスはキーを押すまで表示しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Availability::Enabled);
        click(cx);
        assert!(
            is_focused(&view, cx),
            "クリックでボタンにフォーカスが移ること"
        );
        assert!(
            !is_focus_visible(&view, cx),
            "クリックで得たフォーカスは表示しないこと"
        );
        press_key(cx, "a");
        assert!(
            is_focus_visible(&view, cx),
            "キーを押すとフォーカスを表示すること"
        );
    }
}
