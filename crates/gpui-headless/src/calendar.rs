//! 1 か月の日付から 1 つを選ぶ、見た目を持たないカレンダー

use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::accessibility::A11yInfoExt;
use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, MouseButton, ParentElement, RenderOnce, Stateful, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, accesskit, div, prelude::FluentBuilder,
};

/// グレゴリオ暦の日付
///
/// 年は 1 から 9999、月は 1 から 12、日は 1 からその月の日数までの値を持つ
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// 表せる最も前の日付。1 年 1 月 1 日
    pub const MIN: Self = Self {
        year: 1,
        month: 1,
        day: 1,
    };
    /// 表せる最も後の日付。9999 年 12 月 31 日
    pub const MAX: Self = Self {
        year: 9999,
        month: 12,
        day: 31,
    };

    /// `year` 年 `month` 月 `day` 日を作る。存在しない日付と、[`Date::MIN`] から [`Date::MAX`] の外なら `None`
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((Self::MIN.year..=Self::MAX.year).contains(&year)
            && (1..=12).contains(&month)
            && (1..=Self::days_in_month(year, month)).contains(&day))
        .then_some(Self { year, month, day })
    }

    /// 年
    pub fn year(self) -> i32 {
        self.year
    }

    /// 月。1 から 12
    pub fn month(self) -> u8 {
        self.month
    }

    /// 日。1 からその月の日数まで
    pub fn day(self) -> u8 {
        self.day
    }

    /// `year` 年 `month` 月の日数。`month` は 1 から 12 にすること
    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// 曜日。日曜日を 0、土曜日を 6 とする
    pub fn weekday(self) -> u8 {
        // 1970 年 1 月 1 日は木曜日
        (self.days_since_epoch() + 4).rem_euclid(7) as u8
    }

    /// `days` 日後の日付。負の値なら前の日付。[`Date::MIN`] から [`Date::MAX`] の外になる場合は `None`
    pub fn add_days(self, days: i64) -> Option<Self> {
        let target = self.days_since_epoch().checked_add(days)?;
        (Self::MIN.days_since_epoch()..=Self::MAX.days_since_epoch())
            .contains(&target)
            .then(|| Self::from_days_since_epoch(target))
    }

    /// `months` か月後の同じ日。その月に同じ日が無ければ月末にする。負の値なら前の月。
    /// [`Date::MIN`] から [`Date::MAX`] の外になる場合は `None`
    pub fn add_months(self, months: i64) -> Option<Self> {
        let index = (i64::from(self.year) * 12 + i64::from(self.month) - 1).checked_add(months)?;
        let year = i32::try_from(index.div_euclid(12)).ok()?;
        let month = index.rem_euclid(12) as u8 + 1;
        Self::new(year, month, self.day.min(Self::days_in_month(year, month)))
    }

    // 日付と日数の変換は Howard Hinnant の days_from_civil と civil_from_days
    // (http://howardhinnant.github.io/date_algorithms.html) による
    fn days_since_epoch(self) -> i64 {
        let month = i64::from(self.month);
        let year = i64::from(self.year) - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5
            + i64::from(self.day)
            - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    fn from_days_since_epoch(days: i64) -> Self {
        let days = days + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        };
        Self {
            year: (year_of_era + era * 400 + i64::from(month <= 2)) as i32,
            month: month as u8,
            day: day as u8,
        }
    }
}

/// [`Calendar::day`] に渡す、日付のセルの状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDay {
    /// セルの日付
    pub date: Date,
    /// 選ばれている日付か
    pub selected: bool,
    /// 選べる日付か ([`Calendar::enabled`])
    pub enabled: bool,
    /// [`Calendar::today`] の日付か
    pub today: bool,
    /// フォーカスがあるか
    pub focused: bool,
    /// フォーカスの表示が必要か ([`crate::is_focus_visible`])
    pub focus_visible: bool,
}

