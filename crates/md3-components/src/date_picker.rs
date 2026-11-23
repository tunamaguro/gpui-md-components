// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt
//         (`DatePicker`、`DateEntryContainer`、`DatePickerHeader`、`DatePickerDefaults.DatePickerTitle`、
//         `DatePickerDefaults.DatePickerHeadline`、`DatePickerContent`、`MonthsNavigation`、`WeekDays`、`Month`、
//         `Day`、`dayContentDescription`、`DatePickerColors`、`defaultDatePickerColors`、`YearRange`、
//         `RecommendedSizeForAccessibility`、`MonthYearHeight`、`DatePickerHorizontalPadding`、
//         `DatePickerTitlePadding`、`DatePickerHeadlinePadding`、`MaxCalendarRows`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_date_picker_*`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のデイトピッカー

use std::ops::RangeInclusive;
use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_headless::calendar::{Calendar, Date};
use gpui_headless::text::StaticText;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::date_picker_modal as tokens;
use md3_theme::md3_tokens::component::divider;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::{IconButton, IconButtonVariant};

/// 日付と曜日のセルの大きさ (Compose の `RecommendedSizeForAccessibility`)
const CELL_SIZE: Dp = Dp(48.0);
/// 月を移るボタンの行の高さ (Compose の `MonthYearHeight`)
const MONTH_YEAR_HEIGHT: Dp = Dp(56.0);
/// 月の表示の左右の余白 (Compose の `DatePickerHorizontalPadding`)
const HORIZONTAL_PADDING: Dp = Dp(12.0);
/// タイトルと見出しの左右の余白 (Compose の `DatePickerTitlePadding`、`DatePickerHeadlinePadding`)
const HEADER_START_PADDING: Dp = Dp(24.0);
const HEADER_END_PADDING: Dp = Dp(12.0);
/// タイトルの上と見出しの下の余白
const TITLE_TOP_PADDING: Dp = Dp(16.0);
const HEADLINE_BOTTOM_PADDING: Dp = Dp(12.0);
/// 既定の年の範囲 (Compose の `DatePickerDefaults.YearRange`)
const DEFAULT_YEAR_RANGE: RangeInclusive<i32> = 1900..=2100;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

type ChangeHandler = Rc<dyn Fn(&Date, &mut Window, &mut App)>;
type EnabledFn = Rc<dyn Fn(&Date) -> bool>;

/// 曜日・月・日・年をそろえた長い表記。Tuesday, November 24, 2026 のように書く
fn long_date(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        WEEKDAY_NAMES[usize::from(date.weekday())],
        MONTH_NAMES[usize::from(date.month()) - 1],
        date.day(),
        date.year()
    )
}

