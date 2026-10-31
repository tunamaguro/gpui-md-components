//! Input チップの削除ボタンのアクセシビリティ。

mod support;

use gpui::{Role, TestAppContext, prelude::*, px};
use md3_components::selection::{Chip, ChipKind};

#[test]
fn remove_label_reaches_the_node() {
    let node = Chip::new("tag", ChipKind::Input, "rust")
        .remove_label("rust を削除")
        .remove_a11y_node();
    assert_eq!(node.role, Role::Button);
    assert_eq!(node.label.as_deref(), Some("rust を削除"));
}

/// 位置は `debug_selector` で測るため、デバッグビルドでだけ走る（gpui はリリースでは記録しない）。
#[cfg(debug_assertions)]
#[gpui::test]
fn remove_target_meets_wcag_2_5_8(cx: &mut TestAppContext) {
    let cx = support::open(cx, |_, _| {
        Chip::new("tag", ChipKind::Input, "rust")
            .on_remove(|_, _| {})
            .into_any_element()
    });
    let target = cx
        .debug_bounds("md3-chip-remove-tag")
        .expect("削除ボタンの位置");
    assert!(target.size.width >= px(24.0));
    assert!(target.size.height >= px(24.0));
    let chip = cx.debug_bounds("md3-chip-tag").expect("チップの位置");
    assert_eq!(chip.size.height, px(32.0), "チップの大きさは変えない");
}
