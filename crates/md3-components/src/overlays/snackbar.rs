// Ported from Jetpack Compose: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SnackbarHost.kt @ b8256a52. See NOTICE.
//! Snackbar と、順番に出して時間で消す SnackbarHost。

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnimationExt, App, Context, ElementId, IntoElement, Render, RenderOnce, Role, SharedString,
    SpringAnimation, Task, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::component::snackbar as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt, MotionToken};

use super::layer::{DismissHandler, SNACKBAR_PRIORITY, window_layer};
use crate::a11y::{A11yExt, A11yNode};
use crate::buttons::{Button, ButtonVariant, IconButton};
use crate::{IconName, StyledExt};

/// スナックバー（1 件）。通常は [`SnackbarHost`] が描く。
///
/// 役割は `Status`、live region は polite。退場中は操作と再通知を止める。
#[derive(IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    action: Option<(SharedString, DismissHandler)>,
    dismiss: Option<(SharedString, DismissHandler)>,
    active: bool,
}

impl Snackbar {
    /// ID と文から作る。
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            action: None,
            dismiss: None,
            active: true,
        }
    }

    /// 操作（Text ボタン）を付ける。
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// 閉じるボタン（× のアイコンボタン）を付ける。`label` は読み上げ用の名前。
    pub fn dismiss_action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.dismiss = Some((label.into(), Rc::new(handler)));
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Status)
            .label(self.message.clone())
            .live(self.active)
    }
}

impl RenderOnce for Snackbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let theme = cx.md3();
        let parent = Arc::new(self.id.clone());
        let has_dismiss = self.dismiss.is_some();
        let disabled = !self.active;
        let end_space = if has_dismiss {
            px(0.0)
        } else {
            px(layout::SNACKBAR_BUTTON_SIDE_SPACE)
        };
        div()
            .id(self.id)
            .a11y(node)
            .w_full()
            .max_w(px(layout::SNACKBAR_MAX_WIDTH))
            .min_h(px(tokens::SINGLE_LINE_CONTAINER_HEIGHT))
            .pl(px(layout::SNACKBAR_HORIZONTAL_SPACE))
            .pr(end_space)
            .flex()
            .items_center()
            .bg(theme.color(tokens::CONTAINER_COLOR))
            .corner_radius(theme.corner(tokens::CONTAINER_SHAPE))
            .shadow(theme.shadows(Elevation::from_dp(tokens::CONTAINER_ELEVATION)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .py(px(layout::SNACKBAR_VERTICAL_SPACE))
                    .when(!has_dismiss, |this| {
                        this.pr(px(layout::SNACKBAR_TEXT_END_SPACE))
                    })
                    .type_style(&theme.text_style(tokens::SUPPORTING_TEXT_FONT))
                    .text_color(theme.color(tokens::SUPPORTING_TEXT_COLOR))
                    .child(self.message),
            )
            .when_some(self.action, |this, (label, handler)| {
                this.child(
                    Button::new(
                        ElementId::NamedChild(parent.clone(), "action".into()),
                        label,
                    )
                    .disabled(disabled)
                    .variant(ButtonVariant::Text)
                    .content_color(tokens::ACTION_LABEL_TEXT_COLOR)
                    .on_click(move |_, window, cx| handler(window, cx)),
                )
            })
            .when_some(self.dismiss, |this, (label, handler)| {
                this.child(
                    IconButton::new(
                        ElementId::NamedChild(parent.clone(), "dismiss".into()),
                        IconName::Close,
                        label,
                    )
                    .disabled(disabled)
                    .content_color(tokens::ICON_COLOR)
                    .on_click(move |_, window, cx| handler(window, cx)),
                )
            })
    }
}

/// スナックバーを出しておく時間。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnackbarDuration {
    /// 4 秒。
    #[default]
    Short,
    /// 10 秒。
    Long,
    /// 閉じるまで出したまま。
    Indefinite,
}

impl SnackbarDuration {
    /// 時間。`Indefinite` は `None`。
    pub fn duration(self) -> Option<Duration> {
        match self {
            SnackbarDuration::Short => Some(Duration::from_millis(layout::SNACKBAR_SHORT_DURATION)),
            SnackbarDuration::Long => Some(Duration::from_millis(layout::SNACKBAR_LONG_DURATION)),
            SnackbarDuration::Indefinite => None,
        }
    }
}

type ResultHandler = Box<dyn FnOnce(SnackbarResult, &mut App)>;

/// [`SnackbarHost`] に出してもらう知らせ。
pub struct SnackbarMessage {
    message: SharedString,
    action: Option<(SharedString, DismissHandler)>,
    duration: Option<SnackbarDuration>,
    dismissible: bool,
    on_result: Option<ResultHandler>,
}

