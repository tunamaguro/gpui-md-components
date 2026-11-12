//! テスト用のウィンドウ

use gpui::{
    AppContext, Context, Entity, IntoElement, Render, TestAppContext, VisualTestContext, Window,
};

/// `build` のビューを [`crate::root`] で包んで描画するビュー
struct TestRoot<V> {
    view: Entity<V>,
}

impl<V: Render> Render for TestRoot<V> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        crate::root(self.view.clone())
    }
}

/// [`crate::init`] を登録し、`build` のビューを [`crate::root`] で包んで開いたウィンドウをアクティブにする
///
/// キー操作はアクティブなウィンドウに届く。テスト用のウィンドウは開いただけではアクティブにならない
pub(crate) fn add_test_window<V: Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> (Entity<V>, &mut VisualTestContext) {
    cx.update(crate::init);
    let (root, cx) = cx.add_window_view(|window, cx| TestRoot {
        view: cx.new(|cx| build(window, cx)),
    });
    let view = root.read_with(cx, |root, _| root.view.clone());
    cx.update(|window, _| window.activate_window());
    // アクティブになるときにマウスの位置がテスト用の platform の (0, 0) に戻るため、
    // 最初のイベントより前にアクティブにし終える
    cx.run_until_parked();
    (view, cx)
}
