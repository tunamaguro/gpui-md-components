//! 時刻の入力を並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use md3_components::{HourFormat, Time, TimeInput};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 時刻の入力を並べるページ。12 時間制と 24 時間制の時刻を持つ
pub struct TimeInputPage {
    time_12: Time,
    time_24: Time,
}

impl TimeInputPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            time_12: Time::new(14, 30),
            time_24: Time::new(9, 5),
        }
    }
}

impl Render for TimeInputPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let format = |time: Time| format!("{:02}:{:02}", time.hour, time.minute);
        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(16.))
            .p_6()
            .child(label(&theme, "time-12-label", "12-hour"))
            .child(
                TimeInput::new("time-12", self.time_12).on_change(cx.listener(
                    |this, time: &Time, _, cx| {
                        this.time_12 = *time;
                        cx.notify();
                    },
                )),
            )
            .child(label(
                &theme,
                "time-12-value",
                format!("Selected: {}", format(self.time_12)),
            ))
            .child(label(&theme, "time-24-label", "24-hour"))
            .child(
                TimeInput::new("time-24", self.time_24)
                    .format(HourFormat::Hour24)
                    .on_change(cx.listener(|this, time: &Time, _, cx| {
                        this.time_24 = *time;
                        cx.notify();
                    })),
            )
            .child(label(
                &theme,
                "time-24-value",
                format!("Selected: {}", format(self.time_24)),
            ))
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
