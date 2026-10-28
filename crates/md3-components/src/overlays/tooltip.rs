//! Tooltip（Plain / Rich）。
//!
//! gpui のツールチップ（`.tooltip()` / `.hoverable_tooltip()`）の中身として使う。表示の遅延と位置
//! （ポインタの近く）は gpui が決める。gpui のツールチップはポインタを重ねたときだけ出て、キーボードの
//! フォーカスでは出ないため、基準の要素の説明（`aria_description`）にも同じ文を入れること
//! （[`TooltipExt::md3_tooltip`] は両方を行う）。

use std::rc::Rc;

use gpui::{
    AnyView, App, Context, ElementId, IntoElement, Render, Role, SharedString,
    StatefulInteractiveElement, Window, div, prelude::*, px,
};
use md3_theme::md3_tokens::component::{plain_tooltip, rich_tooltip};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt};

use super::layer::DismissHandler;
use crate::StyledExt;
use crate::buttons::{Button, ButtonVariant};

/// Plain ツールチップ（短い説明）。
pub struct PlainTooltip {
    text: SharedString,
}

impl PlainTooltip {
    /// 文から作る。
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }

    /// `.tooltip()` に渡す関数を作る。
    pub fn builder(
        text: impl Into<SharedString>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let text = text.into();
        move |_, cx| cx.new(|_| PlainTooltip::new(text.clone())).into()
    }
}

impl Render for PlainTooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3();
        div()
            .id("md3-plain-tooltip")
            .role(Role::Tooltip)
            .aria_label(self.text.clone())
            .min_h(px(layout::TOOLTIP_MIN_HEIGHT))
            .min_w(px(layout::TOOLTIP_MIN_WIDTH))
            .max_w(px(layout::PLAIN_TOOLTIP_MAX_WIDTH))
            .px(px(layout::PLAIN_TOOLTIP_HORIZONTAL_SPACE))
            .py(px(layout::PLAIN_TOOLTIP_VERTICAL_SPACE))
            .flex()
            .items_center()
            .bg(theme.color(plain_tooltip::CONTAINER_COLOR))
            .corner_radius(theme.corner(plain_tooltip::CONTAINER_SHAPE))
            .type_style(&theme.text_style(plain_tooltip::SUPPORTING_TEXT_FONT))
            .text_color(theme.color(plain_tooltip::SUPPORTING_TEXT_COLOR))
            .child(self.text.clone())
    }
}

/// Rich ツールチップ（見出し、本文、操作）。操作があるときは `.hoverable_tooltip()` に渡す。
#[derive(Clone)]
pub struct RichTooltip {
    subhead: Option<SharedString>,
    text: SharedString,
    actions: Vec<(SharedString, DismissHandler)>,
}

impl RichTooltip {
    /// 本文から作る。
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            subhead: None,
            text: text.into(),
            actions: Vec::new(),
        }
    }

    /// 見出し。
    pub fn subhead(mut self, subhead: impl Into<SharedString>) -> Self {
        self.subhead = Some(subhead.into());
        self
    }

    /// 操作（Text ボタン）を足す。
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.actions.push((label.into(), Rc::new(handler)));
        self
    }

    /// `.tooltip()` / `.hoverable_tooltip()` に渡す関数を作る。
    pub fn builder(self) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        move |_, cx| {
            let tooltip = self.clone();
            cx.new(|_| tooltip).into()
        }
    }
}

impl Render for RichTooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.md3();
        let subhead_style = theme.text_style(rich_tooltip::SUBHEAD_FONT);
        let text_style = theme.text_style(rich_tooltip::SUPPORTING_TEXT_FONT);
        // Compose は見出しと本文の 1 行目をベースラインで置く（HeightToSubheadFirstLine など）。gpui は
        // ベースラインで置けないため、行の下端をその位置に合わせる。
        let subhead_top =
            (px(layout::RICH_TOOLTIP_SUBHEAD_FIRST_LINE) - subhead_style.line_height).max(px(0.0));
        let text_top =
            (px(layout::RICH_TOOLTIP_TEXT_FIRST_LINE) - text_style.line_height).max(px(0.0));
        let label = match &self.subhead {
            Some(subhead) => format!("{subhead}: {}", self.text),
            None => self.text.to_string(),
        };
        let has_actions = !self.actions.is_empty();
        div()
            .id("md3-rich-tooltip")
            .role(Role::Tooltip)
            .aria_label(label)
            .max_w(px(layout::RICH_TOOLTIP_MAX_WIDTH))
            .min_w(px(layout::TOOLTIP_MIN_WIDTH))
            .px(px(layout::RICH_TOOLTIP_HORIZONTAL_SPACE))
            .flex()
            .flex_col()
            .bg(theme.color(rich_tooltip::CONTAINER_COLOR))
            .corner_radius(theme.corner(rich_tooltip::CONTAINER_SHAPE))
            .shadow(theme.shadows(Elevation::from_dp(rich_tooltip::CONTAINER_ELEVATION)))
            .when_some(self.subhead.clone(), |this, subhead| {
                this.child(
                    div()
                        .pt(subhead_top)
                        .type_style(&subhead_style)
                        .text_color(theme.color(rich_tooltip::SUBHEAD_COLOR))
                        .child(subhead),
                )
            })
            .child(
                div()
                    // 見出しも操作もなければ Plain と同じ上下の余白（Compose の textVerticalPadding）。
                    .map(|this| {
                        if self.subhead.is_none() && !has_actions {
                            this.py(px(layout::PLAIN_TOOLTIP_VERTICAL_SPACE))
                        } else {
                            this.pt(text_top)
                                .pb(px(layout::RICH_TOOLTIP_TEXT_BOTTOM_SPACE))
                        }
                    })
                    .type_style(&text_style)
                    .text_color(theme.color(rich_tooltip::SUPPORTING_TEXT_COLOR))
                    .child(self.text.clone()),
            )
            .when(has_actions, |this| {
                this.child(
                    div()
                        .min_h(px(layout::RICH_TOOLTIP_ACTION_MIN_HEIGHT))
                        .pb(px(layout::RICH_TOOLTIP_ACTION_BOTTOM_SPACE))
                        .flex()
                        .items_center()
                        .children(self.actions.iter().enumerate().map(
                            |(index, (label, handler))| {
                                let handler = handler.clone();
                                Button::new(
                                    ElementId::Name(
                                        format!("md3-rich-tooltip-action-{index}").into(),
                                    ),
                                    label.clone(),
                                )
                                .variant(ButtonVariant::Text)
                                .on_click(move |_, window, cx| handler(window, cx))
                            },
                        )),
                )
            })
    }
}

/// 要素に Plain ツールチップを付ける拡張トレイト。
pub trait TooltipExt: StatefulInteractiveElement + Sized {
    /// Plain ツールチップを付け、同じ文を要素の説明（読み上げ）にも入れる。
    fn md3_tooltip(self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.aria_description(text.clone())
            .tooltip(PlainTooltip::builder(text))
    }
}

impl<E: StatefulInteractiveElement> TooltipExt for E {}
