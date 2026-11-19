//! ナビゲーションバーを並べるページ

use gpui::{
    Context, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    Refineable as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, px,
};
use md3_components::{
    NavigationBar, NavigationBarArrangement, NavigationBarItem, NavigationItemIconPosition,
    TabPanel,
};
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _, dp};

/// 行き先のラベルとアイコン
const DESTINATIONS: [(&str, md3_icons::Icon); 4] = [
    ("Home", md3_icons::HOME),
    ("Explore", md3_icons::EXPLORE),
    ("Inbox", md3_icons::INBOX),
    ("Profile", md3_icons::PERSON),
];

/// ナビゲーションバーを並べるページ。各バーの選ばれている行き先の位置と、パネルのフォーカスを持つ
pub struct NavigationBarPage {
    compact: usize,
    wide: usize,
    compact_panel: FocusHandle,
    wide_panel: FocusHandle,
}

impl NavigationBarPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            compact: 0,
            wide: 0,
            // パネルの中に Tab で到達できる要素が無いため、パネル自身に Tab で到達できるようにする
            compact_panel: cx.focus_handle().tab_stop(true),
            wide_panel: cx.focus_handle().tab_stop(true),
        }
    }
}

impl Render for NavigationBarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let items = |prefix: &'static str, position: NavigationItemIconPosition| {
            DESTINATIONS
                .iter()
                .enumerate()
                .map(move |(index, (label, icon))| {
                    NavigationBarItem::new((prefix, index), index, *icon, *label)
                        .selected_icon(StyledIcon::new(*icon).filled(true))
                        .icon_position(position)
                        // 無効な項目の見た目を示す
                        .disabled(prefix == "compact" && index == 3)
                })
        };
        // 選ばれた行き先の内容。バーは内容を公開しないため、行き先の名前を付けたパネルとして同じ描画で差し替える。
        // Tab でバーの次にパネルへ移れるよう、バーの後に置く
        let ring_color = theme.color(focus_ring::COLOR);
        let panel = |id: &'static str, index: usize, focus_handle: &FocusHandle| {
            let name = DESTINATIONS[index].0;
            TabPanel::new(id)
                .aria_label(name)
                .track_focus(focus_handle)
                .w(px(412.))
                .max_w_full()
                .p_4()
                // Tab でパネルへ移ったときに位置が分かるよう、キーボードのフォーカスでリングを描く
                .border(dp(focus_ring::WIDTH))
                .border_color(gpui::transparent_black())
                .focus_visible(|style| style.border_color(ring_color))
                .child(label(&theme, id, format!("{name} content")))
        };

        div()
            .id("navigation-bar-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_6()
            .child(label(&theme, "equal-weight", "Equal weight, icon on top"))
            .child(
                NavigationBar::new("compact", Some(self.compact))
                    .aria_label("Compact navigation")
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.compact = *index;
                        cx.notify();
                    }))
                    .items(items("compact", NavigationItemIconPosition::Top))
                    .w(px(412.))
                    .max_w_full(),
            )
            .child(panel("compact-panel", self.compact, &self.compact_panel))
            .child(label(&theme, "centered", "Centered, icon at start"))
            .child(
                NavigationBar::new("wide", Some(self.wide))
                    .aria_label("Wide navigation")
                    .arrangement(NavigationBarArrangement::Centered)
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.wide = *index;
                        cx.notify();
                    }))
                    .items(items("wide", NavigationItemIconPosition::Start))
                    .w(px(840.))
                    .max_w_full(),
            )
            .child(panel("wide-panel", self.wide, &self.wide_panel))
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
