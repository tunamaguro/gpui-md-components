//! 見た目を持たないモーダルのダイアログ

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Div, Element, ElementId, Entity, FocusHandle, Global,
    GlobalElementId, InspectorElementId, InteractiveElement, Interactivity, IntoElement, LayoutId,
    ParentElement, Pixels, Position, RenderOnce, Stateful, StatefulInteractiveElement, Style,
    StyleRefinement, Styled, Window, accesskit, div, point, prelude::FluentBuilder,
};

use crate::{
    accessibility::A11yInfoExt,
    navigation::{FocusNext, FocusPrevious},
    popup,
};

/// ダイアログを、中から開くメニューなどのポップアップやツールチップより奥に描くための、遅延描画の優先度
const PRIORITY: usize = popup::PRIORITY - 1;

type DismissHandler = Rc<dyn Fn(&DismissReason, &mut Window, &mut App)>;

/// 描画中のフレームで、ここまでにレイアウトした開いているダイアログの数
#[derive(Default)]
struct OpenDialogs(Cell<usize>);

impl Global for OpenDialogs {}

/// 開いているダイアログを数える状態を登録する。[`crate::init`] から 1 回呼ぶ
pub(crate) fn init(cx: &mut App) {
    cx.set_global(OpenDialogs::default());
}

/// ウィンドウのレイアウトを始める前に呼び、開いているダイアログの数を 0 に戻す
pub(crate) fn begin_layout(cx: &App) {
    if let Some(open) = cx.try_global::<OpenDialogs>() {
        open.0.set(0);
    }
}

/// [`begin_layout`] の後にレイアウトした中に、開いているダイアログがあるか
pub(crate) fn any_open(cx: &App) -> bool {
    cx.try_global::<OpenDialogs>()
        .is_some_and(|open| open.0.get() > 0)
}

/// ダイアログを閉じるよう求めた操作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DismissReason {
    /// Escape が押された
    Escape,
    /// 幕がクリックされた
    Scrim,
}

/// ダイアログが支援技術に公開する role を決める種類
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DialogKind {
    /// 通常のダイアログ。role `dialog` として公開する
    #[default]
    Dialog,
    /// 利用者の作業を中断して短く重要な内容を伝え、応答を求めるダイアログ。role `alertdialog` として公開する
    ///
    /// 伝える内容を `aria_description` で与えること
    Alert,
}

/// 見た目を持たないモーダルのダイアログ
///
/// 開いているかどうかは呼び出し側が持ち、[`Dialog::new`] に毎回渡す。
/// ダイアログ自身は状態を変えず、閉じることは [`Dialog::on_dismiss`] で求める。
/// 置いた場所のレイアウトには影響せず、開いている間は、ウィンドウ全体を覆う幕とその子のダイアログを、ほかの要素より手前に描く。
/// ダイアログの位置は幕 ([`Dialog::scrim`]) のレイアウトで決める
///
/// 開いている間
///
/// - 開いたとき、[`Dialog::initial_focus`] の要素か、ダイアログの中で Tab で到達できる最初の要素にフォーカスを移す。
///   どちらも無ければダイアログにフォーカスを移す
/// - Tab と Shift+Tab は、ダイアログの中で Tab で到達できる要素の間だけを巡る
/// - フォーカスがダイアログの外へ移されると、ダイアログにフォーカスを戻す。
///   複数のダイアログが開いている間は、最後にレイアウトしたダイアログだけが戻す
/// - 修飾キーなしの Escape と、幕のクリックで、閉じるよう求める
/// - 幕とダイアログの上では、クリックやスクロールは背後の要素に届かない
/// - 閉じたとき、フォーカスがダイアログの中にあれば、開く前にフォーカスのあった要素へ戻す
///
/// 支援技術には modal の `dialog` か `alertdialog` として、ウィンドウの直下に公開される。名前は `aria_label` で与えること。
/// ウィンドウのルートを [`crate::root`] で包むと、開いている間はダイアログ以外を支援技術から隠す。
/// ただし、ほかの部品がウィンドウの手前に描く要素 (開いているメニューなど) は隠れないため、ダイアログを開く前に閉じること。
/// 一覧のように子を遅れてレイアウトする要素や、キャッシュされたビューの中に置くと、隠すかどうかの判定に含まれない。
/// アプリ全体のキー割り当てと、ダイアログを置いた場所の祖先のキーのハンドラは、ダイアログが開いている間も働くため、
/// ウィンドウのルートの近くに置くこと。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    inner: Stateful<Div>,
    open: bool,
    on_dismiss: Option<DismissHandler>,
    scrim: Div,
    kind: DialogKind,
    initial_focus: Option<FocusHandle>,
}

