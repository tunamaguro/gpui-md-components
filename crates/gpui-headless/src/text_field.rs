// The editing logic (TextFieldState, its EntityInputHandler impl and TextContent) is ported from
// gpui's example: https://github.com/zed-industries/zed crates/gpui/examples/input.rs @ 37faef81
// Copyright 2022 - 2025 Zed Industries, Inc. Licensed under the Apache License, Version 2.0.

//! 1 行と複数行のテキスト編集と入力欄。

use crate::keys::{
    Backspace, Copy, Cut, Delete, DocumentEnd, DocumentStart, Down, End, Home, Left,
    MULTI_LINE_FIELD_CONTEXT, Newline, PageDown, PageUp, Paste, Right, SelectAll,
    SelectDocumentEnd, SelectDocumentStart, SelectDown, SelectEnd, SelectHome, SelectLeft,
    SelectPageDown, SelectPageUp, SelectRight, SelectUp, TEXT_FIELD_CONTEXT, Up,
};
use crate::scroll::{gesture_step, reveal_in_scopes};
use crate::text_layout::{Caret, TextLayout};
use crate::{A11yExt, A11yNode};
use gpui::{
    A11ySubtreeBuilder, App, AvailableSpace, Bounds, ClipboardItem, ContentMask, Context,
    CursorStyle, Div, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    EventEmitter, FocusHandle, Focusable, Font, Global, GlobalElementId, Hsla, InspectorElementId,
    IntoElement, KeyContext, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, PaintQuad,
    Pixels, Point, Role, ScrollEvent, SharedString, Stateful, Style, Subscription, TextRun,
    UTF16Selection, UnderlineStyle, Window, WindowId, accesskit, div, fill, point, prelude::*, px,
    relative, size,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

// IME の変換は OS 側にあり、`marked_range` を消しただけでは終わらない。Windows では変換が
// 続き、次の更新が別の場所へ入ってしまう（フォーカスを移した先の欄など）。
//
// gpui に変換を終わらせる公開 API はない。ただし Windows 実装（`update_ime_enabled`）は、
// フレームの最後に `accepts_text_input` が false に変わると `CPS_COMPLETE` を送り、確定文字を
// 入力ハンドラーへ返す。そこで 1 フレームだけ false を返して変換を終わらせる。
//
// 他の環境では使わない。X11 / Wayland はクリックで gpui が変換を終え、Web で false を返すと
// IME の textarea が読み取り専用になる。macOS は未確認。テストは Linux で動くので同じ経路を使う。
const ENDS_IME_WITH_HANDLER: bool = cfg!(any(target_os = "windows", test));

/// 変換を終わらせるための、入力を受け付けないフレームの進み具合。
///
/// `on_next_frame` は次の描画より先に走るので、依頼の時点で解除を予約すると false のまま
/// フレームを終えられない。`Requested` のまま描画まで待ち、paint で解除を予約する。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImeFinish {
    /// 通常の状態。
    Idle,
    /// 次の描画で変換を終える。
    Requested,
    /// このフレームの最後に gpui が `CPS_COMPLETE` を送る。次のフレームで `Idle` に戻す。
    Scheduled,
}

/// 変換中にフォーカスが移ったことを、移動先の欄へ伝える。
///
/// `CPS_COMPLETE` はそのとき登録されている入力ハンドラー、つまり移動先へ届く。移動元は
/// もうハンドラーではないので、移動先が 1 フレーム入力を止める必要がある。移動先の欄を
/// 直接は知らないのでグローバルに置く。フォーカスされた欄は他のウィンドウにもあるので、
/// どのウィンドウの話かも持つ。
#[derive(Default)]
struct PendingImeFinish {
    generation: u64,
    window: Option<WindowId>,
}

impl Global for PendingImeFinish {}

/// 入力行の色とキャレットの幅。
pub struct TextPaint {
    /// 入力文字の色。
    pub text: Hsla,
    /// プレースホルダーの色。
    pub placeholder: Hsla,
    /// キャレットの色。
    pub caret: Hsla,
    /// 選択範囲の色。
    pub selection: Hsla,
    /// キャレットの幅。
    pub caret_width: Pixels,
}

/// 入力の内容が変わったときのイベント。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFieldEvent {
    /// 文字が変わった（変換中の文字を含む）。
    Changed,
}

/// IME に伝える入力の範囲と位置。
#[derive(PartialEq)]
struct ImeAnchor {
    bounds: Bounds<Pixels>,
    position: Point<Pixels>,
}

/// 複数行の欄で、段落の改行を選択したときに行の右へ延ばす幅（行の高さに対する割合）。
const LINE_BREAK_SELECTION_WIDTH: f32 = 0.25;

/// テキストフィールドの中身（文字列、選択範囲、IME の変換中の範囲）。
///
/// `cx.new(TextFieldState::new)` で作り、[`TextField`] に渡す。内容が変わると
/// [`TextFieldEvent::Changed`] を出す。
pub struct TextFieldState {
    focus_handle: FocusHandle,
    content: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    ime_finish: ImeFinish,
    finish_after_click_out: bool,
    blur_subscription: Option<Subscription>,
    last_layout: Option<TextLayout>,
    last_bounds: Option<Bounds<Pixels>>,
    last_ime_anchor: Option<ImeAnchor>,
    scroll: Point<Pixels>,
    is_selecting: bool,
    multi_line: bool,
    /// 折り返し位置のカーソルを、前の表示行の末尾に出すか。
    caret_upstream: bool,
    /// 上下の移動で保つカーソルの x。
    vertical_x: Option<Pixels>,
    /// 次の描画でカーソルが見える位置まで送るか。ホイールで動かした位置を描画で戻さないために、
    /// カーソルが動いたときだけ立てる。
    reveal_caret: bool,
    /// ホイールの精密スクロールの状態（`gesture_step`）。
    scroll_gesture: (bool, bool),
}

impl EventEmitter<TextFieldEvent> for TextFieldState {}

