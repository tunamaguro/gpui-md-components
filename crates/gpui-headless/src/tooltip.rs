//! ツールチップの役割と説明。

use crate::{A11yExt, A11yNode};
use gpui::{
    AnyView, App, Div, ElementId, Role, SharedString, Stateful, StatefulInteractiveElement, Window,
    div, prelude::*,
};

/// ツールチップの本体。文は役割 `Tooltip` の名前になる。
pub struct Tooltip {
    id: ElementId,
    text: SharedString,
}

impl Tooltip {
    /// ID と読み上げる文から作る。
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Tooltip).label(self.text.clone())
    }

    /// 役割と名前を付けた土台を返す。
    pub fn build(self) -> Stateful<Div> {
        let node = self.a11y_node();
        div().id(self.id).a11y(node)
    }
}

/// ツールチップと同じ文を要素の説明にも入れる。
pub trait DescribedTooltip: StatefulInteractiveElement + Sized {
    /// gpui のツールチップはキーボードのフォーカスでは出ないため、説明として読み上げられるようにする。
    fn described_tooltip(
        self,
        text: impl Into<SharedString>,
        build: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        self.aria_description(text.into()).tooltip(build)
    }
}

impl<E: StatefulInteractiveElement> DescribedTooltip for E {}
