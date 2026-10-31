//! TextField の入力（キー入力、IME の変換中の文字、編集のキー操作）。

mod support;

use gpui::{
    Axis, Bounds, Entity, EntityInputHandler, Focusable, InputEvent, KeyBinding, Modifiers,
    MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Role, ScrollDelta, ScrollEvent,
    TestAppContext, VisualTestContext, actions, black, div, point, prelude::*, px,
};
use gpui_headless::{
    Scrollbar, ScrollbarGeometry, ScrollbarSource, SearchField, TextField, TextFieldScroll,
    TextFieldState, TextPaint,
};
use std::cell::RefCell;
use std::rc::Rc;

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
            state.a11y_set_selection((0, 1), (0, 2), window, cx);
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
        assert_eq!(
            state.a11y_selection(),
            ((0, 3), (0, 3)),
            "カーソルは 3 文字の末尾"
        );
    });
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.a11y_set_selection((0, 2), (0, 1), window, cx)
        })
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.selected_range(), 1..1 + family.len());
        assert_eq!(
            state.a11y_selection(),
            ((0, 2), (0, 1)),
            "逆向きの選択を保つ"
        );
    });
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.a11y_set_selection((0, 0), (0, 99), window, cx)
        })
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(
            state.selected_range(),
            0..state.text().len(),
            "範囲外は末尾に収める"
        );
        assert_eq!(state.a11y_selection(), ((0, 0), (0, 3)));
    });
}

#[gpui::test]
fn character_lengths_follow_graphemes(cx: &mut TestAppContext) {
    let (state, cx) = open(cx);
    with_text(&state, "aあ😀👨‍👩‍👧", cx);
    cx.read_entity(&state, |state, _| {
        let lengths = state.a11y_lines().remove(0);
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
        let lengths = state.a11y_lines().remove(0);
        assert!(lengths.len() > 1);
        let total: usize = lengths.iter().map(|length| *length as usize).sum();
        assert_eq!(total, long.len());
        assert!(lengths.iter().all(|length| *length > 0));
    });
}

actions!(
    test,
    [
        /// 親の要素に届いたキー。
        ParentKey
    ]
);

const MACOS: bool = cfg!(target_os = "macos");
const DOCUMENT_START: &str = if MACOS { "cmd-up" } else { "ctrl-home" };
const DOCUMENT_END: &str = if MACOS { "cmd-down" } else { "ctrl-end" };
const COPY: &str = if MACOS { "cmd-c" } else { "ctrl-c" };
const CUT: &str = if MACOS { "cmd-x" } else { "ctrl-x" };

const FIELD: &str = "multi-line-field";
const WIDTH: f32 = 102.0;
/// 1 行に入る文字の数（`NoopTextSystem` の 1 文字は 9.6px、幅は `WIDTH` からキャレットの分を引く）。
const ROW: &str = "0123456789";

/// `limits` があれば複数行の欄を開く。親はキーとホイールの届いた回数を数える。
fn open_field<'a>(
    cx: &'a mut TestAppContext,
    text: &str,
    limits: Option<(usize, usize)>,
) -> (
    Entity<TextFieldState>,
    Entity<(usize, usize)>,
    &'a mut VisualTestContext,
) {
    let state = cx.new(|cx| TextFieldState::new(cx).with_text(text));
    let parent = cx.new(|_| (0, 0));
    let cx = support::open(cx, {
        let state = state.clone();
        let parent = parent.clone();
        move |window, cx| {
            let mut field = TextField::new("field", &state).label("本文");
            if let Some((min, max)) = limits {
                field = field.multi_line(min, max);
            }
            let (base, control) = field.build(window, cx);
            let color = black();
            let keys = parent.clone();
            let wheel = parent.clone();
            div()
                .key_context("TestParent")
                .on_action(move |_: &ParentKey, _, cx| keys.update(cx, |count, _| count.0 += 1))
                .id("parent")
                .on_scroll(move |_, _, cx| wheel.update(cx, |count, _| count.1 += 1))
                .child(
                    base.w(px(WIDTH))
                        .debug_selector(|| FIELD.into())
                        .child(control.input(
                            TextPaint {
                                text: color,
                                placeholder: color,
                                caret: color,
                                selection: color,
                                caret_width: px(2.0),
                            },
                            None,
                        )),
                )
                .into_any_element()
        }
    });
    cx.update(|_, cx| {
        cx.bind_keys(
            ["enter", "up", "down"].map(|key| KeyBinding::new(key, ParentKey, Some("TestParent"))),
        )
    });
    support::tab(cx);
    (state, parent, cx)
}

