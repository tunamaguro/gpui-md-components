//! Slider / Chip / Progress のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{Role, TestAppContext, Toggled, prelude::*};
use md3_components::selection::{Chip, ChipKind, CircularProgress, LinearProgress, Slider};

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

#[gpui::test]
fn filter_chip_toggles_with_space(cx: &mut TestAppContext) {
    let on = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let on = on.clone();
        move |_, _| {
            let set = on.clone();
            Chip::new("unread", ChipKind::Filter, "未読")
                .selected(on.get())
                .on_click(move |_, _, _| set.set(!set.get()))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "space");
    assert!(on.get());
}

#[gpui::test]
fn input_chip_is_removed_with_backspace(cx: &mut TestAppContext) {
    let removed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let removed = removed.clone();
        move |_, _| {
            let set = removed.clone();
            Chip::new("tag", ChipKind::Input, "rust")
                .on_remove(move |_, _| set.set(true))
                .into_any_element()
        }
    });
    support::tab(cx);
    support::press(cx, "backspace");
    assert!(removed.get());
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

    let chip = Chip::new("f", ChipKind::Filter, "未読").a11y_node();
    assert_eq!(chip.toggled, Some(Toggled::False));
}
