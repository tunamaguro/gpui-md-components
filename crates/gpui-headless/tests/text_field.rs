//! TextField の入力（キー入力、IME の変換中の文字、編集のキー操作）。

mod support;

use gpui::{Bounds, black, point, px};
use gpui::{Entity, EntityInputHandler, Focusable, TestAppContext, VisualTestContext, prelude::*};
use gpui_headless::{SearchField, TextField, TextFieldState, TextPaint};

fn open(cx: &mut TestAppContext) -> (Entity<TextFieldState>, &mut VisualTestContext) {
    open_with_placeholder(cx, None)
}

fn open_with_placeholder<'a>(
    cx: &'a mut TestAppContext,
    placeholder: Option<&'static str>,
) -> (Entity<TextFieldState>, &'a mut VisualTestContext) {
    let state = cx.new(TextFieldState::new);
    let cx = support::open(cx, {
        let state = state.clone();
        move |window, cx| {
            let (base, control) = TextField::new("name", &state)
                .label("名前")
                .build(window, cx);
            let color = black();
            base.child(control.input(
                TextPaint {
                    text: color,
                    placeholder: color,
                    caret: color,
                    selection: color,
                    caret_width: px(2.0),
                },
                placeholder.map(Into::into),
            ))
            .into_any_element()
        }
    });
    support::tab(cx);
    (state, cx)
}

#[gpui::test]
fn empty_field_with_placeholder_reports_ime_bounds(cx: &mut TestAppContext) {
    let (state, cx) = open_with_placeholder(cx, Some("名前を入力"));
    let bounds = Bounds::from_corners(point(px(42.0), px(20.0)), point(px(200.0), px(44.0)));
    let actual = cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.bounds_for_range(0..0, bounds, window, cx)
        })
    });
    assert_eq!(
        actual,
        Some(Bounds::from_corners(
            point(bounds.left(), bounds.top()),
            point(bounds.left(), bounds.bottom())
        ))
    );
}

#[gpui::test]
fn composition_schedules_one_ime_position_update(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.set_text("a", cx));
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 1);
        assert_eq!(window.simulate_next_frame(cx), 0);
        state.update(cx, |state, cx| {
            state.replace_and_mark_text_in_range(Some(0..1), "にほん", None, window, cx)
        });
        assert_eq!(window.simulate_next_frame(cx), 0, "描画前には予約しない");
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            window.simulate_next_frame(cx),
            1,
            "描画後に一度だけ予約する"
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            window.simulate_next_frame(cx),
            0,
            "変化のない描画では予約しない"
        );
    });
}

#[gpui::test]
fn same_byte_length_replacement_updates_ime_position(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.set_text("iii", cx));
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 1);
        state.update(cx, |state, cx| state.set_text("あ", cx));
        assert_eq!(window.simulate_next_frame(cx), 0, "描画前には予約しない");
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 1, "文字幅の変化を伝える");
    });
}

#[gpui::test]
fn wide_composition_reports_bounds_inside_field(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    let text = "あ".repeat(1000);
    let bounds = Bounds::from_corners(point(px(42.0), px(20.0)), point(px(52.0), px(44.0)));
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_and_mark_text_in_range(None, &text, None, window, cx)
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let start = state.update(cx, |state, cx| {
            state.bounds_for_range(0..0, bounds, window, cx).unwrap()
        });
        assert_eq!(
            start.left(),
            bounds.left(),
            "スクロールした変換開始位置を左端に収める"
        );
        let range = state.update(cx, |state, cx| {
            state.bounds_for_range(0..1000, bounds, window, cx).unwrap()
        });
        assert_eq!(range.right(), bounds.right(), "範囲の右端も欄に収める");
    });
}

#[gpui::test]
fn refocusing_schedules_ime_position_update(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.set_text("abc", cx));
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 1);
        window.blur(cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(window.simulate_next_frame(cx), 0);
        window.focus(&state.read(cx).focus_handle(cx), cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            window.simulate_next_frame(cx),
            1,
            "再フォーカス時に位置を伝える"
        );
    });
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
    let field = TextField::new("name", &state)
        .label("名前")
        .disabled(true)
        .error(true)
        .placeholder("入力")
        .a11y_node();
    assert_eq!(field.role, gpui::Role::TextInput);
    assert_eq!(field.label.as_deref(), Some("名前"));
    assert!(field.disabled);
    assert!(field.invalid);
    let search = SearchField::new("search", &state)
        .aria_label("メールを検索")
        .placeholder("メールを検索")
        .a11y_node();
    assert_eq!(search.role, gpui::Role::SearchInput);
    assert_eq!(search.label.as_deref(), Some("メールを検索"));
}

