//! ナビゲーションの部品のページ。

use gpui::{AnyElement, App, Entity, IntoElement, WeakEntity, Window, div, prelude::*, px};
use gpui_headless::TextFieldState;
use md3_components::buttons::{Fab, IconButton};
use md3_components::containment::{List, ListItem, ListLeading};
use md3_components::inputs::SearchBar;
use md3_components::navigation::{
    NavigationBar, NavigationDrawer, NavigationDrawerKind, NavigationItem, NavigationRail,
    NavigationSuite, NavigationSuiteType, Tab, Tabs, TabsVariant, TopAppBar, TopAppBarVariant,
};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{Md3ThemeExt, TypeScale};
use md3_components::{IconName, Text, WindowSizeClass};

use super::Section;
use crate::{Demo, Gallery};

/// 見本の小見出しと中身を縦に並べる。
fn sample(label: &str, element: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            Text::new(TypeScale::LabelMedium, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        )
        .child(element)
        .into_any_element()
}

/// 枠線で囲んだ見本の入れ物（画面の代わり）。
fn frame(width: f32, height: f32, cx: &App, child: impl IntoElement) -> AnyElement {
    let theme = cx.md3();
    div()
        .w(px(width))
        .h(px(height))
        .flex_none()
        .overflow_hidden()
        .border_1()
        .border_color(theme.color(ColorRole::OutlineVariant))
        .bg(theme.color(ColorRole::Surface))
        .child(child)
        .into_any_element()
}

/// 見本の行き先。
fn destinations() -> [NavigationItem; 4] {
    [
        NavigationItem::new(IconName::Inbox, "受信トレイ"),
        NavigationItem::new(IconName::Send, "送信済み"),
        NavigationItem::new(IconName::Favorite, "お気に入り"),
        NavigationItem::new(IconName::Settings, "設定").disabled(true),
    ]
}

/// 見本の検索の候補。
const PEOPLE: [&str; 8] = [
    "青木 さくら",
    "井上 大輔",
    "上田 美咲",
    "Alice Johnson",
    "Bob Smith",
    "加藤 健",
    "木村 陽菜",
    "Charlie Brown",
];

