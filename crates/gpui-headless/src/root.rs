//! ウィンドウのルートを包む要素

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};

use crate::focus_visible;

/// `child` をウィンドウのルートとして包む
///
/// この crate の部品が必要とするウィンドウ全体の入力を記録する。部品を使うウィンドウでは、
/// ルートビューの render が返す要素を包むこと。包まない場合、[`crate::is_focus_visible`] はマウスボタンの押下を知らない。
/// レイアウトと見た目は `child` のまま
pub fn root(child: impl IntoElement) -> Root {
    Root {
        child: child.into_any_element(),
    }
}

/// [`root`] が返す要素
pub struct Root {
    child: AnyElement,
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
        None
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
        (self.child.request_layout(window, cx), ())
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
}
