//! 文字の拡大で部品が伸びること、100% の寸法が変わらないこと。
//!
//! 大きさは `debug_selector` で測るため、デバッグビルドでだけ走る。
#![cfg(debug_assertions)]

mod support;

use gpui::{AnyElement, Pixels, TestAppContext, div, prelude::*, px};
use gpui_headless::TextFieldState;
use md3_components::IconName;
use md3_components::buttons::IconButton;
use md3_components::containment::{LazyList, ListItem};
use md3_components::inputs::{TextField, TextFieldVariant};
use md3_components::navigation::{
    NavigationItem, NavigationRail, Tab, Tabs, TopAppBar, TopAppBarVariant,
};
use md3_components::theme::{SetMd3ThemeExt, Theme};

const TARGET: &str = "text-scale-target";

fn set_scale(cx: &mut gpui::VisualTestContext, scale: f32) {
    cx.update(|_, cx| {
        let theme = Theme::default().to_builder().text_scale(scale).build();
        cx.set_md3_theme(theme);
    });
    support::draw(cx);
}

fn target(element: impl IntoElement) -> AnyElement {
    div()
        .w(px(360.0))
        .child(div().debug_selector(|| TARGET.into()).child(element))
        .into_any_element()
}

fn height(cx: &mut gpui::VisualTestContext) -> Pixels {
    cx.debug_bounds(TARGET).expect("部品の位置").size.height
}

#[gpui::test]
fn medium_app_bar_keeps_its_height_and_grows_with_the_title(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        target(
            TopAppBar::new("bar", "受信トレイ")
                .variant(TopAppBarVariant::Medium)
                .navigation_icon(IconButton::new("menu", IconName::Menu).aria_label("メニュー")),
        )
    });
    assert_eq!(height(cx), px(112.0));
    set_scale(cx, 2.0);
    assert!(height(cx) > px(112.0), "文字に合わせて伸びる");
}

#[gpui::test]
fn filled_field_height_does_not_change_on_focus(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    let cx = support::open(cx, move |_, _| {
        target(
            TextField::new("name", &state)
                .label("名前")
                .variant(TextFieldVariant::Filled),
        )
    });
    set_scale(cx, 2.0);
    let before = height(cx);
    support::press(cx, "tab");
    support::tab(cx);
    assert_eq!(height(cx), before, "ラベルが浮いても高さは変わらない");
}

#[gpui::test]
fn rail_keeps_its_width_with_a_long_label(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        div()
            .flex()
            .child(
                div().flex_none().debug_selector(|| TARGET.into()).child(
                    NavigationRail::new("rail")
                        .item(NavigationItem::new(IconName::Inbox, "長い名前の行き先の例")),
                ),
            )
            .into_any_element()
    });
    set_scale(cx, 2.0);
    assert_eq!(cx.debug_bounds(TARGET).expect("位置").size.width, px(80.0));
}

#[gpui::test]
fn tab_labels_wrap_instead_of_overlapping(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        target(
            Tabs::new("tabs", 0)
                .tab(Tab::new("長いタブの名前の例"))
                .tab(Tab::new("もう一つの長いタブの名前")),
        )
    });
    let base = height(cx);
    set_scale(cx, 2.0);
    assert!(height(cx) > base, "折り返して伸びる");
}

#[gpui::test]
fn lazy_list_rows_stay_the_same_height(cx: &mut TestAppContext) {
    let scroll = gpui::UniformListScrollHandle::new();
    let cx = support::open(cx, move |_, _| {
        div()
            .w(px(200.0))
            .h(px(400.0))
            .child(
                LazyList::new("list", 2, &scroll, |index, _| {
                    let headline = if index == 0 {
                        "短い".to_owned()
                    } else {
                        "とても長い見出しの例".repeat(8)
                    };
                    (index.to_string().into(), ListItem::new(headline))
                })
                .size_full(),
            )
            .into_any_element()
    });
    set_scale(cx, 2.0);
    let first = cx.debug_bounds("md3-list-item-list-0").expect("1 行目");
    let second = cx.debug_bounds("md3-list-item-list-1").expect("2 行目");
    assert_eq!(first.size.height, second.size.height);
}