impl Focusable for TextFieldState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TextFieldState {
    /// 空の状態を作る。
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            content: String::new(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            ime_finish: ImeFinish::Idle,
            finish_after_click_out: false,
            blur_subscription: None,
            last_layout: None,
            last_bounds: None,
            last_ime_anchor: None,
            scroll: point(px(0.0), px(0.0)),
            is_selecting: false,
            multi_line: false,
            caret_upstream: false,
            vertical_x: None,
            reveal_caret: false,
            scroll_gesture: (false, false),
        }
    }

    /// 初期値を入れる。改行は CRLF と CR も LF にそろえる。
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.content = normalize_line_breaks(&text.into()).into_owned();
        let end = self.content.len();
        self.selected_range = end..end;
        self
    }

    /// 支援技術に渡す、段落（改行で区切った行）ごとの、文字（書記素）の UTF-8 の長さ。
    ///
    /// 改行は段落の最後の 1 文字として数える。文字列が改行で終わるときは最後に空の段落が付く。
    pub fn a11y_lines(&self) -> Vec<Vec<u8>> {
        paragraphs(&self.content)
            .into_iter()
            .map(|paragraph| character_lengths(&self.content[paragraph]))
            .collect()
    }

    /// 支援技術に渡す選択範囲の始点と終点（段落の番号と、段落の中の文字の番号）。
    pub fn a11y_selection(&self) -> ((usize, usize), (usize, usize)) {
        selection_characters(&self.content, &self.selected_range, self.selection_reversed)
    }

    /// 支援技術からの要求で、文字列全体を置き換える。
    pub fn a11y_set_value(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let range = 0..self.content.encode_utf16().count();
        self.replace_from_a11y(range, text, window, cx);
    }

    /// 支援技術からの要求で、選択範囲を置き換える。
    pub fn a11y_replace_selection(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.range_to_utf16(&self.selected_range);
        self.replace_from_a11y(range, text, window, cx);
    }

    /// 支援技術からの要求で、選択範囲を移す。位置は段落の番号と文字の番号。範囲外は末尾に収める。
    pub fn a11y_set_selection(
        &mut self,
        anchor: (usize, usize),
        focus: (usize, usize),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.end_composition();
        let (anchor_offset, focus_offset) = (
            offset_of_character(&self.content, anchor),
            offset_of_character(&self.content, focus),
        );
        self.selected_range = anchor_offset.min(focus_offset)..anchor_offset.max(focus_offset);
        self.selection_reversed = focus_offset < anchor_offset;
        self.caret_moved();
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    /// 範囲を明示して置き換える（範囲なしは `CPS_COMPLETE` の確定文字として捨てられる）。
    fn replace_from_a11y(
        &mut self,
        range_utf16: Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.end_composition();
        self.replace_text_in_range(Some(range_utf16), text, window, cx);
    }

    fn end_composition(&mut self) {
        if self.marked_range.take().is_some() && ENDS_IME_WITH_HANDLER {
            self.ime_finish = ImeFinish::Requested;
        }
    }

    /// 入力されている文字列。
    pub fn text(&self) -> &str {
        &self.content
    }

    /// 文字列を置き換え、カーソルを末尾に置く。改行は CRLF と CR も LF にそろえる。
    /// 1 行の欄では LF は残り、空白として表示する。
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.content = normalize_line_breaks(&text.into()).into_owned();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        self.caret_moved();
        cx.emit(TextFieldEvent::Changed);
        cx.notify();
    }

    /// 今の文字列に対応する配置。描画前や、変更のあと次の描画までは `None`。
    fn current_layout(&self) -> Option<&TextLayout> {
        self.last_layout
            .as_ref()
            .filter(|layout| layout.is_for(&self.content))
    }

    /// IME に伝える位置。欄からはみ出す位置は欄の中に収める。
    fn ime_position_for_offset(
        &self,
        offset: usize,
        bounds: Bounds<Pixels>,
    ) -> Option<Point<Pixels>> {
        if self.content.is_empty() {
            return Some(bounds.origin);
        }
        let layout = self.current_layout()?;
        let upstream = self.caret_upstream && offset == self.cursor_offset();
        let position = layout.position(offset, upstream);
        let bottom = (bounds.bottom() - layout.line_height()).max(bounds.top());
        Some(point(
            (bounds.left() - self.scroll.x + position.x).clamp(bounds.left(), bounds.right()),
            (bounds.top() - self.scroll.y + position.y).clamp(bounds.top(), bottom),
        ))
    }

    /// 選択範囲（UTF-8 のバイト位置）。
    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    /// IME の変換中の範囲（UTF-8 のバイト位置）。
    pub fn marked_range(&self) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    fn observe_blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.blur_subscription.is_some() {
            return;
        }
        self.blur_subscription =
            Some(cx.on_blur(&self.focus_handle, window, |state, window, cx| {
                state.finish_after_click_out = false;
                // gpui はウィンドウの非アクティブ化でも blur を通知する。別のウィンドウへ移った
                // だけなら変換を続ける（Compose の TextFieldDecoratorModifier と同じ）。
                if state.focus_handle.is_focused(window) {
                    return;
                }
                // 文字は残して未確定だけ解除する。`Requested` は、クリックのあと描画より先に
                // フォーカスが移った場合。この欄はもう描画で処理できないので移動先へ渡す。
                let marked = state.marked_range.take().is_some();
                let requested = state.ime_finish == ImeFinish::Requested;
                if marked || requested {
                    if requested {
                        state.ime_finish = ImeFinish::Idle;
                    }
                    cx.notify();
                    if ENDS_IME_WITH_HANDLER {
                        // blur は描画中に届くため、その場の refresh は無視される。
                        // 次の描画前に refresh し、`.cached()` の移動先も描き直す。
                        // 移動先の描画が済んだ次のフレームで取り下げる。入力欄以外へ移ったときに
                        // 依頼が残り、次にフォーカスした欄で入力を止めないようにする。
                        let pending = cx.default_global::<PendingImeFinish>();
                        pending.generation = pending.generation.wrapping_add(1);
                        pending.window = Some(window.window_handle().window_id());
                        let generation = pending.generation;
                        window.on_next_frame(move |window, _| {
                            window.refresh();
                            window.on_next_frame(move |_, cx| {
                                let pending = cx.default_global::<PendingImeFinish>();
                                if pending.generation == generation {
                                    pending.window = None;
                                }
                            });
                        });
                    }
                }
            }));
    }

    /// カーソルを動かしたあとの共通の処理。上下の移動の x と折り返しの扱いを解除する。
    fn caret_moved(&mut self) {
        self.caret_upstream = false;
        self.vertical_x = None;
        self.reveal_caret = true;
    }

    fn left(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(
                previous_boundary(&self.content, self.cursor_offset()),
                window,
                cx,
            );
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    fn right(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(
                next_boundary(&self.content, self.selected_range.end),
                window,
                cx,
            );
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    fn select_left(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(
            previous_boundary(&self.content, self.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_right(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(
            next_boundary(&self.content, self.cursor_offset()),
            window,
            cx,
        );
    }

    fn select_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    fn move_caret(&mut self, caret: Caret, window: &Window, cx: &mut Context<Self>) {
        self.move_to(caret.offset, window, cx);
        self.caret_upstream = caret.upstream;
    }

    fn select_caret(&mut self, caret: Caret, window: &Window, cx: &mut Context<Self>) {
        self.select_to(caret.offset, window, cx);
        self.caret_upstream = caret.upstream;
    }

    /// カーソルのある表示行の先頭。配置がなければ段落の先頭。
    fn row_start(&self) -> Caret {
        let cursor = self.cursor_offset();
        match self.current_layout() {
            Some(layout) => layout.row_start(layout.row_of(cursor, self.caret_upstream)),
            None => Caret {
                offset: self.content[..cursor]
                    .rfind('\n')
                    .map_or(0, |index| index + 1),
                upstream: false,
            },
        }
    }

    /// カーソルのある表示行の末尾。配置がなければ段落の末尾。
    fn row_end(&self) -> Caret {
        let cursor = self.cursor_offset();
        match self.current_layout() {
            Some(layout) => layout.row_end(layout.row_of(cursor, self.caret_upstream)),
            None => Caret {
                offset: self.content[cursor..]
                    .find('\n')
                    .map_or(self.content.len(), |index| cursor + index),
                upstream: false,
            },
        }
    }

    fn home(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_caret(self.row_start(), window, cx);
    }

    fn end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_caret(self.row_end(), window, cx);
    }

    fn select_home(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_caret(self.row_start(), window, cx);
    }

    fn select_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_caret(self.row_end(), window, cx);
    }

    fn document_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn document_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    fn select_document_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, window, cx);
    }

    fn select_document_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.content.len(), window, cx);
    }

    /// `rows` 行だけ上下に動かした先のカーソルと、保つ x。先頭より上は文字列の先頭、末尾より下は
    /// 文字列の末尾にする（Compose の `TextFieldPreparedSelection.jumpByLinesOffset`）。
    fn caret_by_rows(&self, rows: isize) -> (Caret, Option<Pixels>) {
        let edge = |offset| Caret {
            offset,
            upstream: false,
        };
        let backward = rows < 0;
        let Some(layout) = self.current_layout() else {
            return (
                edge(if backward { 0 } else { self.content.len() }),
                self.vertical_x,
            );
        };
        let cursor = self.cursor_offset();
        let x = self
            .vertical_x
            .unwrap_or_else(|| layout.position(cursor, self.caret_upstream).x);
        let target = layout.row_of(cursor, self.caret_upstream) as isize + rows;
        let caret = if target < 0 {
            edge(0)
        } else if target as usize >= layout.row_count() {
            edge(self.content.len())
        } else {
            let line_height = layout.line_height();
            layout.caret_at(point(x, line_height * target as usize + line_height / 2.0))
        };
        (caret, Some(x))
    }

    /// 欄に見えている行の数（1 以上）。
    fn visible_rows(&self) -> isize {
        let (Some(bounds), Some(layout)) = (self.last_bounds, self.last_layout.as_ref()) else {
            return 1;
        };
        ((bounds.size.height / layout.line_height()).floor() as isize).max(1)
    }

    fn move_rows(&mut self, rows: isize, window: &Window, cx: &mut Context<Self>) {
        let (caret, x) = self.caret_by_rows(rows);
        self.move_caret(caret, window, cx);
        self.vertical_x = x;
    }

    fn select_rows(&mut self, rows: isize, window: &Window, cx: &mut Context<Self>) {
        let (caret, x) = self.caret_by_rows(rows);
        self.select_caret(caret, window, cx);
        self.vertical_x = x;
    }

    fn newline(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // gpui_web は変換中の判定より先にキーを渡すので、変換の確定の Enter もここへ来る。
        if self.marked_range.is_none() {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    fn backspace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = previous_boundary(&self.content, self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = next_boundary(&self.content, self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            self.copy(cx);
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // キャレットをクリックした位置に置くため、IME の確定文字を待たずにここで確定する。
        // 変換はそのままだと続き、次の更新でキャレットが変換範囲へ戻ってしまう。
        if ENDS_IME_WITH_HANDLER && self.marked_range.take().is_some() {
            self.ime_finish = ImeFinish::Requested;
        }
        self.is_selecting = true;
        window.request_virtual_keyboard();
        let caret = self.caret_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_caret(caret, window, cx);
        } else {
            self.move_caret(caret, window, cx)
        }
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_caret(self.caret_for_mouse_position(event.position), window, cx);
        }
    }

    /// ホイールで文字を送る。動かせたら `true`。動かせない分は祖先のスクロールに任せる。
    fn scroll_by_wheel(&mut self, event: &ScrollEvent, cx: &mut Context<Self>) -> bool {
        let (Some(bounds), Some(layout)) = (self.last_bounds, self.last_layout.as_ref()) else {
            return false;
        };
        let delta = event.delta.pixel_delta(layout.line_height());
        let max = (layout.height() - bounds.size.height).max(px(0.0));
        let before = self.scroll.y;
        self.scroll.y = (before - delta.y).clamp(px(0.0), max);
        let moved = self.scroll.y != before;
        let (active, claimed) = self.scroll_gesture;
        let (consume, next) = gesture_step(
            active,
            claimed,
            event.delta.precise(),
            event.touch_phase,
            moved,
        );
        self.scroll_gesture = next;
        if moved {
            cx.notify();
        }
        consume
    }

    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.caret_moved();
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn caret_for_mouse_position(&self, position: Point<Pixels>) -> Caret {
        let edge = |offset| Caret {
            offset,
            upstream: false,
        };
        if self.content.is_empty() {
            return edge(0);
        }
        // 変更のあと次の描画までは、配置の位置が今の文字列を指さないのでカーソルを動かさない。
        let (Some(bounds), Some(layout)) = (self.last_bounds.as_ref(), self.current_layout())
        else {
            return edge(self.cursor_offset());
        };
        if !self.multi_line {
            if position.y < bounds.top() {
                return edge(0);
            }
            if position.y > bounds.bottom() {
                return edge(self.content.len());
            }
        }
        // 複数行の欄の外は、最も近い表示行に収める（`caret_at`）。
        layout.caret_at(position - bounds.origin + self.scroll)
    }

    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.caret_moved();
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    /// 入力する文字を欄に合わせて直す。1 行の欄は改行を空白にし、改行だけの入力は `None`。
    fn input_text<'a>(&self, text: &'a str) -> Option<Cow<'a, str>> {
        let text = normalize_line_breaks(text);
        if self.multi_line || !text.contains('\n') {
            return Some(text);
        }
        (!text.chars().all(|c| c == '\n')).then(|| text.replace('\n', " ").into())
    }

    fn changed(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.caret_moved();
        window.notify_text_input_changed(&self.focus_handle);
        cx.emit(TextFieldEvent::Changed);
        cx.notify();
    }
}

