//! 文字の拡大で部品が伸びること、100% の寸法が変わらないこと。
//!
//! 大きさは `debug_selector` で測るため、デバッグビルドでだけ走る。
#![cfg(debug_assertions)]

mod support;

use gpui::{AnyElement, Pixels, TestAppContext, div, prelude::*, px};
use md3_components::IconName;
use md3_components::buttons::IconButton;
use md3_components::containment::{LazyList, ListItem};
use md3_components::inputs::TextFieldState;
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
    let state = cx.update(TextFieldState::new);
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
    assert_eq!(height(cx), before, "ラベルが浮いても高さは変わらない");
}

fn multi_line_field(
    variant: TextFieldVariant,
    label: Option<&'static str>,
    min_lines: usize,
) -> impl Fn(&mut gpui::Window, &mut gpui::Context<support::Harness>) -> AnyElement + 'static {
    let state = std::cell::RefCell::new(None::<TextFieldState>);
    move |_, cx| {
        let field = state
            .borrow_mut()
            .get_or_insert_with(|| TextFieldState::new(cx))
            .clone();
        let mut field = TextField::new("body", &field)
            .variant(variant)
            .multi_line(min_lines, 8);
        if let Some(label) = label {
            field = field.label(label);
        }
        target(field)
    }
}

#[gpui::test]
fn multi_line_field_heights_at_100_percent(cx: &mut TestAppContext) {
    let filled = support::open(
        cx,
        multi_line_field(TextFieldVariant::Filled, Some("本文"), 1),
    );
    assert_eq!(height(filled), px(56.0), "Filled の 1 行は 1 行の欄と同じ");
    let mut app = filled.cx.clone();
    let three = support::open(
        &mut app,
        multi_line_field(TextFieldVariant::Filled, Some("本文"), 3),
    );
    assert_eq!(height(three), px(104.0), "8 + 16 + 72 + 8");
    let mut app = three.cx.clone();
    let bare = support::open(
        &mut app,
        multi_line_field(TextFieldVariant::Filled, None, 3),
    );
    assert_eq!(height(bare), px(104.0), "ラベルなしは上下 16");
    let mut app = bare.cx.clone();
    let outlined = support::open(
        &mut app,
        multi_line_field(TextFieldVariant::Outlined, None, 3),
    );
    assert_eq!(height(outlined), px(104.0), "Outlined は上下 16");
}

#[gpui::test]
fn multi_line_field_height_does_not_change_on_focus(cx: &mut TestAppContext) {
    for variant in TextFieldVariant::ALL {
        let mut app = cx.clone();
        let window = support::open(&mut app, multi_line_field(variant, Some("本文"), 3));
        for scale in [1.0, 2.0] {
            set_scale(window, scale);
            let before = height(window);
            support::press(window, "tab");
            assert_eq!(height(window), before, "入力の行を常に確保する");
            support::press(window, "shift-tab");
        }
    }
}

#[gpui::test]
fn multi_line_field_grows_with_the_text_scale(cx: &mut TestAppContext) {
    let window = support::open(
        cx,
        multi_line_field(TextFieldVariant::Filled, Some("本文"), 3),
    );
    let before = height(window);
    set_scale(window, 2.0);
    assert!(height(window) >= before + px(72.0), "3 行分の文字が伸びる");
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

/// 1 行目を短く、2 行目を長くした LazyList の行の高さを比べる。`uniform_list` は 1 行目の高さを全行に使う。
fn assert_lazy_rows_match(cx: &mut TestAppContext, item: fn(String) -> ListItem) {
    let scroll = gpui::UniformListScrollHandle::new();
    let cx = support::open(cx, move |_, _| {
        div()
            .w(px(200.0))
            .h(px(400.0))
            .child(
                LazyList::new("list", 2, &scroll, move |index, _| {
                    let text = if index == 0 {
                        "短い".to_owned()
                    } else {
                        "とても長い文の例".repeat(8)
                    };
                    (index.to_string().into(), item(text))
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

#[gpui::test]
fn lazy_list_headline_stays_on_one_line(cx: &mut TestAppContext) {
    assert_lazy_rows_match(cx, ListItem::new);
}

#[gpui::test]
fn lazy_list_supporting_text_stays_on_one_line(cx: &mut TestAppContext) {
    assert_lazy_rows_match(cx, |text| ListItem::new("見出し").supporting_text(text));
}

#[gpui::test]
fn lazy_list_overline_stays_on_one_line(cx: &mut TestAppContext) {
    assert_lazy_rows_match(cx, |text| ListItem::new("見出し").overline(text));
}

#[gpui::test]
fn lazy_list_three_line_text_stays_on_one_line(cx: &mut TestAppContext) {
    assert_lazy_rows_match(cx, |text| {
        ListItem::new("見出し")
            .overline("上書き")
            .supporting_text(text)
    });
}
