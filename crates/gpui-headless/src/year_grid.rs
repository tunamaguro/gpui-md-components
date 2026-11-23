//! 年の範囲から 1 年を選ぶ、見た目を持たない格子

use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::accessibility::A11yInfoExt;
use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, ScrollHandle, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

type YearHandler = Rc<dyn Fn(&i32, &mut Window, &mut App)>;
type YearBuilder = Box<dyn Fn(&YearCell, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>>;

/// [`YearGrid::year`] に渡す、1 年のセルの状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YearCell {
    /// セルの年
    pub year: i32,
    /// [`YearGrid::selected`] で選ばれている年か
    pub selected: bool,
    /// 選べる年か。[`YearGrid::enabled`] が `false` を返す年では `false`
    pub enabled: bool,
    /// [`YearGrid::current`] で渡した今年か
    pub current: bool,
    /// フォーカスがあるか
    pub focused: bool,
    /// キーボードで操作していて、フォーカスを見せるべきか
    pub focus_visible: bool,
}

/// 年の範囲から 1 年を選ぶ、見た目を持たない格子
///
/// [`YearGrid::new`] の範囲の年を、範囲の最初の年から左上から右へ、`columns` 年ずつの行に並べる。
/// 選ばれている年は呼び出し側が持ち、[`YearGrid::selected`] に毎回渡す。格子自身は選ばれている年を変えず、
/// 選ぶ操作をされた年を [`YearGrid::on_change`] で求める
///
/// - Tab で到達できる年は 1 つだけで、フォーカスのある年、選ばれている年、[`YearGrid::current`] の年、
///   範囲の最初の年の順に選ぶ
/// - 修飾キーなしの左右の矢印キーで前後の年へ、上下の矢印キーで上下の行の同じ列の年へ、
///   Home と End で行の最初と最後の年へフォーカスを移す。範囲の外へは移らない。選ばれている年は変えない
/// - マウスのクリック・支援技術からのクリック操作・フォーカス中の Enter と Space で、その年を選ぶ操作として扱う。
///   選べない年は、押してもフォーカスを移すだけで、選ぶ操作にならない。
///   格子の中の左ボタンの押下は、年の無い所を含めて親に伝えない
/// - [`YearGrid::scroll`] を渡した場合、フォーカスが別の年へ移ったときに、その年の行を表示範囲に入れる
///
/// 支援技術には role `grid` として公開され、行は role `row`、年は role `gridcell` で、
/// 選ばれているかどうかと選べるかどうかを持つ。格子と年の名前は `aria_label` で呼び出し側が与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`]、[`YearGrid::year`]、[`YearGrid::row`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct YearGrid {
    id: ElementId,
    inner: Div,
    years: RangeInclusive<i32>,
    columns: usize,
    selected: Option<i32>,
    current: Option<i32>,
    enabled: Option<Rc<dyn Fn(i32) -> bool>>,
    on_change: Option<YearHandler>,
    year: Option<YearBuilder>,
    row: Option<Box<dyn Fn(Div) -> Div>>,
    blank: Option<Box<dyn Fn(Div) -> Div>>,
    scroll: Option<ScrollHandle>,
    autofocus: bool,
}

impl YearGrid {
    /// `years` の年を `columns` 年ずつの行に並べる格子を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、年のフォーカスが失われる。
    /// `years` が空の場合は年を置かない。`columns` が 0 の場合は 1 として扱う
    pub fn new(id: impl Into<ElementId>, years: RangeInclusive<i32>, columns: usize) -> Self {
        let id = id.into();
        Self {
            inner: div(),
            id,
            years,
            columns: columns.max(1),
            selected: None,
            current: None,
            enabled: None,
            on_change: None,
            year: None,
            row: None,
            blank: None,
            scroll: None,
            autofocus: false,
        }
    }

    /// 選ばれている年を `year` にする。`None` か範囲の外の年なら、どの年も選ばれていない
    pub fn selected(mut self, year: Option<i32>) -> Self {
        self.selected = year;
        self
    }

    /// 今年を `year` にする。[`YearCell::current`] で示し、選ばれている年が無いときの Tab の到達先にする
    pub fn current(mut self, year: Option<i32>) -> Self {
        self.current = year;
        self
    }

