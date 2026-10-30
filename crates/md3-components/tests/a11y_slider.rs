//! Slider / Chip / Progress のキー操作と a11y 情報。

use gpui::Role;
use md3_components::selection::{CircularProgress, LinearProgress};

#[test]
fn value_components_expose_ranges() {
    let linear = LinearProgress::new("l", Some(0.5), "ダウンロード").a11y_node();
    assert_eq!(linear.role, Role::ProgressIndicator);
    assert_eq!(linear.range.map(|r| r.value), Some(0.5));
    assert_eq!(
        CircularProgress::new("c", None, "読み込み中")
            .a11y_node()
            .range,
        None
    );
}
