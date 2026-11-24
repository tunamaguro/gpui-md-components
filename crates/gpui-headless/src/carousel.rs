//! 項目の大きさを位置によって変えながら横に並べる、見た目を持たないカルーセル

pub mod keylines;

use std::{rc::Rc, time::Duration};

use gpui::{
    AnimationExt, AnyElement, App, AvailableSpace, Bounds, ContentMask, Div, Element, ElementId,
    Entity, FocusHandle, GlobalElementId, InspectorElementId, InteractiveElement, IntoElement,
    KeyDownEvent, LayoutId, MouseButton, ParentElement, Pixels, RenderOnce, ScrollEvent, Size,
    SpringAnimation, SpringConfig, SpringPlayback, Stateful, StatefulInteractiveElement, Style,
    StyleRefinement, Styled, Task, Window, accesskit, div, point, prelude::FluentBuilder, px,
    relative, size,
};

use crate::accessibility::A11yInfoExt;
use keylines::{KeylineList, Strategy};

type KeylinesFn = dyn Fn(f32, f32) -> KeylineList;
type ItemFn = dyn Fn(CarouselItem, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>;
type ClickFn = dyn Fn(&usize, &mut Window, &mut App);

/// ホイールの操作が止まったとみなし、項目の止まる位置へ動かし始めるまでの時間
const SETTLE_DELAY: Duration = Duration::from_millis(150);
/// 項目の止まる位置へ動かすバネ。Compose の `spring(stiffness = Spring.StiffnessMediumLow)` と同じ
const SNAP_SPRING: SpringConfig = SpringConfig::new(400.0, 40.0, 1.0);

/// ホイールでスクロールした後に止まる位置
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselSnap {
    /// 止める位置を決めず、スクロールした位置に留まる
    None,
    /// 最も近い項目の止まる位置に止める
    #[default]
    Nearest,
    /// スクロールを始めたときの項目から、前後 1 つまでの項目の止まる位置に止める
    SingleAdvance,
}

/// 項目を描くときの状態
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselItem {
    /// 項目の番号
    pub index: usize,
    /// 項目の見えている大きさ。横幅はキーラインの大きさで、高さはカルーセルの高さ。項目の要素はこの大きさで配置される
    pub size: Size<Pixels>,
    /// どの位置でも項目が取りうる横幅の最小
    pub min_size: Pixels,
    /// どの位置でも項目が取りうる横幅の最大
    pub max_size: Pixels,
    /// キーボードで操作する対象の項目か
    pub active: bool,
    /// 操作する対象で、カルーセルのフォーカスを表示すべきか
    pub focus_visible: bool,
}

/// 項目の大きさと位置を決めた結果
#[derive(Clone)]
struct Layout {
    strategy: Strategy,
    count: usize,
}

impl Layout {
    fn stride(&self) -> f32 {
        self.strategy.item_size() + self.strategy.item_spacing()
    }

    fn max_offset(&self) -> f32 {
        self.strategy.max_scroll_offset(self.count)
    }

    /// 項目 `index` が止まる位置のスクロール量
    fn snap_offset(&self, index: usize) -> f32 {
        (index as f32 * self.stride() - self.strategy.snap_position_offset(index, self.count))
            .clamp(0.0, self.max_offset())
    }

    /// スクロール量 `offset` に止まる位置が最も近い項目
    fn nearest(&self, offset: f32) -> usize {
        (0..self.count)
            .min_by(|&a, &b| {
                let a = (self.snap_offset(a) - offset).abs();
                let b = (self.snap_offset(b) - offset).abs();
                a.total_cmp(&b)
            })
            .unwrap_or(0)
    }
}

struct State {
    focus: FocusHandle,
    /// 目標のスクロール量
    target: f32,
    /// 目標までバネで動かすか。`false` ならすぐに目標の位置で描く
    animate: bool,
    /// 最後に描いたスクロール量
    presented: f32,
    /// スクロールの位置を止めている項目。広さが変わっても、この項目の止まる位置を保つ
    snapped: Option<usize>,
    active: usize,
    layout: Option<Layout>,
    /// ホイールの操作を始めたときの項目と、描いていたスクロール量
    gesture_start: Option<(usize, f32)>,
    settle: Option<Task<()>>,
}

impl State {
    /// 項目 `index` を操作する対象にし、止まる位置までバネで動かす
    fn snap_to(&mut self, index: usize) {
        self.active = index;
        self.settle = None;
        self.gesture_start = None;
        let Some(layout) = &self.layout else {
            return;
        };
        let index = index.min(layout.count.saturating_sub(1));
        self.target = layout.snap_offset(index);
        self.snapped = Some(index);
        self.animate = true;
    }
}

/// ホイールの操作が止まったときに止める項目
///
/// 操作を始めた項目 `start` から、操作を始めたスクロール量 `origin` から動いた向きに、
/// 止まる位置の異なる項目まで少なくとも 1 つ進める。
/// 1 ノッチのスクロール量が項目の間隔の半分に届かなくても、ゆっくり回して先へ進めるようにする
fn settle_index(
    layout: &Layout,
    snap: CarouselSnap,
    start: usize,
    origin: f32,
    offset: f32,
) -> usize {
    let last = layout.count.saturating_sub(1);
    let start_offset = layout.snap_offset(start);
    let mut step = start;
    if offset > origin {
        while step < last {
            step += 1;
            if layout.snap_offset(step) != start_offset {
                break;
            }
        }
    } else if offset < origin {
        while step > 0 {
            step -= 1;
            if layout.snap_offset(step) != start_offset {
                break;
            }
        }
    }
    match snap {
        CarouselSnap::SingleAdvance => step,
        _ => {
            let nearest = layout.nearest(offset);
            if nearest == start { step } else { nearest }
        }
    }
}

/// 項目の大きさを位置によって変えながら横に並べる、見た目を持たないカルーセル
///
/// 項目は [`Carousel::new`] に渡したキーラインに沿って並べる。スクロールの位置によって、
/// 項目の横幅はキーラインの大きさの間を移り変わる。カルーセルは呼び出し側が与えた高さで描き、
/// 項目は高さいっぱいに配置する。範囲の外の項目も描き、各項目を項目の大きさとカルーセルの範囲で切り抜く。
///
/// 次の操作でスクロールする
///
/// - ホイールの横の量と、横の量が無ければ縦の量。スクロールできた場合は親に伝えない。
///   操作が止まると [`Carousel::snap`] の位置に動かす
/// - フォーカス中の左右の矢印キーで前後の項目、Home と End で最初と最後の項目を操作する対象にし、
///   その項目が止まる位置まで動かす
/// - 支援技術から項目にフォーカスを移すと、その項目を操作する対象にし、止まる位置まで動かす
///
/// カルーセル自体が Tab で到達できる 1 つの要素になり、操作する対象の項目を active descendant として公開する。
/// ホイールでスクロールすると、止まる位置が最も近い項目を操作する対象にする。
/// 項目を押すと、スクロールせずにその項目を操作する対象にする。
/// [`Carousel::on_click`] を与えると、項目のクリック、操作する対象の項目での Enter と Space、
/// 支援技術からの項目の Click で、項目の番号を伝える。
///
/// 支援技術には、カルーセルが role `group` として、項目が role `group` と項目の位置・数として公開される。
/// カルーセルと項目のアクセシブルな名前は `aria_label` で与えること
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    inner: Stateful<Div>,
    item_count: usize,
    item_spacing: Pixels,
    keylines: Rc<KeylinesFn>,
    snap: CarouselSnap,
    item: Option<Rc<ItemFn>>,
    on_click: Option<Rc<ClickFn>>,
}

