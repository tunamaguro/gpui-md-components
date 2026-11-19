// 項目の間隔・押したボタンが隣を縮めて広がる幅・押下の動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt
//         (`ButtonGroup`、`ButtonGroupDefaults`、`ButtonGroupMeasurePolicy`、`EnlargeOnPressNode`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// Compose は収まらない項目をメニューに移すが、この実装は持たない。キーボードで押している間は広げず、ポインターで押している間だけ広げる。
// 隣を縮められる幅は、Compose の既定と同じくボタンの末尾側の余白 (`ButtonDefaults.ContentPadding`) にし、
// 隣の子がそれより狭く描けないボタンでも重ならないよう、隣をボタンの最小の幅より狭くしない
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のボタングループ

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Pixels, Refineable, RenderOnce, SpringAnimation, StyleRefinement, Styled,
    Window, canvas, div, prelude::FluentBuilder, px,
};
use gpui_headless::press::PressArea;
use md3_theme::md3_tokens::component::{baseline_button, button_group_small};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// ボタンを横に並べ、押しているボタンを隣を縮めて広げるグループ
///
/// 項目はボタンなどの要素で、間を 12dp 空けて左から並べる。項目をポインターで押している間は、その項目が隣を縮めた分だけ広がり、
/// 離すと元の幅に戻る。隣は [`ButtonGroupItem::compression_limit`] までと、ボタンの最小の幅 (58dp) までしか縮めない。
/// 幅を指定しない子は項目の幅いっぱいに描かれ、`Button` は縮めると左右の余白から縮む。
/// 幅を指定した子は項目の幅に追従しないため、その項目の `compression_limit` を 0 にする。
/// 押しても反応しない項目は [`ButtonGroupItem::disabled`] にする
///
/// グループ自身は支援技術に役割を公開しない。子のボタンがそれぞれの役割で並ぶ
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    items: Vec<ButtonGroupItem>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl ButtonGroup {
    /// 空のボタングループを作る
    ///
    /// `id` は押している子と幅の動きを描画をまたいで保つために使う。ウィンドウの中で一意にする
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 項目を末尾に加える
    pub fn item(mut self, item: ButtonGroupItem) -> Self {
        self.items.push(item);
        self
    }

    /// 項目をまとめて末尾に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ButtonGroupItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// 押下の動きのバネに使うテーマを指定する。指定しなければアプリのテーマを使う。子のテーマは変えない
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`ButtonGroup`] の項目
pub struct ButtonGroupItem {
    id: ElementId,
    element: AnyElement,
    disabled: bool,
    compression_limit: Option<Pixels>,
}

impl ButtonGroupItem {
    /// `element` を、押している間に広がる項目にする
    ///
    /// `id` はグループの中で一意にし、描画のたびに同じ値を渡すこと。押している状態と子の要素の id は `id` で保つため、
    /// 項目を加えたり並べ替えたりしても、ほかの項目の子のフォーカスは外れない
    pub fn new(id: impl Into<ElementId>, element: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            element: element.into_any_element(),
            disabled: false,
            compression_limit: None,
        }
    }

    /// `true` なら押しても広がらない。無効なボタンを加えるときに使う
    ///
    /// 広がらない項目も、隣の項目が押されると縮む
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 隣の項目が押されたときに、この項目を縮める幅の上限を指定する
    ///
    /// 指定しなければ、ボタンの末尾側の余白 (24dp) にする。幅を指定した子のように縮められない項目は 0 にする
    pub fn compression_limit(mut self, limit: Pixels) -> Self {
        self.compression_limit = Some(limit);
        self
    }
}

/// 項目を押しているか
#[derive(Default)]
struct PressState {
    /// ポインターのボタンを押したまま
    held: bool,
    /// 押下の動きが戻り始められるところまで進むのを待っている
    latched: bool,
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let spring = theme.spring(MotionSchemeKey::FastSpatial);
        let reduce_motion = cx.reduce_motion();
        let count = self.items.len();