fn cursor(state: &Entity<TextFieldState>, cx: &mut VisualTestContext) -> usize {
    cx.read_entity(state, |state, _| state.selected_range().end)
}

#[gpui::test]
fn edge_keys_on_an_empty_field_ignore_the_placeholder(cx: &mut TestAppContext) {
    let (state, cx) = open_with_placeholder(cx, Some("名前を入力"));
    for key in ["end", "shift-end", "shift-home", COPY, CUT] {
        support::press(cx, key);
        cx.read_entity(&state, |state, _| {
            assert_eq!(state.text(), "");
            assert_eq!(state.selected_range(), 0..0, "placeholder の位置を使わない");
        });
    }
}

#[gpui::test]
fn click_before_the_next_paint_keeps_the_selection_in_the_text(cx: &mut TestAppContext) {
    let (state, cx) = open_with_placeholder(cx, Some("0123456789"));
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_text_in_range(None, "a", window, cx)
        });
        window.dispatch_event(
            MouseDownEvent {
                position: point(px(76.0), px(20.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::shift(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(
            state.selected_range(),
            1..1,
            "placeholder の配置の位置を入力の文字列に使わない"
        );
    });
}

#[gpui::test]
fn enter_inserts_a_line_break_and_stays_in_the_field(cx: &mut TestAppContext) {
    let (state, parent, cx) = open_field(cx, "ab", Some((1, 8)));
    support::press(cx, "enter");
    support::press(cx, "shift-enter");
    assert_eq!(
        cx.read_entity(&state, |state, _| state.text().to_owned()),
        "ab\n\n"
    );
    support::press(cx, "up");
    assert_eq!(cursor(&state, cx), 3, "空の段落へ上がる");
    assert_eq!(cx.read_entity(&parent, |count, _| count.0), 0);
}

#[gpui::test]
fn single_line_field_passes_enter_and_arrows_to_its_parent(cx: &mut TestAppContext) {
    let (state, parent, cx) = open_field(cx, "ab", None);
    for key in ["enter", "up", "down"] {
        support::press(cx, key);
    }
    assert_eq!(
        cx.read_entity(&state, |state, _| state.text().to_owned()),
        "ab"
    );
    assert_eq!(cx.read_entity(&parent, |count, _| count.0), 3);
}

#[gpui::test]
fn tab_leaves_the_multi_line_field(cx: &mut TestAppContext) {
    let (state, _, cx) = open_field(cx, "ab", Some((1, 8)));
    let handle = cx.read_entity(&state, |state, cx| state.focus_handle(cx));
    assert!(cx.update(|window, _| handle.is_focused(window)));
    support::tab(cx);
    assert_eq!(
        cx.read_entity(&state, |state, _| state.text().to_owned()),
        "ab"
    );
}

#[gpui::test]
fn enter_while_composing_keeps_the_composition(cx: &mut TestAppContext) {
    let (state, _, cx) = open_field(cx, "", Some((1, 8)));
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.replace_and_mark_text_in_range(None, "か", None, window, cx)
        })
    });
    support::press(cx, "enter");
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.text(), "か");
        assert!(state.marked_range().is_some());
    });
}

#[gpui::test]
fn input_keeps_or_flattens_line_breaks_by_field_kind(cx: &mut TestAppContext) {
    let insert = |state: &Entity<TextFieldState>, text: &str, cx: &mut VisualTestContext| {
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.replace_text_in_range(None, text, window, cx)
            })
        })
    };
    let (multi, _, multi_cx) = open_field(cx, "", Some((1, 8)));
    insert(&multi, "a\r\nb\rc", multi_cx);
    assert_eq!(
        multi_cx.read_entity(&multi, |state, _| state.text().to_owned()),
        "a\nb\nc"
    );

    let mut app = multi_cx.cx.clone();
    let (single, _, single_cx) = open_field(&mut app, "", None);
    insert(&single, "a\r\nb", single_cx);
    assert_eq!(
        single_cx.read_entity(&single, |state, _| state.text().to_owned()),
        "a b",
        "1 行の欄は改行を空白にする"
    );
    insert(&single, "\r\n", single_cx);
    assert_eq!(
        single_cx.read_entity(&single, |state, _| state.text().to_owned()),
        "a b",
        "改行だけの入力は無視する"
    );
}