impl Dialog {
    /// `open` なら開いた状態で表示するダイアログを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, open: bool) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            open,
            on_dismiss: None,
            scrim: div(),
            kind: DialogKind::default(),
            initial_focus: None,
        }
    }

    /// 閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、閉じるよう求めた操作を渡す。
    /// 閉じるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Dialog::new`] に渡すこと。
    /// 設定しない場合、Escape と幕のクリックでは何も起きない
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&DismissReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// ウィンドウ全体を覆う幕にする要素を設定する。既定は見た目もレイアウトの指定も無い要素
    ///
    /// 幕の大きさはウィンドウの大きさにする。ダイアログは幕の子の最後に加えるため、
    /// ダイアログの位置と、ウィンドウの端からの余白は、`scrim` のレイアウトで決める
    pub fn scrim(mut self, scrim: Div) -> Self {
        self.scrim = scrim;
        self
    }

    /// 支援技術に公開する role を決める種類を設定する。既定は [`DialogKind::Dialog`]
    pub fn kind(mut self, kind: DialogKind) -> Self {
        self.kind = kind;
        self
    }

    /// 開いたときに、Tab で到達できる最初の要素の代わりに `focus_handle` の要素へフォーカスを移す
    ///
    /// `focus_handle` はダイアログの中の要素のフォーカスにすること。
    /// 取り消せない操作を確認するダイアログで、最も害の少ない操作にフォーカスしたいときなどに使う。
    /// 開く操作のハンドラでダイアログの中の要素へフォーカスを移しても、開いたときに上書きされる
    pub fn initial_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.initial_focus = Some(focus_handle.clone());
        self
    }
}

/// `inner` を、フォーカスが `focus` のダイアログの要素にする。`on_dismiss` は Escape で閉じるよう求めるときに呼ぶ
fn dialog_element(
    inner: Stateful<Div>,
    kind: DialogKind,
    focus: &FocusHandle,
    on_dismiss: Option<DismissHandler>,
) -> impl IntoElement + use<> {
    let next_focus = focus.clone();
    let previous_focus = focus.clone();
    inner
        .role(match kind {
            DialogKind::Dialog => accesskit::Role::Dialog,
            DialogKind::Alert => accesskit::Role::AlertDialog,
        })
        .track_focus(focus)
        .occlude()
        .on_key_down(move |event, window, cx| {
            if event.keystroke.key != "escape" || event.keystroke.modifiers.modified() {
                return;
            }
            if let Some(on_dismiss) = &on_dismiss {
                on_dismiss(&DismissReason::Escape, window, cx);
            }
            cx.stop_propagation();
        })
        .on_action(move |_: &FocusNext, window, cx| {
            focus_within(&next_focus, Window::focus_next, window, cx);
        })
        .on_action(move |_: &FocusPrevious, window, cx| {
            focus_within(&previous_focus, Window::focus_prev, window, cx);
        })
        .a11y_info(|node: &mut accesskit::Node| node.set_modal())
}

