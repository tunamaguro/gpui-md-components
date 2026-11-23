// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt
//         (`TimePicker`、`VerticalTimePicker`、`VerticalClockDisplay`、`ClockDisplayNumbers`、`TimeSelector`、
//         `ClockFace`、`drawSelector`、`ClockText`、`onTap`、`moveSelector`、`numberContentDescription`、
//         `OuterCircleToSizeRatio`、`InnerCircleToSizeRatio`、`ClockDisplayBottomMargin`、`ClockFaceBottomMargin`、
//         `DisplaySeparatorWidth`、`MaxDistance`、`MinimumInteractiveSize`、`PeriodTogglePaddingSmall`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_time_picker_*`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のタイムピッカー (時計の文字盤)

use std::cell::Cell;
use std::f32::consts::TAU;
use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels,
    Refineable, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas,
    div, fill, point, prelude::FluentBuilder, size,
};
use gpui_headless::clock_dial::{
    ClockDial, ClockDialItem, DialPointer, DialPointerPhase, DialSlider,
};
use gpui_headless::radio::{Radio, RadioGroup};
use md3_theme::md3_tokens::component::time_picker as tokens;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::time::{
    HourFormat, PERIOD_TOGGLE_SPACE, PeriodHandler, SEPARATOR_WIDTH, Time, period_toggle,
};
use crate::util;

/// 時刻の表示と文字盤の間の余白 (Compose の `ClockDisplayBottomMargin`)
const CLOCK_DISPLAY_BOTTOM_MARGIN: Dp = Dp(36.0);
/// 文字盤の下の余白 (Compose の `ClockFaceBottomMargin`)
const CLOCK_FACE_BOTTOM_MARGIN: Dp = Dp(24.0);
/// 文字盤の数字を置く外側と内側の円の半径。文字盤の大きさ 256dp に対する値
/// (Compose の `OuterCircleToSizeRatio`、`InnerCircleToSizeRatio`)
const OUTER_RADIUS: Dp = Dp(101.0);
const INNER_RADIUS: Dp = Dp(69.0);
/// 24 時間制の時で、中心からこの距離より内側を押すと内側の円の時を選ぶ。
/// 数字の項目の上を押せばその数字を選ぶよう、2 つの円の半径の中間にする
const INNER_CIRCLE_DISTANCE: Dp = Dp((OUTER_RADIUS.0 + INNER_RADIUS.0) / 2.);
/// 押した後にポインタがこの距離を超えて動いたらドラッグとみなす (Android の既定の touch slop)
const DRAG_SLOP: Dp = Dp(8.0);
/// 文字盤の数字の字画が中心から左右に届く距離の、1 桁あたりの目安。BodyLarge (16sp) の数字の幅の半分
const DIGIT_HALF_WIDTH: Dp = Dp(5.0);
/// 文字盤の数字の大きさ (Compose の `MinimumInteractiveSize`)
const NUMBER_SIZE: Dp = Dp(48.0);

type ChangeHandler = Box<dyn Fn(&Time, &mut Window, &mut App)>;

/// 文字盤を押してから離すまでの状態
#[derive(Clone, Copy, Default)]
struct Press {
    /// 押した位置。文字盤の中心から見た右と下への距離
    x: f32,
    y: f32,
    /// 円の内側で押したか。外側で押した操作では選ばない
    inside: bool,
    /// 押した位置からドラッグの距離を超えて動かしたか
    dragged: bool,
}

/// 文字盤で選んでいるもの
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Selection {
    #[default]
    Hour,
    Minute,
}

