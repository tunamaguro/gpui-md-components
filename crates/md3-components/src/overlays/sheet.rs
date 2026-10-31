//! BottomSheet（モーダル）と SideSheet（Standard / モーダル）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Window, div,
    prelude::*, px,
};
use md3_theme::md3_tokens::component::{divider, sheet_bottom, side_sheet};
use md3_theme::md3_tokens::layout;
use md3_theme::{CornerRadius, Elevation, Md3ThemeExt};

use super::layer::scrim;
use crate::buttons::IconButton;
use crate::{FocusRing, IconName, StyledExt};
use gpui_headless::overlay::{
    DismissHandler, MODAL_PRIORITY, dismiss_and_restore, modal_container, window_layer,
};

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
    inner: gpui_headless::Dialog,
    handle_label: Option<SharedString>,
    children: Vec<AnyElement>,
    on_dismiss: Option<DismissHandler>,
}

impl BottomSheet {
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            inner: gpui_headless::Dialog::new(id.clone()),
            scrollbar: true,
            id,
            handle_label: None,
            children: Vec::new(),
            on_dismiss: None,
        }
    }

    delegate_accessible_name!(inner);

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
        self.inner = self.inner.modal(false);
        self
    }
}

impl ParentElement for BottomSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BottomSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (base, focus) = self.inner.build(window, cx);
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
                let mut button = gpui_headless::Button::new(id.clone(), label)
                    .group(format!("md3-{id}"))
                    .reveal_margin(crate::focus::reveal_margin());
                if let Some(dismiss) = dismiss.clone() {
                    button = button.on_click(move |_, window, cx| dismiss(window, cx));
                }
                let (base, control) = button.build(window, cx);
                let ring = theme.corner(md3_theme::Shape::ExtraSmall);
                base.py(handle_space)
                    .child(bar)
                    .child(FocusRing::new(ring).visible(control.focus_visible))
                    .into_any_element()
            }
            None => div().py(handle_space).child(bar).into_any_element(),
        };

        let sheet = base
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
    headline: Option<SharedString>,
    name: gpui_headless::AccessibleName,
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

    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            headline: None,
            name: gpui_headless::AccessibleName::default(),
            kind: SideSheetKind::default(),
            close_label: None,
            children: Vec::new(),
            on_dismiss: None,
            inline: false,
        }
    }

    /// 表示する見出しを設定し、既定の名前にする。
    pub fn headline(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.name.set_visible(text.clone());
        self.headline = Some(text);
        self
    }

    accessible_name!();

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

    fn headless(&self) -> gpui_headless::Dialog {
        let dialog = gpui_headless::Dialog::new(self.id.clone()).name(self.name.clone());
        match self.kind {
            SideSheetKind::Standard => dialog.complementary(),
            SideSheetKind::Modal => dialog.modal(!self.inline),
        }
    }
}

impl ParentElement for SideSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (base, focus) = self.headless().build(window, cx);
        let dismiss = match &focus {
            Some(focus) => dismiss_and_restore(self.on_dismiss.clone(), focus),
            None => self.on_dismiss.clone(),
        };
        let theme = &cx.md3().clone();
        let parent = Arc::new(self.id.clone());
        let space = px(layout::SIDE_SHEET_CONTENT_SPACE);

        let has_header = self.headline.is_some() || self.close_label.is_some();
        let header = div()
            .flex_none()
            .flex()
            .items_start()
            .gap(space)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .when_some(self.headline, |this, headline| {
                        this.pb(px(layout::SIDE_SHEET_HEADLINE_BOTTOM_SPACE))
                            .type_style(&theme.text_style(side_sheet::DOCKED_HEADLINE_FONT))
                            .text_color(theme.color(side_sheet::DOCKED_HEADLINE_COLOR))
                            .child(headline)
                    }),
            )
            .when_some(self.close_label, |this, label| {
                let close = dismiss.clone();
                this.child(
                    IconButton::new(
                        ElementId::NamedChild(parent.clone(), "close".into()),
                        IconName::Close,
                    )
                    .aria_label(label)
                    .when_some(close, |button, close| {
                        button.on_click(move |_, window, cx| close(window, cx))
                    }),
                )
            });

        let mut sheet = base
            .w(px(side_sheet::DOCKED_CONTAINER_WIDTH))
            .h_full()
            .max_h(window.viewport_size().height)
            .flex_none()
            .flex()
            .flex_col()
            .p(space)
            .when(has_header, |sheet| sheet.child(header))
            .child(crate::scroll::content(
                &self.id,
                self.children,
                self.scrollbar,
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
    use gpui::Role;

    #[test]
    fn bottom_sheet_name_is_optional() {
        assert_eq!(BottomSheet::new("sheet").inner.a11y_node().label, None);
        assert_eq!(
            BottomSheet::new("sheet")
                .aria_label("共有")
                .inner
                .a11y_node()
                .label
                .as_deref(),
            Some("共有")
        );
        assert_eq!(
            BottomSheet::new("sheet")
                .aria_label("")
                .inner
                .a11y_node()
                .label,
            None
        );
    }

    #[test]
    fn side_sheet_name_follows_headline_and_override() {
        let name = |sheet: SideSheet| sheet.headless().a11y_node().label;
        assert_eq!(name(SideSheet::new("s")), None);
        assert_eq!(
            name(SideSheet::new("s").headline("詳細")).as_deref(),
            Some("詳細")
        );
        assert_eq!(
            name(SideSheet::new("s").headline("詳細").aria_label("詳細情報")).as_deref(),
            Some("詳細情報")
        );
        assert_eq!(
            name(SideSheet::new("s").headline("詳細").aria_label("")),
            None
        );
    }

    #[test]
    fn modal_sheets_are_dialogs() {
        assert_eq!(
            BottomSheet::new("b")
                .aria_label("共有")
                .inner
                .a11y_node()
                .role,
            Role::Dialog
        );
        let side = SideSheet::new("s").headline("フィルタ");
        assert_eq!(side.headless().a11y_node().role, Role::Complementary);
        let modal = SideSheet::new("s")
            .headline("フィルタ")
            .kind(SideSheetKind::Modal);
        assert_eq!(modal.headless().a11y_node().role, Role::Dialog);
        assert!(modal.headless().a11y_node().modal);
        assert!(!side.headless().a11y_node().modal);
        assert!(
            BottomSheet::new("b")
                .aria_label("共有")
                .inner
                .a11y_node()
                .modal
        );
        assert!(
            !BottomSheet::new("b")
                .aria_label("共有")
                .inline()
                .inner
                .a11y_node()
                .modal
        );
    }
}
