// 配置・余白・表示する長さ・モーションは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt
//         (`Snackbar`、`OneRowSnackbar`、`NewLineButtonSnackbar`、`SnackbarDefaults`) と
//         SnackbarHost.kt (`SnackbarHost`、`SnackbarHostState`、`SnackbarDuration`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のスナックバー

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnimationExt, App, AppContext, Div, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, SpringAnimation, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder,
};
use gpui_headless::snackbar::{SnackbarQueue, SnackbarRegion, SnackbarResult as HeadlessResult};
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

    /// 文
    pub fn message(&self) -> &SharedString {
        &self.message
    }

    /// 操作のボタンのラベル。操作が無ければ `None`
    pub fn action_label(&self) -> Option<&SharedString> {
        self.action_label.as_ref()
    }

    /// 閉じるボタンのアクセシブルな名前。閉じるボタンが無ければ `None`
    pub fn dismiss_label(&self) -> Option<&SharedString> {
        self.dismiss_label.as_ref()
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
            // Compose `SnackbarDuration.toMillis`
            Some(SnackbarDuration::Short) => Some(Duration::from_millis(4000)),
            Some(SnackbarDuration::Long) => Some(Duration::from_millis(10000)),
            Some(SnackbarDuration::Indefinite) => None,
            None if message.action_label.is_some() => None,
            None => Some(Duration::from_millis(4000)),
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
        self.queue.update(cx, |queue, cx| {
            if let Some(id) = queue.current_id() {
                queue.dismiss(id, cx);
            }
        });
    }

    /// 表示中のスナックバーの内容。無ければ `None`
    pub fn current(&self, cx: &App) -> Option<SnackbarMessage> {
        self.queue.read(cx).current().cloned()
    }
}

