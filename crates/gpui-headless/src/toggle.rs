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
    roving::{self, GroupChild, RovingItem},
};

type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// 押下状態を持つ、見た目を持たないボタン
///
/// 状態 (押されているかどうか) は呼び出し側が持ち、[`ToggleButton::new`] に毎回渡す。ボタン自身は状態を変えない。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作で
/// [`ToggleButton::on_change`] のハンドラを呼ぶ。
/// 単体で置くと Tab で到達できる。[`ToggleGroup::item`] に渡すと、Tab で到達できるかはグループが決める
///
/// 支援技術には role `button` と、押されているかどうかの状態として公開される。
/// アクセシブルな名前は、子の内容か `aria_label` で与えること。状態によって名前を変えないこと。
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
    /// `pressed` の状態を表示するボタンを作る
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

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態を反転した値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ToggleButton::new`] に渡すこと。
    /// 設定しない場合、ボタンは操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// ボタンを無効にするか、無効でも Tab や矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なボタンは操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`ToggleButton::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// Tab や矢印キーでも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも到達できるようにしたい場合は [`ToggleButton::availability`] を使う
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
    /// 指定しない場合はボタンが内部でフォーカスを持つ
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

    // TODO: フォーカス・無効状態・クリックの扱いは [`Button`](crate::button::Button) と同じ処理を複製している。1 つにまとめたい
    /// `focus_handle` をフォーカスとして使い、`tab_stop` なら Tab で到達できる要素にする
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
                this.track_focus(&focus_handle.tab_stop(tab_stop))
            })
            .when_some(on_change, |this, on_change| {
                this.on_click(move |_, window, cx| on_change(&next, window, cx))
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
        let focus_handle = self.focus_handle(window, cx);
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

/// [`ToggleButton`] を並べる、見た目を持たないグループ
///
/// 押下状態は項目ごとに独立し、グループは状態を持たない。押されている数も制限しない。
/// 項目は [`ToggleGroup::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。
/// 項目は横に並べること。縦に並べても上下の矢印キーでは移れず、支援技術にも縦の向きを公開しない
///
/// - Tab で到達できる項目は 1 つだけで、最初のフォーカスできる項目にする。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。フォーカスできる項目が無ければ Tab で入らない
/// - 修飾キーなしの左右の矢印キーで前後の項目へフォーカスを移す。押下状態は変えない。端では反対の端に戻る。
///   [`Availability::Disabled`] の項目は飛ばす。移れる項目がほかに無い場合は、キー入力を祖先に渡す
///
/// 支援技術には role `toolbar` として公開される。グループの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    inner: Stateful<Div>,
    children: Vec<GroupChild<ToggleButton>>,
}

impl ToggleGroup {
    /// 項目を持たないグループを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            children: Vec::new(),
        }
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ToggleButton) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ToggleButton>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl RenderOnce for ToggleGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items: Rc<[RovingItem]> = self
            .children
            .iter()
            .filter_map(|child| match child {
                GroupChild::Item(button) => Some(RovingItem {
                    // 項目の id はグループの中でだけ一意なので、グループの id で分ける
                    focus_handle: window.with_element_namespace(self.id.clone(), |window| {
                        button.focus_handle(window, cx)
                    }),
                    availability: button.availability,
                }),
                GroupChild::Other(_) => None,
            })
            .collect();
        let tab_stop = roving::tab_stop(&items, None, window);

        let mut ix = 0;
        let elements: Vec<AnyElement> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(button) => {
                    let element = button.into_element_with(
                        items[ix].focus_handle.clone(),
                        tab_stop == Some(ix),
                        window,
                    );
                    ix += 1;
                    element.into_any_element()
                }
                GroupChild::Other(element) => element,
            })
            .collect();

        self.inner
            .role(accesskit::Role::Toolbar)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let forward = match event.keystroke.key.as_str() {
                    "right" => true,
                    "left" => false,
                    _ => return,
                };
                let Some(ix) = roving::neighbor(&items, forward, window) else {
                    return;
                };
                window.focus(&items[ix].focus_handle, cx);
                cx.stop_propagation();
            })
            .children(elements)
    }
}

