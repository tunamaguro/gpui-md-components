//! ダイアログの役割。

use gpui::Role;
use gpui_headless::Dialog;

#[test]
fn dialog_is_named_by_its_label() {
    let node = Dialog::new("d", "削除しますか？").a11y_node();
    assert_eq!(node.role, Role::Dialog);
    assert_eq!(node.label.as_deref(), Some("削除しますか？"));
    assert_eq!(
        Dialog::new("d", "x").alert(true).a11y_node().role,
        Role::AlertDialog
    );
}

#[test]
fn dialog_is_modal_unless_turned_off() {
    assert!(Dialog::new("d", "x").a11y_node().modal);
    assert!(!Dialog::new("d", "x").modal(false).a11y_node().modal);
}

#[test]
fn complementary_panels_are_not_modal() {
    let node = Dialog::new("s", "フィルタ").complementary().a11y_node();
    assert_eq!(node.role, Role::Complementary);
    assert!(!node.modal);
}
