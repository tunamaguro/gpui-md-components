//! MD3 のチェックボックス
//!
//! 寸法・描き方・動きは Compose Material3 の `Checkbox.kt` (@ 2a941dd11aabd9bb7d2016ed159764b220117c20) に、
//! 状態ごとの色は生成トークン ([`checkbox`]) に従う。状態レイヤーの色は Compose に無いため Material Web のトークン
//! ([`web_component::checkbox`]) に従う

use gpui::{
    AnimationExt as _, App, BorderStyle, Bounds, ElementId, Hsla, IntoElement, ParentElement as _,
    PathBuilder, Pixels, Point, RenderOnce, SharedString, SpringAnimation, SpringPlayback,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, point, quad, size,
    transparent_black,
};
pub use gpui_headless::choice::CheckedState;
use md3_theme::md3_tokens::component::checkbox;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{Dp, web_component};
use md3_theme::{ThemeExt as _, dp};

use crate::MIN_INTERACTIVE_SIZE;
use crate::indication::Indication;

/// 箱の角の半径。Compose の `Checkbox.kt` の `RadiusSize`
const CORNER_RADIUS: Dp = Dp(2.0);
/// 枠線とチェックマークの太さ。Compose の `CheckboxDefaults.StrokeWidth`
const STROKE_WIDTH: Dp = Dp(2.0);

type ChangeHandler = Box<dyn Fn(CheckedState, &mut Window, &mut App)>;

