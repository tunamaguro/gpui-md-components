// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SnackbarHost.kt @ b8256a52
// Copyright 2020 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! Snackbar と、順番に出して時間で消す SnackbarHost。

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnimationExt, App, Context, ElementId, IntoElement, Render, RenderOnce, SharedString,
    SpringAnimation, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::component::snackbar as tokens;
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt, MotionToken};

use crate::buttons::{Button, ButtonVariant, IconButton};
use crate::{IconName, StyledExt};
use gpui_headless::overlay::{DismissHandler, SNACKBAR_PRIORITY, window_layer};
use gpui_headless::{SnackbarEntry, SnackbarOwner, SnackbarQueue};

/// 知らせを追加するときの方針。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnackbarPolicy {
    /// 現在の知らせを退場させ、最新の知らせに置き換える（既定）。
    #[default]
    Replace,
    /// 現在の知らせが閉じるまで順番を待つ。
    Queue,
}

impl From<SnackbarPolicy> for gpui_headless::SnackbarPolicy {
    fn from(policy: SnackbarPolicy) -> Self {
        match policy {
            SnackbarPolicy::Replace => Self::Replace,
            SnackbarPolicy::Queue => Self::Queue,
        }
    }
}

impl From<gpui_headless::SnackbarPolicy> for SnackbarPolicy {
    fn from(policy: gpui_headless::SnackbarPolicy) -> Self {
        match policy {
            gpui_headless::SnackbarPolicy::Replace => Self::Replace,
            gpui_headless::SnackbarPolicy::Queue => Self::Queue,
        }
    }
}

/// 知らせが閉じられた理由。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnackbarResult {
    /// 操作ボタンが押された。
    ActionPerformed,
    /// 時間切れ、閉じる操作、または新しい知らせによる置換。
    Dismissed,
}

impl From<gpui_headless::SnackbarResult> for SnackbarResult {
    fn from(result: gpui_headless::SnackbarResult) -> Self {
        match result {
            gpui_headless::SnackbarResult::ActionPerformed => Self::ActionPerformed,
            gpui_headless::SnackbarResult::Dismissed => Self::Dismissed,
        }
    }
}

/// スナックバー（1 件）。通常は [`SnackbarHost`] が描く。
///
/// 役割は `Status`、live region は polite。退場中は操作と再通知を止める。
#[derive(IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    inner: gpui_headless::Snackbar,
    action: Option<(SharedString, DismissHandler)>,
    dismiss: Option<(SharedString, DismissHandler)>,
}

impl Snackbar {
    /// ID と文から作る。
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        let id = id.into();
        let message = message.into();
        Self {
            inner: gpui_headless::Snackbar::new(id.clone(), message.clone()),
            id,
            message,
            action: None,
            dismiss: None,
        }
    }

    delegate_accessible_name!(inner);

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
}

impl RenderOnce for Snackbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = !self.inner.a11y_node().live;
        let base = self.inner.build();
        let theme = cx.md3();
        let parent = Arc::new(self.id.clone());
        let has_dismiss = self.dismiss.is_some();
        let end_space = if has_dismiss {
            px(0.0)
        } else {
            px(layout::SNACKBAR_BUTTON_SIDE_SPACE)
        };
        base.w_full()
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
                    )
                    .aria_label(label)
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

    fn into_entry(self) -> SnackbarEntry {
        let duration = self.effective_duration().duration();
        let mut entry = SnackbarEntry::new(self.message)
            .duration(duration)
            .dismissible(self.dismissible);
        if let Some((label, handler)) = self.action {
            entry = entry.action(label, handler);
        }
        if let Some(on_result) = self.on_result {
            entry = entry.on_result(move |result, cx| on_result(result.into(), cx));
        }
        entry
    }
}

/// スナックバーを 1 件ずつ、ウィンドウの下端の中央に出す。
///
/// `cx.new(|_| SnackbarHost::new("閉じる"))` で作ってルートのビューの子に置き、
/// `host.update(cx, |host, cx| host.show(SnackbarMessage::new("保存しました"), cx))` で出す。
pub struct SnackbarHost {
    dismiss_label: SharedString,
    queue: SnackbarQueue,
}

impl SnackbarOwner for SnackbarHost {
    fn snackbar_queue(&mut self) -> &mut SnackbarQueue {
        &mut self.queue
    }
}

impl SnackbarHost {
    /// 作る。`dismiss_label` は閉じるボタンの読み上げ用の名前。
    pub fn new(dismiss_label: impl Into<SharedString>) -> Self {
        Self {
            dismiss_label: dismiss_label.into(),
            queue: SnackbarQueue::new(),
        }
    }

    /// 追加時の方針を指定する。既定は Replace。
    pub fn policy(mut self, policy: SnackbarPolicy) -> Self {
        self.queue = self.queue.policy(policy.into());
        self
    }

    /// 現在の追加方針。
    pub fn current_policy(&self) -> SnackbarPolicy {
        self.queue.current_policy().into()
    }

    /// 以後の追加方針を変える。すでに待機している知らせの順番は変えない。
    pub fn set_policy(&mut self, policy: SnackbarPolicy, cx: &mut Context<Self>) {
        self.queue.set_policy(policy.into(), cx);
    }

    /// 方針に従って知らせを出す。返り値は知らせの番号。
    pub fn show(&mut self, message: SnackbarMessage, cx: &mut Context<Self>) -> u64 {
        self.queue.show(message.into_entry(), cx)
    }

    /// 出ている知らせの文。
    pub fn current(&self) -> Option<&SharedString> {
        self.queue.current()
    }

    /// 出ている知らせを閉じ、次があれば出す。
    pub fn dismiss_current(&mut self, cx: &mut Context<Self>) {
        self.queue.dismiss_current(cx);
    }
}

impl Render for SnackbarHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.queue.prepare(cx);
        let this = cx.entity().downgrade();
        let Some(front) = self.queue.front(this.clone()) else {
            return div().into_any_element();
        };
        let id = front.id;
        let mut snackbar = Snackbar::new(
            ElementId::Name(format!("md3-snackbar-{id}").into()),
            front.message,
        );
        if let Some((label, handler)) = front.action {
            snackbar = snackbar.action(label, move |window, cx| handler(window, cx));
        }
        if let Some(handler) = front.dismiss {
            snackbar = snackbar.dismiss_action(self.dismiss_label.clone(), move |window, cx| {
                handler(window, cx)
            });
        }
        let exiting = front.exiting;
        snackbar.inner = snackbar.inner.active(!exiting);
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
                                            SnackbarQueue::finish_exit(&this, id, cx)
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
}
