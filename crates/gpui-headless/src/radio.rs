//! 1 つだけを選ぶ、見た目を持たないラジオボタンのグループ

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, MouseButton, ParentElement, RenderOnce,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    roving::{self, GroupChild, RovingItem},
};

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// 見た目を持たないラジオボタンのグループ
///
/// 状態 (選ばれている値) は呼び出し側が持ち、[`RadioGroup::new`] に毎回渡す。グループ自身は状態を変えない。
/// 項目は [`RadioGroup::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初のフォーカスできる項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。フォーカスできる項目が無ければ Tab で入らない
/// - 修飾キーなしの上下左右の矢印キーで前後の項目へフォーカスを移し、移った先の項目を選ぶよう求める。
///   端では反対の端に戻る。[`Availability::Disabled`] の項目は飛ばし、
///   [`Availability::DisabledFocusable`] の項目にはフォーカスだけを移す。
///   移れる項目がほかに無い場合は、キー入力を祖先に渡す
/// - フォーカス中の Space・マウスのクリック・支援技術からのクリック操作で、項目を選ぶよう求める。Enter では求めない
/// - 操作で選択を解除する手段は持たない
///
/// 支援技術には role `radiogroup` として公開され、項目は role `radio` と選ばれているかどうかの状態として公開される。
/// グループの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct RadioGroup<T: PartialEq + 'static> {
    id: ElementId,
    inner: Stateful<Div>,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    children: Vec<GroupChild<Radio<T>>>,
}

impl<T: PartialEq + 'static> RadioGroup<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示するグループを作る。`None` ならどの項目も選ばれていない
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

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目と無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`RadioGroup::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: Radio<T>) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = Radio<T>>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for RadioGroup<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut values = Vec::new();
        let mut items = Vec::new();
        let children: Vec<GroupChild<Stateful<Div>>> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(radio) => {
                    items.push(RovingItem {
                        // 項目の id はグループの中でだけ一意なので、グループの id で分ける
                        focus_handle: window.with_element_namespace(self.id.clone(), |window| {
                            radio.focus_handle(window, cx)
                        }),
                        availability: radio.availability,
                    });
                    values.push(radio.value);
                    GroupChild::Item(radio.inner)
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
                    let element = item_element(
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
            .role(accesskit::Role::RadioGroup)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let forward = match event.keystroke.key.as_str() {
                    "down" | "right" => true,
                    "up" | "left" => false,
                    _ => return,
                };
                let Some(ix) = roving::neighbor(&items, forward, window) else {
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

// TODO: フォーカス・無効状態・クリックの扱いは [`Button`](crate::button::Button) と同じ処理を複製している。1 つにまとめたい
/// `inner` を、グループの中の位置に応じたラジオボタンの項目にする
///
/// `on_select` は項目が操作されたときに呼ぶ。無効な項目では呼ばない
fn item_element<S>(
    inner: Stateful<Div>,
    item: &RovingItem,
    tab_stop: bool,
    checked: bool,
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
        .role(accesskit::Role::RadioButton)
        .aria_toggled(if checked {
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
        .when_some(on_select, |this, on_select| {
            this.on_click(move |event, window, cx| {
                // APG の Radio Group パターンは Enter での操作を定義していない
                if let ClickEvent::Keyboard(KeyboardClickEvent {
                    button: KeyboardButton::Enter,
                    ..
                }) = event
                {
                    return;
                }
                on_select(window, cx);
            })
        })
        .a11y_info(move |node: &mut accesskit::Node| {
            if disabled {
                node.set_disabled();
            }
        })
}

impl<T: PartialEq + 'static> Styled for RadioGroup<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T: PartialEq + 'static> ParentElement for RadioGroup<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl<T: PartialEq + 'static> InteractiveElement for RadioGroup<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T: PartialEq + 'static> StatefulInteractiveElement for RadioGroup<T> {}

/// [`RadioGroup`] の項目
///
/// [`RadioGroup::item`] に渡して使う。項目が表す値が、グループの選ばれている値と等しいときに選ばれた状態になる。
/// アクセシブルな名前は、子の内容か `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct Radio<T> {
    id: ElementId,
    inner: Stateful<Div>,
    value: T,
    availability: Availability,
    provided_focus_handle: Option<FocusHandle>,
}

