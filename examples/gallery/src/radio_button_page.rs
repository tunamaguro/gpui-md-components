//! ラジオボタンの状態を並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{RadioButton, RadioButtonIndicator, RadioGroup};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

const STATES: [(bool, &str); 2] = [(false, "Unselected"), (true, "Selected")];

const FRUITS: [&str; 4] = ["りんご", "なし", "すもも", "ぶどう"];

/// ラジオボタンを並べるページ。操作できるグループの選ばれている値を持つ
#[derive(Default)]
pub struct RadioButtonPage {
    fruit: Option<&'static str>,
    size: Option<&'static str>,
}

impl Render for RadioButtonPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let row =
            |name: &'static str, disabled: bool| {
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, name).w_32())
                    .children(STATES.map(|(selected, _)| {
                        RadioButtonIndicator::new(selected).disabled(disabled)
                    }))
            };

        div()
            .id("radio-button-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(row("Enabled", false))
            .child(row("Disabled", true))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(label(&theme, "ラベルなし").w_32())
                    .child(
                        RadioGroup::new("size", self.size)
                            .aria_label("大きさ")
                            .on_change(cx.listener(|this, size: &&'static str, _, cx| {
                                this.size = Some(*size);
                                cx.notify();
                            }))
                            .flex()
                            .items(
                                ["小", "中", "大"]
                                    .map(|size| RadioButton::new(size, size).aria_label(size)),
                            ),
                    ),
            )
            .child(
                RadioGroup::new("fruit", self.fruit)
                    .aria_label("果物")
                    .on_change(cx.listener(|this, fruit: &&'static str, _, cx| {
                        this.fruit = Some(*fruit);
                        cx.notify();
                    }))
                    .flex()
                    .flex_col()
                    .items(FRUITS.map(|fruit| {
                        RadioButton::new(fruit, fruit)
                            // すもも は無効にして、矢印キーで飛ばされることを確かめられるようにする
                            .disabled(fruit == "すもも")
                            .pr_4()
                            .child(label(&theme, fruit))
                    })),
            )
    }
}

/// 支援技術に公開されるテキスト。[`RadioButton`] の子にすると項目の名前になる
///
/// `text` を支援技術のノードの id にも使うため、ページ内で重ならない文字列を渡すこと
fn label(theme: &Theme, text: impl Into<gpui::SharedString>) -> gpui::Div {
    let text = text.into();
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = text.clone(), text))
}
