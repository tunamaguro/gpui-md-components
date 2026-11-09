//! オンとオフを切り替える、見た目を持たない部品

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit,
    div, prelude::FluentBuilder,
};

use crate::{Availability, InteractionSource, accessibility::A11yInfoExt};

/// チェックボックスの状態
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CheckedState {
    /// チェックされていない
    #[default]
    Unchecked,
    /// チェックされている
    Checked,
    /// 一部だけチェックされている
    ///
    /// まとめて操作するチェックボックスで、配下の一部だけがチェックされていることを表す
    Mixed,
}

impl CheckedState {
    /// [`CheckedState::Checked`] なら `true` を返す。[`CheckedState::Mixed`] では `false`
    pub fn is_checked(self) -> bool {
        self == CheckedState::Checked
    }
}

impl Toggle for CheckedState {
    /// [`CheckedState::Checked`] なら [`CheckedState::Unchecked`] を、それ以外なら [`CheckedState::Checked`] を返す
    fn toggle(self) -> Self {
        match self {
            CheckedState::Checked => CheckedState::Unchecked,
            CheckedState::Unchecked | CheckedState::Mixed => CheckedState::Checked,
        }
    }
}

impl From<CheckedState> for accesskit::Toggled {
    fn from(state: CheckedState) -> Self {
        match state {
            CheckedState::Unchecked => accesskit::Toggled::False,
            CheckedState::Checked => accesskit::Toggled::True,
            CheckedState::Mixed => accesskit::Toggled::Mixed,
        }
    }
}

/// [`Choice`] が操作されたときに、次の状態を決める
pub trait Toggle {
    /// 現在の状態で操作されたときに求められる次の状態を返す
    fn toggle(self) -> Self;
}

impl Toggle for bool {
    /// 反転した値を返す
    fn toggle(self) -> Self {
        !self
    }
}

mod sealed {
    pub trait Sealed {}
}

/// [`Choice`] が支援技術に公開する role と、取りうる状態を決める
///
/// このクレートの外では実装できない
pub trait ChoiceKind: sealed::Sealed + 'static {
    /// [`Choice::new`] に渡し、[`Choice::on_change`] で受け取る状態
    ///
    /// 操作されたときの次の状態は [`Toggle::toggle`] で決まる
    type State: Copy + Toggle + Into<accesskit::Toggled>;
    /// 支援技術に公開する role
    const ROLE: accesskit::Role;
}

/// [`Checkbox`] の種類
#[derive(Debug)]
pub enum CheckboxKind {}

impl sealed::Sealed for CheckboxKind {}

impl ChoiceKind for CheckboxKind {
    type State = CheckedState;
    const ROLE: accesskit::Role = accesskit::Role::CheckBox;
}

/// [`Switch`] の種類
#[derive(Debug)]
pub enum SwitchKind {}

impl sealed::Sealed for SwitchKind {}

impl ChoiceKind for SwitchKind {
    type State = bool;
    const ROLE: accesskit::Role = accesskit::Role::Switch;
}

/// 見た目を持たないチェックボックス
///
/// 支援技術には role `checkbox` と、[`CheckedState`] に対応する checked の状態として公開される。
/// アクセシブルな名前は、子孫に置いたテキストか `aria_label` で与えること。
/// テキストは `gpui::text!` のように id を持つ要素にする。`&str`・`String`・`SharedString` をそのまま置いても名前にならない
pub type Checkbox = Choice<CheckboxKind>;

/// 見た目を持たないスイッチ
///
/// 支援技術には role `switch` と、オンかオフかの状態として公開される。
/// アクセシブルな名前は `aria_label` で与えること。子の内容は名前にならない。
/// 状態によって名前を変えないこと
pub type Switch = Choice<SwitchKind>;

type ChangeHandler<S> = Box<dyn Fn(S, &mut Window, &mut App)>;

/// オンとオフを切り替える、見た目を持たない部品
///
/// 状態は呼び出し側が持ち、[`Choice::new`] に毎回渡す。部品自身は状態を変えない。
/// マウスのクリック・フォーカス中の Space・支援技術からのクリック操作で [`Choice::on_change`] のハンドラを呼び、
/// Enter では呼ばない。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
///
/// 種類ごとの型として [`Checkbox`] と [`Switch`] がある
#[derive(IntoElement)]
pub struct Choice<K: ChoiceKind> {
    id: ElementId,
    inner: Stateful<Div>,
    state: K::State,
    on_change: Option<ChangeHandler<K::State>>,
    provided_focus_handle: Option<FocusHandle>,
    availability: Availability,
    error: Option<SharedString>,
    group: Option<SharedString>,
}