    /// `enabled` が `false` を返す年を選べなくする。既定ではすべての年を選べる
    pub fn enabled(mut self, enabled: impl Fn(i32) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    /// 年を選ぶ操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶ操作をされた年を渡す。選ばれている年でも呼ぶ
    pub fn on_change(mut self, handler: impl Fn(&i32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 年のセルの見た目を `build` で付ける。`build` は年ごとに、その年の状態と、見た目を付けるセルを受け取る
    pub fn year(
        mut self,
        build: impl Fn(&YearCell, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.year = Some(Box::new(build));
        self
    }

    /// 行の見た目を `build` で付ける。並べ方を変える場合に使う。既定では年を左から横に並べる
    pub fn row(mut self, build: impl Fn(Div) -> Div + 'static) -> Self {
        self.row = Some(Box::new(build));
        self
    }

    /// 最後の行の、年の無い列の見た目を `build` で付ける。年のセルと同じ大きさにして列の位置をそろえる場合に使う
    pub fn blank(mut self, build: impl Fn(Div) -> Div + 'static) -> Self {
        self.blank = Some(Box::new(build));
        self
    }

    /// 格子のスクロールを `handle` で追い、フォーカスが別の年へ移ったときに、その年の行を表示範囲に入れる
    ///
    /// 格子の子は行だけになるため、`handle` の子の番号は範囲の最初の年からの行の番号になる。
    /// スクロールさせるには、格子の高さを限り `overflow_y_scroll` を付けること。
    /// `track_scroll` で別のハンドルを付けた場合は、そちらは使わない
    pub fn scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }

    /// `autofocus` なら、描画され始めたときに Tab で到達できる年へフォーカスを移す。既定では移さない
    ///
    /// 描画され始めたときは、初めて描画したときと、1 フレーム以上描画されなかった後に再び描画したとき
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl RenderOnce for YearGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // 反復し終えた範囲は端点が残っていても空なので、空の範囲として扱う
        #[expect(
            clippy::reversed_empty_ranges,
            reason = "years を空の範囲に置き換えるため"
        )]
        let years = if self.years.is_empty() {
            1..=0
        } else {
            self.years.clone()
        };
        let (first, last) = (*years.start(), *years.end());
        let columns = self.columns;
        let count = if years.is_empty() {
            0
        } else {
            (i64::from(last) - i64::from(first) + 1) as usize
        };
        let rows = count.div_ceil(columns);
        let in_range = {
            let years = years.clone();
            move |year: i32| years.contains(&year)
        };
        // 年のフォーカスは年ごとに持ち、範囲が変わっても同じ年には同じフォーカスを使う
        let handles = window.use_keyed_state((self.id.clone(), "handles"), cx, |_, _| {
            HashMap::<i32, FocusHandle>::new()
        });
        handles.update(cx, |handles, cx| {
            for year in years.clone() {
                handles.entry(year).or_insert_with(|| cx.focus_handle());
            }
        });
        let handles_state = handles;
        let handles = handles_state.read(cx).clone();
        let handle = |year: i32| &handles[&year];

        let focused_before = window.focused(cx);
        let mut focused_year = years.clone().find(|&year| handle(year).is_focused(window));
        let fallback_year = self
            .selected
            .filter(|&year| in_range(year))
            .or(self.current.filter(|&year| in_range(year)))
            .or((count > 0).then_some(first));
        let tab_year = focused_year.or(fallback_year);
        // 年が無くなったときに、フォーカスを置く格子のフォーカス。Tab では到達しない
        let grid_focus = window
            .use_keyed_state((self.id.clone(), "grid"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        // 範囲が変わってフォーカスのある年が範囲の外になった場合は、範囲の中の年に移す。
        // 範囲の中に年が無ければ、フォーカスを外すと次の Tab がウィンドウの先頭から始まるため、格子に移して Tab の位置を保つ。
        // 格子に退避している間に年が再び使えるようになったら、年に戻す
        if focused_year.is_none()
            && (grid_focus.is_focused(window)
                || handles
                    .iter()
                    .any(|(&year, handle)| !in_range(year) && handle.is_focused(window)))
        {
            match fallback_year {
                Some(year) => {
                    window.focus(handle(year), cx);
                    focused_year = Some(year);
                }
                None if !grid_focus.is_focused(window) => window.focus(&grid_focus, cx),
                None => {}
            }
        }
        let mounted = window.use_keyed_state((self.id.clone(), "mounted"), cx, |_, _| false);
        let first_frame = !mounted.update(cx, |mounted, _| std::mem::replace(mounted, true));
        if first_frame
            && self.autofocus
            && let Some(year) = tab_year
        {
            window.focus(handle(year), cx);
            focused_year = Some(year);
        }
        // 描画中に移したフォーカスは、gpui がこの描画の後に描き直しを求めないため、
        // 先に描いた要素のフォーカスの見た目を消すよう、次のフレームで描き直す
        if window.focused(cx) != focused_before {
            window.request_animation_frame();
        }
        // 範囲の外の年のフォーカスは、移し先を決めた後は使わない
        handles_state.update(cx, |handles, _| handles.retain(|&year, _| in_range(year)));
        let focused_row =
            focused_year.map(|year| (i64::from(year) - i64::from(first)) as usize / columns);
        // 前の描画で表示範囲に入れたときの、フォーカスのある年・表示領域の高さ・その行の表示領域からの位置
        let placed = window.use_keyed_state((self.id.clone(), "placed"), cx, |_, _| {
            None::<(Option<i32>, Pixels, Option<(Pixels, Pixels)>)>
        });
        // フォーカスのある年・表示領域の高さ・行の配置のどれかが変わったら、その行を表示範囲に入れる。
        // 手で動かしたスクロールは、これらが変わるまで保つ。
        // Tab で入った年、範囲が変わって行が移った年、格子が動いた場合も対象にする。
        // 行と表示領域の位置がどちらもこの描画で決まった後に求めるため、子を prepaint した後に行う
        let scroll_into_view = self.scroll.clone().map(|scroll| {
            move |_: Vec<gpui::Bounds<Pixels>>, window: &mut Window, cx: &mut App| {
                let viewport = scroll.bounds();
                let item = focused_row.and_then(|row| scroll.bounds_for_item(row));
                // 行の範囲はスクロールする前の位置なので、表示領域からの位置はスクロールで変わらず、配置が変わると変わる
                let placement =
                    item.map(|item| (item.top() - viewport.top(), item.bottom() - viewport.top()));
                let key = Some((focused_year, viewport.size.height, placement));
                if placed.update(cx, |placed, _| std::mem::replace(placed, key)) == key {
                    return;
                }
                let Some(item) = item else {
                    return;
                };
                let mut offset = scroll.offset();
                let top = item.top() + offset.y;
                let bottom = item.bottom() + offset.y;
                // 行が表示領域より高い場合は、行の上端を合わせる
                if top < viewport.top() || item.size.height > viewport.size.height {
                    offset.y += viewport.top() - top;
                } else if bottom > viewport.bottom() {
                    offset.y -= bottom - viewport.bottom();
                }
                if offset != scroll.offset() {
                    scroll.set_offset(offset);
                    window.request_animation_frame();
                }
            }
        });

        let enabled = self.enabled.clone();
        let is_enabled = move |year: i32| enabled.as_ref().is_none_or(|enabled| enabled(year));
        let select = self.on_change.clone().map(|on_change| {
            let is_enabled = is_enabled.clone();
            Rc::new(move |year: i32, window: &mut Window, cx: &mut App| {
                if is_enabled(year) {
                    on_change(&year, window, cx);
                }
            })
        });

        let on_key_down = {
            let handles = handles.clone();
            let years = years.clone();
            move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
                let Some(current) = years.clone().find(|year| handles[year].is_focused(window))
                else {
                    return;
                };
                let modifiers = event.keystroke.modifiers;
                if modifiers.modified() {
                    return;
                }
                let index = i64::from(current) - i64::from(first);
                let column = index % columns as i64;
                let target = match event.keystroke.key.as_str() {
                    "right" => index + 1,
                    "left" => index - 1,
                    "down" => index + columns as i64,
                    "up" => index - columns as i64,
                    "home" => index - column,
                    "end" => (index - column + columns as i64 - 1).min(count as i64 - 1),
                    _ => return,
                };
                cx.stop_propagation();
                if !(0..count as i64).contains(&target) || target == index {
                    return;
                }
                let year = (i64::from(first) + target) as i32;
                window.focus(&handles[&year], cx);
            }
        };

        let mut grid_rows = Vec::with_capacity(rows);
        for row in 0..rows {
            let cells = (0..columns).map(|column| {
                let index = row * columns + column;
                if index >= count {
                    // 最後の行の空いた列も埋め、年の列の位置をほかの行とそろえる
                    let blank = div();
                    return match &self.blank {
                        Some(build) => build(blank),
                        None => blank,
                    }
                    .into_any_element();
                }
                let year = first + index as i32;
                let focus_handle = handle(year);
                let state = YearCell {
                    year,
                    selected: self.selected == Some(year),
                    enabled: is_enabled(year),
                    current: self.current == Some(year),
                    focused: focus_handle.is_focused(window),
                    focus_visible: crate::is_focus_visible(focus_handle, window, cx),
                };
                let cell = div().id(ElementId::Name(format!("year-{year}").into()));
                let cell = match &self.year {
                    Some(build) => build(&state, cell, window, cx),
                    None => cell,
                };
                cell.role(accesskit::Role::GridCell)
                    .aria_column_index(column)
                    .aria_selected(state.selected)
                    .track_focus(&focus_handle.clone().tab_stop(tab_year == Some(year)))
                    .when_some(select.clone().filter(|_| state.enabled), |this, select| {
                        let on_click = select.clone();
                        this.on_click(move |_: &ClickEvent, window, cx| on_click(year, window, cx))
                            .on_a11y_action(accesskit::Action::Click, move |_, window, cx| {
                                select(year, window, cx)
                            })
                    })
                    .when(!state.enabled, |this| {
                        let focus_handle = focus_handle.clone();
                        // 押下を親に伝えず、親の on_click を呼ばせない。
                        // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                        this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            window.focus(&focus_handle, cx);
                            cx.stop_propagation();
                        })
                    })
                    .a11y_info(move |node: &mut accesskit::Node| {
                        if !state.enabled {
                            node.set_disabled();
                        }
                    })
                    .into_any_element()
            });
            let row_div = div().flex();
            let row_div = match &self.row {
                Some(build) => build(row_div),
                None => row_div,
            };
            grid_rows.push(
                row_div
                    .id(("row", row))
                    .role(accesskit::Role::Row)
                    .aria_row_index(row)
                    .children(cells),
            );
        }

