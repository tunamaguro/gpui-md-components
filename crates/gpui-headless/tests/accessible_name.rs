//! 読み上げる名前の解決。
use gpui::Role;
use gpui_headless::{A11yNode, AccessibleName};

#[test]
fn visible_and_override_names_follow_the_rules() {
    let mut name = AccessibleName::default();
    assert_eq!(name.resolve(), None);
    name.set_visible("表示");
    assert_eq!(name.resolve().as_deref(), Some("表示"));
    name.set_aria_label("表示の説明");
    assert_eq!(name.resolve().as_deref(), Some("表示の説明"));
    name.set_visible("新しい表示");
    assert_eq!(name.resolve().as_deref(), Some("表示の説明"));
    name.set_aria_label("");
    assert_eq!(name.resolve(), None);
    name.set_visible("さらに変更");
    assert_eq!(name.resolve(), None);
    assert_eq!(A11yNode::new(Role::Button).with_name(&name).label, None);
}

#[test]
fn empty_visible_name_is_absent() {
    let mut name = AccessibleName::new("");
    assert_eq!(name.resolve(), None);
    name.set_visible("表示");
    assert_eq!(name.resolve().as_deref(), Some("表示"));
    name.set_visible("");
    assert_eq!(name.resolve(), None);
}
