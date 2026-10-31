//! 補足テキストと追加の状態のアクセシビリティ契約。

use crate::{
    IconName, Text,
    buttons::{Button, IconButton},
    containment::ListItem,
    inputs::{Select, TextField, TextFieldState},
    overlays::{Dialog, Menu, MenuItem},
    theme::TypeScale,
};
use gpui::{Role, TestAppContext, accesskit::HasPopup};

#[gpui::test]
fn text_field_exposes_supporting_description_and_required(cx: &mut TestAppContext) {
    let state = cx.update(TextFieldState::new);
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
        .a11y_node(2, 5);
    assert_eq!(node.description.as_deref(), Some("上書き、補足"));
    assert_eq!(node.position, Some((3, 5)));
    assert_eq!(
        ListItem::new("見出し")
            .overline("上書き")
            .supporting_text("")
            .a11y_node(0, 1)
            .description
            .as_deref(),
        Some("上書き")
    );
    assert_eq!(
        ListItem::new("見出し")
            .overline("")
            .supporting_text("補足")
            .a11y_node(0, 1)
            .description
            .as_deref(),
        Some("補足")
    );
    assert_eq!(ListItem::new("見出し").a11y_node(0, 1).description, None);
}

#[test]
fn menu_item_exposes_trailing_text_as_description() {
    let menu = Menu::new("menu").item(MenuItem::new("コピー").trailing_text("利用可能"));
    assert_eq!(menu.a11y_node().role, Role::Menu);
    let node = menu.item_a11y_node(0).unwrap();
    assert_eq!(node.description.as_deref(), Some("利用可能"));
    assert_eq!(node.role, Role::MenuItem);
    assert_eq!(node.position, Some((1, 1)));
    assert!(menu.item_a11y_node(1).is_none());
}

#[cfg(feature = "overlays")]
#[test]
fn icon_button_exposes_tooltip_description() {
    let node = IconButton::new("button", IconName::Menu)
        .tooltip("メニューを開く")
        .a11y_node();
    assert_eq!(node.description.as_deref(), Some("メニューを開く"));
}

#[test]
fn direct_aria_descriptions_are_limited_to_focus_tooltip() {
    // 任意の要素に付けるツールチップ以外は headless のノードを通す。
    fn visit(dir: &std::path::Path, root: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, root, found);
            } else if path.extension().is_some_and(|ext| ext == "rs")
                && !path.ends_with("a11y_tests.rs")
                && std::fs::read_to_string(&path)
                    .unwrap()
                    .contains(".aria_description(")
            {
                found.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    visit(&root, &root, &mut found);
    assert_eq!(found, [std::path::PathBuf::from("overlays/tooltip.rs")]);
}

#[gpui::test]
fn text_field_supporting_status_is_live_only_on_error(cx: &mut TestAppContext) {
    let state = cx.update(TextFieldState::new);
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

#[test]
fn button_configuration_order_preserves_accessibility() {
    let first = Button::new("b", "保存")
        .disabled(true)
        .aria_label("保存する")
        .menu_expanded(true)
        .on_click(|_, _, _| {})
        .a11y_node();
    let second = Button::new("b", "保存")
        .on_click(|_, _, _| {})
        .menu_expanded(true)
        .aria_label("保存する")
        .disabled(true)
        .a11y_node();
    assert_eq!(first, second);
    assert!(first.disabled);
    assert_eq!(first.expanded, Some(true));
}

#[test]
fn icon_button_configuration_order_preserves_accessibility() {
    let first = IconButton::new("i", IconName::Menu)
        .disabled(true)
        .aria_label("メニュー")
        .selected(true)
        .menu_expanded(false)
        .on_click(|_, _, _| {})
        .a11y_node();
    let second = IconButton::new("i", IconName::Menu)
        .on_click(|_, _, _| {})
        .menu_expanded(false)
        .selected(true)
        .aria_label("メニュー")
        .disabled(true)
        .a11y_node();
    assert_eq!(first, second);
    assert!(first.disabled);
    assert_eq!(first.label.as_deref(), Some("メニュー"));
    assert_eq!(
        IconButton::new("i", IconName::Menu)
            .aria_label("")
            .a11y_node()
            .label,
        None
    );
}

#[gpui::test]
fn text_field_name_is_independent_of_setter_order(cx: &mut TestAppContext) {
    let state = cx.update(TextFieldState::new);
    for name in ["氏名", ""] {
        let first = TextField::new("name", &state)
            .label("名前")
            .aria_label(name)
            .supporting_text("説明")
            .error(true)
            .required(true)
            .disabled(true)
            .a11y_node();
        let second = TextField::new("name", &state)
            .disabled(true)
            .required(true)
            .error(true)
            .supporting_text("説明")
            .aria_label(name)
            .label("名前")
            .a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.label.as_deref(), (!name.is_empty()).then_some(name));
    }
}

#[test]
fn select_name_is_independent_of_setter_order() {
    for name in ["色を選択", ""] {
        let first = Select::new("choice", ["一"], None)
            .label("色")
            .aria_label(name)
            .supporting_text("説明")
            .error(true)
            .required(true)
            .disabled(true)
            .a11y_node();
        let second = Select::new("choice", ["一"], None)
            .disabled(true)
            .required(true)
            .error(true)
            .supporting_text("説明")
            .aria_label(name)
            .label("色")
            .a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.label.as_deref(), (!name.is_empty()).then_some(name));
    }
}

#[test]
fn dialog_name_is_independent_of_setter_order() {
    for name in ["削除の確認", ""] {
        let first = Dialog::new("dialog")
            .headline("確認")
            .aria_label(name)
            .supporting_text("説明")
            .alert(true)
            .a11y_node();
        let second = Dialog::new("dialog")
            .alert(true)
            .supporting_text("説明")
            .aria_label(name)
            .headline("確認")
            .a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.label.as_deref(), (!name.is_empty()).then_some(name));
    }
}

#[gpui::test]
fn supporting_status_tracks_error_regardless_of_setter_order(cx: &mut TestAppContext) {
    let state = cx.update(TextFieldState::new);
    for error in [false, true] {
        let first = TextField::new("field", &state)
            .error(error)
            .supporting_text("補足")
            .supporting_a11y_node();
        let second = TextField::new("field", &state)
            .supporting_text("補足")
            .error(error)
            .supporting_a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.unwrap().1.live, error);
        let first = Select::new("select", ["一"], None)
            .error(error)
            .supporting_text("補足")
            .supporting_a11y_node();
        let second = Select::new("select", ["一"], None)
            .supporting_text("補足")
            .error(error)
            .supporting_a11y_node();
        assert_eq!(first, second);
        assert_eq!(first.unwrap().1.live, error);
    }
}

#[test]
fn remove_label_reaches_the_node() {
    let node = crate::selection::Chip::new("tag", crate::selection::ChipKind::Input, "rust")
        .remove_label("rust を削除")
        .remove_a11y_node();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label.as_deref(), Some("rust を削除"));
}
