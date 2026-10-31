//! スクロール領域とフォーカス追従。
use crate::Scrollbar;
use crate::scrollbar::ScrollbarInteraction;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, Axis, Div, ElementId, IntoElement, ScrollHandle, Stateful, StyleRefinement,
    Window, div,
};
use gpui_headless::{NestedScroll, ScrollScope, scroll_handle};

/// スクロール領域。
///
/// 中の要素へキーボードフォーカスが移ると見える範囲まで動かし、内側で動いたホイールは外の領域へ渡さない。
/// 位置を読み書きするときは [`ScrollArea::handle`] で同じハンドルを渡す。
///
/// ```ignore
/// ScrollArea::new("page").size_full().child(content)
/// ```
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    base: Stateful<Div>,
    handle: Option<ScrollHandle>,
    axis: Axis,
    scrollbar: bool,
    interaction: ScrollbarInteraction,
    fade: bool,
}

impl ScrollArea {
    /// 安定した ID から縦の領域を作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            base: div().id(id.clone()),
            id,
            handle: None,
            axis: Axis::Vertical,
            scrollbar: true,
            interaction: ScrollbarInteraction::default(),
            fade: true,
        }
    }

    /// 位置の読み書きに使うハンドルを渡す。
    pub fn handle(mut self, handle: ScrollHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    /// スクロール軸を選ぶ。横では中身を折り返さずに並べる。
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// スクロールバーのポインタ操作を選ぶ。
    pub fn scrollbar_interaction(mut self, interaction: ScrollbarInteraction) -> Self {
        self.interaction = interaction;
        self
    }

    /// false ならスクロール可能な間はスクロールバーを常に表示する。
    pub fn scrollbar_fade(mut self, fade: bool) -> Self {
        self.fade = fade;
        self
    }
}

impl Styled for ScrollArea {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements);
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let scroll = self
            .handle
            .unwrap_or_else(|| scroll_handle(&id, window, cx));
        let wheel = NestedScroll::new(&id, scroll.clone(), window, cx);
        let base = match self.axis {
            Axis::Vertical => self.base.overflow_y_scroll(),
            Axis::Horizontal => self.base.flex().overflow_x_scroll(),
        };
        ScrollScope::new(
            scroll.clone(),
            base.track_scroll(&scroll)
                .on_scroll(wheel.on_scroll())
                .child(wheel.capture())
                .when(self.scrollbar, |this| {
                    this.child(
                        Scrollbar::new(
                            ElementId::NamedChild(id.into(), "scrollbar".into()),
                            scroll,
                        )
                        .axis(self.axis)
                        .interaction(self.interaction)
                        .fade(self.fade),
                    )
                }),
        )
    }
}

/// オーバーレイの本文。ヘッダーと操作ボタンを残したまま縮小できる領域。
#[cfg(feature = "overlays")]
pub(crate) fn content(
    parent: &ElementId,
    children: Vec<AnyElement>,
    scrollbar: bool,
) -> impl IntoElement + use<> {
    ScrollArea::new(ElementId::NamedChild(
        parent.clone().into(),
        "scroll-content".into(),
    ))
    .min_h_0()
    .scrollbar(scrollbar)
    .child(div().flex().flex_col().children(children))
}
