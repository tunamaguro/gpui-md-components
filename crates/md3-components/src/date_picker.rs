// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt
//         (`DatePicker`、`DateEntryContainer`、`DatePickerHeader`、`DatePickerDefaults.DatePickerTitle`、
//         `DatePickerDefaults.DatePickerHeadline`、`DatePickerContent`、`MonthsNavigation`、`WeekDays`、`Month`、
//         `Day`、`dayContentDescription`、`YearPicker`、`Year`、`YearPickerMenuButton`、`DatePickerColors`、
//         `defaultDatePickerColors`、`YearRange`、`RecommendedSizeForAccessibility`、`MonthYearHeight`、
//         `DatePickerHorizontalPadding`、`DatePickerTitlePadding`、`DatePickerHeadlinePadding`、
//         `YearsVerticalPadding`、`MaxCalendarRows`、`YearsInRow`、`DisplayMode`、`DisplayModeToggleButton`、
//         `DatePickerModeTogglePadding`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateInput.kt
//         (`DateInputContent`、`DateInputTextField`、`DateInputValidator`、`DateInputTransformation`、
//         `DateOutputTransformation`、`InputTextFieldPadding`、`InputTextNonErroneousBottomPadding`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_date_picker_*`、`m3c_date_input_*`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のデイトピッカー

use std::ops::RangeInclusive;
use std::rc::Rc;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce,
    ScrollHandle, StatefulInteractiveElement, StyleRefinement, Styled, Transformation, Window, div,
    prelude::FluentBuilder, radians, svg,
};
use gpui_headless::calendar::{Calendar, Date};
use gpui_headless::text::StaticText;
use gpui_headless::year_grid::YearGrid;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::date_picker_modal as tokens;
use md3_theme::md3_tokens::component::divider;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use md3_icons::StyledIcon;

use crate::shape::Shaped;
use crate::{
    Button, ButtonColors, ButtonVariant, IconButton, IconButtonVariant, TextField, TextFieldVariant,
};

/// 日付と曜日のセルの大きさ (Compose の `RecommendedSizeForAccessibility`)
pub(crate) const CELL_SIZE: Dp = Dp(48.0);
/// 月を移るボタンの行の高さ (Compose の `MonthYearHeight`)
const MONTH_YEAR_HEIGHT: Dp = Dp(56.0);
/// 月の表示の左右の余白 (Compose の `DatePickerHorizontalPadding`)
pub(crate) const HORIZONTAL_PADDING: Dp = Dp(12.0);
/// タイトルと見出しの左右の余白 (Compose の `DatePickerTitlePadding`、`DatePickerHeadlinePadding`)
const HEADER_START_PADDING: Dp = Dp(24.0);
pub(crate) const HEADER_END_PADDING: Dp = Dp(12.0);
/// タイトルの上と見出しの下の余白
const TITLE_TOP_PADDING: Dp = Dp(16.0);
const HEADLINE_BOTTOM_PADDING: Dp = Dp(12.0);
/// 既定の年の範囲 (Compose の `DatePickerDefaults.YearRange`)
pub(crate) const DEFAULT_YEAR_RANGE: RangeInclusive<i32> = 1900..=2100;
/// 年の一覧の 1 行の年の数 (Compose の `YearsInRow`)
const YEARS_IN_ROW: usize = 3;
/// 年の一覧の行の間隔 (Compose の `YearsVerticalPadding`)
const YEARS_VERTICAL_PADDING: Dp = Dp(16.0);
/// 年の一覧の高さ。曜日と 6 週の格子の高さから、下の区切り線の太さを引く
/// (Compose の `RecommendedSizeForAccessibility * (MaxCalendarRows + 1) - DividerDefaults.Thickness`)
const YEAR_PICKER_HEIGHT: Dp = Dp(48.0 * 7.0 - 1.0);
/// 年を選ぶボタンのアイコンとラベルの間隔 (Compose の `ButtonDefaults.IconSpacing`)
const MENU_ICON_SPACING: Dp = Dp(8.0);
/// 年を選ぶボタンのアイコンの大きさ (Compose の `Icons.Filled.ArrowDropDown` の既定の大きさ)
const MENU_ICON_SIZE: Dp = Dp(24.0);
/// 表示を切り替えるボタンの右と下の余白 (Compose の `DatePickerModeTogglePadding`)
const MODE_TOGGLE_PADDING: Dp = Dp(12.0);
/// 日付を入れる欄の左右と上の余白 (Compose の `InputTextFieldPadding`)
const INPUT_HORIZONTAL_PADDING: Dp = Dp(24.0);
const INPUT_TOP_PADDING: Dp = Dp(10.0);
/// 日付を入れる欄の、誤りを示していないときの下の余白 (Compose の `InputTextNonErroneousBottomPadding`)。
/// 誤りを示している間は、補足の文の上の余白の分だけ減らす
const INPUT_BOTTOM_PADDING: Dp = Dp(16.0);
/// 日付を入れる欄に入れる数字の数。MM/DD/YYYY の区切りを除いた長さ
const INPUT_DIGITS: usize = 8;

