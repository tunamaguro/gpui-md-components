// 配置・余白・色・文言は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt
//         (`VerticalPeriodToggle`、`ToggleItem`、`PeriodTogglePaddingSmall`)
//         compose/material3/material3/src/androidMain/res/values/strings.xml (`m3c_time_picker_period_toggle_description`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! 時刻の部品で共有する型と、午前・午後の切り替え

use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, ParentElement, Refineable, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder,
};
use gpui_headless::radio::{Radio, RadioGroup};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::time_input as tokens;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring};
use md3_theme::{SharedTheme, dp};

use crate::shape::Shaped;
use crate::util;

/// 午前・午後の切り替えと隣の要素の間の余白と、切り替えの 2 つの項目の間の余白 (Compose の `PeriodTogglePaddingSmall`)
pub(crate) const PERIOD_TOGGLE_SPACE: Dp = Dp(4.0);

/// 時刻
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Time {
    /// 時。0 から 23
    pub hour: u8,
    /// 分。0 から 59
    pub minute: u8,
}

impl Time {
    /// `hour` 時 `minute` 分の時刻を作る。`hour` は 0 から 23、`minute` は 0 から 59 にすること
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }
}

/// 時刻の部品の時の表し方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HourFormat {
    /// 1 時から 12 時と、午前・午後で表す
    #[default]
    Hour12,
    /// 0 時から 23 時で表す
    Hour24,
}

/// 午前・午後を選ばれたときに呼ぶハンドラ。午後なら `true` を渡す
pub(crate) type PeriodHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// 午前・午後を縦に並べた切り替え (Compose の更新後の `VerticalPeriodToggle`)
///
/// 幅は 52dp で、高さと外側の余白は呼び出し側が付ける。`pm` が `true` なら午後を選んだ状態で描く。
/// `read_only` なら項目を無効にする。`on_change` が無ければ操作されても何もしない
pub(crate) fn period_toggle(
    id: &ElementId,
    theme: &SharedTheme,
    pm: bool,
    read_only: bool,
    on_change: Option<PeriodHandler>,
    window: &mut Window,
    cx: &mut App,
) -> RadioGroup<bool> {
    let item = |id_suffix: &'static str,
                label: &'static str,
                value: bool,
                window: &mut Window,
                cx: &mut App| {
        let item_id = ElementId::from((id.clone(), id_suffix));
        let focus_handle = util::keyed_focus_handle(&item_id, window, cx);
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let checked = value == pm;
        let (container, content, shape) = if checked {
            (
                ColorRole::PrimaryContainer,
                ColorRole::OnPrimaryContainer,
                ShapeKey::CornerMedium,
            )
        } else {
            (
                ColorRole::SurfaceContainerLowest,
                tokens::PERIOD_SELECTOR_UNSELECTED_LABEL_TEXT_COLOR,
                ShapeKey::CornerFull,
            )
        };
        let mut font = theme.text_style(tokens::PERIOD_SELECTOR_LABEL_TEXT_FONT);
        font.color = Some(theme.color(content));
        if checked {
            font.font_weight = Some(FontWeight::BOLD);
        }
        let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
        let mut radio = Radio::new(item_id, value)
            .disabled(read_only)
            .track_focus(&focus_handle)
            .relative()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .child(Shaped::new(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(theme.color(container)),
                theme.clone(),
                shape,
            ))
            .child(div().relative().child(label))
            .aria_label(label);
        if focus_visible {
            radio = radio.child(
                Shaped::new(
                    div()
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .right(-outset)
                        .bottom(-outset)
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(focus_ring::COLOR)),
                    theme.clone(),
                    shape,
                )
                .outset(outset),
            );
        }
        radio.text_style().refine(&font);
        radio
    };
    let am = item("am", "AM", false, window, cx);
    let pm_item = item("pm", "PM", true, window, cx);
    RadioGroup::new((id.clone(), "period"), Some(pm))
        .when_some(on_change, |this, on_change| {
            this.on_change(move |pm: &bool, window, cx| on_change(*pm, window, cx))
        })
        .item(am)
        .item(pm_item)
        .aria_label("Select AM or PM")
        .flex()
        .flex_col()
        .flex_none()
        .gap(dp(PERIOD_TOGGLE_SPACE))
        .w(dp(tokens::PERIOD_SELECTOR_CONTAINER_WIDTH))
}