impl EntityInputHandler for TextFieldState {
    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        // false にすると Windows の gpui が変換を終える（`ENDS_IME_WITH_HANDLER`）。
        self.ime_finish == ImeFinish::Idle
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        self.content.get(range).map(str::to_string)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.range_from_utf16(&range_utf16);
        self.selection_reversed = false;
        self.caret_moved();
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.content.encode_utf16().count())
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // `CPS_COMPLETE` の確定文字。この文字列は確定済みなので、入れると二重になる。
        // 同じフレームのキー入力も範囲なしで届くが、1 フレームだけなので区別しない。
        if self.ime_finish != ImeFinish::Idle && range_utf16.is_none() {
            return;
        }
        let Some(new_text) = self.input_text(new_text) else {
            return;
        };
        let new_text = new_text.as_ref();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let Some(range) = clamp_range(&self.content, range) else {
            return;
        };
        self.content.replace_range(range.clone(), new_text);
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        self.changed(window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // `CPS_COMPLETE` の確定文字。この文字列は確定済みなので、入れると二重になる。
        // 同じフレームのキー入力も範囲なしで届くが、1 フレームだけなので区別しない。
        if self.ime_finish != ImeFinish::Idle && range_utf16.is_none() {
            return;
        }
        let Some(new_text) = self.input_text(new_text) else {
            return;
        };
        let new_text = new_text.as_ref();
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let Some(range) = clamp_range(&self.content, range) else {
            return;
        };
        self.content.replace_range(range.clone(), new_text);
        self.marked_range =
            (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        let insertion_start_utf16 = self.offset_to_utf16(range.start);
        let inserted_length_utf16 = new_text.encode_utf16().count();
        self.selected_range = new_selected_range_utf16
            .map(|selected| {
                self.range_from_utf16(
                    &(insertion_start_utf16 + selected.start.min(inserted_length_utf16)
                        ..insertion_start_utf16 + selected.end.min(inserted_length_utf16)),
                )
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection_reversed = false;
        self.changed(window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        let start = self.ime_position_for_offset(range.start, bounds)?;
        let end = self.ime_position_for_offset(range.end, bounds)?;
        if !self.multi_line {
            return Some(Bounds::from_corners(
                point(start.x, bounds.top()),
                point(end.x, bounds.bottom()),
            ));
        }
        let line_height = self
            .last_layout
            .as_ref()
            .map_or(bounds.size.height, TextLayout::line_height);
        // 複数の表示行にまたがる範囲は開始行の分だけを返す（IME が欲しいのは候補を出す位置）。
        let right = if end.y == start.y {
            end.x
        } else {
            bounds.right()
        };
        Some(Bounds::from_corners(
            start,
            point(right, start.y + line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let caret = self.current_layout()?.caret_at(line_point + self.scroll);
        Some(self.offset_to_utf16(caret.offset))
    }
}

/// CRLF と CR を LF にそろえる。
fn normalize_line_breaks(text: &str) -> Cow<'_, str> {
    if text.contains('\r') {
        text.replace("\r\n", "\n").replace('\r', "\n").into()
    } else {
        text.into()
    }
}

/// 範囲を文字の境界に収める。境界でなければ `None`。
fn clamp_range(text: &str, range: Range<usize>) -> Option<Range<usize>> {
    let end = range.end.min(text.len());
    let start = range.start.min(end);
    (text.is_char_boundary(start) && text.is_char_boundary(end)).then_some(start..end)
}

/// 前の文字に付いて 1 つの文字（書記素）になる文字か。
///
/// 依存を増やさないため、Unicode の書記素クラスタの規則のうち、結合文字・異体字セレクタ・
/// 濁点の結合文字・ZWJ・肌の色の修飾子・タグ文字だけを扱う簡易版。
fn is_extending(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036F
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x200D
            | 0x20D0..=0x20FF
            | 0x3099..=0x309A
            | 0xFE00..=0xFE0F
            | 0xFE20..=0xFE2F
            | 0x1F3FB..=0x1F3FF
            | 0xE0020..=0xE007F
            | 0xE0100..=0xE01EF
    )
}

/// 地域指示子（2 つで国旗になる）か。
fn is_regional_indicator(c: char) -> bool {
    matches!(c as u32, 0x1F1E6..=0x1F1FF)
}

/// 書記素の先頭の位置（UTF-8 のバイト位置）。
fn boundaries(text: &str) -> impl Iterator<Item = usize> + '_ {
    let mut chars = text.char_indices().peekable();
    std::iter::from_fn(move || {
        let (start, first) = chars.next()?;
        let mut previous = first;
        let mut regional = usize::from(is_regional_indicator(first));
        while let Some(&(_, next)) = chars.peek() {
            let joins = previous != '\n'
                && (is_extending(next)
                    || previous == '\u{200D}'
                    || (regional == 1 && is_regional_indicator(next)));
            if !joins {
                break;
            }
            if is_regional_indicator(next) {
                regional += 1;
            }
            previous = next;
            chars.next();
        }
        Some(start)
    })
}

/// 支援技術に渡す文字の範囲。AccessKit は 1 文字の長さを `u8` で持つので、それを超える書記素は分ける。
fn characters(text: &str) -> Vec<Range<usize>> {
    let mut starts: Vec<usize> = boundaries(text).collect();
    starts.push(text.len());
    let mut characters = Vec::with_capacity(starts.len());
    for pair in starts.windows(2) {
        let (mut start, end) = (pair[0], pair[1]);
        while end - start > u8::MAX as usize {
            let mut split = start + u8::MAX as usize;
            while !text.is_char_boundary(split) {
                split -= 1;
            }
            characters.push(start..split);
            start = split;
        }
        characters.push(start..end);
    }
    characters
}

fn character_lengths(text: &str) -> Vec<u8> {
    characters(text)
        .into_iter()
        .map(|character| character.len() as u8)
        .collect()
}

/// 支援技術に渡す段落の範囲。末尾の改行を含む。文字列が改行で終わるときは最後に空の段落を置く。
fn paragraphs(text: &str) -> Vec<Range<usize>> {
    let mut paragraphs = Vec::new();
    let mut start = 0;
    for (index, _) in text.match_indices('\n') {
        paragraphs.push(start..index + 1);
        start = index + 1;
    }
    paragraphs.push(start..text.len());
    paragraphs
}

/// バイト位置を段落の番号と、段落の中の文字の番号にする。
fn character_of_offset(text: &str, offset: usize) -> (usize, usize) {
    let paragraphs = paragraphs(text);
    let line = paragraphs
        .partition_point(|paragraph| paragraph.end <= offset)
        .min(paragraphs.len() - 1);
    let paragraph = &paragraphs[line];
    let inner = offset - paragraph.start;
    let character =
        characters(&text[paragraph.clone()]).partition_point(|character| character.end <= inner);
    (line, character)
}

/// 段落の番号と文字の番号をバイト位置にする。範囲外はその段落か文字列の末尾に収める。
fn offset_of_character(text: &str, (line, character): (usize, usize)) -> usize {
    let paragraphs = paragraphs(text);
    let Some(paragraph) = paragraphs.get(line) else {
        return text.len();
    };
    characters(&text[paragraph.clone()])
        .get(character)
        .map_or(paragraph.end, |character| paragraph.start + character.start)
}

/// 選択範囲の始点と終点を段落の番号と文字の番号にする。
fn selection_characters(
    text: &str,
    selection: &Range<usize>,
    reversed: bool,
) -> ((usize, usize), (usize, usize)) {
    let (start, end) = (
        character_of_offset(text, selection.start),
        character_of_offset(text, selection.end),
    );
    if reversed { (end, start) } else { (start, end) }
}

fn previous_boundary(text: &str, offset: usize) -> usize {
    boundaries(text)
        .take_while(|&index| index < offset)
        .last()
        .unwrap_or(0)
}

fn next_boundary(text: &str, offset: usize) -> usize {
    boundaries(text)
        .find(|&index| index > offset)
        .unwrap_or(text.len())
}

/// 入力欄の文字（文字、placeholder、変換中の下線、選択、キャレット）を描く要素。
///
/// `line_limits` があれば欄の幅で折り返し、行数に合わせて高さを決める。なければ 1 行で、
/// はみ出した分は横に送る。
struct TextContent {
    pub state: Entity<TextFieldState>,
    pub placeholder: Option<SharedString>,
    pub paint: TextPaint,
    pub focused: bool,
    pub line_limits: Option<(usize, usize)>,
    pub reveal_margin: Pixels,
}

struct TextContentPrepaint {
    layout: Option<TextLayout>,
    origin: Point<Pixels>,
    caret: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
    scroll_y: Pixels,
}

/// 描く文字列。1 行の欄の改行は、バイト位置を変えないよう同じ長さの空白で描く。
fn display_text(
    content: &SharedString,
    placeholder: Option<&SharedString>,
    multi_line: bool,
) -> SharedString {
    let text = match placeholder {
        Some(placeholder) if content.is_empty() => placeholder,
        _ => content,
    };
    if !multi_line && text.contains('\n') {
        text.replace('\n', " ").into()
    } else {
        text.clone()
    }
}

/// 折り返す幅。行の末尾のキャレットが欄に収まるよう、その幅を引く。
fn wrap_width(width: Pixels, caret_width: Pixels) -> Pixels {
    (width - caret_width).max(px(1.0))
}

/// 欄の左端から測ったキャレットの位置。行末の空白が幅を超えても欄の中に収める。
fn caret_left(caret_x: Pixels, scroll_x: Pixels, width: Pixels, caret_width: Pixels) -> Pixels {
    (caret_x - scroll_x).min((width - caret_width).max(px(0.0)))
}

/// 折り返したときの表示行の数。
fn measure_rows(
    text: SharedString,
    font: Font,
    font_size: Pixels,
    width: Option<Pixels>,
    window: &mut Window,
) -> usize {
    let run = TextRun {
        len: text.len(),
        font,
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window
        .text_system()
        .shape_text(text, font_size, &[run], width, None)
        .map_or(1, |lines| {
            lines
                .iter()
                .map(|line| line.wrap_boundaries().len() + 1)
                .sum()
        })
}

impl IntoElement for TextContent {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextContent {
    type RequestLayoutState = ();
    type PrepaintState = TextContentPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        let line_height = window.line_height();
        let Some((min_lines, max_lines)) = self.line_limits else {
            style.size.height = line_height.into();
            return (window.request_layout(style, [], cx), ());
        };
        let content: SharedString = self.state.read(cx).content.clone().into();
        let text = display_text(&content, self.placeholder.as_ref(), true);
        let text_style = window.text_style();
        let font = text_style.font();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let caret_width = self.paint.caret_width;
        let layout_id =
            window.request_measured_layout(style, move |known, available, window, _| {
                let width = known.width.or(match available.width {
                    AvailableSpace::Definite(width) => Some(width),
                    _ => None,
                });
                let rows = measure_rows(
                    text.clone(),
                    font.clone(),
                    font_size,
                    width.map(|width| wrap_width(width, caret_width)),
                    window,
                );
                size(
                    width.unwrap_or(px(0.0)),
                    line_height * rows.clamp(min_lines, max_lines),
                )
            });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let multi_line = self.line_limits.is_some();
        let input = self.state.read(cx);
        let content: SharedString = input.content.clone().into();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let upstream = input.caret_upstream;
        let marked_range = input.marked_range.clone();
        let mut scroll = input.scroll;
        let reveal = input.reveal_caret;
        let style = window.text_style();

        let display_text = display_text(&content, self.placeholder.as_ref(), multi_line);
        let text_color = if content.is_empty() && self.placeholder.is_some() {
            self.paint.placeholder
        } else {
            self.paint.text
        };
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = match marked_range.filter(|_| !content.is_empty()) {
            Some(marked) => vec![
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect(),
            None => vec![run],
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let width = multi_line.then(|| wrap_width(bounds.size.width, self.paint.caret_width));
        // placeholder の配置は入力の文字列の位置に使えないので、`is_for` が通らないようにする。
        let source = if content.is_empty() {
            display_text.clone()
        } else {
            content.clone()
        };
        let lines = window
            .text_system()
            .shape_text(display_text, font_size, &runs, width, None);
        let Ok(lines) = lines else {
            return TextContentPrepaint {
                layout: None,
                origin: bounds.origin,
                caret: None,
                selection: Vec::new(),
                scroll_y: px(0.0),
            };
        };
        let layout = TextLayout::new(source, lines, line_height);

        let caret = if content.is_empty() {
            point(px(0.0), px(0.0))
        } else {
            layout.position(cursor, upstream)
        };
        if multi_line {
            // カーソルが見える位置まで縦に送る。
            if reveal {
                if caret.y < scroll.y {
                    scroll.y = caret.y;
                }
                if caret.y + line_height - scroll.y > bounds.size.height {
                    scroll.y = caret.y + line_height - bounds.size.height;
                }
            }
            scroll.y = scroll
                .y
                .clamp(px(0.0), (layout.height() - bounds.size.height).max(px(0.0)));
        } else {
            // カーソルが見える位置まで横に送る。
            let visible = (bounds.size.width - self.paint.caret_width).max(px(0.0));
            if caret.x - scroll.x > visible {
                scroll.x = caret.x - visible;
            }
            if caret.x < scroll.x {
                scroll.x = caret.x;
            }
            let max_scroll = if content.is_empty() {
                px(0.0)
            } else {
                (layout.width() + self.paint.caret_width - bounds.size.width).max(px(0.0))
            };
            scroll.x = scroll.x.min(max_scroll).max(px(0.0));
        }
        self.state.update(cx, |state, _| {
            state.scroll = scroll;
            state.reveal_caret = false;
        });
        let origin = bounds.origin - scroll;
        let caret_bounds = Bounds::new(
            point(
                bounds.left()
                    + caret_left(caret.x, scroll.x, bounds.size.width, self.paint.caret_width),
                origin.y + caret.y,
            ),
            size(self.paint.caret_width, line_height),
        );
        // 入力で欄が伸びると、カーソルが祖先のスクロール領域の外へ出ることがある。
        if multi_line && reveal && self.focused && window.last_input_was_keyboard() {
            reveal_in_scopes(caret_bounds, self.reveal_margin, window, cx);
        }

        let (selection, caret) = if selected_range.is_empty() || content.is_empty() {
            (Vec::new(), Some(fill(caret_bounds, self.paint.caret)))
        } else {
            let selection = layout
                .selection_spans(&selected_range, line_height * LINE_BREAK_SELECTION_WIDTH)
                .into_iter()
                .map(|(row, left, right)| {
                    let top = origin.y + line_height * row;
                    fill(
                        Bounds::from_corners(
                            point(origin.x + left, top),
                            point(origin.x + right, top + line_height),
                        ),
                        self.paint.selection,
                    )
                })
                .collect();
            (selection, None)
        };
        TextContentPrepaint {
            layout: Some(layout),
            origin,
            caret,
            selection,
            scroll_y: scroll.y,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.state.read(cx).focus_handle.clone();
        if self.focused {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }
        let Some(layout) = prepaint.layout.take() else {
            return;
        };
        let origin = prepaint.origin;
        let scroll_y = prepaint.scroll_y;
        let selection = std::mem::take(&mut prepaint.selection);
        let caret = prepaint.caret.take().filter(|_| self.focused);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for quad in selection {
                window.paint_quad(quad)
            }
            layout.paint(origin, scroll_y..scroll_y + bounds.size.height, window, cx);
            if let Some(caret) = caret {
                window.paint_quad(caret);
            }
        });
        self.state.update(cx, |input, cx| {
            input.last_layout = Some(layout);
            input.last_bounds = Some(bounds);
            if self.focused {
                if ENDS_IME_WITH_HANDLER {
                    // 欄の外のクリックでフォーカスが残ったときだけここへ来る。移ったときは blur が
                    // 印を消して移動先へ引き継ぐ。
                    if input.finish_after_click_out {
                        input.finish_after_click_out = false;
                        if input.marked_range.take().is_some() {
                            if input.ime_finish == ImeFinish::Idle {
                                input.ime_finish = ImeFinish::Requested;
                            }
                            cx.notify();
                        }
                    }
                    let pending = cx.default_global::<PendingImeFinish>();
                    if pending.window == Some(window.window_handle().window_id()) {
                        pending.window = None;
                        if input.ime_finish == ImeFinish::Idle {
                            input.ime_finish = ImeFinish::Requested;
                        }
                    }
                    // このフレームの最後に gpui が false を読むので、解除は次のフレームまで待つ。
                    if input.ime_finish == ImeFinish::Requested {
                        input.ime_finish = ImeFinish::Scheduled;
                        let state = cx.weak_entity();
                        window.on_next_frame(move |_, cx| {
                            let _ = state.update(cx, |state, cx| {
                                state.ime_finish = ImeFinish::Idle;
                                cx.notify();
                            });
                        });
                    }
                }
                let offset = input
                    .marked_range
                    .as_ref()
                    .map_or_else(|| input.cursor_offset(), |range| range.start);
                let anchor = input
                    .ime_position_for_offset(offset, bounds)
                    .map(|position| ImeAnchor { bounds, position });
                if anchor.is_some() && input.last_ime_anchor != anchor {
                    input.last_ime_anchor = anchor;
                    window.invalidate_character_coordinates();
                }
            } else {
                input.last_ime_anchor = None;
            }
        });
    }
}

/// 書記素の数。
pub fn grapheme_count(text: &str) -> usize {
    boundaries(text).count()
}

/// テキスト入力欄。
pub struct TextField {
    id: ElementId,
    state: Entity<TextFieldState>,
    name: crate::AccessibleName,
    placeholder: Option<SharedString>,
    error: bool,
    required: bool,
    description: Option<SharedString>,
    disabled: bool,
    max_length: Option<usize>,
    reveal_margin: Pixels,
    role: Role,
    line_limits: Option<(usize, usize)>,
}

impl TextField {
    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// 入力の補足説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// ID と中身から作る。
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextFieldState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            name: crate::AccessibleName::default(),
            placeholder: None,
            error: false,
            required: false,
            description: None,
            disabled: false,
            max_length: None,
            reveal_margin: px(0.0),
            role: Role::TextInput,
            line_limits: None,
        }
    }
    /// 表示するラベルを既定の名前にする。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.name.set_visible(label);
        self
    }

    accessible_name!();

    /// プレースホルダーを設定する。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    /// エラー状態を設定する。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    /// 無効状態を設定する。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// 文字数の上限を設定する。入力は制限しない。
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
    /// 複数行の入力欄にする。文字は欄の幅で折り返し、Enter で改行する。
    ///
    /// 高さは `min_lines` 行から `max_lines` 行の間で内容に合わせて変わり、`max_lines` を超えた分は
    /// 欄の中でスクロールする。上限なしは `usize::MAX`。`min_lines` は 1 以上に、`max_lines` は
    /// `min_lines` 以上に直す。
    pub fn multi_line(mut self, min_lines: usize, max_lines: usize) -> Self {
        let min_lines = min_lines.max(1);
        self.line_limits = Some((min_lines, max_lines.max(min_lines)));
        self.role = Role::MultilineTextInput;
        self
    }
    /// フォーカス追従に必要な余白を設定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(self.role)
            .with_name(&self.name)
            .disabled(self.disabled)
            .invalid(self.error)
            .required(self.required);
        if let Some(description) = &self.description {
            node = node.description(description.clone());
        }
        node
    }
    /// 入力欄の土台と、描画用の状態を作る。
    pub fn build(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, TextFieldControl) {
        let multi_line = self.line_limits.is_some();
        self.state.update(cx, |state, cx| {
            state.multi_line = multi_line;
            state.observe_blur(window, cx)
        });
        let hovered = window.use_keyed_state(
            ElementId::NamedChild(self.id.clone().into(), "hovered".into()),
            cx,
            |_, _| false,
        );
        let state = self.state.read(cx);
        let text = state.text().to_owned();
        let selection = state.selected_range();
        let reversed = state.selection_reversed;
        let handle = state.focus_handle.clone();
        let focused = !self.disabled && handle.is_focused(window);
        let focus_visible = !self.disabled && crate::is_focus_visible(&handle, window, cx);
        let run_nodes = TextRunNodes::default();
        let mut base = div()
            .id(self.id.clone())
            .a11y_with_children(self.a11y_node(), {
                let text = text.clone();
                let run_nodes = run_nodes.clone();
                move |builder| write_text_runs(builder, text, selection, reversed, &run_nodes)
            })
            .aria_value(SharedString::from(text.clone()))
            .when_some(self.placeholder.clone(), |this, placeholder| {
                this.aria_placeholder(placeholder)
            });
        if !self.disabled {
            base = a11y_actions(base, &self.state, run_nodes);
            base = editable(
                base,
                &self.id,
                &self.state,
                Some(hovered.clone()),
                window,
                cx,
                self.reveal_margin,
            );
        }
        let control = TextFieldControl {
            state: self.state,
            line_limits: self.line_limits,
            reveal_margin: self.reveal_margin,
            max_length: self.max_length,
            text,
            focused,
            focus_visible,
            hovered: !self.disabled && *hovered.read(cx),
        };
        (base, control)
    }
}

/// 入力欄の描画用状態。
pub struct TextFieldControl {
    state: Entity<TextFieldState>,
    line_limits: Option<(usize, usize)>,
    reveal_margin: Pixels,
    /// 設定された文字数の上限（入力は制限しない）。
    pub max_length: Option<usize>,
    /// 現在の文字列。
    pub text: String,
    /// フォーカスされているか。
    pub focused: bool,
    /// キーボードフォーカスを表示するか。
    pub focus_visible: bool,
    /// ポインタが乗っているか。
    pub hovered: bool,
}
impl TextFieldControl {
    /// 指定した色とプレースホルダーで入力行を作る。
    pub fn input(
        &self,
        paint: TextPaint,
        placeholder: Option<SharedString>,
    ) -> impl IntoElement + use<> {
        TextContent {
            state: self.state.clone(),
            placeholder,
            paint,
            focused: self.focused,
            line_limits: self.line_limits,
            reveal_margin: self.reveal_margin,
        }
    }
}

/// 描いた TextRun のノード。段落の順に並ぶ。SetTextSelection の位置から段落を引くために
/// 支援技術の操作のハンドラーと共有する（gpui にノードの逆引きはない）。
type TextRunNodes = Rc<RefCell<Vec<accesskit::NodeId>>>;

/// 文字とカーソルを支援技術に渡す（gpui の `_accessibility` の例と同じ形）。
///
/// AccessKit は TextRun を 1 行として扱い、値の末尾の改行で段落の終わりを判定する。段落ごとに
/// 1 つ作り、支援技術が行単位で移動できるようにする。
fn write_text_runs(
    builder: &mut A11ySubtreeBuilder,
    text: String,
    selection: Range<usize>,
    reversed: bool,
    run_nodes: &TextRunNodes,
) {
    let (anchor, focus) = selection_characters(&text, &selection, reversed);
    let mut nodes = Vec::new();
    for (index, paragraph) in paragraphs(&text).into_iter().enumerate() {
        let value = &text[paragraph];
        let mut run = accesskit::Node::new(Role::TextRun);
        run.set_character_lengths(character_lengths(value));
        run.set_value(value);
        let node = builder.synthetic_node_id(index);
        builder.push_child(node, run);
        nodes.push(node);
    }
    let position = |(line, character_index): (usize, usize)| accesskit::TextPosition {
        node: nodes[line],
        character_index,
    };
    builder
        .parent_node()
        .set_text_selection(accesskit::TextSelection {
            anchor: position(anchor),
            focus: position(focus),
        });
    *run_nodes.borrow_mut() = nodes;
}

/// 支援技術の位置を段落の番号と文字の番号にする。この欄の TextRun でなければ `None`。
fn a11y_position(
    nodes: &[accesskit::NodeId],
    position: &accesskit::TextPosition,
) -> Option<(usize, usize)> {
    let line = nodes.iter().position(|node| *node == position.node)?;
    Some((line, position.character_index))
}

/// 支援技術からの編集を受ける（WCAG 4.1.2。Compose の `setText` など）。
fn a11y_actions(
    field: Stateful<Div>,
    state: &Entity<TextFieldState>,
    run_nodes: TextRunNodes,
) -> Stateful<Div> {
    field
        .on_a11y_action(accesskit::Action::SetValue, {
            let state = state.clone();
            move |data, window, cx| {
                if let Some(accesskit::ActionData::Value(text)) = data {
                    state.update(cx, |state, cx| state.a11y_set_value(text, window, cx));
                }
            }
        })
        .on_a11y_action(accesskit::Action::ReplaceSelectedText, {
            let state = state.clone();
            move |data, window, cx| {
                if let Some(accesskit::ActionData::Value(text)) = data {
                    state.update(cx, |state, cx| {
                        state.a11y_replace_selection(text, window, cx)
                    });
                }
            }
        })
        .on_a11y_action(accesskit::Action::SetTextSelection, {
            let state = state.clone();
            move |data, window, cx| {
                if let Some(accesskit::ActionData::SetTextSelection(selection)) = data {
                    let nodes = run_nodes.borrow();
                    let (Some(anchor), Some(focus)) = (
                        a11y_position(&nodes, &selection.anchor),
                        a11y_position(&nodes, &selection.focus),
                    ) else {
                        return;
                    };
                    state.update(cx, |state, cx| {
                        state.a11y_set_selection(anchor, focus, window, cx)
                    });
                }
            }
        })
}

/// 検索用の入力欄。
pub struct SearchField(TextField);
impl SearchField {
    /// ID と中身から作る。
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextFieldState>) -> Self {
        let mut field = TextField::new(id, state);
        field.role = Role::SearchInput;
        Self(field)
    }
    delegate_accessible_name!(0);
    /// プレースホルダーを設定する。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.0 = self.0.placeholder(placeholder);
        self
    }
    /// フォーカス追従に必要な余白を設定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.0 = self.0.reveal_margin(margin);
        self
    }
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        self.0.a11y_node()
    }
    /// 入力欄の土台と描画用の状態を作る。
    pub fn build(self, window: &mut Window, cx: &mut App) -> (Stateful<Div>, TextFieldControl) {
        self.0.build(window, cx)
    }
}

