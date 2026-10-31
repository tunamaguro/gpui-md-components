//! Slider / Chip / Progress のキー操作と a11y 情報。

use gpui::Role;
use gpui_headless::Progress;

#[test]
fn value_components_expose_ranges() {
    let linear = Progress::new("l", Some(0.5))
        .aria_label("ダウンロード")
        .a11y_node();
    assert_eq!(linear.role, Role::ProgressIndicator);
    assert_eq!(linear.range.map(|r| r.value), Some(0.5));
    assert_eq!(
        Progress::new("c", None)
            .aria_label("読み込み中")
            .a11y_node()
            .range,
        None
    );
}

#[test]
fn progress_clamps_and_keeps_label() {
    let node = Progress::new("p", Some(2.0))
        .aria_label("読み込み中")
        .a11y_node();
    assert_eq!(node.label.as_deref(), Some("読み込み中"));
    assert_eq!(node.range.map(|r| r.value), Some(1.0));
    assert_eq!(
        Progress::new("p", Some(-1.0))
            .aria_label("p")
            .a11y_node()
            .range
            .map(|r| r.value),
        Some(0.0)
    );
    assert_eq!(
        Progress::new("p", Some(0.25))
            .aria_label("p")
            .a11y_node()
            .range
            .map(|r| r.value),
        Some(0.25)
    );
}

#[test]
fn progress_name_is_optional() {
    assert_eq!(Progress::new("p", None).a11y_node().label, None);
    assert_eq!(
        Progress::new("p", None).aria_label("").a11y_node().label,
        None
    );
    assert_eq!(
        Progress::new("p", None)
            .aria_label("進捗")
            .a11y_node()
            .label
            .as_deref(),
        Some("進捗")
    );
}
