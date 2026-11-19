// 配置・余白・表示する長さ・モーションは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt
//         (`Snackbar`、`OneRowSnackbar`、`NewLineButtonSnackbar`、`SnackbarDefaults`) と
//         SnackbarHost.kt (`SnackbarHost`、`SnackbarHostState`、`SnackbarDuration`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のスナックバー

use gpui::{
    AnimationExt, App, AppContext, Div, ElementId, Entity, IntoElement, ParentElement, Refineable,
    RenderOnce, SharedString, SpringAnimation, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_headless::snackbar::{
    SnackbarDuration as HeadlessDuration, SnackbarQueue, SnackbarRegion,
    SnackbarResult as HeadlessResult,
};
use md3_theme::md3_tokens::component::snackbar;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::supplement::snackbar as supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::{Button, ButtonColors, ButtonVariant, IconButton, Tooltip};

/// スナックバーを表示し続ける長さ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SnackbarDuration {
    /// 4 秒
    Short,
    /// 10 秒
    Long,
    /// 閉じるか操作されるまで表示し続ける
    Indefinite,
}

/// スナックバーが表示を終えた理由
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SnackbarResult {
    /// 閉じられたか、表示する長さを過ぎた
    Dismissed,
    /// 操作のボタンが押された
    ActionPerformed,
}

/// 表示するスナックバーの内容
#[derive(Clone, Debug, PartialEq)]
pub struct SnackbarMessage {
    message: SharedString,
    action_label: Option<SharedString>,
    dismiss_label: Option<SharedString>,
    duration: Option<SnackbarDuration>,
    action_on_new_line: bool,
}

impl SnackbarMessage {
    /// 文 `message` だけのスナックバーを作る
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            action_label: None,
            dismiss_label: None,
            duration: None,
            action_on_new_line: false,
        }
    }

    /// 文の後ろに、`label` をラベルにした操作のボタンを置く
    ///
    /// 押されると表示を終え、[`SnackbarResult::ActionPerformed`] を伝える
    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    /// 末尾に閉じるボタンを置く。`label` はボタンのアクセシブルな名前とツールチップになる
    ///
    /// 押されると表示を終え、[`SnackbarResult::Dismissed`] を伝える
    pub fn dismiss_action(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = Some(label.into());
        self
    }

    /// 表示し続ける長さを設定する
    ///
    /// 設定しない場合、操作のボタンがあれば [`SnackbarDuration::Indefinite`]、無ければ [`SnackbarDuration::Short`]
    pub fn duration(mut self, duration: SnackbarDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// `true` なら、操作のボタンを文の下の行に置く。文が長い場合に使う
    pub fn action_on_new_line(mut self, action_on_new_line: bool) -> Self {
        self.action_on_new_line = action_on_new_line;
        self
    }
}

/// スナックバーを 1 つずつ表示する列への参照
///
/// [`SnackbarHost`] に渡して描く。複製しても同じ列を指す
#[derive(Clone)]
pub struct SnackbarHostState {
    queue: Entity<SnackbarQueue<SnackbarMessage>>,
}

impl SnackbarHostState {
    /// 空の列を作る
    pub fn new(cx: &mut App) -> Self {
        Self {
            queue: cx.new(|_| SnackbarQueue::new()),
        }
    }

    /// `message` を列の末尾に加える
    ///
    /// 表示中のスナックバーが無ければすぐに表示し、あれば表示を終えてから順に表示する。
    /// 表示する長さは表示し始めたときから数える。
    /// 表示を終えたときに、終えた理由を `on_result` に渡して 1 回だけ呼ぶ
    pub fn show(
        &self,
        message: SnackbarMessage,
        on_result: impl FnOnce(SnackbarResult, &mut App) + 'static,
        cx: &mut App,
    ) {
        let duration = match message.duration {
            Some(SnackbarDuration::Short) => HeadlessDuration::Short,
            Some(SnackbarDuration::Long) => HeadlessDuration::Long,
            Some(SnackbarDuration::Indefinite) => HeadlessDuration::Indefinite,
            None if message.action_label.is_some() => HeadlessDuration::Indefinite,
            None => HeadlessDuration::Short,
        };
        self.queue.update(cx, |queue, cx| {
            queue.show(
                message,
                duration,
                move |result, cx| {
                    on_result(
                        match result {
                            HeadlessResult::Dismissed => SnackbarResult::Dismissed,
                            HeadlessResult::ActionPerformed => SnackbarResult::ActionPerformed,
                        },
                        cx,
                    )
                },
                cx,
            )
        });
    }

    /// 表示中のスナックバーを閉じ、[`SnackbarResult::Dismissed`] を伝える。表示中のものが無ければ何もしない
    pub fn dismiss(&self, cx: &mut App) {
        self.queue.update(cx, |queue, cx| queue.dismiss(cx));
    }