impl<T> Radio<T> {
    /// `value` を表す項目を作る
    ///
    /// `id` と `value` はグループの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`Radio::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
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

    /// 項目を無効にするか、無効でも Tab や矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な項目は操作されても選ぶよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Radio::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は操作されても選ぶよう求めず、マウスの左ボタンの押下を親に伝えず、
    /// Tab や矢印キーでも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも到達できるようにしたい場合は [`Radio::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab で到達できるかはグループが決める。
    /// 指定しない場合は項目が内部でフォーカスを持つ
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

impl<T> Styled for Radio<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T> ParentElement for Radio<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<T> InteractiveElement for Radio<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T> StatefulInteractiveElement for Radio<T> {}

/// APG の Radio Group パターン (content/patterns/radio/radio-group-pattern.html) と
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
    use crate::button::Button;
    use crate::test_window::add_test_window;

    const VALUES: [&str; 3] = ["apple", "pear", "plum"];

    struct GroupView {
        selected: Option<&'static str>,
        availability: [Availability; 3],
        /// `false` なら求められた値を `selected` に反映しない
        accept: bool,
        /// 最初の項目の後に、項目でない要素を置く
        divider: bool,
        /// 描画する項目
        shown: [bool; 3],
        handles: [FocusHandle; 3],
        after: FocusHandle,
        changes: Rc<RefCell<Vec<&'static str>>>,
        group_node: Option<accesskit::Node>,
    }

