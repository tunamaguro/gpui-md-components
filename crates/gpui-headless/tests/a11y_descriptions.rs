//! 新しい説明と状態のアクセシビリティ契約。

use gpui::{AppContext, Role, TestAppContext, accesskit::HasPopup};
use gpui_headless::{
    A11yNode, Button, Dialog, ListItem, Menu, MenuItem, MenuKind, Select, TextField, TextFieldState,
};

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
        .has_popup(HasPopup::Menu);
    assert!(node.required);
    assert_eq!(node.expanded, Some(false));
    assert_eq!(node.has_popup, Some(HasPopup::Menu));
}

#[test]
fn button_menu_states_distinguish_unset_closed_and_open() {
    let plain = Button::new("plain", "操作").a11y_node();
    assert_eq!(plain.expanded, None);
    assert_eq!(plain.has_popup, None);
    for open in [false, true] {
        let node = Button::new("menu", "操作").menu_expanded(open).a11y_node();
        assert_eq!(node.expanded, Some(open));
        assert_eq!(node.has_popup, Some(HasPopup::Menu));
    }
}

#[test]
fn select_exposes_required_and_popup() {
    let plain = Select::new("select").a11y_node();
    assert!(!plain.required);
    let node = Select::new("select").required(true).a11y_node();
    assert_eq!(node.role, Role::ComboBox);
    assert!(node.required);
    assert_eq!(node.has_popup, Some(HasPopup::Listbox));
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
    assert_eq!(
        A11yNode::new(Role::Button)
            .has_popup(HasPopup::Menu)
            .has_popup,
        Some(HasPopup::Menu)
    );
}

#[test]
fn select_exposes_popup_without_required() {
    assert_eq!(
        Select::new("select").a11y_node().has_popup,
        Some(HasPopup::Listbox)
    );
}

#[test]
fn descriptions_flow_through_headless_nodes() {
    let nodes = [
        Button::new("button", "操作")
            .description("説明")
            .a11y_node(),
        Dialog::new("dialog").description("説明").a11y_node(),
        Menu::new("menu")
            .item(MenuItem::new("項目").description("説明"))
            .item_a11y_node(0)
            .unwrap(),
        ListItem::new("項目").description("説明").a11y_node(2, 5),
    ];
    for node in nodes {
        assert_eq!(node.description.as_deref(), Some("説明"));
    }
    let option = Menu::new("options")
        .kind(MenuKind::Options)
        .item(MenuItem::new("項目").description("説明"))
        .item_a11y_node(0)
        .unwrap();
    assert_eq!(option.role, Role::ListBoxOption);
    assert_eq!(option.description.as_deref(), Some("説明"));
    assert_eq!(option.position, Some((1, 1)));
    assert_eq!(
        Button::new("button", "操作")
            .description("")
            .a11y_node()
            .description,
        None
    );
}

#[test]
fn supporting_status_uses_parent_id_and_live_state() {
    let parent = gpui::ElementId::from("field");
    for live in [false, true] {
        let (id, node) = gpui_headless::supporting_status(&parent, "補足", live);
        assert!(
            matches!(id, gpui::ElementId::NamedChild(ref p, ref name) if **p == parent && name.as_ref() == "supporting-text")
        );
        assert_eq!(node.role, Role::Status);
        assert_eq!(node.label.as_deref(), Some("補足"));
        assert_eq!(node.value.as_deref(), Some("補足"));
        assert_eq!(node.live, live);
    }
}

#[gpui::test]
fn text_field_default_label_and_override_are_order_independent(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    for name in ["氏名", ""] {
        let first = TextField::new("field", &state)
            .label("名前")
            .aria_label(name)
            .a11y_node();
        let second = TextField::new("field", &state)
            .aria_label(name)
            .label("名前")
            .a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.label.as_deref(), (!name.is_empty()).then_some(name));
    }
}

#[test]
fn select_default_label_and_override_are_order_independent() {
    for name in ["色を選択", ""] {
        let first = Select::new("field")
            .label("色")
            .aria_label(name)
            .a11y_node();
        let second = Select::new("field")
            .aria_label(name)
            .label("色")
            .a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.label.as_deref(), (!name.is_empty()).then_some(name));
    }
}
