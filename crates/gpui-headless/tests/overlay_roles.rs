//! Tooltip と Snackbar の役割。

use gpui::Role;
use gpui_headless::{Snackbar, Tooltip};

#[test]
fn tooltip_is_named_by_its_text() {
    let node = Tooltip::new("t", "保存").a11y_node();
    assert_eq!(node.role, Role::Tooltip);
    assert_eq!(node.label.as_deref(), Some("保存"));
}

#[test]
fn snackbar_is_a_status() {
    let node = Snackbar::new("s", "保存しました").a11y_node();
    assert_eq!(node.role, Role::Status);
    assert_eq!(node.label.as_deref(), Some("保存しました"));
    assert_eq!(node.value.as_deref(), Some("保存しました"));
    assert!(node.live, "表示されたら読み上げる");
    assert!(!Snackbar::new("s", "x").active(false).a11y_node().live);
}

#[test]
fn tooltip_goes_above_the_anchor_and_stays_in_the_window() {
    use gpui::{Bounds, point, px, size};
    use gpui_headless::tooltip_origin;
    let viewport = size(px(800.0), px(600.0));
    let tooltip = size(px(100.0), px(24.0));
    let at = |x: f32, y: f32| Bounds::new(point(px(x), px(y)), size(px(40.0), px(40.0)));

    let origin = tooltip_origin(at(200.0, 200.0), tooltip, viewport, px(4.0));
    assert_eq!(origin, point(px(170.0), px(172.0)), "上に中央揃え");

    let origin = tooltip_origin(at(200.0, 10.0), tooltip, viewport, px(4.0));
    assert_eq!(origin.y, px(54.0), "上に入らなければ下");

    assert_eq!(
        tooltip_origin(at(0.0, 200.0), tooltip, viewport, px(4.0)).x,
        px(0.0)
    );
    assert_eq!(
        tooltip_origin(at(780.0, 200.0), tooltip, viewport, px(4.0)).x,
        px(700.0)
    );
}
