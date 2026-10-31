//! カードの操作。

use crate::{
    A11yExt, A11yNode, ControlState, clear_focus_visible, focus_handle, is_focus_visible,
    reveal_on_focus,
};
use gpui::{
    App, ClickEvent, Div, ElementId, MouseButton, Pixels, Role, SharedString, Stateful, Window,
    prelude::*,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// カードの土台。押せないカードは利用者の div をそのまま保つ。
pub enum CardBase {
    /// 装飾の入れ物。
    Plain(Div),
    /// 操作可能な入れ物。
    Clickable(Stateful<Div>, ControlState),
}

/// 見た目を持たないカード。
#[derive(Default)]
pub struct Card {
    clickable: Option<(ElementId, ClickHandler)>,
    label: Option<SharedString>,
    disabled: bool,
    reveal_margin: Pixels,
    group: Option<SharedString>,
}
impl Card {
    /// 装飾のカードを作る。
    pub fn new() -> Self {
        Self::default()
    }
    /// 押せるカードにする。
    pub fn on_click(
        mut self,
        id: impl Into<ElementId>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.clickable = Some((id.into(), Box::new(handler)));
        self
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 押せるカードを無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// ホバーなどに使うグループ名を指定する。
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }
    /// アクセシビリティツリーに出す情報。装飾のカードは出さない。
    pub fn a11y_node(&self) -> Option<A11yNode> {
        self.clickable.as_ref().map(|_| {
            A11yNode::new(Role::Button)
                .with_label(self.label.clone())
                .disabled(self.disabled)
        })
    }
    /// 利用者の要素に操作を付け、描画用の状態とともに返す。
    pub fn build(self, base: Div, window: &mut Window, cx: &mut App) -> CardBase {
        let node = self.a11y_node();
        let (Some((id, handler)), Some(node)) = (self.clickable, node) else {
            return CardBase::Plain(base);
        };
        let group = self.group.unwrap_or_else(|| format!("control-{id}").into());
        let mut base = base
            .relative()
            .id(id.clone())
            .group(group.clone())
            .a11y(node);
        let focus_visible = if self.disabled {
            false
        } else {
            let handle = focus_handle(id.clone(), window, cx);
            base = base
                .track_focus(&handle)
                .on_mouse_down(MouseButton::Left, {
                    let handle = handle.clone();
                    move |_, window, cx| clear_focus_visible(&handle, window, cx)
                })
                .child(reveal_on_focus(
                    &id,
                    handle.clone(),
                    window,
                    cx,
                    self.reveal_margin,
                ))
                .cursor_pointer()
                .on_click(handler);
            is_focus_visible(&handle, window, cx)
        };
        CardBase::Clickable(
            base,
            ControlState {
                group,
                focus_visible,
                disabled: self.disabled,
            },
        )
    }
}
