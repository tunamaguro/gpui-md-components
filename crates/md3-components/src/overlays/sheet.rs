//! BottomSheet（モーダル）と SideSheet（Standard / モーダル）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};
use md3_theme::md3_tokens::component::{divider, sheet_bottom, side_sheet};
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Elevation, Md3ThemeExt};

use super::layer::{
    DismissHandler, InitialFocus, MODAL_PRIORITY, dismiss_and_restore, modal_container,
    overlay_focus, scrim, window_layer,
};
use crate::a11y::{A11yExt, A11yNode};
use crate::buttons::IconButton;
use crate::control::control;
use crate::{FocusRing, IconName, StyledExt};

/// 角丸の半径（`Full` はこの部品では使わない）。
fn radius(corner: CornerRadius) -> gpui::Pixels {
    match corner {
        CornerRadius::Fixed(radius) => radius,
        CornerRadius::Full => px(0.0),
    }
}

/// モーダルなボトムシート。開いている間だけ描く。
///
/// フォーカスの扱いは [`Dialog`](super::Dialog) と同じ（中に閉じ込め、Esc とスクリムで閉じ、閉じたら戻す）。
/// つまみ（drag handle）は、`handle_label` を付けると閉じるボタンになる。ドラッグでの開閉には対応しない。
#[derive(IntoElement)]
pub struct BottomSheet {
    scrollbar: bool,
    id: ElementId,
    label: SharedString,
    handle_label: Option<SharedString>,
    children: Vec<AnyElement>,
    on_dismiss: Option<DismissHandler>,
    inline: bool,
}

impl BottomSheet {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID とラベル（読み上げ用の名前）から作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            label: label.into(),
            handle_label: None,
            children: Vec::new(),
            on_dismiss: None,
            inline: false,
        }
    }

    /// つまみを閉じるボタンにする。`label` は読み上げ用の名前。
    pub fn handle_label(mut self, label: impl Into<SharedString>) -> Self {
        self.handle_label = Some(label.into());
        self
    }

    /// Esc、スクリムの押下、つまみで閉じるときの処理。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// 見本用に、スクリムなしでその場に描く（フォーカスは動かさない）。
    #[doc(hidden)]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::Dialog)
            .label(self.label.clone())
            .modal(!self.inline)
    }
}

impl ParentElement for BottomSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BottomSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let focus =
            (!self.inline).then(|| overlay_focus(&self.id, || InitialFocus::Within, window, cx));
        let dismiss = focus
            .as_ref()
            .and_then(|focus| dismiss_and_restore(self.on_dismiss.clone(), focus));
        let theme = &cx.md3().clone();
        let corner = radius(theme.corner(sheet_bottom::DOCKED_CONTAINER_SHAPE));
        let parent = Arc::new(self.id.clone());

        let bar = div()
            .w(px(sheet_bottom::DOCKED_DRAG_HANDLE_WIDTH))
            .h(px(sheet_bottom::DOCKED_DRAG_HANDLE_HEIGHT))
            .corner_radius(theme.corner(md3_theme::Shape::ExtraLarge))
            .bg(theme.color(sheet_bottom::DOCKED_DRAG_HANDLE_COLOR));
        let handle_space = px(layout::BOTTOM_SHEET_DRAG_HANDLE_VERTICAL_SPACE);
        let handle = match self.handle_label {
            Some(label) => {
                let id = ElementId::NamedChild(parent.clone(), "handle".into());
                let (base, control) = control(&id, false, true, None, window, cx);
                let ring = theme.corner(md3_theme::Shape::ExtraSmall);
                base.a11y(A11yNode::new(Role::Button).label(label))
                    .py(handle_space)
                    .child(bar)
                    .child(FocusRing::new(ring).visible(control.focus_visible))
                    .when_some(dismiss.clone(), |this, dismiss| {
                        this.on_click(move |_, window, cx| dismiss(window, cx))
                    })
                    .into_any_element()
            }
            None => div().py(handle_space).child(bar).into_any_element(),
        };

        let sheet = div()
            .id(self.id.clone())
            .a11y(node)
            .w_full()
            .max_w(px(layout::BOTTOM_SHEET_MAX_WIDTH))
            .max_h(window.viewport_size().height)
            .flex()
            .flex_col()
            .bg(theme.color(sheet_bottom::DOCKED_CONTAINER_COLOR))
            .rounded_t(corner)
            .shadow(theme.shadows(Elevation::from_dp(
                sheet_bottom::DOCKED_MODAL_CONTAINER_ELEVATION,
            )))
            .child(div().flex_none().flex().justify_center().child(handle))
            .child(crate::scroll::content(
                &self.id,
                self.children,
                self.scrollbar,
                window,
                cx,
            ));

        let Some(focus) = focus else {
            return sheet.into_any_element();
        };
        let sheet = modal_container(sheet, &focus, dismiss.clone(), cx);
        let layer = scrim(ElementId::NamedChild(parent, "scrim".into()), dismiss, cx)
            .flex()
            .flex_col()
            .justify_end()
            .items_center()
            .child(sheet);
        window_layer(layer, MODAL_PRIORITY, window).into_any_element()
    }
}

/// サイドシートの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SideSheetKind {
    /// 画面の一部として横に並ぶ（フォーカスは閉じ込めない）。
    #[default]
    Standard,
    /// スクリムの上に重なる。フォーカスの扱いは Dialog と同じ。
    Modal,
}