        self.inner
            .when_some(scroll_into_view, |this, listener| {
                this.on_children_prepainted(listener)
            })
            .id(self.id)
            // 退避先として格子にフォーカスを移した間だけ、格子にフォーカスを付ける。
            // 常に付けると年の無い所を押したときに格子にフォーカスが移る
            .when(grid_focus.is_focused(window), |this| {
                this.track_focus(&grid_focus)
            })
            .role(accesskit::Role::Grid)
            .aria_row_count(rows)
            .aria_column_count(columns)
            .when_some(self.scroll.as_ref(), |this, scroll| {
                this.track_scroll(scroll)
            })
            .on_key_down(on_key_down)
            // 年の無い所 (行の間や空いた列) の押下を親に伝えず、親の track_focus がフォーカスを奪わないようにする
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .flex()
            .flex_col()
            .children(grid_rows)
    }
}

impl Styled for YearGrid {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for YearGrid {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for YearGrid {}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::test_window::add_test_window;

    struct YearGridView {
        years: RangeInclusive<i32>,
        selected: Option<i32>,
        current: Option<i32>,
        visible: bool,
        /// 格子の上の余白
        top: gpui::Pixels,
        autofocus: bool,
        before: FocusHandle,
        after: FocusHandle,
        scroll: ScrollHandle,
        changes: Rc<RefCell<Vec<i32>>>,
        focused: Rc<Cell<Option<i32>>>,
    }

