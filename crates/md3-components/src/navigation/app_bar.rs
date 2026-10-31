//! TopAppBar（Small / CenterAligned / Medium / Large）。

use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::component::{app_bar, app_bar_large, app_bar_medium, app_bar_small};
use md3_theme::md3_tokens::layout;
use md3_theme::{Md3ThemeExt, TypeScale};

use crate::StyledExt;
use crate::buttons::IconButton;
use gpui_headless::AppBar;

/// トップアプリバーの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopAppBarVariant {
    /// 高さ 64dp、タイトルを左に置く。
    #[default]
    Small,
    /// 高さ 64dp、タイトルを中央に置く。
    CenterAligned,
    /// 高さ 112dp、タイトルを下の行に置く（Headline Small）。
    Medium,
    /// 高さ 152dp、タイトルを下の行に置く（Headline Medium）。
    Large,
}

impl TopAppBarVariant {
    fn title_font(self) -> TypeScale {
        match self {
            Self::Small | Self::CenterAligned => app_bar_small::TITLE_FONT,
            Self::Medium => app_bar_medium::TITLE_FONT,
            Self::Large => app_bar_large::TITLE_FONT,
        }
    }

    fn height(self) -> f32 {
        match self {
            Self::Small | Self::CenterAligned => app_bar_small::CONTAINER_HEIGHT,
            Self::Medium => app_bar_medium::CONTAINER_HEIGHT,
            Self::Large => app_bar_large::CONTAINER_HEIGHT,
        }
    }
}

/// トップアプリバー（画面の上端。タイトル、ナビゲーションのアイコン、操作のアイコン）。
///
/// 内容をスクロールしたら [`TopAppBar::scrolled`] で色を変える（`SurfaceContainer`）。
/// 縮むアニメーションはない。
#[derive(IntoElement)]
pub struct TopAppBar {
    id: ElementId,
    title: SharedString,
    aria_label: Option<Option<SharedString>>,
    variant: TopAppBarVariant,
    navigation_icon: Option<IconButton>,
    actions: Vec<IconButton>,
    scrolled: bool,
}

impl TopAppBar {
    /// ID とタイトルから作る。
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            aria_label: None,
            variant: TopAppBarVariant::default(),
            navigation_icon: None,
            actions: Vec::new(),
            scrolled: false,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: TopAppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 先頭のナビゲーションのアイコン（戻る、メニューなど）。
    pub fn navigation_icon(mut self, button: IconButton) -> Self {
        self.navigation_icon = Some(button);
        self
    }

    /// 末尾の操作のアイコンを足す。
    pub fn action(mut self, button: IconButton) -> Self {
        self.actions.push(button);
        self
    }

    /// 下の内容がスクロールされているか（`true` なら `SurfaceContainer` で塗る）。
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }
}

impl RenderOnce for TopAppBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut headless = AppBar::new(self.id, self.title.clone());
        if let Some(label) = self.aria_label {
            headless = headless.aria_label(label.unwrap_or_default());
        }
        let theme = cx.md3();
        let container = if self.scrolled {
            app_bar::ON_SCROLL_CONTAINER_COLOR
        } else {
            app_bar::CONTAINER_COLOR
        };
        let row_height = px(app_bar_small::CONTAINER_HEIGHT);
        let has_navigation = self.navigation_icon.is_some();
        // IconButton と同じ密度適用済みの幅を使い、左右で広い側を両側に確保する。
        let icon_width =
            theme.dense(md3_theme::md3_tokens::component::small_icon_button::CONTAINER_HEIGHT);
        let leading = if has_navigation { icon_width } else { px(0.0) };
        let trailing = icon_width * self.actions.len() as f32
            + px(app_bar::ICON_BUTTON_SPACE) * self.actions.len().saturating_sub(1) as f32;
        let centered_inset = leading.max(trailing) + px(layout::TOP_APP_BAR_HORIZONTAL_SPACE);
        let title = headless
            .build_heading()
            .min_w_0()
            .type_style(&theme.text_style(self.variant.title_font()))
            .text_color(theme.color(app_bar::TITLE_COLOR))
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(self.title);
        let navigation = self.navigation_icon.map(|button| {
            div()
                .flex_none()
                .child(button.content_color(app_bar::LEADING_ICON_COLOR))
        });
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(app_bar::ICON_BUTTON_SPACE))
            .children(
                self.actions
                    .into_iter()
                    .map(|button| button.content_color(app_bar::TRAILING_ICON_COLOR)),
            );
        // ナビゲーションのアイコンが無いとき、タイトルの前を空ける（Compose の `TopAppBarTitleInset`）。
        let title_inset = px(layout::TOP_APP_BAR_TITLE_INSET);

        let top_row = |title: Option<AnyElement>| {
            div()
                .h(row_height)
                .w_full()
                .flex()
                .items_center()
                .px(px(layout::TOP_APP_BAR_HORIZONTAL_SPACE))
                .children(navigation)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .when(!has_navigation, |this| this.pl(title_inset))
                        .children(title),
                )
                .child(actions)
        };

        let bar = headless
            .build()
            .w_full()
            .h(px(self.variant.height()))
            .flex_none()
            .flex()
            .flex_col()
            .bg(theme.color(container));
        match self.variant {
            TopAppBarVariant::Small => bar.child(top_row(Some(title.into_any_element()))),
            TopAppBarVariant::CenterAligned => bar.child(
                div().relative().child(top_row(None)).child(
                    div()
                        .absolute()
                        .inset_0()
                        .px(centered_inset)
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(title.max_w_full()),
                ),
            ),
            TopAppBarVariant::Medium | TopAppBarVariant::Large => {
                let bottom = if self.variant == TopAppBarVariant::Medium {
                    layout::MEDIUM_TOP_APP_BAR_TITLE_BOTTOM_SPACE
                } else {
                    layout::LARGE_TOP_APP_BAR_TITLE_BOTTOM_SPACE
                };
                bar.child(top_row(None)).child(
                    div()
                        .flex_1()
                        .flex()
                        .items_end()
                        .pb(px(bottom))
                        .px(px(layout::TOP_APP_BAR_HORIZONTAL_SPACE) + title_inset)
                        .child(title),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heights_follow_the_variant() {
        assert_eq!(TopAppBarVariant::Small.height(), 64.0);
        assert_eq!(TopAppBarVariant::CenterAligned.height(), 64.0);
        assert_eq!(TopAppBarVariant::Medium.height(), 112.0);
        assert_eq!(TopAppBarVariant::Large.height(), 152.0);
    }
}
