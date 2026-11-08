// 寸法・色の選択・つまみの位置と押下時の動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt
//         (`SwitchImpl`、`ThumbNode`、`SwitchDefaults`、`SwitchColors`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある
//
// hover・フォーカス・押下時のつまみの色と状態レイヤーの色は material-web から移植している
// (https://github.com/material-components/material-web) @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Source: switch/internal/_handle.scss (状態ごとのつまみの色の選択)
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// Source: tokens/versions/v0_192/_md-comp-switch.scss (状態レイヤーの色)
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! MD3 のスイッチ

use gpui::{
    AnimationExt, App, Div, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, SpringAnimation, SpringPlayback,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, size,
    svg,
};
use gpui_headless::choice::Switch as HeadlessSwitch;

use crate::shape::Shaped;
use md3_icons::IconStyle;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::switch;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層がスイッチの hover を知るための group 名
const GROUP: &str = "md3-switch";

/// `Modifier.minimumInteractiveComponentSize` が使う `LocalMinimumInteractiveComponentSize` の既定値
const MIN_INTERACTIVE_SIZE: Dp = Dp(48.0);

/// material-web の `selected-{hover,pressed}-state-layer-color`。Compose の SwitchTokens には無い
const SELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
/// material-web の `unselected-{hover,pressed}-state-layer-color`
const UNSELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;

