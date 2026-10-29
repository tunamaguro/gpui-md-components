//! スクロールバーの比較。操作状態とフェードはポインタで確認する。
//! 隣り合う見本は設定を 1 つだけ変える。
use super::Section;
use gpui::{App, Axis, ElementId, IntoElement, ScrollHandle, Window, div, prelude::*, px};
use md3_components::theme::TypeScale;
use md3_components::{NestedScroll, Scrollbar, ScrollbarInteraction, Text};

pub fn page(window: &mut Window, cx: &mut App) -> impl IntoElement {
    let mut samples = div().flex().flex_wrap().gap_4();
    for (id, label, axis, interaction, fade, count) in [
        (
            "scroll-drag",
            "縦：ドラッグ・ホバー・400ms 後フェード",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            true,
            30,
        ),
        (
            "scroll-always",
            "縦：ドラッグ・常時表示",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            false,
            30,
        ),
        (
            "scroll-passive",
            "縦：表示のみ（つかめない）・常時表示",
            Axis::Vertical,
            ScrollbarInteraction::Passive,
            false,
            30,
        ),
        (
            "scroll-horizontal",
            "横：ドラッグ・常時表示",
            Axis::Horizontal,
            ScrollbarInteraction::Draggable,
            false,
            30,
        ),
        (
            "scroll-fit",
            "内容が収まる：非表示",
            Axis::Vertical,
            ScrollbarInteraction::Draggable,
            true,
            1,
        ),
    ] {
        let handle = window
            .use_keyed_state(id, cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let region = ElementId::from((id, 0usize));
        // ページ全体もスクロールするので、内側が動いたホイールは外へ渡さない。
        let wheel = NestedScroll::new(&region, handle.clone(), window, cx);
        let content = div()
            .flex()
            .flex_none()
            .when(axis == Axis::Vertical, |this| this.flex_col())
            .children((0..count).map(|i| div().flex_none().p_3().child(format!("項目 {}", i + 1))));
        let scroller = div()
            .id(region)
            .w(px(240.0))
            .h(px(180.0))
            .when(axis == Axis::Vertical, |this| this.overflow_y_scroll())
            // 横は内容を折り返さずに並べ、幅を内容に合わせる。
            .when(axis == Axis::Horizontal, |this| {
                this.flex().overflow_x_scroll()
            })
            .track_scroll(&handle)
            .on_scroll(wheel.on_scroll())
            .child(content)
            .child(wheel.capture())
            .child(
                Scrollbar::new((id, 1usize), handle)
                    .axis(axis)
                    .interaction(interaction)
                    .fade(fade),
            );
        samples = samples.child(
            div()
                .flex()
                .flex_col()
                .w(px(240.0))
                .gap_2()
                .child(Text::new(TypeScale::LabelMedium, label))
                .child(scroller),
        );
    }
    Section::new("Scrollbar（操作はマウス、キー操作はコンテナ）").child(samples)
}
