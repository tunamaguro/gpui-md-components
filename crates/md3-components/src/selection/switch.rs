// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt @ b8256a52
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! Switch。

use gpui::{
    AnimationExt, App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation,
    SpringPlayback, Window, div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::switch as tokens;
use md3_theme::{CornerRadius, Md3ThemeExt, MotionToken};

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// 選択状態ごとの色（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SwitchColors {
    pub track: ColorRole,
    pub track_outline: Option<ColorRole>,
    pub handle: ColorRole,
    pub hovered_handle: ColorRole,
    pub icon: ColorRole,
}

impl SwitchColors {
    pub(crate) fn of(selected: bool) -> Self {
        if selected {
            Self {
                track: tokens::SELECTED_TRACK_COLOR,
                track_outline: None,
                handle: tokens::SELECTED_HANDLE_COLOR,
                hovered_handle: tokens::SELECTED_HOVER_HANDLE_COLOR,
                icon: tokens::SELECTED_ICON_COLOR,
            }
        } else {
            Self {
                track: tokens::UNSELECTED_TRACK_COLOR,
                track_outline: Some(tokens::UNSELECTED_TRACK_OUTLINE_COLOR),
                handle: tokens::UNSELECTED_HANDLE_COLOR,
                hovered_handle: tokens::UNSELECTED_HOVER_HANDLE_COLOR,
                icon: tokens::UNSELECTED_ICON_COLOR,
            }
        }
    }
}

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// スイッチ。ラベル（読み上げ用の名前）が必須。
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    selected: bool,
    label: SharedString,
    icons: bool,
    disabled: bool,
    on_change: Option<ChangeHandler>,
}

