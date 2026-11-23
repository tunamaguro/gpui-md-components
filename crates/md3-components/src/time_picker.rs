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

use std::f32::consts::TAU;
use std::rc::Rc;

use gpui::{
    App, Bounds, ElementId, InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels,
    Refineable, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas,
    div, fill, point, prelude::FluentBuilder, size,
};
use gpui_headless::clock_dial::{ClockDial, ClockDialItem, DialPointer, DialPointerPhase};
use gpui_headless::radio::{Radio, RadioGroup};
use md3_theme::md3_tokens::component::time_picker as tokens;
use md3_theme::md3_tokens::{Dp, focus_ring};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::time::{HourFormat, PeriodHandler, Time, period_toggle};
use crate::util;

/// 時と分の表示の間の区切りの幅 (Compose の `DisplaySeparatorWidth`)
const SEPARATOR_WIDTH: Dp = Dp(24.0);
/// 時刻の表示と文字盤の間の余白 (Compose の `ClockDisplayBottomMargin`)
const CLOCK_DISPLAY_BOTTOM_MARGIN: Dp = Dp(36.0);
/// 文字盤の下の余白 (Compose の `ClockFaceBottomMargin`)
const CLOCK_FACE_BOTTOM_MARGIN: Dp = Dp(24.0);
/// 分の表示と午前・午後の切り替えの間の余白 (Compose の `PeriodTogglePaddingSmall`)
const PERIOD_TOGGLE_SPACE: Dp = Dp(4.0);
/// 文字盤の数字を置く外側と内側の円の半径。文字盤の大きさ 256dp に対する値
/// (Compose の `OuterCircleToSizeRatio`、`InnerCircleToSizeRatio`)
const OUTER_RADIUS: Dp = Dp(101.0);
const INNER_RADIUS: Dp = Dp(69.0);
/// 24 時間制の時で、中心からこの距離より内側を押すと内側の円の時を選ぶ (Compose の `MaxDistance`)
const INNER_CIRCLE_DISTANCE: Dp = Dp(74.0);
/// 文字盤の数字の大きさ (Compose の `MinimumInteractiveSize`)
const NUMBER_SIZE: Dp = Dp(48.0);

