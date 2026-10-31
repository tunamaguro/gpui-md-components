//! ツールチップの役割と説明。

use crate::keys::{DismissTooltip, FOCUS_TOOLTIP_CONTEXT};
use crate::{A11yExt, A11yNode, focus_handle};
use gpui::{
    AnyView, App, Bounds, Div, ElementId, Entity, InteractiveElement, Pixels, Point, Role,
    SharedString, Size, Stateful, StatefulInteractiveElement, Window, div, point, px,
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

/// キーボードフォーカスで出すツールチップの状態（APG の Tooltip パターン）。
///
/// gpui のツールチップはポインタでしか出ないので、部品が基準の要素の子として描く。
pub struct FocusTooltip {
    visible: bool,
    dismissed: Entity<bool>,
}

impl FocusTooltip {
    /// `id` は基準の要素の ID、`focus_visible` はそのキーボードフォーカスの表示。
    ///
    /// Esc で閉じた状態とポインタで得たフォーカスは、離れるまで出さない。キーを押した時点で
    /// `focus_visible` が真になるので、後者を外さないと見えないツールチップが Esc を受けてしまう。
    pub fn new(id: &ElementId, focus_visible: bool, window: &mut Window, cx: &mut App) -> Self {
        let dismissed = window.use_keyed_state(
            ElementId::NamedChild(id.clone().into(), "tooltip-dismissed".into()),
            cx,
            |_, _| false,
        );
        if !focus_visible {
            let focused = focus_handle(id.clone(), window, cx).is_focused(window);
            dismissed.update(cx, |dismissed, _| *dismissed = focused);
        }
        let visible = focus_visible && !*dismissed.read(cx);
        Self { visible, dismissed }
    }

    /// ツールチップを出すか。
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// 基準の要素に、Esc で閉じる操作を付ける。出ていないときは何も付けない（Esc を親へ通す）。
    pub fn attach<E: InteractiveElement>(self, element: E) -> E {
        if !self.visible {
            return element;
        }
        let dismissed = self.dismissed;
        element.key_context(FOCUS_TOOLTIP_CONTEXT).on_action(
            move |_: &DismissTooltip, window, cx| {
                dismissed.update(cx, |dismissed, _| *dismissed = true);
                window.refresh();
            },
        )
    }
}

/// ツールチップの位置。基準の要素の上に中央揃えで置き、入らなければ下に置く。左右はウィンドウに収める。
pub fn tooltip_origin(
    anchor: Bounds<Pixels>,
    size: Size<Pixels>,
    viewport: Size<Pixels>,
    spacing: Pixels,
) -> Point<Pixels> {
    let above = anchor.top() - spacing - size.height;
    let y = if above >= px(0.0) {
        above
    } else {
        anchor.bottom() + spacing
    };
    let x = (anchor.center().x - size.width / 2.0)
        .min(viewport.width - size.width)
        .max(px(0.0));
    point(x, y)
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
