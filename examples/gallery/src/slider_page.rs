//! スライダーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::Slider;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// スライダーを並べるページ。操作できるスライダーの値を持つ
pub struct SliderPage {
    volume: f32,
    brightness: f32,
    temperature: f32,
}

impl Default for SliderPage {
    fn default() -> Self {
        Self {
            volume: 0.4,
            brightness: 60.0,
            temperature: 22.0,
        }
    }
}

impl Render for SliderPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        div()
            .id("slider-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .max_w(px(480.))
            .child(label(
                &theme,
                "continuous",
                format!("Volume: {:.2}", self.volume),
            ))
            .child(
                Slider::new("volume", self.volume)
                    .aria_label("Volume")
                    .on_change(cx.listener(|this, value: &f32, _, cx| {
                        this.volume = *value;
                        cx.notify();
                    })),
            )
            .child(label(
                &theme,
                "discrete",
                format!("Brightness (steps of 10): {:.0}", self.brightness),
            ))
            .child(
                Slider::new("brightness", self.brightness)
                    .range(0.0..=100.0)
                    .steps(9)
                    .aria_label("Brightness")
                    .on_change(cx.listener(|this, value: &f32, _, cx| {
                        this.brightness = *value;
                        cx.notify();
                    })),
            )
            .child(label(
                &theme,
                "range",
                format!("Temperature (16 - 30): {:.0}", self.temperature),
            ))
            .child(
                Slider::new("temperature", self.temperature)
                    .range(16.0..=30.0)
                    .steps(13)
                    .aria_label("Temperature")
                    .on_change(cx.listener(|this, value: &f32, _, cx| {
                        this.temperature = *value;
                        cx.notify();
                    })),
            )
            .child(label(&theme, "disabled", "Disabled"))
            .child(
                Slider::new("disabled", 0.3)
                    .aria_label("Disabled continuous")
                    .disabled(true),
            )
            .child(
                Slider::new("disabled-discrete", 0.5)
                    .steps(3)
                    .aria_label("Disabled discrete")
                    .disabled(true),
            )
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
