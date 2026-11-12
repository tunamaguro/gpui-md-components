//! 範囲の中から値を選ぶ、見た目を持たない部品

use std::{cell::Cell, ops::RangeInclusive, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Div, Element, ElementId, Entity, FocusHandle,
    GlobalElementId, InspectorElementId, InteractiveElement, Interactivity, IntoElement, LayoutId,
    MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, SharedString,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder, px,
};

use crate::{Availability, InteractionSource, accessibility::A11yInfoExt};

type ChangeHandler = Rc<dyn Fn(&f32, &mut Window, &mut App)>;

/// 値の範囲と段階
#[derive(Debug, Clone, Copy)]
struct Scale {
    start: f32,
    end: f32,
    step: Option<f32>,
}

impl Scale {
    /// `value` を範囲に収め、段階があれば、範囲の始まりから段階ごとの値と範囲の終わりのうち最も近い値にする
    fn snap(self, value: f32) -> f32 {
        // 始まりが終わりより大きい範囲でも panic しないよう、clamp ではなく max と min で収める
        let value = value.max(self.start).min(self.end);
        let Some(step) = self.step else {
            return value;
        };
        let snapped = (self.start + ((value - self.start) / step).round() * step).min(self.end);
        if self.end - value < (value - snapped).abs() {
            self.end
        } else {
            snapped
        }
    }

    /// 矢印キーで増減する量。段階が無ければ範囲の 1%
    fn key_delta(self) -> f32 {
        self.step.unwrap_or((self.end - self.start) / 100.0)
    }

    /// Page Up と Page Down で増減する量。範囲の段階の数の 1/10 を、2 から 10 段階に収めた量
    fn page_delta(self) -> f32 {
        let delta = self.key_delta();
        let count = if delta > 0.0 {
            ((self.end - self.start) / delta).round()
        } else {
            0.0
        };
        // APG は Page Up と Page Down で矢印キーより大きく変えるよう求めるため、最小を 2 段階にする
        (count / 10.0).floor().clamp(2.0, 10.0) * delta
    }

    /// `bounds` の左右の端から `inset` だけ内側の範囲で、位置 `x` に対応する値
    fn value_at(self, bounds: Bounds<Pixels>, inset: Pixels, x: Pixels) -> f32 {
        let width = bounds.size.width - inset * 2.0;
        let fraction = if width > px(0.0) {
            ((x - bounds.left() - inset) / width).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.snap(self.start + (self.end - self.start) * fraction)
    }
}

/// ポインタで値を選ぶための状態。描画をまたいで保つ
#[derive(Default)]
struct PointerState {
    /// 左ボタンを部品の上で押してから離すまでの間か
    dragging: bool,
}

/// 範囲の中から値を選ぶ、見た目を持たないスライダー
///
/// 値は呼び出し側が持ち、[`Slider::new`] に毎回渡す。部品自身は値を変えない。
/// 次の操作で、[`Slider::on_change`] のハンドラに新しい値を求める
///
/// - マウスの左ボタンを押した位置の値。押したまま動かすと、ボタンを離すまで部品の外でも位置を追う。
///   値を変えられる部品は、左ボタンの押下を親に伝えない
/// - フォーカス中の右と上の矢印キーで 1 段階増やし、左と下の矢印キーで 1 段階減らす。
///   段階が無ければ範囲の 1% だけ増減する
/// - Page Up と Page Down で、範囲の段階の数の 1/10 を 2 から 10 段階に収めた量だけ増減する。
///   段階が無ければ範囲の 10% だけ増減する
/// - Home で最小の値、End で最大の値にする
/// - 支援技術からの増減 (Increment・Decrement) と値の設定 (SetValue)
///
/// 求める値は範囲に収め、段階があれば、範囲の始まりから段階ごとの値と範囲の終わりのうち最も近い値にする。
/// 今の値と同じになる場合は求めない
///
/// 支援技術には role `slider` と、値・最小・最大・段階・横向きとして公開される。
/// アクセシブルな名前は `aria_label` で与えること。子の内容は名前にならない。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    inner: Stateful<Div>,
    value: f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    track_inset: Pixels,
    on_change: Option<ChangeHandler>,
    provided_focus_handle: Option<FocusHandle>,
    availability: Availability,
    group: Option<SharedString>,
}