    impl Render for YearGridView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let focused = self.focused.clone();
            focused.set(None);
            let grid = YearGrid::new("years", self.years.clone(), 3)
                .selected(self.selected)
                .current(self.current)
                // 2030 年は選べない
                .enabled(|year| year != 2030)
                .autofocus(self.autofocus)
                .scroll(&self.scroll)
                .on_change(cx.listener(|this, year: &i32, _, cx| {
                    this.changes.borrow_mut().push(*year);
                    this.selected = Some(*year);
                    cx.notify();
                }))
                .year(move |year, cell, _, _| {
                    if year.focused {
                        focused.set(Some(year.year));
                    }
                    cell.w(px(100.)).h(px(40.))
                })
                .h(px(120.))
                .overflow_y_scroll();
            div()
                .size_full()
                .child(div().id("before").track_focus(&self.before).h(px(10.)))
                .child(div().h(self.top))
                .when(self.visible, |this| this.child(grid))
                .child(div().id("after").track_focus(&self.after).h(px(10.)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selected: Option<i32>,
        current: Option<i32>,
    ) -> (gpui::Entity<YearGridView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| YearGridView {
            years: 2020..=2035,
            selected,
            current,
            visible: true,
            top: px(0.),
            autofocus: false,
            before: cx.focus_handle().tab_stop(true),
            after: cx.focus_handle().tab_stop(true),
            scroll: ScrollHandle::new(),
            changes: Rc::default(),
            focused: Rc::default(),
        });
        cx.run_until_parked();
        (view, cx)
    }

