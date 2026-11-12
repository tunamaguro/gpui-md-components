//! Segmented button を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{
    MultiChoiceSegmentedButtonRow, SegmentedButton, SegmentedToggleButton,
    SingleChoiceSegmentedButtonRow,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 期間を 1 つ選ぶ並びの選択肢
#[derive(Clone, Copy, PartialEq)]
enum Period {
    Day,
    Week,
    Month,
    Year,
}

const PERIODS: [(Period, &str); 4] = [
    (Period::Day, "Day"),
    (Period::Week, "Week"),
    (Period::Month, "Month"),
    (Period::Year, "Year"),
];

/// 移動手段を 1 つ選ぶ並びの選択肢と、選ばれていないときのアイコン
const TRAVEL: [(&str, md3_icons::Icon); 3] = [
    ("Walk", md3_icons::DIRECTIONS_WALK),
    ("Transit", md3_icons::TRAIN),
    ("Drive", md3_icons::DIRECTIONS_CAR),
];

const SIZES: [&str; 4] = ["XS", "S", "M", "L"];

/// Segmented button を並べるページ。各並びの選ばれている値を持つ
pub struct SegmentedButtonPage {
    period: Option<Period>,
    travel: Option<usize>,
    sizes: [bool; 4],
}

impl Default for SegmentedButtonPage {
    fn default() -> Self {
        Self {
            period: Some(Period::Week),
            travel: None,
            sizes: [false, true, true, false],
        }
    }
}

impl Render for SegmentedButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let period_row = |id: &'static str| {
            SingleChoiceSegmentedButtonRow::new(id, self.period)
                .aria_label("Period")
                .on_change(cx.listener(|this, period: &Period, _, cx| {
                    this.period = Some(*period);
                    cx.notify();
                }))
                .segments(PERIODS.map(|(period, name)| SegmentedButton::new(name, period, name)))
        };

        div()
            .id("segmented-button-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(label(&theme, "single", "Single choice"))
            .child(period_row("period"))
            .child(label(&theme, "icons", "Single choice with icons"))
            .child(
                SingleChoiceSegmentedButtonRow::new("travel", self.travel)
                    .aria_label("Travel mode")
                    .on_change(cx.listener(|this, travel: &usize, _, cx| {
                        this.travel = Some(*travel);
                        cx.notify();
                    }))
                    .segments(TRAVEL.iter().enumerate().map(|(index, (name, icon))| {
                        SegmentedButton::new(*name, index, *name).icon(*icon)
                    }))
                    .segment(SegmentedButton::new("Fly", TRAVEL.len(), "Fly").disabled(true)),
            )
            .child(label(&theme, "multi", "Multi choice"))
            .child(
                MultiChoiceSegmentedButtonRow::new("sizes")
                    .aria_label("Sizes")
                    .segments(SIZES.iter().enumerate().map(|(index, name)| {
                        SegmentedToggleButton::new(*name, self.sizes[index], *name).on_change(
                            cx.listener(move |this, checked: &bool, _, cx| {
                                this.sizes[index] = *checked;
                                cx.notify();
                            }),
                        )
                    }))
                    .segment(SegmentedToggleButton::new("XL", true, "XL").disabled(true)),
            )
            .child(label(&theme, "wide", "Stretched to 480px"))
            .child(period_row("period-wide").w(px(480.)))
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡すこと
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