impl Slider {
    /// `value` を表示するスライダーを作る。範囲は 0 から 1 で、段階を持たない
    ///
    /// 範囲の外の値は、範囲の端の値として公開する。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Slider::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value,
            range: 0.0..=1.0,
            step: None,
            track_inset: px(0.0),
            on_change: None,
            provided_focus_handle: None,
            availability: Availability::Enabled,
            group: None,
        }
    }

    /// 選べる値の範囲を設定する。始まりは終わり以下にすること
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// 選べる値を、範囲の始まりから `step` ごとの値と範囲の終わりに限る。既定では範囲のどの値も選べる
    ///
    /// 0 以下か NaN を渡すと段階を持たない
    pub fn step(mut self, step: f32) -> Self {
        self.step = (step > 0.0).then_some(step);
        self
    }

    /// ポインタの位置を値に対応させる範囲を、部品の左右の端から `inset` だけ内側にする。既定は 0
    ///
    /// 内側の範囲の左端が最小の値、右端が最大の値になり、その外側は端の値になる。
    /// つまみの中心で値を示し、つまみを部品からはみ出させない場合に、つまみの幅の半分を渡す
    pub fn track_inset(mut self, inset: Pixels) -> Self {
        self.track_inset = inset;
        self
    }

    /// 値を変える操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には求められた値を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`Slider::new`] に渡すこと。
    /// 設定しない場合、部品は操作されても何も呼ばず、支援技術に値を変える操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 部品を無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な部品は操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Slider::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 部品を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な部品は操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Slider::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
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
    /// [`Slider::track_focus`] と両方を呼んだ場合は、後に呼んだ方のフォーカスを使う
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

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.focus_handle(window, cx);
        let pointer = window.use_keyed_state(self.id.clone(), cx, |_, _| PointerState::default());
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let scale = Scale {
            start: *self.range.start(),
            end: *self.range.end(),
            step: self.step,
        };
        let value = self.value.max(scale.start).min(scale.end);
        // 求める値が今の値と同じなら呼ばない
        let request = self.on_change.filter(|_| !disabled).map(|on_change| {
            Rc::new(move |next: f32, window: &mut Window, cx: &mut App| {
                if next != value {
                    on_change(&next, window, cx);
                }
            })
        });
        // ポインタの位置を値に変えるため、描画のたびに部品の位置を記録する
        let bounds = Rc::new(Cell::new(None));
        let inset = self.track_inset;

        let inner = self
            .inner
            .role(accesskit::Role::Slider)
            .aria_numeric_value(f64::from(value))
            .aria_min_numeric_value(f64::from(scale.start))
            .aria_max_numeric_value(f64::from(scale.end))
            .when_some(scale.step, |this, step| {
                this.aria_numeric_value_step(f64::from(step))
            })
            .aria_orientation(accesskit::Orientation::Horizontal)
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
                this.track_focus(&focus_handle.clone().tab_stop(true))
            })
            .when_some(request.clone(), |this, request| {
                let on_mouse_down = {
                    let request = request.clone();
                    let pointer = pointer.clone();
                    let bounds = bounds.clone();
                    let focus_handle = focus_handle.clone();
                    move |event: &gpui::MouseDownEvent, window: &mut Window, cx: &mut App| {
                        // ドラッグで親の on_click や on_drag を動かさないよう、押下を親に伝えない。
                        // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                        cx.stop_propagation();
                        window.focus(&focus_handle, cx);
                        let Some(bounds) = bounds.get() else {
                            return;
                        };
                        pointer.update(cx, |pointer, _| pointer.dragging = true);
                        request(scale.value_at(bounds, inset, event.position.x), window, cx);
                    }
                };
                let on_key_down = {
                    let request = request.clone();
                    move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
                        if event.keystroke.modifiers.modified() {
                            return;
                        }
                        let next = match event.keystroke.key.as_str() {
                            "right" | "up" => value + scale.key_delta(),
                            "left" | "down" => value - scale.key_delta(),
                            "pageup" => value + scale.page_delta(),
                            "pagedown" => value - scale.page_delta(),
                            "home" => scale.start,
                            "end" => scale.end,
                            _ => return,
                        };
                        cx.stop_propagation();
                        request(scale.snap(next), window, cx);
                    }
                };
                let increment = request.clone();
                let decrement = request.clone();
                this.on_mouse_down(MouseButton::Left, on_mouse_down)
                    .on_key_down(on_key_down)
                    .on_a11y_action(accesskit::Action::Increment, move |_, window, cx| {
                        increment(scale.snap(value + scale.key_delta()), window, cx);
                    })
                    .on_a11y_action(accesskit::Action::Decrement, move |_, window, cx| {
                        decrement(scale.snap(value - scale.key_delta()), window, cx);
                    })
                    .on_a11y_action(accesskit::Action::SetValue, move |data, window, cx| {
                        if let Some(accesskit::ActionData::NumericValue(next)) = data {
                            request(scale.snap(*next as f32), window, cx);
                        }
                    })
            })
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
            });

        PointerLayer {
            inner,
            bounds,
            drag: request.map(|request| {
                Rc::new(
                    move |x: Pixels, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App| {
                        request(scale.value_at(bounds, inset, x), window, cx);
                    },
                ) as DragHandler
            }),
            pointer,
        }
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Slider {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for Slider {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Slider {}

/// ドラッグ中のポインタの位置 `x` と、部品の位置を受け取るハンドラ
type DragHandler = Rc<dyn Fn(Pixels, Bounds<Pixels>, &mut Window, &mut App)>;

/// `inner` の位置を記録し、ドラッグ中のマウスの移動とボタンを離す操作を、部品の外でも受ける要素
struct PointerLayer<E> {
    inner: E,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// 値を変えられない場合は `None`
    drag: Option<DragHandler>,
    pointer: Entity<PointerState>,
}

impl<E: Element> IntoElement for PointerLayer<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element> Element for PointerLayer<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.bounds.set(Some(bounds));
        self.inner
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
        // gpui の要素のマウスのリスナーは、ポインタが要素の上にある間しか移動を受けないため、ウィンドウ全体で受ける。
        // ほかの要素が伝播を止めても受けるよう Capture で受け、ドラッグ中に値を変えられなくなっても押下の状態を戻す
        let drag = self.drag.clone();
        let pointer = self.pointer.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture || !pointer.read(cx).dragging {
                return;
            }
            // ウィンドウの外でボタンを離した場合は離す操作が届かないため、移動のときにも確かめる
            if event.pressed_button != Some(MouseButton::Left) {
                pointer.update(cx, |pointer, _| pointer.dragging = false);
                return;
            }
            if let Some(drag) = &drag {
                drag(event.position.x, bounds, window, cx);
            }
        });
        let pointer = self.pointer.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture
                && event.button == MouseButton::Left
                && pointer.read(cx).dragging
            {
                pointer.update(cx, |pointer, _| pointer.dragging = false);
            }
        });
    }

    fn a11y_role(&self) -> Option<accesskit::Role> {
        self.inner.a11y_role()
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.inner.write_a11y_info(node);
    }
}

