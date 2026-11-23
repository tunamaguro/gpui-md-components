// DateRange::select の規則は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateRangePicker.kt
//         (`updateDateSelection`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! 月を縦に並べて日付の範囲を選ぶ、見た目を持たないカレンダー

use std::cell::Cell;
use std::ops::RangeInclusive;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Div, ElementId, Entity, FocusHandle, InteractiveElement,
    Interactivity, IntoElement, KeyDownEvent, MouseButton, ParentElement, Pixels, RenderOnce,
    ScrollStrategy, Stateful, StatefulInteractiveElement, StyleRefinement, Styled,
    UniformListScrollHandle, Window, accesskit, div, point, prelude::FluentBuilder, uniform_list,
};

use crate::accessibility::A11yInfoExt;
use crate::calendar::Date;

/// 選ばれている日付の範囲
///
/// `end` があるときは `start` もあり、`end` は `start` 以降
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    /// 始まりの日付
    pub start: Option<Date>,
    /// 終わりの日付
    pub end: Option<Date>,
}

impl DateRange {
    /// `date` を選ぶ操作をした後の範囲を返す
    ///
    /// 始まりだけがあり `date` がそれ以降なら、`date` を終わりにする。それ以外は `date` だけを始まりにする
    pub fn select(self, date: Date) -> Self {
        match (self.start, self.end) {
            (Some(start), None) if date >= start => Self {
                start: Some(start),
                end: Some(date),
            },
            _ => Self {
                start: Some(date),
                end: None,
            },
        }
    }

    /// 始まりと終わりがそろっていれば、その間 (両端を含む) に `date` があるか
    pub fn contains(&self, date: &Date) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => (start..=end).contains(date),
            _ => false,
        }
    }
}

/// [`RangeCalendar::day`] に渡す日付の状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeDay {
    /// 日付
    pub date: Date,
    /// 選ばれている範囲の始まりか
    pub start: bool,
    /// 選ばれている範囲の終わりか
    pub end: bool,
    /// 始まりと終わりがそろった範囲の中 (両端を含む) にあるか
    pub in_range: bool,
    /// 選べるか
    pub enabled: bool,
    /// [`RangeCalendar::today`] の日付か
    pub today: bool,
    /// キー操作の対象の日付で、フォーカスがあることを表示すべきか
    pub focus_visible: bool,
}

/// [`RangeCalendar::week`] に渡す週の状態
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RangeWeek {
    /// 始まりと終わりがそろった範囲のうち、この週のこの月の日付にかかる部分。かからなければ `None`
    pub band: Option<RangeBand>,
}

/// 週の行のうち、選ばれている範囲にかかる列
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RangeBand {
    /// 最初の列。日曜日が 0
    pub first: usize,
    /// 最後の列
    pub last: usize,
    /// 最初の列が範囲の始まりか
    pub from_start: bool,
    /// 最後の列が範囲の終わりか
    pub to_end: bool,
}

