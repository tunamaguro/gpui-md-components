//! 1 つだけを選ぶ、見た目を持たないラジオボタンのグループ

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit,
    div, prelude::FluentBuilder,
};

use crate::{
    Availability, InteractionSource,
    accessibility::A11yInfoExt,
    roving::{self, NextItem, PreviousItem, RovingItem},
};

type ChangeHandler<V> = Rc<dyn Fn(V, &mut Window, &mut App)>;

/// 見た目を持たないラジオボタンのグループ
///
/// 状態 (選ばれている値) は呼び出し側が持ち、[`RadioGroup::new`] に毎回渡す。グループ自身は状態を変えない。
/// 項目は [`RadioGroup::item`] で渡した順に並ぶ。
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初の有効な項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる
/// - 上下左右の矢印キーで前後の項目へフォーカスを移し、移った先の項目を選ぶよう求める。端では反対の端に戻る。
///   Tab で到達できない無効な項目は飛ばす。無効でも Tab で到達できる項目にはフォーカスだけを移す。
///   移れる項目がほかに無い場合は、キー入力を祖先に渡す
/// - フォーカス中の Space・マウスのクリック・支援技術からのクリック操作で、まだ選ばれていない項目を選ぶよう求める。
///   Enter では求めない
/// - フォーカス中の項目を消すと、キー入力はどの項目にも届かなくなる。グループが内部で持つフォーカスは捨てられる。
///   [`Radio::interaction_source`] で渡したフォーカスはフォーカスがあるとみなされたまま残り、同じフォーカスで描き直すと戻る。
///   隣の項目へ移すには、消す項目と移す先の項目に [`Radio::interaction_source`] でフォーカスを渡し、項目を消す側で移すこと
///
/// 矢印キーの割り当ては [`crate::init`] が key context を付けて登録し、グループの `key_context` はその値で上書きされる。
/// 最も深い context でも成り立つ割り当て (context の指定なし、`!Foo` など) は、[`crate::init`] より後に登録したか、
/// 項目に `key_context` を付けた場合に、同じキーのグループの割り当てより優先される。
/// その action を祖先か `App::on_action` のリスナーが処理すると、矢印キーで項目の間を移れない。
/// 避けるには、その割り当てにグループの祖先の context を付ける。項目に `key_context` を付けない場合は、
/// [`crate::init`] をその割り当てより後に呼んでもよい
///
/// 支援技術には role `radiogroup` として公開され、項目は role `radio` と選ばれているかどうかの状態として公開される。
/// グループの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct RadioGroup<V: Clone + PartialEq + 'static> {
    id: ElementId,
    inner: Stateful<Div>,
    selected: Option<V>,
    on_change: Option<ChangeHandler<V>>,
    items: Vec<Radio<V>>,
}