type ChangeHandler = Rc<dyn Fn(&Date, &mut Window, &mut App)>;
type EnabledFn = Rc<dyn Fn(&Date) -> bool>;
type DayFn = Box<dyn Fn(&CalendarDay, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>>;
type WeekdayFn = Box<dyn Fn(u8, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>>;
type BlankFn = Box<dyn Fn(Div) -> Div>;

/// 1 か月の日付から 1 つを選ぶ、見た目を持たないカレンダー
///
/// 表示する月と選ばれている日付は呼び出し側が持ち、[`Calendar::new`] と [`Calendar::selected`] に毎回渡す。
/// カレンダー自身は月も日付も変えない。
/// 曜日の見出しの行と、日曜日から始まる 6 週の行を並べる。月の外の日はセルを空にする
///
/// - Tab で到達できる日付は 1 つだけで、フォーカスのある日付、選ばれている日付、[`Calendar::today`]、
///   月の最初の日の順に、表示している月の [`Calendar::range`] の中から選ぶ。範囲の中に日付が無ければ到達できない
/// - 表示する月か [`Calendar::range`] が変わって、フォーカスのある日付が変わるか範囲の外になった場合は、
///   Tab で到達できる日付にフォーカスを移す。範囲の中に日付が無ければ格子自身にフォーカスを移し、
///   次の Tab はカレンダーの後ろへ進む
/// - フォーカス中の左右の矢印キーで前後の日、上下の矢印キーで前後の週の同じ曜日、Home と End で週の始まりと終わり、
///   Page Up と Page Down で前後の月、Shift を押しながら Page Up と Page Down で前後の年の同じ日に、フォーカスだけを移す。
///   月か年を移って同じ日が無ければ月末にし、続けて月か年を移るときは元の日を基準にする。
///   移る先が表示している月の外なら、その月を表示するよう [`Calendar::on_month_change`] で求める。
///   設定しない場合と、移る先が [`Calendar::range`] の外の場合は移らない
/// - フォーカス中の Enter・Space、クリック、支援技術からのクリック操作で、日付を選ぶよう [`Calendar::on_change`] で求める。
///   選べない日付 ([`Calendar::enabled`]) は求めない
///
/// 支援技術には role `grid` と、行ごとの role `row`、曜日の見出しの role `columnheader`、
/// 日付ごとの role `gridcell` と選ばれているかどうか・選べるかどうかとして公開される。
/// カレンダーと日付の名前は `aria_label` で与えること。
/// 見た目は [`Calendar::day`]・[`Calendar::weekday`]・[`Calendar::blank`] と、[`Styled`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    inner: Stateful<Div>,
    year: i32,
    month: u8,
    selected: Option<Date>,
    today: Option<Date>,
    range: Option<RangeInclusive<Date>>,
    enabled: Option<EnabledFn>,
    on_change: Option<ChangeHandler>,
    on_month_change: Option<ChangeHandler>,
    day: Option<DayFn>,
    weekday: Option<WeekdayFn>,
    blank: Option<BlankFn>,
}

impl Calendar {
    /// `month` の日付がある月を表示するカレンダーを作る。日は使わない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、日付のフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, month: Date) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            year: month.year,
            month: month.month,
            selected: None,
            today: None,
            range: None,
            enabled: None,
            on_change: None,
            on_month_change: None,
            day: None,
            weekday: None,
            blank: None,
        }
    }

    /// `date` を選ばれた日付として表示する
    pub fn selected(mut self, date: Option<Date>) -> Self {
        self.selected = date;
        self
    }

    /// 今日の日付を `date` にする。選ばれている日付が表示している月に無い場合に、Tab で到達できる日付になる
    pub fn today(mut self, date: Option<Date>) -> Self {
        self.today = date;
        self
    }

    /// フォーカスを移せる日付と選べる日付を `range` の中に限る。範囲の外へ移るキー操作では何もしない。
    /// 範囲の外の日付は、フォーカスを持たず、選べない日付として公開する。既定では限らない
    pub fn range(mut self, range: RangeInclusive<Date>) -> Self {
        self.range = Some(range);
        self
    }

    /// `enabled` が `false` を返す日付を選べなくする。既定ではすべての日付を選べる
    ///
    /// 選べない日付にもフォーカスは移る
    pub fn enabled(mut self, enabled: impl Fn(&Date) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    /// 日付を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた日付を渡す。選ばれている日付でも呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Calendar::selected`] に渡すこと。
    /// 設定しない場合、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// キー操作で表示している月の外へフォーカスを移すときに、`handler` に移る先の日付を渡す。
    /// 複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 次の描画でその日付の月を [`Calendar::new`] に渡すと、その日付にフォーカスが移る。
    /// 渡さなかった場合と、それまでにフォーカスがカレンダーの日付の外へ移った場合は移さない
    pub fn on_month_change(
        mut self,
        handler: impl Fn(&Date, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_month_change = Some(Rc::new(handler));
        self
    }

    /// 日付のセルを `day` で作る
    ///
    /// `day` には日付の状態と、id を付けたセルの要素を渡す。返した要素をセルとして置く。
    /// 日付の名前はここで `aria_label` で付けること
    pub fn day(
        mut self,
        day: impl Fn(&CalendarDay, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.day = Some(Box::new(day));
        self
    }

    /// 曜日の見出しのセルを `weekday` で作る
    ///
    /// `weekday` には曜日 (日曜日を 0 とする) と、id を付けたセルの要素を渡す。
    /// 曜日の名前はここで `aria_label` で付けること
    pub fn weekday(
        mut self,
        weekday: impl Fn(u8, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.weekday = Some(Box::new(weekday));
        self
    }

    /// 月の外の日の空のセルを `blank` で整える。日付のセルと大きさをそろえるのに使う
    pub fn blank(mut self, blank: impl Fn(Div) -> Div + 'static) -> Self {
        self.blank = Some(Box::new(blank));
        self
    }
}

/// 日付のフォーカスを、月をまたいで同じ日のものを使い回す。日付ごとに 1 から 31 の日で分ける
fn day_focus_handles(id: &ElementId, window: &mut Window, cx: &mut App) -> Vec<FocusHandle> {
    (1..=31usize)
        .map(|day| {
            window
                .use_keyed_state((id.clone(), format!("day-{day}")), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone()
        })
        .collect()
}

/// 週の行の数
const WEEKS: usize = 6;

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handles: Rc<[FocusHandle]> = day_focus_handles(&self.id, window, cx).into();
        let (year, month) = (self.year, self.month);
        let days = Date::days_in_month(year, month);
        let first = Date {
            year,
            month,
            day: 1,
        };
        let offset = usize::from(first.weekday());
        let in_month = |date: Option<Date>| {
            date.filter(|date| date.year == year && date.month == month)
                .map(|date| date.day)
        };
        let in_range = |day: u8| {
            self.range
                .as_ref()
                .is_none_or(|range| range.contains(&Date { year, month, day }))
        };
        // フォーカスが無いときに Tab で到達できる日付。範囲の中に日付が無い月では無い
        let fallback_day = in_month(self.selected)
            .filter(|&day| in_range(day))
            .or(in_month(self.today).filter(|&day| in_range(day)))
            .or((1..=days).find(|&day| in_range(day)));

        // 月の外へ移るキー操作で、月が変わった後にフォーカスを移す日付。その次の描画で使い終える
        let pending = window.use_keyed_state((self.id.clone(), "pending"), cx, |_, _| None::<Date>);
        let pending_target = pending
            .update(cx, |pending, _| pending.take())
            .filter(|target| (target.year, target.month) == (year, month));
        // 日付にフォーカスできなくなったときに、フォーカスを置く格子のフォーカス。Tab では到達しない
        let grid_focus = window
            .use_keyed_state((self.id.clone(), "grid"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        // Page Up・Page Down で移る基準の日。月末に寄せる前の日を持つ
        let anchor = window.use_keyed_state((self.id.clone(), "anchor"), cx, |_, _| None::<u8>);
        // 前の描画で表示していた月
        let shown = window.use_keyed_state((self.id.clone(), "shown"), cx, |_, _| (year, month));
        let month_changed = shown.update(cx, |shown, _| {
            std::mem::replace(shown, (year, month)) != (year, month)
        });
        // 日付のフォーカスは日ごとに月をまたいで使い回すため、フォーカスのある日付が月や範囲の変更で
        // 別の日付や範囲の外の日付になっていれば、範囲の中の日付に移す。
        // 呼び出し側が日付の外へフォーカスを移していれば何もしない
        if let Some(index) = handles.iter().position(|handle| handle.is_focused(window)) {
            let day = (index + 1) as u8;
            let valid = !month_changed && day <= days && in_range(day);
            let next = match pending_target {
                Some(target) => Some(target.day)
                    .filter(|&day| in_range(day))
                    .or(fallback_day),
                None if valid => Some(day),
                None => fallback_day,
            };
            match next {
                Some(next) if next != day => window.focus(&handles[usize::from(next) - 1], cx),
                Some(_) => {}
                // フォーカスを外すと次の Tab がウィンドウの先頭から始まるため、格子に移して Tab の位置を保つ
                None => window.focus(&grid_focus, cx),
            }
        } else if grid_focus.is_focused(window)
            && let Some(day) = fallback_day
        {
            // 格子に退避している間に日付が再び使えるようになったら、日付に戻す
            window.focus(&handles[usize::from(day) - 1], cx);
        } else {
            // 日付の外へフォーカスが移ったら、月の移動の基準日を捨てる
            anchor.update(cx, |anchor, _| *anchor = None);
        }
        let focused_day = (1..=days)
            .find(|&day| in_range(day) && handles[usize::from(day) - 1].is_focused(window));
        let tab_day = focused_day.or(fallback_day);
        let enabled = self.enabled.clone();
        let is_enabled = move |date: &Date| enabled.as_ref().is_none_or(|enabled| enabled(date));
        let select = self.on_change.clone().map(|on_change| {
            let is_enabled = is_enabled.clone();
            Rc::new(move |date: Date, window: &mut Window, cx: &mut App| {
                if is_enabled(&date) {
                    on_change(&date, window, cx);
                }
            })
        });

        let on_key_down = {
            let handles = handles.clone();
            let on_month_change = self.on_month_change.clone();
            let range = self.range.clone();
            let pending = pending.clone();
            let anchor = anchor.clone();
            move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
                let Some(day) =
                    (1..=days).find(|&day| handles[usize::from(day) - 1].is_focused(window))
                else {
                    return;
                };
                let current = Date { year, month, day };
                let modifiers = event.keystroke.modifiers;
                let shift = modifiers.shift;
                if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
                    return;
                }
                let weekday = i64::from(current.weekday());
                let months = match (event.keystroke.key.as_str(), shift) {
                    ("pagedown", false) => Some(1),
                    ("pageup", false) => Some(-1),
                    ("pagedown", true) => Some(12),
                    ("pageup", true) => Some(-12),
                    _ => None,
                };
                // 月か年を続けて移るときは、短い月で月末に寄せる前の日を基準にする
                let base_day = anchor
                    .read(cx)
                    .filter(|&anchor| anchor.min(days) == day)
                    .unwrap_or(day);
                let target = match (months, event.keystroke.key.as_str(), shift) {
                    (Some(months), _, _) => current.add_months(months).map(|target| Date {
                        day: base_day.min(Date::days_in_month(target.year, target.month)),
                        ..target
                    }),
                    (None, "right", false) => current.add_days(1),
                    (None, "left", false) => current.add_days(-1),
                    (None, "down", false) => current.add_days(7),
                    (None, "up", false) => current.add_days(-7),
                    (None, "home", false) => current.add_days(-weekday),
                    (None, "end", false) => current.add_days(6 - weekday),
                    _ => return,
                };
                cx.stop_propagation();
                let Some(target) = target
                    .filter(|target| range.as_ref().is_none_or(|range| range.contains(target)))
                else {
                    return;
                };
                anchor.update(cx, |anchor, _| *anchor = months.map(|_| base_day));
                if (target.year, target.month) == (year, month) {
                    window.focus(&handles[usize::from(target.day) - 1], cx);
                    return;
                }
                let Some(on_month_change) = &on_month_change else {
                    return;
                };
                pending.update(cx, |pending, _| *pending = Some(target));
                on_month_change(&target, window, cx);
                // 呼び出し側が月の変更を見送った場合も次の描画で保留を使い終え、後から別の経路で
                // 月が変わったときに使わないよう、描画を求める
                window.refresh();
            }
        };

        let weekday_row = (0..7u8)
            .map(|weekday| {
                let cell = div().id(("weekday", usize::from(weekday)));
                let cell = match &self.weekday {
                    Some(build) => build(weekday, cell, window, cx),
                    None => cell,
                };
                cell.role(accesskit::Role::ColumnHeader)
                    .aria_column_index(usize::from(weekday))
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        let mut rows: Vec<AnyElement> = vec![
            div()
                .id("weekdays")
                .role(accesskit::Role::Row)
                .aria_row_index(0)
                .flex()
                .children(weekday_row)
                .into_any_element(),
        ];
        for week in 0..WEEKS {
            let cells = (0..7usize)
                .map(|column| {
                    let index = week * 7 + column;
                    let Some(day) = index
                        .checked_sub(offset)
                        .map(|day| day + 1)
                        .filter(|&day| day <= usize::from(days))
                    else {
                        let blank = div();
                        return match &self.blank {
                            Some(blank_fn) => blank_fn(blank),
                            None => blank,
                        }
                        .into_any_element();
                    };
                    let date = Date {
                        year,
                        month,
                        day: day as u8,
                    };
                    let state = CalendarDay {
                        date,
                        selected: self.selected == Some(date),
                        enabled: in_range(date.day) && is_enabled(&date),
                        today: self.today == Some(date),
                        focused: handles[day - 1].is_focused(window),
                        focus_visible: crate::is_focus_visible(&handles[day - 1], window, cx),
                    };
                    // 押した要素の状態を、月が変わった後の同じ位置の日付に引き継がないよう、id に年月を含める
                    let cell =
                        div().id(ElementId::Name(format!("day-{year}-{month}-{day}").into()));
                    let cell = match &self.day {
                        Some(build) => build(&state, cell, window, cx),
                        None => cell,
                    };
                    cell.role(accesskit::Role::GridCell)
                        .aria_column_index(column)
                        .aria_selected(state.selected)
                        .when(in_range(date.day), |this| {
                            this.track_focus(&handles[day - 1].clone().tab_stop(
                                tab_day.is_some_and(|tab_day| usize::from(tab_day) == day),
                            ))
                        })
                        .when_some(select.clone().filter(|_| state.enabled), |this, select| {
                            let on_click = select.clone();
                            this.on_click(move |_: &ClickEvent, window, cx| {
                                on_click(date, window, cx)
                            })
                            .on_a11y_action(accesskit::Action::Click, move |_, window, cx| {
                                select(date, window, cx)
                            })
                        })
                        .when(!state.enabled, |this| {
                            let focus_handle = in_range(date.day).then(|| handles[day - 1].clone());
                            // 押下を親に伝えず、親の on_click を呼ばせない。
                            // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                if let Some(focus_handle) = &focus_handle {
                                    window.focus(focus_handle, cx);
                                }
                                cx.stop_propagation();
                            })
                        })
                        .a11y_info(move |node: &mut accesskit::Node| {
                            if !state.enabled {
                                node.set_disabled();
                            }
                        })
                        .into_any_element()
                })
                .collect::<Vec<_>>();
            // 日付の無い週も高さをそろえるために置くが、セルの無い行は支援技術に公開しない
            let row = if week * 7 < offset + usize::from(days) {
                div()
                    .id(("week", week))
                    .role(accesskit::Role::Row)
                    .aria_row_index(week + 1)
            } else {
                div().id(("week", week))
            };
            rows.push(row.flex().children(cells).into_any_element());
        }
        let weeks = (offset + usize::from(days)).div_ceil(7);

        self.inner
            // 日付にフォーカスできない間だけ、退避先として格子にフォーカスを付ける。
            // 常に付けると曜日の見出しや空白を押したときに格子にフォーカスが移り、日付のキー操作が届かない
            .when(tab_day.is_none() || grid_focus.is_focused(window), |this| {
                this.track_focus(&grid_focus)
            })
            .role(accesskit::Role::Grid)
            .aria_row_count(weeks + 1)
            .aria_column_count(7)
            .on_key_down(on_key_down)
            .flex()
            .flex_col()
            .children(rows)
    }
}

impl Styled for Calendar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Calendar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for Calendar {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Calendar {}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::test_window::add_test_window;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("テストの日付が有効であること")
    }

    #[test]
    fn 日付の計算() {
        assert_eq!(Date::days_in_month(2024, 2), 29);
        assert_eq!(Date::days_in_month(2100, 2), 28);
        assert_eq!(Date::days_in_month(2000, 2), 29);
        assert_eq!(Date::days_in_month(2025, 4), 30);
        assert_eq!(Date::new(2025, 2, 29), None);
        assert_eq!(Date::new(2025, 13, 1), None);
        assert_eq!(date(1970, 1, 1).weekday(), 4);
        assert_eq!(date(2026, 11, 24).weekday(), 2);
        assert_eq!(date(1969, 12, 31).weekday(), 3);
        assert_eq!(date(2024, 2, 28).add_days(1), Some(date(2024, 2, 29)));
        assert_eq!(date(2024, 12, 31).add_days(1), Some(date(2025, 1, 1)));
        assert_eq!(date(2025, 3, 1).add_days(-1), Some(date(2025, 2, 28)));
        assert_eq!(date(1600, 3, 1).add_days(-1), Some(date(1600, 2, 29)));
        assert_eq!(date(2025, 1, 31).add_months(1), Some(date(2025, 2, 28)));
        assert_eq!(date(2025, 1, 15).add_months(-1), Some(date(2024, 12, 15)));
        assert_eq!(date(2024, 2, 29).add_months(12), Some(date(2025, 2, 28)));
        assert_eq!(Date::new(0, 1, 1), None);
        assert_eq!(Date::new(10_000, 1, 1), None);
        assert_eq!(Date::MAX.add_days(1), None);
        assert_eq!(Date::MIN.add_days(-1), None);
        assert_eq!(Date::MIN.add_days(1), Some(date(1, 1, 2)));
        assert_eq!(Date::MAX.add_months(1), None);
        assert_eq!(Date::MIN.add_months(i64::MIN), None);
        assert_eq!(Date::MAX.add_days(i64::MAX), None);
        assert_eq!(date(1, 1, 1).weekday(), 1);
    }

    struct CalendarView {
        year: i32,
        month: u8,
        selected: Option<Date>,
        month_change: bool,
        range: Option<RangeInclusive<Date>>,
        refuse_month: bool,
        parent_clicks: usize,
        other: Option<FocusHandle>,
        before: Option<FocusHandle>,
        after: FocusHandle,
        next_range: Option<RangeInclusive<Date>>,
        changes: Rc<RefCell<Vec<Date>>>,
        focused: Rc<Cell<Option<Date>>>,
    }

    impl Render for CalendarView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let calendar = Calendar::new("calendar", date(self.year, self.month, 1))
                .selected(self.selected)
                .today(Some(date(2026, 11, 24)))
                .enabled(|date| date.weekday() != 0)
                .on_change(cx.listener(|this, date: &Date, _, cx| {
                    this.changes.borrow_mut().push(*date);
                    this.selected = Some(*date);
                    cx.notify();
                }))
                .day({
                    let focused = self.focused.clone();
                    focused.set(None);
                    move |day, cell, _, _| {
                        if day.focused {
                            focused.set(Some(day.date));
                        }
                        cell.size(px(40.)).aria_label(format!("{}", day.date.day()))
                    }
                })
                .weekday(|_, cell, _, _| cell.size(px(40.)))
                .blank(|blank| blank.size(px(40.)))
                .aria_label("Calendar");
            let calendar = match self.range.clone() {
                Some(range) => calendar.range(range),
                None => calendar,
            };
            let calendar = if self.month_change {
                calendar.on_month_change(cx.listener(|this, date: &Date, window, cx| {
                    if this.refuse_month {
                        return;
                    }
                    if let Some(other) = &this.other {
                        window.focus(other, cx);
                    }
                    if let Some(range) = this.next_range.take() {
                        this.range = Some(range);
                    }
                    this.year = date.year();
                    this.month = date.month();
                    cx.notify();
                }))
            } else {
                calendar
            };
            div()
                .id("parent")
                .on_click(cx.listener(|this, _, _, _| this.parent_clicks += 1))
                .size_full()
                .when_some(self.before.clone(), |this, before| {
                    this.child(div().track_focus(&before).size(px(10.)))
                })
                .child(calendar)
                .when_some(self.other.clone(), |this, other| {
                    this.child(div().track_focus(&other).size(px(10.)))
                })
                .child(div().track_focus(&self.after).size(px(10.)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selected: Option<Date>,
        month_change: bool,
    ) -> (gpui::Entity<CalendarView>, &mut VisualTestContext) {
        let (view, cx) = add_test_window(cx, |_, cx| CalendarView {
            year: 2026,
            month: 11,
            selected,
            month_change,
            range: None,
            refuse_month: false,
            parent_clicks: 0,
            other: None,
            before: None,
            after: cx.focus_handle().tab_stop(true),
            next_range: None,
            changes: Rc::default(),
            focused: Rc::default(),
        });
        cx.run_until_parked();
        (view, cx)
    }

    /// フォーカスのある日付。最後の描画の時点の値
    fn focused(view: &gpui::Entity<CalendarView>, cx: &mut VisualTestContext) -> Option<Date> {
        cx.run_until_parked();
        view.read_with(cx, |view, _| view.focused.get())
    }

    fn changes(view: &gpui::Entity<CalendarView>, cx: &mut VisualTestContext) -> Vec<Date> {
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
    fn tab_では選ばれている日付に入り_矢印キーではフォーカスだけを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 10)));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 11)));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 18)));
        cx.simulate_keystrokes("left up");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 10)));
        // 2026 年 11 月 10 日は火曜日
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 8)));
        cx.simulate_keystrokes("end");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 14)));
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 選ばれている日付が無ければ今日の日付に入る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 10, 10)), true);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 24)));
    }

    #[gpui::test]
    fn 月の外へ移るときは表示する月を求める(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), true);
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 1)));
        cx.simulate_keystrokes("pageup");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 1)));
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some(date(2026, 10, 25)));
        cx.simulate_keystrokes("shift-pagedown");
        assert_eq!(focused(&view, cx), Some(date(2027, 10, 25)));
        cx.simulate_keystrokes("pagedown pagedown pagedown pagedown");
        assert_eq!(focused(&view, cx), Some(date(2028, 2, 25)));
    }

    #[gpui::test]
    fn 月の変更を受けなければ月の外へ移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), false);
        cx.simulate_keystrokes("tab right pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 30)));
    }

    #[gpui::test]
    fn 月の途中から始まる範囲では範囲の最初の日に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, true);
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 11, 20)..=date(2026, 11, 22));
            cx.notify();
        });
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 20)));
    }

    #[gpui::test]
    fn 月の変更が反映されなければフォーカスは元の日付に残る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), true);
        view.update(cx, |view, _| view.refuse_month = true);
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 30)));
        press(cx, "enter");
        assert_eq!(changes(&view, cx), [date(2026, 11, 30)]);
    }

    #[gpui::test]
    fn 範囲の外の選択日を飛ばして範囲の中の今日に入る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 11, 20)..=date(2026, 11, 30));
            cx.notify();
        });
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 24)));
    }

    #[gpui::test]
    fn 範囲の外の日付はクリックしてもフォーカスも選択もしない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, true);
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 11, 20)..=date(2026, 11, 30));
            cx.notify();
        });
        // 11 月 10 日は 3 行目の 3 列目
        cx.simulate_click(gpui::point(px(100.), px(100.)), Modifiers::none());
        assert_eq!(focused(&view, cx), None);
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 移る先が範囲から外れたら範囲の中の日付にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 24)), true);
        view.update(cx, |view, _| {
            view.next_range = Some(date(2026, 12, 1)..=date(2026, 12, 10))
        });
        cx.simulate_keystrokes("tab pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 1)));
        press(cx, "enter");
        assert_eq!(changes(&view, cx), [date(2026, 12, 1)]);
    }

    #[gpui::test]
    fn フォーカス中に範囲を狭めると範囲の中の日付に移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 11, 20)..=date(2026, 11, 30));
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 24)));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 25)));
    }

    #[gpui::test]
    fn フォーカス中に表示する月が変わると新しい月の候補に移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        view.update(cx, |view, cx| {
            view.year = 2027;
            view.month = 1;
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(date(2027, 1, 1)));
    }

    #[gpui::test]
    fn 見送った月の変更の保留を後の月の変更で使わない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), true);
        view.update(cx, |view, _| view.refuse_month = true);
        cx.simulate_keystrokes("tab right");
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.month = 12;
            view.selected = Some(date(2026, 12, 15));
            cx.notify();
        });
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 15)));
    }

    #[gpui::test]
    fn 月を続けて移るときは月末に寄せる前の日を基準にする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 1, 31)), true);
        view.update(cx, |view, cx| {
            view.month = 1;
            cx.notify();
        });
        cx.simulate_keystrokes("tab pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 2, 28)));
        cx.simulate_keystrokes("pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 3, 31)));
        cx.simulate_keystrokes("pageup pageup");
        assert_eq!(focused(&view, cx), Some(date(2026, 1, 31)));
        // 月か年の移動以外の操作をすると、そのときの日を基準にする
        cx.simulate_keystrokes("pagedown left pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 3, 27)));
    }

    #[gpui::test]
    fn フォーカス中に範囲から日付が無くなっても次の_tab_はカレンダーの後ろへ進む(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        // カレンダーの前に Tab で到達できる要素を置く
        view.update(cx, |view, cx| {
            view.before = Some(cx.focus_handle().tab_stop(true));
            cx.notify();
        });
        cx.simulate_keystrokes("tab tab");
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 12, 1)..=date(2026, 12, 31));
            cx.notify();
        });
        assert_eq!(focused(&view, cx), None);
        cx.simulate_keystrokes("tab");
        let after = view.read_with(cx, |view, _| view.after.clone());
        assert!(cx.update(|window, _| after.is_focused(window)));
    }

    #[gpui::test]
    fn 格子に退避した後に範囲を戻すと日付にフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 12, 1)..=date(2026, 12, 31));
            cx.notify();
        });
        assert_eq!(focused(&view, cx), None);
        view.update(cx, |view, cx| {
            view.range = None;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 10)));
    }

    #[gpui::test]
    fn 曜日の見出しを押しても日付のフォーカスを奪わない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        // 1 行目は曜日の見出し
        cx.simulate_click(gpui::point(px(20.), px(20.)), Modifiers::none());
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 11)));
    }

    #[gpui::test]
    fn 日付の外へ出て入り直したら月の移動の基準日を捨てる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 1, 31)), true);
        let other = view.update(cx, |view, cx| {
            view.month = 1;
            view.range = Some(date(2026, 1, 1)..=date(2026, 3, 28));
            let other = cx.focus_handle().tab_stop(true);
            view.after = other.clone();
            cx.notify();
            other
        });
        cx.simulate_keystrokes("tab pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 2, 28)));
        cx.simulate_keystrokes("tab");
        assert!(cx.update(|window, _| other.is_focused(window)));
        // 2 月 28 日 (2026 年 2 月 1 日は日曜日なので 5 行目の 7 列目) を押して入り直す
        cx.simulate_click(gpui::point(px(260.), px(180.)), Modifiers::none());
        assert_eq!(focused(&view, cx), Some(date(2026, 2, 28)));
        cx.simulate_keystrokes("pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 3, 28)));
    }

    #[gpui::test]
    fn 選べない日付の押下を親に伝えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, true);
        // 11 月 8 日は日曜日で選べない。3 行目の 1 列目
        cx.simulate_click(gpui::point(px(20.), px(100.)), Modifiers::none());
        assert_eq!(view.read_with(cx, |view, _| view.parent_clicks), 0);
        assert_eq!(focused(&view, cx), Some(date(2026, 11, 8)));
        cx.simulate_click(gpui::point(px(60.), px(100.)), Modifiers::none());
        assert_eq!(changes(&view, cx), [date(2026, 11, 9)]);
    }

    #[gpui::test]
    fn 範囲の中に日付が無い月では_tab_で到達できない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 12, 1)..=date(2026, 12, 31));
            cx.notify();
        });
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 月の変更の間に呼び出し側が移したフォーカスを戻さない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), true);
        let other = view.update(cx, |view, cx| {
            let other = cx.focus_handle();
            view.other = Some(other.clone());
            other
        });
        cx.simulate_keystrokes("tab right");
        cx.run_until_parked();
        assert_eq!(focused(&view, cx), None);
        assert!(cx.update(|window, _| other.is_focused(window)));
    }

    #[gpui::test]
    fn 範囲の外へは移らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), true);
        view.update(cx, |view, cx| {
            view.range = Some(date(2026, 1, 1)..=date(2026, 12, 31));
            cx.notify();
        });
        cx.simulate_keystrokes("tab pagedown");
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 30)));
        cx.simulate_keystrokes("down shift-pageup");
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 30)));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(date(2026, 12, 31)));
    }

    #[gpui::test]
    fn enter_とクリックで選べる日付だけを選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), true);
        cx.simulate_keystrokes("tab");
        press(cx, "enter");
        // 11 月 8 日は日曜日で選べない
        cx.simulate_keystrokes("home");
        press(cx, "enter");
        cx.simulate_keystrokes("right");
        press(cx, "space");
        // 1 行目は曜日で、1 日は日曜日なので 2 行目の 1 列目。10 日は 3 行目の 3 列目
        cx.simulate_click(gpui::point(px(100.), px(100.)), Modifiers::none());
        assert_eq!(
            changes(&view, cx),
            [date(2026, 11, 10), date(2026, 11, 9), date(2026, 11, 10)]
        );
    }
}