#[gpui::test]
fn keyboard_focus_visibility_is_exposed(cx: &mut TestAppContext) {
    use std::{cell::Cell, rc::Rc};
    let state = cx.new(TextFieldState::new);
    let visible = Rc::new(Cell::new(false));
    let cx = support::open(cx, {
        let visible = visible.clone();
        move |window, cx| {
            let (base, control) = TextField::new("focus-field", &state)
                .label("名前")
                .build(window, cx);
            visible.set(control.focus_visible);
            base.into_any_element()
        }
    });
    assert!(!visible.get());
    support::tab(cx);
    support::press(cx, "left");
    assert!(visible.get(), "キー操作でフォーカス表示を得る");
}

#[gpui::test]
fn field_names_are_optional_and_overridable(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    assert_eq!(TextField::new("field", &state).a11y_node().label, None);
    assert_eq!(
        TextField::new("field", &state)
            .label("名前")
            .a11y_node()
            .label
            .as_deref(),
        Some("名前")
    );
    assert_eq!(
        TextField::new("field", &state)
            .label("名前")
            .aria_label("名前を入力")
            .a11y_node()
            .label
            .as_deref(),
        Some("名前を入力")
    );
    assert_eq!(
        TextField::new("field", &state)
            .label("名前")
            .aria_label("")
            .a11y_node()
            .label,
        None
    );
    assert_eq!(SearchField::new("search", &state).a11y_node().label, None);
    assert_eq!(
        SearchField::new("search", &state)
            .aria_label("検索")
            .a11y_node()
            .label
            .as_deref(),
        Some("検索")
    );
    assert_eq!(
        SearchField::new("search", &state)
            .aria_label("")
            .a11y_node()
            .label,
        None
    );
}

fn with_text(state: &Entity<TextFieldState>, text: &str, cx: &mut VisualTestContext) {
    cx.update(|_, cx| state.update(cx, |state, cx| state.set_text(text, cx)));
}

#[gpui::test]
fn assistive_technology_sets_the_whole_value(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    with_text(&state, "古い", cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.a11y_set_value("新しい", window, cx))
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.text(), "新しい");
        assert_eq!(state.selected_range(), 9..9, "カーソルは末尾");
    });
}

#[gpui::test]
fn assistive_technology_replaces_the_selection(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    with_text(&state, "a😀b", cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.a11y_set_selection(1, 2, window, cx);
            state.a11y_replace_selection("x\ny", window, cx);
        })
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.text(), "ax yb", "1 行の入力欄なので改行は空白にする");
    });
}

#[gpui::test]
fn assistive_technology_ends_the_composition(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_and_mark_text_in_range(None, "か", None, window, cx);
            state.a11y_set_value("確定", window, cx);
        })
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.text(), "確定");
        assert_eq!(state.marked_range(), None);
    });
}

#[gpui::test]
fn selection_is_reported_and_set_in_graphemes(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    let family = "👨‍👩‍👧";
    with_text(&state, &format!("a{family}b"), cx);
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.a11y_selection(), (3, 3), "カーソルは 3 文字の末尾");
    });
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.a11y_set_selection(2, 1, window, cx))
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.selected_range(), 1..1 + family.len());
        assert_eq!(state.a11y_selection(), (2, 1), "逆向きの選択を保つ");
    });
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.a11y_set_selection(0, 99, window, cx))
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(
            state.selected_range(),
            0..state.text().len(),
            "範囲外は末尾に収める"
        );
        assert_eq!(state.a11y_selection(), (0, 3));
    });
}

#[gpui::test]
fn character_lengths_follow_graphemes(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    with_text(&state, "aあ😀👨‍👩‍👧", cx);
    cx.read_entity(&state, |state, _| {
        let lengths = state.a11y_character_lengths();
        assert_eq!(&lengths[..3], &[1, 3, 4]);
        assert_eq!(lengths.len(), 4);
        let total: usize = lengths.iter().map(|length| *length as usize).sum();
        assert_eq!(total, state.text().len());
    });
}

#[gpui::test]
fn character_lengths_split_a_grapheme_longer_than_a_byte_can_hold(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    let long = format!("e{}", "\u{301}".repeat(200));
    with_text(&state, &long, cx);
    cx.read_entity(&state, |state, _| {
        let lengths = state.a11y_character_lengths();
        assert!(lengths.len() > 1);
        let total: usize = lengths.iter().map(|length| *length as usize).sum();
        assert_eq!(total, long.len());
        assert!(lengths.iter().all(|length| *length > 0));
    });
}
