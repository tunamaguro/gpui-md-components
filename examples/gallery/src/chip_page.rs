//! チップを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{Chip, ChipKind, ChipVariant, FilterChip};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 操作を表す Assist chip のラベルとアイコン
const ACTIONS: [(&str, md3_icons::Icon); 3] = [
    ("Add to calendar", md3_icons::EVENT),
    ("Directions", md3_icons::DIRECTIONS),
    ("Share", md3_icons::SHARE),
];

/// 返信の候補を表す Suggestion chip のラベル
const REPLIES: [&str; 3] = ["Sounds good", "See you soon", "Thanks!"];

/// 絞り込みの条件を表す Filter chip のラベル。最後の 1 つはアイコンを持つ
const FILTERS: [&str; 4] = ["Vegetarian", "Vegan", "Gluten-free", "Open now"];

/// チップを並べるページ。Filter chip の選ばれている状態と、最後に押したチップのラベルを持つ
pub struct ChipPage {
    flat_filters: [bool; 4],
    elevated_filters: [bool; 4],
    last_pressed: Option<SharedString>,
}

impl Default for ChipPage {
    fn default() -> Self {
        Self {
            flat_filters: [true, false, false, false],
            elevated_filters: [false, true, false, false],
            last_pressed: None,
        }
    }
}

impl ChipPage {
    fn filters(
        &self,
        id: &'static str,
        variant: ChipVariant,
        pick: fn(&mut Self) -> &mut [bool; 4],
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let selected = match variant {
            ChipVariant::Flat => self.flat_filters,
            ChipVariant::Elevated => self.elevated_filters,
        };
        div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(FILTERS.iter().enumerate().map(|(index, name)| {
                let chip = FilterChip::new((id, index), *name, selected[index])
                    .variant(variant)
                    .on_change(cx.listener(move |this, value: &bool, _, cx| {
                        pick(this)[index] = *value;
                        cx.notify();
                    }));
                if index == FILTERS.len() - 1 {
                    chip.icon(md3_icons::SCHEDULE)
                } else {
                    chip
                }
            }))
            .child(
                FilterChip::new((id, FILTERS.len()), "Delivery", true)
                    .variant(variant)
                    .disabled(true),
            )
            .child(
                FilterChip::new((id, FILTERS.len() + 1), "Pickup", false)
                    .variant(variant)
                    .disabled(true),
            )
    }
}

impl Render for ChipPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let actions = |id: &'static str, variant: ChipVariant, cx: &mut Context<Self>| {
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(ACTIONS.iter().enumerate().map(|(index, (name, icon))| {
                    Chip::new((id, index), *name)
                        .variant(variant)
                        .icon(*icon)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.last_pressed = Some((*name).into());
                            cx.notify();
                        }))
                }))
                .child(
                    Chip::new((id, ACTIONS.len()), "Call")
                        .variant(variant)
                        .icon(md3_icons::CALL)
                        .disabled(true),
                )
        };
        let replies = |id: &'static str, variant: ChipVariant, cx: &mut Context<Self>| {
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(REPLIES.iter().enumerate().map(|(index, name)| {
                    Chip::new((id, index), *name)
                        .kind(ChipKind::Suggestion)
                        .variant(variant)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.last_pressed = Some((*name).into());
                            cx.notify();
                        }))
                }))
                .child(
                    Chip::new((id, REPLIES.len()), "Maybe later")
                        .kind(ChipKind::Suggestion)
                        .variant(variant)
                        .disabled(true),
                )
        };
        let last_pressed = match &self.last_pressed {
            Some(name) => format!("Last pressed: {name}"),
            None => "Last pressed: none".into(),
        };

        div()
            .id("chip-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(label(&theme, "assist", "Assist"))
            .child(actions("assist-flat", ChipVariant::Flat, cx))
            .child(label(&theme, "assist-elevated", "Assist (elevated)"))
            .child(actions("assist-elevated-row", ChipVariant::Elevated, cx))
            .child(label(&theme, "suggestion", "Suggestion"))
            .child(replies("suggestion-flat", ChipVariant::Flat, cx))
            .child(label(
                &theme,
                "suggestion-elevated",
                "Suggestion (elevated)",
            ))
            .child(replies(
                "suggestion-elevated-row",
                ChipVariant::Elevated,
                cx,
            ))
            .child(label(&theme, "last-pressed", last_pressed))
            .child(label(&theme, "filter", "Filter"))
            .child(self.filters(
                "filter-flat",
                ChipVariant::Flat,
                |this| &mut this.flat_filters,
                cx,
            ))
            .child(label(&theme, "filter-elevated", "Filter (elevated)"))
            .child(self.filters(
                "filter-elevated-row",
                ChipVariant::Elevated,
                |this| &mut this.elevated_filters,
                cx,
            ))
            .child(label(&theme, "narrow", "Narrowed to 160px"))
            .child(
                Chip::new("narrow", "A label too long to fit in the chip")
                    .icon(md3_icons::MAP)
                    .w(px(160.)),
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
