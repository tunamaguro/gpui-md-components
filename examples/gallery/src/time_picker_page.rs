//! タイムピッカーを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use md3_components::{HourFormat, Time, TimePicker};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// タイムピッカーを並べるページ。12 時間制と 24 時間制の時刻を持つ
pub struct TimePickerPage {
    time_12: Time,
    time_24: Time,
}

impl TimePickerPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            time_12: Time::new(14, 30),
            time_24: Time::new(9, 5),
        }
    }
}

impl Render for TimePickerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let format = |time: Time| format!("{:02}:{:02}", time.hour, time.minute);
        let column = || div().flex().flex_col().items_start().gap(px(16.));
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(32.))
            .p_6()
            .child(
                column()
                    .child(label(&theme, "picker-12-label", "12-hour"))
                    .child(
                        TimePicker::new("picker-12", self.time_12).on_change(cx.listener(
                            |this, time: &Time, _, cx| {
                                this.time_12 = *time;
                                cx.notify();
                            },
                        )),
                    )
                    .child(label(
                        &theme,
                        "picker-12-value",
                        format!("Selected: {}", format(self.time_12)),
                    )),
            )
            .child(
                column()
                    .child(label(&theme, "picker-24-label", "24-hour"))
                    .child(
                        TimePicker::new("picker-24", self.time_24)
                            .format(HourFormat::Hour24)
                            .on_change(cx.listener(|this, time: &Time, _, cx| {
                                this.time_24 = *time;
                                cx.notify();
                            })),
                    )
                    .child(label(
                        &theme,
                        "picker-24-value",
                        format!("Selected: {}", format(self.time_24)),
                    )),
            )
    }
}

/// 支援技術に公開されるテキスト
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