type ChangeHandler = Box<dyn Fn(&Time, &mut Window, &mut App)>;

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
/// - 文字盤を押すかドラッグすると、ポインタの向きの時か分を選ぶ。時は 1 時間ごと、押した分は 5 分ごと、
///   ドラッグした分は 1 分ごとに選ぶ。24 時間制の時は、中心から 74dp より内側なら内側の円の 12 時から 23 時を選ぶ。
///   時を選んでボタンを離すと、文字盤を分に切り替える。支援技術の利用中は切り替えない
/// - 文字盤の数字は、矢印キーでフォーカスを移し、Enter・Space と支援技術からのクリック操作で選ぶ。
///   この操作では分に切り替えない
/// - 午前・午後の切り替えは矢印キー・Space・クリックで選び、時を 12 時間ずらすよう求める
///
/// 支援技術には、時と分の表示が role `radio` (名前は Select hour と Select minutes)、
/// 文字盤が role `radiogroup` (名前は Hour と Minute) と、数字ごとの role `radio` として公開される。
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
        // 今の時刻と違う場合だけ求める
        let request = self.on_change.map(|on_change| {
            Rc::new(move |next: Time, window: &mut Window, cx: &mut App| {
                if next != time {
                    on_change(&next, window, cx);
                }
            })
        });

        // 時と分の表示
        let selector = |value: Selection, window: &mut Window, cx: &mut App| {
            let (suffix, label, number) = match value {
                Selection::Hour => (
                    "hour",
                    "Select hour",
                    match format {
                        HourFormat::Hour12 => match time.hour % 12 {
                            0 => 12,
                            hour => hour,
                        },
                        HourFormat::Hour24 => time.hour,
                    },
                ),
                Selection::Minute => ("minute", "Select minutes", time.minute),
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
            .flex()
            .item(hour_selector)
            .child(separator)
            .item(minute_selector);

        let period = (format == HourFormat::Hour12).then(|| {
            let on_change = request.clone().map(|request| {
                Rc::new(move |pm: bool, window: &mut Window, cx: &mut App| {
                    request(
                        Time::new(time.hour % 12 + if pm { 12 } else { 0 }, time.minute),
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
        let number_size = dp(NUMBER_SIZE);
        let mut number_font = theme.text_style(tokens::CLOCK_DIAL_LABEL_TEXT_FONT);
        let items: Vec<_> = numbers
            .iter()
            .enumerate()
            .map(|(ix, &(label, value, inner))| {
                let angle = angle_of(ix % 12);
                let radius = dp(if inner { INNER_RADIUS } else { OUTER_RADIUS });
                let center = dial_size / 2.;
                let x = center + radius * angle.sin() - number_size / 2.;
                let y = center - radius * angle.cos() - number_size / 2.;
                let name = match (current, format) {
                    (Selection::Minute, _) => format!("{label} minutes"),
                    (Selection::Hour, HourFormat::Hour12) => format!("{label} o'clock"),
                    (Selection::Hour, HourFormat::Hour24) => format!("{label} hours"),
                };
                number_font.color = Some(theme.color(if value == selected_value {
                    tokens::CLOCK_DIAL_SELECTED_LABEL_TEXT_COLOR
                } else {
                    tokens::CLOCK_DIAL_UNSELECTED_LABEL_TEXT_COLOR
                }));
                let mut item = ClockDialItem::new(("number", ix), value)
                    .absolute()
                    .left(x)
                    .top(y)
                    .size(number_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .aria_label(name)
                    .child(label.to_string());
                item.text_style().refine(&number_font);
                item
            })
            .collect();

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

        let on_pointer = request.clone().map(|request| {
            let selection = selection.clone();
            let inner_distance = dp(INNER_CIRCLE_DISTANCE);
            move |pointer: &DialPointer, window: &mut Window, cx: &mut App| {
                let hour_step = TAU / 12.;
                match current {
                    Selection::Hour => {
                        if pointer.phase == DialPointerPhase::Up {
                            // Compose は支援技術の利用中には分に切り替えない
                            if !window.is_a11y_active() {
                                selection.update(cx, |selection, cx| {
                                    *selection = Selection::Minute;
                                    cx.notify();
                                });
                            }
                            return;
                        }
                        let ix = (pointer.angle / hour_step).round() as u8 % 12;
                        let hour = match format {
                            HourFormat::Hour12 => ix + if pm { 12 } else { 0 },
                            HourFormat::Hour24 if pointer.distance < inner_distance => ix + 12,
                            HourFormat::Hour24 => ix,
                        };
                        request(Time::new(hour, time.minute), window, cx);
                    }
                    Selection::Minute => {
                        let minute = match pointer.phase {
                            DialPointerPhase::Down => {
                                (pointer.angle / hour_step).round() as u8 % 12 * 5
                            }
                            DialPointerPhase::Move => {
                                (pointer.angle / (TAU / 60.)).round() as u8 % 60
                            }
                            DialPointerPhase::Up => return,
                        };
                        request(Time::new(time.hour, minute), window, cx);
                    }
                }
            }
        });
        let on_select = request.clone().map(|request| {
            move |value: &u8, window: &mut Window, cx: &mut App| {
                let next = match current {
                    Selection::Hour => Time::new(*value, time.minute),
                    Selection::Minute => Time::new(time.hour, *value),
                };
                request(next, window, cx);
            }
        });
        let dial = ClockDial::new((self.id.clone(), "dial"), Some(selected_value))
            .when_some(on_pointer, |this, on_pointer| this.on_pointer(on_pointer))
            .when_some(on_select, |this, on_select| this.on_change(on_select))
            .aria_label(match current {
                Selection::Hour => "Hour",
                Selection::Minute => "Minute",
            })
            .relative()
            .flex_none()
            .size(dial_size)
            .child(Shaped::new(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(theme.color(tokens::CLOCK_DIAL_COLOR)),
                theme.clone(),
                tokens::CLOCK_DIAL_SHAPE,
            ))
            .child(selector_canvas)
            .items(items);

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
    fn 時を押すと時刻を求め_離すと分に切り替え_押した分は_5_分ごとにする(
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
        // 分の文字盤で、6 時より少し右 (29 分の向き) を押す
        let angle = TAU * 29. / 60.;
        mouse(
            cx,
            DialPointerPhase::Down,
            x + 100. * angle.sin(),
            y - 100. * angle.cos(),
        );
        mouse(cx, DialPointerPhase::Up, x, y);
        assert_eq!(
            changes(&view, cx),
            [Time::new(15, 5), Time::new(15, 30)],
            "押した分は 5 分ごとにする"
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
    fn _24_時間制では内側を押すと_12_時から_23_時を選ぶ(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(9, 0), HourFormat::Hour24);
        let (x, y) = dial_center(cx);
        mouse(cx, DialPointerPhase::Down, x - 69., y);
        assert_eq!(
            changes(&view, cx),
            [Time::new(21, 0)],
            "内側の 9 時の位置は 21 時"
        );
        mouse(cx, DialPointerPhase::Move, x - 101., y);
        assert_eq!(changes(&view, cx), [Time::new(21, 0), Time::new(9, 0)]);
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
    fn 午前午後の切り替えで時を_12_時間ずらす(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, Time::new(14, 5), HourFormat::Hour12);
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "up");
        assert_eq!(changes(&view, cx), [Time::new(2, 5)]);
    }
}