/// MD3 のデイトピッカー (Compose の `DatePicker` のカレンダーの表示)
///
/// 上にタイトル Select date と、選ばれている日付の見出しを置き、区切り線の下に表示している月と前後の月へ移るボタン、
/// 曜日の見出しと 1 か月の日付を、日曜日から始まる 6 週の格子に並べる。
/// 選ばれている日付は呼び出し側が持ち、[`DatePicker::new`] に毎回渡す。ピッカー自身は日付を変えず、
/// 選ぶ操作をされた日付を [`DatePicker::on_change`] で求める。
/// 表示している月はピッカーが持つ。最初は選ばれている日付の月、無ければ [`DatePicker::today`] の月、
/// どちらも無ければ [`DatePicker::year_range`] の最初の年の 1 月にする
///
/// - 前後の月へ移るボタンで、表示する月を変える。[`DatePicker::year_range`] の端の月ではボタンを無効にする
/// - 日付の操作は [`Calendar`] と同じ。キー操作で月の外の日付へ移ると、表示する月も変える。
///   [`DatePicker::year_range`] の外の日付へは移らない
///
/// 支援技術には、日付の格子が表示している月 (November 2026 のような名前) の role `grid` として公開される。
/// 曜日の見出しは role `columnheader`、日付は role `gridcell` で、選ばれているかどうかと選べるかどうかを持ち、
/// 名前は Today, Tuesday, November 24, 2026 のように、今日であることと曜日・月・日・年を書く。
/// 選ばれている日付の見出し (Current selection: に続けて日付、選ばれていなければ None) と表示している月は
/// role `label` で、変わったときに読み上げ中の内容を遮らずに知らせる。
/// 前後の月へ移るボタンの名前は Change to previous month と Change to next month。
/// 文言は英語に固定している
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    selected: Option<Date>,
    today: Option<Date>,
    year_range: RangeInclusive<i32>,
    enabled: Option<EnabledFn>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl DatePicker {
    /// `selected` を選ばれた日付として表示するデイトピッカーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、表示している月とフォーカスが失われる
    pub fn new(id: impl Into<ElementId>, selected: Option<Date>) -> Self {
        Self {
            id: id.into(),
            selected,
            today: None,
            year_range: DEFAULT_YEAR_RANGE,
            enabled: None,
            on_change: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 今日の日付を `date` にする。今日の日付は縁取って示す。既定では示さない
    pub fn today(mut self, date: Date) -> Self {
        self.today = Some(date);
        self
    }

    /// 表示できる年を `range` に限る。既定では 1900 年から 2100 年
    ///
    /// 範囲の外の日付は選べず、[`DatePicker::new`] と [`DatePicker::today`] に渡しても無いものとして扱う。
    /// 1 年から 9999 年の外は除く。`range` が空の場合は、最初の年だけを表示する
    pub fn year_range(mut self, range: RangeInclusive<i32>) -> Self {
        self.year_range = range;
        self
    }

    /// `enabled` が `false` を返す日付を選べなくする。既定では [`DatePicker::year_range`] の中のすべての日付を選べる
    pub fn enabled(mut self, enabled: impl Fn(&Date) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    /// 日付を選ぶ操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶ操作をされた日付を渡す。選ばれている日付でも呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`DatePicker::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for DatePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let first_year = (*self.year_range.start()).clamp(Date::MIN.year(), Date::MAX.year());
        let last_year = (*self.year_range.end()).clamp(first_year, Date::MAX.year());
        let (Some(first), Some(last)) = (Date::new(first_year, 1, 1), Date::new(last_year, 12, 31))
        else {
            unreachable!("years are clamped to the range of Date");
        };
        let in_range = |date: &Date| (first..=last).contains(date);
        // Compose と同じく、範囲の外の日付は選ばれていないものとして扱う
        let selected = self.selected.filter(in_range);
        let today = self.today.filter(in_range);
        let initial = selected.or(today).unwrap_or(first);
        let displayed = window.use_keyed_state((self.id.clone(), "month"), cx, |_, _| initial);
        // year_range が変わって表示している月が範囲の外になった場合は、範囲の端の月を表示する
        let displayed_month = (*displayed.read(cx)).clamp(first, last);
        let (year, month) = (displayed_month.year(), displayed_month.month());
        let set_month = {
            let displayed = displayed.clone();
            move |date: Date, cx: &mut App| {
                displayed.update(cx, |displayed, cx| {
                    *displayed = date;
                    cx.notify();
                });
            }
        };

        // タイトルと見出し
        let mut title_font = theme.text_style(tokens::HEADER_SUPPORTING_TEXT_FONT);
        title_font.color = Some(theme.color(tokens::HEADER_SUPPORTING_TEXT_COLOR));
        let mut title = div()
            .pt(dp(TITLE_TOP_PADDING))
            .pl(dp(HEADER_START_PADDING))
            .pr(dp(HEADER_END_PADDING))
            .child(gpui::text!(id = (self.id.clone(), "title"), "Select date"));
        title.text_style().refine(&title_font);
        let mut headline_font = theme.text_style(tokens::HEADER_HEADLINE_FONT);
        headline_font.color = Some(theme.color(tokens::HEADER_HEADLINE_COLOR));
        let (headline_text, headline_label) = match selected {
            Some(date) => (
                format!(
                    "{} {}, {}",
                    &MONTH_NAMES[usize::from(date.month()) - 1][..3],
                    date.day(),
                    date.year()
                ),
                format!("Current selection: {}", long_date(date)),
            ),
            None => (
                "Selected date".to_owned(),
                "Current selection: None".to_owned(),
            ),
        };
        let mut headline = StaticText::new((self.id.clone(), "headline"), headline_label)
            .live(true)
            .pb(dp(HEADLINE_BOTTOM_PADDING))
            .pl(dp(HEADER_START_PADDING))
            .pr(dp(HEADER_END_PADDING))
            .overflow_hidden()
            .whitespace_nowrap()
            .child(headline_text);
        headline.text_style().refine(&headline_font);
        let header = div()
            .flex()
            .flex_col()
            .justify_between()
            .min_h(dp(tokens::HEADER_CONTAINER_HEIGHT))
            .child(title)
            .child(headline);
        let divider = div()
            .flex_none()
            .h(dp(divider::THICKNESS))
            .bg(theme.color(divider::COLOR));

        // 表示している月と、前後の月へ移るボタン
        let mut month_font = theme.text_style(TypeScale::LabelLarge);
        month_font.color = Some(theme.color(ColorRole::OnSurfaceVariant));
        let month_text = format!("{} {}", MONTH_NAMES[usize::from(month) - 1], year);
        let mut month_label = StaticText::new((self.id.clone(), "month-label"), month_text.clone())
            .live(true)
            .px(dp(Dp(12.0)))
            .child(month_text.clone());
        month_label.text_style().refine(&month_font);
        let neighbor = |delta: i64| {
            displayed_month
                .add_months(delta)
                .filter(|date| (first..=last).contains(date))
        };
        let move_month = |target: Date| {
            let set_month = set_month.clone();
            move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| set_month(target, cx)
        };
        let previous = neighbor(-1);
        let next = neighbor(1);
        let navigation = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .h(dp(MONTH_YEAR_HEIGHT))
            .px(dp(HORIZONTAL_PADDING))
            .child(month_label)
            .child(
                div()
                    .flex()
                    .child(
                        IconButton::new(
                            (self.id.clone(), "previous"),
                            md3_icons::KEYBOARD_ARROW_LEFT,
                            "Change to previous month",
                        )
                        .variant(IconButtonVariant::Standard)
                        .disabled(previous.is_none())
                        .when_some(previous, |this, target| this.on_click(move_month(target)))
                        .theme(theme.clone()),
                    )
                    .child(
                        IconButton::new(
                            (self.id.clone(), "next"),
                            md3_icons::KEYBOARD_ARROW_RIGHT,
                            "Change to next month",
                        )
                        .variant(IconButtonVariant::Standard)
                        .disabled(next.is_none())
                        .when_some(next, |this, target| this.on_click(move_month(target)))
                        .theme(theme.clone()),
                    ),
            );

        // 曜日と日付
        let mut weekday_font = theme.text_style(tokens::WEEKDAYS_LABEL_TEXT_FONT);
        weekday_font.color = Some(theme.color(tokens::WEEKDAYS_LABEL_TEXT_COLOR));
        let mut day_font = theme.text_style(tokens::DATE_LABEL_TEXT_FONT);
        day_font.color = None;
        let enabled = self.enabled.clone();
        let mut calendar = Calendar::new((self.id.clone(), "calendar"), displayed_month)
            .selected(selected)
            .today(today)
            .range(first..=last)
            .enabled(move |date| enabled.as_ref().is_none_or(|enabled| enabled(date)))
            .on_month_change(move |date: &Date, _, cx| set_month(*date, cx))
            .weekday(move |weekday, cell, _, _| {
                let mut cell = cell
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(dp(CELL_SIZE))
                    .aria_label(WEEKDAY_NAMES[usize::from(weekday)])
                    .child(&WEEKDAY_NAMES[usize::from(weekday)][..1]);
                cell.text_style().refine(&weekday_font);
                cell
            })
            .blank(|blank| blank.flex_none().size(dp(CELL_SIZE)))
            .day({
                let theme = theme.clone();
                move |day, cell, _, _| {
                    let disabled_alpha = supplement::color_scheme::DISABLED_ALPHA;
                    let (container, content) = match (day.selected, day.enabled, day.today) {
                        (true, true, _) => (
                            Some(theme.color(tokens::DATE_SELECTED_CONTAINER_COLOR)),
                            theme.color(tokens::DATE_SELECTED_LABEL_TEXT_COLOR),
                        ),
                        (true, false, _) => (
                            Some(
                                theme
                                    .color(tokens::DATE_SELECTED_CONTAINER_COLOR)
                                    .opacity(disabled_alpha),
                            ),
                            theme
                                .color(tokens::DATE_SELECTED_LABEL_TEXT_COLOR)
                                .opacity(disabled_alpha),
                        ),
                        (false, true, true) => {
                            (None, theme.color(tokens::DATE_TODAY_LABEL_TEXT_COLOR))
                        }
                        (false, true, false) => {
                            (None, theme.color(tokens::DATE_UNSELECTED_LABEL_TEXT_COLOR))
                        }
                        (false, false, _) => (
                            None,
                            theme
                                .color(tokens::DATE_UNSELECTED_LABEL_TEXT_COLOR)
                                .opacity(disabled_alpha),
                        ),
                    };
                    // 状態レイヤーは文字の色を重ねる (Compose の Surface の ripple は内容の色を使う)
                    let state_layer = |opacity: f32| {
                        let layer = content.opacity(opacity);
                        match container {
                            Some(container) => container.blend(layer),
                            None => layer,
                        }
                    };
                    let group = format!("date-picker-day-{}", day.date.day());
                    let circle = div()
                        .absolute()
                        .inset_0()
                        .when_some(container, |this, container| this.bg(container))
                        .when(day.today && !day.selected, |this| {
                            this.border(dp(tokens::DATE_TODAY_CONTAINER_OUTLINE_WIDTH))
                                .border_color(
                                    theme.color(tokens::DATE_TODAY_CONTAINER_OUTLINE_COLOR),
                                )
                        })
                        .when(day.enabled, |this| {
                            this.when(day.focus_visible, |this| {
                                this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                            })
                            .group_hover(group.clone(), |style| {
                                style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                            })
                        });
                    let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                    let label = if day.today {
                        format!("Today, {}", long_date(day.date))
                    } else {
                        long_date(day.date)
                    };
                    let mut cell = cell
                        .group(group.clone())
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(dp(CELL_SIZE))
                        .aria_label(label)
                        .child(
                            div()
                                .relative()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(dp(tokens::DATE_CONTAINER_WIDTH))
                                .h(dp(tokens::DATE_CONTAINER_HEIGHT))
                                .text_color(content)
                                .child(Shaped::new(
                                    circle,
                                    theme.clone(),
                                    tokens::DATE_CONTAINER_SHAPE,
                                ))
                                // group_active は id を持つ要素にしか付けられないため、押下の色は別の層に描く。
                                // 押している間は hover の層も残るので、重ねて押下の不透明度になる分だけを足す。
                                // 日付の容器の形 (CornerFull) は円なので rounded_full で同じ形になる
                                .when(day.enabled, |this| {
                                    this.child(
                                        div()
                                            .id("pressed")
                                            .absolute()
                                            .inset_0()
                                            .rounded_full()
                                            .group_active(group.clone(), |style| {
                                                style.bg(content.opacity(
                                                    state::PRESSED_STATE_LAYER_OPACITY
                                                        - state::HOVER_STATE_LAYER_OPACITY,
                                                ))
                                            }),
                                    )
                                })
                                .child(div().relative().child(day.date.day().to_string()))
                                .when(day.focus_visible, |this| {
                                    this.child(
                                        Shaped::new(
                                            div()
                                                .absolute()
                                                .top(-outset)
                                                .left(-outset)
                                                .right(-outset)
                                                .bottom(-outset)
                                                .border(dp(focus_ring::WIDTH))
                                                .border_color(theme.color(focus_ring::COLOR)),
                                            theme.clone(),
                                            ShapeKey::CornerFull,
                                        )
                                        .outset(outset),
                                    )
                                }),
                        );
                    cell.text_style().refine(&day_font);
                    cell
                }
            })
            .aria_label(month_text);
        if let Some(on_change) = self.on_change {
            calendar = calendar.on_change(move |date, window, cx| on_change(date, window, cx));
        }

        let mut root = div()
            .id(self.id)
            .flex()
            .flex_col()
            .w(dp(tokens::CONTAINER_WIDTH))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .child(header)
            .child(divider)
            .child(navigation)
            .child(div().px(dp(HORIZONTAL_PADDING)).child(calendar));
        root.style().refine(&self.style);
        root
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("テストの日付が有効であること")
    }

    /// (0, 0) にデイトピッカーを置き、選ばれている日付と、求められた日付を持つビュー
    struct PickerView {
        selected: Option<Date>,
        year_range: RangeInclusive<i32>,
        changes: Rc<RefCell<Vec<Date>>>,
    }

    impl Render for PickerView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().size_full().child(
                DatePicker::new("picker", self.selected)
                    .today(date(2026, 11, 24))
                    .year_range(self.year_range.clone())
                    // 日曜日を選べなくする
                    .enabled(|date| date.weekday() != 0)
                    .on_change(cx.listener(move |this, date: &Date, _, cx| {
                        changes.borrow_mut().push(*date);
                        this.selected = Some(*date);
                        cx.notify();
                    }))
                    .absolute()
                    .left(px(0.))
                    .top(px(0.)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selected: Option<Date>,
        year_range: RangeInclusive<i32>,
    ) -> (gpui::Entity<PickerView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            PickerView {
                selected,
                year_range,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn changes(view: &gpui::Entity<PickerView>, cx: &mut VisualTestContext) -> Vec<Date> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    /// 色 `role` で塗った矩形 (x, y, 幅, 高さ)
    fn quads(cx: &mut VisualTestContext, role: ColorRole) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background = cx.md3_theme().color(role).into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == color)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    /// 日付の行の上端。見出し 120dp、区切り線 1dp、月の行 56dp、曜日の行 48dp の下
    const FIRST_WEEK_TOP: f32 = 120. + 1. + 56. + 48.;

    /// 表示している月の `week` 行目 `column` 列目 (どちらも 0 から) のセルの中心
    fn cell_center(week: usize, column: usize) -> gpui::Point<gpui::Pixels> {
        point(
            px(12. + 48. * column as f32 + 24.),
            px(FIRST_WEEK_TOP + 48. * week as f32 + 24.),
        )
    }

    #[gpui::test]
    fn 選ばれている日付を_40dp_の円で示す(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        let containers = quads(cx, ColorRole::SurfaceContainerHigh);
        assert!(
            containers.iter().any(|quad| quad.2 == 360.),
            "幅 360dp の容器を描くこと: {containers:?}"
        );
        // 2026 年 11 月 1 日は日曜日なので、10 日は 2 行目の 3 列目
        assert_eq!(
            quads(cx, tokens::DATE_SELECTED_CONTAINER_COLOR),
            [(12. + 96. + 4., FIRST_WEEK_TOP + 48. + 4., 40., 40.)]
        );
    }

    #[gpui::test]
    fn クリックで選べる日付だけを選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        cx.simulate_click(cell_center(1, 3), Modifiers::none());
        // 11 月 15 日は日曜日で選べない
        cx.simulate_click(cell_center(2, 0), Modifiers::none());
        assert_eq!(changes(&view, cx), [date(2026, 11, 11)]);
    }

    #[gpui::test]
    fn 次の月のボタンで表示する月を変える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        // 次の月のボタンは右端の 48dp
        cx.simulate_click(
            point(px(360. - 12. - 24.), px(121. + 28.)),
            Modifiers::none(),
        );
        cx.run_until_parked();
        // 2026 年 12 月 1 日は火曜日
        cx.simulate_click(cell_center(0, 2), Modifiers::none());
        assert_eq!(changes(&view, cx), [date(2026, 12, 1)]);
    }

    #[gpui::test]
    fn キー操作で月の外へ移ると表示する月を変える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), DEFAULT_YEAR_RANGE);
        // 前後の月のボタンの次が日付
        cx.simulate_keystrokes("tab tab tab right");
        let keystroke = gpui::Keystroke::parse("enter").expect("テストのキー表記が有効であること");
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke });
        cx.run_until_parked();
        assert_eq!(changes(&view, cx), [date(2026, 12, 1)]);
    }

    #[gpui::test]
    fn 範囲の外の選択は無いものとして今日の月を表示する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2025, 12, 15)), 2026..=2026);
        assert!(
            quads(cx, tokens::DATE_SELECTED_CONTAINER_COLOR).is_empty(),
            "範囲の外の日付は選ばれた日付として示さない"
        );
        // 今日の 2026 年 11 月を表示する。11 月 24 日は 4 行目の 3 列目
        cx.simulate_click(cell_center(3, 2), Modifiers::none());
        assert_eq!(changes(&view, cx), [date(2026, 11, 24)]);
    }

    #[gpui::test]
    fn 年の範囲の外の月は表示しない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 1, 6)), 2026..=2026);
        // 前の月のボタンは無効
        cx.simulate_click(
            point(px(360. - 12. - 72.), px(121. + 28.)),
            Modifiers::none(),
        );
        cx.run_until_parked();
        // 2026 年 1 月 1 日は木曜日なので、6 日は 2 行目の 3 列目
        cx.simulate_click(cell_center(1, 2), Modifiers::none());
        assert_eq!(changes(&view, cx), [date(2026, 1, 6)]);
    }
}
