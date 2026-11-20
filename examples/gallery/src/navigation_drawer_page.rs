//! ナビゲーションドロワーを並べるページ

use gpui::{
    Context, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    Refineable as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, px,
};
use md3_components::{NavigationDrawer, NavigationDrawerItem, TabPanel};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _, dp};

/// 行き先のラベルとアイコンとバッジ
const DESTINATIONS: [(&str, md3_icons::Icon, Option<&str>); 6] = [
    ("Inbox", md3_icons::INBOX, Some("24")),
    ("Outbox", md3_icons::OUTBOX, None),
    ("Sent", md3_icons::SEND, None),
    ("Favorites", md3_icons::FAVORITE, None),
    ("Archive", md3_icons::ARCHIVE, Some("100+")),
    ("Trash", md3_icons::DELETE, None),
];

/// ナビゲーションドロワーを並べるページ。選ばれている行き先の位置と、パネルのフォーカスを持つ
pub struct NavigationDrawerPage {
    selected: usize,
    panel: FocusHandle,
}

impl NavigationDrawerPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            selected: 0,
            // パネルの中に Tab で到達できる要素が無いため、パネル自身に Tab で到達できるようにする
            panel: cx.focus_handle().tab_stop(true),
        }
    }
}

impl Render for NavigationDrawerPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let name = DESTINATIONS[self.selected].0;
        let mut header = div()
            .px(px(28.))
            .py(px(18.))
            .text_color(theme.color(ColorRole::OnSurfaceVariant));
        header
            .text_style()
            .refine(&theme.text_style(TypeScale::TitleSmall));
        let header = header.child(gpui::text!(id = "drawer-header", "Mail"));
        let drawer = NavigationDrawer::new("drawer", Some(self.selected))
            .aria_label("Mail navigation")
            .header(header)
            .on_change(cx.listener(|this, index: &usize, _, cx| {
                this.selected = *index;
                cx.notify();
            }))
            .items(
                DESTINATIONS
                    .iter()
                    .enumerate()
                    .map(|(index, (label, icon, badge))| {
                        let item =
                            NavigationDrawerItem::new(("drawer", index), index, *label).icon(*icon);
                        match badge {
                            Some(badge) => item.badge(*badge),
                            None => item,
                        }
                    }),
            );
        // 選ばれた行き先の内容。ドロワーは内容を公開しないため、行き先の名前を付けたパネルとして同じ描画で差し替える。
        // Tab でドロワーの次にパネルへ移れるよう、ドロワーの後に置く
        let panel = TabPanel::new("drawer-panel")
            .aria_label(name)
            .track_focus(&self.panel)
            .flex_1()
            .p_4()
            // Tab でパネルへ移ったときに位置が分かるよう、キーボードのフォーカスでリングを描く
            .border(dp(focus_ring::WIDTH))
            .border_color(gpui::transparent_black())
            .focus_visible(|style| style.border_color(theme.color(focus_ring::COLOR)))
            .child(label(&theme, "drawer-content", format!("{name} content")));

        div()
            .id("navigation-drawer-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(label(&theme, "drawer-caption", "Standard drawer"))
            .child(
                div()
                    .flex()
                    .w_full()
                    .h(px(480.))
                    .bg(theme.color(ColorRole::SurfaceContainerLowest))
                    .child(drawer)
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
