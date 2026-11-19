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
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, Refineable,
    RenderOnce, SpringAnimation, StyleRefinement, Styled, Window, canvas, div,
    prelude::FluentBuilder, px,
};
use gpui_headless::press::{PressArea, PressScope};
use md3_theme::md3_tokens::component::{baseline_button, button_group_small};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// ボタンを横に並べ、押しているボタンを隣を縮めて広げるグループ
///
/// 項目はボタンなどの要素で、間を 12dp 空けて左から並べる。項目をポインターで押している間は、その項目が隣を縮めた分だけ広がり、
/// 離すと元の幅に戻る。隣は [`ButtonGroupItem::compression_limit`] までと、ボタンの最小の幅 (58dp) までしか縮めない。
/// 幅を指定しない子は項目の幅いっぱいに描かれ、`Button` は縮めると左右の余白から縮む。
/// 幅を指定した子は項目の幅に追従しないため、その項目は `compression_limit` を 0 にし、
/// 押しても広がらないよう [`ButtonGroupItem::disabled`] にする。子のクリックはそのまま使える。
/// 押しても反応しない項目も `disabled` にする
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

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let spring = theme.spring(MotionSchemeKey::FastSpatial);
        let reduce_motion = cx.reduce_motion();
        let count = self.items.len();

        // 項目ごとの、ポインターで押しているか・押下の動きが戻り始められるところまで進むのを待っているか・
        // 項目のバネが前の描画で記録した押下の動きの進み。項目の id で保つ
        let states: Vec<_> = self
            .items
            .iter()
            .map(|item| {
                window.with_element_namespace(self.id.clone(), |window| {
                    window.with_element_namespace(item.id.clone(), |window| {
                        (
                            window.use_keyed_state("pressed", cx, |_, _| false),
                            window.use_keyed_state("latched", cx, |_, _| false),
                            window
                                .use_keyed_state("progress", cx, |_, _| Rc::new(Cell::new(0.0)))
                                .read(cx)
                                .clone(),
                        )
                    })
                })
            })
            .collect();
        // 何も押していない間に測った各項目の幅。項目の id で保ち、並べ替えても別の項目の幅を使わない
        let natural_widths = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("natural-widths", cx, |_, _| {
                HashMap::<ElementId, Pixels>::new()
            })
        });

        // 押すのをやめても、押下の動きが戻り始められるところまで進むまでは押した状態を保つ (`EnlargeOnPressNode`)。
        // モーションを減らす設定では動きが一瞬で終わるため、押している間だけ広げる
        let targets: Vec<bool> = states
            .iter()
            .map(|(pressed, latched, progress)| {
                let pressed = *pressed.read(cx);
                let waiting = !reduce_motion
                    && (pressed
                        || (*latched.read(cx)
                            && progress.get() < supplement::button_group::RELEASE_PROGRESS));
                if *latched.read(cx) != waiting {
                    latched.update(cx, |latched, _| *latched = waiting);
                }
                pressed || waiting
            })
            .collect();
        let progress: Vec<f32> = states
            .iter()
            .map(|(_, _, progress)| progress.get())
            .collect();
        let idle = progress.iter().all(|progress| *progress == 0.0) && !targets.contains(&true);

        // 描画から外れた項目の幅は捨て、加え直した項目は測り直すまで広げない
        natural_widths.update(cx, |widths, _| {
            widths.retain(|id, _| self.items.iter().any(|item| item.id == *id));
        });
        let natural: Option<Vec<Pixels>> = {
            let measured = natural_widths.read(cx);
            self.items
                .iter()
                .map(|item| measured.get(&item.id).copied())
                .collect()
        };
        let limits: Vec<Pixels> = self
            .items
            .iter()
            .map(|item| {
                item.compression_limit
                    .unwrap_or_else(|| dp(baseline_button::TRAILING_SPACE))
            })
            .collect();
        let held: Vec<bool> = states
            .iter()
            .map(|(pressed, _, _)| *pressed.read(cx))
            .collect();
        let widths = natural
            .filter(|_| !idle && count > 1)
            .map(|natural| item_widths(&natural, &limits, &progress, &held));
        // 項目の幅を決めていない間は、どの項目も子の幅なので測り直す
        let measuring = widths.is_none();

        // 先に描く項目の子が離す操作の伝播を止めても、押している状態を戻せるよう、グループで離す操作を受ける
        let mut group = PressScope::new(
            self.id.clone(),
            states.iter().map(|(pressed, _, _)| pressed.clone()),
        )
        .flex()
        .flex_row()
        .items_start()
        .gap(dp(button_group_small::BETWEEN_SPACE));
        group.style().refine(&self.style);
        let cells: Vec<Rc<Cell<f32>>> = states
            .iter()
            .map(|(_, _, progress)| progress.clone())
            .collect();
        let items = self
            .items
            .into_iter()
            .zip(states)
            .zip(targets)
            .enumerate()
            .map(|(index, ((item, (pressed, _, cell)), target))| {
                // 押下の動きの進みを記録するバネ。項目の幅は、記録した進みで次の描画で決める。
                // 子が付ける文字列の id と重ならないよう、ソースの位置を id にする
                let progress_spring = div().absolute().with_spring(
                    ElementId::CodeLocation(*std::panic::Location::caller()),
                    SpringAnimation::new(spring).to(target),
                    move |element, progress| {
                        cell.set(progress.0);
                        element
                    },
                );
                let natural_widths = natural_widths.clone();
                let id = item.id.clone();
                let measure = canvas(
                    move |bounds, window, cx| {
                        let width = bounds.size.width;
                        if measuring && natural_widths.read(cx).get(&id) != Some(&width) {
                            // 測っていなかった項目があって広げられなかった場合に、測った幅で描き直す
                            natural_widths.update(cx, |widths, _| {
                                widths.insert(id, width);
                            });
                            window.request_animation_frame();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full();
                // 子を縦の flex の中に置き、交差軸の伸長で、幅を指定しない子を項目の幅いっぱいに描く
                PressArea::new(item.id.clone(), &pressed)
                    .disabled(item.disabled)
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

/// 押下の動きの進みが `progress` のときの各項目の幅
///
/// 押している項目は、端なら片側の隣を自分の幅の 15% まで、間なら左右の隣を 15% の半分ずつまで縮めて広がる。
/// 縮める幅は隣の上限で抑えてから進みを掛ける (`ButtonGroupMeasurePolicy`)。
/// 実際に縮める幅は、バネが行き過ぎても、ほかの項目の押下と重なっても、隣の上限とボタンの最小の幅を超えない。
/// 進みが負になっても、隣を広げて押した項目を縮めない。
/// 押すのをやめた項目は、押している項目を縮めない。押している項目が動くと、押したまま離してもクリックにならないため
fn item_widths(
    natural: &[Pixels],
    limits: &[Pixels],
    progress: &[f32],
    held: &[bool],
) -> Vec<Pixels> {
    let count = natural.len();
    let ratio = supplement::button_group::EXPANDED_RATIO;
    let min_width = dp(supplement::button::MIN_WIDTH);
    let mut widths = natural.to_vec();
    for (index, progress) in progress.iter().copied().enumerate() {
        if progress == 0.0 {
            continue;
        }
        let mut shrink = |neighbor: usize, target: Pixels| {
            if held[neighbor] && !held[index] {
                return px(0.0);
            }
            let compressed = natural[neighbor] - widths[neighbor];
            let growth = target
                .min(limits[neighbor] - compressed)
                .min(widths[neighbor] - min_width)
                .max(px(0.0));
            widths[neighbor] -= growth;
            growth
        };
        let own = natural[index];
        let growth = if index == 0 {
            shrink(1, (own * ratio).min(limits[1]) * progress)
        } else if index == count - 1 {
            shrink(index - 1, (own * ratio).min(limits[index - 1]) * progress)
        } else {
            let target = (own * ratio / 2.0)
                .min(limits[index - 1])
                .min(limits[index + 1])
                * progress;
            shrink(index - 1, target) + shrink(index + 1, target)
        };
        widths[index] += growth;
    }
    widths
}

#[cfg(test)]
mod tests {
    use gpui::{
        AnyElement, Context, Hsla, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
        PlatformInput, Render, TestAppContext, VisualTestContext, point,
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
        cx.update(|_, cx| cx.set_reduce_motion(false));
        // 支援技術のクリックは、押す操作と離す操作を描画を挟まずに配送する
        cx.update(|window, cx| {
            let position = point(px(162.0), px(20.0));
            window.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    modifiers: Modifiers::none(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            window.dispatch_event(
                PlatformInput::MouseUp(MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    modifiers: Modifiers::none(),
                    click_count: 1,
                }),
                cx,
            );
        });
        next_frames(cx);
        // バネは実時間で進むため、広げる向きに動いていれば、待つと幅が変わる
        std::thread::sleep(std::time::Duration::from_millis(50));
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

    #[gpui::test]
    fn 押している間に並べ替えても項目の幅を取り違えない(
        cx: &mut TestAppContext,
    ) {
        let reordered = Rc::new(Cell::new(false));
        let state = reordered.clone();
        let cx = open(cx, move || {
            let item = |id: &'static str, width: f32, color: Hsla| {
                let content = div().w(px(width)).flex_shrink(1.0).min_w_0();
                ButtonGroupItem::new(id, div().flex().h(px(40.0)).bg(color).child(content))
            };
            // 2 つ目は幅を指定した子で、縮めず広げない
            let fixed =
                ButtonGroupItem::new("fixed", div().w(px(200.0)).h(px(40.0)).bg(gpui::green()))
                    .compression_limit(px(0.0))
                    .disabled(true);
            let group = ButtonGroup::new("group");
            let group = if state.get() {
                group.item(fixed).item(item("a", 100.0, gpui::red()))
            } else {
                group.item(item("a", 100.0, gpui::red())).item(fixed)
            };
            group
                .item(item("c", 100.0, gpui::blue()))
                .into_any_element()
        });
        press(cx, 374.0);
        assert_eq!(
            items(cx),
            vec![(0.0, 100.0), (112.0, 200.0), (324.0, 100.0)],
            "縮められない隣からは広がらないこと"
        );
        reordered.set(true);
        next_frames(cx);
        cx.update(|window, _| window.refresh());
        next_frames(cx);
        assert_eq!(
            items(cx),
            vec![(212.0, 85.0), (0.0, 200.0), (309.0, 115.0)],
            "並べ替えた後も各項目の幅で広げること"
        );
    }

    #[gpui::test]
    fn 押している間に外して加え直した項目は_測り直すまで広げない(
        cx: &mut TestAppContext,
    ) {
        let phase = Rc::new(Cell::new(0));
        let state = phase.clone();
        let cx = open(cx, move || {
            let item = |id: &'static str, color: Hsla| {
                let content = div().w(px(100.0)).flex_shrink(1.0).min_w_0();
                ButtonGroupItem::new(id, div().flex().h(px(40.0)).bg(color).child(content))
            };
            // 1 つ目は幅を指定した子で、外している間に幅を変える
            let fixed = |width: f32| {
                ButtonGroupItem::new("fixed", div().w(px(width)).h(px(40.0)).bg(gpui::red()))
                    .compression_limit(px(0.0))
                    .disabled(true)
            };
            let group = ButtonGroup::new("group");
            let group = match state.get() {
                0 => group.item(fixed(100.0)),
                1 => group,
                _ => group.item(fixed(160.0)),
            };
            group
                .item(item("b", gpui::green()))
                .item(item("c", gpui::blue()))
                .into_any_element()
        });
        press(cx, 274.0);
        for next in [1, 2] {
            phase.set(next);
            cx.update(|window, _| window.refresh());
            next_frames(cx);
        }
        assert_eq!(
            items(cx),
            vec![(0.0, 160.0), (172.0, 85.0), (269.0, 115.0)],
            "加え直した項目の今の幅で並べること"
        );
    }

    #[test]
    fn 押すのをやめた項目は押している項目を縮めない() {
        let natural = [px(100.0); 2];
        let limits = [px(24.0); 2];
        // 1 つ目の押下の動きが戻り始める前に、2 つ目を押した
        let widths = item_widths(&natural, &limits, &[0.5, 0.0], &[false, true]);
        assert_eq!(widths, vec![px(100.0), px(100.0)]);
        let widths = item_widths(&natural, &limits, &[0.5, 0.0], &[true, false]);
        assert_eq!(widths, vec![px(107.5), px(92.5)]);
    }
}
