//! アクセシビリティ情報の付与。
//!
//! 部品は [`A11yNode`] を純関数で組み立て、[`A11yExt::a11y`] で要素に当てる。
//! gpui の a11y API の変更の影響をここに閉じ込める（docs/agents/a11y.md）。
//!
//! gpui の要素の API（`aria_*`）に無い状態（無効・modal・入力エラー・live region）は、
//! `a11y_synthetic_children` で渡される要素自身の AccessKit のノード（`gpui::accesskit`）に直接書く。

use gpui::{Role, SharedString, StatefulInteractiveElement, Toggled, accesskit};

/// 範囲を持つ値（Slider、Progress）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeValue {
    /// 現在の値。
    pub value: f64,
    /// 最小値。
    pub min: f64,
    /// 最大値。
    pub max: f64,
    /// 1 回の操作で変わる量。`None` なら操作できない値（Progress）。
    pub step: Option<f64>,
}

/// 部品がアクセシビリティツリーに出す情報。
#[derive(Clone, Debug, PartialEq)]
pub struct A11yNode {
    /// 役割。
    pub role: Role,
    /// 名前（読み上げられるラベル）。
    pub label: Option<SharedString>,
    /// 補足の説明。
    pub description: Option<SharedString>,
    /// 必須の入力か。
    pub required: bool,
    /// 開閉の状態。
    pub expanded: Option<bool>,
    /// メニューを開くか。
    pub has_popup_menu: bool,
    /// 切り替えの状態（Checkbox、Switch、トグルボタン）。
    pub toggled: Option<Toggled>,
    /// 選択の状態（RadioButton、SegmentedButton の項目）。
    pub selected: Option<bool>,
    /// 範囲を持つ値。
    pub range: Option<RangeValue>,
    /// グループ内の位置（1 始まり）とグループの大きさ。
    pub position: Option<(usize, usize)>,
    /// 無効状態か。ツリーに無効として出し、Tab でも止まらない。
    pub disabled: bool,
    /// 後ろの要素を操作できない（Dialog、モーダルな Sheet / Drawer）。
    pub modal: bool,
    /// 入力の値が不正か（TextField / Select のエラー）。
    pub invalid: bool,
    /// 内容が変わったら読み上げる（Snackbar。live region の polite）。
    pub live: bool,
}

impl A11yNode {
    /// 役割を指定して作る。
    pub fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            description: None,
            required: false,
            expanded: None,
            has_popup_menu: false,
            toggled: None,
            selected: None,
            range: None,
            position: None,
            disabled: false,
            modal: false,
            invalid: false,
            live: false,
        }
    }

    /// 名前を付ける。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 任意の名前を付ける。
    pub fn with_label(mut self, label: Option<SharedString>) -> Self {
        self.label = label.filter(|label| !label.is_empty());
        self
    }

    /// 補足の説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        let description = description.into();
        self.description = (!description.is_empty()).then_some(description);
        self
    }

    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// 開閉の状態を指定する。
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// メニューを開くか指定する。
    pub fn has_popup_menu(mut self, has_popup_menu: bool) -> Self {
        self.has_popup_menu = has_popup_menu;
        self
    }

    /// 切り替えの状態を付ける。
    pub fn toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// 選択の状態を付ける。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// 範囲を持つ値を付ける。
    pub fn range(mut self, range: RangeValue) -> Self {
        self.range = Some(range);
        self
    }

    /// グループ内の位置を付ける。
    pub fn position(mut self, index: usize, count: usize) -> Self {
        self.position = Some((index + 1, count));
        self
    }

    /// 無効状態にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// modal にする。
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// 入力の値が不正であることを示す。
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// 内容の変化を読み上げさせる（live region の polite）。
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
}

/// `bool` を切り替えの状態にする。
pub fn toggled(on: bool) -> Toggled {
    if on { Toggled::True } else { Toggled::False }
}

/// [`A11yNode`] を要素に当てる拡張トレイト。部品を持つ feature がすべて無効だと使われない。
pub trait A11yExt: StatefulInteractiveElement + Sized {
    /// アクセシビリティ情報を要素に付ける。
    fn a11y(self, node: A11yNode) -> Self {
        let mut this = self.role(node.role);
        if let Some(label) = node.label {
            this = this.aria_label(label);
        }
        if let Some(description) = node.description {
            this = this.aria_description(description);
        }
        if let Some(expanded) = node.expanded {
            this = this.aria_expanded(expanded);
        }
        if let Some(toggled) = node.toggled {
            this = this.aria_toggled(toggled);
        }
        if let Some(selected) = node.selected {
            this = this.aria_selected(selected);
        }
        if let Some(range) = node.range {
            this = this
                .aria_numeric_value(range.value)
                .aria_min_numeric_value(range.min)
                .aria_max_numeric_value(range.max);
            if let Some(step) = range.step {
                this = this.aria_numeric_value_step(step);
            }
        }
        if let Some((position, size)) = node.position {
            this = this.aria_position_in_set(position).aria_size_of_set(size);
        }
        if node.disabled
            || node.modal
            || node.invalid
            || node.live
            || node.required
            || node.has_popup_menu
        {
            this = this.a11y_synthetic_children(move |builder| {
                let raw = builder.parent_node();
                if node.disabled {
                    raw.set_disabled();
                }
                if node.modal {
                    raw.set_modal();
                }
                if node.invalid {
                    raw.set_invalid(accesskit::Invalid::True);
                }
                if node.required {
                    raw.set_required();
                }
                if node.has_popup_menu {
                    raw.set_has_popup(accesskit::HasPopup::Menu);
                }
                if node.live {
                    raw.set_live(accesskit::Live::Polite);
                }
            });
        }
        this
    }
}

impl<E: StatefulInteractiveElement> A11yExt for E {}
