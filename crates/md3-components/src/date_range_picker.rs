// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateRangePicker.kt
//         (`DateRangePicker`、`DateRangePickerDefaults.DateRangePickerTitle`、
//         `DateRangePickerDefaults.DateRangePickerHeadline`、`DateRangePickerContent`、`VerticalMonthsList`、
//         `drawRangeBackground`、`CalendarMonthSubheadPadding`、`DateRangePickerTitlePadding`、
//         `DateRangePickerHeadlinePadding`、`HeaderHeightOffset`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt
//         (`WeekDays`、`Day`、`dayContentDescription`、`DatePickerColors`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_date_range_picker_*`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の日付の範囲を選ぶデイトピッカー

use std::ops::RangeInclusive;
use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};
use gpui_headless::calendar::Date;
use gpui_headless::range_calendar::{DateRange, RangeCalendar};
use gpui_headless::text::StaticText;
use md3_theme::md3_tokens::component::date_picker_modal as tokens;
use md3_theme::md3_tokens::component::divider;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::date_picker::{
    CELL_SIZE, DEFAULT_YEAR_RANGE, HEADER_END_PADDING, HORIZONTAL_PADDING, MONTH_NAMES,
    WEEKDAY_NAMES, long_date, short_date,
};
use crate::shape::Shaped;

/// タイトルと見出しの左の余白 (Compose の `DateRangePickerTitlePadding`、`DateRangePickerHeadlinePadding`)
const HEADER_START_PADDING: Dp = Dp(64.0);
/// 見出しの下の余白
const HEADLINE_BOTTOM_PADDING: Dp = Dp(12.0);
/// 見出しの日付と区切りの間隔
const HEADLINE_SPACING: Dp = Dp(4.0);
/// 見出しの領域の最小の高さ。トークンの値から、置かない保存ボタンと閉じるボタンの分を引く
/// (Compose の `RangeSelectionHeaderContainerHeight - HeaderHeightOffset`)
const HEADER_MIN_HEIGHT: Dp = Dp(tokens::RANGE_SELECTION_HEADER_CONTAINER_HEIGHT.0 - 60.0);
/// 月の見出しの余白 (Compose の `CalendarMonthSubheadPadding`)
const MONTH_SUBHEAD_START_PADDING: Dp = Dp(24.0);
const MONTH_SUBHEAD_TOP_PADDING: Dp = Dp(20.0);
const MONTH_SUBHEAD_BOTTOM_PADDING: Dp = Dp(8.0);

type ChangeHandler = Rc<dyn Fn(&DateRange, &mut Window, &mut App)>;
type EnabledFn = Rc<dyn Fn(&Date) -> bool>;

