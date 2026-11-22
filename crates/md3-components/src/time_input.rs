// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt
//         (`TimeInput`、`TimeInputImpl`、`TimePickerTextField`、`SupportingText`、`VerticalPeriodToggle`、
//         `ToggleItem`、`TimePickerDefaults.defaultTimeInputColors`、`DisplaySeparatorWidth`、`SupportLabelTop`、
//         `PeriodTogglePaddingSmall`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_time_picker_*`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の時刻の入力

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_headless::radio::{Radio, RadioGroup};
use gpui_headless::text_field::TextField as HeadlessTextField;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{time_input as tokens, time_picker};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 時と分の入力欄の間の区切りの幅 (Compose の `DisplaySeparatorWidth`)
const SEPARATOR_WIDTH: Dp = Dp(24.0);
/// 入力欄と補足の文の間の余白 (Compose の `SupportLabelTop`)
const SUPPORTING_TEXT_TOP: Dp = Dp(7.0);
/// 分の入力欄と午前・午後の切り替えの間の余白、切り替えの 2 つの項目の間の余白 (Compose の `PeriodTogglePaddingSmall`)
const PERIOD_TOGGLE_SPACE: Dp = Dp(4.0);
/// 入力欄の枠線の太さ (Compose の `TimePickerTextField` の `focusedBorderThickness` と `unfocusedBorderThickness`)
const FOCUSED_OUTLINE_WIDTH: Dp = Dp(2.0);
const OUTLINE_WIDTH: Dp = Dp(1.0);

type ChangeHandler = Box<dyn Fn(&Time, &mut Window, &mut App)>;

/// 時刻
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Time {
    /// 時。0 から 23
    pub hour: u8,
    /// 分。0 から 59
    pub minute: u8,
}

impl Time {
    /// `hour` 時 `minute` 分の時刻を作る。`hour` は 0 から 23、`minute` は 0 から 59 にすること
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }
}

/// [`TimeInput`] の時の表し方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HourFormat {
    /// 1 時から 12 時と、午前・午後で表す
    #[default]
    Hour12,
    /// 0 時から 23 時で表す
    Hour24,
}

