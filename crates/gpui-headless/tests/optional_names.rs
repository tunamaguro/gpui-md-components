//! 任意の名前と表示文字の上書き。
use gpui::UniformListScrollHandle;
use gpui_headless::{
    AppBar, Card, Chip, LazyList, List, ListItem, Menu, MenuItem, Navigation, NavigationOption,
    RadioGroup, RadioOption, SegmentOption, SegmentedButton, Snackbar, TabOption, Tabs,
};

#[test]
fn containers_and_clickable_cards_allow_optional_names() {
    let scroll = UniformListScrollHandle::new();
    let lazy = || {
        LazyList::new("lazy", 0, &scroll, |_, _| {
            ("key".into(), ListItem::new("項目"), ())
        })
    };
    assert_eq!(List::new("list").a11y_node().label, None);
    assert_eq!(lazy().a11y_node().label, None);
    assert_eq!(Menu::new("menu").a11y_node().label, None);
    assert_eq!(Navigation::new("nav").a11y_node().label, None);
    assert_eq!(
        Card::new()
            .on_click("card", |_, _, _| {})
            .a11y_node()
            .unwrap()
            .label,
        None
    );
    assert_eq!(
        List::new("list")
            .aria_label("一覧")
            .a11y_node()
            .label
            .as_deref(),
        Some("一覧")
    );
    assert_eq!(
        lazy().aria_label("連絡先").a11y_node().label.as_deref(),
        Some("連絡先")
    );
    assert_eq!(
        Menu::new("menu")
            .aria_label("編集")
            .a11y_node()
            .label
            .as_deref(),
        Some("編集")
    );
    assert_eq!(
        Navigation::new("nav")
            .aria_label("行き先")
            .a11y_node()
            .label
            .as_deref(),
        Some("行き先")
    );
    assert_eq!(
        Card::new()
            .aria_label("カード")
            .on_click("card", |_, _, _| {})
            .a11y_node()
            .unwrap()
            .label
            .as_deref(),
        Some("カード")
    );
    assert_eq!(List::new("list").aria_label("").a11y_node().label, None);
}

#[test]
fn item_names_can_override_or_clear_visible_text() {
    assert_eq!(
        ListItem::new("見出し")
            .aria_label("見出し 詳細")
            .a11y_node(0, 1)
            .label
            .as_deref(),
        Some("見出し 詳細")
    );
    assert_eq!(
        Menu::new("menu")
            .item(MenuItem::new("編集").aria_label("編集 メニュー"))
            .item_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("編集 メニュー")
    );
    assert_eq!(
        Navigation::new("nav")
            .item("ホーム", false)
            .item_aria_label(0, "ホーム 画面")
            .item_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("ホーム 画面")
    );
    assert_eq!(
        Tabs::new("tabs", 0)
            .tab("概要", false)
            .tab_aria_label(0, "")
            .tab_a11y_node(0)
            .unwrap()
            .label,
        None
    );
    assert_eq!(
        SegmentedButton::new("segments")
            .segment("中央", false)
            .segment_aria_label(0, "中央揃え")
            .segment_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("中央揃え")
    );
    assert_eq!(
        RadioGroup::new("radio")
            .option_item(RadioOption::new("赤").aria_label("赤色"))
            .option_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("赤色")
    );
    assert_eq!(
        Chip::new("chip", "分類").aria_label("").a11y_node().label,
        None
    );
    assert_eq!(
        AppBar::new("bar", "題名")
            .aria_label("題名 詳細")
            .a11y_node()
            .label
            .as_deref(),
        Some("題名 詳細")
    );
    assert_eq!(
        Snackbar::new("status", "保存")
            .aria_label("")
            .a11y_node()
            .label,
        None
    );
    assert_eq!(
        Tabs::new("tabs", 0)
            .tab_item(TabOption::new("概要", false).aria_label(""))
            .tab_a11y_node(0)
            .unwrap()
            .label,
        None
    );
    assert_eq!(
        SegmentedButton::new("segments")
            .segment_item(SegmentOption::new("中央", false).aria_label("中央揃え"))
            .segment_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("中央揃え")
    );
    assert_eq!(
        Navigation::new("nav")
            .destination(NavigationOption::new("ホーム", false).aria_label("ホーム 画面"))
            .item_a11y_node(0)
            .unwrap()
            .label
            .as_deref(),
        Some("ホーム 画面")
    );
}