type ChangeHandler = Rc<dyn Fn(&DateRange, &mut Window, &mut App)>;
type EnabledFn = Rc<dyn Fn(&Date) -> bool>;
type HeaderFn = Rc<dyn Fn(&Date, &mut Window, &mut App) -> AnyElement>;
type GridFn = Rc<dyn Fn(&Date, Stateful<Div>) -> Stateful<Div>>;
type WeekFn = Rc<dyn Fn(&RangeWeek, Div) -> Div>;
type DayFn = Rc<dyn Fn(&RangeDay, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>>;
type BlankFn = Rc<dyn Fn(Div) -> Div>;
type SelectFn = Rc<dyn Fn(Date, &mut Window, &mut App)>;
type SetActiveFn = dyn Fn(Date, &mut App);

/// 週の行の数
const WEEKS: usize = 6;

/// 月を縦に並べて日付の範囲を選ぶ、見た目を持たないカレンダー
///
/// 年の範囲のすべての月を、月ごとに見出しと日曜日から始まる 6 週の格子にして縦に並べ、見えている月だけを描く。
/// 月の外の日はセルを空にする。選ばれている範囲は呼び出し側が持ち、[`RangeCalendar::new`] に毎回渡す。
/// カレンダー自身は範囲を変えず、選ぶ操作をされた後の範囲を [`RangeCalendar::on_change`] で求める。
/// 月の高さは同じにすること。高さが違うと、スクロールと描く範囲がずれる
///
/// フォーカスはカレンダー全体で 1 つで、Tab で 1 回だけ到達する。キー操作の対象の日付 (操作日) をカレンダーが持つ。
/// 操作日は最初は範囲の始まり、無ければ [`RangeCalendar::today`]、どちらも無ければ年の範囲の最初の日にし、その月を一番上に表示する
///
/// - フォーカス中の左右の矢印キーで前後の日、上下の矢印キーで前後の週の同じ曜日、Home と End で週の始まりと終わり、
///   Page Up と Page Down で前後の月、Shift を押しながら Page Up と Page Down で前後の年の同じ日に、操作日を移す。
///   月か年を移って同じ日が無ければ月末にし、続けて月か年を移るときは元の日を基準にする。
///   年の範囲の外へは移らない。移った操作日の週が見えるまでスクロールする
/// - 操作日の月がスクロールで見えなくなっている間のキー操作は、操作日を見えている最初の月の 1 日に移すだけにする
/// - フォーカス中の Enter・Space、日付のクリック、支援技術からの日付のクリック操作で、その日付を選ぶよう求める。
///   選べない日付 ([`RangeCalendar::enabled`]) は求めない。選び方は [`DateRange::select`]
/// - 日付を押すと、カレンダーにフォーカスを移し、その日付を操作日にする
///
/// 支援技術には、月ごとの role `grid` と、週ごとの role `row`、日付ごとの role `gridcell` と、
/// 範囲の始まりか終わりなら選ばれていること、選べるかどうかとして公開される。
/// フォーカスのある間は、操作日の日付がフォーカスのある要素として伝わる (active descendant)。
/// カレンダー全体の role は `group` で、カレンダー・月・日付の名前は `aria_label` で与えること。
/// 見た目は [`RangeCalendar::header`]・[`RangeCalendar::grid`]・[`RangeCalendar::week`]・[`RangeCalendar::day`]・
/// [`RangeCalendar::blank`] と、[`Styled`] で呼び出し側が付ける。高さを指定すること
#[derive(IntoElement)]
pub struct RangeCalendar {
    id: ElementId,
    inner: Div,
    years: RangeInclusive<i32>,
    selection: DateRange,
    today: Option<Date>,
    enabled: Option<EnabledFn>,
    on_change: Option<ChangeHandler>,
    header: Option<HeaderFn>,
    grid: Option<GridFn>,
    week: Option<WeekFn>,
    day: Option<DayFn>,
    blank: Option<BlankFn>,
    scroll: Option<UniformListScrollHandle>,
}

impl RangeCalendar {
    /// `years` の年のすべての月を並べ、`selection` を選ばれている範囲として表示するカレンダーを作る
    ///
    /// `years` は 1 年から 9999 年の外を除き、空なら最初の年だけにする。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、操作日とフォーカスとスクロールの位置が失われる
    pub fn new(id: impl Into<ElementId>, years: RangeInclusive<i32>, selection: DateRange) -> Self {
        Self {
            id: id.into(),
            inner: div(),
            years,
            selection,
            today: None,
            enabled: None,
            on_change: None,
            header: None,
            grid: None,
            week: None,
            day: None,
            blank: None,
            scroll: None,
        }
    }

    /// 今日の日付を `date` にする。[`RangeDay::today`] で示す
    pub fn today(mut self, date: Option<Date>) -> Self {
        self.today = date;
        self
    }

    /// `enabled` が `false` を返す日付を選べなくする。既定ではすべての日付を選べる
    pub fn enabled(mut self, enabled: impl Fn(&Date) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    /// 日付を選ぶ操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選んだ後の範囲を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`RangeCalendar::new`] に渡すこと
    pub fn on_change(
        mut self,
        handler: impl Fn(&DateRange, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 月の格子の上に置く見出しを作る。`build` には月の 1 日を渡す。既定では置かない
    pub fn header(
        mut self,
        build: impl Fn(&Date, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.header = Some(Rc::new(build));
        self
    }

    /// 月の格子の要素を整える。`build` には月の 1 日と格子の要素を渡す
    pub fn grid(mut self, build: impl Fn(&Date, Stateful<Div>) -> Stateful<Div> + 'static) -> Self {
        self.grid = Some(Rc::new(build));
        self
    }

    /// 週の行の要素を整える。`build` には週の状態と、7 つのセルを横に並べる行の要素を渡す
    pub fn week(mut self, build: impl Fn(&RangeWeek, Div) -> Div + 'static) -> Self {
        self.week = Some(Rc::new(build));
        self
    }

    /// 日付のセルの要素を整える。`build` には日付の状態とセルの要素を渡す
    pub fn day(
        mut self,
        build: impl Fn(&RangeDay, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.day = Some(Rc::new(build));
        self
    }

    /// 月の外の日の空のセルの要素を整える
    pub fn blank(mut self, build: impl Fn(Div) -> Div + 'static) -> Self {
        self.blank = Some(Rc::new(build));
        self
    }

    /// 月の一覧のスクロールを `handle` で扱う。既定ではカレンダーが内部で持つ
    pub fn scroll(mut self, handle: &UniformListScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }
}

impl Styled for RangeCalendar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for RangeCalendar {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for RangeCalendar {}

/// 描画をまたいで保つ状態
struct State {
    focus: FocusHandle,
    /// 操作日。最初の描画で決める
    active: Option<Date>,
    /// Page Up・Page Down で最後に移った先の日付と、月末に寄せる前の日
    anchor: Option<(Date, u8)>,
    /// 描いた月から測った大きさ
    layout: Option<MonthLayout>,
    /// 一覧の最初の年。変わると操作日の月を一覧の一番上に表示し直す
    first_year: Option<i32>,
    /// 次に月を測ったときに、操作日の週が見えるまでスクロールするか
    reveal: bool,
}

/// 描いた月から測った大きさ
#[derive(Clone, Copy)]
struct MonthLayout {
    /// 月の高さ
    month: Pixels,
    /// 月の上端から最初の週の行の上端まで
    grid_top: Pixels,
    /// 週の行の間隔
    row: Pixels,
    /// 週の行の高さ
    row_height: Pixels,
}

impl MonthLayout {
    /// 一覧の上端から `date` の週の行の上端まで
    fn row_top(&self, first_year: i32, date: Date) -> Pixels {
        self.month * month_index(first_year, date) as f32
            + self.grid_top
            + self.row * week_row(date) as f32
    }
}

/// 月の 1 日から、年の範囲の最初の月からの番号を求める
fn month_index(first_year: i32, date: Date) -> usize {
    ((date.year() - first_year) * 12 + i32::from(date.month()) - 1) as usize
}

/// `date` の週の行の番号
fn week_row(date: Date) -> usize {
    let first = Date::new(date.year(), date.month(), 1).expect("the first day of a month is valid");
    (usize::from(first.weekday()) + usize::from(date.day()) - 1) / 7
}

/// `date` の週が見えるまで一覧をスクロールする
fn reveal(
    scroll: &UniformListScrollHandle,
    first_year: i32,
    layout: Option<MonthLayout>,
    date: Date,
) {
    let index = month_index(first_year, date);
    // last_item_size の item は一覧の見えている範囲の大きさ
    let viewport = scroll
        .0
        .borrow()
        .last_item_size
        .map(|size| size.item.height);
    let (Some(viewport), Some(layout)) = (viewport, layout) else {
        // まだ描いていなければ、月が見えるところまで描画のときにスクロールさせる
        scroll.scroll_to_item(index, ScrollStrategy::Nearest);
        return;
    };
    let state = scroll.0.borrow();
    let offset = state.base_handle.offset();
    let top = layout.row_top(first_year, date);
    let bottom = top + layout.row_height;
    let scroll_top = -offset.y;
    let new_top = if top < scroll_top {
        top
    } else if bottom > scroll_top + viewport {
        bottom - viewport
    } else {
        return;
    };
    state.base_handle.set_offset(point(offset.x, -new_top));
}

/// `current` の週の行がスクロールで見えなくなっていれば、全体が見えている最初の週の行のうち月の最初の日付を返す。
/// まだ描いていないときと、`current` の行が少しでも見えているときと、全体が見える行が無いときは `None`
fn retarget(
    scroll: &UniformListScrollHandle,
    layout: Option<MonthLayout>,
    first: Date,
    last: Date,
    current: Date,
) -> Option<Date> {
    let layout = layout.filter(|layout| layout.month > Pixels::ZERO)?;
    let (top, bottom) = {
        let state = scroll.0.borrow();
        let top = -state.base_handle.offset().y;
        (top, top + state.last_item_size?.item.height)
    };
    let current_top = layout.row_top(first.year(), current);
    if current_top + layout.row_height > top && current_top < bottom {
        return None;
    }
    let index = (top / layout.month).floor().max(0.) as i64;
    // 見えている範囲が週の行より高ければ、上端の月か次の月に全体が見える行がある
    (index..index + 2)
        .filter_map(|index| first.add_months(index))
        .filter(|month_first| *month_first <= last)
        .flat_map(|month_first| {
            let offset = usize::from(month_first.weekday());
            let days = Date::days_in_month(month_first.year(), month_first.month());
            (0..WEEKS).filter_map(move |row| {
                let day = (row * 7).saturating_sub(offset) + 1;
                (day <= usize::from(days))
                    .then(|| Date::new(month_first.year(), month_first.month(), day as u8))
                    .flatten()
            })
        })
        .find(|date| {
            let row_top = layout.row_top(first.year(), *date);
            row_top >= top && row_top + layout.row_height <= bottom
        })
}

impl RenderOnce for RangeCalendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let first_year = (*self.years.start()).clamp(Date::MIN.year(), Date::MAX.year());
        let last_year = (*self.years.end()).clamp(first_year, Date::MAX.year());
        let (Some(first), Some(last)) = (Date::new(first_year, 1, 1), Date::new(last_year, 12, 31))
        else {
            unreachable!("years are clamped to the range of Date");
        };
        let in_years = move |date: &Date| (first..=last).contains(date);
        let months = ((last_year - first_year + 1) * 12) as usize;

        let state: Entity<State> = window.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            focus: cx.focus_handle(),
            active: None,
            anchor: None,
            layout: None,
            first_year: None,
            reveal: false,
        });
        let scroll = match self.scroll {
            Some(scroll) => scroll,
            None => window
                .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| {
                    UniformListScrollHandle::new()
                })
                .read(cx)
                .clone(),
        };
        let fallback = self
            .selection
            .start
            .filter(in_years)
            .or(self.today.filter(in_years))
            .unwrap_or(first);
        // 操作日は最初の描画で決め、その後に選ばれた範囲が変わっても動かさない。
        // 年の範囲の最初の年が変わると、保っているスクロールの位置が別の月を指すため、操作日の月を表示し直す
        let (focus, active) = state.update(cx, |state, _| {
            let active = state.active.filter(in_years).unwrap_or(fallback);
            state.active = Some(active);
            if state.first_year != Some(first_year) {
                state.first_year = Some(first_year);
                state.reveal = true;
                scroll.scroll_to_item_strict(month_index(first_year, active), ScrollStrategy::Top);
            }
            (state.focus.clone(), active)
        });
        let focus_visible = crate::is_focus_visible(&focus, window, cx);

        let enabled = self.enabled.clone();
        let is_enabled = move |date: &Date| enabled.as_ref().is_none_or(|enabled| enabled(date));
        let select = self.on_change.clone().map(|on_change| {
            let is_enabled = is_enabled.clone();
            let selection = self.selection;
            Rc::new(move |date: Date, window: &mut Window, cx: &mut App| {
                if is_enabled(&date) {
                    on_change(&selection.select(date), window, cx);
                }
            }) as SelectFn
        });
        let set_active = {
            let state = state.clone();
            move |date: Date, cx: &mut App| {
                state.update(cx, |state, cx| {
                    state.active = Some(date);
                    cx.notify();
                });
            }
        };

        let on_key_down = {
            let state = state.clone();
            let scroll = scroll.clone();
            let select = select.clone();
            let set_active = set_active.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let modifiers = event.keystroke.modifiers;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
                    return;
                }
                let key = event.keystroke.key.as_str();
                let shift = modifiers.shift;
                let current = state.read(cx).active.filter(in_years).unwrap_or(fallback);
                let confirm = matches!(key, "enter" | "space") && !shift;
                let months = match (key, shift) {
                    ("pagedown", false) => Some(1),
                    ("pageup", false) => Some(-1),
                    ("pagedown", true) => Some(12),
                    ("pageup", true) => Some(-12),
                    _ => None,
                };
                let weekday = i64::from(current.weekday());
                if !confirm
                    && months.is_none()
                    && (shift || !matches!(key, "right" | "left" | "down" | "up" | "home" | "end"))
                {
                    return;
                }
                cx.stop_propagation();
                // 押し続けたときの繰り返しで選ぶと、始まりと終わりが交互に入れ替わる
                if confirm && event.is_held {
                    return;
                }
                // 操作日の週がスクロールで見えなくなっていれば、見えている日付に移すだけにする
                if let Some(visible) =
                    retarget(&scroll, state.read(cx).layout, first, last, current)
                {
                    set_active(visible, cx);
                    return;
                }
                if confirm {
                    if let Some(select) = &select {
                        select(current, window, cx);
                    }
                    return;
                }
                let base_day = state
                    .read(cx)
                    .anchor
                    .filter(|&(landed, _)| landed == current)
                    .map_or(current.day(), |(_, base_day)| base_day);
                let target = match (months, key) {
                    (Some(months), _) => current.add_months(months).and_then(|target| {
                        Date::new(
                            target.year(),
                            target.month(),
                            base_day.min(Date::days_in_month(target.year(), target.month())),
                        )
                    }),
                    (None, "right") => current.add_days(1),
                    (None, "left") => current.add_days(-1),
                    (None, "down") => current.add_days(7),
                    (None, "up") => current.add_days(-7),
                    (None, "home") => current.add_days(-weekday),
                    (None, "end") => current.add_days(6 - weekday),
                    _ => None,
                };
                let Some(target) = target.filter(in_years) else {
                    return;
                };
                let layout = state.update(cx, |state, cx| {
                    state.active = Some(target);
                    state.anchor = months.map(|_| (target, base_day));
                    cx.notify();
                    state.layout
                });
                reveal(&scroll, first_year, layout, target);
            }
        };

        let shared = Rc::new(Shared {
            id: self.id.clone(),
            first,
            selection: self.selection,
            today: self.today,
            active,
            focus: focus.clone(),
            focus_visible,
            is_enabled: Rc::new(is_enabled),
            select,
            set_active: Rc::new(set_active),
            header: self.header,
            grid: self.grid,
            week: self.week,
            day: self.day,
            blank: self.blank,
            state: state.clone(),
            scroll: scroll.clone(),
        });
        let list = uniform_list(
            (self.id.clone(), "months"),
            months,
            move |range, window, cx| {
                range
                    .map(|index| shared.render_month(index, window, cx))
                    .collect::<Vec<_>>()
            },
        )
        .track_scroll(&scroll)
        .flex_1()
        .min_h_0();

        self.inner
            .id(self.id)
            .track_focus(&focus.tab_stop(true))
            .role(accesskit::Role::Group)
            .on_key_down(on_key_down)
            .flex()
            .flex_col()
            .child(list)
    }
}

