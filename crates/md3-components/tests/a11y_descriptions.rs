//! 補足テキストと追加の状態のアクセシビリティ契約。

use gpui::{AppContext, Role, TestAppContext, accesskit::HasPopup};
use gpui_headless::TextFieldState;
use md3_components::{
    IconName, Text,
    buttons::{Button, IconButton},
    containment::ListItem,
    inputs::{Select, TextField},
    overlays::{Dialog, MenuItem},
    theme::TypeScale,
};

#[gpui::test]
fn text_field_exposes_supporting_description_and_required(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    let node = TextField::new("name", &state)
        .label("名前")
        .supporting_text("姓と名")
        .required(true)
        .a11y_node();
    assert_eq!(node.role, Role::TextInput);
    assert_eq!(node.description.as_deref(), Some("姓と名"));
    assert_eq!(node.label.as_deref(), Some("名前"));
    assert!(node.required);
}

#[test]
fn select_exposes_supporting_description_required_and_popup() {
    let node = Select::new("choice", ["一"], None)
        .label("選択")
        .supporting_text("一つ選択")
        .required(true)
        .a11y_node();
    assert_eq!(node.role, Role::ComboBox);
    assert_eq!(node.description.as_deref(), Some("一つ選択"));
    assert!(node.required);
    assert_eq!(node.has_popup, Some(HasPopup::Listbox));
}

#[test]
fn dialog_exposes_supporting_description() {
    let node = Dialog::new("confirm")
        .headline("確認")
        .supporting_text("取り消せません")
        .a11y_node();
    assert_eq!(node.description.as_deref(), Some("取り消せません"));
}

#[test]
fn list_item_description_joins_overline_and_supporting_text() {
    let node = ListItem::new("見出し")
        .overline("上書き")
        .supporting_text("補足")
        .a11y_node();
    assert_eq!(node.description.as_deref(), Some("上書き、補足"));
    assert_eq!(
        ListItem::new("見出し")
            .overline("上書き")
            .a11y_node()
            .description
            .as_deref(),
        Some("上書き")
    );
    assert_eq!(
        ListItem::new("見出し")
            .supporting_text("補足")
            .a11y_node()
            .description
            .as_deref(),
        Some("補足")
    );
    assert_eq!(ListItem::new("見出し").a11y_node().description, None);
}

#[test]
fn menu_item_exposes_trailing_text_as_description() {
    let node = MenuItem::new("コピー")
        .trailing_text("利用可能")
        .a11y_node();
    assert_eq!(node.description.as_deref(), Some("利用可能"));
}

#[gpui::test]
fn text_field_supporting_status_is_live_only_on_error(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    assert!(
        TextField::new("name", &state)
            .error(true)
            .supporting_a11y_node()
            .is_none()
    );
    for error in [false, true] {
        let (_, node) = TextField::new("name", &state)
            .supporting_text("入力してください")
            .error(error)
            .supporting_a11y_node()
            .expect("補助テキストのノード");
        assert_eq!(node.role, Role::Status);
        assert_eq!(node.value, node.label);
        assert_eq!(node.label.as_deref(), Some("入力してください"));
        assert_eq!(node.live, error);
    }
}

#[test]
fn select_supporting_status_is_live_only_on_error() {
    assert!(
        Select::new("choice", ["一"], None)
            .error(true)
            .supporting_a11y_node()
            .is_none()
    );
    for error in [false, true] {
        let (_, node) = Select::new("choice", ["一"], None)
            .supporting_text("選択してください")
            .error(error)
            .supporting_a11y_node()
            .expect("補助テキストのノード");
        assert_eq!(node.role, Role::Status);
        assert_eq!(node.value, node.label);
        assert_eq!(node.label.as_deref(), Some("選択してください"));
        assert_eq!(node.live, error);
    }
}

#[test]
fn supporting_status_ids_are_derived_from_the_field() {
    let (first, _) = Select::new("first", ["一"], None)
        .supporting_text("補足")
        .supporting_a11y_node()
        .expect("補助テキストのノード");
    let (second, _) = Select::new("second", ["一"], None)
        .supporting_text("補足")
        .supporting_a11y_node()
        .expect("補助テキストのノード");
    assert_ne!(first, second);
    assert!(matches!(first, gpui::ElementId::NamedChild(parent, _) if *parent == "first".into()));
    assert!(matches!(second, gpui::ElementId::NamedChild(parent, _) if *parent == "second".into()));
}

#[test]
fn md3_button_menu_states_distinguish_unset_closed_and_open() {
    assert_eq!(Button::new("plain", "操作").a11y_node().expanded, None);
    for open in [false, true] {
        let node = Button::new("menu", "操作").menu_expanded(open).a11y_node();
        assert_eq!(node.expanded, Some(open));
        assert_eq!(node.has_popup, Some(HasPopup::Menu));
    }
}

#[test]
fn md3_icon_button_menu_states_distinguish_unset_closed_and_open() {
    assert_eq!(
        IconButton::new("plain", IconName::Menu)
            .a11y_node()
            .expanded,
        None
    );
    for open in [false, true] {
        let node = IconButton::new("menu", IconName::Menu)
            .menu_expanded(open)
            .a11y_node();
        assert_eq!(node.expanded, Some(open));
        assert_eq!(node.has_popup, Some(HasPopup::Menu));
    }
}

#[test]
fn text_only_exposes_a_label_node_with_an_explicit_id() {
    assert!(
        Text::new(TypeScale::BodyLarge, "本文")
            .a11y_node()
            .is_none()
    );
    let node = Text::new(TypeScale::BodyLarge, "本文")
        .id("body")
        .a11y_node()
        .expect("ID を指定した文字のノード");
    assert_eq!(node.role, Role::Label);
    assert_eq!(node.label.as_deref(), Some("本文"));
    assert_eq!(node.value.as_deref(), Some("本文"));
}
