// 寸法・色の選択・つまみの位置と動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt
//         (`SwitchImpl`、`ThumbNode`、`SwitchDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// hover・フォーカス時のつまみの色とフォーカスリングの形は Compose にないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: switch/internal/_handle.scss、switch/internal/_switch.scss @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のスイッチ

use gpui::{
    AnimationExt, App, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, SpringAnimation, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, size, svg,
};
use gpui_headless::choice::Switch as HeadlessSwitch;

use crate::util;
use md3_icons::IconStyle;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::switch;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, Theme, ThemeExt, dp};

/// [`Switch`] の中の [`SwitchIndicator`] が、スイッチの hover を知るための group 名
const GROUP: &str = "md3-switch";

/// MD3 のスイッチ
///
/// 状態は呼び出し側が持ち、[`Switch::new`] に毎回渡す。部品自身は状態を変えない。
/// 押されると [`Switch::on_click`] のハンドラに押した後の状態を渡すため、ハンドラで呼び出し側の状態を更新すること。
/// 支援技術には role `switch` として公開され、[`Switch::aria_label`] がアクセシブルな名前になる。
/// ラベルは表示しないため、名前は必ず [`Switch::aria_label`] で与え、状態によって変えないこと
///
/// 状態が変わると、つまみはテーマのモーションのバネで新しい位置と大きさへ動く
///
/// 大きさは 52×32dp のトラックで、この範囲が押せる範囲になる。
/// つまみの状態レイヤーとフォーカスリングはトラックの外側にはみ出して描く
///
/// [`Styled`] で指定したスタイルはスイッチの外側の箱に付く
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    inner: HeadlessSwitch,
    focus_handle: Option<FocusHandle>,
    indicator: SwitchIndicator,
    outer: gpui::Div,
}

impl Switch {
    /// `checked` を表示するスイッチを作る。`true` がオン
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Switch::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        let id = id.into();
        Self {
            inner: HeadlessSwitch::new(id.clone(), checked).flex(),
            indicator: SwitchIndicator::new(checked),
            id,
            focus_handle: None,
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
        }
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には押した後に求められる状態として、現在の状態を反転した値を渡す。
    /// マウスのクリックとフォーカス中の Space で呼び、Enter では呼ばない。
    /// 設定しない場合、押されても何も呼ばないが、Tab では到達できる。
    /// 操作させない場合は [`Switch::disabled`] を使うこと
    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }

    /// `true` を渡すと、オンのときにつまみへチェックのアイコンを出す。既定では出さない
    ///
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn check_icon(mut self, check_icon: bool) -> Self {
        self.indicator = self.indicator.check_icon(check_icon);
        self
    }

    /// スイッチを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なスイッチは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self.indicator = self.indicator.disabled(disabled);
        self
    }

    /// `focus_handle` をこのスイッチのフォーカスとして使う
    ///
    /// 指定しない場合はスイッチが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.indicator = self.indicator.theme(theme);
        self
    }
}

impl RenderOnce for Switch {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        self.indicator.interaction = Some((GROUP.into(), focus_handle.clone()));
        self.outer.child(
            self.inner
                .track_focus(&focus_handle)
                .group(GROUP)
                .child(self.indicator),
        )
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.outer.style()
    }
}

/// [`Switch`] の見た目。押せず、フォーカスを受けず、支援技術にも公開されない
///
/// 大きさは 52×32dp のトラックで、状態レイヤーとフォーカスリングはトラックの外側にはみ出して描く。
/// つまみの動きの途中の状態は、要素 ID のスタックで親の [`Switch`] の `id` に結び付けて保つ
#[derive(IntoElement)]
struct SwitchIndicator {
    checked: bool,
    disabled: bool,
    check_icon: bool,
    theme: Option<SharedTheme>,
    /// hover を知る group の名前と、フォーカスを表示するためのフォーカス
    interaction: Option<(SharedString, FocusHandle)>,
}

