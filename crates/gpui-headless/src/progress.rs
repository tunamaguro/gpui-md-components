//! 進み具合のアクセシビリティ情報。

use gpui::{Div, ElementId, Role, SharedString, Stateful, div, prelude::*};

use crate::{A11yExt, A11yNode, RangeValue};

/// 進み具合。値が `None` のときは不定。
pub struct Progress {
    id: ElementId,
    value: Option<f32>,
    label: SharedString,
}

impl Progress {
    /// ID、値、ラベルから作る。
    pub fn new(
        id: impl Into<ElementId>,
        value: Option<f32>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
        }
    }

    /// アクセシビリティ情報。値は 0〜1 に収める。
    pub fn a11y_node(&self) -> A11yNode {
        let node = A11yNode::new(Role::ProgressIndicator).label(self.label.clone());
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