impl<K: ChoiceKind> Choice<K> {
    /// `state` を表示する部品を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Choice::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, state: K::State) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            state,
            on_change: None,
            provided_focus_handle: None,
            availability: Availability::Enabled,
            error: None,
            group: None,
        }
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態に [`Toggle::toggle`] を適用した値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Choice::new`] に渡すこと。
    /// 設定しない場合、部品は操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(
        mut self,
        handler: impl Fn(K::State, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 部品を無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な部品は操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Choice::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 部品を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な部品は操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Choice::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// 値にエラーがあることを示す。`None` を渡すとエラーのない状態に戻す。既定ではエラーなし
    ///
    /// `Some` の場合、支援技術には invalid として公開され、`message` が説明 (description) の先頭に加わる。
    /// 部品は `message` を表示しないため、エラーの内容は呼び出し側がテキストとしても表示すること
    pub fn error(mut self, message: Option<SharedString>) -> Self {
        self.error = message;
        self
    }

    /// `focus_handle` をこの部品のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合は部品が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    /// `source` のフォーカスをこの部品のフォーカスとして使い、`source` の group の名前をこの部品に付ける
    ///
    /// 子孫の要素から、`source` を通してこの部品の hover とフォーカスを参照できる。
    /// [`Choice::track_focus`] と両方を呼んだ場合は、後に呼んだ方のフォーカスを使う
    pub fn interaction_source(mut self, source: &InteractionSource) -> Self {
        self.provided_focus_handle = Some(source.focus_handle().clone());
        self.group = Some(source.group().clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| {
                    cx.focus_handle().tab_index(0).tab_stop(true)
                })
                .read(cx)
                .clone()
        })
    }
}

// TODO: フォーカス・無効状態・クリックの扱いは [`Button`](crate::button::Button) と同じ処理を複製している。1 つにまとめたい
impl<K: ChoiceKind> RenderOnce for Choice<K> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.focus_handle(window, cx);
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_change = self.on_change.filter(|_| !disabled);
        let next = self.state.toggle();
        self.inner
            .role(K::ROLE)
            .aria_toggled(self.state.into())
            .when_some(self.group, |this, group| this.group(group))
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
            .when_some(on_change, |this, on_change| {
                this.on_click(move |event, window, cx| {
                    // APG は Enter での操作を、Checkbox パターンでは定義せず、Switch パターンでは任意としている
                    if let ClickEvent::Keyboard(KeyboardClickEvent {
                        button: KeyboardButton::Enter,
                        ..
                    }) = event
                    {
                        return;
                    }
                    on_change(next, window, cx);
                })
            })
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
                if let Some(error) = &self.error {
                    node.set_invalid(accesskit::Invalid::True);
                    // AccessKit のアダプタは invalid や error_message を OS に渡さないため、
                    // メッセージを description に含めて読み上げさせる
                    let description = match node.description() {
                        Some(description) => format!("{error} {description}"),
                        None => error.to_string(),
                    };
                    node.set_description(description);
                }
            })
    }
}

impl<K: ChoiceKind> Styled for Choice<K> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<K: ChoiceKind> ParentElement for Choice<K> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<K: ChoiceKind> InteractiveElement for Choice<K> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<K: ChoiceKind> StatefulInteractiveElement for Choice<K> {}

/// APG の Checkbox パターン (content/patterns/checkbox/checkbox-pattern.html) と
/// Switch パターン (content/patterns/switch/switch-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::test_window::add_test_window;

    struct ChoiceView<K: ChoiceKind> {
        state: K::State,
        availability: Availability,
        error: Option<SharedString>,
        description: Option<SharedString>,
        focus_handle: FocusHandle,
        changes: Rc<RefCell<Vec<K::State>>>,
        node: Option<accesskit::Node>,
    }

    impl<K: ChoiceKind> Render for ChoiceView<K> {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let element = Choice::<K>::new("choice", self.state)
                .on_change(move |state, _, _| changes.borrow_mut().push(state))
                .track_focus(&self.focus_handle)
                .availability(self.availability)
                .error(self.error.clone())
                .aria_label("Lettuce")
                .when_some(self.description.clone(), |this, description| {
                    this.aria_description(description)
                })
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

    fn setup<K: ChoiceKind>(
        cx: &mut TestAppContext,
        state: K::State,
        availability: Availability,
    ) -> (Entity<ChoiceView<K>>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ChoiceView {
            state,
            availability,
            error: None,
            description: None,
            focus_handle: cx.focus_handle(),
            changes: Rc::default(),
            node: None,
        })
    }

    fn changes<K: ChoiceKind>(
        view: &Entity<ChoiceView<K>>,
        cx: &mut VisualTestContext,
    ) -> Vec<K::State> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn a11y_node<K: ChoiceKind>(
        view: &Entity<ChoiceView<K>>,
        cx: &mut VisualTestContext,
    ) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画済みであること")
    }

    fn set_error<K: ChoiceKind>(
        view: &Entity<ChoiceView<K>>,
        cx: &mut VisualTestContext,
        error: Option<&'static str>,
        description: Option<&'static str>,
    ) {
        view.update(cx, |view, cx| {
            view.error = error.map(SharedString::from);
            view.description = description.map(SharedString::from);
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
    fn checkbox_は_role_checkbox_として公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        let node = a11y_node(&view, cx);
        assert_eq!(node.role(), accesskit::Role::CheckBox);
        assert_eq!(node.label(), Some("Lettuce"));
    }

    #[gpui::test]
    fn switch_は_role_switch_として公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup::<SwitchKind>(cx, false, Availability::Enabled);
        let node = a11y_node(&view, cx);
        assert_eq!(node.role(), accesskit::Role::Switch);
        assert_eq!(node.label(), Some("Lettuce"));
    }

    #[gpui::test]
    fn checkbox_の状態が_checked_として公開される(cx: &mut TestAppContext) {
        for (state, toggled) in [
            (CheckedState::Unchecked, accesskit::Toggled::False),
            (CheckedState::Checked, accesskit::Toggled::True),
            (CheckedState::Mixed, accesskit::Toggled::Mixed),
        ] {
            let (view, cx) = setup::<CheckboxKind>(cx, state, Availability::Enabled);
            assert_eq!(a11y_node(&view, cx).toggled(), Some(toggled), "{state:?}");
        }
    }

    #[gpui::test]
    fn switch_の状態が_checked_として公開される(cx: &mut TestAppContext) {
        for (state, toggled) in [
            (false, accesskit::Toggled::False),
            (true, accesskit::Toggled::True),
        ] {
            let (view, cx) = setup::<SwitchKind>(cx, state, Availability::Enabled);
            assert_eq!(a11y_node(&view, cx).toggled(), Some(toggled), "{state:?}");
        }
    }

    #[gpui::test]
    fn checkbox_は_space_で切り替え後の値が渡される(cx: &mut TestAppContext) {
        for (state, next) in [
            (CheckedState::Unchecked, CheckedState::Checked),
            (CheckedState::Checked, CheckedState::Unchecked),
            (CheckedState::Mixed, CheckedState::Checked),
        ] {
            let (view, cx) = setup::<CheckboxKind>(cx, state, Availability::Enabled);
            cx.simulate_keystrokes("tab");
            press_key(cx, "space");
            assert_eq!(changes(&view, cx), vec![next], "{state:?}");
        }
    }

    #[gpui::test]
    fn switch_は_space_で切り替え後の値が渡される(cx: &mut TestAppContext) {
        for (state, next) in [(false, true), (true, false)] {
            let (view, cx) = setup::<SwitchKind>(cx, state, Availability::Enabled);
            cx.simulate_keystrokes("tab");
            press_key(cx, "space");
            assert_eq!(changes(&view, cx), vec![next], "{state:?}");
        }
    }

    #[gpui::test]
    fn マウスのクリックで操作できる(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        click(cx);
        assert_eq!(changes(&view, cx), vec![CheckedState::Checked]);
    }

    #[gpui::test]
    fn enter_では操作しない(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 自身では状態を変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        click(cx);
        click(cx);
        assert_eq!(
            changes(&view, cx),
            vec![CheckedState::Checked, CheckedState::Checked],
            "渡された状態が変わらない限り同じ値を渡すこと"
        );
        assert_eq!(
            a11y_node(&view, cx).toggled(),
            Some(accesskit::Toggled::False)
        );
    }

    #[gpui::test]
    fn 無効な部品は操作できず_disabled_として公開される(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Disabled);
        cx.simulate_keystrokes("tab");
        press_key(cx, "space");
        click(cx);
        assert!(changes(&view, cx).is_empty());
        let node = a11y_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn エラーがない部品は_invalid_として公開されない(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        let node = a11y_node(&view, cx);
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
    }

    #[gpui::test]
    fn エラーは_invalid_とメッセージの説明として公開される(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup::<SwitchKind>(cx, false, Availability::Enabled);
        set_error(&view, cx, Some("Required"), None);
        let node = a11y_node(&view, cx);
        assert_eq!(node.invalid(), Some(accesskit::Invalid::True));
        assert_eq!(node.description(), Some("Required"));

        set_error(&view, cx, None, None);
        let node = a11y_node(&view, cx);
        assert_eq!(
            node.invalid(),
            None,
            "None を渡すとエラーのない状態に戻ること"
        );
        assert_eq!(node.description(), None);
    }

    #[gpui::test]
    fn エラーのメッセージは既存の説明の前に加わる(cx: &mut TestAppContext) {
        let (view, cx) = setup::<CheckboxKind>(cx, CheckedState::Unchecked, Availability::Enabled);
        set_error(&view, cx, Some("Required"), Some("Fresh only"));
        assert_eq!(
            a11y_node(&view, cx).description(),
            Some("Required Fresh only")
        );
    }
}