impl Styled for ToggleGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for ToggleGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl InteractiveElement for ToggleGroup {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for ToggleGroup {}

/// APG の Button パターン (content/patterns/button/button-pattern.html) の Toggle Button と、
/// Keyboard Interface の roving tabindex (content/practices/keyboard-interface/keyboard-interface-practice.html)、
/// Toolbar パターン (content/patterns/toolbar/toolbar-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    struct ButtonView {
        pressed: bool,
        availability: Availability,
        has_handler: bool,
        focus_handle: FocusHandle,
        changes: Rc<RefCell<Vec<bool>>>,
        node: Option<accesskit::Node>,
    }

    impl Render for ButtonView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let element = ToggleButton::new("mute", self.pressed)
                .when(self.has_handler, |this| {
                    this.on_change(move |pressed, _, _| changes.borrow_mut().push(*pressed))
                })
                .availability(self.availability)
                .track_focus(&self.focus_handle)
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
            has_handler: true,
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

    fn is_button_focused(view: &Entity<ButtonView>, cx: &mut VisualTestContext) -> bool {
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, _| focus_handle.is_focused(window))
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

    fn click(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_click(point(px(x), px(10.)), Modifiers::none());
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
            assert_eq!(node.toggled(), Some(toggled));
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(node.supports_action(accesskit::Action::Focus));
        }
    }

    #[gpui::test]
    fn enter_と_space_とクリックで反転した値が渡される(cx: &mut TestAppContext) {
        for (pressed, next) in [(false, true), (true, false)] {
            let (view, cx) = setup_button(cx, pressed, Availability::Enabled);
            cx.simulate_keystrokes("tab");
            assert!(
                is_button_focused(&view, cx),
                "単体では Tab で到達できること"
            );
            press_key(cx, "enter");
            press_key(cx, "space");
            click(cx, 10.);
            assert_eq!(
                button_changes(&view, cx),
                vec![next; 3],
                "渡された状態が変わらない限り同じ値を渡すこと"
            );
            assert_eq!(
                button_node(&view, cx).toggled(),
                Some(if pressed {
                    accesskit::Toggled::True
                } else {
                    accesskit::Toggled::False
                }),
                "自身では状態を変えないこと"
            );
        }
    }

    #[gpui::test]
    fn ハンドラを持たないボタンはクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_button(cx, false, Availability::Enabled);
        view.update(cx, |view, cx| {
            view.has_handler = false;
            cx.notify();
        });
        cx.run_until_parked();
        let node = button_node(&view, cx);
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効なボタンは操作できず_disabled_として公開される(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_button(cx, false, Availability::Disabled);
        cx.simulate_keystrokes("tab");
        assert!(!is_button_focused(&view, cx), "Tab で到達しないこと");
        click(cx, 10.);
        assert!(
            !is_button_focused(&view, cx),
            "クリックでもフォーカスされないこと"
        );
        assert!(button_changes(&view, cx).is_empty());
        let node = button_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効でも到達できるボタンは_tab_で到達できるが操作できない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_button(cx, false, Availability::DisabledFocusable);
        cx.simulate_keystrokes("tab");
        assert!(is_button_focused(&view, cx));
        press_key(cx, "enter");
        press_key(cx, "space");
        click(cx, 10.);
        assert!(button_changes(&view, cx).is_empty());
        let node = button_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    const NAMES: [&str; 3] = ["bold", "italic", "underline"];

    struct GroupView {
        pressed: [bool; 3],
        availability: [Availability; 3],
        /// 最初の項目の後に、項目でない要素を置く
        divider: bool,
        handles: [FocusHandle; 3],
        after: FocusHandle,
        changes: Rc<RefCell<Vec<(&'static str, bool)>>>,
        group_node: Option<accesskit::Node>,
    }

    impl Render for GroupView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut buttons = NAMES.iter().enumerate().map(|(ix, name)| {
                ToggleButton::new(*name, self.pressed[ix])
                    .on_change(cx.listener(move |this, pressed: &bool, _, cx| {
                        this.changes.borrow_mut().push((NAMES[ix], *pressed));
                        this.pressed[ix] = *pressed;
                        cx.notify();
                    }))
                    .availability(self.availability[ix])
                    .track_focus(&self.handles[ix])
                    .aria_label(*name)
                    .w(px(20.))
                    .h(px(20.))
            });
            let first = buttons.next();
            let rest: Vec<_> = buttons.collect();
            let group = ToggleGroup::new("format")
                .aria_label("Format")
                .flex()
                .items(first)
                .when(self.divider, |this| this.child(div().w(px(20.))))
                .items(rest)
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(group.a11y_role().expect("role を持つこと"));
            group.write_a11y_info(&mut node);
            self.group_node = Some(node);
            div()
                .size_full()
                .child(group)
                .child(Button::new("after").track_focus(&self.after).h(px(20.)))
        }
    }

    fn setup_group(
        cx: &mut TestAppContext,
        availability: [Availability; 3],
    ) -> (Entity<GroupView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| GroupView {
            pressed: [false; 3],
            availability,
            divider: false,
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            after: cx.focus_handle(),
            changes: Rc::default(),
            group_node: None,
        })
    }

    const ALL_ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある項目の名前
    fn focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| {
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| NAMES[ix])
        })
    }

    fn is_after_focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> bool {
        let after = view.read_with(cx, |view, _| view.after.clone());
        cx.update(|window, _| after.is_focused(window))
    }

    fn changes(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Vec<(&'static str, bool)> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    #[gpui::test]
    fn グループは_role_toolbar_と名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        let node = view
            .read_with(cx, |view, _| view.group_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::Toolbar);
        assert_eq!(node.label(), Some("Format"));
    }

    #[gpui::test]
    fn tab_で最初のフォーカスできる項目に入り次の_tab_で外へ出る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(
            cx,
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("italic"));
        cx.simulate_keystrokes("tab");
        assert!(
            is_after_focused(&view, cx),
            "グループの中の Tab stop は 1 つだけであること"
        );
    }

    #[gpui::test]
    fn すべての項目が無効なら_tab_で入らない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, [Availability::Disabled; 3]);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None);
        assert!(is_after_focused(&view, cx));
    }

    #[gpui::test]
    fn 左右の矢印キーで移り押下状態は変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("italic"));
        cx.simulate_keystrokes("right right");
        assert_eq!(focused(&view, cx), Some("bold"), "末尾の次は先頭に戻ること");
        cx.simulate_keystrokes("left");
        assert_eq!(
            focused(&view, cx),
            Some("underline"),
            "先頭の前は末尾に戻ること"
        );
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 上下と修飾キー付きの矢印キーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab down up shift-right ctrl-right");
        assert_eq!(focused(&view, cx), Some("bold"));
    }

    #[gpui::test]
    fn 矢印キーの移動は無効な項目を飛ばし無効でも到達できる項目には移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(
            cx,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::DisabledFocusable,
            ],
        );
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("underline"));
        press_key(cx, "space");
        assert!(
            changes(&view, cx).is_empty(),
            "無効な項目は操作できないこと"
        );
    }

    #[gpui::test]
    fn 移った先の項目から_tab_でグループの外へ出て同じ項目に戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab right tab");
        assert!(is_after_focused(&view, cx));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some("bold"),
            "フォーカスが外れた後は最初のフォーカスできる項目から入ること"
        );
    }

    #[gpui::test]
    fn フォーカス中に無効になってもフォーカスを保つ(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        view.update(cx, |view, cx| {
            view.availability[0] = Availability::Disabled;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(focused(&view, cx), Some("bold"));
        press_key(cx, "space");
        assert!(
            changes(&view, cx).is_empty(),
            "無効な項目は操作できないこと"
        );

        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some("italic"));
        cx.simulate_keystrokes("left");
        assert_eq!(
            focused(&view, cx),
            Some("underline"),
            "フォーカスが外れた後は無効な項目を飛ばすこと"
        );
    }

    #[gpui::test]
    fn 項目は独立に押下状態を切り替える(cx: &mut TestAppContext) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        cx.simulate_keystrokes("right");
        press_key(cx, "enter");
        click(cx, 50.);
        click(cx, 10.);
        assert_eq!(
            changes(&view, cx),
            vec![
                ("bold", true),
                ("italic", true),
                ("underline", true),
                ("bold", false)
            ]
        );
        assert_eq!(
            focused(&view, cx),
            Some("bold"),
            "クリックした項目にフォーカスが移ること"
        );
    }

    /// 項目の id が同じ 2 つのグループを、項目にフォーカスを渡さずに並べる
    struct TwoGroupsView {
        changes: Rc<RefCell<Vec<(&'static str, &'static str)>>>,
    }

    impl Render for TwoGroupsView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().children(["first", "second"].map(|group| {
                ToggleGroup::new(group)
                    .flex()
                    .items(["bold", "italic"].map(|name| {
                        let changes = self.changes.clone();
                        ToggleButton::new(name, false)
                            .on_change(move |_, _, _| changes.borrow_mut().push((group, name)))
                            .size(px(20.))
                    }))
            }))
        }
    }

    #[gpui::test]
    fn 項目の_id_が同じ別のグループとフォーカスを共有しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_test_window(cx, |_, _| TwoGroupsView {
            changes: Rc::default(),
        });
        click(cx, 30.);
        cx.simulate_keystrokes("left");
        press_key(cx, "space");
        assert_eq!(
            *view.read_with(cx, |view, _| view.changes.clone()).borrow(),
            vec![("first", "italic"), ("first", "bold")]
        );
    }

    #[gpui::test]
    fn 項目でない子は加えた順に並び矢印キーの移動に含まれない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup_group(cx, ALL_ENABLED);
        view.update(cx, |view, cx| {
            view.divider = true;
            cx.notify();
        });
        cx.run_until_parked();
        click(cx, 30.);
        assert!(
            changes(&view, cx).is_empty(),
            "最初の項目の次は項目でないこと"
        );
        click(cx, 50.);
        assert_eq!(changes(&view, cx), vec![("italic", true)]);
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some("bold"));
    }
}