    /// フォーカスのある年。最後の描画の時点の値
    ///
    /// テストにはフレームの供給が無いため、スクロールを行う次のフレームを進めてから読む
    fn focused(view: &gpui::Entity<YearGridView>, cx: &mut VisualTestContext) -> Option<i32> {
        cx.run_until_parked();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        view.read_with(cx, |view, _| view.focused.get())
    }

    fn changes(view: &gpui::Entity<YearGridView>, cx: &mut VisualTestContext) -> Vec<i32> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    /// Enter と Space はキーを離したときにクリックとして届くため、離すところまで送る
    fn press(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = gpui::Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke });
        cx.run_until_parked();
    }

    #[gpui::test]
    fn tab_では選ばれている年に入り_矢印キーではフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2025), Some(2026));
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2025));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2026));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some(2029));
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&view, cx), Some(2029));
        cx.simulate_keystrokes("end");
        assert_eq!(focused(&view, cx), Some(2031));
        cx.simulate_keystrokes("up left");
        assert_eq!(focused(&view, cx), Some(2027));
        assert!(changes(&view, cx).is_empty());
        // 格子の Tab の到達先は 1 つだけ
        cx.simulate_keystrokes("tab");
        let after = view.read_with(cx, |view, _| view.after.clone());
        assert!(cx.update(|window, _| after.is_focused(window)));
    }

    #[gpui::test]
    fn 選ばれている年が無ければ今年_今年も無ければ最初の年に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, Some(2026));
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2026));
        view.update(cx, |view, cx| {
            view.current = Some(1999);
            cx.notify();
        });
        cx.simulate_keystrokes("shift-tab tab");
        assert_eq!(focused(&view, cx), Some(2020));
    }

    #[gpui::test]
    fn 範囲の外へは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2020), None);
        cx.simulate_keystrokes("tab tab left up");
        assert_eq!(focused(&view, cx), Some(2020));
        view.update(cx, |view, cx| {
            view.selected = Some(2033);
            cx.notify();
        });
        cx.simulate_keystrokes("shift-tab tab");
        assert_eq!(focused(&view, cx), Some(2033));
        cx.simulate_keystrokes("end down");
        assert_eq!(focused(&view, cx), Some(2034));
        // 最後の行は 2035 年だけ
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2035));
        cx.simulate_keystrokes("right down end");
        assert_eq!(focused(&view, cx), Some(2035));
    }

    #[gpui::test]
    fn enter_とクリックで選び_選べない年は選ばない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2026), None);
        cx.simulate_keystrokes("tab tab right");
        press(cx, "enter");
        assert_eq!(changes(&view, cx), [2027]);
        // 2030 年は 4 行目の 2 列目
        cx.simulate_keystrokes("down left");
        press(cx, "space");
        assert_eq!(changes(&view, cx), [2027, 2029]);
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2030));
        press(cx, "enter");
        assert_eq!(changes(&view, cx), [2027, 2029]);
        view.update(cx, |view, cx| {
            view.scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        cx.run_until_parked();
        // 2021 年は 1 行目の 2 列目。格子は before の 10px の下から始まる
        cx.simulate_click(gpui::point(px(150.), px(30.)), Modifiers::none());
        assert_eq!(changes(&view, cx), [2027, 2029, 2021]);
    }

    #[gpui::test]
    fn キー操作で移った年の行を表示範囲に入れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2020), None);
        cx.simulate_keystrokes("tab tab down down down");
        assert_eq!(focused(&view, cx), Some(2029));
        // 4 行目を 3 行分の高さに入れる
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-40.));
    }

    #[gpui::test]
    fn tab_で入った年の行を表示範囲に入れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2020), None);
        cx.simulate_keystrokes("tab tab shift-tab");
        view.update(cx, |view, cx| {
            view.scroll.set_offset(gpui::point(px(0.), px(-120.)));
            cx.notify();
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(2020));
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(0.));
    }

    #[gpui::test]
    fn autofocus_で移った画面外の年を表示範囲に入れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2035), None);
        view.update(cx, |view, cx| {
            view.visible = false;
            view.autofocus = true;
            cx.notify();
        });
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.visible = true;
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2035));
        // テストにはフレームの供給が無いため、表示領域の位置が決まった次のフレームを進める
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        // 6 行目を 3 行分の高さの下端に入れる
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-120.));
    }

    #[gpui::test]
    fn 範囲が変わってフォーカスのある年の行が移ったら表示範囲に入れる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2026), None);
        view.update(cx, |view, cx| {
            view.years = 2026..=2040;
            cx.notify();
        });
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2026));
        // 同じ描画で格子も下へ動かす
        view.update(cx, |view, cx| {
            view.years = 2014..=2040;
            view.top = px(120.);
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2026));
        // 2026 年は 5 行目に移る
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-80.));
    }

    #[gpui::test]
    fn 範囲が変わった次の描画で格子が動いてもフォーカスのある行を表示範囲に入れる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2026), None);
        view.update(cx, |view, cx| {
            view.years = 2026..=2040;
            cx.notify();
        });
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2026));
        view.update(cx, |view, cx| {
            view.years = 2014..=2040;
            cx.notify();
        });
        cx.run_until_parked();
        // 格子を動かすのと同じフレームで描く
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.top = px(120.);
                cx.notify();
            });
            window.simulate_next_frame(cx);
        });
        assert_eq!(focused(&view, cx), Some(2026));
        // 2026 年は 5 行目に移る
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-80.));
    }

    #[gpui::test]
    fn 手で動かした後に同じ行の年へ移っても表示範囲に入れる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2020), None);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2020));
        view.update(cx, |view, cx| {
            view.scroll.set_offset(gpui::point(px(0.), px(-120.)));
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2020));
        // 手で動かしたスクロールは、フォーカスが変わるまで保つ
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-120.));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2021));
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(0.));
        // 描き直した位置で押せる。2022 年は 1 行目の 3 列目
        cx.simulate_click(gpui::point(px(250.), px(30.)), Modifiers::none());
        assert_eq!(changes(&view, cx), [2022]);
    }

    #[gpui::test]
    fn 範囲から外れた年の移動先も表示範囲に入れる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(2020), None);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2020));
        view.update(cx, |view, cx| {
            view.years = 2021..=2038;
            view.selected = Some(2038);
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2038));
        // 2038 年は 6 行目
        let offset = view.read_with(cx, |view, _| view.scroll.offset().y);
        assert_eq!(offset, px(-120.));
    }

    #[gpui::test]
    fn フォーカス中に範囲が空になっても_tab_の位置を保つ(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2026), None);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2026));
        let mut years = 2026..=2026;
        years.next();
        view.update(cx, |view, cx| {
            view.years = years;
            cx.notify();
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        let after = view.read_with(cx, |view, _| view.after.clone());
        assert!(cx.update(|window, _| after.is_focused(window)));
    }

    #[gpui::test]
    fn 反復し終えた範囲は空の範囲として扱う(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, None);
        let mut years = 2026..=2026;
        years.next();
        view.update(cx, |view, cx| {
            view.years = years;
            cx.notify();
        });
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 描画され始めたときに_autofocus_なら_tab_の到達先へフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2026), None);
        assert_eq!(focused(&view, cx), None);
        view.update(cx, |view, cx| {
            view.visible = false;
            view.autofocus = true;
            cx.notify();
        });
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.visible = true;
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2026));
        // 描画を続けている間は、フォーカスを移し直さない
        let before = view.read_with(cx, |view, _| view.before.clone());
        cx.update(|window, cx| window.focus(&before, cx));
        view.update(cx, |_, cx| cx.notify());
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 範囲が変わってフォーカスのある年が外れたら範囲の中の年に移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(2026), None);
        cx.simulate_keystrokes("tab tab");
        assert_eq!(focused(&view, cx), Some(2026));
        view.update(cx, |view, cx| {
            view.years = 2027..=2040;
            view.selected = Some(2028);
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(2028));
    }
}