/// サイドシート（ウィンドウの右端）。
///
/// Standard はレイアウトの中に置く。Modal は開いている間だけ描く。
#[derive(IntoElement)]
pub struct SideSheet {
    scrollbar: bool,
    id: ElementId,
    headline: SharedString,
    kind: SideSheetKind,
    close_label: Option<SharedString>,
    children: Vec<AnyElement>,
    on_dismiss: Option<DismissHandler>,
    inline: bool,
}

impl SideSheet {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID と見出し（読み上げ用の名前を兼ねる）から作る。
    pub fn new(id: impl Into<ElementId>, headline: impl Into<SharedString>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            headline: headline.into(),
            kind: SideSheetKind::default(),
            close_label: None,
            children: Vec::new(),
            on_dismiss: None,
            inline: false,
        }
    }

    /// 種類を変える。
    pub fn kind(mut self, kind: SideSheetKind) -> Self {
        self.kind = kind;
        self
    }

    /// 見出しの横に閉じるボタンを出す。`label` は読み上げ用の名前。押すと `on_dismiss` を呼ぶ。
    pub fn close_button(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// 閉じるときの処理（閉じるボタン。Modal では Esc とスクリムの押下も）。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// 見本用に、Modal でもスクリムなしでその場に描く（フォーカスは動かさない）。
    #[doc(hidden)]
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = match self.kind {
            SideSheetKind::Standard => Role::Complementary,
            SideSheetKind::Modal => Role::Dialog,
        };
        A11yNode::new(role)
            .label(self.headline.clone())
            .modal(self.kind == SideSheetKind::Modal && !self.inline)
    }
}

impl ParentElement for SideSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let modal = self.kind == SideSheetKind::Modal && !self.inline;
        let focus = modal.then(|| overlay_focus(&self.id, || InitialFocus::Within, window, cx));
        let dismiss = match &focus {
            Some(focus) => dismiss_and_restore(self.on_dismiss.clone(), focus),
            None => self.on_dismiss.clone(),
        };
        let theme = &cx.md3().clone();
        let parent = Arc::new(self.id.clone());
        let space = px(layout::SIDE_SHEET_CONTENT_SPACE);

        let header = div()
            .flex_none()
            .flex()
            .items_start()
            .gap(space)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .pb(px(layout::SIDE_SHEET_HEADLINE_BOTTOM_SPACE))
                    .type_style(&theme.text_style(side_sheet::DOCKED_HEADLINE_FONT))
                    .text_color(theme.color(side_sheet::DOCKED_HEADLINE_COLOR))
                    .child(self.headline),
            )
            .when_some(self.close_label, |this, label| {
                let close = dismiss.clone();
                this.child(
                    IconButton::new(
                        ElementId::NamedChild(parent.clone(), "close".into()),
                        IconName::Close,
                        label,
                    )
                    .when_some(close, |button, close| {
                        button.on_click(move |_, window, cx| close(window, cx))
                    }),
                )
            });

        let mut sheet = div()
            .id(self.id.clone())
            .a11y(node)
            .w(px(side_sheet::DOCKED_CONTAINER_WIDTH))
            .h_full()
            .max_h(window.viewport_size().height)
            .flex_none()
            .flex()
            .flex_col()
            .p(space)
            .child(header)
            .child(crate::scroll::content(
                &self.id,
                self.children,
                self.scrollbar,
                window,
                cx,
            ));
        sheet = match self.kind {
            SideSheetKind::Standard => sheet
                .bg(theme.color(side_sheet::DOCKED_STANDARD_CONTAINER_COLOR))
                .border_l(px(divider::THICKNESS))
                .border_color(theme.color(side_sheet::DOCKED_DIVIDER_COLOR))
                .shadow(theme.shadows(Elevation::from_dp(
                    side_sheet::DOCKED_STANDARD_CONTAINER_ELEVATION,
                ))),
            SideSheetKind::Modal => {
                let corner = radius(theme.corner(side_sheet::DOCKED_MODAL_CONTAINER_SHAPE));
                sheet
                    .bg(theme.color(side_sheet::DOCKED_MODAL_CONTAINER_COLOR))
                    .rounded_l(corner)
                    .shadow(theme.shadows(Elevation::from_dp(
                        side_sheet::DOCKED_MODAL_CONTAINER_ELEVATION,
                    )))
            }
        };

        let Some(focus) = focus else {
            return sheet.into_any_element();
        };
        let sheet = modal_container(sheet, &focus, dismiss.clone(), cx);
        let layer = scrim(ElementId::NamedChild(parent, "scrim".into()), dismiss, cx)
            .flex()
            .justify_end()
            .child(sheet);
        window_layer(layer, MODAL_PRIORITY, window).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_sheets_are_dialogs() {
        assert_eq!(BottomSheet::new("b", "共有").a11y_node().role, Role::Dialog);
        let side = SideSheet::new("s", "フィルタ");
        assert_eq!(side.a11y_node().role, Role::Complementary);
        let modal = SideSheet::new("s", "フィルタ").kind(SideSheetKind::Modal);
        assert_eq!(modal.a11y_node().role, Role::Dialog);
        assert!(modal.a11y_node().modal);
        assert!(!side.a11y_node().modal);
        assert!(BottomSheet::new("b", "共有").a11y_node().modal);
        assert!(!BottomSheet::new("b", "共有").inline().a11y_node().modal);
    }
}