/// 月の要素を作るために、描画の後も一覧が持つ値
struct Shared {
    id: ElementId,
    first: Date,
    selection: DateRange,
    today: Option<Date>,
    active: Date,
    focus: FocusHandle,
    focus_visible: bool,
    is_enabled: EnabledFn,
    select: Option<SelectFn>,
    set_active: Rc<SetActiveFn>,
    header: Option<HeaderFn>,
    grid: Option<GridFn>,
    week: Option<WeekFn>,
    day: Option<DayFn>,
    blank: Option<BlankFn>,
    state: Entity<State>,
    scroll: UniformListScrollHandle,
}

impl Shared {
    fn render_month(&self, index: usize, window: &mut Window, cx: &mut App) -> AnyElement {
        let month_first = self
            .first
            .add_months(index as i64)
            .expect("months are within the years");
        let (year, month) = (month_first.year(), month_first.month());
        let days = Date::days_in_month(year, month);
        let offset = usize::from(month_first.weekday());
        let date_at = |cell: usize| {
            cell.checked_sub(offset)
                .filter(|&day| day < usize::from(days))
                .and_then(|day| Date::new(year, month, day as u8 + 1))
        };
        let selection = self.selection;
        let header = self
            .header
            .as_ref()
            .map(|build| build(&month_first, window, cx));
        let mut weeks = Vec::with_capacity(WEEKS);
        for row in 0..WEEKS {
            let dates: Vec<Option<Date>> = (0..7).map(|column| date_at(row * 7 + column)).collect();
            let covered: Vec<(usize, Date)> = dates
                .iter()
                .enumerate()
                .filter_map(|(column, date)| {
                    date.filter(|date| selection.contains(date))
                        .map(|date| (column, date))
                })
                .collect();
            let band = covered.first().zip(covered.last()).map(
                |(&(first, first_date), &(last, last_date))| RangeBand {
                    first,
                    last,
                    from_start: selection.start == Some(first_date),
                    to_end: selection.end == Some(last_date),
                },
            );
            let cells = dates
                .iter()
                .enumerate()
                .map(|(column, date)| {
                    let Some(date) = *date else {
                        let blank = div();
                        return match &self.blank {
                            Some(build) => build(blank),
                            None => blank,
                        }
                        .into_any_element();
                    };
                    self.render_day(date, column, window, cx)
                })
                .collect::<Vec<_>>();
            let week = div().flex();
            let week = match &self.week {
                Some(build) => build(&RangeWeek { band }, week),
                None => week,
            };
            weeks.push(
                week.id(("week", row))
                    .role(accesskit::Role::Row)
                    .aria_row_index(row)
                    .children(cells),
            );
        }
        // 週の行の位置と高さ。格子の子を描いた後に、月の大きさと合わせて状態に入れる
        let rows = Rc::new(Cell::new(None));
        let grid = div()
            .on_children_prepainted({
                let rows = rows.clone();
                move |bounds: Vec<Bounds<Pixels>>, _, _| {
                    if let [first, second, ..] = bounds[..] {
                        rows.set(Some((
                            first.top(),
                            second.top() - first.top(),
                            first.size.height,
                        )));
                    }
                }
            })
            .id("grid");
        let grid = match &self.grid {
            Some(build) => build(&month_first, grid),
            None => grid,
        };
        let state = self.state.clone();
        let scroll = self.scroll.clone();
        let first_year = self.first.year();
        let content = div().flex().flex_col().children(header).child(
            grid.role(accesskit::Role::Grid)
                .aria_row_count(WEEKS)
                .aria_column_count(7)
                .flex()
                .flex_col()
                .children(weeks),
        );
        // 操作日の週を見える位置までスクロールするために、月と週の行の大きさを測る。
        // 子の余白も月の高さに含めるため、月の中身を包んだ要素から測る
        div()
            .child(content)
            .on_children_prepainted(move |bounds: Vec<Bounds<Pixels>>, window, cx| {
                let (Some(month), Some((row_top, row, row_height))) = (bounds.first(), rows.get())
                else {
                    return;
                };
                let layout = MonthLayout {
                    month: month.size.height,
                    grid_top: row_top - month.top(),
                    row,
                    row_height,
                };
                let reveal_active = state.update(cx, |state, _| {
                    state.layout = Some(layout);
                    std::mem::take(&mut state.reveal)
                        .then_some(state.active)
                        .flatten()
                });
                if let Some(active) = reveal_active {
                    reveal(&scroll, first_year, Some(layout), active);
                    window.refresh();
                }
            })
            .id((self.id.clone(), format!("month-{year}-{month}")))
            .into_any_element()
    }

