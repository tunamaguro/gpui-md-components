//! 短い知らせの役割と、順番に出して時間で消すキュー。

use crate::overlay::DismissHandler;
use crate::{A11yExt, A11yNode};
use gpui::{
    App, Context, Div, ElementId, Role, SharedString, Stateful, Task, WeakEntity, Window, div,
    prelude::*,
};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

/// 知らせの本体。役割は `Status`、live region は polite。退場中は再通知しない。
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    aria_label: Option<Option<SharedString>>,
    active: bool,
}

impl Snackbar {
    /// ID と文から作る。
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            aria_label: None,
            active: true,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 表示中か（退場中は `false`）。
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Status)
            .with_label(
                self.aria_label
                    .clone()
                    .unwrap_or_else(|| Some(self.message.clone())),
            )
            .live(self.active)
    }

    /// 役割と名前を付けた土台を返す。
    pub fn build(self) -> Stateful<Div> {
        let node = self.a11y_node();
        div().id(self.id).a11y(node)
    }
}

/// 新しい知らせを受け取ったときの方針。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnackbarPolicy {
    /// 現在の知らせを退場させ、最新の知らせに置き換える（既定）。
    #[default]
    Replace,
    /// 現在の知らせが閉じるまで順番を待つ。
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

type ResultHandler = Box<dyn FnOnce(SnackbarResult, &mut App)>;

/// [`SnackbarQueue`] に入れる知らせ。
pub struct SnackbarEntry {
    message: SharedString,
    action: Option<(SharedString, DismissHandler)>,
    duration: Option<Duration>,
    dismissible: bool,
    on_result: Option<ResultHandler>,
}

impl SnackbarEntry {
    /// 文から作る。時間を指定しなければ閉じるまで出したままになる。
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
    pub fn action(mut self, label: impl Into<SharedString>, handler: DismissHandler) -> Self {
        self.action = Some((label.into(), handler));
        self
    }

    /// 出しておく時間。`None` なら閉じるまで出したまま。
    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// 閉じる操作を付ける。
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// 操作または破棄の結果を、持ち主の更新後に一度だけ受け取る。
    pub fn on_result(mut self, handler: impl FnOnce(SnackbarResult, &mut App) + 'static) -> Self {
        self.on_result = Some(Box::new(handler));
        self
    }
}

/// [`SnackbarQueue`] を持つビュー。
pub trait SnackbarOwner: Sized + 'static {
    /// 持っているキュー。
    fn snackbar_queue(&mut self) -> &mut SnackbarQueue;
}

/// 出ている知らせ。
pub struct SnackbarFront {
    /// 知らせの番号。
    pub id: u64,
    /// 文。
    pub message: SharedString,
    /// 操作の名前と処理。
    pub action: Option<(SharedString, DismissHandler)>,
    /// 閉じる操作の処理。
    pub dismiss: Option<DismissHandler>,
    /// 退場中か。
    pub exiting: bool,
}

/// 知らせを 1 件ずつ出し、時間で消すキュー。
///
/// 持ち主のビューが [`SnackbarOwner`] を実装する。退場の演出が終わったら
/// [`SnackbarQueue::finish_exit`] を呼ぶ。モーションを減らす設定のときは演出なしで次へ進む。
#[derive(Default)]
pub struct SnackbarQueue {
    queue: VecDeque<(u64, SnackbarEntry)>,
    next_id: u64,
    policy: SnackbarPolicy,
    exiting: bool,
    timer: Option<Task<()>>,
}