impl SnackbarMessage {
    /// 文から作る。
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            action: None,
            duration: None,
            dismissible: false,
            on_result: None,
        }
    }

    /// 操作を付ける。押すと処理を呼んでから閉じる。
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// 出しておく時間。指定しなければ、操作があれば `Indefinite`、なければ `Short`（Compose と同じ）。
    pub fn duration(mut self, duration: SnackbarDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 閉じるボタンを付ける。
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// 操作または破棄の結果を、ホストの更新後に一度だけ受け取る。
    ///
    /// 同じホストに次の知らせを追加できる。置換された待機中の知らせも破棄になる。
    pub fn on_result(mut self, handler: impl FnOnce(SnackbarResult, &mut App) + 'static) -> Self {
        self.on_result = Some(Box::new(handler));
        self
    }

    fn effective_duration(&self) -> SnackbarDuration {
        self.duration.unwrap_or(if self.action.is_some() {
            SnackbarDuration::Indefinite
        } else {
            SnackbarDuration::Short
        })
    }
}

/// 新しい知らせを受け取ったときの方針。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnackbarPolicy {
    /// 現在の知らせを退場させ、最新の知らせに置き換える（既定）。
    #[default]
    Replace,
    /// 現在の知らせが閉じるまで順番を待つ（Compose 相当）。
    Queue,
}

/// 知らせが閉じられた理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnackbarResult {
    /// 操作ボタンが押された。
    ActionPerformed,
    /// 時間切れ、閉じる操作、または新しい知らせによる置換。
    Dismissed,
}

/// スナックバーを 1 件ずつ、ウィンドウの下端の中央に出す。
///
/// `cx.new(|_| SnackbarHost::new("閉じる"))` で作ってルートのビューの子に置き、
/// `host.update(cx, |host, cx| host.show(SnackbarMessage::new("保存しました"), cx))` で出す。
pub struct SnackbarHost {
    dismiss_label: SharedString,
    queue: VecDeque<(u64, SnackbarMessage)>,
    next_id: u64,
    policy: SnackbarPolicy,
    exiting: bool,
    timer: Option<Task<()>>,
}

impl SnackbarHost {
    /// 作る。`dismiss_label` は閉じるボタンの読み上げ用の名前。
    pub fn new(dismiss_label: impl Into<SharedString>) -> Self {
        Self {
            dismiss_label: dismiss_label.into(),
            queue: VecDeque::new(),
            next_id: 0,
            policy: SnackbarPolicy::Replace,
            exiting: false,
            timer: None,
        }
    }

    /// 追加時の方針を指定する。既定は Replace。
    pub fn policy(mut self, policy: SnackbarPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// 現在の追加方針。
    pub fn current_policy(&self) -> SnackbarPolicy {
        self.policy
    }

    /// 以後の追加方針を変える。すでに待機している知らせの順番は変えない。
    pub fn set_policy(&mut self, policy: SnackbarPolicy, cx: &mut Context<Self>) {
        self.policy = policy;
        cx.notify();
    }

    /// 方針に従って知らせを出す。返り値は知らせの番号。
    pub fn show(&mut self, message: SnackbarMessage, cx: &mut Context<Self>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        if self.policy == SnackbarPolicy::Replace {
            while self.queue.len() > 1 {
                if let Some((_, mut pending)) = self.queue.pop_back()
                    && let Some(handler) = pending.on_result.take()
                {
                    cx.defer(move |cx| handler(SnackbarResult::Dismissed, cx));
                }
            }
            self.dismiss_current(cx);
        }
        self.queue.push_back((id, message));
        if self.queue.len() == 1 {
            self.start_timer(cx);
        }
        cx.notify();
        id
    }

    /// 出ている知らせの文。
    pub fn current(&self) -> Option<&SharedString> {
        self.queue.front().map(|(_, message)| &message.message)
    }

    /// 出ている知らせを閉じ、次があれば出す。
    pub fn dismiss_current(&mut self, cx: &mut Context<Self>) {
        self.finish(SnackbarResult::Dismissed, cx);
    }

    fn finish(&mut self, result: SnackbarResult, cx: &mut Context<Self>) {
        if self.exiting || self.queue.is_empty() {
            return;
        }
        self.timer = None;
        self.exiting = true;
        if let Some((_, message)) = self.queue.front_mut()
            && let Some(handler) = message.on_result.take()
        {
            cx.defer(move |cx| handler(result, cx));
        }
        if cx.reduce_motion() {
            self.remove_exited(cx);
        }
        cx.notify();
    }

    fn remove_exited(&mut self, cx: &mut Context<Self>) {
        if !self.exiting {
            return;
        }
        self.queue.pop_front();
        self.exiting = false;
        self.start_timer(cx);
        cx.notify();
    }

    fn dismiss(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.queue.front().is_some_and(|(front, _)| *front == id) {
            self.dismiss_current(cx);
        }
    }

    fn start_timer(&mut self, cx: &mut Context<Self>) {
        self.timer = self.queue.front().and_then(|(id, message)| {
            let id = *id;
            let duration = message.effective_duration().duration()?;
            Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(duration).await;
                this.update(cx, |host, cx| host.dismiss(id, cx)).ok();
            }))
        });
    }
}

