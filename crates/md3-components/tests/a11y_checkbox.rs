//! Checkbox / Switch のキー操作と a11y 情報。

mod support;

use std::{cell::Cell, rc::Rc};

use gpui::{TestAppContext, prelude::*};
use md3_components::selection::CheckboxState;
use md3_components::selection::{Checkbox, Switch};

#[cfg(debug_assertions)]
#[gpui::test]
fn switch_cancels_keyboard_press_when_focus_moves_away(cx: &mut TestAppContext) {
    use gpui::{InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, div, px};
    use md3_components::theme::md3_tokens::component::switch as tokens;

    cx.update(|cx| cx.set_reduce_motion(true));
    let changed = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let changed = changed.clone();
        move |_, _| {
            let changed = changed.clone();
            div()
                .child(
                    Switch::new("wifi", false)
                        .aria_label("Wi-Fi")
                        .on_change(move |_, _, _| changed.set(true)),
                )
                .child(Checkbox::new("next", CheckboxState::Unchecked).aria_label("次"))
                .into_any_element()
        }
    });
    // GPUI のフォーカス通知はアクティブなウィンドウで発火する。
    cx.update(|window, _| window.activate_window());
    support::draw(cx);
    for key in ["space", "enter"] {
        support::tab(cx);
        let keystroke = Keystroke::parse(key).expect("有効なキー");
        cx.update(|window, cx| {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
        });
        support::draw(cx);
        assert_eq!(
            cx.debug_bounds("md3-switch-handle-wifi")
                .expect("つまみ")
                .size
                .width,
            px(tokens::PRESSED_HANDLE_WIDTH),
            "キーを押している間は押下寸法"
        );
        support::tab(cx);
        assert_eq!(
            cx.debug_bounds("md3-switch-handle-wifi")
                .expect("つまみ")
                .size
                .width,
            px(tokens::UNSELECTED_HANDLE_WIDTH),
            "キー解放を受け取れなくてもフォーカス喪失時に戻る"
        );
        cx.update(|window, cx| {
            window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        });
        support::draw(cx);
        assert!(!changed.get(), "キャンセルした押下で選択は変わらない");
    }
}
