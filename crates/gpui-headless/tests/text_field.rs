//! TextField の入力（キー入力、IME の変換中の文字、編集のキー操作）。

mod support;

use gpui::{Entity, EntityInputHandler, TestAppContext, VisualTestContext, prelude::*};
use gpui::{black, px};
use gpui_headless::{SearchField, TextField, TextFieldState, TextPaint};

fn open(cx: &mut TestAppContext) -> (Entity<TextFieldState>, &mut VisualTestContext) {
    let state = cx.new(TextFieldState::new);
    let cx = support::open(cx, {
        let state = state.clone();
        move |window, cx| {
            let (base, control) = TextField::new("name", &state, "名前").build(window, cx);
            let color = black();
            base.child(control.input(
                TextPaint {
                    text: color,
                    placeholder: color,
                    caret: color,
                    selection: color,
                    caret_width: px(2.0),
                },
                None,
            ))
            .into_any_element()
        }
    });
    support::tab(cx);
    (state, cx)
}

#[gpui::test]
fn typing_inserts_text_at_the_caret(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.simulate_input("abc");
    support::draw(cx);
    assert_eq!(state.read_with(cx, |s, _| s.text().to_owned()), "abc");

    support::press(cx, "left");
    support::press(cx, "backspace");
    assert_eq!(state.read_with(cx, |s, _| s.text().to_owned()), "ac");

    support::press(cx, "home");
    support::press(cx, "shift-end");
    assert_eq!(state.read_with(cx, |s, _| s.selected_range()), 0..2);
    support::press(cx, "delete");
    assert_eq!(state.read_with(cx, |s, _| s.text().to_owned()), "");
}

#[gpui::test]
fn ime_composition_is_marked_then_committed(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.simulate_input("a");
    support::draw(cx);

    // IME が「にほん」を変換中として渡し、その後「日本」で確定する（プラットフォームの入力ハンドラが呼ぶ順）。
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_and_mark_text_in_range(None, "に", None, window, cx);
            state.replace_and_mark_text_in_range(None, "にほん", None, window, cx);
        })
    });
    support::draw(cx);
    state.read_with(cx, |state, _| {
        assert_eq!(state.text(), "aにほん");
        assert_eq!(state.marked_range(), Some(1..1 + "にほん".len()));
    });
    let marked_utf16 =
        cx.update(|window, cx| state.update(cx, |state, cx| state.marked_text_range(window, cx)));
    assert_eq!(marked_utf16, Some(1..4), "UTF-16 で数えた範囲を IME に返す");

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_text_in_range(None, "日本", window, cx)
        })
    });
    support::draw(cx);
    state.read_with(cx, |state, _| {
        assert_eq!(state.text(), "a日本");
        assert_eq!(state.marked_range(), None);
        let end = "a日本".len();
        assert_eq!(
            state.selected_range(),
            end..end,
            "カーソルは確定した文字の後ろ"
        );
    });
}

#[gpui::test]
fn emoji_and_combining_marks_are_deleted_whole(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_text_in_range(None, "か\u{3099}👍🏽", window, cx)
        })
    });
    support::press(cx, "backspace");
    assert_eq!(
        state.read_with(cx, |s, _| s.text().to_owned()),
        "か\u{3099}"
    );
    support::press(cx, "backspace");
    assert_eq!(state.read_with(cx, |s, _| s.text().to_owned()), "");
}

#[gpui::test]
fn fields_expose_names_roles_and_states(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    let field = TextField::new("name", &state, "名前")
        .disabled(true)
        .error(true)
        .placeholder("入力")
        .a11y_node();
    assert_eq!(field.role, gpui::Role::TextInput);
    assert_eq!(field.label.as_deref(), Some("名前"));
    assert!(field.disabled);
    assert!(field.invalid);
    let search = SearchField::new("search", &state, "メールを検索")
        .placeholder("メールを検索")
        .a11y_node();
    assert_eq!(search.role, gpui::Role::SearchInput);
    assert_eq!(search.label.as_deref(), Some("メールを検索"));
}