impl Carousel {
    /// `item_count` 個の項目を並べるカルーセルを作る
    ///
    /// `keylines` は、カルーセルの横幅と項目の間隔 (どちらもピクセル) からキーラインを返す。
    /// 空のキーラインを返すと項目を描かない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるとスクロールの位置が失われる
    pub fn new(
        id: impl Into<ElementId>,
        item_count: usize,
        keylines: impl Fn(f32, f32) -> KeylineList + 'static,
    ) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            item_count,
            item_spacing: px(0.0),
            keylines: Rc::new(keylines),
            snap: CarouselSnap::default(),
            item: None,
            on_click: None,
        }
    }

    /// 項目の間隔。既定は 0
    pub fn item_spacing(mut self, spacing: Pixels) -> Self {
        self.item_spacing = spacing;
        self
    }

    /// ホイールでスクロールした後に止まる位置。既定は [`CarouselSnap::Nearest`]
    pub fn snap(mut self, snap: CarouselSnap) -> Self {
        self.snap = snap;
        self
    }

    /// 項目の要素を作る
    ///
    /// `build` は項目の状態と、大きさ・位置・操作を設定済みの要素を受け取り、内容と見た目を付けて返す
    pub fn item(
        mut self,
        build: impl Fn(CarouselItem, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.item = Some(Rc::new(build));
        self
    }

    /// 項目が押されたときに、項目の番号を受け取るハンドラ
    pub fn on_click(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl Styled for Carousel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for Carousel {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Carousel {}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state: Entity<State> = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            focus: cx.focus_handle(),
            target: 0.0,
            animate: false,
            presented: 0.0,
            snapped: Some(0),
            active: 0,
            layout: None,
            gesture_start: None,
            settle: None,
        });
        let (focus, target, animate) = state.update(cx, |state, _| {
            state.active = state.active.min(self.item_count.saturating_sub(1));
            (state.focus.clone(), state.target, state.animate)
        });
        let focus_visible = crate::is_focus_visible(&focus, window, cx);

        let on_key_down = {
            let state = state.clone();
            let on_click = self.on_click.clone();
            let count = self.item_count;
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if count == 0 || event.keystroke.modifiers.modified() {
                    return;
                }
                let active = state.read(cx).active;
                let next = match event.keystroke.key.as_str() {
                    "left" => active.saturating_sub(1),
                    "right" => (active + 1).min(count - 1),
                    "home" => 0,
                    "end" => count - 1,
                    "enter" | "space" => {
                        let Some(on_click) = &on_click else {
                            return;
                        };
                        cx.stop_propagation();
                        if !event.is_held {
                            on_click(&active, window, cx);
                        }
                        return;
                    }
                    _ => return,
                };
                cx.stop_propagation();
                state.update(cx, |state, cx| {
                    state.snap_to(next);
                    cx.notify();
                });
            }
        };

        let on_scroll = {
            let state = state.clone();
            let snap = self.snap;
            move |event: &ScrollEvent, window: &mut Window, cx: &mut App| {
                let delta = event.delta.pixel_delta(window.line_height());
                let delta = if delta.x.abs() > delta.y.abs() {
                    delta.x
                } else {
                    delta.y
                };
                let scrolled = state.update(cx, |state, cx| {
                    let Some(layout) = state.layout.clone() else {
                        return false;
                    };
                    // 止まる位置へ動いている途中は、描いている位置から続ける
                    let before = if state.animate {
                        state.presented
                    } else {
                        state.target
                    };
                    let target = (before - f32::from(delta)).clamp(0.0, layout.max_offset());
                    if target == state.target {
                        return false;
                    }
                    if snap != CarouselSnap::None && state.gesture_start.is_none() {
                        let start = state.snapped.unwrap_or_else(|| layout.nearest(before));
                        state.gesture_start = Some((start, before));
                    }
                    state.target = target;
                    state.animate = false;
                    state.snapped = None;
                    state.active = layout.nearest(target);
                    state.settle = (snap != CarouselSnap::None).then(|| {
                        cx.spawn(async move |this, cx| {
                            cx.background_executor().timer(SETTLE_DELAY).await;
                            this.update(cx, |state: &mut State, cx| {
                                let start = state.gesture_start.take();
                                let (Some(layout), Some((start, origin))) = (&state.layout, start)
                                else {
                                    return;
                                };
                                let index = settle_index(layout, snap, start, origin, state.target);
                                state.snap_to(index);
                                cx.notify();
                            })
                            .ok();
                        })
                    });
                    cx.notify();
                    true
                });
                if scrolled {
                    cx.stop_propagation();
                }
            }
        };

        let track = Track {
            state: state.clone(),
            offset: target,
            item_count: self.item_count,
            item_spacing: self.item_spacing,
            keylines: self.keylines,
            item: self.item,
            on_click: self.on_click,
            focus: focus.clone(),
            focus_visible,
        }
        .with_spring(
            (self.id.clone(), "scroll"),
            SpringAnimation::new(SNAP_SPRING)
                .to(target)
                .playback(if animate {
                    SpringPlayback::Running
                } else {
                    SpringPlayback::Completed
                }),
            |track, offset| Track { offset, ..track },
        );

        self.inner
            .track_focus(&focus.tab_stop(true))
            .role(accesskit::Role::Group)
            .on_key_down(on_key_down)
            .on_scroll(on_scroll)
            .overflow_hidden()
            .child(track)
    }
}