/// `step` でフォーカスを移すことを、フォーカスが `dialog` のダイアログの中に入るまで繰り返す
///
/// ウィンドウの Tab で到達できる要素を一巡してもダイアログの中に入らなければ、フォーカスを元に戻す
fn focus_within(
    dialog: &FocusHandle,
    step: fn(&mut Window, &mut App),
    window: &mut Window,
    cx: &mut App,
) {
    let start = window.focused(cx);
    let mut first = None;
    loop {
        step(window, cx);
        let focused = window.focused(cx);
        if focused != start && dialog.contains_focused(window, cx) {
            return;
        }
        if focused == start || first.as_ref() == Some(&focused) {
            break;
        }
        first.get_or_insert(focused);
    }
    if let Some(start) = start {
        window.focus(&start, cx);
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            open: false,
            focus: cx.focus_handle(),
            return_focus: None,
        });
        let dialog = self.open.then(|| {
            let viewport = window.viewport_size();
            let focus = state.read(cx).focus.clone();
            let on_dismiss = self.on_dismiss;
            let dialog = dialog_element(self.inner, self.kind, &focus, on_dismiss.clone());
            InteractiveElement::id(self.scrim, self.id.clone())
                .w(viewport.width)
                .h(viewport.height)
                .occlude()
                .when_some(on_dismiss, |this, on_dismiss| {
                    this.on_click(move |_, window, cx| {
                        on_dismiss(&DismissReason::Scrim, window, cx)
                    })
                })
                .child(dialog)
                .into_any_element()
        });
        Anchor {
            state,
            open: self.open,
            dialog,
            initial_focus: self.initial_focus,
            ordinal: 0,
        }
    }
}

impl Styled for Dialog {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Dialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for Dialog {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Dialog {}

/// フレームをまたいで持つダイアログの状態
struct State {
    /// 最後に描いたフレームで開いていたか
    open: bool,
    /// ダイアログのフォーカス。フォーカスがダイアログの中にあるかの判定に使う
    focus: FocusHandle,
    /// 開いたときにフォーカスのあった要素
    return_focus: Option<FocusHandle>,
}

/// ダイアログを置いた場所に入り、開いたダイアログをウィンドウ全体に描いて、開閉に合わせてフォーカスを移す要素
struct Anchor {
    state: Entity<State>,
    open: bool,
    dialog: Option<AnyElement>,
    initial_focus: Option<FocusHandle>,
    /// このフレームで開いているダイアログの中で、レイアウトした順の番号 (1 始まり)
    ordinal: usize,
}

impl IntoElement for Anchor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Anchor {
    type RequestLayoutState = ();
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
        if self.open
            && let Some(open) = cx.try_global::<OpenDialogs>()
        {
            self.ordinal = open.0.get() + 1;
            open.0.set(self.ordinal);
        }
        let style = Style {
            position: Position::Absolute,
            ..Style::default()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        dialog.layout_as_root(
            window.viewport_size().map(AvailableSpace::Definite),
            window,
            cx,
        );
        window.defer_draw(dialog, point(Pixels::ZERO, Pixels::ZERO), PRIORITY, None);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focused = window.focused(cx);
        let (was_open, focus, return_focus) = self.state.update(cx, |state, _| {
            let was_open = std::mem::replace(&mut state.open, self.open);
            if self.open && !was_open {
                state.return_focus = focused.clone();
            }
            (was_open, state.focus.clone(), state.return_focus.clone())
        });
        // contains_focused は前のフレームで判定する
        let contained = focus.contains_focused(window, cx);
        if self.open && !was_open {
            // 描画中のフォーカスの移動は再描画されず、ダイアログの中の要素も描き終えるまで Tab の順に入らないため、描き終えてから移す
            let initial_focus = self.initial_focus.take();
            window.defer(cx, move |window, cx| match initial_focus {
                Some(initial_focus) => window.focus(&initial_focus, cx),
                None => {
                    window.focus(&focus, cx);
                    focus_within(&focus, Window::focus_next, window, cx);
                }
            });
        } else if self.open && !contained {
            // 手前に別のダイアログが開いている間は、そのダイアログにフォーカスを任せる
            let topmost = cx
                .try_global::<OpenDialogs>()
                .is_none_or(|open| open.0.get() == self.ordinal);
            if topmost {
                window.defer(cx, move |window, cx| {
                    // 同じフレームで閉じたダイアログがフォーカスを戻し終えてから、描き終えた後のフレームで確かめ直す
                    window.defer(cx, move |window, cx| {
                        if !focus.contains_focused(window, cx) {
                            window.focus(&focus, cx);
                        }
                    });
                });
            }
        } else if !self.open
            && was_open
            && contained
            && let Some(return_focus) = return_focus
        {
            // それまでに別の要素へフォーカスが移されていれば戻さない
            window.defer(cx, move |window, cx| {
                if window.focused(cx) == focused {
                    window.focus(&return_focus, cx);
                }
            });
        }
    }
}

/// APG の Dialog (Modal) パターン (content/patterns/dialog-modal/dialog-modal-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        Context, Hsla, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render, TestAppContext,
        VisualTestContext, px, size,
    };

