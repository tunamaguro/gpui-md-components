//! 新しい説明と状態のアクセシビリティ契約。

use gpui::{AppContext, Role, TestAppContext};
use gpui_headless::{A11yNode, Button, Select, TextField, TextFieldState};

#[test]
fn node_description_preserves_nonempty_text() {
    let node = A11yNode::new(Role::TextInput).description("入力方法");
    assert_eq!(node.description.as_deref(), Some("入力方法"));
}

#[test]
fn node_empty_description_is_absent() {
    let node = A11yNode::new(Role::TextInput).description("入力方法");
    assert_eq!(node.description.as_deref(), Some("入力方法"));
    assert_eq!(node.description("").description, None);
}

#[test]
fn node_exposes_required_expanded_and_popup_states() {
    let node = A11yNode::new(Role::Button)
        .required(true)
        .expanded(false)
        .has_popup_menu(true);
    assert!(node.required);
    assert_eq!(node.expanded, Some(false));
    assert!(node.has_popup_menu);
}

#[test]
fn button_menu_states_distinguish_unset_closed_and_open() {
    let plain = Button::new("plain", "操作").a11y_node();
    assert_eq!(plain.expanded, None);
    assert!(!plain.has_popup_menu);
    for open in [false, true] {
        let node = Button::new("menu", "操作").menu_expanded(open).a11y_node();
        assert_eq!(node.expanded, Some(open));
        assert!(node.has_popup_menu);
    }
}

#[test]
fn select_exposes_required_and_popup() {
    let plain = Select::new("select").a11y_node();
    assert!(!plain.required);
    let node = Select::new("select").required(true).a11y_node();
    assert_eq!(node.role, Role::ComboBox);
    assert!(node.required);
    assert!(node.has_popup_menu);
}

#[gpui::test]
fn text_field_exposes_required_state(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    assert!(!TextField::new("name", &state).a11y_node().required);
    let node = TextField::new("name", &state).required(true).a11y_node();
    assert_eq!(node.role, Role::TextInput);
    assert!(node.required);
}

#[test]
fn node_expanded_preserves_false_and_true() {
    for expanded in [false, true] {
        assert_eq!(
            A11yNode::new(Role::Button).expanded(expanded).expanded,
            Some(expanded)
        );
    }
}

#[test]
fn node_exposes_popup_menu_state() {
    assert!(
        A11yNode::new(Role::Button)
            .has_popup_menu(true)
            .has_popup_menu
    );
}

#[test]
fn select_exposes_popup_without_required() {
    assert!(Select::new("select").a11y_node().has_popup_menu);
}