    /// 表示中のスナックバーの内容。無ければ `None`
    pub fn current(&self, cx: &App) -> Option<SnackbarMessage> {
        self.queue.read(cx).current().cloned()
    }
}

/// [`SnackbarHostState`] の表示中のスナックバーを描く要素
///
/// スナックバーは幅 600dp までで、置いた場所の幅に広がる。周りに 12dp の余白を取る。
/// 表示し始めると不透明度を上げて現れ、表示を終えるとすぐに消える。
/// 画面の下端などへの配置は [`Styled`] で呼び出し側が付ける
///
/// 支援技術には role `status` の領域として公開され、表示したスナックバーの文を読み上げる。
/// 領域はスナックバーが無い間も描かれるため、表示する前から描いておくこと。
/// 操作のボタンと閉じるボタンは Tab で到達でき、フォーカスはスナックバーへ移さない
#[derive(IntoElement)]
pub struct SnackbarHost {
    id: ElementId,
    outer: Div,
    state: SnackbarHostState,
    theme: Option<SharedTheme>,
}

impl SnackbarHost {
    /// `state` のスナックバーを描く要素を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, state: &SnackbarHostState) -> Self {
        Self {
            id: id.into(),
            outer: div(),
            state: state.clone(),
            theme: None,
        }
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for SnackbarHost {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let queue = self.state.queue.read(cx);
        let current = queue.current().cloned().zip(queue.current_id());

        let snackbar = current.map(|(message, serial)| {
            let action_colors = ButtonColors {
                content: Some(snackbar::ACTION_LABEL_TEXT_COLOR),
                ..ButtonColors::default()
            };
            let has_dismiss = message.dismiss_label.is_some();
            let action = message.action_label.clone().map(|label| {
                let queue = self.state.queue.clone();
                Button::new("action")
                    .variant(ButtonVariant::Text)
                    .colors(action_colors)
                    .theme(theme.clone())
                    .on_click(move |_, _, cx| {
                        queue.update(cx, |queue, cx| queue.perform_action(cx))
                    })
                    .child(gpui::text!(id = "action-label", label))
            });
            let dismiss = message.dismiss_label.clone().map(|label| {
                let queue = self.state.queue.clone();
                IconButton::new("dismiss", md3_icons::CLOSE, label.clone())
                    .colors(ButtonColors {
                        content: Some(snackbar::ICON_COLOR),
                        ..ButtonColors::default()
                    })
                    .tooltip(Tooltip::new(label))
                    .theme(theme.clone())
                    .on_click(move |_, _, cx| queue.update(cx, |queue, cx| queue.dismiss(cx)))
            });
            let text = div()
                .py(dp(supplement::VERTICAL_PADDING))
                .child(gpui::text!(id = "message", message.message.clone()));

            let mut body = div()
                .w_full()
                .max_w(dp(supplement::CONTAINER_MAX_WIDTH))
                .min_h(dp(snackbar::SINGLE_LINE_CONTAINER_HEIGHT))
                .relative()
                .text_color(theme.color(snackbar::SUPPORTING_TEXT_COLOR))
                .child(Shaped::new(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(theme.color(snackbar::CONTAINER_COLOR))
                        .shadow(theme.shadow(snackbar::CONTAINER_ELEVATION)),
                    theme.clone(),
                    snackbar::CONTAINER_SHAPE,
                ));
            body.text_style()
                .refine(&theme.text_style(snackbar::SUPPORTING_TEXT_FONT));
            let body = if message.action_on_new_line && action.is_some() {
                // `NewLineButtonSnackbar`。文の下の行の末尾にボタンを並べる
                body.flex()
                    .flex_col()
                    .pl(dp(supplement::HORIZONTAL_SPACING))
                    .child(text.relative().pr(dp(supplement::HORIZONTAL_SPACING)))
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .self_end()
                            .pb(dp(supplement::ACTION_BUTTON_BOTTOM_PADDING))
                            .when(!has_dismiss, |this| {
                                this.pr(dp(supplement::HORIZONTAL_SPACING_BUTTON_SIDE))
                            })
                            .children(action)
                            .children(dismiss),
                    )
            } else {
                // `OneRowSnackbar`。文は残りの幅で折り返し、ボタンは上下の中央にそろえる
                body.flex()
                    .items_center()
                    .pl(dp(supplement::HORIZONTAL_SPACING))
                    .when(!has_dismiss, |this| {
                        this.pr(dp(supplement::TEXT_END_EXTRA_SPACING))
                    })
                    .child(text.relative().flex_1().min_w_0())
                    .children(action.map(|action| action.relative().flex_none()))
                    .children(dismiss.map(|dismiss| dismiss.relative().flex_none()))
            };
            // 表示し始めるたびに不透明度を 0 から上げる。番号で表示ごとに別の動きにする
            div()
                .p(dp(supplement::HOST_PADDING))
                .child(body)
                .with_spring(
                    ElementId::NamedInteger("snackbar".into(), serial),
                    SpringAnimation::new(theme.spring(MotionSchemeKey::FastEffects))
                        .to(1.0_f32)
                        .from(0.0),
                    |this, opacity| this.opacity(opacity.clamp(0.0, 1.0)),
                )
        });