impl Switch {
    /// ID、状態、ラベルから作る。
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            icons: false,
            disabled: false,
            on_change: None,
        }
    }

    /// つまみにアイコン（オンはチェック、オフはバツ）を出す。
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 切り替えたときの処理。引数は次の状態。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut headless = gpui_headless::Switch::new(self.id.clone(), self.selected, self.label)
            .disabled(self.disabled)
            .group(format!("md3-{}", self.id))
            .reveal_margin(crate::focus::reveal_margin());
        if let Some(handler) = self.on_change {
            headless = headless.on_change(handler);
        }
        let (base, state) = headless.build(window, cx);
        let pressed = state.pressed;
        let hovered = state.hovered;
        let control = state.control;
        let theme = cx.md3();
        let colors = SwitchColors::of(self.selected);
        let track_radius = theme.corner(tokens::TRACK_SHAPE);
        let disabled = self.disabled;

        let track_color = if disabled {
            let role = if self.selected {
                tokens::DISABLED_SELECTED_TRACK_COLOR
            } else {
                tokens::DISABLED_UNSELECTED_TRACK_COLOR
            };
            theme.color(role).opacity(tokens::DISABLED_TRACK_OPACITY)
        } else {
            theme.color(colors.track)
        };
        let mut handle_color = match (disabled, self.selected) {
            (true, true) => theme
                .color(tokens::DISABLED_SELECTED_HANDLE_COLOR)
                .opacity(tokens::DISABLED_SELECTED_HANDLE_OPACITY),
            (true, false) => theme
                .color(tokens::DISABLED_UNSELECTED_HANDLE_COLOR)
                .opacity(tokens::DISABLED_UNSELECTED_HANDLE_OPACITY),
            (false, _) => theme.color(colors.handle),
        };
        let handle_size = if self.selected || self.icons {
            tokens::SELECTED_HANDLE_WIDTH
        } else {
            tokens::UNSELECTED_HANDLE_WIDTH
        };
        let handle_size = if pressed {
            tokens::PRESSED_HANDLE_WIDTH
        } else {
            handle_size
        };
        if !disabled && (pressed || hovered || control.focus_visible) {
            handle_color = theme.color(colors.hovered_handle);
        }
        let spatial = theme.spring(MotionToken::FastSpatial);
        let effects = theme.spring(MotionToken::FastEffects);
        let playback = if pressed {
            SpringPlayback::Completed
        } else {
            SpringPlayback::Running
        };
        let mut handle = div()
            .debug_selector(|| format!("md3-switch-handle-{}", self.id))
            .absolute()
            .rounded_full()
            .flex()
            .items_center()
            .justify_center();
        if self.icons {
            let (icon, size) = if self.selected {
                (IconName::Check, tokens::SELECTED_ICON_SIZE)
            } else {
                (IconName::Close, tokens::UNSELECTED_ICON_SIZE)
            };
            let color = if disabled {
                let role = if self.selected {
                    tokens::DISABLED_SELECTED_ICON_COLOR
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_COLOR
                };
                let opacity = if self.selected {
                    tokens::DISABLED_SELECTED_ICON_OPACITY
                } else {
                    tokens::DISABLED_UNSELECTED_ICON_OPACITY
                };
                theme.color(role).opacity(opacity)
            } else {
                theme.color(colors.icon)
            };
            handle = handle.child(crate::motion::color(
                Icon::new(icon).size(px(size)),
                "switch-icon-color".into(),
                color,
                effects,
                |icon, color| icon.tint(color),
            ));
        }

        // Compose の押下時は位置と寸法を snap、離した後は同じバネの状態から戻す。
        let offset = if pressed {
            if self.selected {
                tokens::TRACK_WIDTH - tokens::PRESSED_HANDLE_WIDTH - tokens::TRACK_OUTLINE_WIDTH
            } else {
                tokens::TRACK_OUTLINE_WIDTH
            }
        } else if self.selected {
            tokens::TRACK_WIDTH
                - tokens::SELECTED_HANDLE_WIDTH
                - (tokens::TRACK_HEIGHT - tokens::SELECTED_HANDLE_WIDTH) / 2.0
        } else {
            (tokens::TRACK_HEIGHT - handle_size) / 2.0
        };
        let handle = handle
            .with_spring(
                "switch-size",
                SpringAnimation::new(spatial)
                    .to(handle_size)
                    .playback(playback),
                |handle, size| {
                    handle
                        .size(px(size))
                        .top(px((tokens::TRACK_HEIGHT - size) / 2.0))
                },
            )
            .with_spring(
                "switch-offset",
                SpringAnimation::new(spatial).to(offset).playback(playback),
                |handle, offset| handle.map_element(|handle| handle.left(px(offset))),
            );
        let handle = crate::motion::color(
            handle,
            "switch-handle-color".into(),
            handle_color,
            effects,
            |handle, color| {
                handle.map_element(|handle| handle.map_element(|handle| handle.bg(color)))
            },
        );
        let outline = colors
            .track_outline
            .map(|role| {
                if disabled {
                    theme
                        .color(tokens::DISABLED_UNSELECTED_TRACK_OUTLINE_COLOR)
                        .opacity(tokens::DISABLED_TRACK_OPACITY)
                } else {
                    theme.color(role)
                }
            })
            .unwrap_or_else(|| track_color.opacity(0.0));
        let track = crate::motion::color(
            div()
                .size_full()
                .corner_radius(track_radius)
                .border(px(tokens::TRACK_OUTLINE_WIDTH)),
            "switch-outline".into(),
            outline,
            effects,
            |track, color| track.border_color(color),
        );
        let track = crate::motion::color(
            div().size_full().corner_radius(track_radius).child(track),
            "switch-track-color".into(),
            track_color,
            effects,
            |track, color| track.bg(color),
        );
        let layer_size = tokens::STATE_LAYER_SIZE;
        let layer = div()
            .absolute()
            .top(px((tokens::TRACK_HEIGHT - layer_size) / 2.0))
            .size(px(layer_size))
            .child(
                StateLayer::new(
                    SharedString::from(format!("{}-layer", control.group)),
                    control.group.clone(),
                    colors.hovered_handle,
                )
                .radius(CornerRadius::Full)
                .focused(control.focus_visible)
                .disabled(disabled),
            )
            .with_spring(
                "switch-layer-position",
                SpringAnimation::new(spatial)
                    .to(if self.selected {
                        tokens::TRACK_WIDTH - tokens::TRACK_HEIGHT
                    } else {
                        0.0
                    })
                    .playback(playback),
                move |layer, offset| {
                    layer.left(px(offset + (tokens::TRACK_HEIGHT - layer_size) / 2.0))
                },
            );

        base.w(px(tokens::TRACK_WIDTH))
            .h(px(tokens::TRACK_HEIGHT))
            .child(track)
            .child(layer)
            .child(handle)
            .child(FocusRing::new(track_radius).visible(control.focus_visible))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_track_is_primary() {
        assert_eq!(SwitchColors::of(true).track, ColorRole::Primary);
        assert!(SwitchColors::of(false).track_outline.is_some());
    }
}
