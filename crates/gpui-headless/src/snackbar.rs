//! 見た目を持たない、短い通知の表示待ちの列と読み上げの領域

use std::collections::VecDeque;
use std::time::Duration;

use gpui::{
    AnyElement, App, Context, Div, ElementId, InteractiveElement, Interactivity, IntoElement,
    ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Task,
    Window, accesskit, div,
};

use crate::accessibility::A11yInfoExt;

/// 通知が表示を終えた理由
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SnackbarResult {
    /// 閉じられたか、表示する長さを過ぎた
    Dismissed,
    /// 通知の操作が選ばれた
    ActionPerformed,
}

type ResultHandler = Box<dyn FnOnce(SnackbarResult, &mut App)>;

struct Entry<T> {
    content: T,
    duration: Option<Duration>,
    on_result: Option<ResultHandler>,
}

/// 通知を 1 つずつ表示する列
///
/// [`SnackbarQueue::show`] で加えた通知を、加えた順に 1 つずつ表示中にする。
/// 表示中の通知は、[`SnackbarQueue::dismiss`] か [`SnackbarQueue::perform_action`] で終えるか、
/// 表示する長さを過ぎると表示を終え、次の通知が表示中になる。
/// 表示する長さは、[`SnackbarQueue::start_timer`] で通知を画面に出したと伝えたときから数える。
/// 状態が変わるたびに、この列の [`gpui::Entity`] を通知する
pub struct SnackbarQueue<T: 'static> {
    current: Option<Entry<T>>,
    /// 表示中になった通知ごとに増える番号
    serial: u64,
    pending: VecDeque<Entry<T>>,
    timer: Option<Task<()>>,
}

impl<T: 'static> Default for SnackbarQueue<T> {
    fn default() -> Self {
        Self {
            current: None,
            serial: 0,
            pending: VecDeque::new(),
            timer: None,
        }
    }
}

impl<T: 'static> SnackbarQueue<T> {
    /// 空の列を作る
    pub fn new() -> Self {
        Self::default()
    }

    /// 通知 `content` を列の末尾に加える
    ///
    /// 表示中の通知が無ければ、すぐに表示中にする。`duration` は表示する長さで、`None` なら終えるまで表示し続ける。
    /// 表示を終えたときに、終えた理由を `on_result` に渡して 1 回だけ呼ぶ。
    /// `on_result` は列の更新を終えた後に呼ぶため、その中で同じ列を読み書きできる
    pub fn show(
        &mut self,
        content: T,
        duration: Option<Duration>,
        on_result: impl FnOnce(SnackbarResult, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.pending.push_back(Entry {
            content,
            duration,
            on_result: Some(Box::new(on_result)),
        });
        if self.current.is_none() {
            self.advance(cx);
        }
    }

    /// 表示中の通知。無ければ `None`
    pub fn current(&self) -> Option<&T> {
        self.current.as_ref().map(|entry| &entry.content)
    }

    /// 表示中の通知を区別する番号。通知が表示中になるたびに変わる。表示中の通知が無ければ `None`
    pub fn current_id(&self) -> Option<u64> {
        self.current.as_ref().map(|_| self.serial)
    }

    /// 番号 `id` の通知を画面に出したと伝え、表示する長さを数え始める
    ///
    /// `id` の通知が表示中でないか、すでに数え始めていれば何もしない。状態を変えても通知しない
    pub fn start_timer(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.current_id() != Some(id) || self.timer.is_some() {
            return;
        }
        let Some(duration) = self.current.as_ref().and_then(|entry| entry.duration) else {
            return;
        };
        self.timer = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(duration).await;
            this.update(cx, |queue, cx| queue.dismiss(id, cx)).ok();
        }));
    }

    /// 番号 `id` の通知を閉じ、[`SnackbarResult::Dismissed`] を伝える。`id` の通知が表示中でなければ何もしない
    pub fn dismiss(&mut self, id: u64, cx: &mut Context<Self>) {
        self.finish(id, SnackbarResult::Dismissed, cx);
    }

    /// 番号 `id` の通知の操作が選ばれたとして表示を終え、[`SnackbarResult::ActionPerformed`] を伝える。
    /// `id` の通知が表示中でなければ何もしない
    pub fn perform_action(&mut self, id: u64, cx: &mut Context<Self>) {
        self.finish(id, SnackbarResult::ActionPerformed, cx);
    }

    fn finish(&mut self, id: u64, result: SnackbarResult, cx: &mut Context<Self>) {
        if self.current_id() != Some(id) {
            return;
        }
        let Some(mut entry) = self.current.take() else {
            return;
        };
        self.timer = None;
        self.advance(cx);
        if let Some(on_result) = entry.on_result.take() {
            // 列を更新している間は、`on_result` から同じ列を読み書きできないため、更新を終えてから呼ぶ
            cx.defer(move |cx| on_result(result, cx));
        }
    }

    /// 次の通知を表示中にする
    fn advance(&mut self, cx: &mut Context<Self>) {
        self.current = self.pending.pop_front();
        if self.current.is_some() {
            self.serial += 1;
        }
        cx.notify();
    }
}

