// The editing logic (TextFieldState, its EntityInputHandler impl and TextLine) is ported from
// gpui's example: https://github.com/zed-industries/zed crates/gpui/examples/input.rs @ 37faef81
// Copyright 2022 - 2025 Zed Industries, Inc. Licensed under the Apache License, Version 2.0.

//! TextField（Filled / Outlined）。1 行の入力で、IME の変換中の文字に下線を引く。

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, Element, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, PaintQuad, Pixels, Point, RenderOnce, Role, ShapedLine, SharedString, Style,
    TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window, div, fill, point, prelude::*, px,
    relative, size,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::filled_text_field as tokens;
use md3_theme::md3_tokens::layout;

use super::field::{FieldFrame, FieldState, TextFieldVariant, tint};
use crate::a11y::{A11yExt, A11yNode};
use crate::keys::{
    Backspace, Copy, Cut, Delete, End, Home, Left, Paste, Right, SelectAll, SelectEnd, SelectHome,
    SelectLeft, SelectRight, TEXT_FIELD_CONTEXT,
};
use crate::{ForcedState, IconName};

/// カーソルの幅。
const CARET_WIDTH: f32 = 2.0;

/// 入力の内容が変わったときのイベント。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFieldEvent {
    /// 文字が変わった（変換中の文字を含む）。
    Changed,
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
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
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
            last_layout: None,
            last_bounds: None,
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

    /// 選択範囲（UTF-8 のバイト位置）。
    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    /// IME の変換中の範囲（UTF-8 のバイト位置）。
    pub fn marked_range(&self) -> Option<Range<usize>> {
        self.marked_range.clone()
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
        let last_layout = self.last_layout.as_ref()?;
        if last_layout.text.as_ref() != self.content {
            return None;
        }
        let range = self.range_from_utf16(&range_utf16);
        let left = bounds.left() - self.scroll_x;
        Some(Bounds::from_corners(
            point(left + last_layout.x_for_index(range.start), bounds.top()),
            point(left + last_layout.x_for_index(range.end), bounds.bottom()),
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
struct TextLine {
    state: Entity<TextFieldState>,
    placeholder: Option<SharedString>,
    placeholder_color: Hsla,
    caret: Hsla,
    selection: Hsla,
    focused: bool,
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
                (placeholder.clone(), self.placeholder_color)
            }
            _ => (content.clone(), style.color),
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
        let visible = (bounds.size.width - px(CARET_WIDTH)).max(px(0.0));
        if caret_x - scroll_x > visible {
            scroll_x = caret_x - visible;
        }
        if caret_x < scroll_x {
            scroll_x = caret_x;
        }
        let max_scroll = if content.is_empty() {
            px(0.0)
        } else {
            (line.width + px(CARET_WIDTH) - bounds.size.width).max(px(0.0))
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
                        size(px(CARET_WIDTH), bounds.size.height),
                    ),
                    self.caret,
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
                    self.selection,
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
        self.state.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

/// テキストフィールド。ラベル（読み上げ用の名前を兼ねる）が必須。
///
/// ```ignore
/// let state = cx.new(TextFieldState::new);
/// TextField::new("name", &state, "名前").supporting_text("必須")
/// ```
///
/// キー操作は [`crate::init`] で割り当てる。
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    state: Entity<TextFieldState>,
    label: SharedString,
    variant: TextFieldVariant,
    placeholder: Option<SharedString>,
    supporting_text: Option<SharedString>,
    error: bool,
    max_length: Option<usize>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    disabled: bool,
    forced: Option<ForcedState>,
}

impl TextField {
    /// ID、中身、ラベルから作る。
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<TextFieldState>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: label.into(),
            variant: TextFieldVariant::default(),
            placeholder: None,
            supporting_text: None,
            error: false,
            max_length: None,
            leading_icon: None,
            trailing_icon: None,
            prefix: None,
            suffix: None,
            disabled: false,
            forced: None,
        }
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 空のときに出す例（ラベルが上に出ているときだけ見える）。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 下に出す補助テキスト（エラーのときはエラーの説明）。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// エラー状態にする。
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// 文字数の上限。補助テキストの行に「文字数/上限」を出す（入力は制限しない）。
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// 先頭のアイコン（装飾）。
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// 末尾のアイコン（装飾）。
    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    /// 入力の前に付ける文字（例: `¥`）。
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 入力の後に付ける文字（例: `kg`）。
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 見本用に状態を固定する（`Hovered` と `Focused` だけを使う）。
    #[doc(hidden)]
    pub fn forced(mut self, forced: ForcedState) -> Self {
        self.forced = Some(forced);
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        A11yNode::new(Role::TextInput)
            .label(self.label.clone())
            .disabled(self.disabled)
    }
}

/// 文字数（書記素の数）。
fn char_count(text: &str) -> usize {
    boundaries(text).count()
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let hovered = window.use_keyed_state(
            ElementId::NamedChild(self.id.clone().into(), "hovered".into()),
            cx,
            |_, _| false,
        );
        let state = self.state.read(cx);
        let focus_handle = state.focus_handle.clone();
        let text = state.content.clone();
        let really_focused = !self.disabled && focus_handle.is_focused(window);
        let focused =
            !self.disabled && (really_focused || self.forced == Some(ForcedState::Focused));
        let field_state = FieldState {
            focused,
            hovered: !self.disabled
                && (*hovered.read(cx) || self.forced == Some(ForcedState::Hovered)),
            error: self.error,
            disabled: self.disabled,
        };
        let floated = focused || !text.is_empty();
        let theme = cx.md3();
        let colors = super::field::FieldColors::of(self.variant, field_state);
        let affix_color = |role: ColorRole| {
            if self.disabled {
                tint(theme, colors.input)
            } else {
                theme.color(role)
            }
        };

        let line = TextLine {
            state: self.state.clone(),
            placeholder: self.placeholder.clone().filter(|_| focused),
            placeholder_color: affix_color(tokens::INPUT_PLACEHOLDER_COLOR),
            caret: theme.color(colors.caret),
            selection: theme
                .color(ColorRole::Primary)
                .opacity(layout::TEXT_SELECTION_OPACITY),
            focused: really_focused,
        };
        let affix_space = px(layout::TEXT_FIELD_PREFIX_SUFFIX_SPACE);
        let content = div()
            .flex()
            .items_center()
            .w_full()
            .when_some(self.prefix.clone(), |this, prefix| {
                this.child(
                    div()
                        .flex_none()
                        .pr(affix_space)
                        .text_color(affix_color(tokens::INPUT_PREFIX_COLOR))
                        .child(prefix),
                )
            })
            .child(div().flex_1().min_w_0().child(line))
            .when_some(self.suffix.clone(), |this, suffix| {
                this.child(
                    div()
                        .flex_none()
                        .pl(affix_space)
                        .text_color(affix_color(tokens::INPUT_SUFFIX_COLOR))
                        .child(suffix),
                )
            })
            .into_any_element();

        let counter = self
            .max_length
            .map(|max| SharedString::from(format!("{}/{max}", char_count(&text))));
        let (container, supporting) = FieldFrame {
            variant: self.variant,
            state: field_state,
            label: Some(self.label.clone()),
            floated,
            leading_icon: self.leading_icon,
            trailing: self.trailing_icon.map(|icon| {
                crate::Icon::new(icon)
                    .size(px(tokens::TRAILING_ICON_SIZE))
                    .tint(tint(theme, colors.trailing_icon))
                    .into_any_element()
            }),
            content,
            supporting_text: self.supporting_text.clone(),
            counter,
        }
        .render(cx);

        let mut field = div()
            .id(self.id.clone())
            .a11y(node)
            .aria_value(SharedString::from(text))
            .when_some(self.placeholder.clone(), |this, placeholder| {
                this.aria_placeholder(placeholder)
            })
            .min_w(px(layout::TEXT_FIELD_MIN_WIDTH))
            .flex()
            .flex_col()
            .child(container)
            .children(supporting);
        if !self.disabled {
            let state = self.state.clone();
            field = field
                .key_context(TEXT_FIELD_CONTEXT)
                .track_focus(&focus_handle)
                .cursor(CursorStyle::IBeam)
                .on_hover(move |hover, _, cx| {
                    hovered.update(cx, |value, cx| {
                        *value = *hover;
                        cx.notify();
                    })
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
                });
        }
        field
    }
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
        assert_eq!(char_count(text), 2);
    }

    #[test]
    fn emoji_sequences_are_one_character() {
        let family = "👨\u{200D}👩\u{200D}👧";
        assert_eq!(next_boundary(family, 0), family.len());
        let flags = "🇯🇵🇺🇸";
        assert_eq!(next_boundary(flags, 0), "🇯🇵".len());
        assert_eq!(char_count(flags), 2);
    }

    #[test]
    fn boundaries_at_the_ends() {
        assert_eq!(previous_boundary("abc", 0), 0);
        assert_eq!(next_boundary("abc", 3), 3);
        assert_eq!(char_count(""), 0);
    }
}