impl SwitchIndicator {
    fn new(checked: bool) -> Self {
        Self {
            checked,
            disabled: false,
            check_icon: false,
            theme: None,
            interaction: None,
        }
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn check_icon(mut self, check_icon: bool) -> Self {
        self.check_icon = check_icon;
        self
    }

    fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for SwitchIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let checked = self.checked;
        let (group, focus_handle) = self.interaction.unzip();
        let colors = Colors::resolve(
            &theme,
            checked,
            self.disabled,
            focus_handle
                .as_ref()
                .is_some_and(|focus_handle| focus_handle.is_focused(window)),
            focus_handle.as_ref().is_some_and(|focus_handle| {
                gpui_headless::is_focus_visible(focus_handle, window, cx)
            }),
        );
        let icon = (self.check_icon && checked).then_some(colors.icon);

        let track_size = size(dp(switch::TRACK_WIDTH), dp(switch::TRACK_HEIGHT));
        let track_radii = theme.corner_radii(switch::TRACK_SHAPE, track_size);
        // つまみの位置はトラックの外縁から測るため、枠線は子の配置に影響しない別の層に描く
        let track = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .rounded_tl(track_radii.top_left)
            .rounded_tr(track_radii.top_right)
            .rounded_br(track_radii.bottom_right)
            .rounded_bl(track_radii.bottom_left)
            .bg(colors.track)
            .when_some(colors.outline, |this, outline| {
                this.border(dp(switch::TRACK_OUTLINE_WIDTH))
                    .border_color(outline)
            });

        // Compose はつまみの大きさと位置を FastSpatial のバネで動かす
        let spring = theme.spring(MotionSchemeKey::FastSpatial);
        let (diameter, offset) = thumb_geometry(checked);
        let layer_size = dp(switch::STATE_LAYER_SIZE);
        let layer_radii =
            theme.corner_radii(switch::STATE_LAYER_SHAPE, size(layer_size, layer_size));
        // 状態レイヤーはつまみの中心に置く、境界のないリップル (`ripple(bounded = false)`)
        let state_layer = div()
            .absolute()
            .size(layer_size)
            .rounded_tl(layer_radii.top_left)
            .rounded_tr(layer_radii.top_right)
            .rounded_br(layer_radii.bottom_right)
            .rounded_bl(layer_radii.bottom_left)
            .when_some(colors.focus_state_layer, |this, layer| this.bg(layer))
            .when_some(
                group.clone().zip(colors.hover_state_layer),
                |this, (group, layer)| this.group_hover(group, |style| style.bg(layer)),
            );
        let handle = div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.thumb)
            .when_some(group.zip(colors.hover_thumb), |this, (group, thumb)| {
                this.group_hover(group, |style| style.bg(thumb))
            })
            .when_some(icon, |this, icon| {
                this.child(
                    svg()
                        .data(md3_icons::CHECK.svg(IconStyle::Outlined, false).as_bytes())
                        .flex_none()
                        .size(dp(switch::SELECTED_ICON_SIZE))
                        .text_color(icon),
                )
            });
        let thumb_theme = theme.clone();
        let thumb = div().relative().flex_none().with_spring(
            "thumb-diameter",
            SpringAnimation::new(spring).to(dp(diameter)),
            move |this, diameter| {
                let radii =
                    thumb_theme.corner_radii(switch::HANDLE_SHAPE, size(diameter, diameter));
                let layer_offset = (diameter - layer_size) / 2.0;
                this.size(diameter)
                    .child(state_layer.top(layer_offset).left(layer_offset))
                    .child(
                        handle
                            .rounded_tl(radii.top_left)
                            .rounded_tr(radii.top_right)
                            .rounded_br(radii.bottom_right)
                            .rounded_bl(radii.bottom_left),
                    )
            },
        );
        let thumb = div()
            .absolute()
            .top_0()
            .bottom_0()
            .flex()
            .items_center()
            .child(thumb)
            .with_spring(
                "thumb-offset",
                SpringAnimation::new(spring).to(dp(offset)),
                |this, offset| this.left(offset),
            );

