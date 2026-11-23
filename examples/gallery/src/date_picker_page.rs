//! デイトピッカーを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use md3_components::{Date, DatePicker};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// ギャラリーで今日として示す日付。実行する日によって表示が変わらないよう固定する
const TODAY: (i32, u8, u8) = (2026, 11, 24);

/// デイトピッカーを並べるページ。選ばれている日付を持つ
pub struct DatePickerPage {
    selected: Option<Date>,
    weekdays: Option<Date>,
}

impl DatePickerPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected: None,
            weekdays: Date::new(2026, 11, 20),
        }
    }
}

impl Render for DatePickerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let today = Date::new(TODAY.0, TODAY.1, TODAY.2).expect("today is a valid date");
        let format = |date: Option<Date>| match date {
            Some(date) => format!(
                "Selected: {}-{:02}-{:02}",
                date.year(),
                date.month(),
                date.day()
            ),
            None => "Selected: none".to_owned(),
        };
        let column = || div().flex().flex_col().items_start().gap(px(16.));
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(32.))
            .p_6()
            .child(
                column()
                    .child(label(&theme, "picker-label", "Date picker"))
                    .child(
                        DatePicker::new("picker", self.selected)
                            .today(today)
                            .on_change(cx.listener(|this, date: &Date, _, cx| {
                                this.selected = Some(*date);
                                cx.notify();
                            })),
                    )
                    .child(label(&theme, "picker-value", format(self.selected))),
            )
            .child(
                column()
                    .child(label(&theme, "weekdays-label", "Weekdays only, 2026"))
                    .child(
                        DatePicker::new("weekdays", self.weekdays)
                            .today(today)
                            .year_range(2026..=2026)
                            .enabled(|date| !matches!(date.weekday(), 0 | 6))
                            .on_change(cx.listener(|this, date: &Date, _, cx| {
                                this.weekdays = Some(*date);
                                cx.notify();
                            })),
                    )
                    .child(label(&theme, "weekdays-value", format(self.weekdays))),
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
