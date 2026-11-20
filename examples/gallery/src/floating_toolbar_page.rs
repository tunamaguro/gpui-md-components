//! フローティングツールバーとドッキングツールバーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{
    DockedToolbar, DockedToolbarArrangement, FloatingToolbar, FloatingToolbarColor,
    FloatingToolbarOrientation, IconButton,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// フローティングツールバーとドッキングツールバーを並べるページ
pub struct FloatingToolbarPage;

/// 編集の操作を並べたツールバー。`id` で各操作の id を分ける
fn edit_toolbar(id: &'static str) -> FloatingToolbar {
    FloatingToolbar::new(id, "Edit")
        .action(IconButton::new("cut", md3_icons::CONTENT_CUT, "Cut"))
        .action(IconButton::new("copy", md3_icons::CONTENT_COPY, "Copy"))
        .action(IconButton::new("paste", md3_icons::CONTENT_PASTE, "Paste").disabled(true))
        .action(IconButton::new("share", md3_icons::SHARE, "Share"))
        .action(IconButton::new("delete", md3_icons::DELETE, "Delete"))
}

impl Render for FloatingToolbarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();

        div()
            .id("floating-toolbar-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(label(&theme, "Standard"))
            .child(edit_toolbar("standard"))
            .child(label(&theme, "Vibrant"))
            .child(edit_toolbar("vibrant").color(FloatingToolbarColor::Vibrant))
            .child(label(&theme, "Vertical"))
            .child(
                FloatingToolbar::new("vertical", "Format")
                    .orientation(FloatingToolbarOrientation::Vertical)
                    .action(IconButton::new("bold", md3_icons::FORMAT_BOLD, "Bold"))
                    .action(IconButton::new(
                        "italic",
                        md3_icons::FORMAT_ITALIC,
                        "Italic",
                    ))
                    .action(IconButton::new(
                        "underline",
                        md3_icons::FORMAT_UNDERLINED,
                        "Underline",
                    )),
            )
            .child(label(&theme, "Docked"))
            .child(
                DockedToolbar::new("docked", "Navigation")
                    .w(px(412.))
                    .action(IconButton::new("back", md3_icons::ARROW_BACK, "Back"))
                    .action(IconButton::new("undo", md3_icons::UNDO, "Undo"))
                    .action(IconButton::new("redo", md3_icons::REDO, "Redo"))
                    .action(IconButton::new("edit", md3_icons::EDIT, "Edit")),
            )
            .child(label(&theme, "Docked (centered)"))
            .child(
                DockedToolbar::new("docked-centered", "Format")
                    .arrangement(DockedToolbarArrangement::Centered)
                    .w(px(412.))
                    .action(IconButton::new("bold", md3_icons::FORMAT_BOLD, "Bold"))
                    .action(IconButton::new(
                        "italic",
                        md3_icons::FORMAT_ITALIC,
                        "Italic",
                    ))
                    .action(IconButton::new(
                        "underline",
                        md3_icons::FORMAT_UNDERLINED,
                        "Underline",
                    )),
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
        .refine(&theme.text_style(TypeScale::TitleSmall));
    element.child(gpui::text!(id = text, text))
}
