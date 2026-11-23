//! 円の上に並べた項目から 1 つを選ぶ、見た目を持たない文字盤

use std::{cell::Cell, f32::consts::TAU, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement,
    Interactivity, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

use crate::{
    Availability,
    pointer::{DragHandler, PointerLayer, PointerState},
    roving::{self, GroupChild, RovingItem},
};

type ChangeHandler<T> = Rc<dyn Fn(&T, &mut Window, &mut App)>;
type PointerHandler = Rc<dyn Fn(&DialPointer, &mut Window, &mut App)>;
type SelectHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// [`ClockDial::on_pointer`] に渡す、ポインタの操作
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DialPointer {
    /// 文字盤の中心から見たポインタの向き。真上を 0 として時計回りに増えるラジアンで、0 以上 2π 未満
    pub angle: f32,
    /// 文字盤の中心からポインタまでの距離
    pub distance: Pixels,
    /// 操作の段階
    pub phase: DialPointerPhase,
}

/// ポインタの操作の段階
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DialPointerPhase {
    /// 文字盤の上でマウスの左ボタンを押した
    Down,
    /// 押したまま動かした。文字盤の外も含む
    Move,
    /// 押した後にボタンを離した。文字盤の外も含む
    Up,
}

/// 円の上に並べた項目から 1 つを選ぶ、見た目を持たない文字盤 (時計の文字盤など)
///
/// 選ばれている値は呼び出し側が持ち、[`ClockDial::new`] に毎回渡す。文字盤自身は値を変えない。
/// 項目は [`ClockDial::item`] で、項目でない要素は `child` で加える。項目の配置は呼び出し側が決める
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初の項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる
/// - 修飾キーなしの上下左右の矢印キーで、前後の項目へフォーカスだけを移す。端では反対の端に戻る。
///   下と右で次へ、上と左で前へ移る
/// - フォーカス中の Enter・Space と、支援技術からのクリック操作で、項目を選ぶよう [`ClockDial::on_change`] で求める
/// - マウスの左ボタンの押下・ドラッグ・離す操作を、文字盤の中心から見た向きと距離として [`ClockDial::on_pointer`] に渡す。
///   ポインタの位置から値を決めるのは呼び出し側で、項目の上での操作も項目を選ぶ操作にはしない
///
/// 支援技術には role `radiogroup` として公開され、項目は role `radio` と選ばれているかどうかの状態として公開される。
/// 文字盤と項目の名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct ClockDial<T: PartialEq + 'static> {
    id: ElementId,
    inner: Stateful<Div>,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    on_pointer: Option<PointerHandler>,
    children: Vec<GroupChild<ClockDialItem<T>>>,
}

impl<T: PartialEq + 'static> ClockDial<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示する文字盤を作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、ドラッグが途切れる
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            selected,
            on_change: None,
            on_pointer: None,
            children: Vec::new(),
        }
    }

    /// キーボードか支援技術から項目を選ぶよう求められたときに `handler` を呼ぶ。
    /// 複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目でも呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ClockDial::new`] に渡すこと。
    /// 設定しない場合、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// マウスで操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 設定すると、文字盤はマウスの左ボタンの押下を親に伝えない
    pub fn on_pointer(
        mut self,
        handler: impl Fn(&DialPointer, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pointer = Some(Rc::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ClockDialItem<T>) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ClockDialItem<T>>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

/// `bounds` の中心から見た `position` の向きと距離
fn polar(position: Point<Pixels>, bounds: Bounds<Pixels>) -> (f32, Pixels) {
    let center = bounds.center();
    let dx = f32::from(position.x - center.x);
    let dy = f32::from(position.y - center.y);
    // 真上を 0 とし、画面の y は下向きに増えるため時計回りに増える
    let angle = dx.atan2(-dy).rem_euclid(TAU);
    (angle % TAU, Pixels::from(dx.hypot(dy)))
}

impl<T: PartialEq + 'static> RenderOnce for ClockDial<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let pointer = window.use_keyed_state(self.id.clone(), cx, |_, _| PointerState::default());
        let mut values = Vec::new();
        let mut items = Vec::new();
        let children: Vec<GroupChild<Stateful<Div>>> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(item) => {
                    items.push(RovingItem {
                        // 項目の id は文字盤の中でだけ一意なので、文字盤の id で分ける
                        focus_handle: window.with_element_namespace(self.id.clone(), |window| {
                            item.focus_handle(window, cx)
                        }),
                        availability: Availability::Enabled,
                    });
                    values.push(item.value);
                    GroupChild::Item(item.inner)
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
            let values = values.clone();
            Rc::new(move |ix: usize, window: &mut Window, cx: &mut App| {
                on_change(&values[ix], window, cx)
            })
        });

        let mut ix = 0;
        let elements: Vec<AnyElement> = children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(inner) => {
                    let item_ix = ix;
                    ix += 1;
                    item_element(
                        inner,
                        &items[item_ix].focus_handle,
                        selected_ix == Some(item_ix),
                        tab_stop == Some(item_ix),
                        select.clone().map(|select| {
                            Rc::new(move |window: &mut Window, cx: &mut App| {
                                select(item_ix, window, cx)
                            }) as SelectHandler
                        }),
                    )
                    .into_any_element()
                }
                GroupChild::Other(element) => element,
            })
            .collect();

        // ポインタの位置を向きと距離に変えるため、描画のたびに文字盤の位置を記録する
        let bounds = Rc::new(Cell::new(None));
        let handler = |phase: DialPointerPhase| {
            self.on_pointer.clone().map(|on_pointer| {
                Rc::new(
                    move |position: Point<Pixels>,
                          bounds: Bounds<Pixels>,
                          window: &mut Window,
                          cx: &mut App| {
                        let (angle, distance) = polar(position, bounds);
                        on_pointer(
                            &DialPointer {
                                angle,
                                distance,
                                phase,
                            },
                            window,
                            cx,
                        );
                    },
                ) as DragHandler
            })
        };
        let down = handler(DialPointerPhase::Down);
        let inner = self
            .inner
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
                cx.stop_propagation();
            })
            .when_some(down, |this, down| {
                let pointer = pointer.clone();
                let bounds = bounds.clone();
                this.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    // ドラッグで親の on_click や on_drag を動かさないよう、押下を親に伝えない
                    cx.stop_propagation();
                    let Some(bounds) = bounds.get() else {
                        return;
                    };
                    pointer.update(cx, |pointer, _| pointer.dragging = true);
                    down(event.position, bounds, window, cx);
                })
            })
            .children(elements);

        PointerLayer {
            inner,
            bounds,
            drag: handler(DialPointerPhase::Move),
            release: handler(DialPointerPhase::Up),
            pointer,
        }
    }
}

