// 項目の間隔・押したボタンが隣を縮めて広がる幅・押下の動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt
//         (`ButtonGroup`、`ButtonGroupDefaults`、`ButtonGroupMeasurePolicy`、`EnlargeOnPressNode`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// Compose は収まらない項目をメニューに移すが、この実装は持たない。キーボードで押している間は広げず、ポインターで押している間だけ広げる。
// 隣を縮められる幅は、Compose の既定と同じくボタンの末尾側の余白 (`ButtonDefaults.ContentPadding`) にする
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のボタングループ

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnimationExt, AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Refineable, RenderOnce, SharedString, SpringAnimation, StyleRefinement,
    Styled, Window, canvas, div, prelude::FluentBuilder, px,
};
use md3_theme::md3_tokens::component::{baseline_button, button_group_small};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

/// ボタンを横に並べ、押しているボタンを隣を縮めて広げるグループ
///
/// 子はボタンなどの要素で、間を 12dp 空けて左から並べる。子を押している間は、その子が隣を縮めた分だけ広がり、
/// 離すと元の幅に戻る。子は広がった幅いっぱいに描かれる。押しても反応しない子は [`ButtonGroupItem::disabled`] で加える
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

    /// 色と動きに使うテーマを指定する。指定しなければアプリのテーマを使う
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl ParentElement for ButtonGroup {
    /// 子を、押している間に広がる項目として末尾に加える
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.items
            .extend(elements.into_iter().map(ButtonGroupItem::new));
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`ButtonGroup`] の項目
pub struct ButtonGroupItem {
    element: AnyElement,
    disabled: bool,
}

impl ButtonGroupItem {
    /// `element` を、押している間に広がる項目にする
    pub fn new(element: impl IntoElement) -> Self {
        Self {
            element: element.into_any_element(),
            disabled: false,
        }
    }

    /// `true` なら押しても広がらない。無効なボタンを加えるときに使う
    ///
    /// 広がらない項目も、隣の項目が押されると縮む
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let count = self.items.len();