impl Render for SnackbarHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.exiting && cx.reduce_motion() {
            self.remove_exited(cx);
        }
        let Some((id, message)) = self.queue.front() else {
            return div().into_any_element();
        };
        let id = *id;
        let this = cx.entity().downgrade();
        let mut snackbar = Snackbar::new(
            ElementId::Name(format!("md3-snackbar-{id}").into()),
            message.message.clone(),
        );
        if let Some((label, handler)) = message.action.clone() {
            let this = this.clone();
            snackbar = snackbar.action(label, move |window, cx| {
                let accepted = this
                    .update(cx, |host, cx| {
                        if host.exiting
                            || !host.queue.front().is_some_and(|(front, _)| *front == id)
                        {
                            return false;
                        }
                        host.finish(SnackbarResult::ActionPerformed, cx);
                        true
                    })
                    .unwrap_or(false);
                if accepted {
                    handler(window, cx);
                }
            });
        }
        if message.dismissible {
            snackbar = snackbar.dismiss_action(self.dismiss_label.clone(), move |_, cx| {
                this.update(cx, |host, cx| host.dismiss(id, cx)).ok();
            });
        }
        snackbar.active = !self.exiting;
        let exiting = self.exiting;
        let this = cx.entity().downgrade();
        let effects = cx.md3().spring(MotionToken::FastEffects);
        let spatial = cx.md3().spring(MotionToken::FastSpatial);
        // gpui には要素全体の scale 変換がないため、コンテナの幅で拡大を表す。
        let snackbar = div()
            .w_full()
            .max_w(px(layout::SNACKBAR_MAX_WIDTH))
            .child(snackbar)
            .with_spring(
                ("snackbar-scale", id),
                SpringAnimation::new(spatial)
                    .to(1.0_f32)
                    .from(layout::SNACKBAR_ENTER_SCALE),
                |element, scale| {
                    element
                        .w(gpui::relative(scale))
                        .max_w(px(layout::SNACKBAR_MAX_WIDTH * scale))
                },
            )
            .with_spring(
                ("snackbar-opacity", id),
                SpringAnimation::new(effects)
                    .to(if exiting { 0.0_f32 } else { 1.0 })
                    .from(0.0),
                move |element, opacity| {
                    element.map_element(|element| {
                        element.opacity(opacity.clamp(0.0, 1.0)).child(
                            gpui::canvas(
                                move |_, _, cx| {
                                    if exiting && opacity == 0.0 {
                                        cx.defer(move |cx| {
                                            this.update(cx, |host, cx| {
                                                if host
                                                    .queue
                                                    .front()
                                                    .is_some_and(|(front, _)| *front == id)
                                                {
                                                    host.remove_exited(cx);
                                                }
                                            })
                                            .ok();
                                        });
                                    }
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_0(),
                        )
                    })
                },
            );
        window_layer(
            div()
                .size_full()
                .flex()
                .flex_col()
                .justify_end()
                .items_center()
                .p(px(layout::SNACKBAR_OUTER_SPACE))
                .child(snackbar),
            SNACKBAR_PRIORITY,
            window,
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_duration_follows_compose() {
        assert_eq!(
            SnackbarMessage::new("x").effective_duration(),
            SnackbarDuration::Short
        );
        assert_eq!(
            SnackbarMessage::new("x")
                .action("元に戻す", |_, _| {})
                .effective_duration(),
            SnackbarDuration::Indefinite
        );
        assert_eq!(
            SnackbarDuration::Long.duration(),
            Some(Duration::from_millis(10_000))
        );
    }

    #[test]
    fn snackbar_is_a_status() {
        let node = Snackbar::new("s", "保存しました").a11y_node();
        assert_eq!(node.role, Role::Status);
        assert_eq!(node.label.as_deref(), Some("保存しました"));
        assert!(node.live, "表示されたら読み上げる");
    }
}
