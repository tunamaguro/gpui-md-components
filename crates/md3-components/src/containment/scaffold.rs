// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Scaffold.kt @ 2a941dd11aa
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! 画面のバー、本文、FAB と Snackbar の配置。

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::layout;

/// FAB の横方向の位置。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabPosition {
    /// 左端。
    Start,
    /// 中央。
    Center,
    /// 右端（既定）。
    #[default]
    End,
}

/// 画面の各領域を並べる、役割を持たないレイアウト。
#[derive(IntoElement)]
pub struct Scaffold {
    id: ElementId,
    top: Option<AnyElement>,
    bottom: Option<AnyElement>,
    fab: Option<AnyElement>,
    position: FabPosition,
    content: Vec<AnyElement>,
    #[cfg(feature = "overlays")]
    snackbar: Option<gpui::Entity<crate::overlays::SnackbarHost>>,
}
impl Scaffold {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            top: None,
            bottom: None,
            fab: None,
            position: FabPosition::default(),
            content: Vec::new(),
            #[cfg(feature = "overlays")]
            snackbar: None,
        }
    }
    /// 上部バーを指定する。
    pub fn top_bar(mut self, bar: impl IntoElement) -> Self {
        self.top = Some(bar.into_any_element());
        self
    }
    /// 下部バーを指定する。
    pub fn bottom_bar(mut self, bar: impl IntoElement) -> Self {
        self.bottom = Some(bar.into_any_element());
        self
    }
    /// FAB を指定する。
    pub fn floating_action_button(mut self, fab: impl IntoElement) -> Self {
        self.fab = Some(fab.into_any_element());
        self
    }
    /// FAB の横方向の位置を指定する。
    pub fn fab_position(mut self, position: FabPosition) -> Self {
        self.position = position;
        self
    }
    /// Snackbar を FAB の上に表示するホストを指定する。
    #[cfg(feature = "overlays")]
    pub fn snackbar_host(mut self, host: &gpui::Entity<crate::overlays::SnackbarHost>) -> Self {
        self.snackbar = Some(host.clone());
        self
    }
}
impl ParentElement for Scaffold {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements);
    }
}
impl RenderOnce for Scaffold {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let overlay = div().absolute().inset_0().flex().flex_col().justify_end();
        #[cfg(feature = "overlays")]
        let overlay = overlay.when_some(self.snackbar, |overlay, host| {
            host.update(_cx, |host, _| host.inline = true);
            overlay.child(host)
        });
        let overlay = overlay.when_some(self.fab, |overlay, fab| {
            let wrapper = div()
                .w_full()
                .flex()
                .px(px(layout::SCAFFOLD_FAB_SPACE))
                .pb(px(layout::SCAFFOLD_FAB_SPACE));
            let wrapper = match self.position {
                FabPosition::Start => wrapper.justify_start(),
                FabPosition::Center => wrapper.justify_center(),
                FabPosition::End => wrapper.justify_end(),
            };
            overlay.child(wrapper.child(fab))
        });
        div()
            .id(self.id)
            .size_full()
            .flex()
            .flex_col()
            .children(self.top)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(
                        div()
                            .size_full()
                            .min_w_0()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .children(self.content),
                    )
                    .child(overlay),
            )
            .children(self.bottom)
    }
}
