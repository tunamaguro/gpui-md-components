//! ナビゲーションバーを並べるページ

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _, Refineable as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use md3_components::{
    NavigationBar, NavigationBarArrangement, NavigationBarItem, NavigationItemIconPosition,
    TabPanel,
};
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 行き先のラベルとアイコン
const DESTINATIONS: [(&str, md3_icons::Icon); 4] = [
    ("Home", md3_icons::HOME),
    ("Explore", md3_icons::EXPLORE),
    ("Inbox", md3_icons::INBOX),
    ("Profile", md3_icons::PERSON),
];

/// ナビゲーションバーを並べるページ。各バーの選ばれている行き先の位置を持つ
#[derive(Default)]
pub struct NavigationBarPage {
    compact: usize,
    wide: usize,
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
        // 選ばれた行き先の内容。バーは内容を公開しないため、行き先の名前を付けたパネルとして同じ描画で差し替える
        let panel = |id: &'static str, index: usize| {
            let name = DESTINATIONS[index].0;
            TabPanel::new(id)
                .aria_label(name)
                .w(px(412.))
                .p_4()
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
            .child(panel("compact-panel", self.compact))
            .child(
                NavigationBar::new("compact", Some(self.compact))
                    .aria_label("Compact navigation")
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.compact = *index;
                        cx.notify();
                    }))
                    .items(items("compact", NavigationItemIconPosition::Top))
                    .w(px(412.)),
            )
            .child(label(&theme, "centered", "Centered, icon at start"))
            .child(panel("wide-panel", self.wide))
            .child(
                NavigationBar::new("wide", Some(self.wide))
                    .aria_label("Wide navigation")
                    .arrangement(NavigationBarArrangement::Centered)
                    .on_change(cx.listener(|this, index: &usize, _, cx| {
                        this.wide = *index;
                        cx.notify();
                    }))
                    .items(items("wide", NavigationItemIconPosition::Start))
                    .w(px(840.)),
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