impl<V: Clone + PartialEq + 'static> RadioGroup<V> {
    /// `selected` の値を持つ項目を選ばれた状態で表示するグループを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、項目のフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, selected: Option<V>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            selected,
            on_change: None,
            items: Vec::new(),
        }
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`RadioGroup::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(V, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: Radio<V>) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = Radio<V>>) -> Self {
        self.items.extend(items);
        self
    }
}

impl<V: Clone + PartialEq + 'static> RenderOnce for RadioGroup<V> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handles = roving::focus_handles(
            &self.id,
            self.items.iter().map(|item| {
                (
                    item.id.clone(),
                    item.source
                        .as_ref()
                        .map(|source| source.focus_handle().clone()),
                )
            }),
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
        let selected_ix = self
            .items
            .iter()
            .position(|item| Some(&item.value) == self.selected.as_ref());
        let tab_stop = roving::tab_stop(&roving_items, selected_ix, window);
        let values: Rc<[V]> = self.items.iter().map(|item| item.value.clone()).collect();

        let move_focus = {
            let on_change = self.on_change.clone();
            let selected = self.selected.clone();
            move |forward: bool, window: &mut Window, cx: &mut App| {
                let Some(ix) = roving::neighbor(&roving_items, forward, window) else {
                    // 移る先が無いキー入力は、祖先や他のキー割り当てに渡す
                    cx.propagate();
                    return;
                };
                window.focus(&roving_items[ix].focus_handle, cx);
                if let Some(on_change) = &on_change
                    && !roving_items[ix].disabled
                    && Some(&values[ix]) != selected.as_ref()
                {
                    on_change(values[ix].clone(), window, cx);
                }
            }
        };
        let move_focus = Rc::new(move_focus);

        let items = self.items.into_iter().zip(focus_handles).enumerate().map(
            |(ix, (item, focus_handle))| {
                let checked = Some(ix) == selected_ix;
                item.into_element_with(
                    focus_handle,
                    tab_stop == Some(ix),
                    checked,
                    self.on_change.clone(),
                    window,
                )
            },
        );

        self.inner
            .role(accesskit::Role::RadioGroup)
            .key_context(roving::RADIO_GROUP_CONTEXT)
            .on_action({
                let move_focus = move_focus.clone();
                move |_: &NextItem, window, cx| move_focus(true, window, cx)
            })
            .on_action(move |_: &PreviousItem, window, cx| move_focus(false, window, cx))
            .children(items)
    }
}

impl<V: Clone + PartialEq + 'static> Styled for RadioGroup<V> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<V: Clone + PartialEq + 'static> InteractiveElement for RadioGroup<V> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<V: Clone + PartialEq + 'static> StatefulInteractiveElement for RadioGroup<V> {}

/// [`RadioGroup`] の項目
///
/// [`RadioGroup::item`] に渡して使う。アクセシブルな名前は、子孫に置いたテキストか `aria_label` で与えること。
/// テキストは `gpui::text!` のように id を持つ要素にする。`&str`・`String`・`SharedString` をそのまま置いても名前にならない
pub struct Radio<V> {
    id: ElementId,
    inner: Stateful<Div>,
    value: V,
    availability: Availability,
    source: Option<InteractionSource>,
}

impl<V> Radio<V> {
    /// `value` を表す項目を作る
    ///
    /// `id` はグループの中で一意にし、描画のたびに同じ値を渡すこと。
    /// [`Radio::interaction_source`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: V) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value,
            availability: Availability::Enabled,
            source: None,
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

    /// `source` のフォーカスをこの項目のフォーカスとして使い、`source` の group の名前をこの項目に付ける
    ///
    /// 子孫の要素から、`source` を通してこの項目の hover とフォーカスを参照できる。
    /// Tab で到達できるかはグループが決める
    pub fn interaction_source(mut self, source: &InteractionSource) -> Self {
        self.source = Some(source.clone());
        self
    }

    // TODO: フォーカス・無効状態・クリックの扱いは [`Button`](crate::button::Button) と同じ処理を複製している。1 つにまとめたい
    fn into_element_with(
        self,
        focus_handle: FocusHandle,
        tab_stop: bool,
        checked: bool,
        on_change: Option<ChangeHandler<V>>,
        window: &Window,
    ) -> impl IntoElement + use<V>
    where
        V: Clone + 'static,
    {
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        // 選ばれている項目は操作されても選ぶよう求めないが、支援技術にはクリック操作を公開する
        let on_change = on_change.filter(|_| !disabled);
        let value = self.value;
        let group: Option<SharedString> = self.source.map(|source| source.group().clone());
        self.inner
            .role(accesskit::Role::RadioButton)
            .aria_toggled(if checked {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            })
            .when_some(group, |this, group| this.group(group))
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
                let select = Rc::new(move |window: &mut Window, cx: &mut App| {
                    if !checked {
                        on_change(value.clone(), window, cx);
                    }
                });
                this.on_click({
                    let select = select.clone();
                    move |event, window, cx| {
                        // APG の Radio Group パターンは Enter での操作を定義していない
                        if let ClickEvent::Keyboard(KeyboardClickEvent {
                            button: KeyboardButton::Enter,
                            ..
                        }) = event
                        {
                            return;
                        }
                        select(window, cx);
                    }
                })
                // gpui は on_a11y_action が無いと、支援技術のクリックをノードの中央へのマウス操作として届け、
                // 上に重なった要素が受け取る。マウスのクリックと同じくフォーカスも移す
                .on_a11y_action(accesskit::Action::Click, {
                    let focus_handle = focus_handle.clone();
                    move |_, window, cx| {
                        window.focus(&focus_handle, cx);
                        select(window, cx);
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

impl<V> Styled for Radio<V> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<V> ParentElement for Radio<V> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<V> InteractiveElement for Radio<V> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<V> StatefulInteractiveElement for Radio<V> {}

/// APG の Radio Group パターン (content/patterns/radio/radio-group-pattern.html) と
/// Keyboard Interface の roving tabindex (content/practices/keyboard-interface/keyboard-interface-practice.html)
/// で期待される振る舞い
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

    actions!(radio_test, [OuterDown]);

    const VALUES: [&str; 3] = ["apple", "pear", "plum"];
    const OUTER_CONTEXT: &str = "RadioTestOuter";

    struct GroupView {
        selected: Option<&'static str>,
        availability: [Availability; 3],
        /// 描画する項目
        shown: [bool; 3],
        /// `false` なら項目に `sources` を渡さず、グループが内部で持つフォーカスを使う
        with_sources: bool,
        sources: [InteractionSource; 3],
        before: FocusHandle,
        after: FocusHandle,
        changes: Rc<RefCell<Vec<&'static str>>>,
        /// グループの外で `OuterDown` を処理した回数
        outer_downs: Rc<RefCell<usize>>,
        group_node: Option<accesskit::Node>,
    }

    impl Render for GroupView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let group = RadioGroup::new("fruit", self.selected)
                .on_change(move |value, _, _| changes.borrow_mut().push(value))
                .aria_label("Fruit")
                .flex()
                .flex_col()
                .items(
                    VALUES
                        .iter()
                        .enumerate()
                        .filter(|(ix, _)| self.shown[*ix])
                        .map(|(ix, value)| {
                            let item = Radio::new(*value, *value)
                                .availability(self.availability[ix])
                                .aria_label(*value)
                                .h(px(20.));
                            if self.with_sources {
                                item.interaction_source(&self.sources[ix])
                            } else {
                                item
                            }
                        }),
                )
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(group.a11y_role().expect("role を持つこと"));
            group.write_a11y_info(&mut node);
            self.group_node = Some(node);
            let outer_downs = self.outer_downs.clone();
            div()
                .key_context(OUTER_CONTEXT)
                .on_action(move |_: &OuterDown, _, _| *outer_downs.borrow_mut() += 1)
                .flex()
                .flex_col()
                .child(Button::new("before").track_focus(&self.before).h(px(20.)))
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
            shown: [true; 3],
            with_sources: true,
            sources: [
                InteractionSource::new(cx),
                InteractionSource::new(cx),
                InteractionSource::new(cx),
            ],
            before: cx.focus_handle(),
            after: cx.focus_handle(),
            changes: Rc::default(),
            outer_downs: Rc::default(),
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

    const ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある項目の位置。前のボタンは `Some(-1)`、後のボタンは `Some(3)`
    fn focused(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Option<i32> {
        let (sources, before, after) = view.read_with(cx, |view, _| {
            (
                view.sources.clone(),
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
            sources
                .iter()
                .position(|source| source.is_focused(window))
                .map(|ix| ix as i32)
        })
    }

    fn changes(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
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

    /// `ix` 番目の項目をクリックする
    fn click_item(cx: &mut VisualTestContext, ix: usize) {
        cx.simulate_click(point(px(10.), px(30. + 20. * ix as f32)), Modifiers::none());
    }

    /// 項目を、選ばれているかを `checked`、Tab で到達できるかを `tab_stop` として描いたときの a11y ノード
    fn item_node(
        cx: &mut VisualTestContext,
        availability: Availability,
        tab_stop: bool,
        checked: bool,
        on_change: bool,
    ) -> accesskit::Node {
        cx.update(|window, cx| {
            let on_change: Option<ChangeHandler<&'static str>> =
                on_change.then(|| Rc::new(|_, _: &mut Window, _: &mut App| {}) as _);
            let element = Radio::new("apple", "apple")
                .availability(availability)
                .aria_label("Apple")
                .into_element_with(cx.focus_handle(), tab_stop, checked, on_change, window)
                .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        })
    }

    #[gpui::test]
    fn グループは_role_radiogroup_と名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        let node = view
            .read_with(cx, |view, _| view.group_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::RadioGroup);
        assert_eq!(node.label(), Some("Fruit"));
    }

    #[gpui::test]
    fn 項目は_role_radio_と選ばれているかの状態で公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ENABLED);
        let node = item_node(cx, Availability::Enabled, true, true, true);
        assert_eq!(node.role(), accesskit::Role::RadioButton);
        assert_eq!(node.label(), Some("Apple"));
        assert_eq!(node.toggled(), Some(accesskit::Toggled::True));
        assert!(node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
        assert!(!node.is_disabled());

        let node = item_node(cx, Availability::Enabled, false, false, true);
        assert_eq!(node.toggled(), Some(accesskit::Toggled::False));
        assert!(
            node.supports_action(accesskit::Action::Focus),
            "Tab で到達できない項目にも支援技術からフォーカスを移せること"
        );
    }

    #[gpui::test]
    fn 無効な項目は_disabled_として公開され操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None, ENABLED);
        let node = item_node(cx, Availability::Disabled, true, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));

        let node = item_node(cx, Availability::DisabledFocusable, true, false, true);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 選ばれていなければ_tab_で最初の項目に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(0));
    }

    #[gpui::test]
    fn tab_で選ばれている項目に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("pear"), ENABLED);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(1));
    }

    #[gpui::test]
    fn shift_tab_でも選ばれている項目に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("pear"), ENABLED);
        cx.simulate_keystrokes("shift-tab shift-tab");
        assert_eq!(focused(&view, cx), Some(1));
    }

    #[gpui::test]
    fn tab_で到達できる項目は_1_つだけ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab tab");
        assert_eq!(focused(&view, cx), Some(3), "Tab でグループの外に出ること");
        cx.simulate_keystrokes("shift-tab shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some(-1),
            "Shift+Tab でグループの外に出ること"
        );
    }

    #[gpui::test]
    fn 矢印キーで次の項目に移って選ぶよう求める(cx: &mut TestAppContext) {
        for key in ["down", "right"] {
            let (view, cx) = setup(cx, Some("apple"), ENABLED);
            cx.simulate_keystrokes("tab tab");
            cx.simulate_keystrokes(key);
            assert_eq!(focused(&view, cx), Some(1), "{key}");
            assert_eq!(changes(&view, cx), vec!["pear"], "{key}");
        }
    }

    #[gpui::test]
    fn 矢印キーで前の項目に移って選ぶよう求める(cx: &mut TestAppContext) {
        for key in ["up", "left"] {
            let (view, cx) = setup(cx, Some("pear"), ENABLED);
            cx.simulate_keystrokes("tab tab");
            cx.simulate_keystrokes(key);
            assert_eq!(focused(&view, cx), Some(0), "{key}");
            assert_eq!(changes(&view, cx), vec!["apple"], "{key}");
        }
    }

    #[gpui::test]
    fn 端では反対の端に戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("plum"), ENABLED);
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(focused(&view, cx), Some(0));
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some(2));
        assert_eq!(
            changes(&view, cx),
            vec!["apple"],
            "選ばれたままの項目に戻っても求めないこと"
        );
    }

    #[gpui::test]
    fn 選ばれた値を反映しなくても移ったフォーカスは残り_tab_で外に出る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), ENABLED);
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(focused(&view, cx), Some(1));
        cx.simulate_keystrokes("tab");
        assert_eq!(
            focused(&view, cx),
            Some(3),
            "選ばれている項目を経由せずに外に出ること"
        );
    }

    #[gpui::test]
    fn 無効な項目は矢印キーで飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            None,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(focused(&view, cx), Some(2));
        assert_eq!(changes(&view, cx), vec!["plum"]);
    }

    #[gpui::test]
    fn 無効でも到達できる項目にはフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            None,
            [
                Availability::Enabled,
                Availability::DisabledFocusable,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(focused(&view, cx), Some(1));
        assert!(changes(&view, cx).is_empty());
        press_key(cx, "space");
        assert!(changes(&view, cx).is_empty(), "Space でも選ばないこと");

        cx.simulate_keystrokes("up");
        click_item(cx, 1);
        assert_eq!(
            focused(&view, cx),
            Some(1),
            "クリックでフォーカスは移ること"
        );
        assert_eq!(
            changes(&view, cx),
            vec!["apple"],
            "クリックでは選ばないこと"
        );
    }

    #[gpui::test]
    fn 選ばれている項目が無効なら最初の有効な項目に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            Some("apple"),
            [
                Availability::Disabled,
                Availability::DisabledFocusable,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2));
    }

    #[gpui::test]
    fn すべて無効ならグループに_tab_で入らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, [Availability::Disabled; 3]);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(3));
    }

    #[gpui::test]
    fn space_で選ぶよう求め_enter_では求めない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab");
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty());
        press_key(cx, "space");
        assert_eq!(changes(&view, cx), vec!["apple"]);
    }

    #[gpui::test]
    fn 選ばれている項目では選ぶよう求めない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some("apple"), ENABLED);
        cx.simulate_keystrokes("tab tab");
        press_key(cx, "space");
        click_item(cx, 0);
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn クリックでフォーカスを移して選ぶよう求める(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        click_item(cx, 2);
        assert_eq!(focused(&view, cx), Some(2));
        assert_eq!(changes(&view, cx), vec!["plum"]);
    }

    #[gpui::test]
    fn 無効な項目はクリックしても選ぶよう求めない(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            None,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        click_item(cx, 1);
        assert!(changes(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 外から値を変えても求めを通知しない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        update(&view, cx, |view| view.selected = Some("plum"));
        assert!(changes(&view, cx).is_empty());
        cx.simulate_keystrokes("tab tab");
        assert_eq!(
            focused(&view, cx),
            Some(2),
            "新しく選ばれた項目に Tab で入ること"
        );
    }

    #[gpui::test]
    fn フォーカス中に無効になってもフォーカスを保つ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab down");
        update(&view, cx, |view| {
            view.availability[1] = Availability::Disabled
        });
        assert_eq!(focused(&view, cx), Some(1));
        press_key(cx, "space");
        click_item(cx, 1);
        assert_eq!(
            changes(&view, cx),
            vec!["pear"],
            "無効になった項目は Space でもクリックでも選ばないこと"
        );
        cx.simulate_keystrokes("down");
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
        cx.update(|cx| cx.bind_keys([KeyBinding::new("down", OuterDown, Some(OUTER_CONTEXT))]));
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab down");
        let outer_downs = view.read_with(cx, |view, _| view.outer_downs.clone());
        assert_eq!(*outer_downs.borrow(), 0, "移れる間は渡さないこと");

        update(&view, cx, |view| {
            view.availability = [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Disabled,
            ]
        });
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some(1));
        assert_eq!(*outer_downs.borrow(), 1);
    }

    #[gpui::test]
    fn グループが持つフォーカスは描画をまたいで保たれる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, ENABLED);
        update(&view, cx, |view| view.with_sources = false);
        cx.simulate_keystrokes("tab tab");
        cx.simulate_keystrokes("down");
        cx.simulate_keystrokes("down");
        assert_eq!(changes(&view, cx), vec!["pear", "plum"]);
    }

    #[gpui::test]
    fn グループが持つフォーカスは項目が消えると捨てられる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, ENABLED);
        update(&view, cx, |view| view.with_sources = false);
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(changes(&view, cx), vec!["pear"]);

        update(&view, cx, |view| view.shown[1] = false);
        cx.simulate_keystrokes("down");
        update(&view, cx, |view| view.shown[1] = true);
        cx.simulate_keystrokes("down");
        assert_eq!(
            changes(&view, cx),
            vec!["pear"],
            "消えた項目のフォーカスは、再び描画されても戻らないこと"
        );
    }

    #[gpui::test]
    fn 加えた項目にも矢印キーで移れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        update(&view, cx, |view| {
            view.with_sources = false;
            view.shown[1] = false;
        });
        cx.simulate_keystrokes("tab tab down");
        update(&view, cx, |view| view.shown[1] = true);
        cx.simulate_keystrokes("up");
        cx.simulate_keystrokes("up");
        assert_eq!(
            changes(&view, cx),
            vec!["plum", "pear", "apple"],
            "加えた項目に移り、そこから先にも移れること"
        );
    }

    #[gpui::test]
    fn 渡したフォーカスは項目を描き直すと戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, ENABLED);
        cx.simulate_keystrokes("tab tab down");
        update(&view, cx, |view| view.shown[1] = false);
        assert_eq!(
            focused(&view, cx),
            Some(1),
            "消えている間もフォーカスがあるとみなされること"
        );
        update(&view, cx, |view| view.shown[1] = true);
        cx.simulate_keystrokes("down");
        assert_eq!(
            changes(&view, cx),
            vec!["pear", "plum"],
            "描き直した項目から矢印キーで移れること"
        );
    }
}
