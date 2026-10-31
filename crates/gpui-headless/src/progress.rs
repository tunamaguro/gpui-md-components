//! 進み具合のアクセシビリティ情報。

use gpui::{Div, ElementId, Role, SharedString, Stateful, div, prelude::*};

use crate::{A11yExt, A11yNode, RangeValue};

/// 進み具合。値が `None` のときは不定。
pub struct Progress {
    id: ElementId,
    value: Option<f32>,
    label: Option<SharedString>,
}

impl Progress {
    /// ID と値から作る。
    pub fn new(id: impl Into<ElementId>, value: Option<f32>) -> Self {
        Self {
            id: id.into(),
            value,
            label: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// アクセシビリティ情報。値は 0〜1 に収める。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(Role::ProgressIndicator);
        node.label = self.label.clone();
        match self.value {
            Some(value) => node.range(RangeValue {
                value: f64::from(value.clamp(0.0, 1.0)),
                min: 0.0,
                max: 1.0,
                step: None,
            }),
            None => node,
        }
    }

    /// 進み具合の土台を作る。
    pub fn build(&self) -> Stateful<Div> {
        div().id(self.id.clone()).a11y(self.a11y_node())
    }
}
