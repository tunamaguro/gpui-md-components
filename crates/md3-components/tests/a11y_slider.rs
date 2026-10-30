//! Slider / Chip / Progress のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, prelude::*};
use md3_components::selection::{CircularProgress, LinearProgress, Slider};

#[gpui::test]
fn slider_keys_follow_apg(cx: &mut TestAppContext) {
    let value = Rc::new(Cell::new(5.0_f32));
    let cx = support::open(cx, {
        let value = value.clone();
        move |_, _| {
            let set = value.clone();
            Slider::new("volume", value.get(), "音量")
                .range(0.0, 10.0)
                .step(1.0)
                .on_change(move |next, _, _| set.set(next))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "right");
    assert_eq!(value.get(), 6.0);
    support::press(cx, "down");
    assert_eq!(value.get(), 5.0);
    support::press(cx, "end");
    assert_eq!(value.get(), 10.0);
    support::press(cx, "pagedown");
    assert_eq!(value.get(), 0.0);
    support::press(cx, "home");
    assert_eq!(value.get(), 0.0);
}

#[test]
fn value_components_expose_ranges() {
    let slider = Slider::new("s", 3.0, "明るさ")
        .range(0.0, 10.0)
        .step(1.0)
        .a11y_node();
    assert_eq!(slider.role, Role::Slider);
    let range = slider.range.expect("range");
    assert_eq!(
        (range.value, range.min, range.max, range.step),
        (3.0, 0.0, 10.0, Some(1.0))
    );

    let linear = LinearProgress::new("l", Some(0.5), "ダウンロード").a11y_node();
    assert_eq!(linear.role, Role::ProgressIndicator);
    assert_eq!(linear.range.map(|r| r.value), Some(0.5));
    assert_eq!(
        CircularProgress::new("c", None, "読み込み中")
            .a11y_node()
            .range,
        None
    );
}