    use super::*;
    use crate::{
        button::Button,
        menu::{Menu, MenuItem},
        select::{Select, SelectListbox, SelectOption},
        test_window::add_test_window,
    };

    /// ダイアログの最後に置く、一覧を開く部品
    #[derive(Clone, Copy, PartialEq)]
    enum Popup {
        Select,
        Menu,
    }

    const DIALOG_COLOR: fn() -> Hsla = gpui::red;

    /// 500x400 のウィンドウに、(0, 0) から 40x20 の開くボタンと、(100, 0) から 40x20 の別のボタンを置く。
    /// ダイアログは 200x100 で、幕の中央 (150, 150) に置き、`buttons` なら 2 つのボタンを持つ。
    /// `second_open` なら、ボタンを 1 つ持つ 100x50 の 2 つ目のダイアログを手前に開く。
    /// (200, 0) から 40x20 のボタンにメニューを付け、その項目 (幅 100、高さ 20) を操作するとダイアログを開く。
    /// `popup` なら、ダイアログのボタンの後に、項目を 1 つ持つセレクトかメニューを付けたボタンを置く
    struct DialogView {
        popup: Option<Popup>,
        popup_expanded: bool,
        popup_focus: FocusHandle,
        open: bool,
        second_open: bool,
        menu_expanded: bool,
        buttons: bool,
        dismissable: bool,
        /// 開いたときに 2 つ目のボタンにフォーカスを移すか
        initial_ok: bool,
        /// 閉じるよう求められたときに、別のボタンにフォーカスを移すか
        focus_other_on_dismiss: bool,
        kind: DialogKind,
        /// `on_dismiss` に渡された値
        dismissals: Rc<RefCell<Vec<DismissReason>>>,
        other_clicks: Rc<RefCell<usize>>,
        opener_focus: FocusHandle,
        other_focus: FocusHandle,
        cancel_focus: FocusHandle,
        ok_focus: FocusHandle,
        inner_focus: FocusHandle,
        menu_focus: FocusHandle,
    }