        div()
            .relative()
            .flex_none()
            .w(track_size.width)
            .h(track_size.height)
            .child(track)
            .child(thumb)
            .when_some(colors.focus_ring, |this, ring| {
                // Material Web はトラックの形のリングを、トラックから外側へ 2px 離して太さ 3px で描く
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                this.child(
                    div()
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .right(-outset)
                        .bottom(-outset)
                        .rounded_tl(track_radii.top_left + outset)
                        .rounded_tr(track_radii.top_right + outset)
                        .rounded_br(track_radii.bottom_right + outset)
                        .rounded_bl(track_radii.bottom_left + outset)
                        .border(dp(focus_ring::WIDTH))
                        .border_color(ring),
                )
            })
    }
}

/// つまみの直径と、トラックの左端からつまみの左端までの距離 (`ThumbNode.measure`)
///
/// アイコンはオンのときだけ出すため、つまみに内容があるときの大きさ (`ThumbDiameter`) はオンの大きさと変わらない。
/// 押下中の大きさと位置は扱わない
fn thumb_geometry(checked: bool) -> (Dp, Dp) {
    if checked {
        // `ThumbPadding` と `thumbPathLength`。どちらも `ThumbDiameter` (オンの大きさ) を基準にする
        let thumb_padding = (switch::TRACK_HEIGHT.0 - switch::SELECTED_HANDLE_WIDTH.0) / 2.0;
        let max_bound = switch::TRACK_WIDTH.0 - switch::SELECTED_HANDLE_WIDTH.0 - thumb_padding;
        (switch::SELECTED_HANDLE_WIDTH, Dp(max_bound))
    } else {
        let min_bound = (switch::TRACK_HEIGHT.0 - switch::UNSELECTED_HANDLE_WIDTH.0) / 2.0;
        (switch::UNSELECTED_HANDLE_WIDTH, Dp(min_bound))
    }
}

/// 状態から決まる描画の色
struct Colors {
    /// トラックの塗り
    track: Hsla,
    /// トラックの枠線。`None` は枠線を付けない
    outline: Option<Hsla>,
    /// つまみ
    thumb: Hsla,
    /// hover 中のつまみ。`None` は hover で変えない
    hover_thumb: Option<Hsla>,
    /// つまみのアイコン
    icon: Hsla,
    /// フォーカスの状態レイヤー。`None` は描かない
    focus_state_layer: Option<Hsla>,
    /// hover の状態レイヤー。`None` は描かない
    hover_state_layer: Option<Hsla>,
    /// フォーカスリング。`None` は描かない
    focus_ring: Option<Hsla>,
}