/// 要素を文字の入力欄にする（キーコンテキスト、フォーカス、編集のアクション、マウスでの選択）。
/// `hovered` を渡すと、ポインタが乗っているかを書き込む。
fn editable(
    field: Stateful<Div>,
    id: &ElementId,
    state: &Entity<TextFieldState>,
    hovered: Option<Entity<bool>>,
    window: &mut Window,
    cx: &mut App,
    reveal_margin: Pixels,
) -> Stateful<Div> {
    let focus_handle = state.read(cx).focus_handle.clone();
    let multi_line = state.read(cx).multi_line;
    let state = state.clone();
    let mut key_context = KeyContext::default();
    key_context.add(TEXT_FIELD_CONTEXT);
    if multi_line {
        key_context.add(MULTI_LINE_FIELD_CONTEXT);
    }
    field
        .relative()
        .child(crate::reveal_on_focus(
            id,
            focus_handle.clone(),
            window,
            cx,
            reveal_margin,
        ))
        .key_context(key_context)
        .track_focus(&focus_handle)
        .cursor(CursorStyle::IBeam)
        .on_hover(move |hover, _, cx| {
            if let Some(hovered) = &hovered {
                hovered.update(cx, |value, cx| {
                    *value = *hover;
                    cx.notify();
                })
            }
        })
        .on_action(action(&state, |s, w, cx| s.backspace(w, cx), Backspace))
        .on_action(action(&state, |s, w, cx| s.delete(w, cx), Delete))
        .on_action(action(&state, |s, w, cx| s.left(w, cx), Left))
        .on_action(action(&state, |s, w, cx| s.right(w, cx), Right))
        .on_action(action(&state, |s, w, cx| s.select_left(w, cx), SelectLeft))
        .on_action(action(
            &state,
            |s, w, cx| s.select_right(w, cx),
            SelectRight,
        ))
        .on_action(action(&state, |s, w, cx| s.select_all(w, cx), SelectAll))
        .on_action(action(&state, |s, w, cx| s.home(w, cx), Home))
        .on_action(action(&state, |s, w, cx| s.end(w, cx), End))
        .on_action(action(&state, |s, w, cx| s.select_home(w, cx), SelectHome))
        .on_action(action(&state, |s, w, cx| s.select_end(w, cx), SelectEnd))
        .on_action(action(&state, |s, w, cx| s.paste(w, cx), Paste))
        .on_action(action(&state, |s, w, cx| s.cut(w, cx), Cut))
        .on_action(action(&state, |s, _, cx| s.copy(cx), Copy))
        .on_action(action(&state, |s, w, cx| s.newline(w, cx), Newline))
        .on_action(action(&state, |s, w, cx| s.move_rows(-1, w, cx), Up))
        .on_action(action(&state, |s, w, cx| s.move_rows(1, w, cx), Down))
        .on_action(action(
            &state,
            |s, w, cx| s.select_rows(-1, w, cx),
            SelectUp,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.select_rows(1, w, cx),
            SelectDown,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.move_rows(-s.visible_rows(), w, cx),
            PageUp,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.move_rows(s.visible_rows(), w, cx),
            PageDown,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.select_rows(-s.visible_rows(), w, cx),
            SelectPageUp,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.select_rows(s.visible_rows(), w, cx),
            SelectPageDown,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.document_start(w, cx),
            DocumentStart,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.document_end(w, cx),
            DocumentEnd,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.select_document_start(w, cx),
            SelectDocumentStart,
        ))
        .on_action(action(
            &state,
            |s, w, cx| s.select_document_end(w, cx),
            SelectDocumentEnd,
        ))
        .when(multi_line, |field| {
            let state = state.clone();
            field.on_scroll(move |event, _, cx| {
                if state.update(cx, |state, cx| state.scroll_by_wheel(event, cx)) {
                    cx.stop_propagation();
                }
            })
        })
        .on_mouse_down_out({
            let state = state.clone();
            move |_, _, cx| {
                // 空白のクリックではフォーカスが動かず blur が来ない。ここはフォーカスの移動より
                // 先に走るので `marked_range` には触らない。消すと、blur が変換中だったことを
                // 知れず移動先へ引き継げない。印だけ付け、次の描画で判断する。
                state.update(cx, |state, cx| {
                    if ENDS_IME_WITH_HANDLER && state.marked_range.is_some() {
                        state.finish_after_click_out = true;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_down(MouseButton::Left, {
            let state = state.clone();
            let focus_handle = focus_handle.clone();
            move |event, window, cx| {
                window.focus(&focus_handle, cx);
                state.update(cx, |state, cx| state.on_mouse_down(event, window, cx));
            }
        })
        .on_mouse_up(MouseButton::Left, {
            let state = state.clone();
            move |_, _, cx| state.update(cx, |state, _| state.is_selecting = false)
        })
        .on_mouse_up_out(MouseButton::Left, {
            let state = state.clone();
            move |_, _, cx| state.update(cx, |state, _| state.is_selecting = false)
        })
        .on_mouse_move(move |event, window, cx| {
            state.update(cx, |state, cx| state.on_mouse_move(event, window, cx))
        })
}

/// アクションを中身の操作につなぐ。
fn action<A: gpui::Action>(
    state: &Entity<TextFieldState>,
    f: impl Fn(&mut TextFieldState, &mut Window, &mut Context<TextFieldState>) + 'static,
    _: A,
) -> impl Fn(&A, &mut Window, &mut App) + 'static {
    let state = state.clone();
    move |_, window, cx| state.update(cx, |state, cx| f(state, window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_stays_inside_the_scrolled_field() {
        assert_eq!(
            caret_left(px(430.0), px(380.0), px(102.0), px(2.0)),
            px(50.0),
            "横に送った分を引いて測る"
        );
        assert_eq!(
            caret_left(px(110.0), px(0.0), px(102.0), px(2.0)),
            px(100.0),
            "行末の空白が幅を超えても欄の中に収める"
        );
    }

    #[test]
    fn line_breaks_become_lf() {
        assert_eq!(normalize_line_breaks("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn a_mark_after_a_line_break_starts_its_own_character() {
        assert_eq!(next_boundary("\n\u{301}", 0), 1);
    }

    #[test]
    fn paragraphs_keep_their_line_break() {
        assert_eq!(paragraphs("ab\nc"), [0..3, 3..4]);
        assert_eq!(
            paragraphs("ab\n"),
            [0..3, 3..3],
            "末尾の改行のあとに空の段落を置く"
        );
        assert_eq!(paragraphs(""), vec![0..0]);
    }

    #[test]
    fn offsets_map_to_paragraphs_and_back() {
        let text = "aあ\n\nb";
        for (offset, position) in [
            (1, (0, 1)),
            (4, (0, 2)),
            (5, (1, 0)),
            (6, (2, 0)),
            (7, (2, 1)),
        ] {
            assert_eq!(character_of_offset(text, offset), position);
            assert_eq!(offset_of_character(text, position), offset);
        }
        assert_eq!(
            character_of_offset("a\n", 2),
            (1, 0),
            "末尾は空の段落の先頭"
        );
        assert_eq!(
            offset_of_character(text, (9, 0)),
            text.len(),
            "範囲外は末尾に収める"
        );
        assert_eq!(
            offset_of_character(text, (0, 9)),
            5,
            "段落の中の範囲外は段落の末尾に収める"
        );
    }

    #[test]
    fn positions_outside_the_field_are_ignored() {
        let nodes = [accesskit::NodeId(1), accesskit::NodeId(2)];
        let position = |node| accesskit::TextPosition {
            node: accesskit::NodeId(node),
            character_index: 3,
        };
        assert_eq!(a11y_position(&nodes, &position(2)), Some((1, 3)));
        assert_eq!(
            a11y_position(&nodes, &position(9)),
            None,
            "他のノードの位置は受けない"
        );
    }

    #[test]
    fn combining_marks_stay_with_their_base() {
        // が = か + 濁点の結合文字
        let text = "か\u{3099}き";
        assert_eq!(next_boundary(text, 0), "か\u{3099}".len());
        assert_eq!(previous_boundary(text, text.len()), "か\u{3099}".len());
        assert_eq!(grapheme_count(text), 2);
    }

    #[test]
    fn emoji_sequences_are_one_character() {
        let family = "👨\u{200D}👩\u{200D}👧";
        assert_eq!(next_boundary(family, 0), family.len());
        let flags = "🇯🇵🇺🇸";
        assert_eq!(next_boundary(flags, 0), "🇯🇵".len());
        assert_eq!(grapheme_count(flags), 2);
    }

    #[test]
    fn boundaries_at_the_ends() {
        assert_eq!(previous_boundary("abc", 0), 0);
        assert_eq!(next_boundary("abc", 3), 3);
        assert_eq!(grapheme_count(""), 0);
    }
}

#[cfg(test)]
mod ime_focus_tests {
    use super::*;
    use gpui::{
        AnyView, AppContext, InputEvent, Modifiers, Render, StyleRefinement, TestAppContext,
        VisualTestContext, black,
    };

    struct CachedField {
        id: &'static str,
        state: Entity<TextFieldState>,
    }

    impl Render for CachedField {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let color = black();
            let (base, control) = TextField::new(self.id, &self.state)
                .label("入力")
                .build(window, cx);
            base.size_full().child(control.input(
                TextPaint {
                    text: color,
                    placeholder: color,
                    caret: color,
                    selection: color,
                    caret_width: px(2.0),
                },
                None,
            ))
        }
    }

    struct CachedHarness {
        a: Entity<CachedField>,
        b: Entity<CachedField>,
    }

    impl Render for CachedHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let style = StyleRefinement::default().w(px(300.0)).h(px(40.0));
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(AnyView::from(self.a.clone()).cached(style.clone()))
                .child(AnyView::from(self.b.clone()).cached(style))
        }
    }

    struct Harness {
        a: Entity<TextFieldState>,
        b: Entity<TextFieldState>,
        other: FocusHandle,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let color = black();
            let field = |id,
                         state: &Entity<TextFieldState>,
                         window: &mut Window,
                         cx: &mut Context<Self>| {
                let (base, control) = TextField::new(id, state).label("入力").build(window, cx);
                base.w(px(300.0)).h(px(40.0)).child(control.input(
                    TextPaint {
                        text: color,
                        placeholder: color,
                        caret: color,
                        selection: color,
                        caret_width: px(2.0),
                    },
                    None,
                ))
            };
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(field("a", &self.a, window, cx))
                .child(field("b", &self.b, window, cx))
                .child(div().id("other").h(px(40.0)).track_focus(&self.other))
        }
    }

    fn open(
        cx: &mut TestAppContext,
    ) -> (
        Entity<TextFieldState>,
        Entity<TextFieldState>,
        FocusHandle,
        &mut VisualTestContext,
    ) {
        let a = cx.new(TextFieldState::new);
        let b = cx.new(TextFieldState::new);
        let other = cx.update(|cx| cx.focus_handle());
        let (_, cx) = cx.add_window_view({
            let a = a.clone();
            let b = b.clone();
            let other = other.clone();
            move |_, _| Harness { a, b, other }
        });
        cx.simulate_resize(size(px(400.0), px(300.0)));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        draw(cx);
        next_frame(cx);
        focus(cx, &a);
        draw(cx);
        next_frame(cx);
        (a, b, other, cx)
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    fn next_frame(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
    }

    fn focus(cx: &mut VisualTestContext, state: &Entity<TextFieldState>) {
        cx.update(|window, cx| {
            let handle = state.read(cx).focus_handle(cx);
            window.focus(&handle, cx);
            assert!(handle.is_focused(window));
        });
    }

    fn mark(cx: &mut VisualTestContext, state: &Entity<TextFieldState>) {
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.replace_and_mark_text_in_range(None, "にほん", None, window, cx);
            })
        });
        draw(cx);
        next_frame(cx);
        assert!(state.read_with(cx, |state, _| state.marked_range().is_some()));
    }

    fn accepts(cx: &mut VisualTestContext, state: &Entity<TextFieldState>) -> bool {
        cx.update(|window, cx| state.update(cx, |state, cx| state.accepts_text_input(window, cx)))
    }

    fn insert(cx: &mut VisualTestContext, state: &Entity<TextFieldState>, text: &str) {
        cx.update(|window, cx| {
            state.update(cx, |state, cx| {
                state.replace_text_in_range(None, text, window, cx);
            })
        });
    }

    #[gpui::test]
    fn blur_confirms_marked_text(cx: &mut TestAppContext) {
        let (a, _, other, cx) = open(cx);
        mark(cx, &a);
        cx.update(|window, cx| {
            window.focus(&other, cx);
            assert!(other.is_focused(window));
        });
        draw(cx);
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "にほん");
            assert_eq!(state.marked_range(), None);
        });
    }

    #[gpui::test]
    fn window_deactivation_keeps_marked_text(cx: &mut TestAppContext) {
        let (a, _, _, cx) = open(cx);
        mark(cx, &a);
        assert!(cx.update(|window, _| window.is_window_active()));
        cx.deactivate_window();
        assert!(!cx.update(|window, _| window.is_window_active()));
        draw(cx);
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "にほん");
            assert_eq!(state.marked_range(), Some(0.."にほん".len()));
        });
    }

    #[gpui::test]
    fn focus_move_to_another_field_finishes_composition(cx: &mut TestAppContext) {
        let (a, b, _, cx) = open(cx);
        mark(cx, &a);
        focus(cx, &b);
        draw(cx);
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "にほん");
            assert_eq!(state.marked_range(), None);
        });
        assert!(!accepts(cx, &b), "変換終了のフレームでは入力を受け付けない");
        // Windows の CPS_COMPLETE による確定文字は移動先に挿入しない。
        insert(cx, &b, "にほん");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "");
        next_frame(cx);
        draw(cx);
        assert!(accepts(cx, &b));
        insert(cx, &b, "a");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "a");
    }

    #[gpui::test]
    fn focus_move_reaches_field_in_cached_view(cx: &mut TestAppContext) {
        let a = cx.new(TextFieldState::new);
        let b = cx.new(TextFieldState::new);
        let view_a = cx.new(|_| CachedField {
            id: "a",
            state: a.clone(),
        });
        let view_b = cx.new(|_| CachedField {
            id: "b",
            state: b.clone(),
        });
        let (_, cx) = cx.add_window_view(move |_, _| CachedHarness {
            a: view_a,
            b: view_b,
        });
        cx.simulate_resize(size(px(400.0), px(300.0)));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let draw_without_refresh = |cx: &mut VisualTestContext| {
            cx.update(|window, cx| window.draw(cx).clear(cx));
        };
        draw_without_refresh(cx);
        next_frame(cx);
        focus(cx, &a);
        draw_without_refresh(cx);
        next_frame(cx);
        cx.update(|window, cx| {
            a.update(cx, |state, cx| {
                state.replace_and_mark_text_in_range(None, "にほん", None, window, cx);
            });
        });
        draw_without_refresh(cx);
        next_frame(cx);
        assert!(a.read_with(cx, |state, _| state.marked_range().is_some()));

        focus(cx, &b);
        draw_without_refresh(cx);
        assert_eq!(a.read_with(cx, |state, _| state.marked_range()), None);
        next_frame(cx);
        draw_without_refresh(cx);
        assert!(!accepts(cx, &b), "移動先で変換終了のフレームを描く");
        insert(cx, &b, "にほん");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "");
        next_frame(cx);
        draw_without_refresh(cx);
        assert!(accepts(cx, &b));
        insert(cx, &b, "a");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "a");
    }

    #[gpui::test]
    fn click_in_same_field_finishes_composition(cx: &mut TestAppContext) {
        let (a, _, _, cx) = open(cx);
        insert(cx, &a, "ab");
        mark(cx, &a);
        cx.simulate_mouse_down(
            point(px(1.0), px(10.0)),
            MouseButton::Left,
            Modifiers::none(),
        );
        draw(cx);
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "abにほん");
            assert_eq!(state.selected_range(), 0..0, "クリックで先頭に移動する");
            assert_eq!(state.marked_range(), None);
        });
        assert!(!accepts(cx, &a));
        insert(cx, &a, "にほん");
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "abにほん");
            assert_eq!(state.selected_range(), 0..0);
        });
        next_frame(cx);
        draw(cx);
        assert!(accepts(cx, &a));
        insert(cx, &a, "a");
        assert_eq!(
            a.read_with(cx, |state, _| state.text().to_owned()),
            "aabにほん"
        );
    }

    #[gpui::test]
    fn click_outside_field_finishes_composition(cx: &mut TestAppContext) {
        let (a, _, _, cx) = open(cx);
        mark(cx, &a);
        cx.simulate_mouse_down(
            point(px(10.0), px(200.0)),
            MouseButton::Left,
            Modifiers::none(),
        );
        draw(cx);
        cx.update(|window, cx| assert!(a.read(cx).focus_handle.is_focused(window)));
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "にほん");
            assert_eq!(state.marked_range(), None);
        });
        assert!(!accepts(cx, &a));
        insert(cx, &a, "にほん");
        assert_eq!(
            a.read_with(cx, |state, _| state.text().to_owned()),
            "にほん"
        );
        next_frame(cx);
        draw(cx);
        assert!(accepts(cx, &a));
        insert(cx, &a, "a");
        assert_eq!(
            a.read_with(cx, |state, _| state.text().to_owned()),
            "にほんa"
        );
    }

    #[gpui::test]
    fn click_on_other_field_hands_over_finish(cx: &mut TestAppContext) {
        let (a, b, _, cx) = open(cx);
        mark(cx, &a);
        cx.simulate_mouse_down(
            point(px(10.0), px(50.0)),
            MouseButton::Left,
            Modifiers::none(),
        );
        draw(cx);
        cx.update(|window, cx| assert!(b.read(cx).focus_handle.is_focused(window)));
        a.read_with(cx, |state, _| {
            assert_eq!(state.text(), "にほん");
            assert_eq!(state.marked_range(), None);
        });
        assert!(accepts(cx, &a));
        assert!(!accepts(cx, &b));
        insert(cx, &b, "にほん");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "");
        next_frame(cx);
        draw(cx);
        assert!(accepts(cx, &b));
        insert(cx, &b, "a");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "a");
    }

    #[gpui::test]
    fn click_then_focus_move_before_paint_hands_over(cx: &mut TestAppContext) {
        let (a, b, _, cx) = open(cx);
        mark(cx, &a);
        cx.update(|window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    position: point(px(10.0), px(10.0)),
                    button: MouseButton::Left,
                    modifiers: Modifiers::none(),
                    click_count: 1,
                    first_mouse: false,
                }
                .to_platform_input(),
                cx,
            );
            assert!(a.read(cx).ime_finish == ImeFinish::Requested);
            let handle = b.read(cx).focus_handle.clone();
            window.focus(&handle, cx);
        });
        draw(cx);
        assert!(!accepts(cx, &b));
        insert(cx, &b, "にほん");
        assert_eq!(b.read_with(cx, |state, _| state.text().to_owned()), "");
        next_frame(cx);
        draw(cx);
        assert!(accepts(cx, &a));
        assert!(accepts(cx, &b));
    }

    #[gpui::test]
    fn pending_finish_stays_in_its_window(cx: &mut TestAppContext) {
        let (a, b, _, first) = open(cx);
        let mut app = first.cx.clone();
        let c = app.new(TextFieldState::new);
        let d = app.new(TextFieldState::new);
        let other = app.update(|cx| cx.focus_handle());
        let (_, second) = app.add_window_view({
            let c = c.clone();
            move |_, _| Harness { a: c, b: d, other }
        });
        second.simulate_resize(size(px(400.0), px(300.0)));
        second.update(|window, _| window.activate_window());
        draw(second);
        focus(second, &c);
        draw(second);
        next_frame(second);
        mark(first, &a);
        focus(first, &b);
        draw(second);
        assert!(accepts(second, &c), "別のウィンドウの入力を止めない");
        draw(first);
        assert!(!accepts(first, &b), "移動先のウィンドウで変換を終了する");
    }

    #[gpui::test]
    fn plain_focus_move_does_not_reset_ime(cx: &mut TestAppContext) {
        let (a, b, _, cx) = open(cx);
        assert!(accepts(cx, &a));
        focus(cx, &b);
        assert!(accepts(cx, &b));
        draw(cx);
        assert!(accepts(cx, &b));
        next_frame(cx);
        assert!(accepts(cx, &b));
        draw(cx);
        assert!(accepts(cx, &b));
    }
}
