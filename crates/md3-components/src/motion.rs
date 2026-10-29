//! 色の各成分をテーマのバネで補間する内部ヘルパー。

use gpui::{AnimationExt, AnyElement, ElementId, Hsla, IntoElement, SpringAnimation, SpringConfig};
use std::{cell::Cell, rc::Rc, sync::Arc};

/// 同じ ID の色を描画間で追跡する。gpui が reduce_motion を処理する。
pub(crate) fn color<E: IntoElement + 'static>(
    element: E,
    id: ElementId,
    target: Hsla,
    spring: SpringConfig,
    apply: impl FnOnce(E, Hsla) -> E + 'static,
) -> AnyElement {
    let id = Arc::new(id);
    // 外側のバネから内側へ request_layout されるので、最後に h と全成分を適用する。
    let value = Rc::new(Cell::new(target));
    let h_value = value.clone();
    let s_value = value.clone();
    let l_value = value.clone();
    element
        .with_spring(
            ElementId::NamedChild(id.clone(), "h".into()),
            SpringAnimation::new(spring).to(target.h),
            move |element, h| apply(element, Hsla { h, ..h_value.get() }),
        )
        .with_spring(
            ElementId::NamedChild(id.clone(), "s".into()),
            SpringAnimation::new(spring).to(target.s),
            move |element, s| {
                s_value.set(Hsla { s, ..s_value.get() });
                element
            },
        )
        .with_spring(
            ElementId::NamedChild(id.clone(), "l".into()),
            SpringAnimation::new(spring).to(target.l),
            move |element, l| {
                l_value.set(Hsla { l, ..l_value.get() });
                element
            },
        )
        .with_spring(
            ElementId::NamedChild(id, "a".into()),
            SpringAnimation::new(spring).to(target.a),
            move |element, a| {
                value.set(Hsla { a, ..value.get() });
                element
            },
        )
        .into_any_element()
}
