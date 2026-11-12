//! 見た目を持たない進捗の表示

use gpui::{
    AnyElement, App, Div, ElementId, InteractiveElement, Interactivity, IntoElement, ParentElement,
    RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit,
    div,
};

use crate::accessibility::A11yInfoExt;

/// 進捗の状態
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Progress {
    /// 進み具合が分からない
    Indeterminate,
    /// 全体のうち進んだ割合。0.0 で未着手、1.0 で完了
    ///
    /// 0.0〜1.0 の外の値はその範囲に収め、NaN は 0.0 として扱う
    Determinate(f32),
}

impl Progress {
    /// 進んだ割合を 0.0〜1.0 に収めた値。進み具合が分からなければ `None`
    pub fn fraction(self) -> Option<f32> {
        match self {
            Self::Indeterminate => None,
            Self::Determinate(fraction) if fraction.is_nan() => Some(0.0),
            Self::Determinate(fraction) => Some(fraction.clamp(0.0, 1.0)),
        }
    }
}

/// 見た目を持たない進捗の表示
///
/// 支援技術には role `progressbar` として公開される。進み具合が分かるときは、割合を 0〜100 の値として公開し、
/// 分からないときは値を公開しない。
/// アクセシブルな名前は [`StatefulInteractiveElement::aria_label`] で与えること (AccessKit は progressbar の名前を子の内容から求めない)。
/// フォーカスも操作も持たない。見た目は [`Styled`] と子の要素で呼び出し側が付ける
#[derive(IntoElement)]
pub struct ProgressBar {
    inner: Stateful<Div>,
    progress: Progress,
}

impl ProgressBar {
    /// `progress` を表す進捗の表示を作る
    ///
    /// `id` は同じ親の中で一意にすること
    pub fn new(id: impl Into<ElementId>, progress: Progress) -> Self {
        Self {
            inner: div().id(id),
            progress,
        }
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let fraction = self.progress.fraction();
        self.inner
            .role(accesskit::Role::ProgressIndicator)
            .a11y_info(move |node: &mut accesskit::Node| {
                // WAI-ARIA の progressbar の既定の範囲 (0〜100) に合わせる
                if let Some(fraction) = fraction {
                    node.set_min_numeric_value(0.0);
                    node.set_max_numeric_value(100.0);
                    node.set_numeric_value(f64::from(fraction * 100.0));
                }
            })
    }
}

impl Styled for ProgressBar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for ProgressBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for ProgressBar {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for ProgressBar {}

#[cfg(test)]
mod tests {
    use gpui::{Context, Element, Render, TestAppContext};

    use super::*;
    use crate::test_window::add_test_window;

    struct ProgressView {
        progress: Progress,
        node: Option<accesskit::Node>,
    }

    impl Render for ProgressView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let element = ProgressBar::new("progress", self.progress)
                .aria_label("Downloading")
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(element.a11y_role().expect("role を持つこと"));
            element.write_a11y_info(&mut node);
            self.node = Some(node);
            element
        }
    }

    fn node(cx: &mut TestAppContext, progress: Progress) -> accesskit::Node {
        let (view, cx) = add_test_window(cx, |_, _| ProgressView {
            progress,
            node: None,
        });
        cx.run_until_parked();
        view.read_with(cx, |view, _| view.node.clone())
            .expect("描画されていること")
    }

    #[gpui::test]
    fn 進み具合が分かるときは割合を_0_から_100_の値で公開する(
        cx: &mut TestAppContext,
    ) {
        let node = node(cx, Progress::Determinate(0.25));
        assert_eq!(node.role(), accesskit::Role::ProgressIndicator);
        assert_eq!(node.label(), Some("Downloading"));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        assert_eq!(node.numeric_value(), Some(25.0));
    }

    #[gpui::test]
    fn 進み具合が分からないときは値を公開しない(cx: &mut TestAppContext) {
        let node = node(cx, Progress::Indeterminate);
        assert_eq!(node.role(), accesskit::Role::ProgressIndicator);
        assert_eq!(node.numeric_value(), None);
        assert_eq!(node.min_numeric_value(), None);
        assert_eq!(node.max_numeric_value(), None);
    }

    #[gpui::test]
    fn 範囲外と_nan_の割合は範囲に収めて公開する(cx: &mut TestAppContext) {
        for (fraction, expected) in [(-0.5, 0.0), (1.5, 100.0), (f32::NAN, 0.0)] {
            let node = node(cx, Progress::Determinate(fraction));
            assert_eq!(node.numeric_value(), Some(expected), "{fraction}");
        }
    }

    #[test]
    fn 割合は範囲に収める() {
        assert_eq!(Progress::Indeterminate.fraction(), None);
        assert_eq!(Progress::Determinate(0.5).fraction(), Some(0.5));
        assert_eq!(Progress::Determinate(-1.0).fraction(), Some(0.0));
        assert_eq!(Progress::Determinate(2.0).fraction(), Some(1.0));
        assert_eq!(Progress::Determinate(f32::NAN).fraction(), Some(0.0));
    }
}
