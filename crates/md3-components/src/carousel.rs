// 種類ごとのキーラインの選び方と既定値は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/Carousel.kt
//         (`HorizontalMultiBrowseCarousel`、`HorizontalUncontainedCarousel`、
//         `HorizontalCenteredHeroCarousel`、`CarouselDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! MD3 のカルーセル

use std::rc::Rc;

use gpui::{
    App, Div, ElementId, IntoElement, ParentElement, Pixels, Refineable, RenderOnce, Stateful,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_headless::carousel::{Carousel as HeadlessCarousel, keylines};
pub use gpui_headless::carousel::{CarouselItem, CarouselSnap};
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 小さい項目の横幅の最小 (Compose の `CarouselDefaults.MinSmallItemSize`)
const MIN_SMALL_ITEM_WIDTH: Pixels = px(40.0);
/// 小さい項目の横幅の最大 (Compose の `CarouselDefaults.MaxSmallItemSize`)
const MAX_SMALL_ITEM_WIDTH: Pixels = px(56.0);

type ItemFn = dyn Fn(CarouselItem, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div>;
type ClickFn = dyn Fn(&usize, &mut Window, &mut App);

/// カルーセルの種類
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CarouselVariant {
    /// 大きい項目の後ろに、中くらいと小さい項目を並べる。
    /// 大きい項目の横幅は `preferred_item_width` に近づけ、カルーセルに収まるよう調整する
    MultiBrowse {
        /// 大きい項目の横幅の目安
        preferred_item_width: Pixels,
    },
    /// 同じ横幅の項目を並べ、収まらない分の項目を端で小さくする
    Uncontained {
        /// 項目の横幅
        item_width: Pixels,
    },
    /// 大きい項目を中央に置き、両側に小さい項目を並べる。項目が 3 つ未満なら、大きい項目を始まりに寄せる
    CenteredHero {
        /// 大きい項目の横幅の目安。`None` ならカルーセルの横幅
        preferred_item_width: Option<Pixels>,
    },
}

impl CarouselVariant {
    /// 種類ごとの、ホイールでスクロールした後に止まる位置。
    /// Compose の各カルーセルの既定の `flingBehavior` に対応する
    fn snap(self) -> CarouselSnap {
        match self {
            Self::MultiBrowse { .. } | Self::CenteredHero { .. } => CarouselSnap::SingleAdvance,
            Self::Uncontained { .. } => CarouselSnap::None,
        }
    }
}

/// 大きさの異なる項目を横に並べ、スクロールで項目の大きさを移り変わらせる MD3 のカルーセル
///
/// 項目の容器は角を extra large の形で丸め、内容をその大きさで切り抜く。
/// 切り抜きは矩形のため、画像などの内容の角は `theme.corner_radii(ShapeKey::CornerExtraLarge, item.size)` で丸めること。
/// カルーセルの高さは [`Styled`] で与える。
///
/// 操作とアクセシビリティは [`gpui_headless::carousel::Carousel`] と同じ。
/// 操作する対象の項目には、カルーセルのフォーカスを表示すべきときにフォーカスリングを内側に描く。
/// カルーセルと項目のアクセシブルな名前は `aria_label` で与えること
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    item_count: usize,
    variant: CarouselVariant,
    item_spacing: Pixels,
    snap: Option<CarouselSnap>,
    item: Option<Rc<ItemFn>>,
    on_click: Option<Rc<ClickFn>>,
    style: StyleRefinement,
    aria_label: Option<gpui::SharedString>,
    theme: Option<SharedTheme>,
}

impl Carousel {
    /// `variant` の種類で `item_count` 個の項目を並べるカルーセルを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるとスクロールの位置が失われる
    pub fn new(id: impl Into<ElementId>, item_count: usize, variant: CarouselVariant) -> Self {
        Self {
            id: id.into(),
            item_count,
            variant,
            item_spacing: px(0.0),
            snap: None,
            item: None,
            on_click: None,
            style: StyleRefinement::default(),
            aria_label: None,
            theme: None,
        }
    }

    /// 項目の間隔。既定は 0
    pub fn item_spacing(mut self, spacing: Pixels) -> Self {
        self.item_spacing = spacing;
        self
    }

    /// ホイールでスクロールした後に止まる位置。既定は、[`CarouselVariant::Uncontained`] なら
    /// [`CarouselSnap::None`]、それ以外は [`CarouselSnap::SingleAdvance`]
    pub fn snap(mut self, snap: CarouselSnap) -> Self {
        self.snap = Some(snap);
        self
    }

    /// 項目の内容を作る
    ///
    /// `build` は項目の状態と、大きさ・形・操作を設定済みの容器を受け取り、内容を付けて返す
    pub fn item(
        mut self,
        build: impl Fn(CarouselItem, Stateful<Div>, &mut Window, &mut App) -> Stateful<Div> + 'static,
    ) -> Self {
        self.item = Some(Rc::new(build));
        self
    }

    /// 項目が押されたときに、項目の番号を受け取るハンドラ
    pub fn on_click(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// カルーセルのアクセシブルな名前
    pub fn aria_label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for Carousel {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Carousel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let count = self.item_count;
        let variant = self.variant;
        let min_small = f32::from(MIN_SMALL_ITEM_WIDTH);
        let max_small = f32::from(MAX_SMALL_ITEM_WIDTH);
        let keylines = move |width: f32, spacing: f32| match variant {
            CarouselVariant::MultiBrowse {
                preferred_item_width,
            } => keylines::multi_browse(
                width,
                f32::from(preferred_item_width),
                spacing,
                count,
                min_small,
                max_small,
            ),
            CarouselVariant::Uncontained { item_width } => {
                keylines::uncontained(width, f32::from(item_width), spacing)
            }
            CarouselVariant::CenteredHero {
                preferred_item_width,
            } => keylines::hero(
                width,
                preferred_item_width.map(f32::from),
                spacing,
                count,
                true,
                min_small,
                max_small,
            ),
        };
        let build = self.item;
        let mut carousel = HeadlessCarousel::new(self.id, count, keylines)
            .item_spacing(self.item_spacing)
            .snap(self.snap.unwrap_or(variant.snap()))
            .item(move |item, container, window, cx| {
                // extra large の形は 4 つの角が同じ丸め
                let radius = theme
                    .corner_radii(ShapeKey::CornerExtraLarge, item.size)
                    .top_left;
                let container = container.relative().overflow_hidden().rounded(radius);
                let container = match &build {
                    Some(build) => build(item, container, window, cx),
                    None => container,
                };
                container.when(item.focus_visible, |this| {
                    this.child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded(radius)
                            .border(dp(focus_ring::WIDTH))
                            .border_color(theme.color(focus_ring::COLOR)),
                    )
                })
            })
            .when_some(self.on_click, |this, on_click| {
                this.on_click(move |index, window, cx| on_click(index, window, cx))
            })
            .when_some(self.aria_label, |this, label| this.aria_label(label));
        carousel.style().refine(&self.style);
        carousel
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::{
        Context, Modifiers, Render, ScrollDelta, ScrollEvent, TestAppContext, VisualTestContext,
        point,
    };

    use super::*;

    struct CarouselView {
        variant: CarouselVariant,
        count: usize,
        /// 項目の内容の最小の横幅
        min_width: f32,
    }

    impl Render for CarouselView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().flex().items_start().child(
                Carousel::new("carousel", self.count, self.variant)
                    .item({
                        let min_width = self.min_width;
                        move |_, item, _, _| item.min_w(px(min_width)).bg(gpui::red())
                    })
                    .w(px(360.))
                    .h(px(200.)),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        variant: CarouselVariant,
        count: usize,
    ) -> &mut VisualTestContext {
        add_view_with(cx, variant, count, 0.)
    }

    fn add_view_with(
        cx: &mut TestAppContext,
        variant: CarouselVariant,
        count: usize,
        min_width: f32,
    ) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            CarouselView {
                variant,
                count,
                min_width,
            }
        });
        cx.update(|window, cx| {
            window.activate_window();
            cx.set_reduce_motion(true);
        });
        cx.run_until_parked();
        cx
    }

    /// 赤く塗った項目のうち、切り抜いた後に見えている部分の左端と幅。左から順
    fn items(cx: &mut VisualTestContext) -> Vec<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut items: Vec<(f32, f32)> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == gpui::red().into())
                .map(|quad| quad.bounds.intersect(&quad.content_mask.bounds))
                // 重ならない矩形の交わりは、丸めのため 1px 未満の幅を持つことがある
                .filter(|b| b.size.width.0 / scale >= 1.)
                .map(|b| (b.origin.x.0 / scale, b.size.width.0 / scale))
                .collect();
            items.sort_by(|a, b| a.0.total_cmp(&b.0));
            items
        })
    }

    #[gpui::test]
    fn 種類ごとのキーラインで項目を並べる(cx: &mut TestAppContext) {
        let cx = add_view(
            cx,
            CarouselVariant::Uncontained {
                item_width: px(150.),
            },
            10,
        );
        // 150 を 2 つと、残りの 60 の 1.5 倍の 90 を 1 つ並べ、3 つ目はカルーセルの右端で切れる
        assert_eq!(items(cx), [(0., 150.), (150., 150.), (300., 60.)]);

        let cx = add_view(
            cx,
            CarouselVariant::MultiBrowse {
                preferred_item_width: px(186.),
            },
            10,
        );
        let multi = items(cx);
        assert_eq!(multi.len(), 3, "{multi:?}");
        assert_eq!(multi[0].0, 0.);
        assert!(
            multi[0].1 > multi[1].1 && multi[1].1 > multi[2].1,
            "{multi:?}"
        );

        let cx = add_view(
            cx,
            CarouselVariant::CenteredHero {
                preferred_item_width: None,
            },
            10,
        );
        let hero = items(cx);
        let (x, width) = hero
            .iter()
            .copied()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("項目を描くこと");
        // 最初の項目は中央ではなく始まりに寄せる
        assert_eq!(x, 0., "{hero:?}");
        assert!(width > 200., "{hero:?}");
    }

    #[gpui::test]
    fn 中央に寄せたヒーローは止めた項目を中央に置く(cx: &mut TestAppContext) {
        let cx = add_view(
            cx,
            CarouselVariant::CenteredHero {
                preferred_item_width: None,
            },
            10,
        );
        cx.simulate_keystrokes("tab right right");
        let hero = items(cx);
        let (x, width) = hero
            .iter()
            .copied()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("項目を描くこと");
        assert!((x + width / 2. - 180.).abs() < 0.01, "{hero:?}");
    }

    #[gpui::test]
    fn 内容が広くても項目の大きさで切り抜く(cx: &mut TestAppContext) {
        let cx = add_view_with(
            cx,
            CarouselVariant::CenteredHero {
                preferred_item_width: None,
            },
            10,
            100.,
        );
        cx.simulate_keystrokes("tab right right");
        // 左の見えない端に置いた項目 (横幅 10) は、内容が 100 あってもカルーセルの外に収まる
        let hero = items(cx);
        assert_eq!(hero.len(), 3, "{hero:?}");
        assert_eq!(hero[0], (0., 40.));
    }

    #[gpui::test]
    fn 項目のフォーカスリングを内側に描く(cx: &mut TestAppContext) {
        let cx = add_view(
            cx,
            CarouselVariant::Uncontained {
                item_width: px(150.),
            },
            10,
        );
        let ring = |cx: &mut VisualTestContext| {
            cx.update(|window, cx| {
                let scale = window.scale_factor();
                let color = cx.md3_theme().color(focus_ring::COLOR);
                window
                    .painted_quads()
                    .into_iter()
                    .filter(|quad| quad.border_color == color)
                    .map(|quad| {
                        let b = quad.bounds;
                        (b.origin.x.0 / scale, b.size.width.0 / scale)
                    })
                    // 枠線は辺ごとの矩形に分けて描かれる
                    .fold(Vec::new(), |mut rings, ring| {
                        if !rings.contains(&ring) {
                            rings.push(ring);
                        }
                        rings
                    })
            })
        };
        assert_eq!(ring(cx), []);
        cx.simulate_keystrokes("tab");
        assert_eq!(ring(cx), [(0., 150.)]);
        // 最後の項目は右端に止まる
        cx.simulate_keystrokes("end");
        assert_eq!(ring(cx), [(210., 150.)]);
    }

    #[gpui::test]
    fn 種類ごとにホイールの後に止める位置を変える(cx: &mut TestAppContext) {
        for (variant, snaps) in [
            (
                CarouselVariant::Uncontained {
                    item_width: px(150.),
                },
                false,
            ),
            (
                CarouselVariant::MultiBrowse {
                    preferred_item_width: px(186.),
                },
                true,
            ),
        ] {
            let cx = add_view(cx, variant, 10);
            cx.simulate_event(ScrollEvent {
                position: point(px(100.), px(100.)),
                delta: ScrollDelta::Pixels(point(px(0.), px(-40.))),
                ..Default::default()
            });
            let scrolled = items(cx);
            cx.executor().advance_clock(Duration::from_secs(1));
            cx.run_until_parked();
            let settled = items(cx);
            assert_eq!(
                settled == scrolled,
                !snaps,
                "{variant:?}: {scrolled:?} {settled:?}"
            );
            if snaps {
                assert_eq!(settled[0].0, 0., "{variant:?}");
            }
        }
    }

    #[gpui::test]
    fn 項目を押すと番号を伝える(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let clicks = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view({
            let clicks = clicks.clone();
            move |window, cx| {
                cx.init_md3_theme(window);
                ClickView(clicks)
            }
        });
        cx.simulate_click(point(px(200.), px(100.)), Modifiers::none());
        assert_eq!(*clicks.borrow(), [1]);
    }

    struct ClickView(std::rc::Rc<std::cell::RefCell<Vec<usize>>>);

    impl Render for ClickView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.0.clone();
            div().flex().items_start().child(
                Carousel::new(
                    "carousel",
                    10,
                    CarouselVariant::Uncontained {
                        item_width: px(150.),
                    },
                )
                .on_click(move |index, _, _| clicks.borrow_mut().push(*index))
                .w(px(360.))
                .h(px(200.)),
            )
        }
    }
}
