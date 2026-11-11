//! 見た目を持たない、支援技術に公開するテキスト

use gpui::{
    AnyElement, App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit,
    div,
};

/// 子の内容の代わりに、渡した文字列をテキストとして支援技術に公開する要素
///
/// 支援技術には role `label` (静的なテキスト) として公開され、文字列は祖先のボタンなどの名前にも含まれる。
/// 画面の表示が意味を伝えきれない短い内容 (件数の数字だけのバッジなど) に、意味の伝わる文字列を与えるのに使う。
/// フォーカスも操作も持たない。見た目は [`Styled`] と子の要素で呼び出し側が付ける。
/// 子に `gpui::text!` のような id を持つテキストを置くと、それも別のノードとして公開されるため、
/// `&str` などのテキストをそのまま置くこと
#[derive(IntoElement)]
pub struct StaticText {
    inner: Stateful<Div>,
    text: SharedString,
}

impl StaticText {
    /// `text` を公開する要素を作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。変わると支援技術にはノードの削除と追加として伝わる
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            inner: div().id(id),
            text: text.into(),
        }
    }
}

impl RenderOnce for StaticText {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // AccessKit は Label の名前を値から求めるため、gpui のテキスト (`gpui::text!`) と同じく値に書く
        self.inner
            .role(accesskit::Role::Label)
            .aria_value(self.text)
    }
}

impl Styled for StaticText {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for StaticText {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Element, Render, TestAppContext};

    use super::*;
    use crate::test_window::add_test_window;

    struct TextView {
        node: Option<accesskit::Node>,
    }

    impl Render for TextView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let element = StaticText::new("text", "3 new messages")
                .child("3")
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

    #[gpui::test]
    fn 渡した文字列を_label_の値として公開する(cx: &mut TestAppContext) {
        let (view, cx) = add_test_window(cx, |_, _| TextView { node: None });
        cx.run_until_parked();
        let node = view
            .read_with(cx, |view, _| view.node.clone())
            .expect("描画されていること");
        assert_eq!(node.role(), accesskit::Role::Label);
        assert_eq!(node.value(), Some("3 new messages"));
    }
}