impl<T: PartialEq + 'static> Styled for ClockDial<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T: PartialEq + 'static> ParentElement for ClockDial<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl<T: PartialEq + 'static> InteractiveElement for ClockDial<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T: PartialEq + 'static> StatefulInteractiveElement for ClockDial<T> {}

/// 項目を role `radio` として公開し、Enter・Space と支援技術からのクリック操作で `select` を呼ぶ
fn item_element(
    inner: Stateful<Div>,
    focus_handle: &FocusHandle,
    selected: bool,
    tab_stop: bool,
    select: Option<SelectHandler>,
) -> Stateful<Div> {
    inner
        .role(accesskit::Role::RadioButton)
        .aria_toggled(if selected {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        })
        .track_focus(&focus_handle.clone().tab_stop(tab_stop))
        .when_some(select, |this, select| {
            let on_click = select.clone();
            this.on_click(move |event, window, cx| {
                // マウスの操作は文字盤の on_pointer で扱う
                if let ClickEvent::Keyboard(_) = event {
                    on_click(window, cx);
                }
            })
            .on_a11y_action(accesskit::Action::Click, move |_, window, cx| {
                select(window, cx)
            })
        })
}

/// [`ClockDial`] の項目
///
/// [`ClockDial::item`] に渡して使う。項目が表す値が、文字盤の選ばれている値と等しいときに選ばれた状態になる。
/// 見た目と配置は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
pub struct ClockDialItem<T> {
    id: ElementId,
    inner: Stateful<Div>,
    value: T,
    provided_focus_handle: Option<FocusHandle>,
}

impl<T> ClockDialItem<T> {
    /// `value` を表す項目を作る
    ///
    /// `id` と `value` は文字盤の中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// [`ClockDialItem::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value,
            provided_focus_handle: None,
        }
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab で到達できるかは文字盤が決める
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