pub(crate) const MONTH_NAMES: [&str; 12] = [
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
pub(crate) const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

type ChangeHandler = Rc<dyn Fn(&Option<Date>, &mut Window, &mut App)>;
type ModeChangeHandler = Rc<dyn Fn(&DatePickerDisplayMode, &mut Window, &mut App)>;
type EnabledFn = Rc<dyn Fn(&Date) -> bool>;

/// デイトピッカーの表示
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerDisplayMode {
    /// 1 か月の日付を並べたカレンダーから選ぶ
    #[default]
    Picker,
    /// 日付を MM/DD/YYYY の形の文字で入れる
    Input,
}

/// 日付を入れる欄の数字。MMDDYYYY の 8 桁
fn input_digits(date: Date) -> String {
    format!("{:02}{:02}{:04}", date.month(), date.day(), date.year())
}

/// 日付を入れる欄に表示する文字。月と日の後ろに数字が続くときだけ区切りの `/` を入れる
fn input_text(digits: &str) -> String {
    let mut text = String::with_capacity(digits.len() + 2);
    for (i, c) in digits.chars().enumerate() {
        if i == 2 || i == 4 {
            text.push('/');
        }
        text.push(c);
    }
    text
}

/// 月の略称・日・年の表記。Nov 24, 2026 のように書く
pub(crate) fn short_date(date: Date) -> String {
    format!(
        "{} {}, {}",
        &MONTH_NAMES[usize::from(date.month()) - 1][..3],
        date.day(),
        date.year()
    )
}

/// 日付を入れる欄の数字 (MMDDYYYY の 8 桁まで) を日付として確かめる。誤りなら、利用者に示す文を返す
fn validate_input(
    digits: &str,
    years: &RangeInclusive<i32>,
    enabled: Option<&EnabledFn>,
) -> Result<Date, String> {
    let date = (digits.len() == INPUT_DIGITS)
        .then(|| {
            Date::new(
                digits[4..].parse().ok()?,
                digits[..2].parse().ok()?,
                digits[2..4].parse().ok()?,
            )
        })
        .flatten()
        .ok_or_else(|| "Date does not match expected pattern: MM/DD/YYYY".to_owned())?;
    if !years.contains(&date.year()) {
        return Err(format!(
            "Date out of expected year range {} - {}",
            years.start(),
            years.end()
        ));
    }
    if enabled.is_some_and(|enabled| !enabled(&date)) {
        return Err(format!("Date not allowed: {}", short_date(date)));
    }
    Ok(date)
}

/// 曜日・月・日・年をそろえた長い表記。Tuesday, November 24, 2026 のように書く
pub(crate) fn long_date(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        WEEKDAY_NAMES[usize::from(date.weekday())],
        MONTH_NAMES[usize::from(date.month()) - 1],
        date.day(),
        date.year()
    )
}

