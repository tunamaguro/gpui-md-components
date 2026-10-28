//! アクセシビリティ情報の付与。
//!
//! 部品は [`A11yNode`] を純関数で組み立て、[`A11yExt::a11y`] で要素に当てる。
//! gpui の a11y API の変更の影響をここに閉じ込める（docs/agents/a11y.md）。

use gpui::{Role, SharedString, StatefulInteractiveElement, Toggled};

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
    /// 切り替えの状態（Checkbox、Switch、トグルボタン）。
    pub toggled: Option<Toggled>,
    /// 選択の状態（RadioButton、SegmentedButton の項目）。
    pub selected: Option<bool>,
    /// 範囲を持つ値。
    pub range: Option<RangeValue>,
    /// グループ内の位置（1 始まり）とグループの大きさ。
    pub position: Option<(usize, usize)>,
    /// 無効状態か。gpui の a11y API に無効状態がないため、ツリーには出せない（Tab で止まらないことで表す）。
    pub disabled: bool,
}

impl A11yNode {
    /// 役割を指定して作る。
    pub fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            toggled: None,
            selected: None,
            range: None,
            position: None,
            disabled: false,
        }
    }

    /// 名前を付ける。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
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
}

/// `bool` を切り替えの状態にする。
pub fn toggled(on: bool) -> Toggled {
    if on { Toggled::True } else { Toggled::False }
}

/// [`A11yNode`] を要素に当てる拡張トレイト。
pub(crate) trait A11yExt: StatefulInteractiveElement + Sized {
    fn a11y(self, node: A11yNode) -> Self {
        let mut this = self.role(node.role);
        if let Some(label) = node.label {
            this = this.aria_label(label);
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
        this
    }
}

impl<E: StatefulInteractiveElement> A11yExt for E {}
