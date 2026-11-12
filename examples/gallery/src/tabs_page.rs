//! タブを並べるページ

use gpui::{
    Context, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _,
    Refineable as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, px,
};
use md3_components::{Tab, TabPanel, TabRow, TabRowVariant};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{Theme, ThemeExt as _};

/// 旅行の画面を切り替える Primary の並びのラベル
const TRIPS: [&str; 3] = ["Flights", "Hotels", "Explore"];

/// メディアの種類を切り替える、アイコン付きの Primary の並びのラベルとアイコン
const MEDIA: [(&str, md3_icons::Icon); 3] = [
    ("Photos", md3_icons::PHOTO),
    ("Music", md3_icons::MUSIC_NOTE),
    ("Favorites", md3_icons::FAVORITE),
];

/// 商品の情報を切り替える Secondary の並びのラベル
const DETAILS: [&str; 3] = ["Overview", "Specifications", "Reviews"];

/// タブを並べるページ。各並びの選ばれているタブの位置と、パネルのフォーカスを持つ
pub struct TabsPage {
    trip: usize,
    media: usize,
    detail: usize,
    narrow: usize,
    trip_panel: FocusHandle,
    detail_panel: FocusHandle,
}

impl TabsPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            trip: 0,
            media: 1,
            detail: 0,
            narrow: 0,
            trip_panel: cx.focus_handle().tab_stop(true),
            detail_panel: cx.focus_handle().tab_stop(true),
        }
    }
}

impl Render for TabsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        // パネルの中にフォーカスできる要素が無いため、パネル自身を Tab で到達できるようにする
        let panel = |id: &'static str, name: &'static str, focus_handle: &FocusHandle| {
            TabPanel::new(id)
                .aria_label(name)
                .track_focus(focus_handle)
                .p_4()
                .child(label(&theme, id, format!("{name} content")))
        };

        div()
            .id("tabs-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_col()
            .items_start()
            .gap_4()
            .child(label(&theme, "primary", "Primary"))
            .child(
                TabRow::new("trips", Some(self.trip))
                    .aria_label("Trips")
                    .on_change(cx.listener(|this, trip: &usize, _, cx| {
                        this.trip = *trip;
                        cx.notify();
                    }))
                    .tabs(
                        TRIPS
                            .iter()
                            .enumerate()
                            .map(|(index, name)| Tab::new(*name, index, *name)),
                    ),
            )
            .child(panel("trip-panel", TRIPS[self.trip], &self.trip_panel))
            .child(label(&theme, "icons", "Primary with icons"))
            .child(
                TabRow::new("media", Some(self.media))
                    .aria_label("Media")
                    .on_change(cx.listener(|this, media: &usize, _, cx| {
                        this.media = *media;
                        cx.notify();
                    }))
                    .tabs(
                        MEDIA
                            .iter()
                            .enumerate()
                            .map(|(index, (name, icon))| Tab::new(*name, index, *name).icon(*icon)),
                    ),
            )
            .child(label(&theme, "secondary", "Secondary"))
            .child(
                TabRow::new("details", Some(self.detail))
                    .variant(TabRowVariant::Secondary)
                    .aria_label("Details")
                    .on_change(cx.listener(|this, detail: &usize, _, cx| {
                        this.detail = *detail;
                        cx.notify();
                    }))
                    .tabs(
                        DETAILS
                            .iter()
                            .enumerate()
                            .map(|(index, name)| Tab::new(*name, index, *name)),
                    ),
            )
            .child(panel(
                "detail-panel",
                DETAILS[self.detail],
                &self.detail_panel,
            ))
            .child(label(&theme, "narrow", "Narrowed to 240px"))
            .child(
                TabRow::new("narrow-details", Some(self.narrow))
                    .aria_label("Narrow details")
                    .on_change(cx.listener(|this, narrow: &usize, _, cx| {
                        this.narrow = *narrow;
                        cx.notify();
                    }))
                    .tabs(
                        DETAILS
                            .iter()
                            .enumerate()
                            .map(|(index, name)| Tab::new(*name, index, *name)),
                    )
                    .w(px(240.)),
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
