// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt @ b8256a52
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! スイッチの状態と操作。

use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    App, Div, ElementId, MouseButton, Pixels, Role, SharedString, Stateful, Subscription, Window,
    prelude::*, px,
};

/// 押下・ホバーと、要素の寿命に結び付いた押下キャンセル。
struct SwitchInteraction {
    pressed: bool,
    hovered: bool,
    _blur_subscription: Subscription,
}

type ChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// スイッチの描画に使う状態。
pub struct SwitchState {
    /// 操作要素に共通する状態。
    pub control: ControlState,
    /// 押下中か。
    pub pressed: bool,
    /// ホバー中か。
    pub hovered: bool,
}

/// 見た目を持たないスイッチ。
pub struct Switch {
    id: ElementId,
    selected: bool,
    label: Option<SharedString>,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    reveal_margin: Pixels,
    group: SharedString,
}
impl Switch {
    /// ID と状態から作る。
    pub fn new(id: impl Into<ElementId>, selected: bool) -> Self {
        let id = id.into();
        Self {
            group: format!("control-{id}").into(),
            id,
            selected,
            label: None,
            disabled: false,
            on_change: None,
            reveal_margin: px(0.0),
        }
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 現在の状態を返す。
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// 値が変わるときの処理を設定する。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// ホバーなどに使うグループ名を指定する。
    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = group.into();
        self
    }
    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(Role::Switch)
            .toggled(toggled(self.selected))
            .disabled(self.disabled);
        node.label = self.label.clone();
        node
    }
    /// 操作を付けた要素と描画用の状態を返す。
    pub fn build(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, SwitchState) {
        let node = self.a11y_node();
        let (mut base, control) = control(
            &self.id,
            self.disabled,
            true,
            window,
            cx,
            self.reveal_margin,
            self.group,
        );
        let focus_handle = crate::focus_handle(self.id.clone(), window, cx);
        let interaction = window.use_keyed_state(
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "interaction".into()),
            cx,
            |window, cx| SwitchInteraction {
                pressed: false,
                hovered: false,
                _blur_subscription: cx.on_blur(
                    &focus_handle,
                    window,
                    |state: &mut SwitchInteraction, _, cx| {
                        // フォーカス移動後はキー解放が届かないため、ここで押下を取り消す。
                        if state.pressed {
                            state.pressed = false;
                            cx.notify();
                        }
                    },
                ),
            },
        );
        let pressed = !self.disabled && interaction.read(cx).pressed;
        let hovered = interaction.read(cx).hovered;
        if !self.disabled {
            let down = interaction.clone();
            let up = interaction.clone();
            let outside = interaction.clone();
            let key_down = interaction.clone();
            let key_up = interaction.clone();
            base = base
                .on_key_down(move |event, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        key_down.update(cx, |s, cx| {
                            s.pressed = true;
                            cx.notify();
                        });
                    }
                })
                .on_key_up(move |event, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                        key_up.update(cx, |s, cx| {
                            s.pressed = false;
                            cx.notify();
                        });
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    down.update(cx, |s, cx| {
                        s.pressed = true;
                        cx.notify();
                    })
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up.update(cx, |s, cx| {
                        s.pressed = false;
                        cx.notify();
                    })
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    outside.update(cx, |s, cx| {
                        s.pressed = false;
                        cx.notify();
                    })
                })
                .on_hover(move |hovered, _, cx| {
                    interaction.update(cx, |s, cx| {
                        s.hovered = *hovered;
                        cx.notify();
                    })
                });
        }
        let base = base.a11y(node).when_some(
            self.on_change.filter(|_| !self.disabled),
            |base, handler| {
                let next = !self.selected;
                base.on_click(move |_, window, cx| handler(next, window, cx))
            },
        );
        (
            base,
            SwitchState {
                control,
                pressed,
                hovered,
            },
        )
    }
}
