//! 進み具合のアクセシビリティ情報。

use gpui::{Div, ElementId, Role, Stateful, div, prelude::*};

use crate::{A11yExt, A11yNode, RangeValue};

/// 進み具合。値が `None` のときは不定。
pub struct Progress {
    id: ElementId,
    value: Option<f32>,
    name: crate::AccessibleName,
}

impl Progress {
    /// ID と値から作る。
    pub fn new(id: impl Into<ElementId>, value: Option<f32>) -> Self {
        Self {
            id: id.into(),
            value,
            name: crate::AccessibleName::default(),
        }
    }

    accessible_name!();

    /// アクセシビリティ情報。値は 0〜1 に収める。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(Role::ProgressIndicator);
        node = node.with_name(&self.name);
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
