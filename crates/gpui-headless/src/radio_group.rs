// Ported from Jetpack Compose (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt @ b8256a52
// Copyright 2020 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//! ラジオグループの状態と操作。
use crate::control::Roving;
use crate::{A11yExt, A11yNode, ControlState, control, toggled};
use gpui::{
    AnyElement, App, Div, ElementId, KeyDownEvent, Pixels, Role, SharedString, Stateful, Window,
    div, prelude::*, px,
};
use std::rc::Rc;

/// ラジオボタンの選択肢。
#[derive(Clone, Debug)]
pub struct RadioOption {
    label: SharedString,
    name: crate::AccessibleName,
    disabled: bool,
}

impl RadioOption {
    /// 表示する文字から選択肢を作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            name: crate::AccessibleName::new(label.clone()),
            label,
            disabled: false,
        }
    }
    accessible_name!();
    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// 見た目を持たないラジオボタンのグループ。
///
/// APG の Radio Group に従い、Tab で止まるのは選択中の項目（無ければ最初の項目）だけで、
/// 矢印キーでグループ内を移ると同時に選択する。
pub struct RadioGroup {
    id: ElementId,
    name: crate::AccessibleName,
    options: Vec<RadioOption>,
    selected: Option<usize>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
    on_change: Option<ChangeHandler>,
}

impl RadioGroup {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            options: Vec::new(),
            selected: None,
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
            on_change: None,
        }
    }

    accessible_name!();

    /// 選択肢を加える。
    pub fn option(mut self, label: impl Into<SharedString>) -> Self {
        self.options.push(RadioOption::new(label));
        self
    }

    /// 無効な選択肢を加える。
    pub fn disabled_option(mut self, label: impl Into<SharedString>) -> Self {
        self.options.push(RadioOption::new(label).disabled(true));
        self
    }

    /// 選択肢を加える。
    pub fn option_item(mut self, option: RadioOption) -> Self {
        self.options.push(option);
        self
    }

    /// 選択中の項目。
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }

    /// 項目のホバーなどに使うグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }

    /// 選択が変わったときの処理。引数は選ばれた項目の位置。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// グループのアクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::RadioGroup).with_name(&self.name)
    }

    /// 選択肢のアクセシビリティ情報。
    pub fn option_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let option = self.options.get(index)?;
        Some(
            A11yNode::new(Role::RadioButton)
                .with_name(&option.name)
                .toggled(toggled(self.selected == Some(index)))
                .position(index, self.options.len())
                .disabled(option.disabled),
        )
    }

    fn option_id(&self, index: usize) -> ElementId {
        ElementId::NamedChild(
            std::sync::Arc::new(self.id.clone()),
            SharedString::from(format!("{index}")),
        )
    }
}

/// 選択肢の要素と描画に使う状態。
pub struct RadioItem {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 選択肢の位置。
    pub index: usize,
    /// 選択肢のラベル。
    pub label: SharedString,
    /// 選択されているか。
    pub selected: bool,
}

impl RadioGroup {
    /// 選択肢の描画を受け取り、操作を付けたグループを返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(RadioItem, &mut Window, &mut App) -> AnyElement,
    ) -> Stateful<Div> {
        window.with_element_namespace(self.id.clone(), |window| {
            let roving = Roving::new(
                self.options
                    .iter()
                    .enumerate()
                    .map(|(index, item)| (self.option_id(index), item.disabled)),
                self.selected,
                window,
                cx,
            );
            let mut group = div().id(self.id.clone()).a11y(self.a11y_node());
            for (index, option) in self.options.iter().enumerate() {
                let id = self.option_id(index);
                let (mut base, state) = control(
                    &id,
                    option.disabled,
                    roving.is_tab_stop(index),
                    window,
                    cx,
                    self.reveal_margin,
                    format!("{}-{id}", self.group_prefix).into(),
                );
                if let Some(node) = self.option_a11y_node(index) {
                    base = base.a11y(node);
                }
                if !option.disabled
                    && let Some(on_change) = self.on_change.clone()
                {
                    let on_click = on_change.clone();
                    base = base.on_click(move |_, window, cx| on_click(index, window, cx));
                    let roving = roving.clone();
                    base = base.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        let forward = match event.keystroke.key.as_str() {
                            "right" | "down" => true,
                            "left" | "up" => false,
                            _ => return,
                        };
                        let Some(next) = roving.next(index, forward) else {
                            return;
                        };
                        cx.stop_propagation();
                        roving.focus(next, window, cx);
                        on_change(next, window, cx);
                    });
                }
                group = group.child(window.with_element_namespace(id, |window| {
                    render(
                        RadioItem {
                            base,
                            state,
                            index,
                            label: option.label.clone(),
                            selected: self.selected == Some(index),
                        },
                        window,
                        cx,
                    )
                }));
            }
            group
        })
    }
}