    impl Render for DialogView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let scrim = || div().flex().items_center().justify_center();
            let dismissals = self.dismissals.clone();
            let other_clicks = self.other_clicks.clone();
            let dialog = Dialog::new("dialog", self.open)
                .scrim(scrim())
                .kind(self.kind)
                .aria_label("Discard draft?")
                .w(px(200.))
                .h(px(100.))
                .bg(DIALOG_COLOR())
                .when(self.dismissable, |this| {
                    this.on_dismiss(
                        cx.listener(move |this, reason: &DismissReason, window, cx| {
                            dismissals.borrow_mut().push(*reason);
                            if this.focus_other_on_dismiss {
                                window.focus(&this.other_focus, cx);
                            }
                            this.open = false;
                            cx.notify();
                        }),
                    )
                })
                .when(self.initial_ok, |this| this.initial_focus(&self.ok_focus))
                .when(self.buttons, |this| {
                    this.child(
                        Button::new("cancel")
                            .track_focus(&self.cancel_focus)
                            .size(px(10.)),
                    )
                    .child(Button::new("ok").track_focus(&self.ok_focus).size(px(10.)))
                })
                .when_some(self.popup, |this, popup| {
                    let request = cx.listener(|this, expanded: &bool, _, cx| {
                        this.popup_expanded = *expanded;
                        cx.notify();
                    });
                    match popup {
                        Popup::Select => this.child(
                            Select::new("inner-select", None::<()>, self.popup_expanded)
                                .on_expanded_change(request)
                                .listbox(SelectListbox::new().option(
                                    SelectOption::new("only", (), "Only").w(px(50.)).h(px(10.)),
                                ))
                                .track_focus(&self.popup_focus)
                                .size(px(10.)),
                        ),
                        Popup::Menu => this.child(
                            Menu::new("inner-menu", self.popup_expanded)
                                .on_expanded_change(request)
                                .item(MenuItem::new("only").w(px(50.)).h(px(10.)))
                                .attach(
                                    Button::new("inner-menu")
                                        .track_focus(&self.popup_focus)
                                        .size(px(10.)),
                                ),
                        ),
                    }
                });
            let second = Dialog::new("second", self.second_open)
                .scrim(scrim())
                .w(px(100.))
                .h(px(50.))
                .on_dismiss(cx.listener(|this, _, _, cx| {
                    this.second_open = false;
                    cx.notify();
                }))
                .child(
                    Button::new("inner")
                        .track_focus(&self.inner_focus)
                        .size(px(10.)),
                );
            let menu = Menu::new("menu", self.menu_expanded)
                .on_expanded_change(cx.listener(|this, expanded, _, cx| {
                    this.menu_expanded = *expanded;
                    cx.notify();
                }))
                .item(
                    MenuItem::new("delete")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open = true;
                            cx.notify();
                        }))
                        .w(px(100.))
                        .h(px(20.)),
                );
            div()
                .size_full()
                .child(
                    Button::new("opener")
                        .track_focus(&self.opener_focus)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.open = true;
                            cx.notify();
                        }))
                        .w(px(40.))
                        .h(px(20.)),
                )
                .child(
                    Button::new("other")
                        .track_focus(&self.other_focus)
                        .on_click(move |_, _, _| *other_clicks.borrow_mut() += 1)
                        .absolute()
                        .left(px(100.))
                        .top(px(0.))
                        .w(px(40.))
                        .h(px(20.)),
                )
                .child(
                    menu.attach(
                        Button::new("menu")
                            .track_focus(&self.menu_focus)
                            .absolute()
                            .left(px(200.))
                            .top(px(0.))
                            .w(px(40.))
                            .h(px(20.)),
                    ),
                )
                .child(dialog)
                .child(second)
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<DialogView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| DialogView {
            popup: None,
            popup_expanded: false,
            popup_focus: cx.focus_handle(),
            open: false,
            second_open: false,
            menu_expanded: false,
            buttons: true,
            dismissable: true,
            initial_ok: false,
            focus_other_on_dismiss: false,
            kind: DialogKind::Dialog,
            dismissals: Rc::default(),
            other_clicks: Rc::default(),
            opener_focus: cx.focus_handle(),
            other_focus: cx.focus_handle(),
            cancel_focus: cx.focus_handle(),
            ok_focus: cx.focus_handle(),
            inner_focus: cx.focus_handle(),
            menu_focus: cx.focus_handle(),
        });
        cx.simulate_resize(size(px(500.), px(400.)));
        (view, cx)
    }

    fn update(
        view: &Entity<DialogView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut DialogView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn open(view: &Entity<DialogView>, cx: &mut VisualTestContext) -> bool {
        view.read_with(cx, |view, _| view.open)
    }

    fn dismissals(view: &Entity<DialogView>, cx: &mut VisualTestContext) -> Vec<DismissReason> {
        view.read_with(cx, |view, _| view.dismissals.borrow().clone())
    }

    /// フォーカスのある要素の名前
    fn focused(view: &Entity<DialogView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| {
            [
                ("opener", view.opener_focus.clone()),
                ("other", view.other_focus.clone()),
                ("cancel", view.cancel_focus.clone()),
                ("ok", view.ok_focus.clone()),
                ("inner", view.inner_focus.clone()),
                ("menu", view.menu_focus.clone()),
                ("popup", view.popup_focus.clone()),
            ]
        });
        cx.update(|window, _| {
            handles
                .iter()
                .find(|(_, handle)| handle.is_focused(window))
                .map(|(name, _)| *name)
        })
    }

    fn press_enter(cx: &mut VisualTestContext) {
        let keystroke = Keystroke::parse("enter").unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    /// 開くボタンを Tab でフォーカスしてから、Enter で開く
    fn open_with_keyboard(cx: &mut VisualTestContext) {
        cx.simulate_keystrokes("tab");
        press_enter(cx);
    }

    fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
        cx.simulate_click(point(px(x), px(y)), Modifiers::none());
    }

    /// ダイアログの x, y, 幅, 高さ。描いていなければ `None`
    fn dialog_bounds(cx: &mut VisualTestContext) -> Option<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.background == DIALOG_COLOR().into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
        })
    }

    #[gpui::test]
    fn 開くと幕のレイアウトで置き最初の要素にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        assert_eq!(dialog_bounds(cx), None);
        click(cx, 10., 10.);
        assert!(open(&view, cx));
        assert_eq!(dialog_bounds(cx), Some((150., 150., 200., 100.)));
        assert_eq!(focused(&view, cx), Some("cancel"));
    }

    #[gpui::test]
    fn tab_で到達できる要素が無ければダイアログにフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.buttons = false);
        open_with_keyboard(cx);
        assert!(open(&view, cx));
        assert_eq!(focused(&view, cx), None);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None, "Tab で背後の要素に移らないこと");
        cx.simulate_keystrokes("escape");
        assert!(!open(&view, cx), "ダイアログがキーを受けること");
        assert_eq!(focused(&view, cx), Some("opener"));
    }

    #[gpui::test]
    fn tab_と_shift_tab_はダイアログの中だけを巡る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("ok"));
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("cancel"));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("ok"));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("cancel"));
    }

    #[gpui::test]
    fn ダイアログにフォーカスがあっても_tab_と_shift_tab_で中の要素に移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        // ダイアログの中の、ボタンの無い場所
        click(cx, 300., 240.);
        assert_eq!(focused(&view, cx), None);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("cancel"));
        click(cx, 300., 240.);
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("ok"));
    }

    #[gpui::test]
    fn initial_focus_の要素に最初のフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.initial_ok = true);
        click(cx, 10., 10.);
        assert_eq!(focused(&view, cx), Some("ok"));
    }

    #[gpui::test]
    fn メニューの項目から開いて閉じるとメニューのボタンにフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 210., 10.);
        assert!(view.read_with(cx, |view, _| view.menu_expanded));
        press_enter(cx);
        assert!(open(&view, cx));
        assert!(!view.read_with(cx, |view, _| view.menu_expanded));
        assert_eq!(focused(&view, cx), Some("cancel"));
        cx.simulate_keystrokes("escape");
        assert!(!open(&view, cx));
        assert_eq!(focused(&view, cx), Some("menu"));
    }

    #[gpui::test]
    fn 閉じるまでに別の要素へフォーカスが移されていれば戻さない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.focus_other_on_dismiss = true);
        click(cx, 10., 10.);
        cx.simulate_keystrokes("escape");
        assert!(!open(&view, cx));
        assert_eq!(focused(&view, cx), Some("other"));
    }

    #[gpui::test]
    fn 閉じるのと同時に別のダイアログが開くと_開いた方にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.simulate_keystrokes("tab");
        update(&view, cx, |view| view.second_open = true);
        assert_eq!(focused(&view, cx), Some("inner"));
        update(&view, cx, |view| {
            view.second_open = false;
            view.open = true;
        });
        assert_eq!(focused(&view, cx), Some("cancel"));
    }

    #[gpui::test]
    fn escape_で閉じるよう求め_開く前の要素にフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        open_with_keyboard(cx);
        assert_eq!(focused(&view, cx), Some("cancel"));
        cx.simulate_keystrokes("shift-escape");
        assert!(open(&view, cx), "修飾キー付きでは閉じないこと");
        cx.simulate_keystrokes("escape");
        assert!(!open(&view, cx));
        assert_eq!(dismissals(&view, cx), [DismissReason::Escape]);
        assert_eq!(dialog_bounds(cx), None);
        assert_eq!(focused(&view, cx), Some("opener"));
    }

    #[gpui::test]
    fn 幕のクリックで閉じるよう求め_ダイアログの上のクリックでは求めない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        click(cx, 300., 240.);
        assert!(open(&view, cx));
        assert_eq!(dismissals(&view, cx), []);
        click(cx, 10., 300.);
        assert!(!open(&view, cx));
        assert_eq!(dismissals(&view, cx), [DismissReason::Scrim]);
    }

    #[gpui::test]
    fn 幕の上のクリックは背後の要素に届かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        click(cx, 110., 10.);
        let other_clicks = view.read_with(cx, |view, _| *view.other_clicks.borrow());
        assert_eq!(other_clicks, 0);
        assert_ne!(focused(&view, cx), Some("other"));
    }

    #[gpui::test]
    fn on_dismiss_が無ければ_escape_と幕のクリックでは閉じない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        update(&view, cx, |view| view.dismissable = false);
        click(cx, 10., 10.);
        cx.simulate_keystrokes("escape");
        click(cx, 10., 300.);
        assert!(open(&view, cx));
        assert_eq!(focused(&view, cx), Some("cancel"));
    }

    #[gpui::test]
    fn フォーカスが外へ移されるとダイアログに戻す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        let other = view.read_with(cx, |view, _| view.other_focus.clone());
        cx.update(|window, cx| window.focus(&other, cx));
        cx.run_until_parked();
        assert_eq!(focused(&view, cx), None, "ダイアログにフォーカスがあること");
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("cancel"));
    }

    #[gpui::test]
    fn 手前に開いたダイアログがフォーカスを持ち_閉じると奥のダイアログに戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        click(cx, 10., 10.);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("ok"));
        update(&view, cx, |view| view.second_open = true);
        assert_eq!(focused(&view, cx), Some("inner"));
        // 描き直しても、奥のダイアログがフォーカスを取り戻さない
        update(&view, cx, |_| {});
        assert_eq!(focused(&view, cx), Some("inner"));
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("inner"));
        cx.simulate_keystrokes("escape");
        assert!(open(&view, cx), "奥のダイアログは閉じないこと");
        assert!(!view.read_with(cx, |view, _| view.second_open));
        assert_eq!(focused(&view, cx), Some("ok"));
    }

    #[gpui::test]
    fn 種類に応じた_role_の_modal_として公開される(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx);
        let cases = [
            (DialogKind::Dialog, accesskit::Role::Dialog),
            (DialogKind::Alert, accesskit::Role::AlertDialog),
        ];
        for (kind, role) in cases {
            let node = cx.update(|_, cx| {
                let inner = div().id("dialog").aria_label("Discard draft?");
                let element = dialog_element(inner, kind, &cx.focus_handle(), None).into_element();
                let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
                element.write_a11y_info(&mut node);
                node
            });
            assert_eq!(node.role(), role);
            assert!(node.is_modal());
            assert_eq!(node.label(), Some("Discard draft?"));
        }
    }

    #[gpui::test]
    fn 開いている間だけ開いているダイアログがあると判定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        assert!(!cx.update(|_, cx| any_open(cx)));
        update(&view, cx, |view| view.open = true);
        assert!(cx.update(|_, cx| any_open(cx)));
        update(&view, cx, |view| view.open = false);
        assert!(!cx.update(|_, cx| any_open(cx)));
    }

    #[gpui::test]
    fn ダイアログの中で開いた一覧から_tab_で出てもダイアログの中を巡る(
        cx: &mut TestAppContext,
    ) {
        for popup in [Popup::Select, Popup::Menu] {
            let (view, cx) = setup(cx);
            update(&view, cx, |view| {
                view.popup = Some(popup);
                view.open = true;
            });
            cx.simulate_keystrokes("tab tab");
            assert_eq!(focused(&view, cx), Some("popup"));
            cx.simulate_keystrokes("down");
            assert!(view.read_with(cx, |view, _| view.popup_expanded));
            cx.simulate_keystrokes("tab");
            cx.run_until_parked();
            assert!(!view.read_with(cx, |view, _| view.popup_expanded));
            assert_eq!(
                focused(&view, cx),
                Some("cancel"),
                "最後の要素の次はダイアログの最初の要素に戻ること"
            );
        }
    }
}