/// 項目をキーラインに沿って配置する要素。カルーセルの横幅が決まった後に項目を作る
struct Track {
    state: Entity<State>,
    offset: f32,
    item_count: usize,
    item_spacing: Pixels,
    keylines: Rc<KeylinesFn>,
    item: Option<Rc<ItemFn>>,
    on_click: Option<Rc<ClickFn>>,
    focus: FocusHandle,
    focus_visible: bool,
}

impl IntoElement for Track {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Track {
    type RequestLayoutState = ();
    /// 項目の要素と、項目を切り抜く範囲
    type PrepaintState = Vec<(AnyElement, Bounds<Pixels>)>;

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
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let width = f32::from(bounds.size.width);
        let spacing = f32::from(self.item_spacing);
        let strategy = Strategy::new((self.keylines)(width, spacing), width, spacing, 0.0, 0.0);
        if !strategy.is_valid() || self.item_count == 0 {
            self.state.update(cx, |state, _| state.layout = None);
            return Vec::new();
        }
        let layout = Layout {
            strategy,
            count: self.item_count,
        };
        let max = layout.max_offset();
        // 止めている項目の位置は、広さや項目の数が変わると動くため、新しい配置で求め直してすぐに移す
        let (offset, active) = self.state.update(cx, |state, _| {
            let mut offset = self.offset.clamp(0.0, max);
            match state.snapped {
                Some(index) => {
                    let index = index.min(layout.count - 1);
                    let target = layout.snap_offset(index);
                    state.snapped = Some(index);
                    if target != state.target {
                        state.target = target;
                        state.animate = false;
                        offset = target;
                    }
                }
                None => state.target = state.target.clamp(0.0, max),
            }
            state.layout = Some(layout.clone());
            state.presented = offset;
            (offset, state.active)
        });