#[gpui::test]
fn single_line_field_draws_a_line_break_from_set_text(cx: &mut TestAppContext) {
    let (state, _, cx) = open_field(cx, "a\nb", None);
    support::press(cx, "home");
    assert_eq!(cursor(&state, cx), 0, "改行があっても 1 行として扱う");
}

#[gpui::test]
fn arrows_keep_the_column_and_stop_at_the_ends(cx: &mut TestAppContext) {
    let text = format!("{ROW}\nab\n{ROW}");
    let (state, _, cx) = open_field(cx, &text, Some((1, 8)));
    support::press(cx, "left");
    support::press(cx, "left");
    let expected = text.len() - 2;
    assert_eq!(cursor(&state, cx), expected);
    support::press(cx, "up");
    assert_eq!(cursor(&state, cx), 13, "短い行は行末に収める");
    support::press(cx, "up");
    assert_eq!(cursor(&state, cx), 8, "x を保つ");
    support::press(cx, "up");
    assert_eq!(cursor(&state, cx), 0, "先頭より上は文字列の先頭");
    support::press(cx, DOCUMENT_END);
    assert_eq!(cursor(&state, cx), text.len());
    support::press(cx, "down");
    assert_eq!(cursor(&state, cx), text.len(), "末尾より下は文字列の末尾");
    support::press(cx, DOCUMENT_START);
    assert_eq!(cursor(&state, cx), 0);
    support::press(cx, "shift-down");
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.selected_range(), 0..11, "1 行下の同じ列まで選択する")
    });
}

#[gpui::test]
fn page_keys_move_by_the_visible_rows(cx: &mut TestAppContext) {
    let text = (0..8)
        .map(|index| format!("line{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let (state, _, cx) = open_field(cx, &text, Some((1, 3)));
    support::press(cx, DOCUMENT_START);
    support::press(cx, "pagedown");
    assert_eq!(cursor(&state, cx), 18, "3 行下の先頭");
    support::press(cx, "pageup");
    assert_eq!(cursor(&state, cx), 0);
}

#[gpui::test]
fn multi_line_role_and_a11y_lines(cx: &mut TestAppContext) {
    let state = cx.new(TextFieldState::new);
    cx.update(|_| {
        let node = TextField::new("body", &state).multi_line(1, 4).a11y_node();
        assert_eq!(node.role, Role::MultilineTextInput);
        let node = TextField::new("name", &state).a11y_node();
        assert_eq!(node.role, Role::TextInput);
    });
    let (state, _, cx) = open_field(cx, "aあ\n\n", Some((1, 8)));
    cx.read_entity(&state, |state, _| {
        assert_eq!(
            state.a11y_lines(),
            vec![vec![1, 3, 1], vec![1], vec![]],
            "段落ごとの文字。改行は段落の最後の文字で、末尾に空の段落が付く"
        );
        assert_eq!(state.a11y_selection(), ((2, 0), (2, 0)));
    });
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.a11y_set_selection((1, 0), (0, 1), window, cx)
        })
    });
    cx.read_entity(&state, |state, _| {
        assert_eq!(state.selected_range(), 1..5);
        assert_eq!(
            state.a11y_selection(),
            ((1, 0), (0, 1)),
            "逆向きの選択を保つ"
        );
    });
}

/// 描いた結果を `debug_bounds` で測るテスト。`debug_selector` はデバッグビルドでだけ働く。
#[cfg(debug_assertions)]
mod measured {
    use super::*;
    use gpui::{Pixels, ScrollDelta, ScrollEvent, TouchPhase};

    fn line_height(cx: &mut VisualTestContext) -> Pixels {
        cx.update(|window, _| window.line_height())
    }

    fn field_bounds(cx: &mut VisualTestContext) -> Bounds<Pixels> {
        cx.debug_bounds(FIELD).expect("入力欄の位置")
    }