/// ナビゲーションの部品のページ。
pub fn page(
    demo: &Demo,
    search: &Entity<TextFieldState>,
    view: WeakEntity<Gallery>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let select_nav = |view: &WeakEntity<Gallery>| {
        let view = view.clone();
        move |index: usize, _: &mut Window, cx: &mut App| {
            Gallery::update_demo(&view, cx, |demo| demo.destination = index)
        }
    };

    // トップアプリバー
    let app_bar = |id: &str, variant: TopAppBarVariant, scrolled: bool| {
        TopAppBar::new(id.to_owned(), "受信トレイ")
            .variant(variant)
            .scrolled(scrolled)
            .navigation_icon(
                IconButton::new(format!("{id}-menu"), IconName::Menu).aria_label("メニュー"),
            )
            .action(IconButton::new(format!("{id}-search"), IconName::Search).aria_label("検索"))
            .action(IconButton::new(format!("{id}-more"), IconName::MoreVert).aria_label("その他"))
    };
    let app_bars = Section::new("TopAppBar").child(
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(sample(
                "Small",
                frame(
                    600.0,
                    64.0,
                    cx,
                    app_bar("bar-small", TopAppBarVariant::Small, false),
                ),
            ))
            .child(sample(
                "Small（内容をスクロールした後）",
                frame(
                    600.0,
                    64.0,
                    cx,
                    app_bar("bar-scrolled", TopAppBarVariant::Small, true),
                ),
            ))
            .child(sample(
                "CenterAligned",
                frame(
                    600.0,
                    64.0,
                    cx,
                    app_bar("bar-center", TopAppBarVariant::CenterAligned, false),
                ),
            ))
            .child(sample(
                "CenterAligned（長いタイトル・複数の操作）",
                frame(
                    600.0,
                    64.0,
                    cx,
                    TopAppBar::new(
                        "bar-long",
                        "プロジェクトの共有受信トレイと未読メッセージの一覧・詳細設定",
                    )
                    .variant(TopAppBarVariant::CenterAligned)
                    .navigation_icon(
                        IconButton::new("bar-long-menu", IconName::Menu).aria_label("メニュー"),
                    )
                    .action(IconButton::new("bar-long-search", IconName::Search).aria_label("検索"))
                    .action(IconButton::new("bar-long-add", IconName::Add).aria_label("追加"))
                    .action(
                        IconButton::new("bar-long-more", IconName::MoreVert).aria_label("その他"),
                    ),
                ),
            ))
            .child(sample(
                "Medium",
                frame(
                    600.0,
                    112.0,
                    cx,
                    app_bar("bar-medium", TopAppBarVariant::Medium, false),
                ),
            ))
            .child(sample(
                "Large",
                frame(
                    600.0,
                    152.0,
                    cx,
                    app_bar("bar-large", TopAppBarVariant::Large, false),
                ),
            )),
    );

    // タブ
    let tabs = |id: &str, variant: TabsVariant, icons: bool, selected: usize| {
        let icon = |tab: Tab, icon: IconName| if icons { tab.icon(icon) } else { tab };
        Tabs::new(id.to_owned(), selected)
            .aria_label("表示")
            .variant(variant)
            .tab(icon(Tab::new("写真"), IconName::Image))
            .tab(icon(Tab::new("アルバム"), IconName::Star))
            .tab(icon(Tab::new("共有"), IconName::Share))
            .tab(icon(Tab::new("無効"), IconName::Delete).disabled(true))
    };
    let on_tab = |view: &WeakEntity<Gallery>, secondary: bool| {
        let view = view.clone();
        move |index: usize, _: &mut Window, cx: &mut App| {
            Gallery::update_demo(&view, cx, |demo| {
                if secondary {
                    demo.secondary_tab = index
                } else {
                    demo.tab = index
                }
            })
        }
    };
    let tabs_section = Section::new("Tabs（←→ で選ぶ、Home / End）")
        .child(sample(
            "Primary",
            frame(
                560.0,
                49.0,
                cx,
                tabs("tabs-primary", TabsVariant::Primary, false, demo.tab)
                    .on_change(on_tab(&view, false)),
            ),
        ))
        .child(sample(
            "Primary（アイコン付き）",
            frame(
                560.0,
                73.0,
                cx,
                tabs("tabs-primary-icons", TabsVariant::Primary, true, demo.tab)
                    .on_change(on_tab(&view, false)),
            ),
        ))
        .child(sample(
            "Secondary（アイコン付き）",
            frame(
                560.0,
                49.0,
                cx,
                tabs(
                    "tabs-secondary",
                    TabsVariant::Secondary,
                    true,
                    demo.secondary_tab,
                )
                .on_change(on_tab(&view, true)),
            ),
        ))
        .child(sample(
            "Secondary",
            frame(
                560.0,
                49.0,
                cx,
                tabs(
                    "tabs-Secondary-Enabled",
                    TabsVariant::Secondary,
                    false,
                    demo.secondary_tab,
                )
                .on_change(on_tab(&view, true)),
            ),
        ));

    // ナビゲーションバー・レール・ドロワー
    let bars = Section::new("NavigationBar（←→ でフォーカス、Enter / Space で選ぶ）").child(frame(
        400.0,
        80.0,
        cx,
        NavigationBar::new("nav-bar")
            .aria_label("メイン")
            .items(destinations())
            .selected(Some(demo.destination))
            .on_change(select_nav(&view)),
    ));

    let rails = Section::new("NavigationRail（↑↓ でフォーカス）").child(
        div().flex().gap_6().child(sample(
            "操作できる",
            frame(
                80.0,
                420.0,
                cx,
                NavigationRail::new("nav-rail")
                    .aria_label("メイン")
                    .header(Fab::new("rail-fab", IconName::Edit).aria_label("作成"))
                    .items(destinations())
                    .selected(Some(demo.destination))
                    .on_change(select_nav(&view)),
            ),
        )),
    );

    let drawer_header = || {
        div()
            .px_4()
            .py_4()
            .child(Text::new(TypeScale::TitleSmall, "メール").color(ColorRole::OnSurfaceVariant))
    };
    let drawers = Section::new("NavigationDrawer")
        .child(
            div().flex().child(
                md3_components::buttons::Button::new("open-drawer", "モーダルなドロワーを開く")
                    .variant(md3_components::buttons::ButtonVariant::Outlined)
                    .on_click({
                        let view = view.clone();
                        move |_, _, cx| {
                            Gallery::update_demo(&view, cx, |demo| demo.drawer_open = true)
                        }
                    }),
            ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_6()
                .child(sample(
                    "Standard",
                    frame(
                        360.0,
                        360.0,
                        cx,
                        NavigationDrawer::new("nav-drawer")
                            .aria_label("メイン")
                            .child(drawer_header())
                            .items(destinations())
                            .selected(Some(demo.destination))
                            .on_change(select_nav(&view)),
                    ),
                ))
                .child(sample(
                    "Modal（見本）",
                    frame(
                        360.0,
                        360.0,
                        cx,
                        NavigationDrawer::new("nav-drawer-modal-sample")
                            .aria_label("メイン")
                            .kind(NavigationDrawerKind::Modal)
                            .inline()
                            .child(drawer_header())
                            .items(destinations())
                            .selected(Some(demo.destination))
                            .on_change(select_nav(&view)),
                    ),
                )),
        )
        .when(demo.drawer_open, |this| {
            let view = view.clone();
            let close = view.clone();
            this.child(
                NavigationDrawer::new("nav-drawer-modal")
                    .aria_label("メイン")
                    .kind(NavigationDrawerKind::Modal)
                    .child(drawer_header())
                    .items(destinations())
                    .selected(Some(demo.destination))
                    .on_change(move |index, _, cx| {
                        Gallery::update_demo(&view, cx, |demo| {
                            demo.destination = index;
                            demo.drawer_open = false;
                        })
                    })
                    .on_dismiss(move |_, cx| {
                        Gallery::update_demo(&close, cx, |demo| demo.drawer_open = false)
                    }),
            )
        });

    // ウィンドウの幅で切り替わるナビゲーション
    let suite = |id: &str, layout: Option<NavigationSuiteType>| {
        let mut suite =
            NavigationSuite::new(id.to_owned())
                .aria_label("メイン")
                .items(destinations())
                .selected(Some(demo.destination))
                .on_change(select_nav(&view))
                .child(div().p_4().child(
                    Text::new(TypeScale::BodyLarge, "内容").color(ColorRole::OnSurfaceVariant),
                ));
        if let Some(layout) = layout {
            suite = suite.layout(layout);
        }
        suite
    };
    let class = WindowSizeClass::of(window);
    let suites = Section::new("NavigationSuite（ウィンドウの幅のクラスで切り替わる）")
        .child(Text::new(
            TypeScale::BodyMedium,
            format!(
                "このウィンドウの幅のクラス: {class:?} → {:?}（ウィンドウの幅を変えると切り替わる）",
                NavigationSuiteType::for_size_class(class)
            ),
        ))
        .child(
            div()
                .w_full()
                .max_w(px(1100.0))
                .h(px(300.0))
                .overflow_hidden()
                .border_1()
                .border_color(cx.md3().color(ColorRole::OutlineVariant))
                .child(suite("suite-window", None)),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_4()
                .child(sample(
                    "Compact → Bar",
                    frame(360.0, 300.0, cx, suite("suite-bar", Some(NavigationSuiteType::Bar))),
                ))
                .child(sample(
                    "Medium → Rail",
                    frame(360.0, 300.0, cx, suite("suite-rail", Some(NavigationSuiteType::Rail))),
                ))
                .child(sample(
                    "Expanded 以上 → Drawer",
                    frame(
                        600.0,
                        300.0,
                        cx,
                        suite("suite-drawer", Some(NavigationSuiteType::Drawer)),
                    ),
                )),
        );

    // 検索
    let query = search.read(cx).text().to_lowercase();
    let results = PEOPLE
        .iter()
        .filter(|name| !query.is_empty() && name.to_lowercase().contains(&query))
        .fold(
            List::new("search-results").aria_label("検索結果"),
            |list, name| {
                let initial: String = name.chars().take(1).collect();
                list.item(
                    *name,
                    ListItem::new(*name).leading(ListLeading::Avatar(initial.into())),
                )
            },
        );
    let search_section = Section::new("SearchBar").child(
        div()
            .w(px(480.0))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                SearchBar::new("search", search, "連絡先を検索").trailing(
                    IconButton::new("search-clear", IconName::Close)
                        .aria_label("検索の文字を消す")
                        .on_click({
                            let search = search.clone();
                            move |_, _, cx| search.update(cx, |state, cx| state.set_text("", cx))
                        }),
                ),
            )
            .child(results),
    );

    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(app_bars)
        .child(tabs_section)
        .child(bars)
        .child(rails)
        .child(drawers)
        .child(suites)
        .child(search_section)
}