/// MD3 のチェックボックス
///
/// 状態は呼び出し側が持ち、[`Checkbox::new`] に毎回渡す。チェックボックス自身は状態を変えない。
/// 支援技術には role `checkbox` と、[`Checkbox::new`] に渡したラベルを名前として公開する。
/// マウスのクリック・フォーカス中の Space・支援技術からのクリック操作で [`Checkbox::on_change`] のハンドラを呼び、
/// Enter では呼ばない。操作できる範囲は 48x48dp を下回らない
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckedState,
    label: SharedString,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Checkbox {
    /// `state` を表示するチェックボックスを作る。`label` は表示せず、支援技術に名前として公開する
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// フォーカスと押下の表示が失われ、状態の切り替えの動きが途中で止まる
    pub fn new(
        id: impl Into<ElementId>,
        state: CheckedState,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state,
            label: label.into(),
            disabled: false,
            on_change: None,
        }
    }

    /// チェックボックスを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なチェックボックスは操作されてもハンドラを呼ばず、Tab で到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態を渡す。
    /// 現在の状態が [`CheckedState::Checked`] なら [`CheckedState::Unchecked`] を、それ以外なら
    /// [`CheckedState::Checked`] を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`Checkbox::new`] に渡すこと。
    /// 設定しない場合、チェックボックスは操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(
        mut self,
        handler: impl Fn(CheckedState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let enabled = !self.disabled;
        let indication = Indication::new(&self.id, enabled, window, cx);
        let state = self.state;
        let selected = state != CheckedState::Unchecked;
        let active = indication.hovered || indication.focus_visible || indication.pressed;

        // 選択中の色。未選択の箱とチェックマークは、この色を透明にした色から切り替える
        let (box_color, check_color) = if enabled {
            (
                theme.color(checkbox::SELECTED_CONTAINER_COLOR),
                theme.color(checkbox::SELECTED_ICON_COLOR),
            )
        } else {
            (
                theme
                    .color(checkbox::SELECTED_DISABLED_CONTAINER_COLOR)
                    .opacity(checkbox::SELECTED_DISABLED_CONTAINER_OPACITY),
                theme.color(checkbox::SELECTED_DISABLED_ICON_COLOR),
            )
        };
        let unselected_outline = if !enabled {
            theme
                .color(checkbox::UNSELECTED_DISABLED_OUTLINE_COLOR)
                .opacity(checkbox::UNSELECTED_DISABLED_CONTAINER_OPACITY)
        } else if active {
            theme.color(checkbox::UNSELECTED_HOVER_OUTLINE_COLOR)
        } else {
            theme.color(checkbox::UNSELECTED_OUTLINE_COLOR)
        };
        let (layer_color, ripple_color) = if selected {
            (
                web_component::checkbox::SELECTED_STATE_LAYER_COLOR,
                web_component::checkbox::SELECTED_PRESSED_STATE_LAYER_COLOR,
            )
        } else {
            (
                web_component::checkbox::UNSELECTED_STATE_LAYER_COLOR,
                web_component::checkbox::UNSELECTED_PRESSED_STATE_LAYER_COLOR,
            )
        };

        // Compose は選択に向かう色を DefaultEffects、未選択に向かう色を FastEffects で動かす
        let color_spring = SpringAnimation::new(theme.spring(if selected {
            MotionSchemeKey::DefaultEffects
        } else {
            MotionSchemeKey::FastEffects
        }))
        .to(selected);
        let spatial = theme.spring(MotionSchemeKey::DefaultSpatial);
        // 未選択に向かうとき、Compose は線を引く割合を 100ms 後に 0 へ飛ばし、それまでは色だけを薄くする。
        // ここでは未選択の間は線を引き切った形で描いて色だけを薄くし、割合はすぐ 0 に戻して次の選択に備える
        let draw_spring = SpringAnimation::new(spatial)
            .to(selected)
            .playback(if selected {
                SpringPlayback::Running
            } else {
                SpringPlayback::Completed
            });

        let check = div().size_full().with_spring(
            "md3-check-color",
            color_spring,
            move |this, color_phase| {
                let color_phase = color_phase.clamp(0.0..=1.0);
                this.child(div().size_full().with_spring(
                    "md3-check-draw",
                    draw_spring,
                    move |this, draw| {
                        // 未選択から選択に向かうときは、Compose と同じく形を動かさずに目標の形にする
                        let mixed_spring = SpringAnimation::new(spatial)
                            .to(state == CheckedState::Mixed)
                            .playback(match state {
                                CheckedState::Unchecked => SpringPlayback::Paused,
                                _ if draw.0 <= 0.0 => SpringPlayback::Completed,
                                _ => SpringPlayback::Running,
                            });
                        let fraction = if selected { draw.0 } else { 1.0 };
                        this.child(div().size_full().with_spring(
                            "md3-check-mixed",
                            mixed_spring,
                            move |this, mixed| {
                                this.child(
                                    canvas(
                                        |_, _, _| {},
                                        move |bounds, (), window, _| {
                                            paint_box(
                                                bounds,
                                                color_phase
                                                    .interpolate(box_color.opacity(0.0), box_color),
                                                color_phase
                                                    .interpolate(unselected_outline, box_color),
                                                window,
                                            );
                                            paint_check(
                                                bounds,
                                                color_phase.interpolate(
                                                    check_color.opacity(0.0),
                                                    check_color,
                                                ),
                                                fraction,
                                                mixed.0,
                                                window,
                                            );
                                        },
                                    )
                                    .size_full(),
                                )
                            },
                        ))
                    },
                ))
            },
        );

        let layer_size = dp(checkbox::STATE_LAYER_SIZE);
        let radii = theme.corner_radii(checkbox::STATE_LAYER_SHAPE, size(layer_size, layer_size));
        let layer = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .size(layer_size);
        let layer = indication.layers(
            layer,
            theme.color(layer_color),
            theme.color(ripple_color),
            radii,
        );
        let layer = indication.focus_ring(layer, &theme, radii).child(
            div()
                .flex_none()
                .size(dp(checkbox::CONTAINER_SIZE))
                .child(check),
        );

        let mut checkbox = gpui_headless::choice::Checkbox::new(self.id, state)
            .track_focus(indication.focus_handle())
            .disabled(self.disabled)
            .aria_label(self.label)
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(MIN_INTERACTIVE_SIZE))
            .min_h(dp(MIN_INTERACTIVE_SIZE))
            .child(layer);
        if let Some(handler) = self.on_change {
            checkbox = checkbox.on_change(handler);
        }
        indication.track(checkbox, &["space"])
    }
}

/// `bounds` に、中を `fill`・枠線を `outline` で塗った箱を描く
fn paint_box(bounds: Bounds<Pixels>, fill: Hsla, outline: Hsla, window: &mut Window) {
    let stroke = dp(STROKE_WIDTH);
    let radius = dp(CORNER_RADIUS);
    if fill == outline {
        window.paint_quad(quad(
            bounds,
            radius,
            fill,
            Pixels::ZERO,
            outline,
            BorderStyle::Solid,
        ));
        return;
    }
    // 半透明の色が重なって濃くならないよう、Compose と同じく中は枠線の内側だけを塗る
    let inner = Bounds {
        origin: bounds.origin + point(stroke, stroke),
        size: size(
            bounds.size.width - stroke * 2.0,
            bounds.size.height - stroke * 2.0,
        ),
    };
    window.paint_quad(quad(
        inner,
        (radius - stroke).max(Pixels::ZERO),
        fill,
        Pixels::ZERO,
        outline,
        BorderStyle::Solid,
    ));
    window.paint_quad(quad(
        bounds,
        radius,
        transparent_black(),
        stroke,
        outline,
        BorderStyle::Solid,
    ));
}