    /// 欄の左上に見えている文字のバイト位置。
    fn first_visible(state: &Entity<TextFieldState>, cx: &mut VisualTestContext) -> usize {
        let origin = field_bounds(cx).origin + point(px(1.0), px(1.0));
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.character_index_for_point(origin, window, cx)
            })
        })
        .expect("描画済みの欄の位置")
    }

    /// カーソルの範囲を IME が受け取る矩形の y（欄の上端から）。
    fn ime_row_y(state: &Entity<TextFieldState>, cx: &mut VisualTestContext) -> Pixels {
        let bounds = field_bounds(cx);
        let offset = cursor(state, cx);
        let rect = cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.bounds_for_range(offset..offset, bounds, window, cx)
            })
        });
        rect.expect("IME の矩形").top() - bounds.top()
    }

    fn wheel(cx: &mut VisualTestContext, rows: f32) {
        let position = field_bounds(cx).center();
        cx.simulate_event(ScrollEvent {
            position,
            delta: ScrollDelta::Lines(point(0.0, rows)),
            modifiers: Modifiers::none(),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });
        support::draw(cx);
    }

    #[gpui::test]
    fn home_and_end_stay_on_the_wrapped_row(cx: &mut TestAppContext) {
        let text = format!("{ROW}{ROW}");
        let (state, _, cx) = open_field(cx, &text, Some((1, 8)));
        let line = line_height(cx);
        support::press(cx, "home");
        assert_eq!(cursor(&state, cx), 10, "2 行目の先頭");
        assert_eq!(
            ime_row_y(&state, cx),
            line,
            "折り返し位置のカーソルは 2 行目に出る"
        );
        support::press(cx, "left");
        support::press(cx, "home");
        assert_eq!(cursor(&state, cx), 0);
        support::press(cx, "end");
        assert_eq!(cursor(&state, cx), 10, "1 行目の末尾");
        assert_eq!(
            ime_row_y(&state, cx),
            px(0.0),
            "End のカーソルは 1 行目に残る"
        );
        support::press(cx, "end");
        assert_eq!(cursor(&state, cx), 10, "もう一度押しても次の行へ進まない");
        support::press(cx, DOCUMENT_START);
        support::press(cx, "shift-end");
        assert_eq!(ime_row_y(&state, cx), px(0.0), "選択の End も 1 行目に残る");
    }

    #[gpui::test]
    fn height_follows_the_rows_within_the_limits(cx: &mut TestAppContext) {
        let (state, _, cx) = open_field(cx, "a", Some((2, 4)));
        let line = line_height(cx);
        assert_eq!(
            field_bounds(cx).size.height,
            line * 2usize,
            "最小の行数を確保する"
        );
        cx.update(|_, cx| state.update(cx, |state, cx| state.set_text("a\nb\nc", cx)));
        support::draw(cx);
        assert_eq!(
            field_bounds(cx).size.height,
            line * 3usize,
            "内容に合わせて伸びる"
        );
        let wrapped = format!("{ROW}{ROW}{ROW}{ROW}{ROW}{ROW}");
        cx.update(|_, cx| state.update(cx, |state, cx| state.set_text(wrapped, cx)));
        support::draw(cx);
        assert_eq!(
            field_bounds(cx).size.height,
            line * 4usize,
            "折り返しても上限で止まる"
        );
    }

    #[gpui::test]
    fn caret_scrolls_into_view_and_wheel_is_not_pulled_back(cx: &mut TestAppContext) {
        let text = format!("{ROW}\n{ROW}\n{ROW}\n{ROW}");
        let (state, parent, cx) = open_field(cx, &text, Some((2, 2)));
        support::press(cx, DOCUMENT_START);
        assert_eq!(first_visible(&state, cx), 0);
        support::press(cx, DOCUMENT_END);
        assert_eq!(first_visible(&state, cx), 22, "カーソルの行まで送る");
        wheel(cx, 1.0);
        assert_eq!(first_visible(&state, cx), 11, "ホイールで上へ戻る");
        assert_eq!(
            cx.read_entity(&parent, |count, _| count.1),
            0,
            "動いた分は親へ渡さない"
        );
        support::draw(cx);
        assert_eq!(
            first_visible(&state, cx),
            11,
            "再描画でカーソルへ引き戻さない"
        );
        wheel(cx, 5.0);
        assert_eq!(first_visible(&state, cx), 0);
        wheel(cx, 1.0);
        assert_eq!(
            cx.read_entity(&parent, |count, _| count.1),
            1,
            "端に達したら親へ渡す"
        );
    }

    #[gpui::test]
    fn single_line_field_scrolls_sideways_to_the_caret(cx: &mut TestAppContext) {
        let text = format!("{ROW}{ROW}{ROW}");
        let (state, _, cx) = open_field(cx, &text, None);
        support::press(cx, "left");
        assert!(first_visible(&state, cx) > 0, "カーソルの位置まで横に送る");
        support::press(cx, "home");
        assert_eq!(first_visible(&state, cx), 0);
    }

    #[gpui::test]
    fn click_after_scrolling_uses_the_scrolled_row(cx: &mut TestAppContext) {
        let text = format!("{ROW}\n{ROW}\n{ROW}\n{ROW}");
        let (state, _, cx) = open_field(cx, &text, Some((2, 2)));
        support::press(cx, DOCUMENT_END);
        let position = field_bounds(cx).origin + point(px(1.0), px(1.0));
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
        support::draw(cx);
        assert_eq!(cursor(&state, cx), 22, "見えている 1 行目の先頭");
    }

    #[gpui::test]
    fn click_selects_the_row_under_the_pointer(cx: &mut TestAppContext) {
        let text = format!("{ROW}\n{ROW}");
        let (state, _, cx) = open_field(cx, &text, Some((4, 8)));
        let bounds = field_bounds(cx);
        let line = line_height(cx);
        let position = bounds.origin + point(px(1.0), line * 1.5);
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
        support::draw(cx);
        assert_eq!(cursor(&state, cx), 11, "2 行目の先頭");
        let below = bounds.origin + point(px(1.0), bounds.size.height - px(1.0));
        cx.simulate_mouse_down(below, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(below, MouseButton::Left, Modifiers::none());
        support::draw(cx);
        assert_eq!(cursor(&state, cx), 11, "文字のない行は最後の行に収める");
    }

    #[gpui::test]
    fn ime_position_follows_the_row_and_the_scroll(cx: &mut TestAppContext) {
        let text = format!("{ROW}\n{ROW}\n{ROW}");
        let (state, _, cx) = open_field(cx, &text, Some((2, 2)));
        let line = line_height(cx);
        support::press(cx, DOCUMENT_END);
        assert_eq!(ime_row_y(&state, cx), line, "最後の行は 2 行目に見える");
        let bounds = field_bounds(cx);
        let rect = cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.bounds_for_range(13..25, bounds, window, cx)
            })
        });
        let rect = rect.expect("IME の矩形");
        assert_eq!(rect.size.height, line, "またがる範囲は開始行の高さ");
        assert_eq!(rect.right(), bounds.right());
    }
}

