//! ツールチップを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, prelude::*,
};
use md3_components::{IconButton, IconButtonVariant, IconToggleButton, Tooltip, TooltipVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// ツールチップを付けたアイコンボタンを並べるページ。toggle の状態を持つ
#[derive(Default)]
pub struct TooltipPage {
    favorite: bool,
}

impl Render for TooltipPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let favorite = self.favorite;
        let row = || div().flex().flex_wrap().items_center().gap_4();

        div()
            .id("tooltip-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .pt_16()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(label(&theme, "plain", "Plain"))
            .child(
                row()
                    .child(
                        IconButton::new("edit", md3_icons::EDIT, "Edit")
                            .tooltip(Tooltip::new("Edit")),
                    )
                    .child(
                        IconButton::new("share", md3_icons::SHARE, "Share")
                            .tooltip(Tooltip::new("Share with others")),
                    )
                    .child(
                        IconToggleButton::new(
                            "favorite",
                            md3_icons::FAVORITE,
                            "Favorite",
                            favorite,
                        )
                        .variant(IconButtonVariant::FilledTonal)
                        .on_change(cx.listener(|this, value: &bool, _, cx| {
                            this.favorite = *value;
                            cx.notify();
                        }))
                        .tooltip(Tooltip::new(if favorite {
                            "Remove from favorites"
                        } else {
                            "Add to favorites"
                        })),
                    )
                    .child(
                        IconButton::new("delete", md3_icons::DELETE, "Delete")
                            .disabled(true)
                            .tooltip(Tooltip::new("You don't have permission to delete")),
                    )
                    .child(
                        IconButton::new("archive", md3_icons::ARCHIVE, "Archive").tooltip(
                            Tooltip::new(
                                "Move the conversation out of the inbox without deleting it",
                            ),
                        ),
                    ),
            )
            .child(label(&theme, "rich", "Rich"))
            .child(
                row()
                    .child(
                        IconButton::new("info", md3_icons::INFO, "About storage").tooltip(
                            Tooltip::new(
                                "Your storage is shared across mail, photos, and files. \
                                 Delete large attachments to free up space.",
                            )
                            .variant(TooltipVariant::Rich {
                                title: Some(SharedString::from("Storage")),
                            }),
                        ),
                    )
                    .child(
                        IconButton::new("help", md3_icons::HELP, "Help").tooltip(
                            Tooltip::new("Press Escape to close a tooltip without moving focus")
                                .variant(TooltipVariant::Rich { title: None }),
                        ),
                    ),
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