        self.outer
            .child(SnackbarRegion::new(self.id).children(snackbar))
    }
}

impl Styled for SnackbarHost {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    struct HostView(SnackbarHostState);

    impl Render for HostView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // 原点のマウスの位置で hover にならないよう、原点から離す
            div()
                .pl(px(100.0))
                .pt(px(100.0))
                .w(px(900.0))
                .child(SnackbarHost::new("host", &self.0))
        }
    }

    fn open(cx: &mut TestAppContext) -> (SnackbarHostState, &mut VisualTestContext) {
        cx.update(crate::init);
        let state = cx.update(SnackbarHostState::new);
        let host = state.clone();
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            HostView(host)
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.run_until_parked();
        (state, cx)
    }

    /// 容器の x・y・幅・高さ (ホストの左上から)
    fn containers(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let background: gpui::Background =
                cx.md3_theme().color(ColorRole::InverseSurface).into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background)
                .map(|quad| {
                    (
                        quad.bounds.origin.x.0 / scale - 100.0,
                        quad.bounds.origin.y.0 / scale - 100.0,
                        quad.bounds.size.width.0 / scale,
                        quad.bounds.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    type Results = Rc<RefCell<Vec<SnackbarResult>>>;

    fn show(
        state: &SnackbarHostState,
        message: SnackbarMessage,
        results: &Results,
        cx: &mut VisualTestContext,
    ) {
        let results = results.clone();
        cx.update(|_, cx| {
            state.show(
                message,
                move |result, _| results.borrow_mut().push(result),
                cx,
            )
        });
        cx.run_until_parked();
    }

    fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
        cx.simulate_click(point(px(100.0 + x), px(100.0 + y)), Modifiers::none());
        cx.simulate_mouse_move(point(px(10.0), px(10.0)), None, Modifiers::none());
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 文だけのスナックバーは周りに余白を取り_最大の幅で描いて_4_秒で閉じる(
        cx: &mut TestAppContext,
    ) {
        let (state, cx) = open(cx);
        assert_eq!(containers(cx), vec![], "表示する前は描かないこと");
        let results = Results::default();
        show(&state, SnackbarMessage::new("Saved"), &results, cx);
        assert_eq!(containers(cx), vec![(12.0, 12.0, 600.0, 48.0)]);

        cx.executor().advance_clock(Duration::from_millis(4000));
        cx.run_until_parked();
        assert_eq!(containers(cx), vec![]);
        assert_eq!(*results.borrow(), vec![SnackbarResult::Dismissed]);
    }

    #[gpui::test]
    fn 操作のボタンを押すと閉じて伝え_次のスナックバーを表示する(
        cx: &mut TestAppContext,
    ) {
        let (state, cx) = open(cx);
        let results = Results::default();
        show(
            &state,
            SnackbarMessage::new("Deleted").action("Undo"),
            &results,
            cx,
        );
        show(&state, SnackbarMessage::new("Next"), &results, cx);
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(
            cx.update(|_, cx| state.current(cx).map(|message| message.message)),
            Some("Deleted".into()),
            "操作のボタンがあれば閉じるまで表示し続けること"
        );
        // 操作のボタンは右端から 8dp 内側にある
        click(cx, 12.0 + 600.0 - 8.0 - 10.0, 12.0 + 24.0);
        assert_eq!(*results.borrow(), vec![SnackbarResult::ActionPerformed]);
        assert_eq!(
            cx.update(|_, cx| state.current(cx).map(|message| message.message)),
            Some("Next".into())
        );
    }

    #[gpui::test]
    fn 閉じるボタンを押すと閉じたことを伝える(cx: &mut TestAppContext) {
        let (state, cx) = open(cx);
        let results = Results::default();
        show(
            &state,
            SnackbarMessage::new("Saved").dismiss_action("Dismiss"),
            &results,
            cx,
        );
        // 閉じるボタンは右端にある
        click(cx, 12.0 + 600.0 - 20.0, 12.0 + 24.0);
        assert_eq!(*results.borrow(), vec![SnackbarResult::Dismissed]);
        assert_eq!(containers(cx), vec![]);
    }

    #[gpui::test]
    fn 下の行に操作のボタンを置くと容器が高くなる(cx: &mut TestAppContext) {
        let (state, cx) = open(cx);
        let results = Results::default();
        show(
            &state,
            SnackbarMessage::new("Saved")
                .action("Open")
                .action_on_new_line(true),
            &results,
            cx,
        );
        let found = containers(cx);
        assert_eq!(found.len(), 1);
        assert!(found[0].3 > 48.0, "文の下にボタンの行を置くこと: {found:?}");
    }
}
