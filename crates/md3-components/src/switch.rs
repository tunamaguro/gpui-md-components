//! MD3 のスイッチ
//!
//! 寸法・つまみの動きは Compose Material3 の `Switch.kt` (@ 2a941dd11aabd9bb7d2016ed159764b220117c20) に、
//! 状態ごとの色は生成トークン ([`switch`]) に従う。状態レイヤーの色は Compose に無いため Material Web のトークン
//! ([`web_component::switch`]) に従う

use gpui::{
    AnimationExt as _, App, ElementId, IntoElement, ParentElement as _, RenderOnce, SharedString,
    SpringAnimation, SpringPlayback, StatefulInteractiveElement as _, Styled as _, Window, div,
    size, transparent_black,
};
use md3_icons::IconName;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::switch;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::web_component;
use md3_theme::{ThemeExt as _, dp};

use crate::indication::{self, Indication};
use crate::{MIN_INTERACTIVE_SIZE, icon};

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// MD3 のスイッチ
///
/// 状態は呼び出し側が持ち、[`Switch::new`] に毎回渡す。スイッチ自身は状態を変えない。
/// 支援技術には role `switch` と、[`Switch::new`] に渡したラベルを名前として公開する。
/// マウスのクリック・フォーカス中の Space・支援技術からのクリック操作で [`Switch::on_change`] のハンドラを呼び、
/// Enter では呼ばない。操作できる範囲は 48dp の高さを下回らない
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    selected: bool,
    label: SharedString,
    icons: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Switch {
    /// `selected` がオンかオフかを表示するスイッチを作る。`label` は表示せず、支援技術に名前として公開する
    ///
    /// `label` は状態によって変えないこと。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// フォーカスと押下の表示が失われ、つまみの動きが途中で止まる
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            icons: false,
            disabled: false,
            on_change: None,
        }
    }

    /// `true` を渡すと、つまみにオンでは check、オフでは close のアイコンを表示する。既定では表示しない
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// スイッチを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なスイッチは操作されてもハンドラを呼ばず、Tab で到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在と逆の状態を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Switch::new`] に渡すこと。
    /// 設定しない場合、スイッチは操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.md3_theme().clone();
        let enabled = !self.disabled;
        let indication = Indication::new(&self.id, enabled, window, cx);
        let selected = self.selected;
        let pressed = indication.pressed;
        let active = indication.hovered || indication.focus_visible || pressed;

        let (track, outline, handle, icon_color) = if !enabled {
            // Compose の SwitchDefaults.colors と同じく、無効な色は Surface に重ねた不透明な色にする
            let surface = theme.color(ColorRole::Surface);
            let over = |role, opacity| surface.blend(theme.color(role).opacity(opacity));
            if selected {
                (
                    over(
                        switch::DISABLED_SELECTED_TRACK_COLOR,
                        switch::DISABLED_TRACK_OPACITY,
                    ),
                    None,
                    over(
                        switch::DISABLED_SELECTED_HANDLE_COLOR,
                        switch::DISABLED_SELECTED_HANDLE_OPACITY,
                    ),
                    over(
                        switch::DISABLED_SELECTED_ICON_COLOR,
                        switch::DISABLED_SELECTED_ICON_OPACITY,
                    ),
                )
            } else {
                (
                    over(
                        switch::DISABLED_UNSELECTED_TRACK_COLOR,
                        switch::DISABLED_TRACK_OPACITY,
                    ),
                    Some(over(
                        switch::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR,
                        switch::DISABLED_TRACK_OPACITY,
                    )),
                    over(
                        switch::DISABLED_UNSELECTED_HANDLE_COLOR,
                        switch::DISABLED_UNSELECTED_HANDLE_OPACITY,
                    ),
                    over(
                        switch::DISABLED_UNSELECTED_ICON_COLOR,
                        switch::DISABLED_UNSELECTED_ICON_OPACITY,
                    ),
                )
            }
        } else if selected {
            (
                theme.color(switch::SELECTED_TRACK_COLOR),
                None,
                theme.color(if active {
                    switch::SELECTED_HOVER_HANDLE_COLOR
                } else {
                    switch::SELECTED_HANDLE_COLOR
                }),
                theme.color(switch::SELECTED_ICON_COLOR),
            )
        } else {
            (
                theme.color(switch::UNSELECTED_TRACK_COLOR),
                Some(theme.color(switch::UNSELECTED_TRACK_OUTLINE_COLOR)),
                theme.color(if active {
                    switch::UNSELECTED_HOVER_HANDLE_COLOR
                } else {
                    switch::UNSELECTED_HANDLE_COLOR
                }),
                theme.color(switch::UNSELECTED_ICON_COLOR),
            )
        };
        let layer_color = theme.color(if selected {
            web_component::switch::SELECTED_STATE_LAYER_COLOR
        } else {
            web_component::switch::UNSELECTED_STATE_LAYER_COLOR
        });

        // つまみの大きさと位置は Compose の ThumbNode と同じく決める
        let track_width = dp(switch::TRACK_WIDTH);
        let track_height = dp(switch::TRACK_HEIGHT);
        let outline_width = dp(switch::TRACK_OUTLINE_WIDTH);
        let thumb_diameter = dp(switch::SELECTED_HANDLE_WIDTH);
        let thumb_size = if pressed {
            dp(switch::PRESSED_HANDLE_WIDTH)
        } else if self.icons || selected {
            thumb_diameter
        } else {
            dp(switch::UNSELECTED_HANDLE_WIDTH)
        };
        let max_offset = track_width - thumb_diameter - (track_height - thumb_diameter) / 2.0;
        let offset = match (pressed, selected) {
            (true, true) => max_offset - outline_width,
            (true, false) => outline_width,
            (false, true) => max_offset,
            (false, false) => (track_height - thumb_size) / 2.0,
        };
        // Compose は位置と大きさを同じバネで別々に動かす。バネは線形なので、中心 (位置 + 大きさ / 2) も同じバネで動く
        let center = offset + thumb_size / 2.0;
        let spring = theme.spring(MotionSchemeKey::FastSpatial);
        // 押した瞬間は動かさずに押下の形にする
        let playback = if pressed {
            SpringPlayback::Completed
        } else {
            SpringPlayback::Running
        };

        let thumb_icon = self.icons.then(|| {
            icon(
                if selected {
                    IconName::Check
                } else {
                    IconName::Close
                },
                dp(switch::SELECTED_ICON_SIZE),
                icon_color,
            )
        });
        let thumb = div()
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .rounded_full()
            .bg(handle)
            .children(thumb_icon)
            .with_spring(
                "md3-switch-thumb-size",
                SpringAnimation::new(spring)
                    .to(thumb_size)
                    .playback(playback),
                |this, thumb_size| this.size(thumb_size),
            );

        let layer_size = dp(switch::STATE_LAYER_SIZE);
        let layer_radii =
            theme.corner_radii(switch::STATE_LAYER_SHAPE, size(layer_size, layer_size));
        let handle_container = div()
            .absolute()
            .top((track_height - layer_size) / 2.0)
            .flex()
            .items_center()
            .justify_center()
            .size(layer_size);
        let handle_container = indication
            .layers(handle_container, layer_color, layer_color, layer_radii)
            .child(thumb)
            .with_spring(
                "md3-switch-thumb-center",
                SpringAnimation::new(spring).to(center).playback(playback),
                move |this, center| this.left(center - layer_size / 2.0),
            );

        let track_radii = theme.corner_radii(switch::TRACK_SHAPE, size(track_width, track_height));
        let track = indication::with_corners(
            div()
                .relative()
                .flex_none()
                .w(track_width)
                .h(track_height)
                .bg(track)
                .border(outline_width)
                .border_color(outline.unwrap_or(transparent_black())),
            track_radii,
        )
        .child(handle_container);
        // Material Web の md-switch と同じく、フォーカスリングはつまみではなくトラックの外側に出す
        let track = indication.focus_ring(track, &theme, track_radii);

        let mut element = gpui_headless::choice::Switch::new(self.id, selected)
            .track_focus(indication.focus_handle())
            .disabled(self.disabled)
            .aria_label(self.label)
            .flex()
            .items_center()
            .justify_center()
            .min_h(dp(MIN_INTERACTIVE_SIZE))
            .child(track);
        if let Some(handler) = self.on_change {
            element = element.on_change(handler);
        }
        indication.track(element, &["space"])
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Entity, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;
    use crate::test_support::{focus_next, init_theme, press_key};

    struct SwitchView {
        selected: bool,
        disabled: bool,
        changes: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for SwitchView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let entity = cx.entity();
            div().size_full().flex().items_start().child(
                Switch::new("switch", self.selected, "Wi-Fi")
                    .icons(true)
                    .disabled(self.disabled)
                    .on_change(move |selected, _, cx| {
                        changes.borrow_mut().push(selected);
                        entity.update(cx, |view, cx| {
                            view.selected = selected;
                            cx.notify();
                        });
                    }),
            )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Entity<SwitchView>, &mut VisualTestContext) {
        init_theme(cx);
        cx.add_window_view(|_, _| SwitchView {
            selected: false,
            disabled,
            changes: Rc::default(),
        })
    }

    fn changes(view: &Entity<SwitchView>, cx: &mut VisualTestContext) -> Vec<bool> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    #[gpui::test]
    fn 押すたびにオンとオフを切り替える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false);
        // トラックは幅 52dp・高さ 32dp で、操作できる範囲の高さは 48dp に広がる
        cx.simulate_click(point(px(26.0), px(2.0)), Modifiers::none());
        cx.simulate_click(point(px(50.0), px(46.0)), Modifiers::none());
        cx.simulate_click(point(px(26.0), px(52.0)), Modifiers::none());
        cx.simulate_click(point(px(56.0), px(24.0)), Modifiers::none());
        assert_eq!(
            changes(&view, cx),
            [true, false],
            "トラックの幅と 48dp の高さの外は押せないこと"
        );
    }

    #[gpui::test]
    fn space_で切り替わり_enter_では切り替わらない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, false);
        focus_next(cx);
        press_key(cx, "enter");
        press_key(cx, "space");
        assert_eq!(changes(&view, cx), [true]);
    }

    #[gpui::test]
    fn 無効なスイッチは_tab_で到達できず切り替わらない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, true);
        focus_next(cx);
        press_key(cx, "space");
        cx.simulate_click(point(px(26.0), px(24.0)), Modifiers::none());
        assert!(changes(&view, cx).is_empty());
    }
}
