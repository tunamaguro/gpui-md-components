//! テスト用のウィンドウ

use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext, Window};

/// [`crate::init`] を登録し、`build` のビューを開いたウィンドウをアクティブにする
///
/// キー操作はアクティブなウィンドウに届く。テスト用のウィンドウは開いただけではアクティブにならない
pub(crate) fn add_test_window<V: Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> (Entity<V>, &mut VisualTestContext) {
    cx.update(crate::init);
    let (view, cx) = cx.add_window_view(build);
    cx.update(|window, _| window.activate_window());
    (view, cx)
}