/// MD3 の日付の範囲を選ぶデイトピッカー (Compose の `DateRangePicker` のカレンダーの表示)
///
/// 上にタイトル Select dates と、選ばれている範囲の見出し (Start date - End date) を置き、区切り線の下に曜日の見出しと、
/// [`DateRangePicker::year_range`] のすべての月を縦に並べてスクロールする一覧を置く。
/// 月ごとに November 2026 のような見出しと、日曜日から始まる 6 週の日付を並べる。
/// 範囲の始まりと終わりの日付を円で示し、始まりと終わりがそろうと、その間を帯で塗る。
/// 一覧は残りの高さに広がるため、[`Styled`] で高さを指定すること。幅は 360dp
///
/// 選ばれている範囲は呼び出し側が持ち、[`DateRangePicker::new`] に毎回渡す。ピッカー自身は範囲を変えず、
/// 選ぶ操作をされた後の範囲を [`DateRangePicker::on_change`] で求める。
/// 始まりだけが選ばれていてそれ以降の日付を選ぶと終わりにし、それ以外は選んだ日付だけを始まりにする。
/// 最初は範囲の始まりの月、無ければ [`DatePicker::today`](crate::DatePicker::today) と同じく今日の月を一覧の一番上に表示する
///
/// キー操作は [`RangeCalendar`] と同じ。一覧は Tab で 1 回だけ到達し、矢印キーなどで操作する日付を移し、
/// Enter・Space で選ぶ。選べない日付 ([`DateRangePicker::enabled`]) は選べない
///
/// 支援技術には、月ごとに月の名前 (November 2026 のような) の role `grid` として公開される。
/// 日付は role `gridcell` で、範囲の始まりと終わりは選ばれていることを持ち、名前は
/// Start date, Today, Tuesday, November 24, 2026 のように、始まり・終わり・範囲の中 (In range) であることと、
/// 今日であることと、曜日・月・日・年を書く。
/// 見出しは Start date: に続けて日付 (無ければ None)、End date: に続けて日付を、変わったときに読み上げ中の内容を遮らずに知らせる。
/// 一覧の名前は Select dates。文言は英語に固定している
#[derive(IntoElement)]
pub struct DateRangePicker {
    id: ElementId,
    selection: DateRange,
    today: Option<Date>,
    year_range: RangeInclusive<i32>,
    enabled: Option<EnabledFn>,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl DateRangePicker {
    /// `selection` を選ばれた範囲として表示するピッカーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、操作する日付とフォーカスとスクロールの位置が失われる
    pub fn new(id: impl Into<ElementId>, selection: DateRange) -> Self {
        Self {
            id: id.into(),
            selection,
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
    /// 範囲の外の日付は選べず、[`DateRangePicker::new`] と [`DateRangePicker::today`] に渡しても無いものとして扱う。
    /// 1 年から 9999 年の外は除く。`range` が空の場合は、最初の年だけを表示する
    pub fn year_range(mut self, range: RangeInclusive<i32>) -> Self {
        self.year_range = range;
        self
    }

    /// `enabled` が `false` を返す日付を選べなくする。既定では [`DateRangePicker::year_range`] の中のすべての日付を選べる
    pub fn enabled(mut self, enabled: impl Fn(&Date) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    /// 日付を選ぶ操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選んだ後の範囲を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`DateRangePicker::new`] に渡すこと
    pub fn on_change(
        mut self,
        handler: impl Fn(&DateRange, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for DateRangePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for DateRangePicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let first_year = (*self.year_range.start()).clamp(Date::MIN.year(), Date::MAX.year());
        let last_year = (*self.year_range.end()).clamp(first_year, Date::MAX.year());
        let (Some(first), Some(last)) = (Date::new(first_year, 1, 1), Date::new(last_year, 12, 31))
        else {
            unreachable!("years are clamped to the range of Date");
        };
        let in_range = |date: &Date| (first..=last).contains(date);
        // Compose と同じく、範囲の外の日付は選ばれていないものとして扱う
        let start = self.selection.start.filter(in_range);
        let selection = DateRange {
            start,
            end: self
                .selection
                .end
                .filter(in_range)
                .filter(|_| start.is_some()),
        };
        let today = self.today.filter(in_range);

        // タイトルと見出し
        let mut title_font = theme.text_style(tokens::HEADER_SUPPORTING_TEXT_FONT);
        title_font.color = Some(theme.color(tokens::HEADER_SUPPORTING_TEXT_COLOR));
        let mut title = div()
            .pl(dp(HEADER_START_PADDING))
            .pr(dp(HEADER_END_PADDING))
            .child(gpui::text!(id = (self.id.clone(), "title"), "Select dates"));
        title.text_style().refine(&title_font);
        let mut headline_font = theme.text_style(tokens::RANGE_SELECTION_HEADER_HEADLINE_FONT);
        headline_font.color = Some(theme.color(tokens::HEADER_HEADLINE_COLOR));
        let describe = |date: Option<Date>| date.map_or("None".to_owned(), long_date);
        let mut headline = StaticText::new(
            (self.id.clone(), "headline"),
            format!(
                "Start date: {}, End date: {}",
                describe(selection.start),
                describe(selection.end)
            ),
        )
        .live(true)
        .flex()
        .items_center()
        .min_w_0()
        .gap(dp(HEADLINE_SPACING))
        .pb(dp(HEADLINE_BOTTOM_PADDING))
        .pl(dp(HEADER_START_PADDING))
        .pr(dp(HEADER_END_PADDING))
        .overflow_hidden()
        .whitespace_nowrap()
        .child(selection.start.map_or("Start date".to_owned(), short_date))
        .child("-")
        .child(selection.end.map_or("End date".to_owned(), short_date));
        headline.text_style().refine(&headline_font);
        let header = div()
            .flex()
            .flex_col()
            .flex_none()
            .justify_between()
            .min_h(dp(HEADER_MIN_HEIGHT))
            .child(title)
            .child(headline);

        // 曜日の見出し
        let mut weekday_font = theme.text_style(tokens::WEEKDAYS_LABEL_TEXT_FONT);
        weekday_font.color = Some(theme.color(tokens::WEEKDAYS_LABEL_TEXT_COLOR));
        let mut weekdays = div()
            .flex()
            .flex_none()
            .px(dp(HORIZONTAL_PADDING))
            .children(WEEKDAY_NAMES.iter().enumerate().map(|(index, name)| {
                div()
                    .id((self.id.clone(), format!("weekday-{index}")))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(dp(CELL_SIZE))
                    .role(accesskit::Role::Label)
                    .aria_label(*name)
                    .child(&name[..1])
            }));
        weekdays.text_style().refine(&weekday_font);

        // 月の一覧
        let mut subhead_font = theme.text_style(tokens::RANGE_SELECTION_MONTH_SUBHEAD_FONT);
        subhead_font.color = Some(theme.color(tokens::RANGE_SELECTION_MONTH_SUBHEAD_COLOR));
        let mut day_font = theme.text_style(tokens::DATE_LABEL_TEXT_FONT);
        day_font.color = None;
        let month_name = |date: &Date| {
            format!(
                "{} {}",
                MONTH_NAMES[usize::from(date.month()) - 1],
                date.year()
            )
        };
        let band_color = theme.color(tokens::RANGE_SELECTION_ACTIVE_INDICATOR_CONTAINER_COLOR);
        let mut calendar = RangeCalendar::new(
            (self.id.clone(), "calendar"),
            first_year..=last_year,
            selection,
        )
        .today(today)
        .header({
            let id = self.id.clone();
            move |date, _, _| {
                let mut subhead = div()
                    .pl(dp(MONTH_SUBHEAD_START_PADDING))
                    .pt(dp(MONTH_SUBHEAD_TOP_PADDING))
                    .pb(dp(MONTH_SUBHEAD_BOTTOM_PADDING))
                    .child(gpui::text!(
                        id = (
                            id.clone(),
                            format!("subhead-{}-{}", date.year(), date.month())
                        ),
                        month_name(date)
                    ));
                subhead.text_style().refine(&subhead_font);
                subhead.into_any_element()
            }
        })
        .grid(move |date, grid| grid.aria_label(month_name(date)))
        .week(move |week, row| {
            // 範囲の帯は、日付の容器と同じ高さで、範囲の両端では日付の中央まで塗る (Compose の `drawRangeBackground`)
            let band = week.band.map(|band| {
                let cell = dp(CELL_SIZE);
                let left = cell * band.first as f32
                    + if band.from_start {
                        cell / 2.
                    } else {
                        gpui::px(0.)
                    };
                let right = cell * band.last as f32 + if band.to_end { cell / 2. } else { cell };
                div()
                    .absolute()
                    .top((cell - dp(tokens::DATE_STATE_LAYER_HEIGHT)) / 2.)
                    .left(left)
                    .w(right - left)
                    .h(dp(tokens::DATE_STATE_LAYER_HEIGHT))
                    .bg(band_color)
            });
            row.relative().children(band)
        })
        .blank(|blank| blank.flex_none().size(dp(CELL_SIZE)))
        .day({
            let theme = theme.clone();
            move |day, cell, _, _| {
                let disabled_alpha = supplement::color_scheme::DISABLED_ALPHA;
                let selected = day.start || day.end;
                let (container, content) = match (selected, day.enabled, day.in_range, day.today) {
                    (true, true, _, _) => (
                        Some(theme.color(tokens::DATE_SELECTED_CONTAINER_COLOR)),
                        theme.color(tokens::DATE_SELECTED_LABEL_TEXT_COLOR),
                    ),
                    (true, false, _, _) => (
                        Some(
                            theme
                                .color(tokens::DATE_SELECTED_CONTAINER_COLOR)
                                .opacity(disabled_alpha),
                        ),
                        theme
                            .color(tokens::DATE_SELECTED_LABEL_TEXT_COLOR)
                            .opacity(disabled_alpha),
                    ),
                    (false, true, true, _) => (
                        None,
                        theme.color(tokens::SELECTION_DATE_IN_RANGE_LABEL_TEXT_COLOR),
                    ),
                    (false, true, false, true) => {
                        (None, theme.color(tokens::DATE_TODAY_LABEL_TEXT_COLOR))
                    }
                    (false, true, false, false) => {
                        (None, theme.color(tokens::DATE_UNSELECTED_LABEL_TEXT_COLOR))
                    }
                    (false, false, _, _) => (
                        None,
                        theme
                            .color(tokens::DATE_UNSELECTED_LABEL_TEXT_COLOR)
                            .opacity(disabled_alpha),
                    ),
                };
                let circle = div()
                    .absolute()
                    .inset_0()
                    .when_some(container, |this, container| this.bg(container))
                    .when(day.today && !selected, |this| {
                        this.border(dp(tokens::DATE_TODAY_CONTAINER_OUTLINE_WIDTH))
                            .border_color(theme.color(tokens::DATE_TODAY_CONTAINER_OUTLINE_COLOR))
                    });
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                let mut label = Vec::new();
                if day.start {
                    label.push("Start date".to_owned());
                } else if day.end {
                    label.push("End date".to_owned());
                } else if day.in_range {
                    label.push("In range".to_owned());
                }
                if day.today {
                    label.push("Today".to_owned());
                }
                label.push(long_date(day.date));
                let group = format!(
                    "date-range-picker-day-{}-{}-{}",
                    day.date.year(),
                    day.date.month(),
                    day.date.day()
                );
                let mut cell =
                    cell.group(group.clone())
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(dp(CELL_SIZE))
                        .aria_label(label.join(", "))
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
                                // 状態レイヤーは容器の上に文字の色を重ねる。日付の容器の形 (CornerFull) は円なので rounded_full で同じ形になる
                                .when(day.enabled, |this| {
                                    this.child(
                                        div()
                                            .id("state-layer")
                                            .absolute()
                                            .inset_0()
                                            .rounded_full()
                                            .when(day.focus_visible, |this| {
                                                this.bg(content
                                                    .opacity(state::FOCUS_STATE_LAYER_OPACITY))
                                            })
                                            .group_hover(group.clone(), |style| {
                                                style.bg(content
                                                    .opacity(state::HOVER_STATE_LAYER_OPACITY))
                                            })
                                            .group_active(group.clone(), |style| {
                                                style.bg(content
                                                    .opacity(state::PRESSED_STATE_LAYER_OPACITY))
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
        .aria_label("Select dates")
        .flex_1()
        .min_h_0()
        .px(dp(HORIZONTAL_PADDING));
        if let Some(enabled) = self.enabled {
            calendar = calendar.enabled(move |date| enabled(date));
        }
        if let Some(on_change) = self.on_change {
            calendar = calendar.on_change(move |range, window, cx| on_change(range, window, cx));
        }

        let mut root = div()
            .id(self.id)
            .flex()
            .flex_col()
            .w(dp(tokens::CONTAINER_WIDTH))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .child(header)
            .child(
                div()
                    .flex_none()
                    .h(dp(divider::THICKNESS))
                    .bg(theme.color(divider::COLOR)),
            )
            .child(weekdays)
            .child(calendar);
        root.style().refine(&self.style);
        root
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use md3_theme::md3_color::ColorRole;

    fn date(year: i32, month: u8, day: u8) -> Date {
        Date::new(year, month, day).expect("テストの日付が有効であること")
    }

    /// (0, 0) に高さ 520 のピッカーを置くビュー
    struct RangeView {
        selection: DateRange,
        changes: Rc<RefCell<Vec<DateRange>>>,
    }

    impl Render for RangeView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().size_full().child(
                DateRangePicker::new("range", self.selection)
                    .today(date(2026, 11, 24))
                    .on_change(cx.listener(move |this, range: &DateRange, _, cx| {
                        changes.borrow_mut().push(*range);
                        this.selection = *range;
                        cx.notify();
                    }))
                    .h(px(520.)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        selection: DateRange,
    ) -> (gpui::Entity<RangeView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            RangeView {
                selection,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
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

    #[gpui::test]
    fn 範囲の帯を両端の日付の中央から塗る(cx: &mut TestAppContext) {
        let (_, cx) = setup(
            cx,
            DateRange {
                start: Some(date(2026, 11, 10)),
                end: Some(date(2026, 11, 20)),
            },
        );
        // 2026 年 11 月は日曜日から始まる。10 日は 2 週目の火曜日、20 日は 3 週目の金曜日
        let bands = quads(cx, tokens::RANGE_SELECTION_ACTIVE_INDICATOR_CONTAINER_COLOR);
        assert_eq!(bands.len(), 2, "週ごとに帯を塗ること: {bands:?}");
        let (first, second) = (bands[0], bands[1]);
        assert_eq!(
            (first.0, first.2, first.3),
            (12. + 48. * 2. + 24., 48. * 4. + 24., 40.)
        );
        assert_eq!((second.0, second.2, second.3), (12., 48. * 5. + 24., 40.));
        assert_eq!(second.1 - first.1, 48.);
        // 始まりと終わりの日付は円で塗る
        let circles = quads(cx, tokens::DATE_SELECTED_CONTAINER_COLOR);
        assert_eq!(
            circles
                .iter()
                .map(|circle| (circle.0, circle.2))
                .collect::<Vec<_>>(),
            [(12. + 48. * 2. + 4., 40.), (12. + 48. * 5. + 4., 40.)]
        );
    }

    #[gpui::test]
    fn 日付を_2_回選ぶと範囲を求める(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            DateRange {
                start: Some(date(2026, 11, 10)),
                end: None,
            },
        );
        // 10 日の円の中央を基準に、3 週目の金曜日 (20 日) を押す
        let circles = quads(cx, tokens::DATE_SELECTED_CONTAINER_COLOR);
        let (x, y) = (circles[0].0 + 20., circles[0].1 + 20.);
        cx.simulate_click(point(px(x + 48. * 3.), px(y + 48.)), Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| view.changes.borrow().clone()),
            [DateRange {
                start: Some(date(2026, 11, 10)),
                end: Some(date(2026, 11, 20)),
            }]
        );
    }
}