impl<T> Styled for ClockDialItem<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<T> ParentElement for ClockDialItem<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<T> InteractiveElement for ClockDialItem<T> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<T> StatefulInteractiveElement for ClockDialItem<T> {}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::f32::consts::{FRAC_PI_2, PI};

    use gpui::{
        Context, Element, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseDownEvent,
        MouseMoveEvent, MouseUpEvent, Render, TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::test_window::add_test_window;

    /// 文字盤は (100, 100) から 200x200。中心は (200, 200)
    struct DialView {
        selected: Option<u8>,
        handles: [FocusHandle; 4],
        changes: Rc<RefCell<Vec<u8>>>,
        pointers: Rc<RefCell<Vec<DialPointer>>>,
        node: Option<accesskit::Node>,
    }

    impl Render for DialView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let pointers = self.pointers.clone();
            let dial = ClockDial::new("dial", self.selected)
                .on_change(cx.listener(|this, value: &u8, _, cx| {
                    this.changes.borrow_mut().push(*value);
                    this.selected = Some(*value);
                    cx.notify();
                }))
                .on_pointer(move |pointer, _, _| pointers.borrow_mut().push(*pointer))
                .aria_label("Hour")
                .absolute()
                .left(px(100.))
                .top(px(100.))
                .size(px(200.))
                .items((0..4u8).map(|value| {
                    ClockDialItem::new(("item", value as usize), value)
                        .track_focus(&self.handles[value as usize])
                        .aria_label(format!("{value}"))
                        .size(px(10.))
                }))
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(dial.a11y_role().expect("role を持つこと"));
            dial.write_a11y_info(&mut node);
            self.node = Some(node);
            div().size_full().child(dial)
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selected: Option<u8>,
    ) -> (gpui::Entity<DialView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| DialView {
            selected,
            handles: std::array::from_fn(|_| cx.focus_handle()),
            changes: Rc::default(),
            pointers: Rc::default(),
            node: None,
        });
        cx.run_until_parked();
        (view, cx)
    }

    fn press(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        cx.run_until_parked();
    }

    fn focused(view: &gpui::Entity<DialView>, cx: &mut VisualTestContext) -> Option<usize> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| handles.iter().position(|handle| handle.is_focused(window)))
    }

    #[gpui::test]
    fn tab_では選ばれている項目に入り_矢印キーではフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2));
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(2));
        cx.simulate_keystrokes("down right");
        assert_eq!(focused(&view, cx), Some(0), "端では反対の端に戻る");
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some(3));
        assert!(view.read_with(cx, |view, _| view.changes.borrow().is_empty()));
        // Enter と Space はキーを離したときに選ぶ
        press(cx, "enter");
        cx.simulate_keystrokes("left");
        press(cx, "space");
        assert_eq!(
            *view.read_with(cx, |view, _| view.changes.clone()).borrow(),
            [3, 2]
        );
    }

    #[gpui::test]
    fn ポインタの操作を中心から見た向きと距離で渡す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None);
        cx.simulate_event(MouseDownEvent {
            position: point(px(200.), px(150.)),
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
            click_count: 1,
            first_mouse: false,
        });
        cx.simulate_event(MouseMoveEvent {
            position: point(px(400.), px(200.)),
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        });
        cx.simulate_event(MouseUpEvent {
            position: point(px(200.), px(230.)),
            button: MouseButton::Left,
            modifiers: Modifiers::none(),
            click_count: 1,
        });
        let pointers = view.read_with(cx, |view, _| view.pointers.borrow().clone());
        let summary: Vec<_> = pointers
            .iter()
            .map(|pointer| {
                (
                    (pointer.angle * 1000.).round() / 1000.,
                    f32::from(pointer.distance),
                    pointer.phase,
                )
            })
            .collect();
        let round = |angle: f32| (angle * 1000.).round() / 1000.;
        assert_eq!(
            summary,
            [
                (0., 50., DialPointerPhase::Down),
                (round(FRAC_PI_2), 200., DialPointerPhase::Move),
                (round(PI), 30., DialPointerPhase::Up),
            ],
            "文字盤の外への移動と、外で離す操作も渡す"
        );
        assert!(
            view.read_with(cx, |view, _| view.changes.borrow().is_empty()),
            "ポインタの操作では選ぶよう求めない"
        );
    }

    #[gpui::test]
    fn radiogroup_として公開する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(1));
        let node = view
            .read_with(cx, |view, _| view.node.clone())
            .expect("描画されていること");
        assert_eq!(node.role(), accesskit::Role::RadioGroup);
        assert_eq!(node.label(), Some("Hour"));
    }

    /// 項目だけを描画したときの a11y ノード
    fn item_node(cx: &mut VisualTestContext, selected: bool, has_handler: bool) -> accesskit::Node {
        cx.update(|_, cx| {
            let element = item_element(
                div().id("item").aria_label("3 o'clock"),
                &cx.focus_handle(),
                selected,
                true,
                has_handler.then(|| Rc::new(|_: &mut Window, _: &mut App| {}) as SelectHandler),
            )
            .into_element();
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            node
        })
    }

    #[gpui::test]
    fn 項目は_role_radio_と選ばれているかどうかで公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None);
        for (selected, toggled) in [
            (false, accesskit::Toggled::False),
            (true, accesskit::Toggled::True),
        ] {
            let node = item_node(cx, selected, true);
            assert_eq!(node.role(), accesskit::Role::RadioButton);
            assert_eq!(node.label(), Some("3 o'clock"));
            assert_eq!(node.toggled(), Some(toggled));
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(node.supports_action(accesskit::Action::Focus));
        }
    }

    #[gpui::test]
    fn ハンドラを持たない文字盤の項目はクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, None);
        let node = item_node(cx, false, false);
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }
}