/// MD3 のデイトピッカー (Compose の `DatePicker`)
///
/// 上にタイトル Select date と、選ばれている日付の見出しを置き、区切り線の下に日付を選ぶ内容を置く。
/// 内容は [`DatePicker::display_mode`] で変わる。
/// [`DatePickerDisplayMode::Picker`] では、表示している月の年を選ぶボタンと前後の月へ移るボタン、
/// 曜日の見出しと 1 か月の日付を、日曜日から始まる 6 週の格子に並べる。
/// [`DatePickerDisplayMode::Input`] では、日付を MM/DD/YYYY の形で入れる欄を置く。
/// [`DatePicker::on_display_mode_change`] を設定すると、見出しの右に表示を切り替えるボタンを置く。
/// 選ばれている日付は呼び出し側が持ち、[`DatePicker::new`] に毎回渡す。ピッカー自身は日付を変えず、
/// 選ぶ操作をされた日付を [`DatePicker::on_change`] で求める。
/// 表示している月はピッカーが持つ。最初は選ばれている日付の月、無ければ [`DatePicker::today`] の月、
/// どちらも無ければ [`DatePicker::year_range`] の最初の年の 1 月にする
///
/// - 前後の月へ移るボタンで、表示する月を変える。[`DatePicker::year_range`] の端の月ではボタンを無効にする
/// - 表示している月のボタンで、日付の代わりに [`DatePicker::year_range`] の年を 3 年ずつ並べた一覧を開閉する。
///   開くと表示している年にフォーカスを移す。年を選ぶと、その年の表示していた月を表示して一覧を閉じ、
///   フォーカスを表示している月のボタンへ戻す。年の一覧の操作は [`YearGrid`] と同じ。一覧を開いている間は
///   前後の月へ移るボタンを置かない
/// - 日付の操作は [`Calendar`] と同じ。キー操作で月の外の日付へ移ると、表示する月も変える。
///   [`DatePicker::year_range`] の外の日付へは移らない
/// - 日付を入れる欄は、数字を 8 桁まで受け付け、月と日の後ろに区切りの `/` を表示する。ほかの文字を入れる編集と、
///   区切りだけを消す編集は受け付けない。8 桁そろうと日付として確かめ、選べる日付ならその日付を、
///   そろっていないか選べない日付なら `None` を [`DatePicker::on_change`] で求める。
///   選べない日付なら欄の下に理由を示す。欄の文字は、[`DatePicker::new`] に渡す日付が欄の日付と違う日付に
///   変わったときに、その日付にそろえる
/// - [`DatePickerDisplayMode::Input`] の内容を表示し始めたとき、日付を入れる欄にフォーカスを移す
///
/// 支援技術には、日付の格子が表示している月 (November 2026 のような名前) の role `grid` として公開される。
/// 曜日の見出しは role `columnheader`、日付は role `gridcell` で、選ばれているかどうかと選べるかどうかを持ち、
/// 名前は Today, Tuesday, November 24, 2026 のように、今日であることと曜日・月・日・年を書く。
/// 選ばれている日付の見出し (Current selection: に続けて日付、選ばれていなければ None) と表示している月は
/// role `label` で、変わったときに読み上げ中の内容を遮らずに知らせる。
/// 表示している月のボタンは role `button` で、一覧を開いているかどうかを持ち、名前は表示している月に続けて、
/// 閉じているときは Activate to change the year、開いているときは Activate to change month and date と書く。
/// 年の一覧は role `grid` で名前は Year picker visible、年は role `gridcell` で名前は Navigate to year 2026 のように書く。
/// 前後の月へ移るボタンの名前は Change to previous month と Change to next month。
/// [`DatePickerDisplayMode::Input`] では、見出しは Entered date: に続けて日付 (無ければ None) を知らせ、
/// 日付を入れる欄は role `textbox` で名前は Date、プレースホルダーは MM/DD/YYYY、誤りの理由は説明として公開する。
/// 表示を切り替えるボタンの名前は Switch to text input mode と Switch to calendar input mode。
/// 文言は英語に固定している
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    selected: Option<Date>,
    today: Option<Date>,
    year_range: RangeInclusive<i32>,
    enabled: Option<EnabledFn>,
    on_change: Option<ChangeHandler>,
    display_mode: DatePickerDisplayMode,
    on_display_mode_change: Option<ModeChangeHandler>,
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
            display_mode: DatePickerDisplayMode::default(),
            on_display_mode_change: None,
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
    /// [`DatePickerDisplayMode::Input`] で入れた文字が選べる日付にならない間は `None` を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`DatePicker::new`] に渡すこと。
    /// `None` も反映しないと、見出しは前の日付のまま、欄は入れた文字のままになる
    pub fn on_change(
        mut self,
        handler: impl Fn(&Option<Date>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// 表示を `mode` にする。既定は [`DatePickerDisplayMode::Picker`]
    ///
    /// 表示が変わると、選ばれている日付があればその月を表示する
    pub fn display_mode(mut self, mode: DatePickerDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// 表示を切り替えるボタンを置き、押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には切り替えた後の表示を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`DatePicker::display_mode`] に渡すこと。設定しない場合はボタンを置かない
    pub fn on_display_mode_change(
        mut self,
        handler: impl Fn(&DatePickerDisplayMode, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_display_mode_change = Some(Rc::new(handler));
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
        // Compose と同じく、表示を切り替えたときは選ばれている日付の月を表示する
        let last_mode =
            window.use_keyed_state((self.id.clone(), "last-mode"), cx, |_, _| self.display_mode);
        if *last_mode.read(cx) != self.display_mode {
            last_mode.update(cx, |last_mode, _| *last_mode = self.display_mode);
            if let Some(date) = selected {
                displayed.update(cx, |displayed, _| *displayed = date);
            }
        }
        // year_range が変わって表示している月が範囲の外になった場合は、範囲の端の月を表示する。
        // 後で範囲を広げたときに元の月へ戻らないよう、補正した月を持つ
        let stored = *displayed.read(cx);
        let displayed_month = stored.clamp(first, last);
        if displayed_month != stored {
            displayed.update(cx, |displayed, _| *displayed = displayed_month);
        }
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
        let input_mode = self.display_mode == DatePickerDisplayMode::Input;
        let (empty_headline, headline_prefix) = if input_mode {
            ("Entered date", "Entered date")
        } else {
            ("Selected date", "Current selection")
        };
        let (headline_text, headline_label) = match selected {
            Some(date) => (
                short_date(date),
                format!("{headline_prefix}: {}", long_date(date)),
            ),
            None => (
                empty_headline.to_owned(),
                format!("{headline_prefix}: None"),
            ),
        };
        let mut headline = StaticText::new((self.id.clone(), "headline"), headline_label)
            .live(true)
            .flex_1()
            .min_w_0()
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
            .child(div().flex().items_center().child(headline).when_some(
                self.on_display_mode_change,
                |this, handler| {
                    let (icon, label, target) = if input_mode {
                        (
                            md3_icons::DATE_RANGE,
                            "Switch to calendar input mode",
                            DatePickerDisplayMode::Picker,
                        )
                    } else {
                        (
                            md3_icons::EDIT,
                            "Switch to text input mode",
                            DatePickerDisplayMode::Input,
                        )
                    };
                    this.child(
                        div()
                            .flex_none()
                            .mr(dp(MODE_TOGGLE_PADDING))
                            .mb(dp(MODE_TOGGLE_PADDING))
                            .child(
                                IconButton::new((self.id.clone(), "mode-toggle"), icon, label)
                                    .variant(IconButtonVariant::Standard)
                                    .on_click(move |_, window, cx| handler(&target, window, cx))
                                    .theme(theme.clone()),
                            ),
                    )
                },
            ));
        let divider = || {
            div()
                .flex_none()
                .h(dp(divider::THICKNESS))
                .bg(theme.color(divider::COLOR))
        };

        // 表示している月の、年を選ぶ一覧との切り替えボタンと、前後の月へ移るボタン
        let year_mode = window.use_keyed_state((self.id.clone(), "year-mode"), cx, |_, _| false);
        let showing_years = *year_mode.read(cx);
        // カレンダーへ戻したときは日付を表示する
        if input_mode && showing_years {
            year_mode.update(cx, |year_mode, _| *year_mode = false);
        }
        let menu_focus = window
            .use_keyed_state((self.id.clone(), "menu-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let year_scroll = window
            .use_keyed_state((self.id.clone(), "year-scroll"), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        let month_text = format!("{} {}", MONTH_NAMES[usize::from(month) - 1], year);
        let content_color = theme.color(ColorRole::OnSurfaceVariant);
        let menu_button = Button::new((self.id.clone(), "year-menu"))
            .variant(ButtonVariant::Text)
            .colors(ButtonColors {
                content: Some(ColorRole::OnSurfaceVariant),
                ..ButtonColors::default()
            })
            .track_focus(&menu_focus)
            .expanded(showing_years)
            .aria_label(format!(
                "{month_text}, {}",
                if showing_years {
                    "Activate to change month and date"
                } else {
                    "Activate to change the year"
                }
            ))
            .on_click({
                let year_mode = year_mode.clone();
                let year_scroll = year_scroll.clone();
                move |_, _, cx| {
                    let opening = !*year_mode.read(cx);
                    if opening {
                        // 表示している年の 1 行上から見せる (Compose の `initialFirstVisibleItemIndex`)。
                        // 行の位置はこの描画で決まるため、行の高さと間隔から求める
                        let row = (year - first_year) as usize / YEARS_IN_ROW;
                        let pitch = dp(tokens::SELECTION_YEAR_CONTAINER_HEIGHT)
                            + dp(YEARS_VERTICAL_PADDING);
                        year_scroll.set_offset(gpui::point(
                            gpui::px(0.),
                            -pitch * row.saturating_sub(1) as f32,
                        ));
                    }
                    year_mode.update(cx, |year_mode, cx| {
                        *year_mode = opening;
                        cx.notify();
                    });
                }
            })
            .child(
                StaticText::new((self.id.clone(), "month-label"), month_text.clone())
                    .live(true)
                    .child(month_text.clone()),
            )
            .child(
                svg()
                    .data(
                        StyledIcon::from(md3_icons::ARROW_DROP_DOWN)
                            .svg()
                            .as_bytes(),
                    )
                    .flex_none()
                    .size(dp(MENU_ICON_SIZE))
                    .ml(dp(MENU_ICON_SPACING))
                    .text_color(content_color)
                    .when(showing_years, |this| {
                        this.with_transformation(Transformation::rotate(radians(
                            std::f32::consts::PI,
                        )))
                    }),
            )
            .theme(theme.clone());
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
            .child(menu_button)
            .when(!showing_years, |this| {
                this.child(
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
                )
            });

        // 年の一覧。選んだ年の、表示している月と同じ月を表示して、日付の表示に戻す
        let mut year_font = theme.text_style(tokens::SELECTION_YEAR_LABEL_TEXT_FONT);
        year_font.color = None;
        let years = YearGrid::new(
            (self.id.clone(), "years"),
            first_year..=last_year,
            YEARS_IN_ROW,
        )
        .selected(Some(year))
        .current(today.map(|today| today.year()))
        .scroll(&year_scroll)
        .autofocus(true)
        .on_change({
            let set_month = set_month.clone();
            let year_mode = year_mode.clone();
            let menu_focus = menu_focus.clone();
            move |&selected_year: &i32, window, cx| {
                // 支援技術のクリック操作はフォーカスを年へ移さないため、一覧の外から選ばれても
                // 表示している月のボタンへフォーカスを戻す
                window.focus(&menu_focus, cx);
                if let Some(date) = Date::new(selected_year, month, 1) {
                    set_month(date.clamp(first, last), cx);
                }
                year_mode.update(cx, |year_mode, cx| {
                    *year_mode = false;
                    cx.notify();
                });
            }
        })
        .row(|row| row.flex_none().justify_around())
        .blank(|blank| {
            blank
                .flex_none()
                .w(dp(tokens::SELECTION_YEAR_CONTAINER_WIDTH))
                .h(dp(tokens::SELECTION_YEAR_CONTAINER_HEIGHT))
        })
        .year({
            let theme = theme.clone();
            move |cell, element, _, _| {
                let disabled_alpha = supplement::color_scheme::DISABLED_ALPHA;
                let (container, content) = match (cell.selected, cell.enabled, cell.current) {
                    (true, true, _) => (
                        Some(theme.color(tokens::SELECTION_YEAR_SELECTED_CONTAINER_COLOR)),
                        theme.color(tokens::SELECTION_YEAR_SELECTED_LABEL_TEXT_COLOR),
                    ),
                    (true, false, _) => (
                        Some(
                            theme
                                .color(tokens::SELECTION_YEAR_SELECTED_CONTAINER_COLOR)
                                .opacity(disabled_alpha),
                        ),
                        theme
                            .color(tokens::SELECTION_YEAR_SELECTED_LABEL_TEXT_COLOR)
                            .opacity(disabled_alpha),
                    ),
                    (false, true, true) => (None, theme.color(tokens::DATE_TODAY_LABEL_TEXT_COLOR)),
                    (false, true, false) => (
                        None,
                        theme.color(tokens::SELECTION_YEAR_UNSELECTED_LABEL_TEXT_COLOR),
                    ),
                    (false, false, _) => (
                        None,
                        theme
                            .color(tokens::SELECTION_YEAR_UNSELECTED_LABEL_TEXT_COLOR)
                            .opacity(disabled_alpha),
                    ),
                };
                let group = format!("date-picker-year-{}", cell.year);
                // 今年は日付の今日と同じ縁取りで示す (Compose の `Year`)
                let container = div()
                    .absolute()
                    .inset_0()
                    .when_some(container, |this, container| this.bg(container))
                    .when(cell.current && !cell.selected, |this| {
                        this.border(dp(tokens::DATE_TODAY_CONTAINER_OUTLINE_WIDTH))
                            .border_color(theme.color(tokens::DATE_TODAY_CONTAINER_OUTLINE_COLOR))
                    });
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                let mut element = element
                    .group(group.clone())
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .w(dp(tokens::SELECTION_YEAR_CONTAINER_WIDTH))
                    .h(dp(tokens::SELECTION_YEAR_CONTAINER_HEIGHT))
                    .text_color(content)
                    .aria_label(format!("Navigate to year {}", cell.year))
                    .child(Shaped::new(
                        container,
                        theme.clone(),
                        tokens::SELECTION_YEAR_STATE_LAYER_SHAPE,
                    ))
                    // 日付と同じく、状態レイヤーは容器の上に文字の色を重ねる別の層にする。
                    // 年の容器の形 (CornerFull) は短辺の両端を半円にした形なので rounded_full で同じ形になる
                    .when(cell.enabled, |this| {
                        this.child(
                            div()
                                .id("state-layer")
                                .absolute()
                                .inset_0()
                                .rounded_full()
                                .when(cell.focus_visible, |this| {
                                    this.bg(content.opacity(state::FOCUS_STATE_LAYER_OPACITY))
                                })
                                .group_hover(group.clone(), |style| {
                                    style.bg(content.opacity(state::HOVER_STATE_LAYER_OPACITY))
                                })
                                .group_active(group.clone(), |style| {
                                    style.bg(content.opacity(state::PRESSED_STATE_LAYER_OPACITY))
                                }),
                        )
                    })
                    .child(div().relative().child(cell.year.to_string()))
                    .when(cell.focus_visible, |this| {
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
                    });
                element.text_style().refine(&year_font);
                element
            }
        })
        .aria_label("Year picker visible")
        .h(dp(YEAR_PICKER_HEIGHT))
        .gap(dp(YEARS_VERTICAL_PADDING))
        .overflow_y_scroll();

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
                                // 状態レイヤーは容器の上に文字の色を重ねる (Compose の Surface の ripple は内容の色を使う)。
                                // group_active は id を持つ要素にしか付けられないため、容器とは別の層にする。
                                // gpui は group_active を group_hover の後に当てるので、押している間は押下の色だけになる。
                                // 日付の容器の形 (CornerFull) は円なので rounded_full で同じ形になる
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
            .aria_label(month_text);
        if let Some(on_change) = self.on_change.clone() {
            calendar =
                calendar.on_change(move |date, window, cx| on_change(&Some(*date), window, cx));
        }

        // 日付を入れる欄。欄の数字は、この表示の間だけピッカーが持つ
        let input = input_mode.then(|| {
            let years = first_year..=last_year;
            let digits = window.use_keyed_state((self.id.clone(), "input-digits"), cx, |_, _| {
                (selected.map(input_digits).unwrap_or_default(), selected)
            });
            let (mut text, synced) = digits.read(cx).clone();
            if synced != selected {
                if let Some(date) = selected
                    && validate_input(&text, &years, None).ok() != Some(date)
                {
                    text = input_digits(date);
                }
                digits.update(cx, |digits, _| *digits = (text.clone(), selected));
            }
            let error = (text.len() == INPUT_DIGITS)
                .then(|| validate_input(&text, &years, self.enabled.as_ref()).err())
                .flatten();

            let field_focus = window
                .use_keyed_state((self.id.clone(), "input-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone();
            // 表示し始めたらフォーカスを移す。ほかの要素を描いた後で描かれる場合があるため、
            // 同じフレームで複数の要素がフォーカスを持たないよう、描き終えてから移す。
            // 同じフレームを描き終えてから最初のフォーカスを移す部品 (モーダルの部品など) の中でも欄に移るよう、2 段遅らせる
            let entered =
                window.use_keyed_state((self.id.clone(), "input-entered"), cx, |_, _| false);
            if !*entered.read(cx) {
                entered.update(cx, |entered, _| *entered = true);
                let field_focus = field_focus.clone();
                window.defer(cx, move |window, cx| {
                    window.defer(cx, move |window, cx| window.focus(&field_focus, cx));
                });
            }

            let on_change = self.on_change.clone();
            let enabled = self.enabled.clone();
            let field = TextField::new((self.id.clone(), "input"), input_text(&text))
                .variant(TextFieldVariant::Outlined)
                .label("Date")
                .placeholder("MM/DD/YYYY")
                .track_focus(&field_focus)
                .error(error.is_some())
                .when_some(error.clone(), |this, error| this.supporting_text(error))
                .on_change(move |value, window, cx| {
                    let mut new_digits = String::new();
                    for c in value.chars() {
                        match c {
                            '0'..='9' => new_digits.push(c),
                            '/' => {}
                            _ => return,
                        }
                    }
                    if new_digits.len() > INPUT_DIGITS || new_digits == digits.read(cx).0 {
                        return;
                    }
                    digits.update(cx, |digits, cx| {
                        digits.0 = new_digits.clone();
                        cx.notify();
                    });
                    if let Some(on_change) = &on_change {
                        let date = validate_input(&new_digits, &years, enabled.as_ref()).ok();
                        on_change(&date, window, cx);
                    }
                })
                .theme(theme.clone())
                .w_full();
            div()
                .px(dp(INPUT_HORIZONTAL_PADDING))
                .pt(dp(INPUT_TOP_PADDING))
                .pb(if error.is_some() {
                    dp(INPUT_BOTTOM_PADDING) - dp(supplement::text_field::SUPPORTING_TOP_PADDING)
                } else {
                    dp(INPUT_BOTTOM_PADDING)
                })
                .child(field)
        });

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w(dp(tokens::CONTAINER_WIDTH))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .child(header)
            .child(divider())
            .when(!input_mode, |this| {
                this.child(navigation)
                    // 年の一覧は、年を選ぶか表示している月のボタンを押したときだけ閉じ、
                    // どちらの場合もフォーカスは表示している月のボタンにある
                    .when(showing_years, |this| {
                        this.child(years.px(dp(HORIZONTAL_PADDING)))
                            .child(divider())
                    })
                    .when(!showing_years, |this| {
                        this.child(div().px(dp(HORIZONTAL_PADDING)).child(calendar))
                    })
            })
            .children(input);
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
        changes: Rc<RefCell<Vec<Option<Date>>>>,
        mode: DatePickerDisplayMode,
        /// 表示を切り替えるボタンを置くか
        toggle: bool,
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
                    .on_change(cx.listener(move |this, date: &Option<Date>, _, cx| {
                        changes.borrow_mut().push(*date);
                        this.selected = *date;
                        cx.notify();
                    }))
                    .display_mode(self.mode)
                    .when(self.toggle, |this| {
                        this.on_display_mode_change(cx.listener(
                            |this, mode: &DatePickerDisplayMode, _, cx| {
                                this.mode = *mode;
                                cx.notify();
                            },
                        ))
                    })
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
                mode: DatePickerDisplayMode::Picker,
                toggle: false,
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn changes(view: &gpui::Entity<PickerView>, cx: &mut VisualTestContext) -> Vec<Option<Date>> {
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
        assert_eq!(changes(&view, cx), [Some(date(2026, 11, 11))]);
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
        assert_eq!(changes(&view, cx), [Some(date(2026, 12, 1))]);
    }

    /// 年の一覧の上端。見出し 120dp、区切り線 1dp、月の行 56dp の下
    const YEARS_TOP: f32 = 120. + 1. + 56.;

    #[gpui::test]
    fn 年を選ぶボタンで年の一覧を開き_選んだ年の同じ月を表示する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        cx.simulate_click(point(px(60.), px(121. + 28.)), Modifiers::none());
        cx.run_until_parked();
        // 1900 年から 3 年ずつ並ぶので、2026 年は 43 行目の 1 列目。その 1 行上から見せる。
        // 列は 336dp を 3 等分した 112dp の中央に置く
        assert_eq!(
            quads(cx, tokens::SELECTION_YEAR_SELECTED_CONTAINER_COLOR),
            [(12. + 56. - 36., YEARS_TOP + 36. + 16., 72., 36.)]
        );
        cx.simulate_click(
            point(px(12. + 112. + 56.), px(YEARS_TOP + 52. + 18.)),
            Modifiers::none(),
        );
        cx.run_until_parked();
        // 2027 年 11 月 1 日は月曜日
        cx.simulate_click(cell_center(0, 1), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 11, 1))]);
    }

    #[gpui::test]
    fn 年の一覧の最後の行も列の位置をそろえる(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, Some(date(2027, 11, 10)), 2024..=2027);
        cx.simulate_click(point(px(60.), px(121. + 28.)), Modifiers::none());
        cx.run_until_parked();
        // 2027 年は 2 行目の 1 列目
        assert_eq!(
            quads(cx, tokens::SELECTION_YEAR_SELECTED_CONTAINER_COLOR),
            [(12. + 56. - 36., YEARS_TOP + 52., 72., 36.)]
        );
    }

    #[gpui::test]
    fn 年の一覧の行の間を押してもキー操作で年を選べる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        let press = |cx: &mut VisualTestContext| {
            let keystroke =
                gpui::Keystroke::parse("enter").expect("テストのキー表記が有効であること");
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
            cx.run_until_parked();
        };
        cx.simulate_keystrokes("tab");
        press(cx);
        // 1 行目と 2 行目の間の余白
        cx.simulate_click(
            point(px(12. + 56.), px(YEARS_TOP + 36. + 8.)),
            Modifiers::none(),
        );
        cx.simulate_keystrokes("right");
        press(cx);
        // 2027 年 11 月 1 日は月曜日
        cx.simulate_click(cell_center(0, 1), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 11, 1))]);
    }

    #[gpui::test]
    fn 年の一覧の下の区切り線と行の間を右ボタンで押してもキー操作で年を選べる(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        let press = |cx: &mut VisualTestContext| {
            let keystroke =
                gpui::Keystroke::parse("enter").expect("テストのキー表記が有効であること");
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
            cx.run_until_parked();
        };
        cx.simulate_keystrokes("tab");
        press(cx);
        // 一覧の下の区切り線
        cx.simulate_click(point(px(100.), px(YEARS_TOP + 335.5)), Modifiers::none());
        // 1 行目と 2 行目の間の余白を右ボタンで押す
        let gap = point(px(12. + 56.), px(YEARS_TOP + 36. + 8.));
        cx.simulate_mouse_down(gap, gpui::MouseButton::Right, Modifiers::none());
        cx.simulate_mouse_up(gap, gpui::MouseButton::Right, Modifiers::none());
        cx.simulate_keystrokes("right");
        press(cx);
        // 2027 年 11 月 1 日は月曜日
        cx.simulate_click(cell_center(0, 1), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 11, 1))]);
    }

    #[gpui::test]
    fn キーボードで年を選ぶと年を選ぶボタンにフォーカスを戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        let press = |cx: &mut VisualTestContext| {
            let keystroke =
                gpui::Keystroke::parse("enter").expect("テストのキー表記が有効であること");
            cx.simulate_event(gpui::KeyDownEvent {
                keystroke: keystroke.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(gpui::KeyUpEvent { keystroke });
            cx.run_until_parked();
        };
        // 開くと表示している年にフォーカスが移る
        cx.simulate_keystrokes("tab");
        press(cx);
        cx.simulate_keystrokes("right");
        press(cx);
        // 年を選ぶボタンの次は前の月のボタン
        cx.simulate_keystrokes("tab");
        press(cx);
        // 2027 年 10 月 1 日は金曜日
        cx.simulate_click(cell_center(0, 5), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 10, 1))]);
    }

    #[gpui::test]
    fn キー操作で月の外へ移ると表示する月を変える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 30)), DEFAULT_YEAR_RANGE);
        // 年を選ぶボタンと前後の月のボタンの次が日付
        cx.simulate_keystrokes("tab tab tab tab right");
        let keystroke = gpui::Keystroke::parse("enter").expect("テストのキー表記が有効であること");
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke });
        cx.run_until_parked();
        assert_eq!(changes(&view, cx), [Some(date(2026, 12, 1))]);
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
        assert_eq!(changes(&view, cx), [Some(date(2026, 11, 24))]);
    }

    #[gpui::test]
    fn 年の範囲を狭めてから広げても補正した月を表示する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        for range in [2027..=2027, 2026..=2027] {
            view.update(cx, |view, cx| {
                view.year_range = range;
                cx.notify();
            });
            cx.run_until_parked();
        }
        // 2027 年 1 月 1 日は金曜日なので、2 行目の 3 列目は 5 日
        cx.simulate_click(cell_center(1, 2), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 1, 5))]);
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
        assert_eq!(changes(&view, cx), [Some(date(2026, 1, 6))]);
    }

    #[test]
    fn 入れた日付を確かめて誤りの理由を返す() {
        let years = 2026..=2026;
        let weekdays: EnabledFn = Rc::new(|date: &Date| date.weekday() != 0);
        assert_eq!(
            validate_input("11242026", &years, Some(&weekdays)),
            Ok(date(2026, 11, 24))
        );
        assert_eq!(
            validate_input("13012026", &years, None),
            Err("Date does not match expected pattern: MM/DD/YYYY".to_owned())
        );
        assert_eq!(
            validate_input("1124202", &years, None),
            Err("Date does not match expected pattern: MM/DD/YYYY".to_owned())
        );
        assert_eq!(
            validate_input("01012027", &years, None),
            Err("Date out of expected year range 2026 - 2026".to_owned())
        );
        assert_eq!(
            validate_input("11222026", &years, Some(&weekdays)),
            Err("Date not allowed: Nov 22, 2026".to_owned())
        );
    }

    /// 日付を入れる欄の文字。すべてを選択してクリップボードに写して読む
    fn field_text(cx: &mut VisualTestContext) -> String {
        if cfg!(target_os = "macos") {
            cx.simulate_keystrokes("cmd-a cmd-c");
        } else {
            cx.simulate_keystrokes("ctrl-a ctrl-c");
        }
        cx.read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default()
    }

    fn type_text(cx: &mut VisualTestContext, text: &str) {
        for c in text.chars() {
            cx.simulate_input(&c.to_string());
        }
    }

    fn mode(view: &gpui::Entity<PickerView>, cx: &mut VisualTestContext) -> DatePickerDisplayMode {
        view.read_with(cx, |view, _| view.mode)
    }

    #[gpui::test]
    fn 表示を切り替えるボタンで文字の入力に変え_欄にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        view.update(cx, |view, cx| {
            view.toggle = true;
            cx.notify();
        });
        cx.run_until_parked();
        // 見出しの行の右端の、右と下に 12 の余白を空けたボタン
        let toggle = point(px(326.), px(86.));
        cx.simulate_click(toggle, Modifiers::none());
        assert_eq!(mode(&view, cx), DatePickerDisplayMode::Input);
        assert_eq!(
            field_text(cx),
            "11/10/2026",
            "選ばれている日付を欄に入れ、欄にフォーカスを移すこと"
        );
        cx.simulate_click(toggle, Modifiers::none());
        assert_eq!(mode(&view, cx), DatePickerDisplayMode::Picker);
    }

    /// 文字の入力の表示で置いたビュー
    fn setup_input(
        cx: &mut TestAppContext,
        selected: Option<Date>,
    ) -> (gpui::Entity<PickerView>, &mut VisualTestContext) {
        let (view, cx) = setup(cx, selected, DEFAULT_YEAR_RANGE);
        view.update(cx, |view, cx| {
            view.mode = DatePickerDisplayMode::Input;
            cx.notify();
        });
        cx.run_until_parked();
        (view, cx)
    }

    #[gpui::test]
    fn 数字を_8_桁入れると日付を求め区切りを表示する(cx: &mut TestAppContext) {
        let (view, cx) = setup_input(cx, None);
        type_text(cx, "11252026");
        let mut expected = vec![None; 7];
        expected.push(Some(date(2026, 11, 25)));
        assert_eq!(changes(&view, cx), expected);
        assert_eq!(field_text(cx), "11/25/2026");
    }

    #[gpui::test]
    fn 数字以外と_8_桁を超える入力は受け付けない(cx: &mut TestAppContext) {
        let (view, cx) = setup_input(cx, None);
        type_text(cx, "1a/1");
        assert_eq!(changes(&view, cx), [None, None]);
        type_text(cx, "2520269");
        assert_eq!(changes(&view, cx).len(), 8);
        assert_eq!(field_text(cx), "11/25/2026");
    }

    #[gpui::test]
    fn 選べない日付は_none_を求める(cx: &mut TestAppContext) {
        let (view, cx) = setup_input(cx, None);
        // 日曜日は選べない
        type_text(cx, "11222026");
        assert_eq!(changes(&view, cx).last(), Some(&None));
    }

    #[gpui::test]
    fn 区切りだけを消す編集は受け付けない(cx: &mut TestAppContext) {
        let (view, cx) = setup_input(cx, Some(date(2026, 11, 25)));
        // 11/| の後ろから 1 文字消す
        cx.simulate_keystrokes("home right right right backspace");
        assert_eq!(changes(&view, cx), []);
        // カーソルは区切りの後ろに残り、後ろの数字を消せる
        cx.simulate_keystrokes("delete");
        assert_eq!(field_text(cx), "11/52/026");
    }

    #[gpui::test]
    fn 外から日付が変わると欄の文字をそろえる(cx: &mut TestAppContext) {
        let (view, cx) = setup_input(cx, None);
        type_text(cx, "1125");
        view.update(cx, |view, cx| {
            view.selected = Some(date(2027, 1, 5));
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(field_text(cx), "01/05/2027");
    }

    #[gpui::test]
    fn 表示を切り替えると選ばれている日付の月を表示する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some(date(2026, 11, 10)), DEFAULT_YEAR_RANGE);
        view.update(cx, |view, cx| {
            view.toggle = true;
            cx.notify();
        });
        cx.run_until_parked();
        let toggle = point(px(326.), px(86.));
        cx.simulate_click(toggle, Modifiers::none());
        if cfg!(target_os = "macos") {
            cx.simulate_keystrokes("cmd-a");
        } else {
            cx.simulate_keystrokes("ctrl-a");
        }
        type_text(cx, "03152027");
        cx.simulate_click(toggle, Modifiers::none());
        assert_eq!(mode(&view, cx), DatePickerDisplayMode::Picker);
        view.update(cx, |view, _| view.changes.borrow_mut().clear());
        // 2027 年 3 月は月曜日から始まるので、3 週目の月曜日は 15 日
        cx.simulate_click(cell_center(2, 1), Modifiers::none());
        assert_eq!(changes(&view, cx), [Some(date(2027, 3, 15))]);
    }
}