/// つまみの直径と、トラックの左端からつまみの左端までの距離 (`ThumbNode.measure`)
fn thumb_geometry(checked: bool, pressed: bool) -> (Dp, Dp) {
    // つまみに内容があるときの大きさは `ThumbDiameter` だが、
    // アイコンはオンのときだけ出すため、オンの大きさと変わらない
    let diameter = if pressed {
        switch::PRESSED_HANDLE_WIDTH.0
    } else if checked {
        switch::SELECTED_HANDLE_WIDTH.0
    } else {
        switch::UNSELECTED_HANDLE_WIDTH.0
    };
    // `ThumbPadding` と `thumbPathLength`。どちらも `ThumbDiameter` を基準にする
    let thumb_padding = (switch::TRACK_HEIGHT.0 - switch::SELECTED_HANDLE_WIDTH.0) / 2.0;
    let max_bound = switch::TRACK_WIDTH.0 - switch::SELECTED_HANDLE_WIDTH.0 - thumb_padding;
    let offset = match (pressed, checked) {
        (true, true) => max_bound - switch::TRACK_OUTLINE_WIDTH.0,
        (true, false) => switch::TRACK_OUTLINE_WIDTH.0,
        (false, true) => max_bound,
        (false, false) => (switch::TRACK_HEIGHT.0 - diameter) / 2.0,
    };
    (Dp(diameter), Dp(offset))
}

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// MD3 のスイッチ
///
/// オンかオフかを、トラックとその上を動くつまみで描く。状態は呼び出し側が持ち、[`Switch::new`] に毎回渡す。
/// マウスのクリック・フォーカス中の Space で [`Switch::on_change`] のハンドラを呼び、Enter では呼ばない。
/// 支援技術には role `switch` と、オンかオフかの状態として公開される。
/// アクセシブルな名前は [`Switch::aria_label`] で必ず与えること
///
/// [`Styled`] で指定したスタイルはスイッチの外側の箱に付く。外側の余白や配置の指定に使う。
/// 大きさは 52×32dp のトラックで決まり、押せる範囲はトラックを中央に置いた高さ 48dp の矩形になる
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    outer: Div,
    checked: bool,
    check_icon: bool,
    on_change: Option<ChangeHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Switch {
    /// `checked` を表示するスイッチを作る。`true` がオン
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Switch::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            checked,
            check_icon: false,
            on_change: None,
            aria_label: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態を反転した値を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Switch::new`] に渡すこと。
    /// 設定しない場合、スイッチは操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// `label` をアクセシブルな名前にする
    ///
    /// スイッチは名前を持つ内容を描かないため、必ず指定すること。状態によって名前を変えないこと
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// `true` を渡すと、オンのときにつまみへチェックのアイコンを出す。既定では出さない
    ///
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn check_icon(mut self, check_icon: bool) -> Self {
        self.check_icon = check_icon;
        self
    }

    /// スイッチを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なスイッチは無効の色で描かれ、操作されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let checked = self.checked;
        let disabled = self.disabled;

        let focus_handle = self.focus_handle.unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        // リングはトラックの外側に描くため、スイッチ自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        let headless = HeadlessSwitch::new(self.id, checked)
            .track_focus(&focus_handle)
            .disabled(disabled);
        // `SwitchDefaults.colors`。無効の色は surface に合成して不透明にする
        let surface = theme.color(ColorRole::Surface);
        let color = |role: ColorRole| theme.color(role);
        let over_surface =
            |role: ColorRole, opacity: f32| surface.blend(color(role).opacity(opacity));
        let (track, border, thumb) = match (disabled, checked) {
            (false, true) => (
                color(switch::SELECTED_TRACK_COLOR),
                None,
                color(switch::SELECTED_HANDLE_COLOR),
            ),
            (false, false) => (
                color(switch::UNSELECTED_TRACK_COLOR),
                Some(color(switch::UNSELECTED_FOCUS_TRACK_OUTLINE_COLOR)),
                color(switch::UNSELECTED_HANDLE_COLOR),
            ),
            (true, true) => (
                over_surface(
                    switch::DISABLED_SELECTED_TRACK_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                ),
                None,
                over_surface(
                    switch::DISABLED_SELECTED_HANDLE_COLOR,
                    switch::DISABLED_SELECTED_HANDLE_OPACITY,
                ),
            ),
            (true, false) => (
                over_surface(
                    switch::DISABLED_UNSELECTED_TRACK_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                ),
                Some(over_surface(
                    switch::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR,
                    switch::DISABLED_TRACK_OPACITY,
                )),
                over_surface(
                    switch::DISABLED_UNSELECTED_HANDLE_COLOR,
                    switch::DISABLED_UNSELECTED_HANDLE_OPACITY,
                ),
            ),
        };
        let icon = (self.check_icon && checked).then(|| {
            if disabled {
                over_surface(
                    switch::DISABLED_SELECTED_ICON_COLOR,
                    switch::DISABLED_SELECTED_ICON_OPACITY,
                )
            } else {
                color(switch::SELECTED_ICON_COLOR)
            }
        });
        // hover・フォーカス・押下時のつまみの色と、状態レイヤーの色
        let (hovered_thumb, focused_thumb, pressed_thumb, state_layer) = if checked {
            (
                color(switch::SELECTED_HOVER_HANDLE_COLOR),
                color(switch::SELECTED_FOCUS_HANDLE_COLOR),
                color(switch::SELECTED_PRESSED_HANDLE_COLOR),
                color(SELECTED_STATE_LAYER_COLOR),
            )
        } else {
            (
                color(switch::UNSELECTED_HOVER_HANDLE_COLOR),
                color(switch::UNSELECTED_FOCUS_HANDLE_COLOR),
                color(switch::UNSELECTED_PRESSED_HANDLE_COLOR),
                color(UNSELECTED_STATE_LAYER_COLOR),
            )
        };
        // material-web はつまみの色を :focus-visible ではなく :focus-within で変える
        let focused = focus_handle.is_focused(window);

        // Compose は Enter でも押下を示すが、このスイッチは Enter で操作しないため、headless の押下に従う
        let track = move |pressed: bool, _: &mut Window, _: &mut App| {
            let thumb = match (disabled, pressed, focused) {
                (true, _, _) => thumb,
                (false, true, _) => pressed_thumb,
                (false, false, true) => focused_thumb,
                (false, false, false) => thumb,
            };

            // Compose は押下による変化を SnapSpec で即座に反映し、それ以外を FastSpatial のバネで動かす
            let spring = theme.spring(MotionSchemeKey::FastSpatial);
            let playback = if pressed {
                SpringPlayback::Completed
            } else {
                SpringPlayback::Running
            };
            let (diameter, offset) = thumb_geometry(checked, pressed);

            // 状態レイヤーはつまみの中心に置く、境界の無いリップル (`ripple(bounded = false)`)
            let layer =
                div()
                    .absolute()
                    .size(dp(switch::STATE_LAYER_SIZE))
                    .when(!disabled, |this| {
                        let hover = state_layer.opacity(state::HOVER_STATE_LAYER_OPACITY);
                        if pressed {
                            this.bg(state_layer.opacity(state::PRESSED_STATE_LAYER_OPACITY))
                        } else {
                            this.group_hover(GROUP, |style| style.bg(hover))
                        }
                    });
            let handle_theme = theme.clone();
            let handle = div().with_spring(
                "thumb-diameter",
                SpringAnimation::new(spring)
                    .to(dp(diameter))
                    .playback(playback),
                move |this, diameter| {
                    let layer_size = dp(switch::STATE_LAYER_SIZE);
                    let layer_radii = handle_theme
                        .corner_radii(switch::STATE_LAYER_SHAPE, size(layer_size, layer_size));
                    let radii =
                        handle_theme.corner_radii(switch::HANDLE_SHAPE, size(diameter, diameter));
                    let layer_offset = (diameter - layer_size) / 2.0;
                    this.relative()
                        .flex_none()
                        .size(diameter)
                        .child(
                            layer
                                .top(layer_offset)
                                .left(layer_offset)
                                .rounded_tl(layer_radii.top_left)
                                .rounded_tr(layer_radii.top_right)
                                .rounded_br(layer_radii.bottom_right)
                                .rounded_bl(layer_radii.bottom_left),
                        )
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .bottom_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(thumb)
                                .when(!disabled, |this| {
                                    this.group_hover(GROUP, |style| style.bg(hovered_thumb))
                                })
                                .rounded_tl(radii.top_left)
                                .rounded_tr(radii.top_right)
                                .rounded_br(radii.bottom_right)
                                .rounded_bl(radii.bottom_left)
                                .when_some(icon, |this, icon: Hsla| {
                                    this.child(
                                        svg()
                                            .data(
                                                md3_icons::CHECK
                                                    .svg(IconStyle::Outlined, false)
                                                    .as_bytes(),
                                            )
                                            .flex_none()
                                            .size(dp(switch::SELECTED_ICON_SIZE))
                                            .text_color(icon),
                                    )
                                }),
                        )
                },
            );
            let thumb = div()
                .absolute()
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .child(handle)
                .with_spring(
                    "thumb-offset",
                    SpringAnimation::new(spring)
                        .to(dp(offset))
                        .playback(playback),
                    |this, offset| this.left(offset),
                );

            div()
                .relative()
                .flex_none()
                .w(dp(switch::TRACK_WIDTH))
                .h(dp(switch::TRACK_HEIGHT))
                .child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(track)
                        .when_some(border, |this, border| {
                            this.border(dp(switch::TRACK_OUTLINE_WIDTH))
                                .border_color(border)
                        }),
                    theme.clone(),
                    switch::TRACK_SHAPE,
                ))
                .child(thumb)
                .when(focus_visible, |this| {
                    let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                    let ring = div()
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .right(-outset)
                        .bottom(-outset)
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(switch::FOCUS_INDICATOR_COLOR));
                    this.child(Shaped::new(ring, theme.clone(), switch::TRACK_SHAPE).outset(outset))
                })
        };

        let switch = headless
            .group(GROUP)
            .flex()
            .items_center()
            .justify_center()
            .min_w(dp(MIN_INTERACTIVE_SIZE))
            .min_h(dp(MIN_INTERACTIVE_SIZE))
            .child_with_press(track)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change));

        self.outer.child(switch)
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 期待値は Compose の `ThumbNode.measure` の式に、SwitchTokens の値
    /// (トラック 52×32dp、枠線 2dp、つまみ 16 / 24 / 28dp) を手で代入して求めた
    #[test]
    fn つまみの大きさと位置が_compose_と一致する() {
        let cases = [
            ((false, false), (16.0, 8.0)),
            ((true, false), (24.0, 24.0)),
            ((false, true), (28.0, 2.0)),
            ((true, true), (28.0, 22.0)),
        ];
        for ((checked, pressed), (diameter, offset)) in cases {
            assert_eq!(
                thumb_geometry(checked, pressed),
                (Dp(diameter), Dp(offset)),
                "checked: {checked}, pressed: {pressed}"
            );
        }
    }
}