/// `bounds` に、始点から `fraction` の割合までのチェックマークを `color` で描く
///
/// `mixed` が 1 に近いほど、チェックマークを中央の横線に寄せる
fn paint_check(
    bounds: Bounds<Pixels>,
    color: Hsla,
    fraction: f32,
    mixed: f32,
    window: &mut Window,
) {
    if fraction <= 0.0 || color.a <= 0.0 {
        return;
    }
    let width = bounds.size.width;
    let lerp = |from: f32, to: f32| from + (to - from) * mixed;
    let at = |x: f32, y: f32| bounds.origin + point(width * x, width * y);
    // Compose の drawCheck の点 (isCheckboxStylingFixEnabled のとき)
    let points = [
        at(0.25, lerp(0.5, 0.5)),
        at(lerp(0.4, 0.5), lerp(0.65, 0.5)),
        at(0.75, lerp(0.3, 0.5)),
    ];
    let length = |a: Point<Pixels>, b: Point<Pixels>| {
        let (dx, dy) = (f32::from(b.x - a.x), f32::from(b.y - a.y));
        (dx * dx + dy * dy).sqrt()
    };
    let first = length(points[0], points[1]);
    let total = first + length(points[1], points[2]);
    let end = total * fraction.min(1.0);
    let towards = |a: Point<Pixels>, b: Point<Pixels>, distance: f32| {
        let t = distance / length(a, b).max(f32::EPSILON);
        a + point((b.x - a.x) * t, (b.y - a.y) * t)
    };

    // gpui の線は端を切り落とすため、Compose の StrokeCap.Square と同じく両端を太さの半分だけ延ばす
    let cap = f32::from(dp(STROKE_WIDTH)) / 2.0;
    let mut path = PathBuilder::stroke(dp(STROKE_WIDTH));
    path.move_to(towards(points[0], points[1], -cap));
    if end <= first {
        path.line_to(towards(points[0], points[1], end + cap));
    } else {
        path.line_to(points[1]);
        path.line_to(towards(points[1], points[2], end - first + cap));
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::test_support::{focus_next, init_theme, press_key};

    struct CheckboxView {
        state: CheckedState,
        disabled: bool,
        changes: Rc<RefCell<Vec<CheckedState>>>,
    }

    impl Render for CheckboxView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let entity = cx.entity();
            div().size_full().child(
                Checkbox::new("checkbox", self.state, "Agree")
                    .disabled(self.disabled)
                    .on_change(move |state, _, cx| {
                        changes.borrow_mut().push(state);
                        entity.update(cx, |view, cx| {
                            view.state = state;
                            cx.notify();
                        });
                    }),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        state: CheckedState,
        disabled: bool,
    ) -> (Entity<CheckboxView>, &mut VisualTestContext) {
        init_theme(cx);
        cx.add_window_view(|_, _| CheckboxView {
            state,
            disabled,
            changes: Rc::default(),
        })
    }

    fn changes(view: &Entity<CheckboxView>, cx: &mut VisualTestContext) -> Vec<CheckedState> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    #[gpui::test]
    fn 押すたびに状態を切り替える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Mixed, false);
        // 操作できる範囲は 48x48dp で、箱 (18dp) の外側の角を押しても切り替わる
        cx.simulate_click(point(px(2.0), px(46.0)), Modifiers::none());
        cx.simulate_click(point(px(24.0), px(24.0)), Modifiers::none());
        cx.simulate_click(point(px(24.0), px(52.0)), Modifiers::none());
        assert_eq!(
            changes(&view, cx),
            [CheckedState::Checked, CheckedState::Unchecked],
            "48dp の範囲の外は押せないこと"
        );
    }

    #[gpui::test]
    fn space_で切り替わり_enter_では切り替わらない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, CheckedState::Unchecked, false);
        focus_next(cx);
        press_key(cx, "enter");
        press_key(cx, "space");
        assert_eq!(changes(&view, cx), [CheckedState::Checked]);
    }

    #[gpui::test]
    fn 無効なチェックボックスは_tab_で到達できず切り替わらない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, CheckedState::Checked, true);
        focus_next(cx);
        press_key(cx, "space");
        cx.simulate_click(point(px(24.0), px(24.0)), Modifiers::none());
        assert!(changes(&view, cx).is_empty());
    }
}