/// MD3 のタイムピッカー (Compose の `TimePicker` の縦の配置)
///
/// 上に時と分の表示を「:」で区切って並べ、[`HourFormat::Hour12`] ではその右に午前・午後の切り替えを置く。
/// その下に、直径 256dp の時計の文字盤を置く。
/// 時刻は呼び出し側が持ち、[`TimePicker::new`] に毎回渡す。ピッカー自身は時刻を変えず、変えた後の時刻を
/// [`TimePicker::on_change`] で求める
///
/// - 時と分の表示を押すと、文字盤で選ぶものをその表示に切り替える。最初は時を選ぶ
/// - 文字盤をマウスで押して離すか、ドラッグすると、ポインタの向きの時か分を選ぶ。押して離した場合は離したときに選び、
///   文字盤の外で離すと取り消す。押した位置から 8dp を超えて動かすとドラッグになり、動かすたびに選ぶ。
///   時は 1 時間ごと、分は 1 分ごとに選ぶ。
///   24 時間制の時は、中心から 85dp より内側なら内側の円の 12 時から 23 時を選ぶ。
///   ただし選んでいる時の円の内側を押した場合は、選んでいる時の円の数字を選ぶ。円の外側で押した操作では選ばない。
///   時を選んでボタンを離すと、文字盤を分に切り替え、時の数字にあったフォーカスを分の文字盤に移す。
///   支援技術の利用中は切り替えない
/// - 時の文字盤の数字は、矢印キーでフォーカスを移し、Enter・Space と支援技術からのクリック操作で選ぶ。
///   この操作では分に切り替えない
/// - 分の文字盤は、フォーカス中の右と上の矢印キーで 1 分増やし、左と下の矢印キーで 1 分減らす。
///   Page Up と Page Down で 5 分、Home で 0 分、End で 59 分にする。0 分と 59 分の間では戻らない
/// - 午前・午後の切り替えは矢印キー・Space・クリックで選び、時を 12 時間ずらすよう求める
///
/// 支援技術には、時と分の表示が名前 Time の role `radiogroup` の中の role `radio`
/// (名前は Select hour, 02 o'clock のように、選ぶものと表示している値)、
/// 時の文字盤が名前 Hour の role `radiogroup` と数字ごとの role `radio`、
/// 分の文字盤が名前 Minute の role `slider` (値は 0 から 59) として公開される。
/// 文言は英語に固定している
#[derive(IntoElement)]
pub struct TimePicker {
    id: ElementId,
    time: Time,
    format: HourFormat,
    on_change: Option<ChangeHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl TimePicker {
    /// `time` を表示する [`HourFormat::Hour12`] のタイムピッカーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。
    /// `id` が変わるか 1 フレームでも描画されないと、文字盤で選んでいるもの・フォーカス・ドラッグが失われる
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
    /// `handler` には変えた後の時刻を渡す。今の時刻と同じになる操作では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`TimePicker::new`] に渡すこと。
    /// 設定しない場合、文字盤と午前・午後の切り替えは時刻を変えない
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

impl Styled for TimePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// 時計の文字盤で `ix` 番目 (真上が 0、時計回り) の位置の向き
fn angle_of(ix: usize) -> f32 {
    TAU * ix as f32 / 12.
}

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let time = self.time;
        let format = self.format;
        let selection = window.use_keyed_state((self.id.clone(), "selection"), cx, |_, _| {
            Selection::default()
        });
        let current = *selection.read(cx);
        // 最後に求めた時刻。描画を挟まずに続く操作は、この時刻を元に変える
        let last = Rc::new(Cell::new(time));
        // 最後に求めた時刻と違う場合だけ求める
        let request = self.on_change.map(|on_change| {
            let last = last.clone();
            Rc::new(move |next: Time, window: &mut Window, cx: &mut App| {
                if next != last.get() {
                    last.set(next);
                    on_change(&next, window, cx);
                }
            })
        });
        let press = window.use_keyed_state((self.id.clone(), "press"), cx, |_, _| Press::default());

        // 時と分の表示
        let selector = |value: Selection, window: &mut Window, cx: &mut App| {
            let (suffix, number) = match value {
                Selection::Hour => (
                    "hour",
                    match format {
                        HourFormat::Hour12 => match time.hour % 12 {
                            0 => 12,
                            hour => hour,
                        },
                        HourFormat::Hour24 => time.hour,
                    },
                ),
                Selection::Minute => ("minute", time.minute),
            };
            // Compose は表示に選ぶものの説明と値の説明を付ける。
            // 音声操作で見えている文字を指定できるよう、値は表示と同じ 2 桁で書く
            let label = match (value, format) {
                (Selection::Hour, HourFormat::Hour12) => {
                    format!("Select hour, {number:02} o'clock")
                }
                (Selection::Hour, HourFormat::Hour24) => format!("Select hour, {number:02} hours"),
                (Selection::Minute, _) => format!("Select minutes, {number:02} minutes"),
            };
            let item_id = ElementId::from((self.id.clone(), suffix));
            let focus_handle = util::keyed_focus_handle(&item_id, window, cx);
            let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
            let (container, content) = if value == current {
                (
                    tokens::TIME_SELECTOR_SELECTED_CONTAINER_COLOR,
                    tokens::TIME_SELECTOR_SELECTED_LABEL_TEXT_COLOR,
                )
            } else {
                (
                    tokens::TIME_SELECTOR_UNSELECTED_CONTAINER_COLOR,
                    tokens::TIME_SELECTOR_UNSELECTED_LABEL_TEXT_COLOR,
                )
            };
            let mut font = theme.text_style(tokens::TIME_SELECTOR_LABEL_TEXT_FONT);
            font.color = Some(theme.color(content));
            let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
            let mut radio = Radio::new(item_id, value)
                .track_focus(&focus_handle)
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .w(dp(tokens::TIME_SELECTOR_CONTAINER_WIDTH))
                .h(dp(tokens::TIME_SELECTOR_CONTAINER_HEIGHT))
                .child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(theme.color(container)),
                    theme.clone(),
                    tokens::TIME_SELECTOR_CONTAINER_SHAPE,
                ))
                .child(div().relative().child(format!("{number:02}")))
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
                        tokens::TIME_SELECTOR_CONTAINER_SHAPE,
                    )
                    .outset(outset),
                );
            }
            radio.text_style().refine(&font);
            radio
        };
        let hour_selector = selector(Selection::Hour, window, cx);
        let minute_selector = selector(Selection::Minute, window, cx);
        let mut separator_font = theme.text_style(tokens::TIME_SELECTOR_SEPARATOR_FONT);
        separator_font.color = Some(theme.color(tokens::TIME_SELECTOR_SEPARATOR_COLOR));
        let mut separator = div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .w(dp(SEPARATOR_WIDTH))
            .h(dp(tokens::PERIOD_SELECTOR_VERTICAL_CONTAINER_HEIGHT))
            .child(":");
        separator.text_style().refine(&separator_font);
        let display = RadioGroup::new((self.id.clone(), "display"), Some(current))
            .on_change({
                let selection = selection.clone();
                move |value: &Selection, _, cx| {
                    selection.update(cx, |selection, cx| {
                        *selection = *value;
                        cx.notify();
                    });
                }
            })
            .aria_label("Time")
            .flex()
            .item(hour_selector)
            .child(separator)
            .item(minute_selector);

        let period = (format == HourFormat::Hour12).then(|| {
            let on_change = request.clone().map(|request| {
                let last = last.clone();
                Rc::new(move |pm: bool, window: &mut Window, cx: &mut App| {
                    let base = last.get();
                    request(
                        Time::new(base.hour % 12 + if pm { 12 } else { 0 }, base.minute),
                        window,
                        cx,
                    );
                }) as PeriodHandler
            });
            period_toggle(
                &self.id,
                &theme,
                time.hour >= 12,
                request.is_none(),
                on_change,
                window,
                cx,
            )
            .ml(dp(PERIOD_TOGGLE_SPACE))
            .h(dp(tokens::PERIOD_SELECTOR_VERTICAL_CONTAINER_HEIGHT))
        });

        // 文字盤の数字。値は選んだときの時か分
        let pm = time.hour >= 12;
        let numbers: Vec<(u8, u8, bool)> = match (current, format) {
            (Selection::Hour, HourFormat::Hour12) => (0..12u8)
                .map(|ix| {
                    let label = if ix == 0 { 12 } else { ix };
                    (label, ix + if pm { 12 } else { 0 }, false)
                })
                .collect(),
            (Selection::Hour, HourFormat::Hour24) => (0..12u8)
                .map(|ix| (ix, ix, false))
                .chain((0..12u8).map(|ix| (ix + 12, ix + 12, true)))
                .collect(),
            (Selection::Minute, _) => (0..12u8).map(|ix| (ix * 5, ix * 5, false)).collect(),
        };
        let selected_value = match current {
            Selection::Hour => time.hour,
            Selection::Minute => time.minute,
        };
        let dial_size = dp(tokens::CLOCK_DIAL_CONTAINER_SIZE);
        let dial_id = ElementId::from((self.id.clone(), "dial"));
        let number_size = dp(NUMBER_SIZE);
        let number_font = theme.text_style(tokens::CLOCK_DIAL_LABEL_TEXT_FONT);
        let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
        let ring = |shape: ShapeKey| {
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
            .outset(outset)
        };
        // セレクタは選んでいる時か分の向きへ中心から伸ばし、先端に数字と同じ大きさの円を描く
        let selector_angle = match current {
            Selection::Hour => angle_of(usize::from(time.hour % 12)),
            Selection::Minute => TAU * f32::from(time.minute) / 60.,
        };
        let selector_radius = dp(
            if current == Selection::Hour && format == HourFormat::Hour24 && time.hour >= 12 {
                INNER_RADIUS
            } else {
                OUTER_RADIUS
            },
        );
        let handle_radius = dp(tokens::CLOCK_DIAL_SELECTOR_HANDLE_CONTAINER_SIZE) / 2.;
        // 数字の左上の位置と、数字の中心からセレクタの円の中心までの距離
        let place = |ix: usize, inner: bool| {
            let angle = angle_of(ix % 12);
            let radius = dp(if inner { INNER_RADIUS } else { OUTER_RADIUS });
            let center = dial_size / 2.;
            let dx = radius * angle.sin() - selector_radius * selector_angle.sin();
            let dy = radius * angle.cos() - selector_radius * selector_angle.cos();
            (
                center + radius * angle.sin() - number_size / 2.,
                center - radius * angle.cos() - number_size / 2.,
                Pixels::from(f32::from(dx).hypot(f32::from(dy))),
            )
        };
        let font_of = |selected: bool| {
            let mut font = number_font.clone();
            font.color = Some(theme.color(if selected {
                tokens::CLOCK_DIAL_SELECTED_LABEL_TEXT_COLOR
            } else {
                tokens::CLOCK_DIAL_UNSELECTED_LABEL_TEXT_COLOR
            }));
            font
        };
        let selector_color = theme.color(tokens::CLOCK_DIAL_SELECTOR_HANDLE_CONTAINER_COLOR);
        let track_width = dp(tokens::CLOCK_DIAL_SELECTOR_TRACK_CONTAINER_WIDTH);
        let center_size = dp(tokens::CLOCK_DIAL_SELECTOR_CENTER_CONTAINER_SIZE);
        let handle_size = dp(tokens::CLOCK_DIAL_SELECTOR_HANDLE_CONTAINER_SIZE);
        let selector_canvas = canvas(
            |_, _, _| {},
            move |bounds: Bounds<Pixels>, _, window, _| {
                let center = bounds.center();
                let handle = center
                    + point(
                        selector_radius * selector_angle.sin(),
                        -selector_radius * selector_angle.cos(),
                    );
                let mut track = PathBuilder::stroke(track_width);
                track.move_to(center);
                track.line_to(handle);
                if let Ok(path) = track.build() {
                    window.paint_path(path, selector_color);
                }
                let circle = |at: gpui::Point<Pixels>, diameter: Pixels| {
                    fill(
                        Bounds::new(
                            at - point(diameter / 2., diameter / 2.),
                            size(diameter, diameter),
                        ),
                        selector_color,
                    )
                    .corner_radii(diameter / 2.)
                };
                window.paint_quad(circle(center, center_size));
                window.paint_quad(circle(handle, handle_size));
            },
        )
        .absolute()
        .size_full();

        // 分の文字盤と、時の数字のフォーカス
        let minute_focus = util::keyed_focus_handle(&dial_id, window, cx);
        let hour_focus: Rc<[_]> = match current {
            Selection::Hour => (0..numbers.len())
                .map(|ix| {
                    let item_id = ElementId::from((self.id.clone(), format!("hour-{ix}")));
                    util::keyed_focus_handle(&item_id, window, cx)
                })
                .collect(),
            Selection::Minute => Rc::new([]),
        };
        let on_pointer = request.clone().map(|request| {
            let selection = selection.clone();
            let hour_focus = hour_focus.clone();
            let minute_focus = minute_focus.clone();
            let last = last.clone();
            let inner_distance = dp(INNER_CIRCLE_DISTANCE);
            move |pointer: &DialPointer, window: &mut Window, cx: &mut App| {
                let base = last.get();
                let hour_step = TAU / 12.;
                let distance = f32::from(pointer.distance);
                let (x, y) = (
                    distance * pointer.angle.sin(),
                    -distance * pointer.angle.cos(),
                );
                let value = match current {
                    Selection::Hour => {
                        let ix = (pointer.angle / hour_step).round() as u8 % 12;
                        // 画面に描いているセレクタの円の内側か
                        let on_handle = (x - f32::from(selector_radius) * selector_angle.sin())
                            .hypot(y + f32::from(selector_radius) * selector_angle.cos())
                            < f32::from(handle_radius);
                        let hour = match format {
                            HourFormat::Hour12 => ix + if base.hour >= 12 { 12 } else { 0 },
                            // 描画しているセレクタの円の内側なら、ほかの数字と重なる位置でも描いている時にする
                            HourFormat::Hour24 if on_handle => time.hour,
                            HourFormat::Hour24 if pointer.distance < inner_distance => ix + 12,
                            HourFormat::Hour24 => ix,
                        };
                        Time::new(hour, base.minute)
                    }
                    // Compose は押した分を 5 分ごとにするが、ドラッグできなくても任意の分を選べるよう 1 分ごとにする
                    Selection::Minute => {
                        Time::new(base.hour, (pointer.angle / (TAU / 60.)).round() as u8 % 60)
                    }
                };
                let inside = pointer.distance <= dial_size / 2.;
                if pointer.phase == DialPointerPhase::Down {
                    press.update(cx, |press, _| {
                        *press = Press {
                            x,
                            y,
                            inside,
                            dragged: false,
                        }
                    });
                    return;
                }
                let state = *press.read(cx);
                // 文字盤の四隅など、円の外側で押した操作では選ばない
                if !state.inside {
                    return;
                }
                match pointer.phase {
                    DialPointerPhase::Down => {}
                    DialPointerPhase::Move => {
                        if !state.dragged
                            && (x - state.x).hypot(y - state.y) <= f32::from(dp(DRAG_SLOP))
                        {
                            return;
                        }
                        press.update(cx, |press, _| press.dragged = true);
                        request(value, window, cx);
                    }
                    DialPointerPhase::Up => {
                        if !state.dragged {
                            // 文字盤の外で離した場合は取り消す (Compose の onTap と同じ)
                            if !inside {
                                return;
                            }
                            request(value, window, cx);
                        }
                        // Compose は支援技術の利用中には分に切り替えない
                        if current == Selection::Hour && !window.is_a11y_active() {
                            selection.update(cx, |selection, cx| {
                                *selection = Selection::Minute;
                                cx.notify();
                            });
                            // 時の数字は描画されなくなるため、フォーカスを分の文字盤に引き継ぐ
                            if hour_focus.iter().any(|handle| handle.is_focused(window)) {
                                window.focus(&minute_focus, cx);
                            }
                        } else if current == Selection::Hour
                            && hour_focus.iter().any(|handle| handle.is_focused(window))
                        {
                            // 押した位置の数字がフォーカスを取るが、24 時間制では重なった別の数字のことがあるため、
                            // 選んだ時の数字に移す
                            let hour = last.get().hour;
                            let ix = match format {
                                HourFormat::Hour12 => hour % 12,
                                HourFormat::Hour24 => hour,
                            };
                            window.focus(&hour_focus[usize::from(ix)], cx);
                        }
                    }
                }
            }
        });
        let background = Shaped::new(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .bg(theme.color(tokens::CLOCK_DIAL_COLOR)),
            theme.clone(),
            tokens::CLOCK_DIAL_SHAPE,
        );
        let dial = match current {
            // 時は Compose と同じく数字を項目として選ぶ
            Selection::Hour => {
                let items: Vec<_> = numbers
                    .iter()
                    .enumerate()
                    .map(|(ix, &(label, value, inner))| {
                        // 時のセレクタは選んでいる数字の中心にある
                        let (x, y, distance) = place(ix, inner);
                        let font = font_of(distance < handle_radius);
                        let name = match format {
                            HourFormat::Hour12 => format!("{label} o'clock"),
                            HourFormat::Hour24 => format!("{label} hours"),
                        };
                        let item_id = ElementId::from((self.id.clone(), format!("hour-{ix}")));
                        let focus_handle = hour_focus[ix].clone();
                        let focus_visible =
                            gpui_headless::is_focus_visible(&focus_handle, window, cx);
                        let mut item = ClockDialItem::new(item_id, value)
                            .track_focus(&focus_handle)
                            .absolute()
                            .left(x)
                            .top(y)
                            .size(number_size)
                            .flex()
                            .items_center()
                            .justify_center()
                            .aria_label(name)
                            .child(label.to_string())
                            .children(focus_visible.then(|| ring(ShapeKey::CornerFull)));
                        item.text_style().refine(&font);
                        item
                    })
                    .collect();
                let on_select = request.clone().map(|request| {
                    let last = last.clone();
                    move |hour: &u8, window: &mut Window, cx: &mut App| {
                        let base = last.get();
                        // 12 時間制の数字は描画時の午前・午後の値を持つため、描画を挟まずに切り替えた午前・午後に合わせる
                        let hour = match format {
                            HourFormat::Hour12 => hour % 12 + if base.hour >= 12 { 12 } else { 0 },
                            HourFormat::Hour24 => *hour,
                        };
                        request(Time::new(hour, base.minute), window, cx);
                    }
                });
                ClockDial::new(dial_id, Some(selected_value))
                    .when_some(on_pointer, |this, on_pointer| this.on_pointer(on_pointer))
                    .when_some(on_select, |this, on_select| this.on_change(on_select))
                    .aria_label("Hour")
                    .relative()
                    .flex_none()
                    .size(dial_size)
                    .child(background)
                    .child(selector_canvas)
                    .items(items)
                    .into_any_element()
            }
            // 分は 5 分ごとの数字だけでは選べない分があるため、文字盤全体を 1 分ずつ増減するスライダーにする
            Selection::Minute => {
                let labels = numbers.iter().enumerate().map(|(ix, &(label, _, inner))| {
                    // Compose はセレクタの円に重なった字画だけを選ばれた色で描く。
                    // 字画の一部だけを塗り分けられないため、字画が円に重なりうる数字には数字の位置にも同じ円を描き、
                    // 数字全体を選ばれた色で描く
                    let (x, y, distance) = place(ix, inner);
                    let digits = if label < 10 { 1. } else { 2. };
                    let covered = distance < handle_radius + dp(DIGIT_HALF_WIDTH) * digits;
                    let font = font_of(covered);
                    let mut number = div()
                        .absolute()
                        .left(x)
                        .top(y)
                        .size(number_size)
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(covered, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .size(number_size)
                                    .rounded(number_size / 2.)
                                    .bg(selector_color),
                            )
                        })
                        .child(div().relative().child(label.to_string()));
                    number.text_style().refine(&font);
                    number
                });
                let focus_handle = minute_focus;
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let on_change = request.clone().map(|request| {
                    let last = last.clone();
                    move |minute: &f32, window: &mut Window, cx: &mut App| {
                        request(Time::new(last.get().hour, *minute as u8), window, cx);
                    }
                });
                DialSlider::new(dial_id, f32::from(time.minute))
                    .range(0.0..=59.0)
                    .step(1.0)
                    .track_focus(&focus_handle)
                    .when_some(on_pointer, |this, on_pointer| this.on_pointer(on_pointer))
                    .when_some(on_change, |this, on_change| this.on_change(on_change))
                    .aria_label("Minute")
                    .relative()
                    .flex_none()
                    .size(dial_size)
                    .child(background)
                    .child(selector_canvas)
                    .children(labels)
                    .when(focus_visible, |this| {
                        this.child(ring(tokens::CLOCK_DIAL_SHAPE))
                    })
                    .into_any_element()
            }
        };

        let mut root = div()
            .id(self.id)
            .flex()
            .flex_col()
            .items_center()
            .pb(dp(CLOCK_FACE_BOTTOM_MARGIN))
            .child(
                div()
                    .flex()
                    .mb(dp(CLOCK_DISPLAY_BOTTOM_MARGIN))
                    .child(display)
                    .children(period),
            )
            .child(dial);
        root.style().refine(&self.style);
        root
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        Context, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent,
        MouseMoveEvent, MouseUpEvent, Render, TestAppContext, VisualTestContext,
    };

    use gpui::px;
    use md3_theme::md3_color::ColorRole;

    use super::*;

    /// (0, 0) にタイムピッカーを置き、時刻と、求められた時刻を持つビュー
    struct PickerView {
        time: Time,
        format: HourFormat,
        changes: Rc<RefCell<Vec<Time>>>,
    }

    impl Render for PickerView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().size_full().child(
                TimePicker::new("picker", self.time)
                    .format(self.format)
                    .on_change(cx.listener(move |this, time: &Time, _, cx| {
                        changes.borrow_mut().push(*time);
                        this.time = *time;
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
        time: Time,
        format: HourFormat,
    ) -> (gpui::Entity<PickerView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            PickerView {
                time,
                format,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn changes(view: &gpui::Entity<PickerView>, cx: &mut VisualTestContext) -> Vec<Time> {
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

    /// 文字盤の左上と中心。文字盤は色 `CLOCK_DIAL_COLOR` で塗った 256x256 の矩形
    fn dial_center(cx: &mut VisualTestContext) -> (f32, f32) {
        let dials: Vec<_> = quads(cx, tokens::CLOCK_DIAL_COLOR)
            .into_iter()
            .filter(|quad| quad.2 == 256.)
            .collect();
        assert_eq!(dials.len(), 1, "文字盤を 1 つ描くこと");
        (dials[0].0 + 128., dials[0].1 + 128.)
    }

    fn mouse(cx: &mut VisualTestContext, phase: DialPointerPhase, x: f32, y: f32) {
        let position = gpui::point(px(x), px(y));
        match phase {
            DialPointerPhase::Down => cx.simulate_event(MouseDownEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::none(),
                click_count: 1,
                first_mouse: false,
            }),
            DialPointerPhase::Move => cx.simulate_event(MouseMoveEvent {
                position,
                pressed_button: Some(MouseButton::Left),
                modifiers: Modifiers::none(),
            }),
            DialPointerPhase::Up => cx.simulate_event(MouseUpEvent {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::none(),
                click_count: 1,
            }),
        }
        cx.run_until_parked();
    }

    /// 押して離す。ラジオボタンと文字盤の数字はキーを離したときに選ばれる
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

    #[gpui::test]
    fn 時刻の表示と文字盤を_compose_と同じ寸法で並べる(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        // 表示 96 + 24 + 96 + 4 + 52 = 272 を、文字盤 256 と中央でそろえる
        assert_eq!(
            quads(cx, tokens::TIME_SELECTOR_SELECTED_CONTAINER_COLOR)
                .into_iter()
                .filter(|quad| quad.3 == 80.)
                .collect::<Vec<_>>(),
            [(0., 0., 96., 80.)],
            "選んでいる時の表示"
        );
        let (x, y) = dial_center(cx);
        assert_eq!((x, y), (136., 80. + 36. + 128.));
    }

    #[gpui::test]
    fn 時を押して離すと時刻を求めて分に切り替え_押した分は_1_分ごとにする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        let (x, y) = dial_center(cx);
        // 3 時の向き
        mouse(cx, DialPointerPhase::Down, x + 100., y);
        mouse(cx, DialPointerPhase::Up, x + 100., y);
        assert_eq!(
            changes(&view, cx),
            [Time::new(15, 5)],
            "午後のまま 3 時にする"
        );
        // 分の文字盤で、29 分の向きを押す
        let angle = TAU * 29. / 60.;
        let (x29, y29) = (x + 100. * angle.sin(), y - 100. * angle.cos());
        mouse(cx, DialPointerPhase::Down, x29, y29);
        assert_eq!(changes(&view, cx).len(), 1, "押しただけでは求めない");
        mouse(cx, DialPointerPhase::Up, x29, y29);
        assert_eq!(
            changes(&view, cx),
            [Time::new(15, 5), Time::new(15, 29)],
            "ドラッグしなくても 1 分ごとに選ぶ"
        );
    }

    #[gpui::test]
    fn ドラッグした分は_1_分ごとに選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        // 分の表示を押して分に切り替える
        press(cx, "tab");
        press(cx, "right");
        let (x, y) = dial_center(cx);
        let angle = TAU * 17. / 60.;
        mouse(cx, DialPointerPhase::Down, x, y - 100.);
        mouse(
            cx,
            DialPointerPhase::Move,
            x + 100. * angle.sin(),
            y - 100. * angle.cos(),
        );
        mouse(cx, DialPointerPhase::Up, x, y);
        assert_eq!(changes(&view, cx), [Time::new(9, 17)]);
    }

    #[gpui::test]
    fn 描画を挟まずにドラッグを戻しても最後の分を求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(9, 15), HourFormat::Hour24);
        press(cx, "tab");
        press(cx, "right");
        let (x, y) = dial_center(cx);
        let at = |minute: f32| {
            let angle = TAU * minute / 60.;
            gpui::point(px(x + 100. * angle.sin()), px(y - 100. * angle.cos()))
        };
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(MouseDownEvent {
                    position: at(15.),
                    button: MouseButton::Left,
                    modifiers: Modifiers::none(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            for minute in [17., 15.] {
                window.dispatch_event(
                    gpui::PlatformInput::MouseMove(MouseMoveEvent {
                        position: at(minute),
                        pressed_button: Some(MouseButton::Left),
                        modifiers: Modifiers::none(),
                    }),
                    cx,
                );
            }
        });
        assert_eq!(
            changes(&view, cx),
            [Time::new(9, 17), Time::new(9, 15)],
            "最後にドラッグした 15 分に戻すこと"
        );
    }

    #[gpui::test]
    fn _24_時間制では内側を押すと_12_時から_23_時を選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        mouse(cx, DialPointerPhase::Down, x - 69., y);
        mouse(cx, DialPointerPhase::Up, x - 69., y);
        assert_eq!(
            changes(&view, cx),
            [Time::new(21, 0)],
            "内側の 9 時の位置は 21 時"
        );
    }

    #[gpui::test]
    fn _24_時間制では内側の数字の項目の上を押すとその数字を選ぶ(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // 内側の 15 時の中心 (69dp) から外へ 6dp
        mouse(cx, DialPointerPhase::Down, x + 75., y);
        mouse(cx, DialPointerPhase::Up, x + 75., y);
        assert_eq!(changes(&view, cx), [Time::new(15, 0)]);
    }

    #[gpui::test]
    fn _24_時間制では選んでいる時の円の内側を押すとその時を選ぶ(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(15, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // 内側の 15 時の円 (中心 69dp、半径 24dp) の内側で、境界の 85dp より外
        mouse(cx, DialPointerPhase::Down, x + 90., y);
        mouse(cx, DialPointerPhase::Up, x + 90., y);
        assert!(changes(&view, cx).is_empty(), "3 時にしないこと");
    }

    #[gpui::test]
    fn _24_時間制ではセレクタの円の内側かどうかで判定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(15, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // セレクタ (中心 69dp) から 22dp 下で、円の内側。向きは 16 時に近い
        mouse(cx, DialPointerPhase::Down, x + 69., y + 22.);
        mouse(cx, DialPointerPhase::Up, x + 69., y + 22.);
        assert!(changes(&view, cx).is_empty(), "16 時にしないこと");
    }

    #[gpui::test]
    fn _24_時間制では描画を挟まずに続くドラッグも描いているセレクタの円で判定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(15, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        let at = |dx: f32| gpui::point(px(x + dx), px(y));
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(MouseDownEvent {
                    position: at(69.),
                    button: MouseButton::Left,
                    modifiers: Modifiers::none(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            // 外側の 3 時へ動かし、描いている 15 時の円の内側へ戻す
            for dx in [101., 90.] {
                window.dispatch_event(
                    gpui::PlatformInput::MouseMove(MouseMoveEvent {
                        position: at(dx),
                        pressed_button: Some(MouseButton::Left),
                        modifiers: Modifiers::none(),
                    }),
                    cx,
                );
            }
        });
        assert_eq!(changes(&view, cx), [Time::new(3, 0), Time::new(15, 0)]);
    }

    #[gpui::test]
    fn _24_時間制でセレクタの円の外なら境界で判定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(15, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // セレクタから約 30dp で円の外、中心から約 92.6dp で境界より外
        mouse(cx, DialPointerPhase::Down, x + 90., y + 22.);
        mouse(cx, DialPointerPhase::Up, x + 90., y + 22.);
        assert_eq!(changes(&view, cx), [Time::new(3, 0)]);
    }

    #[gpui::test]
    fn 押したまま時から分に切り替えると離しても選ばない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // 外側の 3 時を押し、押したまま時と分の表示に戻って分を選ぶ
        mouse(cx, DialPointerPhase::Down, x + 101., y);
        press(cx, "shift-tab");
        press(cx, "right");
        mouse(cx, DialPointerPhase::Up, x + 101., y);
        assert!(changes(&view, cx).is_empty(), "{:?}", changes(&view, cx));
        // 分の文字盤に切り替わっていて、押し直せば分を選ぶ
        mouse(cx, DialPointerPhase::Down, x, y + 101.);
        mouse(cx, DialPointerPhase::Up, x, y + 101.);
        assert_eq!(changes(&view, cx), [Time::new(9, 30)]);
    }

    #[gpui::test]
    fn 円の外で押したドラッグでは選ばない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        mouse(cx, DialPointerPhase::Down, x + 110., y + 110.);
        mouse(cx, DialPointerPhase::Move, x + 120., y + 110.);
        mouse(cx, DialPointerPhase::Up, x + 120., y + 110.);
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 時を押して分に切り替えると分の文字盤にフォーカスを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(9, 15), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        // 外側の 3 時の数字の上
        mouse(cx, DialPointerPhase::Down, x + 101., y);
        mouse(cx, DialPointerPhase::Up, x + 101., y);
        press(cx, "right");
        assert_eq!(changes(&view, cx), [Time::new(3, 15), Time::new(3, 16)]);
    }

    #[gpui::test]
    fn 文字盤の外で離すと取り消し_分に切り替えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        mouse(cx, DialPointerPhase::Down, x + 100., y);
        mouse(cx, DialPointerPhase::Up, x + 300., y);
        assert!(changes(&view, cx).is_empty());
        // 押した位置から 8dp 以内の動きはドラッグにしないため、外で離すと取り消す
        mouse(cx, DialPointerPhase::Down, x + 100., y);
        mouse(cx, DialPointerPhase::Move, x + 100., y + 5.);
        mouse(cx, DialPointerPhase::Up, x + 300., y);
        assert!(changes(&view, cx).is_empty());
        // 時の文字盤のままなので、3 時の位置は 3 時になる
        mouse(cx, DialPointerPhase::Down, x + 100., y);
        mouse(cx, DialPointerPhase::Up, x + 100., y);
        assert_eq!(changes(&view, cx), [Time::new(3, 0)]);
    }

    #[gpui::test]
    fn 文字盤の数字はキーボードで選び_分に切り替えない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Time::new(3, 0), HourFormat::Hour12);
        // 時の表示、午前・午後 (選ばれている午前)、文字盤 (選ばれている 3 時) の順
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "right");
        // 選んでいる 3 時のまま、フォーカスのある 4 時にリングを描く
        let rings = cx.update(|window, cx| {
            let ring = cx.md3_theme().color(focus_ring::COLOR);
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_color == ring && quad.border_widths.top.0 > 0.)
                .map(|quad| (quad.bounds.size.width.0 / scale).round())
                .collect::<Vec<_>>()
        });
        assert_eq!(rings, [58.], "48dp の数字の外側に描く");
        assert!(changes(&view, cx).is_empty());
        press(cx, "enter");
        assert_eq!(changes(&view, cx), [Time::new(4, 0)]);
        press(cx, "right");
        press(cx, "space");
        assert_eq!(
            changes(&view, cx),
            [Time::new(4, 0), Time::new(5, 0)],
            "時の文字盤のまま次の時を選ぶ"
        );
    }

    #[gpui::test]
    fn 分の文字盤は矢印キーで_1_分ずつ変える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 15), HourFormat::Hour24);
        // 時の表示から分の表示に移って分を選び、文字盤に入る
        press(cx, "tab");
        press(cx, "right");
        press(cx, "tab");
        let rings = cx.update(|window, cx| {
            let ring = cx.md3_theme().color(focus_ring::COLOR);
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_color == ring && quad.border_widths.top.0 > 0.)
                .map(|quad| (quad.bounds.size.width.0 / scale).round())
                .collect::<Vec<_>>()
        });
        assert_eq!(rings, [266.], "文字盤の外側にリングを描く");
        press(cx, "right");
        press(cx, "left");
        press(cx, "left");
        press(cx, "end");
        assert_eq!(
            changes(&view, cx),
            [
                Time::new(9, 16),
                Time::new(9, 15),
                Time::new(9, 14),
                Time::new(9, 59)
            ]
        );
    }

    #[gpui::test]
    fn 分のセレクタと重なる数字の位置にも円を描く(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, Time::new(9, 2), HourFormat::Hour24);
        press(cx, "tab");
        press(cx, "right");
        let (x, y) = dial_center(cx);
        let mut circles: Vec<_> = quads(cx, tokens::CLOCK_DIAL_SELECTOR_HANDLE_CONTAINER_COLOR)
            .into_iter()
            .filter(|quad| quad.2 == 48.)
            .map(|quad| ((quad.0 + 24. - x).round(), (quad.1 + 24. - y).round()))
            .collect();
        circles.sort_by(|a, b| a.0.total_cmp(&b.0));
        // 0 分の数字は 2 分のセレクタと重なり、5 分の数字 (1 桁) は重ならない
        let angle = TAU * 2. / 60.;
        assert_eq!(
            circles,
            [
                (0., -101.),
                ((101. * angle.sin()).round(), (-101. * angle.cos()).round())
            ]
        );
    }

    #[gpui::test]
    fn 午前午後の切り替えで時を_12_時間ずらす(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "up");
        assert_eq!(changes(&view, cx), [Time::new(2, 5)]);
    }
}
