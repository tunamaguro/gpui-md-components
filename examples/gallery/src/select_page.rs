//! セレクトを並べるページ

use gpui::{
    Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div, prelude::*,
};
use md3_components::{Select, SelectOption, SelectVariant};
use md3_theme::ThemeExt as _;
use md3_theme::md3_color::ColorRole;

const FRUITS: [&str; 5] = ["Apple", "Banana", "Cherry", "Durian", "Elderberry"];

/// 種類と状態の違うセレクトを並べるページ。セレクトごとに選ばれている値と開いているかを持つ
#[derive(Default)]
pub struct SelectPage {
    selected: [Option<usize>; 4],
    /// 開いているセレクトの番号
    expanded: Option<usize>,
}

impl SelectPage {
    /// 番号 `ix` のセレクト。果物の名前を項目にし、3 つ目は無効にする
    fn select(&self, ix: usize, cx: &mut Context<Self>) -> Select<usize> {
        Select::new(("select", ix), self.selected[ix], self.expanded == Some(ix))
            .on_change(cx.listener(move |this, value: &usize, _, cx| {
                this.selected[ix] = Some(*value);
                cx.notify();
            }))
            .on_expanded_change(cx.listener(move |this, expanded: &bool, _, cx| {
                this.expanded = expanded.then_some(ix);
                cx.notify();
            }))
            .options(
                FRUITS.iter().enumerate().map(|(value, name)| {
                    SelectOption::new(*name, value, *name).disabled(value == 2)
                }),
            )
            .w(gpui::px(280.))
    }
}

impl Render for SelectPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        div()
            .id("select-page")
            .size_full()
            .overflow_y_scroll()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .p_6()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_6()
            .child(self.select(0, cx).label("Fruit"))
            .child(
                self.select(1, cx)
                    .variant(SelectVariant::Outlined)
                    .label("Fruit")
                    .leading_icon(md3_icons::NUTRITION)
                    .supporting_text("Pick your favorite"),
            )
            .child(
                self.select(2, cx)
                    .label("Fruit")
                    .error(self.selected[2].is_none())
                    .supporting_text("Required"),
            )
            .child(
                self.select(3, cx)
                    .variant(SelectVariant::Outlined)
                    .label("Fruit")
                    .disabled(true),
            )
    }
}