    fn render_day(
        &self,
        date: Date,
        column: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let selection = self.selection;
        let active = self.active == date;
        let state = RangeDay {
            date,
            start: selection.start == Some(date),
            end: selection.end == Some(date),
            in_range: selection.contains(&date),
            enabled: (self.is_enabled)(&date),
            today: self.today == Some(date),
            focus_visible: active && self.focus_visible,
        };
        let cell = div().id(("day", usize::from(date.day())));
        let cell = match &self.day {
            Some(build) => build(&state, cell, window, cx),
            None => cell,
        };
        let set_active = self.set_active.clone();
        let focus = self.focus.clone();
        cell.role(accesskit::Role::GridCell)
            .aria_column_index(column)
            .aria_selected(state.start || state.end)
            .when(active, |this| this.aria_active_descendant())
            .on_mouse_down(MouseButton::Left, {
                let set_active = set_active.clone();
                move |_, _, cx| set_active(date, cx)
            })
            .on_a11y_action(accesskit::Action::Focus, move |_, window, cx| {
                window.focus(&focus, cx);
                set_active(date, cx);
            })
            .when_some(
                self.select.clone().filter(|_| state.enabled),
                |this, select| {
                    let on_click = select.clone();
                    this.on_click(move |_: &ClickEvent, window, cx| on_click(date, window, cx))
                        .on_a11y_action(accesskit::Action::Click, move |_, window, cx| {
                            select(date, window, cx)
                        })
                },
            )
            .a11y_info(move |node: &mut accesskit::Node| {
                if !state.enabled {
                    node.set_disabled();
                }
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::test_window::add_test_window;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("テストの日付が有効であること")
    }

    /// (0, 0) に置いたカレンダー。既定では高さ 300 で、月は高さ 20 の見出しと 40x40 のセルで、高さ 260
    struct RangeView {
        selection: DateRange,
        years: RangeInclusive<i32>,
        height: f32,
        /// 格子の下の余白
        grid_margin: f32,
        scroll: UniformListScrollHandle,
        before: FocusHandle,
        after: FocusHandle,
        changes: Rc<RefCell<Vec<DateRange>>>,
        /// フォーカスがあることを表示している日付
        visible: Rc<Cell<Option<Date>>>,
        /// 描いた週の範囲の列
        bands: Rc<RefCell<Vec<RangeBand>>>,
    }

    impl Render for RangeView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let visible = self.visible.clone();
            visible.set(None);
            let bands = self.bands.clone();
            bands.borrow_mut().clear();
            div()
                .size_full()
                .child(div().id("before").track_focus(&self.before).h(px(0.)))
                .child(
                    RangeCalendar::new("range", self.years.clone(), self.selection)
                        // 2026 年 1 月 15 日は選べない
                        .enabled(|day| *day != date(2026, 1, 15))
                        .scroll(&self.scroll)
                        .on_change(cx.listener(|this, range: &DateRange, _, cx| {
                            this.changes.borrow_mut().push(*range);
                            this.selection = *range;
                            cx.notify();
                        }))
                        .header(|_, _, _| div().h(px(20.)).into_any_element())
                        .grid({
                            let margin = self.grid_margin;
                            move |_, grid| grid.mb(px(margin))
                        })
                        .week(move |week, row| {
                            bands.borrow_mut().extend(week.band);
                            row
                        })
                        .day(move |day, cell, _, _| {
                            if day.focus_visible {
                                visible.set(Some(day.date));
                            }
                            cell.size(px(40.))
                        })
                        .blank(|blank| blank.size(px(40.)))
                        .w(px(280.))
                        .h(px(self.height)),
                )
                .child(div().id("after").track_focus(&self.after).h(px(10.)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selection: DateRange,
    ) -> (gpui::Entity<RangeView>, &mut VisualTestContext) {
        setup_with(cx, selection, |_| {})
    }

    fn setup_with(
        cx: &mut TestAppContext,
        selection: DateRange,
        configure: impl FnOnce(&mut RangeView),
    ) -> (gpui::Entity<RangeView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| {
            let mut view = RangeView {
                selection,
                years: 2026..=2027,
                height: 300.,
                grid_margin: 0.,
                scroll: UniformListScrollHandle::new(),
                before: cx.focus_handle().tab_stop(true),
                after: cx.focus_handle().tab_stop(true),
                changes: Rc::default(),
                visible: Rc::default(),
                bands: Rc::default(),
            };
            configure(&mut view);
            view
        });
        cx.run_until_parked();
        (view, cx)
    }

    fn range(start: Option<Date>, end: Option<Date>) -> DateRange {
        DateRange { start, end }
    }

    fn visible(view: &gpui::Entity<RangeView>, cx: &mut VisualTestContext) -> Option<Date> {
        view.read_with(cx, |view, _| view.visible.get())
    }

    fn changes(view: &gpui::Entity<RangeView>, cx: &mut VisualTestContext) -> Vec<DateRange> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn scroll_top(view: &gpui::Entity<RangeView>, cx: &mut VisualTestContext) -> f32 {
        let scroll = view.read_with(cx, |view, _| view.scroll.clone());
        -f32::from(scroll.0.borrow().base_handle.offset().y)
    }

    /// 一番上に表示している月の `row` 週目の `column` 列のセルの中央
    fn cell_center(row: usize, column: usize) -> gpui::Point<gpui::Pixels> {
        gpui::point(
            px(40. * column as f32 + 20.),
            px(20. + 40. * row as f32 + 20.),
        )
    }

    #[test]
    fn 始まりだけのときに後の日付を選ぶと終わりにし_それ以外は始まりから選び直す() {
        let start = date(2026, 1, 10);
        assert_eq!(DateRange::default().select(start), range(Some(start), None));
        assert_eq!(
            range(Some(start), None).select(date(2026, 1, 10)),
            range(Some(start), Some(start)),
            "同じ日も終わりにすること"
        );
        assert_eq!(
            range(Some(start), None).select(date(2026, 1, 9)),
            range(Some(date(2026, 1, 9)), None)
        );
        assert_eq!(
            range(Some(start), Some(date(2026, 1, 20))).select(date(2026, 1, 15)),
            range(Some(date(2026, 1, 15)), None)
        );
    }

    #[gpui::test]
    fn 始まりの月を一番上に表示する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, range(Some(date(2027, 3, 10)), None));
        assert_eq!(scroll_top(&view, cx), 260. * 14.);
    }

