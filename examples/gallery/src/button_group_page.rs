//! ボタングループを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
};
use md3_components::{
    Button, ButtonGroup, ButtonGroupItem, ButtonVariant, ConnectedButton, ConnectedToggleButton,
    MultiChoiceConnectedButtonGroup, SingleChoiceConnectedButtonGroup, ToggleButtonVariant,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// ボタングループを並べるページ。最後に押したボタンと、Connected button group の選択を持つ
#[derive(Default)]
pub struct ButtonGroupPage {
    last_clicked: Option<SharedString>,
    /// 1 つだけを選ぶ並びで選ばれている項目の番号
    period: usize,
    /// 複数を選ぶ並びで選ばれている項目
    places: [bool; 4],
}

const PERIODS: [&str; 3] = ["Day", "Week", "Month"];

const PLACES: [(&str, md3_icons::Icon); 4] = [
    ("Work", md3_icons::WORK),
    ("Restaurant", md3_icons::RESTAURANT),
    ("Coffee", md3_icons::COFFEE),
    ("Home", md3_icons::HOME),
];

impl Render for ButtonGroupPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let button = |id: &'static str, text: &'static str| {
            Button::new(id)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.last_clicked = Some(id.into());
                    cx.notify();
                }))
                .child(gpui::text!(text))
        };
        let filled = ButtonGroup::new("filled-group")
            .item(ButtonGroupItem::new(
                "filled-back",
                button("filled-back", "Back"),
            ))
            .item(ButtonGroupItem::new(
                "filled-play",
                button("filled-play", "Play").icon(md3_icons::PLAY_ARROW),
            ))
            .item(ButtonGroupItem::new(
                "filled-forward",
                button("filled-forward", "Forward"),
            ));
        let tonal = ButtonGroup::new("tonal-group")
            .item(ButtonGroupItem::new(
                "tonal-day",
                button("tonal-day", "Day").variant(ButtonVariant::FilledTonal),
            ))
            .item(ButtonGroupItem::new(
                "tonal-week",
                button("tonal-week", "Week").variant(ButtonVariant::FilledTonal),
            ))
            .item(ButtonGroupItem::new(
                "tonal-month",
                button("tonal-month", "Month").variant(ButtonVariant::FilledTonal),
            ))
            .item(
                ButtonGroupItem::new(
                    "tonal-year",
                    button("tonal-year", "Year")
                        .variant(ButtonVariant::FilledTonal)
                        .disabled(true),
                )
                .disabled(true),
            );
        let outlined = ButtonGroup::new("outlined-group")
            .item(ButtonGroupItem::new(
                "outlined-cut",
                button("outlined-cut", "Cut")
                    .variant(ButtonVariant::Outlined)
                    .icon(md3_icons::CONTENT_CUT),
            ))
            .item(ButtonGroupItem::new(
                "outlined-copy",
                button("outlined-copy", "Copy")
                    .variant(ButtonVariant::Outlined)
                    .icon(md3_icons::CONTENT_COPY),
            ))
            .item(ButtonGroupItem::new(
                "outlined-paste",
                button("outlined-paste", "Paste")
                    .variant(ButtonVariant::Outlined)
                    .icon(md3_icons::CONTENT_PASTE),
            ));

        let single = SingleChoiceConnectedButtonGroup::new("connected-single", Some(self.period))
            .aria_label("Period")
            .on_change(cx.listener(|this, period: &usize, _, cx| {
                this.period = *period;
                cx.notify();
            }))
            .items(
                PERIODS
                    .iter()
                    .enumerate()
                    .map(|(index, name)| ConnectedButton::new(*name, index, *name)),
            );
        let multi = MultiChoiceConnectedButtonGroup::new("connected-multi")
            .aria_label("Places")
            .variant(ToggleButtonVariant::Outlined)
            .items(PLACES.iter().enumerate().map(|(index, (name, icon))| {
                ConnectedToggleButton::new(*name, self.places[index], *name)
                    .icon(*icon)
                    .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                        this.places[index] = *checked;
                        cx.notify();
                    }))
            }));

        div()
            .id("button-group-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(label(
                &theme,
                "last-clicked",
                format!(
                    "最後に押したボタン: {}",
                    self.last_clicked.as_deref().unwrap_or("なし")
                ),
            ))
            .child(label(&theme, "filled-label", "Filled"))
            .child(filled)
            .child(label(
                &theme,
                "tonal-label",
                "Filled tonal (Year is disabled)",
            ))
            .child(tonal)
            .child(label(&theme, "outlined-label", "Outlined with icons"))
            .child(outlined)
            .child(label(
                &theme,
                "connected-single-label",
                "Connected, single choice",
            ))
            .child(single)
            .child(label(
                &theme,
                "connected-multi-label",
                "Connected, multiple choice (outlined)",
            ))
            .child(multi)
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id に使うため、ページ内で重ならない値を渡し、テキストが変わっても同じ値を渡すこと
fn label(
    theme: &Theme,
    id: impl Into<gpui::ElementId>,
    text: impl Into<gpui::SharedString>,
) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id.into(), text.into()))
}