    impl Render for GroupView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let shown = self.shown;
            let mut radios =
                VALUES
                    .iter()
                    .enumerate()
                    .filter(|(ix, _)| shown[*ix])
                    .map(|(ix, value)| {
                        Radio::new(*value, *value)
                            .availability(self.availability[ix])
                            .track_focus(&self.handles[ix])
                            .aria_label(*value)
                            .w(px(50.))
                            .h(px(20.))
                    });
            let group = RadioGroup::new("fruit", self.selected)
                .on_change(cx.listener(|this, value: &&'static str, _, cx| {
                    this.changes.borrow_mut().push(*value);
                    if this.accept {
                        this.selected = Some(*value);
                        cx.notify();
                    }
                }))
                .aria_label("Fruit")
                .flex()
                .flex_col()
                .items(radios.next())
                .when(self.divider, |this| this.child(div().h(px(20.))))
                .items(radios)
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

    fn setup<'a>(
        cx: &'a mut TestAppContext,
        selected: Option<&'static str>,
        availability: [Availability; 3],
    ) -> (Entity<GroupView>, &'a mut VisualTestContext) {
        add_test_window(cx, |_, cx| GroupView {
            selected,
            availability,
            accept: true,
            divider: false,
            shown: [true; 3],
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            after: cx.focus_handle(),
            changes: Rc::default(),
            group_node: None,
        })
    }

    const ALL_ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある項目の値
    fn focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| {
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| VALUES[ix])
        })
    }

    fn is_after_focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> bool {
        let after = view.read_with(cx, |view, _| view.after.clone());
        cx.update(|window, _| after.is_focused(window))
    }

    fn changes(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
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

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    /// 上から `row` 番目 (0 始まり) の高さ 20px の行をクリックする
    fn click_row(cx: &mut VisualTestContext, row: usize) {
        cx.simulate_click(
            point(px(10.), px(10. + 20. * row as f32)),
            Modifiers::none(),
        );
    }

    /// 項目だけを描画したときの a11y ノード
    fn item_node(
        cx: &mut VisualTestContext,
        availability: Availability,
        checked: bool,
        has_handler: bool,
    ) -> accesskit::Node {
        cx.update(|window, cx| {
            let item = RovingItem {
                focus_handle: cx.focus_handle(),
                availability,
            };
            let element = item_element(
                div().id("radio").aria_label("Apple"),
                &item,
                true,
                checked,
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
    fn グループは_role_radiogroup_と名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        let node = view
            .read_with(cx, |view, _| view.group_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::RadioGroup);
        assert_eq!(node.label(), Some("Fruit"));
    }

    #[gpui::test]
    fn 項目は_role_radio_と選ばれているかどうかで公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        for (checked, toggled) in [
            (false, accesskit::Toggled::False),
            (true, accesskit::Toggled::True),
        ] {
            let node = item_node(cx, Availability::Enabled, checked, true);
            assert_eq!(node.role(), accesskit::Role::RadioButton);
            assert_eq!(node.label(), Some("Apple"));
            assert_eq!(node.toggled(), Some(toggled));
            assert!(!node.is_disabled());
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(node.supports_action(accesskit::Action::Focus));
        }
    }

    #[gpui::test]
    fn ハンドラを持たないグループの項目はクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        let node = item_node(cx, Availability::Enabled, false, false);
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効な項目は_disabled_として公開され操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ALL_ENABLED);
        let node = item_node(cx, Availability::Disabled, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));

        let node = item_node(cx, Availability::DisabledFocusable, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(
            node.supports_action(accesskit::Action::Focus),
            "無効でも到達できる項目はフォーカス操作を公開すること"
        );
    }

    #[gpui::test]
    fn tab_で選ばれている項目に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("pear"), ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("pear"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn どれも選ばれていなければ_tab_で最初の項目に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(
            changes(&view, cx).is_empty(),
            "Tab で入っただけでは選ぶよう求めないこと"
        );
    }

    #[gpui::test]
    fn tab_で入る先は無効な項目を飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("pear"));
    }

    #[gpui::test]
    fn すべての項目が無効なら_tab_で入らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("pear"), [Availability::Disabled; 3]);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None);
        assert!(is_after_focused(&view, cx));
    }

    #[gpui::test]
    fn tab_と_shift_tab_でグループの外へ出て戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("pear"), ALL_ENABLED);
        cx.simulate_keystrokes("tab tab");
        assert!(
            is_after_focused(&view, cx),
            "グループの中の Tab stop は 1 つだけであること"
        );
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("shift-tab");
        assert!(is_after_focused(&view, cx), "前へもグループの外へ出ること");
    }

    #[gpui::test]
    fn 下と右の矢印キーで次の項目へ移り選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some("plum"));
        assert_eq!(changes(&view, cx), vec!["pear", "plum"]);
    }

    #[gpui::test]
    fn 上と左の矢印キーで前の項目へ移り選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("plum"), ALL_ENABLED);
        cx.simulate_keystrokes("tab up");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert_eq!(changes(&view, cx), vec!["pear", "apple"]);
    }

    #[gpui::test]
    fn 矢印キーの移動は端で反対の端に戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab up");
        assert_eq!(focused(&view, cx), Some("plum"));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert_eq!(changes(&view, cx), vec!["plum", "apple"]);
    }

    #[gpui::test]
    fn 矢印キーの移動は無効な項目を飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("plum"));
        assert_eq!(changes(&view, cx), vec!["plum"]);
    }

    #[gpui::test]
    fn 無効でも到達できる項目にはフォーカスだけを移す(
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
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("pear"));
        press_key(cx, "space");
        click_row(cx, 1);
        assert!(changes(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), Some("pear"));

        cx.simulate_keystrokes("tab");
        assert!(
            is_after_focused(&view, cx),
            "選ばれていない項目からも Tab でグループの外へ出ること"
        );
    }

    #[gpui::test]
    fn 移れる項目がほかに無ければ矢印キーで何もしない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            None,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Disabled,
            ],
        );
        cx.simulate_keystrokes("tab down up");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 修飾キー付きの矢印キーでは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab shift-down ctrl-right alt-down");
        assert_eq!(focused(&view, cx), Some("apple"));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn space_で選ばれていない項目を選ぶよう求める(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        assert_eq!(changes(&view, cx), vec!["apple"]);
        press_key(cx, "space");
        assert_eq!(
            changes(&view, cx),
            vec!["apple"],
            "選ばれている項目では求めないこと"
        );
    }

    #[gpui::test]
    fn enter_では選ぶよう求めない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn クリックで選ぶよう求めフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        click_row(cx, 2);
        assert_eq!(changes(&view, cx), vec!["plum"]);
        assert_eq!(focused(&view, cx), Some("plum"));
        click_row(cx, 2);
        assert_eq!(
            changes(&view, cx),
            vec!["plum"],
            "選ばれている項目では求めないこと"
        );
    }

    #[gpui::test]
    fn 無効な項目はクリックしても選ばれずフォーカスされない(
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
        click_row(cx, 1);
        assert!(changes(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 自身では状態を変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.accept = false);
        click_row(cx, 1);
        click_row(cx, 1);
        assert_eq!(
            changes(&view, cx),
            vec!["pear", "pear"],
            "渡された値が変わらない限り同じ値を求めること"
        );

        cx.simulate_keystrokes("tab");
        assert!(
            is_after_focused(&view, cx),
            "値が反映されなくても Tab でグループの外へ出ること"
        );
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some("apple"),
            "Tab で入る先は渡された値の項目であること"
        );
    }

    #[gpui::test]
    fn 外部から値を変えてもハンドラを呼ばない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.selected = Some("plum"));
        assert!(changes(&view, cx).is_empty());
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("plum"));
    }

    #[gpui::test]
    fn 項目でない子は加えた順に並び矢印キーの移動に含まれない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.divider = true);
        click_row(cx, 1);
        assert!(
            changes(&view, cx).is_empty(),
            "最初の項目の次の行は項目でないこと"
        );
        click_row(cx, 2);
        assert_eq!(changes(&view, cx), vec!["pear"]);
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("apple"));
    }

    /// 項目の id が同じ 2 つのグループを、項目にフォーカスを渡さずに並べる
    struct TwoGroupsView {
        changes: Rc<RefCell<Vec<(&'static str, &'static str)>>>,
    }

    impl Render for TwoGroupsView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().children(["first", "second"].map(|group| {
                let changes = self.changes.clone();
                RadioGroup::new(group, None)
                    .on_change(move |value: &&'static str, _, _| {
                        changes.borrow_mut().push((group, *value))
                    })
                    .items(["yes", "no"].map(|value| Radio::new(value, value).h(px(20.))))
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
        click_row(cx, 1);
        cx.simulate_keystrokes("up");
        assert_eq!(
            *view.read_with(cx, |view, _| view.changes.clone()).borrow(),
            vec![("first", "no"), ("first", "yes")]
        );
    }

    #[gpui::test]
    fn 加えた項目に矢印キーで移れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        update(&view, cx, |view| view.shown = [true, false, true]);
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("plum"));

        update(&view, cx, |view| view.shown = [true; 3]);
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("pear"));
        assert_eq!(changes(&view, cx), vec!["plum", "pear"]);
    }

    #[gpui::test]
    fn フォーカス中の項目を消すと_tab_で残った項目に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("pear"));

        update(&view, cx, |view| view.shown = [true, false, true]);
        cx.simulate_keystrokes("down");
        assert_eq!(
            changes(&view, cx),
            vec!["pear"],
            "消えた項目からは矢印キーで移らないこと"
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(
            focused(&view, cx),
            Some("apple"),
            "選ばれている値の項目が無ければ最初の項目に入ること"
        );
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("plum"));
    }

    #[gpui::test]
    fn フォーカス中に無効になってもフォーカスを保つ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        update(&view, cx, |view| {
            view.availability[0] = Availability::Disabled;
        });
        assert_eq!(focused(&view, cx), Some("apple"));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("pear"));
        cx.simulate_keystrokes("up");
        assert_eq!(
            focused(&view, cx),
            Some("plum"),
            "フォーカスが外れた後は無効な項目を飛ばすこと"
        );
    }
}