/// [`SnackbarHostState`] の表示中のスナックバーを描く要素
///
/// 要素は置いた場所の幅に広がり、スナックバーをその中央に置く。スナックバーは要素の幅に広がり、
/// 文の幅が 600dp を超えないところで止まる。周りに 12dp の余白を取る。
/// 表示し始めると不透明度を上げて現れ、表示を終えるとすぐに消える。
/// 表示する長さは、この要素がスナックバーを初めて描いたときから数える。
/// スナックバーの容器の上のマウスの操作は、後ろの要素に届かない。
/// 画面の下端などへの配置は [`Styled`] で呼び出し側が付ける
///
/// 支援技術には、スナックバーの文を置く role `status` の領域を公開し、表示した文を読み上げる。
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
            outer: div().w_full().flex().justify_center(),
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let queue = self.state.queue.clone();
        let current = {
            let queue = queue.read(cx);
            queue.current().cloned().zip(queue.current_id())
        };
        if let Some((_, serial)) = current {
            // 表示する長さは、この要素がスナックバーを描いている間だけ数える。
            // 要素の状態は続けて描いたフレームの間だけ残るため、描かなくなると状態ごとタスクを捨てて止まる
            let queue = queue.clone();
            window.use_keyed_state(
                ElementId::NamedChild(Arc::new(self.id.clone()), format!("timer-{serial}").into()),
                cx,
                move |_, cx| queue.update(cx, |queue, cx| queue.timer(serial, cx)),
            );
        }

        // 読み上げの領域は、スナックバーが無い間も同じ場所に描き、表示した文だけを置く。
        // ボタンの id が同じ親に置いた別のホストと重ならないよう、ホストの id の下に置く
        let outer = self.outer.id(self.id);
        let region = SnackbarRegion::new("status")
            .py(dp(supplement::VERTICAL_PADDING))
            .min_w_0();
        let Some((message, serial)) = current else {
            return outer.child(div().child(region));
        };

        // 表示し始めるたびに不透明度を 0 から上げる。番号で表示ごとに別の動きにする
        let fade_in = |element: Div, name: &'static str| {
            element.with_spring(
                ElementId::NamedInteger(name.into(), serial),
                SpringAnimation::new(theme.spring(MotionSchemeKey::FastEffects))
                    .to(1.0_f32)
                    .from(0.0),
                |this, opacity| this.opacity(opacity.clamp(0.0, 1.0)),
            )
        };
        let has_dismiss = message.dismiss_label.is_some();
        let action = message.action_label.clone().map(|label| {
            let queue = queue.clone();
            Button::new("action")
                .variant(ButtonVariant::Text)
                .colors(ButtonColors {
                    content: Some(snackbar::ACTION_LABEL_TEXT_COLOR),
                    ..ButtonColors::default()
                })
                .theme(theme.clone())
                // 描いたときのスナックバーだけを終える。時間切れで次のものに変わっていれば何もしない
                .on_click(move |_, _, cx| {
                    queue.update(cx, |queue, cx| queue.perform_action(serial, cx))
                })
                .child(gpui::text!(id = "action-label", label))
        });
        let dismiss = message.dismiss_label.clone().map(|label| {
            let queue = queue.clone();
            IconButton::new("dismiss", md3_icons::CLOSE, label.clone())
                .colors(ButtonColors {
                    content: Some(snackbar::ICON_COLOR),
                    ..ButtonColors::default()
                })
                .tooltip(Tooltip::new(label))
                .theme(theme.clone())
                .on_click(move |_, _, cx| queue.update(cx, |queue, cx| queue.dismiss(serial, cx)))
        });
        let buttons = div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .children(action)
            .children(dismiss);
        let region = region.child(fade_in(
            div().child(gpui::text!(id = "message", message.message.clone())),
            "snackbar-message",
        ));
        let new_line = message.action_on_new_line && message.action_label.is_some();
        let horizontal = dp(supplement::HORIZONTAL_SPACING);
        // `OneRowSnackbar` は左右の余白を除いた幅に、`NewLineButtonSnackbar` は余白を含む幅に最大の幅を当てる
        let end_padding = match (new_line, has_dismiss) {
            (false, false) => dp(supplement::TEXT_END_EXTRA_SPACING),
            _ => gpui::px(0.0),
        };
        let max_width = if new_line {
            dp(supplement::CONTAINER_MAX_WIDTH)
        } else {
            dp(supplement::CONTAINER_MAX_WIDTH) + horizontal + end_padding
        };

        let mut body = div()
            .relative()
            .w_full()
            .max_w(max_width)
            .min_h(dp(snackbar::SINGLE_LINE_CONTAINER_HEIGHT))
            .pl(horizontal)
            .pr(end_padding)
            .flex()
            .text_color(theme.color(snackbar::SUPPORTING_TEXT_COLOR))
            // 容器の上の押下とホバーを、後ろの要素に届けない
            .block_mouse_except_scroll()
            .child(fade_in(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
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
                    )),
                "snackbar-container",
            ));
        body.text_style()
            .refine(&theme.text_style(snackbar::SUPPORTING_TEXT_FONT));
        let body = if new_line {
            // `NewLineButtonSnackbar`。文の下の行の末尾にボタンを並べる
            body.flex_col()
                .child(region.relative().pr(horizontal))
                .child(fade_in(
                    buttons
                        .self_end()
                        .pb(dp(supplement::ACTION_BUTTON_BOTTOM_PADDING))
                        .when(!has_dismiss, |this| {
                            this.pr(dp(supplement::HORIZONTAL_SPACING_BUTTON_SIDE))
                        }),
                    "snackbar-buttons",
                ))
        } else {
            // `OneRowSnackbar`。文は残りの幅で折り返し、ボタンは上下の中央にそろえる
            body.items_center()
                .child(
                    region
                        .relative()
                        .flex_1()
                        // Compose は閉じるボタンが無いとき、文の幅からも右端の余白の分を引く
                        .when(!has_dismiss, |this| {
                            this.pr(dp(supplement::TEXT_END_EXTRA_SPACING))
                        }),
                )
                .child(fade_in(buttons, "snackbar-buttons"))
        };
        outer.child(
            div()
                .w_full()
                .max_w(max_width + dp(supplement::HOST_PADDING) * 2.0)
                .p(dp(supplement::HOST_PADDING))
                .child(body),
        )
    }
}

impl Styled for SnackbarHost {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    /// 後ろの要素でマウスのボタンを押した回数を `1` に数える
    #[derive(Clone, Copy, Default, PartialEq)]
    enum Placement {
        #[default]
        Start,
        Centered,
        Hidden,
    }

    /// `behind` に後ろの要素でマウスのボタンを押した回数を数え、`placement` に従ってホストを置く
    struct HostView {
        state: SnackbarHostState,
        behind: Rc<Cell<usize>>,
        placement: Rc<Cell<Placement>>,
    }