        let strategy = &layout.strategy;
        let keylines = strategy.keylines_for_scroll_offset(offset, max);
        let (min_size, max_size) = strategy.size_range();
        let item_size = strategy.item_size();
        let mut items = Vec::with_capacity(self.item_count);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for index in 0..self.item_count {
                // Compose の Modifier.carouselItem と同じく、すべての項目を焦点の大きさで並べたときの中心から、
                // 前後のキーラインの間で大きさと位置を補間する。最後のキーラインより外の項目は、そのキーラインに重ねる
                let unadjusted = index as f32 * layout.stride() + item_size / 2.0 - offset;
                let before = keylines.keyline_before(unadjusted);
                let after = keylines.keyline_after(unadjusted);
                let (item_width, center) = if before == after {
                    (
                        before.size,
                        before.offset + (unadjusted - before.unadjusted_offset) / before.size,
                    )
                } else {
                    let progress = (unadjusted - before.unadjusted_offset)
                        / (after.unadjusted_offset - before.unadjusted_offset);
                    (
                        before.size + (after.size - before.size) * progress,
                        before.offset + (after.offset - before.offset) * progress,
                    )
                };
                let info = CarouselItem {
                    index,
                    size: size(px(item_width), bounds.size.height),
                    min_size: px(min_size),
                    max_size: px(max_size),
                    active: index == active,
                    focus_visible: index == active && self.focus_visible,
                };
                let mut element = self.render_item(info, window, cx);
                let available = size(
                    AvailableSpace::Definite(px(item_width)),
                    AvailableSpace::Definite(bounds.size.height),
                );
                element.layout_as_root(available, window, cx);
                let origin = bounds.origin + point(px(center - item_width / 2.0), px(0.0));
                // 内容が大きさより広がっても、ほかの項目に重ならないよう項目の大きさで切り抜く
                let mask = Bounds::new(origin, info.size);
                window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
                    element.prepaint_at(origin, window, cx)
                });
                items.push((element, mask));
            }
        });
        items
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        items: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for (item, mask) in items {
                window.with_content_mask(Some(ContentMask { bounds: *mask }), |window| {
                    item.paint(window, cx)
                });
            }
        });
    }
}