        // 項目ごとの押している状態と、項目のバネが前の描画で記録した押下の動きの進み
        let presses: Vec<(Entity<PressState>, Rc<Cell<f32>>)> = self
            .items
            .iter()
            .map(|item| {
                window.with_element_namespace(self.id.clone(), |window| {
                    window.with_element_namespace(item.id.clone(), |window| {
                        (
                            window.use_keyed_state("press", cx, |_, _| PressState::default()),
                            window
                                .use_keyed_state("progress", cx, |_, _| Rc::new(Cell::new(0.0)))
                                .read(cx)
                                .clone(),
                        )
                    })
                })
            })
            .collect();
        // 何も押していない間に測った各項目の幅。項目の数が変わったら測り直す
        let natural_widths = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("natural-widths", cx, |_, _| Vec::<Pixels>::new())
        });
        if natural_widths.read(cx).len() != count {
            natural_widths.update(cx, |widths, _| widths.clear());
        }

        // 押すのをやめても、押下の動きが戻り始められるところまで進むまでは押した状態を保つ (`EnlargeOnPressNode`)。
        // モーションを減らす設定では動きが一瞬で終わるため、押している間だけ広げる
        let targets: Vec<bool> = presses
            .iter()
            .map(|(press, progress)| {
                let state = press.read(cx);
                if state.latched
                    && !state.held
                    && (reduce_motion
                        || progress.get() >= supplement::button_group::RELEASE_PROGRESS)
                {
                    press.update(cx, |state, _| state.latched = false);
                }
                let state = press.read(cx);
                state.held || state.latched
            })
            .collect();
        let progress: Vec<f32> = presses.iter().map(|(_, progress)| progress.get()).collect();
        let idle = progress.iter().all(|progress| *progress == 0.0) && !targets.contains(&true);

        // 押している項目は、端なら片側の隣を自分の幅の 15% まで、間なら左右の隣を 15% の半分ずつまで縮めて広がる。
        // 縮める幅は隣の上限で先に抑えてから押下の進みを掛ける (`ButtonGroupMeasurePolicy`)。
        // バネが行き過ぎて進みが負になっても、隣を広げて押した項目を縮めない。隣はボタンの最小の幅より狭くしない
        let natural = natural_widths.read(cx).clone();
        let limits: Vec<Pixels> = self
            .items
            .iter()
            .map(|item| {
                item.compression_limit
                    .unwrap_or_else(|| dp(baseline_button::TRAILING_SPACE))
            })
            .collect();
        let widths = (!idle && natural.len() == count && count > 1).then(|| {
            let ratio = supplement::button_group::EXPANDED_RATIO;
            let min_width = dp(supplement::button::MIN_WIDTH);
            let mut widths = natural.clone();
            let shrink = |widths: &mut Vec<Pixels>, neighbor: usize, target: Pixels| {
                let growth = target.min(widths[neighbor] - min_width).max(px(0.0));
                widths[neighbor] -= growth;
                growth
            };
            for (index, progress) in progress.iter().copied().enumerate() {
                if progress == 0.0 {
                    continue;
                }
                let own = widths[index];
                let growth = if index == 0 {
                    shrink(&mut widths, 1, (own * ratio).min(limits[1]) * progress)
                } else if index == count - 1 {
                    let target = (own * ratio).min(limits[index - 1]) * progress;
                    shrink(&mut widths, index - 1, target)
                } else {
                    let target = (own * ratio / 2.0)
                        .min(limits[index - 1])
                        .min(limits[index + 1])
                        * progress;
                    shrink(&mut widths, index - 1, target) + shrink(&mut widths, index + 1, target)
                };
                widths[index] += growth;
            }
            widths
        });

        // 項目の id をグループの中で解決するよう、グループに id を付ける
        let mut group = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_start()
            .gap(dp(button_group_small::BETWEEN_SPACE));
        group.style().refine(&self.style);
        let cells: Vec<Rc<Cell<f32>>> = presses
            .iter()
            .map(|(_, progress)| progress.clone())
            .collect();
        let items = self
            .items
            .into_iter()
            .zip(presses)
            .zip(targets)
            .enumerate()
            .map(|(index, ((item, (press, cell)), target))| {
                // 押下の動きの進みを記録するバネ。項目の幅は、記録した進みで次の描画で決める。
                // 項目の id を持つ要素の中に置くため、この id は項目どうしで重ならない
                let progress_spring = div().absolute().with_spring(
                    "press",
                    SpringAnimation::new(spring).to(target),
                    move |element, progress| {
                        cell.set(progress.0);
                        element
                    },
                );
                // 何も押していない間だけ、項目の幅を測り直す
                let natural_widths = natural_widths.clone();
                let measure = canvas(
                    move |bounds, _, cx| {
                        let width = bounds.size.width;
                        if idle && natural_widths.read(cx).get(index) != Some(&width) {
                            natural_widths.update(cx, |widths, _| {
                                widths.resize(count, px(0.0));
                                widths[index] = width;
                            });
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full();
                // 子を縦の flex の中に置き、交差軸の伸長で、幅を指定しない子を項目の幅いっぱいに描く
                PressArea::new(item.id.clone())
                    .disabled(item.disabled)
                    .on_press_change(move |pressed, _, cx| {
                        press.update(cx, |state, cx| {
                            state.held = pressed;
                            state.latched |= pressed;
                            cx.notify();
                        });
                    })
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .when_some(
                        widths.as_ref().map(|widths| widths[index]),
                        |this, width| this.w(width),
                    )
                    .child(item.element)
                    .child(measure)
                    .child(progress_spring)
            });
        // バネが記録した進みがこの描画で使った進みと違えば、記録した進みで描き直す
        let redraw = canvas(
            move |_, window, _| {
                if cells
                    .iter()
                    .zip(&progress)
                    .any(|(cell, progress)| cell.get() != *progress)
                {
                    window.request_animation_frame();
                }
            },
            |_, _, _, _| {},
        )
        .absolute();
        group.children(items).child(redraw)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        AnyElement, Context, Hsla, Modifiers, MouseButton, Render, TestAppContext,
        VisualTestContext, point,
    };

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(600.0)).child((self.0)())
        }
    }

    fn open(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        // バネを終端まで即座に進める
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.run_until_parked();
        cx
    }

    const COLORS: [fn() -> Hsla; 3] = [gpui::red, gpui::green, gpui::blue];

    /// 幅が `widths` で、狭くすると縮む項目。`disabled` の番号の項目は押しても広がらない
    fn group_of(widths: &[f32], disabled: Option<usize>) -> AnyElement {
        ButtonGroup::new("group")
            .items(
                widths
                    .iter()
                    .zip(COLORS)
                    .enumerate()
                    .map(|(index, (width, color))| {
                        // Button と同じく、内容より狭い幅を指定されたら縮められる子にする
                        let content = div().w(px(*width)).flex_shrink(1.0).min_w_0();
                        ButtonGroupItem::new(
                            index,
                            div().flex().h(px(40.0)).bg(color()).child(content),
                        )
                        .disabled(disabled == Some(index))
                    }),
            )
            .into_any_element()
    }

    /// 幅が 100dp の 3 つの項目
    fn group(disabled: Option<usize>) -> AnyElement {
        group_of(&[100.0; 3], disabled)
    }

    /// 各項目の x と幅
    fn items(cx: &mut VisualTestContext) -> Vec<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            COLORS
                .iter()
                .filter_map(|color| {
                    let background: gpui::Background = color().into();
                    let quad = quads.iter().find(|quad| quad.background == background)?;
                    Some((
                        quad.bounds.origin.x.0 / scale,
                        quad.bounds.size.width.0 / scale,
                    ))
                })
                .collect()
        })
    }

    /// バネが記録した進みで描き直すフレームを進める
    fn next_frames(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        for _ in 0..2 {
            cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
        }
    }

    fn press(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_down(point(px(x), px(20.0)), MouseButton::Left, Modifiers::none());
        next_frames(cx);
    }

    fn release(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_up(point(px(x), px(20.0)), MouseButton::Left, Modifiers::none());
        next_frames(cx);
    }

    #[gpui::test]
    fn 項目を_12dp_空けて並べる(cx: &mut TestAppContext) {
        let cx = open(cx, || group(None));
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 100.0), (224.0, 100.0)]
        );
    }

    #[gpui::test]
    fn 間の項目を押すと左右の隣を自分の幅の_15_パーセントの半分ずつ縮めて広がり_離すと戻る(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || group(None));
        press(cx, 162.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 92.5), (104.5, 115.0), (231.5, 92.5)],
            "左右の隣を 7.5dp ずつ縮めること"
        );
        release(cx, 162.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 100.0), (224.0, 100.0)]
        );
    }

    #[gpui::test]
    fn 端の項目を押すと片側の隣を自分の幅の_15_パーセント縮めて広がる(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || group(None));
        press(cx, 50.0);
        assert_eq!(items(cx), vec![(0.0, 115.0), (127.0, 85.0), (224.0, 100.0)]);
    }

    #[gpui::test]
    fn 項目の外で離しても戻る(cx: &mut TestAppContext) {
        let cx = open(cx, || group(None));
        press(cx, 274.0);
        assert_eq!(items(cx), vec![(0.0, 100.0), (112.0, 85.0), (209.0, 115.0)]);
        release(cx, 500.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 100.0), (224.0, 100.0)]
        );
    }

    #[gpui::test]
    fn 無効な項目は押しても広がらない(cx: &mut TestAppContext) {
        let cx = open(cx, || group(Some(1)));
        press(cx, 162.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 100.0), (224.0, 100.0)]
        );
    }

    #[gpui::test]
    fn 隣をボタンの最小の幅より狭くしない(cx: &mut TestAppContext) {
        let cx = open(cx, || group_of(&[200.0, 80.0, 58.0], None));
        press(cx, 100.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 222.0), (234.0, 58.0), (304.0, 58.0)],
            "隣は 58dp までの 22dp だけ縮むこと"
        );
        release(cx, 100.0);
        press(cx, 252.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 194.0), (206.0, 86.0), (304.0, 58.0)],
            "縮められない隣からは広がらないこと"
        );
    }

    #[gpui::test]
    fn 描画を挟まずに押して離すと広がったまま残らない(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || group(None));
        // 支援技術のクリックは、押す操作と離す操作を描画を挟まずに配送する
        cx.simulate_mouse_down(
            point(px(162.0), px(20.0)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.simulate_mouse_up(
            point(px(162.0), px(20.0)),
            MouseButton::Left,
            Modifiers::none(),
        );
        next_frames(cx);
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 100.0), (224.0, 100.0)]
        );
    }

    #[gpui::test]
    fn 項目を減らしても押すと広がる(cx: &mut TestAppContext) {
        let three = Rc::new(Cell::new(true));
        let state = three.clone();
        let cx = open(cx, move || {
            let count = if state.get() { 3 } else { 2 };
            group_of(&[100.0; 3][..count], None)
        });
        three.set(false);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        press(cx, 50.0);
        assert_eq!(items(cx), vec![(0.0, 115.0), (127.0, 85.0)]);
    }

    #[gpui::test]
    fn 縮める上限を_0_にした隣は縮めない(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            ButtonGroup::new("group")
                .item(ButtonGroupItem::new(
                    "first",
                    div().w(px(200.0)).h(px(40.0)).bg(gpui::red()),
                ))
                // 幅を指定した子は縮まないため、上限を 0 にする
                .item(
                    ButtonGroupItem::new(
                        "second",
                        div().w(px(100.0)).h(px(40.0)).bg(gpui::green()),
                    )
                    .compression_limit(px(0.0)),
                )
                .into_any_element()
        });
        press(cx, 100.0);
        assert_eq!(items(cx), vec![(0.0, 200.0), (212.0, 100.0)]);
    }
}
