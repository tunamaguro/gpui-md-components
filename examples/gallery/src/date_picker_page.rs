//! デイトピッカーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{
    Button, ButtonVariant, Date, DatePicker, DatePickerDialog, DatePickerDisplayMode, DateRange,
    DateRangePicker,
};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// ギャラリーで今日として示す日付。実行する日によって表示が変わらないよう固定する
const TODAY: (i32, u8, u8) = (2026, 11, 24);

/// デイトピッカーを並べるページ。選ばれている日付を持つ
pub struct DatePickerPage {
    selected: Option<Date>,
    weekdays: Option<Date>,
    /// ダイアログで確定した日付
    dialog: Option<Date>,
    /// ダイアログで選んでいる、まだ確定していない日付。ダイアログを開いている間だけ `Some`
    dialog_draft: Option<Option<Date>>,
    /// 最初のピッカーの表示
    mode: DatePickerDisplayMode,
    /// ダイアログのピッカーの表示
    dialog_mode: DatePickerDisplayMode,
    /// 範囲のピッカーで選ばれている範囲
    range: DateRange,
    /// 範囲のピッカーの表示
    range_mode: DatePickerDisplayMode,
}

impl DatePickerPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected: None,
            weekdays: Date::new(2026, 11, 20),
            dialog: None,
            dialog_draft: None,
            mode: DatePickerDisplayMode::Picker,
            dialog_mode: DatePickerDisplayMode::Picker,
            range: DateRange::default(),
            range_mode: DatePickerDisplayMode::Picker,
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
            .id("date-picker-page")
            .size_full()
            .overflow_y_scroll()
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
                            .on_change(cx.listener(|this, date: &Option<Date>, _, cx| {
                                this.selected = *date;
                                cx.notify();
                            }))
                            .display_mode(self.mode)
                            .on_display_mode_change(cx.listener(
                                |this, mode: &DatePickerDisplayMode, _, cx| {
                                    this.mode = *mode;
                                    cx.notify();
                                },
                            )),
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
                            .on_change(cx.listener(|this, date: &Option<Date>, _, cx| {
                                this.weekdays = *date;
                                cx.notify();
                            })),
                    )
                    .child(label(&theme, "weekdays-value", format(self.weekdays))),
            )
            .child(
                column()
                    .child(label(&theme, "dialog-label", "Date picker dialog"))
                    .child(
                        Button::new("open-dialog")
                            .variant(ButtonVariant::Outlined)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dialog_draft = Some(this.dialog);
                                cx.notify();
                            }))
                            .child(gpui::text!("Open date picker")),
                    )
                    .child(label(&theme, "dialog-value", format(self.dialog))),
            )
            .child(
                column()
                    .child(label(&theme, "range-label", "Date range picker"))
                    .child(
                        DateRangePicker::new("range", self.range)
                            .today(today)
                            .on_change(cx.listener(|this, range: &DateRange, _, cx| {
                                this.range = *range;
                                cx.notify();
                            }))
                            .display_mode(self.range_mode)
                            .on_display_mode_change(cx.listener(
                                |this, mode: &DatePickerDisplayMode, _, cx| {
                                    this.range_mode = *mode;
                                    cx.notify();
                                },
                            ))
                            .h(px(520.)),
                    )
                    .child(label(
                        &theme,
                        "range-value",
                        format!("{} / {}", format(self.range.start), format(self.range.end)),
                    )),
            )
            .child(
                DatePickerDialog::new("dialog", self.dialog_draft.is_some())
                    .on_dismiss(cx.listener(|this, _, _, cx| {
                        this.dialog_draft = None;
                        cx.notify();
                    }))
                    .confirm_button(
                        Button::new("dialog-confirm")
                            .variant(ButtonVariant::Text)
                            // 日付が選ばれていない間は確定させない
                            .disabled(self.dialog_draft.flatten().is_none())
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(draft) = this.dialog_draft.take() {
                                    this.dialog = draft;
                                }
                                cx.notify();
                            }))
                            .child(gpui::text!("OK")),
                    )
                    .dismiss_button(
                        Button::new("dialog-dismiss")
                            .variant(ButtonVariant::Text)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dialog_draft = None;
                                cx.notify();
                            }))
                            .child(gpui::text!("Cancel")),
                    )
                    .child(
                        DatePicker::new("dialog-picker", self.dialog_draft.flatten())
                            .today(today)
                            // ダイアログの角丸の外まで塗らないよう、背景はダイアログの容器に任せる
                            .bg(gpui::transparent_black())
                            .on_change(cx.listener(|this, date: &Option<Date>, _, cx| {
                                this.dialog_draft = Some(*date);
                                cx.notify();
                            }))
                            .display_mode(self.dialog_mode)
                            .on_display_mode_change(cx.listener(
                                |this, mode: &DatePickerDisplayMode, _, cx| {
                                    this.dialog_mode = *mode;
                                    cx.notify();
                                },
                            )),
                    ),
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
