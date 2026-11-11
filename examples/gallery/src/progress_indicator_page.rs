//! 進捗インジケーターを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{
    Button, ButtonVariant, Progress, ProgressIndicator, ProgressIndicatorColors,
    ProgressIndicatorVariant,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 1 回の操作で変える進み具合 (%)
const STEP: i32 = 10;

/// 進捗インジケーターを並べるページ。操作で変える進み具合を持つ
pub struct ProgressIndicatorPage {
    /// 進み具合 (%)。小数の足し引きで 0 や 100 から誤差がずれないよう整数で持つ
    percent: i32,
}

impl Default for ProgressIndicatorPage {
    fn default() -> Self {
        Self { percent: 30 }
    }
}

impl Render for ProgressIndicatorPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let progress = Progress::Determinate(self.percent as f32 / 100.0);
        let row = |name: &'static str| {
            div()
                .flex()
                .items_center()
                .gap_6()
                .child(label(&theme, name).w_32())
        };
        let step = |id: &'static str, text: &'static str, delta: i32, cx: &mut Context<Self>| {
            Button::new(id)
                .variant(ButtonVariant::Outlined)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.percent = (this.percent + delta).clamp(0, 100);
                    cx.notify();
                }))
                .child(gpui::text!(id = id, text))
        };

        div()
            .id("progress-indicator-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(
                row("Progress")
                    .child(step("decrease", "-10%", -STEP, cx))
                    .child(step("increase", "+10%", STEP, cx)),
            )
            .child(
                row("Linear").child(
                    ProgressIndicator::new("linear", progress).aria_label("Linear determinate"),
                ),
            )
            .child(
                row("Linear thick").child(
                    ProgressIndicator::new("linear-thick", progress)
                        .h(px(8.))
                        .aria_label("Linear thick"),
                ),
            )
            .child(
                row("Linear indeterminate").child(
                    ProgressIndicator::new("linear-indeterminate", Progress::Indeterminate)
                        .aria_label("Linear indeterminate"),
                ),
            )
            .child(
                row("Circular").child(
                    ProgressIndicator::new("circular", progress)
                        .variant(ProgressIndicatorVariant::Circular)
                        .aria_label("Circular determinate"),
                ),
            )
            .child(
                row("Circular indeterminate")
                    .child(
                        ProgressIndicator::new("circular-indeterminate", Progress::Indeterminate)
                            .variant(ProgressIndicatorVariant::Circular)
                            .aria_label("Circular indeterminate"),
                    )
                    .child(
                        ProgressIndicator::new("circular-track", Progress::Indeterminate)
                            .variant(ProgressIndicatorVariant::Circular)
                            .colors(ProgressIndicatorColors {
                                indicator: Some(ColorRole::Tertiary),
                                track: Some(ColorRole::TertiaryContainer),
                            })
                            .aria_label("Circular indeterminate with track"),
                    ),
            )
    }
}

/// 支援技術に公開されるテキスト
///
/// `text` を支援技術のノードの id にも使うため、ページ内で重ならない文字列を渡すこと
fn label(theme: &Theme, text: &'static str) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = text, text))
}
