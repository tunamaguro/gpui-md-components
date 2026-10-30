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
    assert!(node.live, "表示されたら読み上げる");
    assert!(!Snackbar::new("s", "x").active(false).a11y_node().live);
}