impl Track {
    fn render_item(&self, info: CarouselItem, window: &mut Window, cx: &mut App) -> AnyElement {
        let index = info.index;
        let count = self.item_count;
        let state = self.state.clone();
        let focus = self.focus.clone();
        let element = div()
            .id(("carousel-item", index))
            .w(info.size.width)
            .h(info.size.height)
            .role(accesskit::Role::Group)
            .when(info.active, |this| this.aria_active_descendant())
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let focus = focus.clone();
                move |_, window, cx| {
                    window.focus(&focus, cx);
                    state.update(cx, |state, cx| {
                        state.active = index;
                        cx.notify();
                    });
                }
            })
            .on_a11y_action(accesskit::Action::Focus, move |_, window, cx| {
                window.focus(&focus, cx);
                state.update(cx, |state, cx| {
                    state.snap_to(index);
                    cx.notify();
                });
            })
            .when_some(self.on_click.clone(), |this, on_click| {
                let on_a11y_click = on_click.clone();
                this.on_click(move |_, window, cx| on_click(&index, window, cx))
                    .on_a11y_action(accesskit::Action::Click, move |_, window, cx| {
                        on_a11y_click(&index, window, cx)
                    })
            });
        let element = match &self.item {
            Some(build) => build(info, element, window, cx),
            None => element,
        };
        element
            .a11y_info(move |node: &mut accesskit::Node| {
                node.set_position_in_set(index + 1);
                node.set_size_of_set(count);
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::BTreeMap};

    use gpui::{
        Context, Modifiers, Render, ScrollDelta, TestAppContext, VisualTestContext, canvas,
    };

    use super::*;
    use crate::test_window::add_test_window;

    /// 番号ごとの、描いた項目の状態と位置
    type DrawnItems = BTreeMap<usize, (CarouselItem, Bounds<Pixels>)>;

    /// 幅 360、高さ 200 で (0, 0) に置いた、間隔 8 で大きさ 186 を目安にするカルーセル
    struct CarouselView {
        count: usize,
        width: f32,
        snap: CarouselSnap,
        before: FocusHandle,
        /// 描いた項目の状態と位置
        items: Rc<RefCell<DrawnItems>>,
        clicks: Rc<RefCell<Vec<usize>>>,
        /// 親に届いたホイールの操作の数
        parent_scrolls: Rc<RefCell<usize>>,
        node: Rc<RefCell<Option<accesskit::Node>>>,
    }

    impl Render for CarouselView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let items = self.items.clone();
            items.borrow_mut().clear();
            let clicks = self.clicks.clone();
            let count = self.count;
            let carousel = Carousel::new("carousel", count, move |width, spacing| {
                keylines::multi_browse(width, 186.0, spacing, count, 40.0, 56.0)
            })
            .item_spacing(px(8.))
            .snap(self.snap)
            .item(move |info, item, _, _| {
                let items = items.clone();
                item.child(
                    canvas(
                        move |bounds, _, _| {
                            items.borrow_mut().insert(info.index, (info, bounds));
                        },
                        |_, _, _, _| {},
                    )
                    .size_full(),
                )
            })
            .on_click(move |index, _, _| clicks.borrow_mut().push(*index))
            .w(px(self.width))
            .h(px(200.))
            .render(window, cx)
            .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(carousel.a11y_role().expect("role を持つこと"));
            carousel.write_a11y_info(&mut node);
            *self.node.borrow_mut() = Some(node);
            let parent_scrolls = self.parent_scrolls.clone();
            div()
                .flex()
                .flex_col()
                .items_start()
                .on_scroll(move |_, _, _| *parent_scrolls.borrow_mut() += 1)
                .child(div().id("before").track_focus(&self.before).h(px(0.)))
                .child(carousel)
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        configure: impl FnOnce(&mut CarouselView),
    ) -> (Entity<CarouselView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| {
            let mut view = CarouselView {
                count: 10,
                width: 360.,
                snap: CarouselSnap::Nearest,
                before: cx.focus_handle().tab_stop(true),
                items: Rc::default(),
                clicks: Rc::default(),
                parent_scrolls: Rc::default(),
                node: Rc::default(),
            };
            configure(&mut view);
            view
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.run_until_parked();
        (view, cx)
    }

    fn item(
        view: &Entity<CarouselView>,
        cx: &mut VisualTestContext,
        index: usize,
    ) -> (CarouselItem, Bounds<Pixels>) {
        view.read_with(cx, |view, _| view.items.borrow().get(&index).copied())
            .expect("項目が描かれていること")
    }

    fn left(view: &Entity<CarouselView>, cx: &mut VisualTestContext, index: usize) -> f32 {
        f32::from(item(view, cx, index).1.left())
    }

    fn width(view: &Entity<CarouselView>, cx: &mut VisualTestContext, index: usize) -> f32 {
        f32::from(item(view, cx, index).1.size.width)
    }

    fn active(view: &Entity<CarouselView>, cx: &mut VisualTestContext) -> Option<usize> {
        view.read_with(cx, |view, _| {
            view.items
                .borrow()
                .values()
                .find(|(info, _)| info.active)
                .map(|(info, _)| info.index)
        })
    }

    fn scroll(cx: &mut VisualTestContext, delta: f32) {
        cx.simulate_event(ScrollEvent {
            position: point(px(100.), px(100.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(delta))),
            ..Default::default()
        });
    }

    fn settle(cx: &mut VisualTestContext) {
        cx.executor().advance_clock(SETTLE_DELAY);
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 最初の項目を大きく描き後ろを小さくする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        assert_eq!(left(&view, cx, 0), 0.);
        let sizes: Vec<f32> = (0..3).map(|i| width(&view, cx, i)).collect();
        assert!(sizes[0] > sizes[1] && sizes[1] > sizes[2], "{sizes:?}");
        // 3 つ目の項目の右端がカルーセルの右端に揃う
        let right = left(&view, cx, 2) + sizes[2];
        assert!((right - 360.).abs() < 0.01, "{right}");
        assert!(
            left(&view, cx, 3) >= 360.,
            "4 つ目より後はカルーセルの外に置くこと"
        );
        let (info, _) = item(&view, cx, 0);
        assert_eq!(info.size, size(px(sizes[0]), px(200.)));
        assert!(info.min_size <= px(sizes[2]) && info.max_size >= px(sizes[0]));
    }

    #[gpui::test]
    fn 矢印キーで前後の項目を止まる位置まで動かす(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        let large = width(&view, cx, 0);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(active(&view, cx), Some(0));
        assert!(item(&view, cx, 0).0.focus_visible);
        cx.simulate_keystrokes("right");
        assert_eq!(active(&view, cx), Some(1));
        assert!(item(&view, cx, 1).0.focus_visible && !item(&view, cx, 0).0.focus_visible);
        assert_eq!(left(&view, cx, 1), 0.);
        assert_eq!(width(&view, cx, 1), large);
        cx.simulate_keystrokes("end");
        assert_eq!(active(&view, cx), Some(9));
        let right = left(&view, cx, 9) + width(&view, cx, 9);
        assert!(
            (right - 360.).abs() < 0.01,
            "最後の項目を右端に止めること: {right}"
        );
        assert_eq!(width(&view, cx, 9), large);
        cx.simulate_keystrokes("right");
        assert_eq!(active(&view, cx), Some(9), "最後の項目より先に進まないこと");
        cx.simulate_keystrokes("home");
        assert_eq!(active(&view, cx), Some(0));
        assert_eq!(left(&view, cx, 0), 0.);
    }

    #[gpui::test]
    fn ホイールの操作が止まると最も近い項目に止める(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        let large = width(&view, cx, 0);
        let stride = large + 8.;
        scroll(cx, -(stride * 2. + 20.));
        assert_eq!(active(&view, cx), Some(2));
        assert_ne!(
            left(&view, cx, 2),
            0.,
            "操作の途中では止める位置に動かさないこと"
        );
        settle(cx);
        assert_eq!(left(&view, cx, 2), 0.);
        assert_eq!(width(&view, cx, 2), large);
    }

    #[gpui::test]
    fn 項目の間隔の半分に届かないホイールでも動いた向きに進める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |_| {});
        scroll(cx, -60.);
        settle(cx);
        assert_eq!(active(&view, cx), Some(1));
        assert_eq!(left(&view, cx, 1), 0.);
        scroll(cx, -60.);
        settle(cx);
        assert_eq!(active(&view, cx), Some(2));
        scroll(cx, 60.);
        settle(cx);
        assert_eq!(active(&view, cx), Some(1));
    }

    #[test]
    fn 止まる位置へ動いている途中の操作は描いていた位置からの向きで止める() {
        let strategy = Strategy::new(
            keylines::multi_browse(360.0, 186.0, 8.0, 10, 40.0, 56.0),
            360.0,
            8.0,
            0.0,
            0.0,
        );
        let layout = Layout {
            strategy,
            count: 10,
        };
        // 項目 1 へ動いている途中の 95 から 60 進めると、項目 1 の止まる位置より手前でも先へ進める
        assert!(95.0 + 60.0 < layout.snap_offset(1));
        assert_eq!(
            settle_index(&layout, CarouselSnap::Nearest, 1, 95.0, 155.0),
            2
        );
        assert_eq!(
            settle_index(&layout, CarouselSnap::SingleAdvance, 1, 95.0, 35.0),
            0
        );
    }

    #[test]
    fn 止まる位置が同じ項目は飛ばして進める() {
        let strategy = Strategy::new(
            keylines::uncontained(360.0, 150.0, 0.0),
            360.0,
            0.0,
            0.0,
            0.0,
        );
        let layout = Layout {
            strategy,
            count: 10,
        };
        assert_eq!(layout.snap_offset(7), layout.snap_offset(8));
        let origin = layout.snap_offset(8);
        assert_eq!(
            settle_index(
                &layout,
                CarouselSnap::SingleAdvance,
                8,
                origin,
                origin - 10.0
            ),
            6
        );
    }

    #[gpui::test]
    fn 止める位置が無ければスクロールした位置に留まる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |view| view.snap = CarouselSnap::None);
        scroll(cx, -100.);
        let before = left(&view, cx, 1);
        settle(cx);
        assert_eq!(left(&view, cx, 1), before);
        assert!(before < width(&view, cx, 0), "{before}");
    }

    #[gpui::test]
    fn 一つずつ進めるときは始めの項目の隣までに止める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |view| view.snap = CarouselSnap::SingleAdvance);
        let stride = width(&view, cx, 0) + 8.;
        scroll(cx, -stride * 2.);
        scroll(cx, -stride * 2.);
        assert_eq!(active(&view, cx), Some(4));
        settle(cx);
        assert_eq!(active(&view, cx), Some(1));
        assert_eq!(left(&view, cx, 1), 0.);
        // 止めた後の次の操作は、止めた項目から数える
        scroll(cx, -stride * 3.);
        settle(cx);
        assert_eq!(active(&view, cx), Some(2));
    }

    #[gpui::test]
    fn 端でスクロールできないホイールは親に伝える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        let parent_scrolls = |cx: &mut VisualTestContext| {
            view.read_with(cx, |view, _| *view.parent_scrolls.borrow())
        };
        scroll(cx, -50.);
        assert_eq!(parent_scrolls(cx), 0, "スクロールできたら伝えないこと");
        scroll(cx, 100.);
        scroll(cx, 50.);
        assert_eq!(
            parent_scrolls(cx),
            1,
            "始まりより前にはスクロールできないこと"
        );
    }

    #[gpui::test]
    fn 広さが変わっても止めた項目の位置を保つ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        cx.simulate_keystrokes("tab tab right right");
        assert_eq!(left(&view, cx, 2), 0.);
        view.update(cx, |view, cx| {
            view.width = 500.;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(left(&view, cx, 2), 0.);
        let widest = (0..10).map(|i| width(&view, cx, i)).fold(0., f32::max);
        assert_eq!(width(&view, cx, 2), widest, "止めた項目を大きく描くこと");
    }

    #[gpui::test]
    fn 項目を押すと操作する対象にして伝える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        let x = left(&view, cx, 1) + 5.;
        cx.simulate_click(point(px(x), px(100.)), Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| view.clicks.borrow().clone()),
            [1]
        );
        assert_eq!(active(&view, cx), Some(1));
        assert_eq!(left(&view, cx, 0), 0., "押してもスクロールしないこと");
        cx.simulate_keystrokes("enter");
        assert_eq!(
            view.read_with(cx, |view, _| view.clicks.borrow().clone()),
            [1, 1]
        );
    }

    #[gpui::test]
    fn 項目の数が減ると操作する対象を最後の項目にする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, |_| {});
        cx.simulate_keystrokes("tab tab end");
        view.update(cx, |view, cx| {
            view.count = 4;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(active(&view, cx), Some(3));
        let right = left(&view, cx, 3) + width(&view, cx, 3);
        assert!((right - 360.).abs() < 0.01, "{right}");
    }

    #[gpui::test]
    fn カルーセルとして公開する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, |_| {});
        let node = view
            .read_with(cx, |view, _| view.node.borrow().clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::Group);
    }
}
