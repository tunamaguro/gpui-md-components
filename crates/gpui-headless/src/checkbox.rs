//! 見た目を持たないチェックボックス

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, MouseButton, ParentElement, RenderOnce,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

use crate::{Availability, accessibility::A11yInfoExt};

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

impl From<bool> for CheckedState {
    fn from(checked: bool) -> Self {
        if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
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

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// 見た目を持たないチェックボックス
///
/// 状態は呼び出し側が持ち、[`Checkbox::new`] に毎回渡す。チェックボックス自身は状態を変えない。
/// 支援技術には role `checkbox` と、渡した状態に対応する checked の状態として公開される。
/// アクセシブルな名前は、子の内容か `aria_label` で与えること。
/// マウスのクリック・フォーカス中の Space・支援技術からのクリック操作で [`Checkbox::on_change`] のハンドラを呼び、
/// Enter では呼ばない。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    inner: Stateful<Div>,
    state: CheckedState,
    on_change: Option<ChangeHandler>,
    provided_focus_handle: Option<FocusHandle>,
    availability: Availability,
}

impl Checkbox {
    /// `state` を表示するチェックボックスを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Checkbox::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, state: impl Into<CheckedState>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            state: state.into(),
            on_change: None,
            provided_focus_handle: None,
            availability: Availability::Enabled,
        }
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められるチェックの有無を渡す。[`CheckedState::Checked`] のときは `false`、
    /// [`CheckedState::Unchecked`] と [`CheckedState::Mixed`] のときは `true` になる。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Checkbox::new`] に渡すこと。
    /// 設定しない場合、チェックボックスは操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// チェックボックスを無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なチェックボックスは操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Checkbox::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// チェックボックスを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なチェックボックスは操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Checkbox::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのチェックボックスのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合はチェックボックスが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
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
impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.focus_handle(window, cx);
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_change = self.on_change.filter(|_| !disabled);
        let next = self.state != CheckedState::Checked;
        self.inner
            .role(accesskit::Role::CheckBox)
            .aria_toggled(self.state.into())
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
                    // APG の Checkbox パターンは Enter での操作を定義していない
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
            })
    }
}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Checkbox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for Checkbox {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Checkbox {}

/// APG の Checkbox パターン (content/patterns/checkbox/checkbox-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;

    struct CheckboxView {
        state: CheckedState,
        availability: Availability,
        focus_handle: FocusHandle,
        changes: Rc<RefCell<Vec<bool>>>,
        node: Option<accesskit::Node>,
    }

    impl Render for CheckboxView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let element = Checkbox::new("checkbox", self.state)
                .on_change(move |checked, _, _| changes.borrow_mut().push(checked))
                .track_focus(&self.focus_handle)
                .availability(self.availability)
                .aria_label("Lettuce")
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

    fn setup(
        cx: &mut TestAppContext,
        state: CheckedState,
        availability: Availability,
    ) -> (Entity<CheckboxView>, &mut VisualTestContext) {
        cx.add_window_view(|_, cx| CheckboxView {
            state,
            availability,
            focus_handle: cx.focus_handle(),
            changes: Rc::default(),
            node: None,
        })
    }

    fn changes(view: &Entity<CheckboxView>, cx: &mut VisualTestContext) -> Vec<bool> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn a11y_node(view: &Entity<CheckboxView>, cx: &mut VisualTestContext) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画済みであること")
    }

    fn focus_next(cx: &mut VisualTestContext) {
        cx.update(|window, cx| window.focus_next(cx));
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
    fn role_checkbox_として公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, Availability::Enabled);
        let node = a11y_node(&view, cx);
        assert_eq!(node.role(), accesskit::Role::CheckBox);
        assert_eq!(node.label(), Some("Lettuce"));
    }

    #[gpui::test]
    fn 状態が_checked_として公開される(cx: &mut TestAppContext) {
        for (state, toggled) in [
            (CheckedState::Unchecked, accesskit::Toggled::False),
            (CheckedState::Checked, accesskit::Toggled::True),
            (CheckedState::Mixed, accesskit::Toggled::Mixed),
        ] {
            let (view, cx) = setup(cx, state, Availability::Enabled);
            assert_eq!(a11y_node(&view, cx).toggled(), Some(toggled), "{state:?}");
        }
    }

    #[gpui::test]
    fn space_で切り替え後の値が渡される(cx: &mut TestAppContext) {
        for (state, next) in [
            (CheckedState::Unchecked, true),
            (CheckedState::Checked, false),
            (CheckedState::Mixed, true),
        ] {
            let (view, cx) = setup(cx, state, Availability::Enabled);
            focus_next(cx);
            press_key(cx, "space");
            assert_eq!(changes(&view, cx), vec![next], "{state:?}");
        }
    }

    #[gpui::test]
    fn マウスのクリックで操作できる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, Availability::Enabled);
        click(cx);
        assert_eq!(changes(&view, cx), vec![true]);
    }

    #[gpui::test]
    fn enter_では操作しない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, Availability::Enabled);
        focus_next(cx);
        press_key(cx, "enter");
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 自身では状態を変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, Availability::Enabled);
        click(cx);
        click(cx);
        assert_eq!(
            changes(&view, cx),
            vec![true, true],
            "渡された状態が変わらない限り同じ値を渡すこと"
        );
        assert_eq!(
            a11y_node(&view, cx).toggled(),
            Some(accesskit::Toggled::False)
        );
    }

    #[gpui::test]
    fn 無効なチェックボックスは操作できず_disabled_として公開される(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, Availability::Disabled);
        focus_next(cx);
        press_key(cx, "space");
        click(cx);
        assert!(changes(&view, cx).is_empty());
        let node = a11y_node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }
}
