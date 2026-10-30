//! SearchBar（検索の入力欄。結果の一覧は呼び出し側が描く）。

use gpui::{
    App, ElementId, Entity, Focusable, IntoElement, RenderOnce, Role, SharedString, Window, div,
    prelude::*, px,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{filled_text_field, search_bar as tokens};
use md3_theme::md3_tokens::layout;

use super::text_field::{TextFieldState, TextLine, editable};
use crate::buttons::IconButton;
use crate::{Icon, IconName, StyledExt};
use gpui_headless::{A11yExt, A11yNode};

/// 検索バー（高さ 56dp、丸い `SurfaceContainerHigh` の入力欄）。
///
/// 中身は [`TextFieldState`] に持ち、[`TextFieldEvent`](super::TextFieldEvent) で変更を受け取る。
/// 先頭は検索のアイコン（[`SearchBar::leading`] で戻るボタンなどに置き換えられる）。
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    state: Entity<TextFieldState>,
    placeholder: SharedString,
    leading: Option<IconButton>,
    trailing: Vec<IconButton>,
}

impl SearchBar {
    /// ID、中身、placeholder（読み上げられる名前を兼ねる）から作る。
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<TextFieldState>,
        placeholder: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            placeholder: placeholder.into(),
            leading: None,
            trailing: Vec::new(),
        }
    }

    /// 先頭の検索のアイコンを、押せるアイコン（戻る、メニューなど）に置き換える。
    pub fn leading(mut self, button: IconButton) -> Self {
        self.leading = Some(button);
        self
    }

    /// 末尾に押せるアイコン（消去、音声入力など）を足す。
    pub fn trailing(mut self, button: IconButton) -> Self {
        self.trailing.push(button);
        self
    }

    /// アクセシビリティツリーに出す情報（役割 SearchInput）。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::SearchInput).label(self.placeholder.clone())
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let state = self.state.read(cx);
        let text = state.text().to_owned();
        let focused = state.focus_handle(cx).is_focused(window);
        let theme = cx.md3();
        let line = TextLine {
            state: self.state.clone(),
            placeholder: Some(self.placeholder.clone()),
            placeholder_color: theme.color(tokens::SUPPORTING_TEXT_COLOR),
            caret: theme.color(filled_text_field::CARET_COLOR),
            selection: theme
                .color(ColorRole::Primary)
                .opacity(layout::TEXT_SELECTION_OPACITY),
            focused,
        };
        // Compose はアイコンを 48dp の領域の中央に置き、領域を端から 4dp 寄せる（アイコンは端から 16dp）。
        let icon_slot = || {
            div()
                .flex_none()
                .size(px(layout::MIN_INTERACTIVE_SIZE))
                .flex()
                .items_center()
                .justify_center()
        };
        let leading = match self.leading {
            Some(button) => icon_slot()
                .child(button.content_color(tokens::LEADING_ICON_COLOR))
                .into_any_element(),
            None => icon_slot()
                .child(
                    Icon::new(IconName::Search)
                        .size(px(filled_text_field::LEADING_ICON_SIZE))
                        .color(tokens::LEADING_ICON_COLOR),
                )
                .into_any_element(),
        };
        let field = div()
            .id(self.id.clone())
            .a11y(node)
            .aria_value(SharedString::from(text))
            .aria_placeholder(self.placeholder)
            .h(px(tokens::CONTAINER_HEIGHT))
            .min_w(px(layout::SEARCH_BAR_MIN_WIDTH))
            .max_w(px(layout::SEARCH_BAR_MAX_WIDTH))
            .flex()
            .items_center()
            .px(px(layout::SEARCH_BAR_ICON_OFFSET))
            .corner_radius(theme.corner(tokens::CONTAINER_SHAPE))
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .type_style(&theme.text_style(tokens::INPUT_TEXT_FONT))
            .text_color(theme.color(tokens::INPUT_TEXT_COLOR))
            .child(leading)
            .child(div().flex_1().min_w_0().child(line))
            .children(self.trailing.into_iter().map(|button| {
                icon_slot().child(button.content_color(tokens::TRAILING_ICON_COLOR))
            }));
        editable(field, &self.id, &self.state, None, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_bars_are_search_inputs() {
        let node = A11yNode::new(Role::SearchInput).label("メールを検索");
        assert_eq!(node.role, Role::SearchInput);
    }
}