/// APG の Slider パターン (content/patterns/slider/slider-pattern.html) で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point};

    use super::*;
    use crate::test_window::add_test_window;

    /// 部品の幅。ポインタの位置を値に対応させる範囲は、左右の端から [`INSET`] だけ内側の 200px
    const WIDTH: f32 = 220.0;
    const INSET: f32 = 10.0;

    struct SliderView {
        value: f32,
        range: RangeInclusive<f32>,
        step: Option<f32>,
        availability: Availability,
        has_handler: bool,
        changes: Rc<RefCell<Vec<f32>>>,
        parent_mouse_downs: Rc<Cell<usize>>,
        focus_handle: FocusHandle,
        node: Option<accesskit::Node>,
    }

    impl Render for SliderView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let element = Slider::new("slider", self.value)
                .range(self.range.clone())
                .when_some(self.step, |this, step| this.step(step))
                .track_inset(px(INSET))
                .availability(self.availability)
                .track_focus(&self.focus_handle)
                .aria_label("Volume")
                .when(self.has_handler, |this| {
                    this.on_change(cx.listener(|view, value: &f32, _, cx| {
                        view.changes.borrow_mut().push(*value);
                        view.value = *value;
                        cx.notify();
                    }))
                })
                .w(px(WIDTH))
                .h(px(20.))
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            self.node = Some(node);
            let parent_mouse_downs = self.parent_mouse_downs.clone();
            div()
                .size_full()
                .on_mouse_down(MouseButton::Left, move |_, _, _| {
                    parent_mouse_downs.set(parent_mouse_downs.get() + 1)
                })
                .child(element)
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        value: f32,
        range: RangeInclusive<f32>,
        step: Option<f32>,
    ) -> (Entity<SliderView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| SliderView {
            value,
            range,
            step,
            availability: Availability::Enabled,
            has_handler: true,
            changes: Rc::default(),
            parent_mouse_downs: Rc::default(),
            focus_handle: cx.focus_handle(),
            node: None,
        })
    }

    fn update(
        view: &Entity<SliderView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut SliderView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn changes(view: &Entity<SliderView>, cx: &mut VisualTestContext) -> Vec<f32> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn node(view: &Entity<SliderView>, cx: &mut VisualTestContext) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画済みであること")
    }

    fn focus(view: &Entity<SliderView>, cx: &mut VisualTestContext) {
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.update(|window, cx| window.focus(&focus_handle, cx));
    }

    /// 部品の左端からの位置 `x` (縦は中央)
    fn at(x: f32) -> gpui::Point<Pixels> {
        point(px(x), px(10.))
    }

    #[gpui::test]
    fn 右と上の矢印キーで増やし左と下の矢印キーで減らす(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, Some(10.0));
        focus(&view, cx);
        cx.simulate_keystrokes("right up left down down");
        assert_eq!(changes(&view, cx), vec![60.0, 70.0, 60.0, 50.0, 40.0]);
    }

    #[gpui::test]
    fn 段階が無ければ矢印キーで範囲の_1_パーセントずつ増減する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 100.0, 0.0..=200.0, None);
        focus(&view, cx);
        cx.simulate_keystrokes("right left left");
        assert_eq!(changes(&view, cx), vec![102.0, 100.0, 98.0]);
    }

    /// 増減する段階の数は、段階の数の 1/10 を 2 から 10 に収めた数。
    /// Compose の `slideOnKeyEvents` は最小を 1 にするが、APG に合わせて矢印キーより大きく変える
    #[gpui::test]
    fn page_up_と_page_down_で段階の数の_10_分の_1_だけ増減する(
        cx: &mut TestAppContext,
    ) {
        for (range, step, expected) in [
            // 100 段階なら 10 段階
            (0.0..=100.0, Some(1.0), [60.0, 50.0]),
            // 20 段階なら 2 段階
            (0.0..=100.0, Some(5.0), [60.0, 50.0]),
            // 10 段階でも 2 段階
            (0.0..=100.0, Some(10.0), [70.0, 50.0]),
            // 4 段階でも 2 段階
            (0.0..=100.0, Some(25.0), [100.0, 50.0]),
            // 1000 段階でも 10 段階
            (0.0..=1000.0, Some(1.0), [60.0, 50.0]),
            // 段階が無ければ範囲の 10%
            (0.0..=100.0, None, [60.0, 50.0]),
        ] {
            let (view, cx) = setup(cx, 50.0, range.clone(), step);
            focus(&view, cx);
            cx.simulate_keystrokes("pageup pagedown");
            assert_eq!(changes(&view, cx), expected, "{range:?} {step:?}");
        }
    }

    #[gpui::test]
    fn home_で最小の値に_end_で最大の値にする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 50.0, 20.0..=80.0, Some(5.0));
        focus(&view, cx);
        cx.simulate_keystrokes("home end");
        assert_eq!(changes(&view, cx), vec![20.0, 80.0]);
    }

    #[gpui::test]
    fn 端では範囲の外の値を求めず今の値と同じ値も求めない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 100.0, 0.0..=100.0, Some(10.0));
        focus(&view, cx);
        cx.simulate_keystrokes("right pageup end");
        assert_eq!(changes(&view, cx), Vec::<f32>::new());
        cx.simulate_keystrokes("left");
        assert_eq!(changes(&view, cx), vec![90.0]);
    }

    #[gpui::test]
    fn 修飾キー付きの矢印キーでは値を変えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, Some(10.0));
        focus(&view, cx);
        cx.simulate_keystrokes("shift-right ctrl-left alt-home");
        assert_eq!(changes(&view, cx), Vec::<f32>::new());
    }

    #[gpui::test]
    fn 押した位置の値を求め押したまま動かすと部品の外でも追う(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 0.0, 0.0..=100.0, None);
        // 内側の範囲の 1/4
        cx.simulate_mouse_down(at(INSET + 50.0), MouseButton::Left, Modifiers::none());
        // 部品の右の外
        cx.simulate_mouse_move(at(WIDTH + 100.0), MouseButton::Left, Modifiers::none());
        // 部品の左の外
        cx.simulate_mouse_move(
            point(px(INSET), px(300.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.simulate_mouse_up(at(INSET), MouseButton::Left, Modifiers::none());
        // 離した後は追わない
        cx.simulate_mouse_move(at(INSET + 100.0), None, Modifiers::none());
        assert_eq!(changes(&view, cx), vec![25.0, 100.0, 0.0]);
    }

    #[gpui::test]
    fn 押下を親に伝えずフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 0.0, 0.0..=100.0, None);
        cx.simulate_click(at(INSET + 100.0), Modifiers::none());
        assert_eq!(changes(&view, cx), vec![50.0]);
        assert_eq!(
            view.read_with(cx, |view, _| view.parent_mouse_downs.get()),
            0
        );
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        assert!(cx.update(|window, _| focus_handle.is_focused(window)));
    }

    #[gpui::test]
    fn ボタンを押していない移動が来たらドラッグを終える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 0.0, 0.0..=100.0, None);
        cx.simulate_mouse_down(at(INSET + 50.0), MouseButton::Left, Modifiers::none());
        // ウィンドウの外でボタンを離し、押さずに戻ってきた
        cx.simulate_mouse_move(at(INSET + 100.0), None, Modifiers::none());
        // 部品の外で別の押下を始めた
        cx.simulate_mouse_move(
            point(px(INSET + 150.0), px(300.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert_eq!(changes(&view, cx), vec![25.0]);
    }

    #[gpui::test]
    fn ドラッグ中に無効になってもボタンを離せばドラッグを終える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 0.0, 0.0..=100.0, None);
        cx.simulate_mouse_down(at(INSET + 50.0), MouseButton::Left, Modifiers::none());
        update(&view, cx, |view| view.availability = Availability::Disabled);
        cx.simulate_mouse_up(at(INSET + 50.0), MouseButton::Left, Modifiers::none());
        update(&view, cx, |view| view.availability = Availability::Enabled);
        // 部品の外で別の押下を始めた
        cx.simulate_mouse_move(
            point(px(INSET + 150.0), px(300.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert_eq!(changes(&view, cx), vec![25.0]);
    }

    #[gpui::test]
    fn 押した位置の値は最も近い段階の値にする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 0.0, 0.0..=100.0, Some(10.0));
        // 内側の範囲の 26% と 34%
        cx.simulate_click(at(INSET + 52.0), Modifiers::none());
        cx.simulate_click(at(INSET + 68.0), Modifiers::none());
        assert_eq!(changes(&view, cx), vec![30.0]);
    }

    #[gpui::test]
    fn 段階で割り切れない範囲でも終わりの値を選べる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 8.0, 0.0..=10.0, Some(4.0));
        focus(&view, cx);
        cx.simulate_keystrokes("right left home end");
        // 内側の範囲の 95% と 85%
        cx.simulate_click(at(INSET + 190.0), Modifiers::none());
        cx.simulate_click(at(INSET + 170.0), Modifiers::none());
        assert_eq!(changes(&view, cx), vec![10.0, 8.0, 0.0, 10.0, 8.0]);
    }

    #[gpui::test]
    fn 無効な部品は操作されても値を求めず押下を親に伝えない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, Some(10.0));
        update(&view, cx, |view| view.availability = Availability::Disabled);
        cx.simulate_click(at(INSET), Modifiers::none());
        focus(&view, cx);
        cx.simulate_keystrokes("right home");
        assert_eq!(changes(&view, cx), Vec::<f32>::new());
        assert_eq!(
            view.read_with(cx, |view, _| view.parent_mouse_downs.get()),
            0
        );
    }

    #[gpui::test]
    fn tab_で到達でき無効なら到達できない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, None);
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.simulate_keystrokes("tab");
        assert!(cx.update(|window, _| focus_handle.is_focused(window)));

        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, None);
        update(&view, cx, |view| view.availability = Availability::Disabled);
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        cx.simulate_keystrokes("tab");
        assert!(!cx.update(|window, _| focus_handle.is_focused(window)));
    }

    #[gpui::test]
    fn 値と範囲と段階を横向きの_slider_として公開する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 30.0, 0.0..=100.0, Some(10.0));
        let node = node(&view, cx);
        assert_eq!(node.role(), accesskit::Role::Slider);
        assert_eq!(node.label(), Some("Volume"));
        assert_eq!(node.numeric_value(), Some(30.0));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert_eq!(node.numeric_value_step(), Some(10.0));
        assert_eq!(node.orientation(), Some(accesskit::Orientation::Horizontal));
        assert!(!node.is_disabled());
        for action in [
            accesskit::Action::Increment,
            accesskit::Action::Decrement,
            accesskit::Action::SetValue,
            accesskit::Action::Focus,
        ] {
            assert!(node.supports_action(action), "{action:?}");
        }
    }

    #[gpui::test]
    fn 範囲の外の値は端の値として公開する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, 150.0, 0.0..=100.0, None);
        assert_eq!(node(&view, cx).numeric_value(), Some(100.0));
        update(&view, cx, |view| view.value = -5.0);
        assert_eq!(node(&view, cx).numeric_value(), Some(0.0));
    }

    #[gpui::test]
    fn 値を変えられない部品は値を変える操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, 50.0, 0.0..=100.0, None);
        update(&view, cx, |view| view.availability = Availability::Disabled);
        let node = node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Increment));
        assert!(!node.supports_action(accesskit::Action::SetValue));

        update(&view, cx, |view| {
            view.availability = Availability::Enabled;
            view.has_handler = false;
        });
        let node = self::node(&view, cx);
        assert!(!node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Increment));
        assert!(!node.supports_action(accesskit::Action::SetValue));
    }
}
