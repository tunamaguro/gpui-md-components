// 寸法・色・トラックと目盛りの描き方・つまみの位置と幅は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt
//         (`SliderImpl`、`SliderDefaults` の `colors`・`Track`・`drawTrack`、`ThumbContent`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// フォーカスリングをつまみの外側に描くことは Material Web に合わせている
// (https://github.com/material-components/material-web)
// Source: slider/internal/slider.ts (つまみの中の `md-focus-ring`) @ de5470f
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のスライダー

use std::ops::RangeInclusive;

use gpui::{
    App, Bounds, Corners, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, div, fill, point, prelude::FluentBuilder, relative,
    size,
};
use gpui_headless::slider::Slider as HeadlessSlider;

use crate::shape::Shaped;
use crate::util;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::slider as tokens;
use md3_theme::md3_tokens::{focus_ring, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// つまみが、スライダーが押されているかを知るための group 名
const GROUP: &str = "md3-slider";

type ChangeHandler = Box<dyn Fn(&f32, &mut Window, &mut App)>;

/// MD3 のスライダー
///
/// 値は呼び出し側が持ち、[`Slider::new`] に毎回渡す。部品自身は値を変えない。
/// 次の操作で [`Slider::on_change`] のハンドラに新しい値を求める
///
/// - マウスの左ボタンを押した位置の値。押したまま動かすと、ボタンを離すまで部品の外でも位置を追う。
///   値を変えられる間は、左ボタンの押下を親に伝えない
/// - フォーカス中の右と上の矢印キーで 1 段階増やし、左と下の矢印キーで 1 段階減らす。段階が無ければ範囲の 1% ずつ増減する
/// - Page Up と Page Down で、段階の数の 1/10 を 2 から 10 段階に収めた量だけ増減する。段階が無ければ範囲の 10% ずつ増減する
/// - Home で最小の値、End で最大の値にする
/// - 支援技術からの増減と値の設定
///
/// 支援技術には role `slider` と、値・最小・最大・段階・横向きとして公開され、
/// [`Slider::aria_label`] がアクセシブルな名前になる。ラベルは表示しないため、名前は必ず [`Slider::aria_label`] で与えること
///
/// トラックの左端が最小の値、右端が最大の値を表し、つまみの位置で値を示す。
/// [`Styled`] で指定したスタイルはスライダーの箱に付く。既定では親の幅いっぱいに広がり、高さはつまみと同じ 44dp。
/// フォーカスリングはつまみの外側に、箱からはみ出して描く
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f32,
    range: RangeInclusive<f32>,
    steps: usize,
    on_change: Option<ChangeHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl Slider {
    /// `value` を表示するスライダーを作る。範囲は 0 から 1 で、範囲のどの値も選べる
    ///
    /// 範囲の外の値は範囲の端の値として、段階がある場合は最も近い段階の値として表示し、支援技術に公開する。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Slider::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            range: 0.0..=1.0,
            steps: 0,
            on_change: None,
            aria_label: None,
            disabled: false,
            focus_handle: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 選べる値の範囲を設定する。始まりは終わり以下にすること
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    /// 範囲の両端の間に、等間隔に `steps` 個の選べる値を置く。既定の 0 では範囲のどの値も選べる
    ///
    /// 選べる値は両端と合わせて `steps + 2` 個になり、トラックの上に目盛りを描く
    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }

    /// 値を変える操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には求められた値を渡す。今の値と同じ値では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Slider::new`] に渡すこと。
    /// 設定しない場合、スライダーは操作されても何も呼ばず、支援技術に値を変える操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// スライダーを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なスライダーは無効の色で描かれ、操作されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのスライダーのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合はスライダーが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

/// 状態で決まるスライダーの色。`SliderDefaults.colors`
struct Colors {
    thumb: Hsla,
    active_track: Hsla,
    inactive_track: Hsla,
    /// 有効な部分のトラックに描く目盛り
    active_tick: Hsla,
    /// 無効な部分のトラックに描く目盛り
    inactive_tick: Hsla,
}

impl Colors {
    fn resolve(theme: &SharedTheme, disabled: bool) -> Self {
        if disabled {
            let active = theme
                .color(tokens::DISABLED_ACTIVE_TRACK_COLOR)
                .opacity(tokens::DISABLED_ACTIVE_TRACK_OPACITY);
            let inactive = theme
                .color(tokens::DISABLED_INACTIVE_TRACK_COLOR)
                .opacity(tokens::DISABLED_INACTIVE_TRACK_OPACITY);
            Self {
                // Compose は無効なつまみの色を Surface に重ねて不透明にする
                thumb: theme.color(ColorRole::Surface).blend(
                    theme
                        .color(tokens::DISABLED_HANDLE_COLOR)
                        .opacity(tokens::DISABLED_HANDLE_OPACITY),
                ),
                active_track: active,
                inactive_track: inactive,
                active_tick: inactive,
                inactive_tick: active,
            }
        } else {
            let active = theme.color(tokens::ACTIVE_TRACK_COLOR);
            let inactive = theme.color(tokens::INACTIVE_TRACK_COLOR);
            Self {
                thumb: theme.color(tokens::HANDLE_COLOR),
                active_track: active,
                inactive_track: inactive,
                active_tick: inactive,
                inactive_tick: active,
            }
        }
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let colors = Colors::resolve(&theme, self.disabled);
        let (start, end) = (*self.range.start(), *self.range.end());
        let steps = self.steps;

        // 範囲に収め、段階があれば最も近い段階にした値と、その範囲の中での割合 (`SliderState.coercedValueAsFraction`)
        let step = (end - start) / (steps + 1) as f32;
        let (value, fraction) = if end > start {
            let value = self.value.max(start).min(end);
            let fraction = (value - start) / (end - start);
            if steps > 0 {
                let index = (fraction * (steps + 1) as f32).round();
                // headless の Slider の段階で選べる値 (始まりから段階ごとの値と終わり) に合わせる。
                // 最後の段階は、浮動小数点の誤差で終わりに届かないことがあるため終わりにする
                let value = if index as usize > steps {
                    end
                } else {
                    (start + step * index).min(end)
                };
                (value, index / (steps + 1) as f32)
            } else {
                (value, fraction)
            }
        } else {
            (start, 0.0)
        };

        let thumb_width = dp(tokens::HANDLE_WIDTH);
        let track_height = dp(tokens::INACTIVE_TRACK_HEIGHT);
        // トラックの端の角丸は高さの半分 (`TrackImpl` の cornerSize)
        let corner = track_height / 2.0;
        // 段階があり両端以外の値では、値の位置を角丸の内側の範囲に置く (`SliderImpl` と `drawTrack`)
        let inset = thumb_width / 2.0
            + if steps > 0 && fraction > 0.0 && fraction < 1.0 {
                corner
            } else {
                Pixels::ZERO
            };
        // スライダーの幅が `width` のときの、左端から値の位置までの距離
        let value_x = move |width: Pixels| width * fraction + inset * (1.0 - 2.0 * fraction);

        let track = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                // トラックはつまみの幅の半分だけ内側から始まる (`SliderImpl` の trackOffsetX)
                let left = bounds.left() + thumb_width / 2.0;
                let width = bounds.size.width - thumb_width;
                let center_y = bounds.center().y;
                let rect = |from: Pixels, to: Pixels| {
                    Bounds::new(
                        point(left + from, center_y - track_height / 2.0),
                        size(to - from, track_height),
                    )
                };
                let inside_corner = dp(supplement::slider::TRACK_INSIDE_CORNER_SIZE);
                let dot = |x: Pixels, color: Hsla, window: &mut Window| {
                    let diameter = dp(tokens::STOP_INDICATOR_SIZE);
                    window.paint_quad(
                        fill(
                            Bounds::centered_at(
                                point(left + x, center_y),
                                size(diameter, diameter),
                            ),
                            color,
                        )
                        .corner_radii(diameter / 2.0),
                    );
                };

                let value_end = value_x(bounds.size.width) - thumb_width / 2.0;
                // つまみの両側に空ける幅。つまみの幅の半分と隙間の和
                let gap = thumb_width / 2.0 + dp(tokens::ACTIVE_HANDLE_LEADING_SPACE);
                if value_end < width - gap - corner {
                    window.paint_quad(
                        fill(rect(value_end + gap, width), colors.inactive_track).corner_radii(
                            Corners {
                                top_left: inside_corner,
                                top_right: corner,
                                bottom_right: corner,
                                bottom_left: inside_corner,
                            },
                        ),
                    );
                    // 終わりの印は有効な部分のトラックの色で描く
                    dot(width - corner, colors.active_track, window);
                }
                let active_end = value_end - gap;
                if active_end > corner {
                    window.paint_quad(
                        fill(rect(Pixels::ZERO, active_end), colors.active_track).corner_radii(
                            Corners {
                                top_left: corner,
                                top_right: inside_corner,
                                bottom_right: inside_corner,
                                bottom_left: corner,
                            },
                        ),
                    );
                }
                // 目盛りは角丸の内側の範囲に等間隔に置く。最後の目盛りは終わりの印と重なるため描かず、
                // つまみの両側の空きに入る目盛りも描かない
                for index in (0..=steps).filter(|_| steps > 0) {
                    let x = corner + (width - corner * 2.0) * (index as f32 / (steps + 1) as f32);
                    if (x - value_end).abs() <= gap {
                        continue;
                    }
                    let color = if x <= active_end {
                        colors.active_tick
                    } else {
                        colors.inactive_tick
                    };
                    dot(x, color, window);
                }
            },
        )
        .absolute()
        .size_full();

        let handle_height = dp(tokens::HANDLE_HEIGHT);
        // 押されている間とフォーカスを示す間は、つまみを細くする (`ThumbContent`)
        let narrow = (thumb_width - dp(tokens::PRESSED_HANDLE_WIDTH)) / 2.0;
        let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
        let thumb = div()
            .id("thumb")
            .absolute()
            .top(relative(0.5))
            .mt(-handle_height / 2.0)
            .left(relative(fraction))
            .ml(inset * (1.0 - 2.0 * fraction) - thumb_width / 2.0)
            .w(thumb_width)
            .h(handle_height)
            .cursor_pointer()
            .when(focus_visible, |this| this.px(narrow))
            .when(!self.disabled, |this| {
                this.group_active(GROUP, |style| style.px(narrow))
            })
            .child(Shaped::new(
                div().size_full().bg(colors.thumb),
                theme.clone(),
                tokens::HANDLE_SHAPE,
            ))
            .when(focus_visible, |this| {
                this.child(
                    Shaped::new(
                        div()
                            .absolute()
                            .top(-outset)
                            .left(-outset)
                            .right(-outset)
                            .bottom(-outset)
                            .border(dp(focus_ring::WIDTH))
                            .border_color(theme.color(focus_ring::COLOR)),
                        theme.clone(),
                        tokens::HANDLE_SHAPE,
                    )
                    .outset(outset),
                )
            });

        let mut slider = HeadlessSlider::new(self.id, value)
            .range(self.range)
            .when(steps > 0, |this| this.step(step))
            // ポインタの位置はトラックの範囲で値に対応させる (`SliderState` の thumbWidth / 2 から)
            .track_inset(thumb_width / 2.0)
            .disabled(self.disabled)
            .track_focus(&focus_handle)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .group(GROUP)
            .relative()
            .w_full()
            .h(handle_height);
        slider.style().refine(&self.style);
        slider.child(track).child(thumb)
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, px};

    use super::*;

    /// スライダーの幅。トラックはつまみの幅の半分 (2dp) ずつ内側の 400dp になる
    const WIDTH: f32 = 404.0;

    struct SliderView {
        value: f32,
        range: RangeInclusive<f32>,
        steps: usize,
        disabled: bool,
        changes: Rc<RefCell<Vec<f32>>>,
    }

    impl Render for SliderView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            div().flex().flex_col().w(px(WIDTH)).child(
                Slider::new("slider", self.value)
                    .range(self.range.clone())
                    .steps(self.steps)
                    .disabled(self.disabled)
                    .aria_label("Volume")
                    .on_change(move |value, _, _| changes.borrow_mut().push(*value)),
            )
        }
    }

    fn add_slider(
        cx: &mut TestAppContext,
        value: f32,
        steps: usize,
        disabled: bool,
    ) -> (gpui::Entity<SliderView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            SliderView {
                value,
                range: 0.0..=1.0,
                steps,
                disabled,
                changes: Rc::default(),
            }
        });
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        (view, cx)
    }

    /// 塗りのある矩形の x, y, 幅, 高さと、左上と右上の角丸の半径 (論理ピクセル)、塗り。左から順に並べる
    type Fill = ((f32, f32, f32, f32), (f32, f32), gpui::Background);

    fn fills(cx: &mut VisualTestContext) -> Vec<Fill> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut fills: Vec<Fill> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_widths.top.0 == 0.0)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        (
                            quad.corner_radii.top_left.0 / scale,
                            quad.corner_radii.top_right.0 / scale,
                        ),
                        quad.background,
                    )
                })
                .collect();
            fills.sort_by(|a, b| a.0.0.total_cmp(&b.0.0));
            fills
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 期待値は Compose の `SliderImpl` と `drawTrack`。トラックは高さ 16dp で、つまみの幅の半分 (2dp) 内側から始まる。
    /// 値の位置の両側に、つまみの幅の半分と `ThumbTrackGapSize` (6dp) の和の 8dp を空け、
    /// 有効な部分は左端を 8dp、つまみ側を `TrackInsideCornerSize` (2dp) の角丸にする。
    /// 終わりの印は直径 4dp で、トラックの右端から角丸の半径 (8dp) 内側に描く。つまみは 4×44dp
    #[gpui::test]
    fn 値の位置につまみを置きトラックを空けて描く(cx: &mut TestAppContext) {
        let (_, cx) = add_slider(cx, 0.5, 0, false);
        let primary = color(cx, ColorRole::Primary).into();
        let container = color(cx, ColorRole::SecondaryContainer).into();
        assert_eq!(
            fills(cx),
            vec![
                ((2.0, 14.0, 192.0, 16.0), (8.0, 2.0), primary),
                ((200.0, 0.0, 4.0, 44.0), (2.0, 2.0), primary),
                ((210.0, 14.0, 192.0, 16.0), (2.0, 8.0), container),
                ((392.0, 20.0, 4.0, 4.0), (2.0, 2.0), primary),
            ]
        );
    }

    /// 端の値では、有効な部分が角丸の半径より短いか、無効な部分が値の位置の空きと角丸に収まるため描かない
    #[gpui::test]
    fn 端の値では片側のトラックだけを描く(cx: &mut TestAppContext) {
        let (view, cx) = add_slider(cx, 0.0, 0, false);
        let primary = color(cx, ColorRole::Primary).into();
        let container = color(cx, ColorRole::SecondaryContainer).into();
        assert_eq!(
            fills(cx),
            vec![
                ((0.0, 0.0, 4.0, 44.0), (2.0, 2.0), primary),
                ((10.0, 14.0, 392.0, 16.0), (2.0, 8.0), container),
                ((392.0, 20.0, 4.0, 4.0), (2.0, 2.0), primary),
            ]
        );

        view.update(cx, |view, cx| {
            view.value = 1.0;
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(
            fills(cx),
            vec![
                ((2.0, 14.0, 392.0, 16.0), (8.0, 2.0), primary),
                ((400.0, 0.0, 4.0, 44.0), (2.0, 2.0), primary),
            ]
        );
    }

    /// 期待値は Compose の `drawTrack` と `SliderImpl`。段階が 3 つなら目盛りは 5 つで、角丸の内側 (8dp から 392dp) に
    /// 等間隔に置く。値の位置も角丸の内側の範囲で決める。最後の目盛りと、値の位置から 8dp 以内の目盛りは描かない。
    /// 有効な部分の目盛りは SecondaryContainer、無効な部分の目盛りは Primary
    #[gpui::test]
    fn 段階があれば目盛りを描きつまみを角丸の内側の範囲に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_slider(cx, 0.25, 3, false);
        let primary = color(cx, ColorRole::Primary).into();
        let container = color(cx, ColorRole::SecondaryContainer).into();
        assert_eq!(
            fills(cx),
            vec![
                ((2.0, 14.0, 96.0, 16.0), (8.0, 2.0), primary),
                // 8dp の目盛り
                ((8.0, 20.0, 4.0, 4.0), (2.0, 2.0), container),
                // 値の位置 8 + 384 × 0.25 = 104dp
                ((104.0, 0.0, 4.0, 44.0), (2.0, 2.0), primary),
                ((114.0, 14.0, 288.0, 16.0), (2.0, 8.0), container),
                // 200dp と 296dp の目盛りと終わりの印
                ((200.0, 20.0, 4.0, 4.0), (2.0, 2.0), primary),
                ((296.0, 20.0, 4.0, 4.0), (2.0, 2.0), primary),
                ((392.0, 20.0, 4.0, 4.0), (2.0, 2.0), primary),
            ]
        );
    }

    /// 期待値は Compose の `defaultSliderColors`。つまみは OnSurface の 38% を Surface に重ねた不透明な色、
    /// 有効な部分は OnSurface の 38%、無効な部分は OnSurface の 12% で、目盛りは逆の色にする
    #[gpui::test]
    fn 無効なスライダーは_on_surface_の色で描く(cx: &mut TestAppContext) {
        let (_, cx) = add_slider(cx, 0.5, 3, true);
        let on_surface = color(cx, ColorRole::OnSurface);
        let surface = color(cx, ColorRole::Surface);
        let active = on_surface.opacity(0.38).into();
        let inactive = on_surface.opacity(0.12).into();
        let backgrounds: Vec<_> = fills(cx).into_iter().map(|(_, _, fill)| fill).collect();
        assert_eq!(
            backgrounds,
            vec![
                // 有効な部分のトラックと 8dp・104dp の目盛り
                active,
                inactive,
                inactive,
                surface.blend(on_surface.opacity(0.38)).into(),
                // 無効な部分のトラックと 296dp の目盛り、終わりの印
                inactive,
                active,
                active,
            ]
        );
    }

    /// つまみの幅
    fn thumb_width(cx: &mut VisualTestContext) -> f32 {
        fills(cx)
            .into_iter()
            .find(|((_, _, _, height), _, _)| *height == 44.0)
            .map(|((_, _, width, _), _, _)| width)
            .expect("つまみを描くこと")
    }

    /// 期待値は Compose の `ThumbContent`。押している間とフォーカスを示す間はつまみの幅を半分 (2dp) にする
    #[gpui::test]
    fn 押している間とフォーカスを示す間はつまみを細くする(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_slider(cx, 0.5, 0, false);
        assert_eq!(thumb_width(cx), 4.0);
        cx.simulate_mouse_down(
            point(px(100.), px(22.)),
            gpui::MouseButton::Left,
            Modifiers::none(),
        );
        assert_eq!(thumb_width(cx), 2.0);
        cx.simulate_mouse_up(
            point(px(100.), px(22.)),
            gpui::MouseButton::Left,
            Modifiers::none(),
        );
        assert_eq!(thumb_width(cx), 4.0);

        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        assert_eq!(thumb_width(cx), 2.0);
    }

    /// リングはつまみ (4×44dp) の外側 5dp に描き、角丸は形 (CornerFull) の半径 2dp を 5dp 広げた 7dp になる
    #[gpui::test]
    fn tab_のフォーカスをつまみの外側のリングで示す(cx: &mut TestAppContext) {
        let (_, cx) = add_slider(cx, 0.5, 0, false);
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        let ring = color(cx, ColorRole::Secondary);
        let rings: Vec<_> = cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_widths.top.0 > 0.0)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                        quad.corner_radii.top_left.0 / scale,
                        quad.border_color,
                    )
                })
                .collect()
        });
        assert!(!rings.is_empty(), "リングを描くこと");
        // gpui は塗りの無い枠線を辺ごとに分けて描く
        for quad in rings {
            assert_eq!(quad, (195.0, -5.0, 14.0, 54.0, 7.0, ring));
        }
    }

    /// トラックの範囲 (左右 2dp 内側) で位置を値に対応させ、段階があれば最も近い段階の値を求める
    #[gpui::test]
    fn 押した位置の値を求め段階があれば最も近い段階にする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_slider(cx, 0.0, 0, false);
        cx.simulate_click(point(px(102.), px(22.)), Modifiers::none());
        let changes = view.read_with(cx, |view, _| view.changes.clone());
        assert_eq!(*changes.borrow(), vec![0.25]);

        let (view, cx) = add_slider(cx, 0.0, 3, false);
        // トラックの 30% の位置
        cx.simulate_click(point(px(122.), px(22.)), Modifiers::none());
        let changes = view.read_with(cx, |view, _| view.changes.clone());
        assert_eq!(*changes.borrow(), vec![0.25]);
    }

    /// 段階が 3 つなら、段階の大きさは範囲の 1/4
    #[gpui::test]
    fn 段階の値に丸めた値から矢印キーで_1_段階ずつ増減する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_slider(cx, 0.3, 3, false);
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab right");
        view.update(cx, |view, cx| {
            view.value = 0.7;
            cx.notify();
        });
        cx.simulate_keystrokes("left");
        let changes = view.read_with(cx, |view, _| view.changes.clone());
        assert_eq!(*changes.borrow(), vec![0.5, 0.5]);
    }

    /// f32 では 0.7 / 9 * 9 が 0.7 に届かない
    #[gpui::test]
    fn 最大の値では_end_と右の矢印キーで値を求めない(cx: &mut TestAppContext) {
        let (view, cx) = add_slider(cx, 0.7, 8, false);
        view.update(cx, |view, cx| {
            view.range = 0.0..=0.7;
            cx.notify();
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab end right");
        let changes = view.read_with(cx, |view, _| view.changes.clone());
        assert_eq!(*changes.borrow(), Vec::<f32>::new());
    }
}
