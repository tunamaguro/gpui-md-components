//! ウィンドウのルートを包む要素

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window, accesskit,
};

use crate::{dialog, focus_visible};

/// `child` をウィンドウのルートとして包む
///
/// この crate の部品が必要とするウィンドウ全体の入力を記録する。部品を使うウィンドウでは、
/// ルートビューの render が返す要素を包むこと。包まない場合、[`crate::is_focus_visible`] はマウスボタンの押下を知らない。
/// [`crate::dialog::Dialog`] が開いている間は、`child` を支援技術から隠す。
/// レイアウトと見た目は `child` のまま
pub fn root(child: impl IntoElement) -> Root {
    Root {
        child: child.into_any_element(),
        hidden: false,
    }
}

/// [`root`] が返す要素
pub struct Root {
    child: AnyElement,
    /// `child` を支援技術から隠すか。`child` をレイアウトすると決まる
    hidden: bool,
}

impl IntoElement for Root {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Root {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        // a11y ノードを書くには id が要る
        Some("gpui-headless-root".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        dialog::begin_layout(cx);
        let layout_id = self.child.request_layout(window, cx);
        self.hidden = dialog::any_open(cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // gpui にはアプリ全体でマウスの入力を受ける仕組みがなく、ウィンドウのリスナーは描画のたびに登録し直す必要がある
        focus_visible::listen(window);
        self.child.paint(window, cx);
    }

    fn a11y_role(&self) -> Option<accesskit::Role> {
        // 隠さない間はノードを作らない。GenericContainer は、支援技術には子を親に付けて渡される
        self.hidden.then_some(accesskit::Role::GenericContainer)
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        node.set_hidden();
    }
}

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn 隠すときだけ_hidden_のノードを書く() {
        let root = |hidden| Root {
            child: div().into_any_element(),
            hidden,
        };
        assert_eq!(root(false).a11y_role(), None);
        let hidden = root(true);
        let mut node = accesskit::Node::new(hidden.a11y_role().expect("role を持つこと"));
        hidden.write_a11y_info(&mut node);
        assert!(node.is_hidden());
    }
}