    #[gpui::test]
    fn tab_ではカレンダーに_1_回だけ到達する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, DateRange::default());
        cx.simulate_keystrokes("tab tab");
        assert_eq!(visible(&view, cx), Some(date(2026, 1, 1)));
        cx.simulate_keystrokes("tab");
        let after = view.read_with(cx, |view, _| view.after.clone());
        assert!(cx.update(|window, _| after.is_focused(window)));
    }

    #[gpui::test]
    fn キーで操作日を移し_enter_で範囲を選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, DateRange::default());
        cx.simulate_keystrokes("tab tab right enter");
        assert_eq!(visible(&view, cx), Some(date(2026, 1, 2)));
        cx.simulate_keystrokes("down down enter");
        assert_eq!(
            changes(&view, cx),
            [
                range(Some(date(2026, 1, 2)), None),
                range(Some(date(2026, 1, 2)), Some(date(2026, 1, 16)))
            ]
        );
        // 選べない日付は選ばない
        cx.simulate_keystrokes("left enter");
        assert_eq!(changes(&view, cx).len(), 2);
    }

    #[gpui::test]
    fn 範囲にかかる週の列を渡す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, range(Some(date(2026, 1, 8)), Some(date(2026, 1, 20))));
        // 2026 年 1 月は木曜日から始まる
        // 一覧は 1 回の描画で同じ月を複数回作ることがあるため、重複を除いて比べる
        assert_eq!(
            view.read_with(cx, |view, _| view
                .bands
                .borrow()
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()),
            std::collections::HashSet::from([
                RangeBand {
                    first: 4,
                    last: 6,
                    from_start: true,
                    to_end: false
                },
                RangeBand {
                    first: 0,
                    last: 6,
                    from_start: false,
                    to_end: false
                },
                RangeBand {
                    first: 0,
                    last: 2,
                    from_start: false,
                    to_end: true
                },
            ])
        );
    }

    #[gpui::test]
    fn 月をまたいで移ると移った週が見えるまでスクロールする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, range(Some(date(2026, 1, 31)), None));
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(visible(&view, cx), Some(date(2026, 2, 7)));
        // 2 月 1 日は日曜日なので 7 日は 1 週目。月の高さ 260、見出し 20、行 40 で、下端 320 が見える位置
        assert_eq!(scroll_top(&view, cx), 20.);
    }

    #[gpui::test]
    fn 月を移ると元の日を基準に月末に寄せる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, range(Some(date(2026, 1, 31)), None));
        cx.simulate_keystrokes("tab tab pagedown");
        assert_eq!(visible(&view, cx), Some(date(2026, 2, 28)));
        cx.simulate_keystrokes("pagedown");
        assert_eq!(visible(&view, cx), Some(date(2026, 3, 31)));
    }

    #[gpui::test]
    fn 操作日の月が見えないときのキー操作は見えている月の_1_日に移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, DateRange::default());
        cx.simulate_keystrokes("tab tab");
        let scroll = view.read_with(cx, |view, _| view.scroll.clone());
        scroll
            .0
            .borrow()
            .base_handle
            .set_offset(gpui::point(px(0.), px(-260. * 5.)));
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        cx.simulate_keystrokes("right");
        assert_eq!(visible(&view, cx), Some(date(2026, 6, 1)));
    }

    #[gpui::test]
    fn 日付を押すと選び_操作日にする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, DateRange::default());
        // 1 月 1 日は木曜日
        cx.simulate_click(cell_center(0, 5), Modifiers::none());
        assert_eq!(changes(&view, cx), [range(Some(date(2026, 1, 2)), None)]);
        cx.simulate_keystrokes("right");
        assert_eq!(visible(&view, cx), Some(date(2026, 1, 3)));
    }

    #[gpui::test]
    fn 最初に操作日の週が隠れていれば見えるまでスクロールする(
        cx: &mut TestAppContext,
    ) {
        // 2026 年 1 月 31 日は 5 週目 (上端 180、下端 220)
        let (view, cx) = setup_with(cx, range(Some(date(2026, 1, 31)), None), |view| {
            view.height = 200.
        });
        assert_eq!(scroll_top(&view, cx), 20.);
    }

    #[gpui::test]
    fn 格子の余白も月の高さに含めてスクロールする(cx: &mut TestAppContext) {
        // 月の高さは 270 になる
        let (view, cx) = setup_with(cx, range(Some(date(2026, 1, 31)), None), |view| {
            view.grid_margin = 10.
        });
        cx.simulate_keystrokes("tab tab down");
        assert_eq!(visible(&view, cx), Some(date(2026, 2, 7)));
        // 2 月の 1 週目は 270 + 20 = 290 から 330
        assert_eq!(scroll_top(&view, cx), 30.);
    }

    #[gpui::test]
    fn 操作日の週が見えないときの_enter_は選ばずに見えている日付に移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, DateRange::default());
        cx.simulate_keystrokes("tab tab");
        let scroll = view.read_with(cx, |view, _| view.scroll.clone());
        scroll
            .0
            .borrow()
            .base_handle
            .set_offset(gpui::point(px(0.), px(-260. * 5.)));
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert!(changes(&view, cx).is_empty());
        assert_eq!(visible(&view, cx), Some(date(2026, 6, 1)));
        cx.simulate_keystrokes("enter");
        assert_eq!(changes(&view, cx), [range(Some(date(2026, 6, 1)), None)]);
    }

    #[gpui::test]
    fn enter_を押し続けても_1_回だけ選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, DateRange::default());
        cx.simulate_keystrokes("tab tab");
        let enter = gpui::Keystroke::parse("enter").expect("キーを解釈できること");
        for is_held in [false, true, true] {
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: enter.clone(),
                is_held,
                prefer_character_input: false,
            });
        }
        assert_eq!(changes(&view, cx), [range(Some(date(2026, 1, 1)), None)]);
    }

    #[gpui::test]
    fn 最初の年が変わると操作日の月を表示し直す(cx: &mut TestAppContext) {
        let (view, cx) = setup_with(cx, range(Some(date(2027, 6, 10)), None), |view| {
            view.years = 2026..=2028
        });
        assert_eq!(scroll_top(&view, cx), 260. * 17.);
        view.update(cx, |view, cx| {
            view.years = 2027..=2028;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(scroll_top(&view, cx), 260. * 5.);
        cx.simulate_keystrokes("tab tab right");
        assert_eq!(visible(&view, cx), Some(date(2027, 6, 11)));
    }

    #[gpui::test]
    fn 操作日は呼び出し側が始まりを変えても動かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, range(Some(date(2026, 1, 10)), None));
        view.update(cx, |view, cx| {
            view.selection = range(Some(date(2026, 1, 20)), None);
            cx.notify();
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab tab");
        assert_eq!(visible(&view, cx), Some(date(2026, 1, 10)));
    }
}
