//! ナビゲーションレールを並べるページ

use gpui::{
    Context, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    Refineable as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, px,
};
use md3_components::{IconButton, NavigationRail, NavigationRailItem, TabPanel};
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _, dp};

/// 行き先のラベルとアイコン
const DESTINATIONS: [(&str, md3_icons::Icon); 4] = [
    ("Inbox", md3_icons::INBOX),
    ("Outbox", md3_icons::SEND),
    ("Favorites", md3_icons::FAVORITE),
    ("Trash", md3_icons::DELETE),
];

/// ナビゲーションレールを並べるページ。選ばれている行き先の位置と、展開しているか、パネルのフォーカスを持つ
pub struct NavigationRailPage {
    selected: usize,
    expanded: bool,
    panel: FocusHandle,
}

impl NavigationRailPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            selected: 0,
            expanded: false,
            // パネルの中に Tab で到達できる要素が無いため、パネル自身に Tab で到達できるようにする
            panel: cx.focus_handle().tab_stop(true),
        }
    }
}

impl Render for NavigationRailPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let name = DESTINATIONS[self.selected].0;
        // ヘッダーのメニューボタンでレールを展開し、折りたたむ。折りたたんだ 96dp の幅の中央に置く
        let menu = IconButton::new(
            "rail-menu",
            md3_icons::MENU,
            if self.expanded {
                "Collapse navigation"
            } else {
                "Expand navigation"
            },
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.expanded = !this.expanded;
            cx.notify();
        }))
        .ml(px(24.));
        let rail = NavigationRail::new("rail", Some(self.selected))
            .expanded(self.expanded)
            .aria_label("Mail navigation")
            .header(menu)
            .on_change(cx.listener(|this, index: &usize, _, cx| {
                this.selected = *index;
                cx.notify();
            }))
            .items(
                DESTINATIONS
                    .iter()
                    .enumerate()
                    .map(|(index, (label, icon))| {
                        NavigationRailItem::new(("rail", index), index, *icon, *label)
                            .selected_icon(StyledIcon::new(*icon).filled(true))
                    }),
            );
        // 選ばれた行き先の内容。レールは内容を公開しないため、行き先の名前を付けたパネルとして同じ描画で差し替える。
        // Tab でレールの次にパネルへ移れるよう、レールの後に置く
        let panel = TabPanel::new("rail-panel")
            .aria_label(name)
            .track_focus(&self.panel)
            .flex_1()
            .p_4()
            // Tab でパネルへ移ったときに位置が分かるよう、キーボードのフォーカスでリングを描く
            .border(dp(focus_ring::WIDTH))
            .border_color(gpui::transparent_black())
            .focus_visible(|style| style.border_color(theme.color(focus_ring::COLOR)))
            .child(label(&theme, "rail-content", format!("{name} content")));

        div()
            .id("navigation-rail-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(label(&theme, "rail-caption", "Collapsed and expanded rail"))
            .child(
                div()
                    .flex()
                    .w_full()
                    .h(px(480.))
                    .bg(theme.color(ColorRole::SurfaceContainerLowest))
                    .child(rail)
                    .child(panel),
            )
    }
}

/// 支援技術に公開されるテキスト
///
/// `id` は支援技術のノードの id になるため、ページ内で重ならない文字列を渡すこと
fn label(theme: &Theme, id: &'static str, text: impl Into<SharedString>) -> gpui::Div {
    let mut element = div();
    element
        .text_style()
        .refine(&theme.text_style(TypeScale::BodyMedium));
    element.child(gpui::text!(id = id, text.into()))
}