/// MD3 の時刻の入力 (Compose の `TimeInput`)
///
/// 時と分の 2 つの入力欄を「:」で区切って並べる。[`HourFormat::Hour12`] では、その右に午前・午後の切り替えを縦に並べる。
/// 入力欄は幅 96dp、高さ 72dp で、下に補足の文 (Hour・Minute) を表示する。
/// 時刻は呼び出し側が持ち、[`TimeInput::new`] に毎回渡す。入力自身は時刻を変えず、変えた後の時刻を [`TimeInput::on_change`] で求める
///
/// - 入力欄には 2 桁までの数字だけを入れる。範囲内の数になると時刻を変えるよう求め、範囲外か空なら求めない。
///   範囲外か空の間は入力欄をエラーの色にし、補足の文を範囲を示す文に変える。
///   入力欄の文字列は描画をまたいで保ち、渡された時刻が入力欄の数と違えば、時刻の数で置き換える
/// - 時の入力欄に範囲内の 2 桁の数を入れると、分の入力欄へフォーカスを移す
/// - 午前・午後の切り替えは矢印キー・Space・クリックで選び、時を 12 時間ずらすよう求める
/// - フォーカスのある入力欄を選んだ色で表示する
///
/// 支援技術には、入力欄が role `textbox` (名前は Hour と Minute)、午前・午後の切り替えが role `radiogroup` として公開される。
/// 範囲外か空の入力欄は、範囲を示す文を説明として公開する。文言は英語に固定している
#[derive(IntoElement)]
pub struct TimeInput {
    id: ElementId,
    time: Time,
    format: HourFormat,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl TimeInput {
    /// `time` を表示する [`HourFormat::Hour12`] の時刻の入力を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、入力欄の文字列・フォーカス・カーソルが失われる
    pub fn new(id: impl Into<ElementId>, time: Time) -> Self {
        Self {
            id: id.into(),
            time,
            format: HourFormat::default(),
            on_change: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 時の表し方を `format` にする
    pub fn format(mut self, format: HourFormat) -> Self {
        self.format = format;
        self
    }

    /// 時刻を変える操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には変えた後の時刻を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`TimeInput::new`] に渡すこと。
    /// 設定しない場合、入力欄は文字列を編集させず、支援技術には read-only として公開される
    pub fn on_change(mut self, handler: impl Fn(&Time, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for TimeInput {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// 入力欄の種類
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Hour,
    Minute,
}

/// 入力欄の文字列と、その文字列が表す時刻の数 (時は 0 から 23)
struct FieldText {
    text: String,
    value: u8,
}

impl RenderOnce for TimeInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let on_change = self.on_change.map(std::rc::Rc::new);
        let time = self.time;
        let format = self.format;
        let minute_focus =
            util::keyed_focus_handle(&(self.id.clone(), "minute").into(), window, cx);

        let field = |kind: Field, window: &mut Window, cx: &mut App| {
            let field_id = ElementId::from((
                self.id.clone(),
                match kind {
                    Field::Hour => "hour",
                    Field::Minute => "minute",
                },
            ));
            let focus_handle = util::keyed_focus_handle(&field_id, window, cx);
            let value = match kind {
                Field::Hour => time.hour,
                Field::Minute => time.minute,
            };
            let display = |value: u8| match (kind, format) {
                (Field::Hour, HourFormat::Hour12) => match value % 12 {
                    0 => 12,
                    hour => hour,
                },
                _ => value,
            };
            let state = window.use_keyed_state((field_id.clone(), "text"), cx, |_, _| FieldText {
                text: format!("{:02}", display(value)),
                value,
            });
            // 渡された時刻が入力欄の数と違えば、時刻の数で置き換える
            let text = state.update(cx, |state, _| {
                if state.value != value {
                    state.text = format!("{:02}", display(value));
                    state.value = value;
                }
                state.text.clone()
            });
            // 入力欄の文字列を時刻の数にする。範囲外か空なら None
            let parse = move |text: &str| -> Option<u8> {
                let number: u8 = text.parse().ok()?;
                match (kind, format) {
                    (Field::Hour, HourFormat::Hour12) => (1..=12)
                        .contains(&number)
                        .then(|| number % 12 + if time.hour >= 12 { 12 } else { 0 }),
                    (Field::Hour, HourFormat::Hour24) => (number <= 23).then_some(number),
                    (Field::Minute, _) => (number <= 59).then_some(number),
                }
            };
            let valid = parse(&text).is_some();
            let selected = focus_handle.is_focused(window);
            let (container, content) = if !valid {
                (ColorRole::ErrorContainer, ColorRole::Error)
            } else if selected {
                (
                    time_picker::TIME_SELECTOR_SELECTED_CONTAINER_COLOR,
                    time_picker::TIME_SELECTOR_SELECTED_LABEL_TEXT_COLOR,
                )
            } else {
                (
                    time_picker::TIME_SELECTOR_UNSELECTED_CONTAINER_COLOR,
                    time_picker::TIME_SELECTOR_UNSELECTED_LABEL_TEXT_COLOR,
                )
            };
            // 選ばれていない有効な入力欄は Compose では枠線の無い `TimeSelector` で描く
            let outline = match (valid, selected) {
                (false, true) => Some((ColorRole::Error, FOCUSED_OUTLINE_WIDTH)),
                (false, false) => Some((ColorRole::Error, OUTLINE_WIDTH)),
                (true, true) => Some((ColorRole::Outline, FOCUSED_OUTLINE_WIDTH)),
                (true, false) => None,
            };
            let (label, error) = match (kind, format) {
                (Field::Hour, HourFormat::Hour12) => ("Hour", "Hour must be 1–12"),
                (Field::Hour, HourFormat::Hour24) => ("Hour", "Hour must be 0–23"),
                (Field::Minute, _) => ("Minute", "Minute must be 0–59"),
            };

            let mut input = HeadlessTextField::new((field_id.clone(), "input"), text.clone())
                .track_focus(&focus_handle.clone().tab_index(0))
                .caret_color(theme.color(ColorRole::Primary))
                .aria_label(label)
                .text_center()
                .w_full();
            if !valid {
                input = input.aria_description(error);
            }
            if let Some(on_change) = on_change.clone() {
                let minute_focus = minute_focus.clone();
                input = input.on_change(move |new: &SharedString, window, cx| {
                    let digits: String = new.chars().filter(char::is_ascii_digit).take(2).collect();
                    // 数字でない文字だけを入れた編集は、描き直して取り消す
                    if state.read(cx).text == digits {
                        window.refresh();
                        return;
                    }
                    let parsed = parse(&digits);
                    let complete = digits.len() == 2;
                    state.update(cx, |state, cx| {
                        state.text = digits;
                        if let Some(parsed) = parsed {
                            state.value = parsed;
                        }
                        cx.notify();
                    });
                    window.refresh();
                    if let Some(parsed) = parsed {
                        let new_time = match kind {
                            Field::Hour => Time::new(parsed, time.minute),
                            Field::Minute => Time::new(time.hour, parsed),
                        };
                        on_change(&new_time, window, cx);
                        if kind == Field::Hour && complete {
                            window.focus(&minute_focus, cx);
                        }
                    }
                });
            }

            let mut font = theme.text_style(tokens::TIME_FIELD_LABEL_TEXT_FONT);
            font.color = Some(theme.color(content));
            let mut field = div()
                .relative()
                .flex()
                .items_center()
                .w(dp(tokens::TIME_FIELD_CONTAINER_WIDTH))
                .h(dp(tokens::TIME_FIELD_CONTAINER_HEIGHT))
                .child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(theme.color(container))
                        .when_some(outline, |this, (role, width)| {
                            this.border(dp(width)).border_color(theme.color(role))
                        }),
                    theme.clone(),
                    tokens::TIME_FIELD_CONTAINER_SHAPE,
                ))
                .child(div().relative().w_full().child(input));
            field.text_style().refine(&font);

            let mut supporting_font = theme.text_style(tokens::TIME_FIELD_SUPPORTING_TEXT_FONT);
            supporting_font.color = Some(theme.color(if valid {
                tokens::TIME_FIELD_SUPPORTING_TEXT_COLOR
            } else {
                ColorRole::Error
            }));
            let line_height = theme.sp(tokens::TIME_FIELD_SUPPORTING_TEXT_FONT.style().line_height);
            let mut supporting = div()
                .w(dp(tokens::TIME_FIELD_CONTAINER_WIDTH))
                .pt(dp(SUPPORTING_TEXT_TOP))
                // Compose は補足の文に 2 行分の高さを取る (`minLines = 2`)
                .min_h(dp(SUPPORTING_TEXT_TOP) + line_height * 2.)
                .child(if valid { label } else { error });
            supporting.text_style().refine(&supporting_font);

            div().flex().flex_col().child(field).child(supporting)
        };

        let hour = field(Field::Hour, window, cx);
        let minute = field(Field::Minute, window, cx);
        let mut separator_font = theme.text_style(tokens::TIME_FIELD_SEPARATOR_FONT);
        separator_font.color = Some(theme.color(tokens::TIME_FIELD_SEPARATOR_COLOR));
        let mut separator = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .w(dp(SEPARATOR_WIDTH))
            .h(dp(tokens::TIME_FIELD_CONTAINER_HEIGHT))
            .child(":");
        separator.text_style().refine(&separator_font);

        let period = (format == HourFormat::Hour12).then(|| {
            let pm = time.hour >= 12;
            let item = |id: &'static str,
                        label: &'static str,
                        value: bool,
                        window: &mut Window,
                        cx: &mut App| {
                let item_id = ElementId::from((self.id.clone(), id));
                let focus_handle = util::keyed_focus_handle(&item_id, window, cx);
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let checked = value == pm;
                let (container, content, shape) = if checked {
                    (
                        ColorRole::PrimaryContainer,
                        ColorRole::OnPrimaryContainer,
                        ShapeKey::CornerMedium,
                    )
                } else {
                    (
                        ColorRole::SurfaceContainerLowest,
                        tokens::PERIOD_SELECTOR_UNSELECTED_LABEL_TEXT_COLOR,
                        ShapeKey::CornerFull,
                    )
                };
                let mut font = theme.text_style(tokens::PERIOD_SELECTOR_LABEL_TEXT_FONT);
                font.color = Some(theme.color(content));
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                let mut radio = Radio::new(item_id, value)
                    .track_focus(&focus_handle)
                    .relative()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(Shaped::new(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .bg(theme.color(container)),
                        theme.clone(),
                        shape,
                    ))
                    .child(div().relative().child(label))
                    .aria_label(label);
                if focus_visible {
                    radio = radio.child(
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
                            shape,
                        )
                        .outset(outset),
                    );
                }
                radio.text_style().refine(&font);
                radio
            };
            let am = item("am", "AM", false, window, cx);
            let pm_item = item("pm", "PM", true, window, cx);
            RadioGroup::new((self.id.clone(), "period"), Some(pm))
                .when_some(on_change.clone(), |this, on_change| {
                    this.on_change(move |pm: &bool, window, cx| {
                        let hour = time.hour % 12 + if *pm { 12 } else { 0 };
                        on_change(&Time::new(hour, time.minute), window, cx);
                    })
                })
                .item(am)
                .item(pm_item)
                .aria_label("Select AM or PM")
                .flex()
                .flex_col()
                .flex_none()
                .gap(dp(PERIOD_TOGGLE_SPACE))
                .ml(dp(PERIOD_TOGGLE_SPACE))
                .w(dp(tokens::PERIOD_SELECTOR_CONTAINER_WIDTH))
                .h(dp(tokens::PERIOD_SELECTOR_CONTAINER_HEIGHT))
        });

        let mut root = div()
            .id(self.id)
            .flex()
            .items_start()
            .child(hour)
            .child(separator)
            .child(minute)
            .children(period);
        root.style().refine(&self.style);
        root
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        Context, KeyDownEvent, KeyUpEvent, Keystroke, Render, TestAppContext, VisualTestContext, px,
    };

    use super::*;

    /// (100, 100) に時刻の入力を置き、時刻と、求められた時刻を持つビュー
    struct InputView {
        time: Time,
        format: HourFormat,
        changes: Rc<RefCell<Vec<Time>>>,
    }

    impl Render for InputView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().size_full().child(
                TimeInput::new("time", self.time)
                    .format(self.format)
                    .on_change(cx.listener(move |this, time: &Time, _, cx| {
                        changes.borrow_mut().push(*time);
                        this.time = *time;
                        cx.notify();
                    }))
                    .absolute()
                    .left(px(100.))
                    .top(px(100.)),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        time: Time,
        format: HourFormat,
    ) -> (gpui::Entity<InputView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            InputView {
                time,
                format,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    /// 押して離す。ラジオボタンはキーを離したときに選ばれる
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

    fn changes(view: &gpui::Entity<InputView>, cx: &mut VisualTestContext) -> Vec<Time> {
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

    #[gpui::test]
    fn 入力欄_区切り_午前午後の切り替えを_compose_と同じ寸法で並べる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        // 入力欄 96x72、区切り 24、切り替えとの間 4
        assert_eq!(
            quads(cx, time_picker::TIME_SELECTOR_UNSELECTED_CONTAINER_COLOR),
            [(100., 100., 96., 72.), (220., 100., 96., 72.)]
        );
        // 切り替えは幅 52、項目の高さは (72 - 4) / 2
        assert_eq!(
            quads(cx, ColorRole::SurfaceContainerLowest),
            [(320., 100., 52., 34.)],
            "選ばれていない午前"
        );
        assert_eq!(
            quads(cx, ColorRole::PrimaryContainer),
            [(320., 138., 52., 34.)],
            "選ばれている午後"
        );
    }

    #[gpui::test]
    fn 範囲内の数で時刻を求め_範囲外ならエラーの色にして求めない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        press(cx, "tab");
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("3");
        assert_eq!(
            changes(&view, cx),
            [Time::new(15, 5)],
            "午後のまま 3 時にする"
        );
        cx.simulate_input("x");
        assert_eq!(changes(&view, cx).len(), 1, "数字でない文字は入れない");
        cx.simulate_keystrokes("backspace");
        assert_eq!(changes(&view, cx).len(), 1, "空では求めない");
        assert_eq!(quads(cx, ColorRole::ErrorContainer).len(), 1);
        cx.simulate_input("13");
        assert_eq!(
            changes(&view, cx),
            [Time::new(15, 5), Time::new(13, 5)],
            "1 は午後 1 時として求め、12 時間制で範囲外の 13 は求めない"
        );
        assert_eq!(quads(cx, ColorRole::ErrorContainer).len(), 1);
    }

    #[gpui::test]
    fn 時に_2_桁の数を入れると分の入力欄へ移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 5), HourFormat::Hour24);
        press(cx, "tab");
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("21");
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("30");
        assert_eq!(
            changes(&view, cx),
            [
                Time::new(2, 5),
                Time::new(21, 5),
                Time::new(21, 3),
                Time::new(21, 30)
            ]
        );
        assert!(
            quads(cx, ColorRole::SurfaceContainerLowest).is_empty(),
            "24 時間制では午前・午後の切り替えを置かない"
        );
    }

    #[gpui::test]
    fn 午前午後の切り替えで時を_12_時間ずらす(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        // 時、分、午後 (選ばれている項目) の順
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "up");
        assert_eq!(changes(&view, cx), [Time::new(2, 5)]);
    }

    #[gpui::test]
    fn 外から時刻が変わると入力欄の文字列を置き換える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        press(cx, "tab");
        cx.simulate_keystrokes("ctrl-a backspace");
        assert_eq!(quads(cx, ColorRole::ErrorContainer).len(), 1);
        view.update(cx, |view, cx| {
            view.time = Time::new(8, 0);
            cx.notify();
        });
        cx.run_until_parked();
        assert!(quads(cx, ColorRole::ErrorContainer).is_empty());
        cx.simulate_keystrokes("end");
        cx.simulate_input("x");
        assert!(changes(&view, cx).is_empty());
    }
}
