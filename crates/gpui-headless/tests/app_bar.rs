//! アプリバーのアクセシビリティ情報。
use gpui::Role;
use gpui_headless::AppBar;

#[test]
fn app_bar_and_heading_expose_title() {
    let bar = AppBar::new("bar", "タイトル");
    let node = bar.a11y_node();
    assert_eq!(node.role, Role::Header);
    assert_eq!(node.label.as_deref(), Some("タイトル"));
    let heading = bar.heading_a11y_node();
    assert_eq!(heading.role, Role::Heading);
    assert_eq!(heading.label.as_deref(), Some("タイトル"));
}