    impl Render for HostView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let behind = self.behind.clone();
            let placement = self.placement.get();
            // 原点のマウスの位置で hover にならないよう、原点から離す
            div()
                .id("behind")
                .size_full()
                .on_mouse_down(gpui::MouseButton::Left, move |_, _, _| {
                    behind.set(behind.get() + 1)
                })
                .pl(px(100.0))
                .pt(px(100.0))
                // ホストの幅をスナックバーの最大の幅 (周りの余白を含む) にそろえ、左上を原点から (100, 100) に置く
                .w(px(100.0 + 648.0))
                .when(placement == Placement::Centered, |this| {
                    this.w(px(100.0 + 800.0)).flex().justify_center()
                })
                .when(placement != Placement::Hidden, |this| {
                    this.child(SnackbarHost::new("host", &self.state))
                })
        }
    }

    fn open(cx: &mut TestAppContext) -> (SnackbarHostState, &mut VisualTestContext) {
        open_with(cx, Rc::default(), Rc::default())
    }

    fn open_with(
        cx: &mut TestAppContext,
        behind: Rc<Cell<usize>>,
        placement: Rc<Cell<Placement>>,
    ) -> (SnackbarHostState, &mut VisualTestContext) {
        cx.update(crate::init);
        let state = cx.update(SnackbarHostState::new);
        let host = state.clone();
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            HostView {
                state: host,
                behind,
                placement,
            }
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
        // 文の幅 600dp に、左の 16dp と右の 8dp を足した幅
        assert_eq!(containers(cx), vec![(12.0, 12.0, 624.0, 48.0)]);

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
        click(cx, 12.0 + 624.0 - 8.0 - 10.0, 12.0 + 24.0);
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
        // 閉じるボタンがあれば右の余白は取らず、文の幅 600dp に左の 16dp を足した幅になる。
        // 周りの余白を含めて幅 648dp のホストの中央に置く
        assert_eq!(containers(cx), vec![(16.0, 12.0, 616.0, 48.0)]);
        click(cx, 16.0 + 616.0 - 20.0, 12.0 + 24.0);
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

    #[gpui::test]
    fn 容器の上のマウスの操作は後ろの要素に届けない(cx: &mut TestAppContext) {
        let behind = Rc::new(Cell::new(0));
        let (state, cx) = open_with(cx, behind.clone(), Rc::default());
        let results = Results::default();
        click(cx, 300.0, 400.0);
        assert_eq!(behind.get(), 1, "容器の外は後ろの要素に届くこと");
        show(&state, SnackbarMessage::new("Saved"), &results, cx);
        click(cx, 12.0 + 100.0, 12.0 + 24.0);
        assert_eq!(behind.get(), 1, "容器の上は後ろの要素に届けないこと");
    }

    #[gpui::test]
    fn 表示する長さはホストを描いている間だけ数える(cx: &mut TestAppContext) {
        let placement = Rc::new(Cell::new(Placement::Hidden));
        let (state, cx) = open_with(cx, Rc::default(), placement.clone());
        let results = Results::default();
        show(&state, SnackbarMessage::new("Saved"), &results, cx);
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(*results.borrow(), vec![], "描く前は数えないこと");

        let redraw = |value: Placement, cx: &mut VisualTestContext| {
            placement.set(value);
            cx.update(|window, _| window.refresh());
            // 次の更新で汚れたウィンドウを描き直す
            cx.update(|_, _| {});
            cx.run_until_parked();
        };
        redraw(Placement::Start, cx);
        cx.executor().advance_clock(Duration::from_secs(3));
        redraw(Placement::Hidden, cx);
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(
            *results.borrow(),
            vec![],
            "描かなくなったら数えるのを止めること"
        );

        redraw(Placement::Start, cx);
        cx.executor().advance_clock(Duration::from_millis(3999));
        cx.run_until_parked();
        assert_eq!(*results.borrow(), vec![], "描き直したら初めから数えること");
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(*results.borrow(), vec![SnackbarResult::Dismissed]);
    }

    #[gpui::test]
    fn 中央にそろえて置いても短い文で最大の幅まで広がる(
        cx: &mut TestAppContext,
    ) {
        let (state, cx) = open_with(cx, Rc::default(), Rc::new(Cell::new(Placement::Centered)));
        let results = Results::default();
        show(&state, SnackbarMessage::new("Saved"), &results, cx);
        // 幅 800 の中央に、周りの余白を含めて幅 648 のスナックバーを置く
        assert_eq!(containers(cx), vec![(76.0 + 12.0, 12.0, 624.0, 48.0)]);
    }
}
