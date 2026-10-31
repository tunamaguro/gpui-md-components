// The editing logic (TextFieldState, its EntityInputHandler impl and TextLine) is ported from
// gpui's example: https://github.com/zed-industries/zed crates/gpui/examples/input.rs @ 37faef81
// Copyright 2022 - 2025 Zed Industries, Inc. Licensed under the Apache License, Version 2.0.

//! 1 行のテキスト編集と入力欄。

use crate::keys::{
    Backspace, Copy, Cut, Delete, End, Home, Left, Paste, Right, SelectAll, SelectEnd, SelectHome,
    SelectLeft, SelectRight, TEXT_FIELD_CONTEXT,
};
use crate::{A11yExt, A11yNode};
use gpui::{
    A11ySubtreeBuilder, App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, Div,
    Element, ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, Global, GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, PaintQuad, Pixels, Point, Role, ShapedLine,
    SharedString, Stateful, Style, Subscription, TextAlign, TextRun, UTF16Selection,
    UnderlineStyle, Window, WindowId, accesskit, div, fill, point, prelude::*, px, relative, size,
};
use std::ops::Range;

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

/// IME に伝える入力行の範囲と位置。
#[derive(PartialEq)]
struct ImeAnchor {
    bounds: Bounds<Pixels>,
    x: Pixels,
}

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
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    last_ime_anchor: Option<ImeAnchor>,
    scroll_x: Pixels,
    is_selecting: bool,
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
            scroll_x: px(0.0),
            is_selecting: false,
        }
    }

    /// 初期値を入れる。
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.content = text.into();
        let end = self.content.len();
        self.selected_range = end..end;
        self
    }

    /// 支援技術に渡す、文字（書記素）ごとの UTF-8 の長さ。
    pub fn a11y_character_lengths(&self) -> Vec<u8> {
        character_lengths(&self.content)
    }

    /// 支援技術に渡す選択範囲の始点と終点（文字の番号）。
    pub fn a11y_selection(&self) -> (usize, usize) {
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

    /// 支援技術からの要求で、選択範囲を移す。位置は文字の番号。
    pub fn a11y_set_selection(
        &mut self,
        anchor: usize,
        focus: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.end_composition();
        let characters = characters(&self.content);
        let offset = |index: usize| {
            characters
                .get(index)
                .map_or(self.content.len(), |character| character.start)
        };
        let (anchor_offset, focus_offset) = (offset(anchor), offset(focus));
        self.selected_range = anchor_offset.min(focus_offset)..anchor_offset.max(focus_offset);
        self.selection_reversed = focus_offset < anchor_offset;
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
        self.replace_text_in_range(Some(range_utf16), &text.replace('\n', " "), window, cx);
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

    /// 文字列を置き換え、カーソルを末尾に置く。
    pub fn set_text(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.content = text.into();
        let end = self.content.len();
        self.selected_range = end..end;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.emit(TextFieldEvent::Changed);
        cx.notify();
    }

    fn ime_x_for_offset(&self, offset: usize, bounds: Bounds<Pixels>) -> Option<Pixels> {
        if self.content.is_empty() {
            return Some(bounds.left());
        }
        let line = self.last_layout.as_ref()?;
        if line.text.as_ref() != self.content {
            return None;
        }
        Some(
            (bounds.left() - self.scroll_x + line.x_for_index(offset))
                .clamp(bounds.left(), bounds.right()),
        )
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

    fn home(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    fn select_home(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, window, cx);
    }

    fn select_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.content.len(), window, cx);
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
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
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
        let index = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(index, window, cx);
        } else {
            self.move_to(index, window, cx)
        }
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
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

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left() + self.scroll_x)
    }

    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
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

    fn changed(&mut self, window: &Window, cx: &mut Context<Self>) {
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
        Some(Bounds::from_corners(
            point(self.ime_x_for_offset(range.start, bounds)?, bounds.top()),
            point(self.ime_x_for_offset(range.end, bounds)?, bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        if last_layout.text.as_ref() != self.content {
            return None;
        }
        let utf8_index = last_layout.closest_index_for_x(line_point.x + self.scroll_x);
        Some(self.offset_to_utf16(utf8_index))
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
            let joins = is_extending(next)
                || previous == '\u{200D}'
                || (regional == 1 && is_regional_indicator(next));
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

/// 選択範囲の始点と終点を文字の番号にする。
fn selection_characters(text: &str, selection: &Range<usize>, reversed: bool) -> (usize, usize) {
    let characters = characters(text);
    let index = |offset: usize| characters.partition_point(|character| character.end <= offset);
    let (start, end) = (index(selection.start), index(selection.end));
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

/// 入力行（文字、選択範囲、カーソル）を描く要素。
/// 入力欄の 1 行（文字、placeholder、変換中の下線、選択、キャレット）。
struct TextLine {
    pub state: Entity<TextFieldState>,
    pub placeholder: Option<SharedString>,
    pub paint: TextPaint,
    pub focused: bool,
}

struct TextLinePrepaint {
    line: Option<ShapedLine>,
    origin_x: Pixels,
    caret: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextLine {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextLine {
    type RequestLayoutState = ();
    type PrepaintState = TextLinePrepaint;

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
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
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
        let input = self.state.read(cx);
        let content: SharedString = input.content.clone().into();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let marked_range = input.marked_range.clone();
        let mut scroll_x = input.scroll_x;
        let style = window.text_style();

        let (display_text, text_color) = match &self.placeholder {
            Some(placeholder) if content.is_empty() => {
                (placeholder.clone(), self.paint.placeholder)
            }
            _ => (content.clone(), self.paint.text),
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
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        // カーソルが見える位置まで横に送る。
        let caret_x = if content.is_empty() {
            px(0.0)
        } else {
            line.x_for_index(cursor)
        };
        let visible = (bounds.size.width - self.paint.caret_width).max(px(0.0));
        if caret_x - scroll_x > visible {
            scroll_x = caret_x - visible;
        }
        if caret_x < scroll_x {
            scroll_x = caret_x;
        }
        let max_scroll = if content.is_empty() {
            px(0.0)
        } else {
            (line.width + self.paint.caret_width - bounds.size.width).max(px(0.0))
        };
        scroll_x = scroll_x.min(max_scroll).max(px(0.0));
        self.state.update(cx, |state, _| state.scroll_x = scroll_x);
        let origin_x = bounds.left() - scroll_x;

        let (selection, caret) = if selected_range.is_empty() || content.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(origin_x + caret_x, bounds.top()),
                        size(self.paint.caret_width, bounds.size.height),
                    ),
                    self.paint.caret,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            origin_x + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            origin_x + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    self.paint.selection,
                )),
                None,
            )
        };
        TextLinePrepaint {
            line: Some(line),
            origin_x,
            caret,
            selection,
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
        let Some(line) = prepaint.line.take() else {
            return;
        };
        let line_height = window.line_height();
        let origin = point(prepaint.origin_x, bounds.top());
        let selection = prepaint.selection.take();
        let caret = prepaint.caret.take().filter(|_| self.focused);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = selection {
                window.paint_quad(selection)
            }
            // 描けなかった行は飛ばす（次のフレームで描き直す）。
            let _ = line.paint(origin, line_height, TextAlign::Left, None, window, cx);
            if let Some(caret) = caret {
                window.paint_quad(caret);
            }
        });
        self.state.update(cx, |input, cx| {
            input.last_layout = Some(line);
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
                    .ime_x_for_offset(offset, bounds)
                    .map(|x| ImeAnchor { bounds, x });
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
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    placeholder: Option<SharedString>,
    error: bool,
    required: bool,
    description: Option<SharedString>,
    disabled: bool,
    max_length: Option<usize>,
    reveal_margin: Pixels,
    role: Role,
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
            label: None,
            aria_label: None,
            placeholder: None,
            error: false,
            required: false,
            description: None,
            disabled: false,
            max_length: None,
            reveal_margin: px(0.0),
            role: Role::TextInput,
        }
    }
    /// 表示するラベルを既定の名前にする。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

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
    /// フォーカス追従に必要な余白を設定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> A11yNode {
        let mut node = A11yNode::new(self.role)
            .with_label(self.aria_label.clone().or_else(|| self.label.clone()))
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
        self.state
            .update(cx, |state, cx| state.observe_blur(window, cx));
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
        let mut base = div()
            .id(self.id.clone())
            .a11y_with_children(self.a11y_node(), {
                let text = text.clone();
                move |builder| write_text_run(builder, text, selection, reversed)
            })
            .aria_value(SharedString::from(text.clone()))
            .when_some(self.placeholder.clone(), |this, placeholder| {
                this.aria_placeholder(placeholder)
            });
        if !self.disabled {
            base = a11y_actions(base, &self.state);
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
        TextLine {
            state: self.state.clone(),
            placeholder,
            paint,
            focused: self.focused,
        }
    }
}

/// 文字とカーソルを支援技術に渡す（gpui の `_accessibility` の例と同じ形）。
fn write_text_run(
    builder: &mut A11ySubtreeBuilder,
    text: String,
    selection: Range<usize>,
    reversed: bool,
) {
    let (anchor, focus) = selection_characters(&text, &selection, reversed);
    let mut run = accesskit::Node::new(Role::TextRun);
    run.set_character_lengths(character_lengths(&text));
    run.set_value(text);
    let node = builder.synthetic_node_id(0);
    builder.push_child(node, run);
    builder
        .parent_node()
        .set_text_selection(accesskit::TextSelection {
            anchor: accesskit::TextPosition {
                node,
                character_index: anchor,
            },
            focus: accesskit::TextPosition {
                node,
                character_index: focus,
            },
        });
}

/// 支援技術からの編集を受ける（WCAG 4.1.2。Compose の `setText` など）。
fn a11y_actions(field: Stateful<Div>, state: &Entity<TextFieldState>) -> Stateful<Div> {
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
                    let (anchor, focus) = (
                        selection.anchor.character_index,
                        selection.focus.character_index,
                    );
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
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.0 = self.0.aria_label(label);
        self
    }
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
    let state = state.clone();
    field
        .relative()
        .child(crate::reveal_on_focus(
            id,
            focus_handle.clone(),
            window,
            cx,
            reveal_margin,
        ))
        .key_context(TEXT_FIELD_CONTEXT)
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