/// 複数行の欄と、その位置を読むスクロールバーを開く。
fn open_with_scrollbar<'a>(
    cx: &'a mut TestAppContext,
    text: &str,
    disabled: bool,
) -> (
    Entity<TextFieldState>,
    Rc<RefCell<Option<TextFieldScroll>>>,
    &'a mut VisualTestContext,
) {
    let state = cx.new(|cx| TextFieldState::new(cx).with_text(text));
    let scroll = Rc::new(RefCell::new(None));
    let cx = support::open(cx, {
        let state = state.clone();
        let scroll = scroll.clone();
        move |window, cx| {
            let (base, control) = TextField::new("field", &state)
                .label("本文")
                .multi_line(1, 4)
                .disabled(disabled)
                .build(window, cx);
            let source = control.scroll(cx).expect("複数行の欄は位置を持つ");
            *scroll.borrow_mut() = Some(source.clone());
            let color = black();
            let bar = Scrollbar::new(
                "bar",
                source,
                ScrollbarGeometry {
                    track_inset: px(2.0),
                    cross_inset: px(0.0),
                    thickness: px(4.0),
                    min_thumb_length: px(24.0),
                    max_thumb_fraction: 0.9,
                    hit_width: px(24.0),
                },
            )
            .build(window, cx, |_, _, _| {});
            base.w(px(WIDTH))
                .debug_selector(|| FIELD.into())
                .child(control.input(
                    TextPaint {
                        text: color,
                        placeholder: color,
                        caret: color,
                        selection: color,
                        caret_width: px(2.0),
                    },
                    None,
                ))
                .child(bar)
                .into_any_element()
        }
    });
    (state, scroll, cx)
}

fn source(scroll: &Rc<RefCell<Option<TextFieldScroll>>>) -> TextFieldScroll {
    scroll.borrow().clone().expect("描画で位置が渡されている")
}

fn offset(scroll: &Rc<RefCell<Option<TextFieldScroll>>>) -> Pixels {
    ScrollbarSource::offset(&source(scroll), Axis::Vertical)
}

fn max_offset(scroll: &Rc<RefCell<Option<TextFieldScroll>>>) -> Pixels {
    ScrollbarSource::max_offset(&source(scroll), Axis::Vertical)
}