impl Colors {
    // `defaultSwitchColors` の色の選択。オンの枠線は透明のため付けない。
    // オフの枠線は Compose と同じく `UnselectedFocusTrackOutlineColor` を使う
    fn resolve(
        theme: &Theme,
        checked: bool,
        disabled: bool,
        focused: bool,
        focus_visible: bool,
    ) -> Self {
        // Compose は無効の色を surface に合成して不透明にする (`compositeOver(surface)`)
        let surface = theme.color(ColorRole::Surface);
        let disabled_color =
            |role: ColorRole, opacity: f32| surface.blend(theme.color(role).opacity(opacity));
        let (track, outline, thumb, icon) = match (checked, disabled) {
            (true, false) => (
                theme.color(switch::SELECTED_TRACK_COLOR),
                None,
                theme.color(switch::SELECTED_HANDLE_COLOR),
                theme.color(switch::SELECTED_ICON_COLOR),
            ),
            (false, false) => (
                theme.color(switch::UNSELECTED_TRACK_COLOR),
                Some(theme.color(switch::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR)),
                theme.color(switch::UNSELECTED_HANDLE_COLOR),
                theme.color(switch::UNSELECTED_ICON_COLOR),
            ),
            (true, true) => (
                disabled_color(
                    switch::DISABLED_SELECTED_TRACK_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                ),
                None,
                disabled_color(
                    switch::DISABLED_SELECTED_HANDLE_COLOR,
                    switch::DISABLED_SELECTED_HANDLE_OPACITY,
                ),
                disabled_color(
                    switch::DISABLED_SELECTED_ICON_COLOR,
                    switch::DISABLED_SELECTED_ICON_OPACITY,
                ),
            ),
            (false, true) => (
                disabled_color(
                    switch::DISABLED_UNSELECTED_TRACK_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                ),
                Some(disabled_color(
                    switch::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                )),
                disabled_color(
                    switch::DISABLED_UNSELECTED_HANDLE_COLOR,
                    switch::DISABLED_UNSELECTED_HANDLE_OPACITY,
                ),
                disabled_color(
                    switch::DISABLED_UNSELECTED_ICON_COLOR,
                    switch::DISABLED_UNSELECTED_ICON_OPACITY,
                ),
            ),
        };
        let (hover_thumb, focus_thumb, hover_layer, focus_layer) = if checked {
            (
                switch::SELECTED_HOVER_HANDLE_COLOR,
                switch::SELECTED_FOCUS_HANDLE_COLOR,
                supplement::switch::SELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::switch::SELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        } else {
            (
                switch::UNSELECTED_HOVER_HANDLE_COLOR,
                switch::UNSELECTED_FOCUS_HANDLE_COLOR,
                supplement::switch::UNSELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::switch::UNSELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        };
        // Material Web はリングの有無にかかわらず、フォーカスがあればつまみをフォーカスの色にする (`:focus-within`)。
        // フォーカスの色は hover の色より優先する
        let focused = !disabled && focused;
        Self {
            track,
            outline,
            thumb: if focused {
                theme.color(focus_thumb)
            } else {
                thumb
            },
            hover_thumb: (!disabled && !focused).then(|| theme.color(hover_thumb)),
            icon,
            focus_state_layer: (!disabled && focus_visible).then(|| {
                theme
                    .color(focus_layer)
                    .opacity(state::FOCUS_STATE_LAYER_OPACITY)
            }),
            hover_state_layer: (!disabled).then(|| {
                theme
                    .color(hover_layer)
                    .opacity(state::HOVER_STATE_LAYER_OPACITY)
            }),
            focus_ring: focus_visible.then(|| theme.color(switch::FOCUS_INDICATOR_COLOR)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        AnyElement, Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point,
        px,
    };

    use super::*;

    /// 期待値は Compose の `ThumbNode.measure` の式に、SwitchTokens の値
    /// (トラック 52×32dp、つまみ 16 / 24dp) を手で代入して求めた
    #[test]
    fn つまみの大きさと位置が_compose_と一致する() {
        assert_eq!(thumb_geometry(false), (Dp(16.0), Dp(8.0)), "オフ");
        assert_eq!(thumb_geometry(true), (Dp(24.0), Dp(24.0)), "オン");
    }

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx
    }

    /// 描いた矩形の左上の座標と幅 (論理ピクセル)
    fn quad_rects(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                    )
                })
                .collect()
        })
    }

    #[gpui::test]
    fn つまみは状態に応じた位置と大きさで描かれる(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for (checked, thumb) in [(false, (8., 8., 16.)), (true, (24., 4., 24.))] {
            let cx = add_view(cx, move || {
                Switch::new("switch", checked)
                    .aria_label("Wi-Fi")
                    .into_any_element()
            });
            assert!(
                quad_rects(cx).contains(&thumb),
                "checked={checked} のつまみが {thumb:?} に描かれること"
            );
        }
    }

    /// `element` だけを描いたときの矩形。ポインターは要素の外に置く
    fn painted_quads(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> String {
        let cx = add_view(cx, element);
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        cx.update(|window, _| format!("{:?}", window.painted_quads()))
    }

    #[gpui::test]
    fn switch_は状態と無効を見た目に渡す(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for checked in [false, true] {
            for disabled in [false, true] {
                let switch = painted_quads(cx, move || {
                    Switch::new("switch", checked)
                        .aria_label("Wi-Fi")
                        .disabled(disabled)
                        .into_any_element()
                });
                let indicator = painted_quads(cx, move || {
                    SwitchIndicator::new(checked)
                        .disabled(disabled)
                        .into_any_element()
                });
                assert_eq!(
                    switch, indicator,
                    "checked={checked} disabled={disabled} で同じ見た目になること"
                );
            }
        }
    }

    // アイコンは矩形ではなく sprite として描かれ、テストから描画結果を取り出せないため、受け渡しだけを確かめる
    #[test]
    fn switch_はアイコンの有無を見た目に渡す() {
        for check_icon in [false, true] {
            let switch = Switch::new("switch", true).check_icon(check_icon);
            assert_eq!(switch.indicator.check_icon, check_icon);
        }
    }

    struct SwitchView {
        checked: bool,
        changes: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for SwitchView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            Switch::new("switch", self.checked)
                .aria_label("Wi-Fi")
                .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                    changes.borrow_mut().push(*checked);
                    this.checked = *checked;
                    cx.notify();
                }))
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<SwitchView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            SwitchView {
                checked: false,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        (view, cx)
    }

    #[gpui::test]
    fn 押すと反転した状態をハンドラに渡す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        cx.simulate_click(point(px(26.), px(16.)), Modifiers::none());
        cx.simulate_click(point(px(26.), px(16.)), Modifiers::none());
        let changes = view.read_with(cx, |view, _| view.changes.borrow().clone());
        assert_eq!(changes, vec![true, false]);
    }

    #[gpui::test]
    fn hover_中はつまみの中心に状態レイヤーを描く(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx);
        // 40px の状態レイヤーは、左上 (8, 8) にある 16px のつまみと中心をそろえる
        let layer = (-4., -4., 40.);
        cx.simulate_mouse_move(point(px(26.), px(16.)), None, Modifiers::none());
        assert!(quad_rects(cx).contains(&layer), "トラックに乗ると描くこと");
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        assert!(
            !quad_rects(cx).contains(&layer),
            "トラックから外れると消すこと"
        );
    }

    #[gpui::test]
    fn tab_のフォーカスをトラックの外側のリングで示す(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        // リングは 52×32px のトラックの外側 5px に描く
        let ring = (-5., -5., 62.);
        assert!(
            !quad_rects(cx).contains(&ring),
            "フォーカスの前はリングを描かないこと"
        );
        cx.simulate_keystrokes("tab");
        assert!(
            quad_rects(cx).contains(&ring),
            "フォーカスが移るとリングを描くこと"
        );
    }

    // gpui のバネは実時間 (`Instant::now`) で進み、テストの時計では進められないため、
    // 途中の位置は確かめず、十分な時間の後に着くことだけを確かめる
    #[gpui::test]
    fn 状態が変わるとつまみは新しい位置まで動く(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx);
        view.update(cx, |view, cx| {
            view.checked = true;
            cx.notify();
        });
        cx.run_until_parked();
        // FastSpatial のバネは 1 秒以内に収まる
        std::thread::sleep(std::time::Duration::from_secs(1));
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        assert!(quad_rects(cx).contains(&(24., 4., 24.)));
    }

    #[gpui::test]
    fn モーションを減らす設定ではつまみをすぐに新しい位置に描く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        cx.update(|_, cx| cx.set_reduce_motion(true));
        view.update(cx, |view, cx| {
            view.checked = true;
            cx.notify();
        });
        cx.run_until_parked();
        assert!(quad_rects(cx).contains(&(24., 4., 24.)));
    }
}