/// 通知を表示する、見た目を持たない領域
///
/// 支援技術には role `status` として公開され、内容が変わると、支援技術は手の空いたときに領域の内容全体を読み上げる
/// (live region の polite と atomic)。領域には読み上げる文だけを置き、ボタンなどの操作は領域の外に置くこと。
/// 領域は通知が無い間も同じ場所で描き続けること。支援技術は、領域ができたときにすでにある内容を読み上げないことがある。
/// フォーカスも操作も持たない。見た目は [`Styled`] と子の要素で呼び出し側が付ける
#[derive(IntoElement)]
pub struct SnackbarRegion {
    inner: Stateful<Div>,
}

impl SnackbarRegion {
    /// 通知を表示する領域を作る
    ///
    /// `id` は同じ親の中で一意にすること
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            inner: div().id(id),
        }
    }
}

impl RenderOnce for SnackbarRegion {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.inner
            .role(accesskit::Role::Status)
            .a11y_info(|node: &mut accesskit::Node| {
                // WAI-ARIA の status が暗黙に持つ aria-live と aria-atomic
                node.set_live(accesskit::Live::Polite);
                node.set_live_atomic();
            })
    }
}

impl Styled for SnackbarRegion {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for SnackbarRegion {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for SnackbarRegion {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for SnackbarRegion {}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{AppContext as _, Element, Entity, TestAppContext};

    use super::*;

    type Log = Rc<RefCell<Vec<(&'static str, SnackbarResult)>>>;

    fn show(
        queue: &Entity<SnackbarQueue<&'static str>>,
        content: &'static str,
        duration: Option<Duration>,
        log: &Log,
        cx: &mut TestAppContext,
    ) {
        let log = log.clone();
        queue.update(cx, |queue, cx| {
            queue.show(
                content,
                duration,
                move |result, _| log.borrow_mut().push((content, result)),
                cx,
            )
        });
    }

    fn current(
        queue: &Entity<SnackbarQueue<&'static str>>,
        cx: &mut TestAppContext,
    ) -> Option<&'static str> {
        queue.read_with(cx, |queue, _| queue.current().copied())
    }

    fn start(queue: &Entity<SnackbarQueue<&'static str>>, cx: &mut TestAppContext) {
        queue.update(cx, |queue, cx| {
            if let Some(id) = queue.current_id() {
                queue.start_timer(id, cx);
            }
        });
    }