impl SnackbarQueue {
    /// 空のキューを作る。
    pub fn new() -> Self {
        Self::default()
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
    pub fn set_policy<H: SnackbarOwner>(&mut self, policy: SnackbarPolicy, cx: &mut Context<H>) {
        self.policy = policy;
        cx.notify();
    }

    /// 方針に従って知らせを出す。返り値は知らせの番号。
    pub fn show<H: SnackbarOwner>(&mut self, entry: SnackbarEntry, cx: &mut Context<H>) -> u64 {
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
        self.queue.push_back((id, entry));
        if self.queue.len() == 1 {
            self.start_timer(cx);
        }
        cx.notify();
        id
    }

    /// 出ている知らせの文。
    pub fn current(&self) -> Option<&SharedString> {
        self.queue.front().map(|(_, entry)| &entry.message)
    }

    /// 退場中か。
    pub fn exiting(&self) -> bool {
        self.exiting
    }

    /// 出ている知らせを閉じ、次があれば出す。
    pub fn dismiss_current<H: SnackbarOwner>(&mut self, cx: &mut Context<H>) {
        self.finish(SnackbarResult::Dismissed, cx);
    }

    /// 描画の前に呼ぶ。モーションを減らす設定なら、退場中の知らせをすぐ取り除く。
    pub fn prepare<H: SnackbarOwner>(&mut self, cx: &mut Context<H>) {
        if self.exiting && cx.reduce_motion() {
            self.remove_exited(cx);
        }
    }

    /// 出ている知らせと、その操作の処理を返す。
    pub fn front<H: SnackbarOwner>(&self, owner: WeakEntity<H>) -> Option<SnackbarFront> {
        let (id, entry) = self.queue.front()?;
        let id = *id;
        let action = entry.action.clone().map(|(label, handler)| {
            let owner = owner.clone();
            let run: DismissHandler = Rc::new(move |window: &mut Window, cx: &mut App| {
                let accepted = owner
                    .update(cx, |host, cx| {
                        let queue = host.snackbar_queue();
                        if queue.exiting || !queue.is_front(id) {
                            return false;
                        }
                        queue.finish(SnackbarResult::ActionPerformed, cx);
                        true
                    })
                    .unwrap_or(false);
                if accepted {
                    handler(window, cx);
                }
            });
            (label, run)
        });
        let dismiss = entry.dismissible.then(|| {
            let run: DismissHandler = Rc::new(move |_: &mut Window, cx: &mut App| {
                owner
                    .update(cx, |host, cx| host.snackbar_queue().dismiss(id, cx))
                    .ok();
            });
            run
        });
        Some(SnackbarFront {
            id,
            message: entry.message.clone(),
            action,
            dismiss,
            exiting: self.exiting,
        })
    }

    /// 退場の演出が終わった知らせを取り除く。
    pub fn finish_exit<H: SnackbarOwner>(owner: &WeakEntity<H>, id: u64, cx: &mut App) {
        owner
            .update(cx, |host, cx| {
                let queue = host.snackbar_queue();
                if queue.is_front(id) {
                    queue.remove_exited(cx);
                }
            })
            .ok();
    }

    fn is_front(&self, id: u64) -> bool {
        self.queue.front().is_some_and(|(front, _)| *front == id)
    }

    fn finish<H: SnackbarOwner>(&mut self, result: SnackbarResult, cx: &mut Context<H>) {
        if self.exiting || self.queue.is_empty() {
            return;
        }
        self.timer = None;
        self.exiting = true;
        if let Some((_, entry)) = self.queue.front_mut()
            && let Some(handler) = entry.on_result.take()
        {
            cx.defer(move |cx| handler(result, cx));
        }
        if cx.reduce_motion() {
            self.remove_exited(cx);
        }
        cx.notify();
    }

    fn remove_exited<H: SnackbarOwner>(&mut self, cx: &mut Context<H>) {
        if !self.exiting {
            return;
        }
        self.queue.pop_front();
        self.exiting = false;
        self.start_timer(cx);
        cx.notify();
    }

    fn dismiss<H: SnackbarOwner>(&mut self, id: u64, cx: &mut Context<H>) {
        if self.is_front(id) {
            self.dismiss_current(cx);
        }
    }

    fn start_timer<H: SnackbarOwner>(&mut self, cx: &mut Context<H>) {
        self.timer = self.queue.front().and_then(|(id, entry)| {
            let id = *id;
            let duration = entry.duration?;
            Some(
                cx.spawn(async move |this: WeakEntity<H>, cx: &mut gpui::AsyncApp| {
                    cx.background_executor().timer(duration).await;
                    this.update(cx, |host, cx| host.snackbar_queue().dismiss(id, cx))
                        .ok();
                }),
            )
        });
    }
}