        // 項目ごとの押している状態と、前の描画での押下の動きの進み
        let presses: Vec<(Entity<PressState>, Rc<Cell<f32>>)> = (0..count)
            .map(|index| {
                window.with_element_namespace(
                    (self.id.clone(), SharedString::from(format!("item-{index}"))),
                    |window| {
                        (
                            window.use_keyed_state("press", cx, |_, _| PressState::default()),
                            window
                                .use_keyed_state("progress", cx, |_, _| Rc::new(Cell::new(0.0)))
                                .read(cx)
                                .clone(),
                        )
                    },
                )
            })
            .collect();
        // 何も押していない間に測った各項目の幅
        let natural_widths = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("natural-widths", cx, |_, _| Vec::<Pixels>::new())
        });

        // 押すのをやめても、押下の動きが戻り始められるところまで進むまでは押した状態を保つ (`EnlargeOnPressNode`)
        let targets: Vec<bool> = presses
            .iter()
            .map(|(press, progress)| {
                let state = press.read(cx);
                if state.latched
                    && !state.held
                    && progress.get() >= supplement::button_group::RELEASE_PROGRESS
                {
                    press.update(cx, |state, _| state.latched = false);
                }
                let state = press.read(cx);
                state.held || state.latched
            })
            .collect();

        let gap = dp(button_group_small::BETWEEN_SPACE);
        let compression_limit = dp(baseline_button::TRAILING_SPACE);
        let ratio = supplement::button_group::EXPANDED_RATIO;
        let natural = natural_widths.read(cx).clone();
        let mut group = div().flex().flex_row().items_start().gap(gap);
        group.style().refine(&self.style);

        let items: Vec<_> = self
            .items
            .into_iter()
            .zip(presses.iter().map(|(press, _)| press.clone()))
            .collect();
        let progress_cells: Vec<Rc<Cell<f32>>> = presses
            .iter()
            .map(|(_, progress)| progress.clone())
            .collect();

        // 項目の幅はすべての項目の押下の動きの進みで決まる。各項目のバネを外側から順に重ね、
        // 外側の各バネが記録した進みを、最も内側 (最初の項目) のバネで項目を組み立てるときに使う
        let build = {
            let natural_widths = natural_widths.clone();
            move |group: gpui::Div, progress: Vec<f32>| {
                let idle = progress.iter().all(|progress| *progress == 0.0);
                // 押している項目は、端なら片側の隣を自分の幅の 15% まで、間なら左右の隣を 15% の半分ずつまで縮めて広がる。
                // 縮める幅は隣の末尾側の余白を超えない (`ButtonGroupMeasurePolicy`)
                let widths = (!idle && natural.len() == count && count > 1).then(|| {
                    let mut widths = natural.clone();
                    for (index, progress) in progress.iter().copied().enumerate() {
                        if progress == 0.0 {
                            continue;
                        }
                        if index == 0 {
                            let target = (widths[index] * ratio).min(compression_limit) * progress;
                            let growth = target.min(widths[index + 1]);
                            widths[index + 1] -= growth;
                            widths[index] += growth;
                        } else if index == count - 1 {
                            let target = (widths[index] * ratio).min(compression_limit) * progress;
                            let growth = target.min(widths[index - 1]);
                            widths[index - 1] -= growth;
                            widths[index] += growth;
                        } else {
                            let target =
                                (widths[index] * ratio / 2.0).min(compression_limit) * progress;
                            let growth_left = target.min(widths[index - 1]);
                            let growth_right = target.min(widths[index + 1]);
                            widths[index - 1] -= growth_left;
                            widths[index + 1] -= growth_right;
                            widths[index] += growth_left + growth_right;
                        }
                    }
                    widths
                });
                group.children(items.into_iter().enumerate().map(|(index, (item, press))| {
                    let natural_widths = natural_widths.clone();
                    // 何も押していない間だけ、項目の幅を測り直す
                    let measure = canvas(
                        move |bounds, _, cx| {
                            if !idle {
                                return;
                            }
                            let width = bounds.size.width;
                            if natural_widths.read(cx).get(index) != Some(&width) {
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
                    // 子を縦の flex の中に置き、交差軸の伸長で項目の幅いっぱいに描く
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .flex_none()
                        .when_some(
                            widths.as_ref().map(|widths| widths[index]),
                            |this, width| this.w(width),
                        )
                        .when(!item.disabled, |this| {
                            let down = press.clone();
                            let up = press.clone();
                            let up_out = press;
                            this.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                down.update(cx, |state, cx| {
                                    state.held = true;
                                    state.latched = true;
                                    cx.notify();
                                });
                            })
                            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                                up.update(cx, |state, cx| {
                                    state.held = false;
                                    cx.notify();
                                });
                            })
                            .on_mouse_up_out(
                                MouseButton::Left,
                                move |_, _, cx| {
                                    up_out.update(cx, |state, cx| {
                                        if state.held {
                                            state.held = false;
                                            cx.notify();
                                        }
                                    });
                                },
                            )
                        })
                        .child(item.element)
                        .child(measure)
                }))
            }
        };

        if count == 0 {
            return group.into_any_element();
        }
        // 外側のバネが進みを記録する場所。最も内側のバネが自分の進みを加えて項目を組み立てる
        let recorded: Rc<Vec<Cell<f32>>> = Rc::new((0..count).map(|_| Cell::new(0.0)).collect());
        let spring_id = |index: usize| -> ElementId {
            (
                self.id.clone(),
                SharedString::from(format!("press-{index}")),
            )
                .into()
        };
        let mut element = {
            let recorded = recorded.clone();
            let cell = progress_cells[0].clone();
            group
                .with_spring(
                    spring_id(0),
                    SpringAnimation::new(spring).to(targets[0]),
                    move |group, progress| {
                        cell.set(progress.0);
                        recorded[0].set(progress.0);
                        let progress = recorded.iter().map(Cell::get).collect();
                        build(group, progress)
                    },
                )
                .into_any_element()
        };
        for index in 1..count {
            let recorded = recorded.clone();
            let cell = progress_cells[index].clone();
            element = element
                .with_spring(
                    spring_id(index),
                    SpringAnimation::new(spring).to(targets[index]),
                    move |element, progress| {
                        cell.set(progress.0);
                        recorded[index].set(progress.0);
                        element
                    },
                )
                .into_any_element();
        }
        element
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        AnyElement, Context, Hsla, Modifiers, Render, TestAppContext, VisualTestContext, point,
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

    /// 幅が 100dp で、狭くすると縮む 3 つの項目。`disabled` の番号の項目は押しても広がらない
    fn group(disabled: Option<usize>) -> AnyElement {
        ButtonGroup::new("group")
            .items(COLORS.iter().enumerate().map(|(index, color)| {
                // Button と同じく、内容より狭い幅を指定されたら縮められる子にする
                let content = div().w(px(100.0)).flex_shrink(1.0).min_w_0();
                ButtonGroupItem::new(div().flex().h(px(40.0)).bg(color()).child(content))
                    .disabled(disabled == Some(index))
            }))
            .into_any_element()
    }

    /// 各項目の x と幅
    fn items(cx: &mut VisualTestContext) -> Vec<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            COLORS
                .iter()
                .map(|color| {
                    let background: gpui::Background = color().into();
                    let quad = quads
                        .iter()
                        .find(|quad| quad.background == background)
                        .expect("項目が描かれていること");
                    (
                        quad.bounds.origin.x.0 / scale,
                        quad.bounds.size.width.0 / scale,
                    )
                })
                .collect()
        })
    }

    fn press(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_down(point(px(x), px(20.0)), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
    }

    fn release(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_mouse_up(point(px(x), px(20.0)), MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
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
}