fn wheel_on_field(
    scroll: &Rc<RefCell<Option<TextFieldScroll>>>,
    cx: &mut VisualTestContext,
    delta: Pixels,
) {
    let bounds = ScrollbarSource::viewport(&source(scroll));
    cx.update(|window, cx| {
        window.dispatch_event(
            ScrollEvent {
                position: bounds.center(),
                delta: ScrollDelta::Pixels(point(px(0.0), delta)),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    });
    support::draw(cx);
}

#[gpui::test]
fn scroll_reports_hidden_rows(cx: &mut TestAppContext) {
    let (_, scroll, cx) = open_with_scrollbar(cx, "1\n2\n3\n4", false);
    assert_eq!(max_offset(&scroll), px(0.0), "4 行は上限に収まる");
    let mut app = cx.cx.clone();
    let (_, scroll, cx) = open_with_scrollbar(&mut app, &"行\n".repeat(30), false);
    let viewport = ScrollbarSource::viewport(&source(&scroll));
    assert!(viewport.size.height > px(0.0), "表示領域は入力行の範囲");
    assert!(max_offset(&scroll) > px(0.0), "隠れた行の分だけ動かせる");
    assert_eq!(
        ScrollbarSource::max_offset(&source(&scroll), Axis::Horizontal),
        px(0.0),
        "横には動かない"
    );
    support::draw(cx);
}

#[gpui::test]
fn scrollbar_position_survives_redraw_until_the_caret_moves(cx: &mut TestAppContext) {
    let (state, scroll, cx) = open_with_scrollbar(cx, &"行\n".repeat(30), false);
    support::tab(cx);
    let max = max_offset(&scroll);
    ScrollbarSource::set_offset(&source(&scroll), Axis::Vertical, px(-10.0));
    assert_eq!(offset(&scroll), px(0.0), "先頭より前へは動かない");
    ScrollbarSource::set_offset(&source(&scroll), Axis::Vertical, max + px(10.0));
    assert_eq!(offset(&scroll), max, "末尾より後へは動かない");
    ScrollbarSource::set_offset(&source(&scroll), Axis::Vertical, max / 2.0);
    support::draw(cx);
    assert_eq!(offset(&scroll), max / 2.0, "再描画で引き戻さない");
    wheel_on_field(&scroll, cx, px(-24.0));
    assert_eq!(
        offset(&scroll),
        max / 2.0 + px(24.0),
        "ホイールは同じ位置から動く"
    );
    support::press(cx, "ctrl-home");
    assert_eq!(cursor(&state, cx), 0);
    assert_eq!(
        offset(&scroll),
        px(0.0),
        "カーソルが動くと見える位置まで戻る"
    );
}

#[gpui::test]
fn disabled_field_still_scrolls_with_the_wheel(cx: &mut TestAppContext) {
    let (_, scroll, cx) = open_with_scrollbar(cx, &"行\n".repeat(30), true);
    assert!(max_offset(&scroll) > px(0.0));
    wheel_on_field(&scroll, cx, px(-24.0));
    assert_eq!(offset(&scroll), px(24.0), "無効の欄でも隠れた行を読める");
}

#[gpui::test]
fn selection_drag_continues_over_the_scrollbar(cx: &mut TestAppContext) {
    let (state, scroll, cx) = open_with_scrollbar(cx, &"行\n".repeat(30), false);
    let bounds = ScrollbarSource::viewport(&source(&scroll));
    let start = bounds.origin + point(px(1.0), px(1.0));
    // バーの操作領域（右端の 24px）の上。
    let over_bar = point(bounds.right() - px(2.0), bounds.bottom() - px(2.0));
    cx.update(|window, cx| {
        for event in [
            MouseDownEvent {
                position: start,
                button: MouseButton::Left,
                ..Default::default()
            }
            .to_platform_input(),
            MouseMoveEvent {
                position: start,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            }
            .to_platform_input(),
        ] {
            window.dispatch_event(event, cx);
        }
    });
    support::draw(cx);
    cx.update(|window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position: over_bar,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    });
    support::draw(cx);
    let selected = cx.read_entity(&state, |state, _| state.selected_range());
    assert!(
        !selected.is_empty(),
        "バーの上でも選択が広がる: {selected:?}"
    );
    assert_eq!(offset(&scroll), px(0.0), "バーは動かない");
}
