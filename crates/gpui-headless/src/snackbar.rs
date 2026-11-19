//! 見た目を持たない、短い通知の表示待ちの列と読み上げの領域

use std::collections::VecDeque;
use std::time::Duration;

use gpui::{
    AnyElement, App, Context, Div, ElementId, InteractiveElement, Interactivity, IntoElement,
    ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Task,
    Window, accesskit, div,
};

use crate::accessibility::A11yInfoExt;

/// 通知を表示し続ける長さ
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SnackbarDuration {
    /// 4 秒
    #[default]
    Short,
    /// 10 秒
    Long,
    /// 閉じるか操作されるまで表示し続ける
    Indefinite,
}

impl SnackbarDuration {
    /// 表示し続ける長さ。[`SnackbarDuration::Indefinite`] なら `None`
    pub fn duration(self) -> Option<Duration> {
        match self {
            Self::Short => Some(Duration::from_millis(4000)),
            Self::Long => Some(Duration::from_millis(10000)),
            Self::Indefinite => None,
        }
    }
}

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
    duration: SnackbarDuration,
    on_result: Option<ResultHandler>,
}

/// 通知を 1 つずつ表示する列
///
/// [`SnackbarQueue::show`] で加えた通知を、加えた順に 1 つずつ表示中にする。
/// 表示中の通知は、[`SnackbarQueue::dismiss`] か [`SnackbarQueue::perform_action`] を呼ぶか、
/// 表示する長さを過ぎると表示を終え、次の通知が表示中になる。
/// 表示する長さは、通知が表示中になったときから数える。
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
    /// 表示中の通知が無ければ、すぐに表示中にする。
    /// 表示を終えたときに、終えた理由を `on_result` に渡して 1 回だけ呼ぶ
    pub fn show(
        &mut self,
        content: T,
        duration: SnackbarDuration,
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

    /// 表示中の通知を閉じ、[`SnackbarResult::Dismissed`] を伝える。表示中の通知が無ければ何もしない
    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.finish(SnackbarResult::Dismissed, cx);
    }

    /// 表示中の通知の操作が選ばれたとして表示を終え、[`SnackbarResult::ActionPerformed`] を伝える。
    /// 表示中の通知が無ければ何もしない
    pub fn perform_action(&mut self, cx: &mut Context<Self>) {
        self.finish(SnackbarResult::ActionPerformed, cx);
    }

    fn finish(&mut self, result: SnackbarResult, cx: &mut Context<Self>) {
        let Some(mut entry) = self.current.take() else {
            return;
        };
        self.timer = None;
        if let Some(on_result) = entry.on_result.take() {
            on_result(result, cx);
        }
        self.advance(cx);
    }

    /// 次の通知を表示中にし、表示する長さの後に閉じる
    fn advance(&mut self, cx: &mut Context<Self>) {
        self.current = self.pending.pop_front();
        if let Some(entry) = &self.current {
            self.serial += 1;
            let serial = self.serial;
            self.timer = entry.duration.duration().map(|duration| {
                cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(duration).await;
                    this.update(cx, |queue, cx| {
                        // 先に閉じられて次の通知に変わっていれば、その通知は閉じない
                        if queue.current_id() == Some(serial) {
                            queue.dismiss(cx);
                        }
                    })
                    .ok();
                })
            });
        }
        cx.notify();
    }
}

/// 通知を表示する、見た目を持たない領域
///
/// 支援技術には role `status` として公開され、内容が変わると支援技術が読み上げを待つ間に読み上げる (live region の polite)。
/// 領域は通知が無い間も描き続けること。支援技術は、領域ができたときにすでにある内容を読み上げないことがある。
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
            .a11y_info(|node: &mut accesskit::Node| node.set_live(accesskit::Live::Polite))
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

    use gpui::{AppContext as _, TestAppContext};

    use super::*;

    type Log = Rc<RefCell<Vec<(&'static str, SnackbarResult)>>>;

    fn show(
        queue: &gpui::Entity<SnackbarQueue<&'static str>>,
        content: &'static str,
        duration: SnackbarDuration,
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

    #[gpui::test]
    fn 通知は加えた順に_1_つずつ表示し_長さを過ぎると次に進む(
        cx: &mut TestAppContext,
    ) {
        let queue = cx.new(|_| SnackbarQueue::new());
        let log = Log::default();
        show(&queue, "first", SnackbarDuration::Short, &log, cx);
        show(&queue, "second", SnackbarDuration::Long, &log, cx);
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            Some("first")
        );

        cx.executor().advance_clock(Duration::from_millis(3999));
        cx.run_until_parked();
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            Some("first")
        );
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            Some("second")
        );
        assert_eq!(*log.borrow(), vec![("first", SnackbarResult::Dismissed)]);

        // 次の通知の長さは、表示中になったときから数える
        cx.executor().advance_clock(Duration::from_millis(9999));
        cx.run_until_parked();
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            Some("second")
        );
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            None
        );
    }

    #[gpui::test]
    fn 操作か閉じる求めで表示を終え_前の通知の時間切れは次の通知を閉じない(
        cx: &mut TestAppContext,
    ) {
        let queue = cx.new(|_| SnackbarQueue::new());
        let log = Log::default();
        show(&queue, "first", SnackbarDuration::Short, &log, cx);
        show(&queue, "second", SnackbarDuration::Indefinite, &log, cx);
        let first = queue.read_with(cx, |queue, _| queue.current_id());
        queue.update(cx, |queue, cx| queue.perform_action(cx));
        assert_ne!(queue.read_with(cx, |queue, _| queue.current_id()), first);

        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        assert_eq!(
            queue.read_with(cx, |queue, _| queue.current().copied()),
            Some("second"),
            "期限の無い通知は表示し続けること"
        );
        queue.update(cx, |queue, cx| queue.dismiss(cx));
        assert_eq!(
            *log.borrow(),
            vec![
                ("first", SnackbarResult::ActionPerformed),
                ("second", SnackbarResult::Dismissed)
            ]
        );
        queue.update(cx, |queue, cx| queue.dismiss(cx));
        assert_eq!(
            log.borrow().len(),
            2,
            "表示中の通知が無ければ何もしないこと"
        );
    }
}