    #[gpui::test]
    fn 通知は加えた順に_1_つずつ表示し_画面に出してから長さを過ぎると次に進む(
        cx: &mut TestAppContext,
    ) {
        let queue = cx.new(|_| SnackbarQueue::new());
        let log = Log::default();
        show(&queue, "first", Some(Duration::from_secs(4)), &log, cx);
        show(&queue, "second", Some(Duration::from_secs(10)), &log, cx);
        assert_eq!(current(&queue, cx), Some("first"));

        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(
            current(&queue, cx),
            Some("first"),
            "画面に出すまでは数えないこと"
        );

        start(&queue, cx);
        start(&queue, cx);
        cx.executor().advance_clock(Duration::from_millis(3999));
        cx.run_until_parked();
        assert_eq!(current(&queue, cx), Some("first"));
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(current(&queue, cx), Some("second"));
        assert_eq!(*log.borrow(), vec![("first", SnackbarResult::Dismissed)]);

        start(&queue, cx);
        cx.executor().advance_clock(Duration::from_secs(10));
        cx.run_until_parked();
        assert_eq!(current(&queue, cx), None);
    }

    #[gpui::test]
    fn 終えるのは指定した番号の通知だけで_長さの無い通知は表示し続ける(
        cx: &mut TestAppContext,
    ) {
        let queue = cx.new(|_| SnackbarQueue::new());
        let log = Log::default();
        show(&queue, "first", Some(Duration::from_secs(4)), &log, cx);
        show(&queue, "second", None, &log, cx);
        let first = queue.read_with(cx, |queue, _| queue.current_id()).unwrap();
        queue.update(cx, |queue, cx| queue.perform_action(first, cx));
        cx.run_until_parked();
        // 前の通知への求めは、次の通知を終えない
        queue.update(cx, |queue, cx| queue.dismiss(first, cx));
        start(&queue, cx);
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(current(&queue, cx), Some("second"));
        let second = queue.read_with(cx, |queue, _| queue.current_id()).unwrap();
        queue.update(cx, |queue, cx| queue.dismiss(second, cx));
        cx.run_until_parked();
        assert_eq!(
            *log.borrow(),
            vec![
                ("first", SnackbarResult::ActionPerformed),
                ("second", SnackbarResult::Dismissed)
            ]
        );
    }

    #[gpui::test]
    fn 結果を受けて同じ列に次の通知を加えられる(cx: &mut TestAppContext) {
        let queue = cx.new(|_| SnackbarQueue::new());
        let log = Log::default();
        show(&queue, "waiting", None, &log, cx);
        let handle = queue.clone();
        queue.update(cx, |queue, cx| {
            queue.show(
                "first",
                None,
                move |_, cx| {
                    handle.update(cx, |queue, cx| queue.show("follow-up", None, |_, _| {}, cx))
                },
                cx,
            )
        });
        show(&queue, "queued", None, &log, cx);
        let dismiss = |cx: &mut TestAppContext| {
            let id = queue.read_with(cx, |queue, _| queue.current_id()).unwrap();
            queue.update(cx, |queue, cx| queue.dismiss(id, cx));
            cx.run_until_parked();
        };
        dismiss(cx);
        assert_eq!(current(&queue, cx), Some("first"));
        dismiss(cx);
        assert_eq!(
            current(&queue, cx),
            Some("queued"),
            "先に待っていた通知を先に表示すること"
        );
        dismiss(cx);
        assert_eq!(current(&queue, cx), Some("follow-up"));
    }

    struct RegionView(Option<accesskit::Node>);

    impl gpui::Render for RegionView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let element = SnackbarRegion::new("region")
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            self.0 = Some(node);
            element
        }
    }

    #[gpui::test]
    fn 領域は_status_として_polite_と_atomic_で公開する(cx: &mut TestAppContext) {
        let (view, cx) = crate::test_window::add_test_window(cx, |_, _| RegionView(None));
        let node = view
            .read_with(cx, |view, _| view.0.clone())
            .expect("描画されていること");
        assert_eq!(node.role(), accesskit::Role::Status);
        assert_eq!(node.live(), Some(accesskit::Live::Polite));
        assert!(node.is_live_atomic());
    }
}
