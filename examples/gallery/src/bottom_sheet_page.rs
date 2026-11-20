//! ボトムシートを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Refineable as _, Render, SharedString, Styled as _,
    Window, div, px,
};
use md3_components::{
    Button, ButtonVariant, IconButton, List, ListItem, ListItemKind, ModalBottomSheet,
};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// シートに並べる操作のラベルとアイコン
const ACTIONS: [(&str, md3_icons::Icon); 4] = [
    ("Share", md3_icons::SHARE),
    ("Get link", md3_icons::LINK),
    ("Edit name", md3_icons::EDIT),
    ("Delete", md3_icons::DELETE),
];

/// ボトムシートを並べるページ。シートを開いているかと、最後に選んだ操作を持つ
pub struct BottomSheetPage {
    open: bool,
    last_action: Option<SharedString>,
}

impl BottomSheetPage {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            open: false,
            last_action: None,
        }
    }
}

impl Render for BottomSheetPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        // シートがウィンドウを覆うと幕を押せないため、シートの上端に閉じるボタンを置く
        let header = div().flex().justify_end().px(px(12.)).pt(px(12.)).child(
            IconButton::new("close-sheet", md3_icons::CLOSE, "Close").on_click(cx.listener(
                |this, _, _, cx| {
                    this.open = false;
                    cx.notify();
                },
            )),
        );
        let actions = List::new("sheet-actions")
            .aria_label("Actions")
            .items(ACTIONS.iter().map(|(label, icon)| {
                let label = *label;
                ListItem::new(label, label)
                    .kind(ListItemKind::Action)
                    .leading_icon(*icon)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open = false;
                        this.last_action = Some(label.into());
                        cx.notify();
                    }))
            }));
        let sheet = ModalBottomSheet::new("sheet", self.open, "File actions")
            .on_dismiss(cx.listener(|this, reason, _, cx| {
                this.open = false;
                this.last_action = Some(format!("Dismissed ({reason:?})").into());
                cx.notify();
            }))
            .pb(px(16.))
            .child(header)
            .child(actions);
        let status = self
            .last_action
            .clone()
            .unwrap_or_else(|| "No action yet".into());

        div()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(16.))
            .child(
                Button::new("open-sheet")
                    .variant(ButtonVariant::Outlined)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open = true;
                        cx.notify();
                    }))
                    .child(gpui::text!("Open bottom sheet")),
            )
            .child(label(&theme, "sheet-status", status))
            .child(sheet)
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
