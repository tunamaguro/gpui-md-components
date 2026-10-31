// The wrapped-row geometry follows gpui's WrappedLineLayout:
// https://github.com/zed-industries/zed crates/gpui/src/text_system/line_layout.rs @ 37faef81
// Copyright 2022 - 2025 Zed Industries, Inc. Licensed under the Apache License, Version 2.0.

//! 入力欄の文字の配置。段落（改行で区切った文字）と、折り返した表示行を扱う。

use gpui::{App, Pixels, Point, SharedString, TextAlign, Window, WrappedLine, point, px};
use std::ops::Range;

/// 表示行。範囲は文字列全体のバイト位置で、段落の改行を含まない。
#[derive(Clone)]
struct Row {
    range: Range<usize>,
    paragraph: usize,
    /// 段落の先頭から測った、行の始まりの x。
    start_x: Pixels,
    width: Pixels,
}

struct Paragraph {
    line: WrappedLine,
    start: usize,
    first_row: usize,
}

/// 位置から求めたカーソル。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Caret {
    /// 文字列のバイト位置。
    pub offset: usize,
    /// 折り返し位置を前の行の末尾として扱うか。
    pub upstream: bool,
}

/// 整形済みの文字の配置。
pub(crate) struct TextLayout {
    source: SharedString,
    paragraphs: Vec<Paragraph>,
    rows: Vec<Row>,
    line_height: Pixels,
}

impl TextLayout {
    /// `shape_text` の結果から作る。`source` は入力欄の文字列で、整形した文字列と同じバイト長にする。
    pub(crate) fn new(
        source: SharedString,
        lines: impl IntoIterator<Item = WrappedLine>,
        line_height: Pixels,
    ) -> Self {
        let mut paragraphs = Vec::new();
        let mut rows = Vec::new();
        let mut start = 0;
        for line in lines {
            let paragraph = paragraphs.len();
            let first_row = rows.len();
            let (mut row_start, mut start_x) = (0, px(0.0));
            for boundary in line.wrap_boundaries() {
                let glyph = &line.runs()[boundary.run_ix].glyphs[boundary.glyph_ix];
                rows.push(Row {
                    range: start + row_start..start + glyph.index,
                    paragraph,
                    start_x,
                    width: glyph.position.x - start_x,
                });
                row_start = glyph.index;
                start_x = glyph.position.x;
            }
            rows.push(Row {
                range: start + row_start..start + line.len(),
                paragraph,
                start_x,
                width: line.unwrapped_layout.width - start_x,
            });
            let next = start + line.len() + '\n'.len_utf8();
            paragraphs.push(Paragraph {
                line,
                start,
                first_row,
            });
            start = next;
        }
        Self {
            source,
            paragraphs,
            rows,
            line_height,
        }
    }

    /// この配置が `text` のものか。古い配置を位置の計算に使わないために確かめる。
    pub(crate) fn is_for(&self, text: &str) -> bool {
        self.source.as_ref() == text
    }

    pub(crate) fn line_height(&self) -> Pixels {
        self.line_height
    }

    pub(crate) fn row_count(&self) -> usize {
        self.rows.len().max(1)
    }

    pub(crate) fn height(&self) -> Pixels {
        self.line_height * self.row_count()
    }

    pub(crate) fn width(&self) -> Pixels {
        self.rows
            .iter()
            .map(|row| row.width)
            .fold(px(0.0), Pixels::max)
    }

    fn is_soft_end(&self, row: usize) -> bool {
        self.rows
            .get(row + 1)
            .is_some_and(|next| next.paragraph == self.rows[row].paragraph)
    }

    /// `offset` のある表示行の番号。
    pub(crate) fn row_of(&self, offset: usize, upstream: bool) -> usize {
        let row = self
            .rows
            .partition_point(|row| row.range.start <= offset)
            .saturating_sub(1);
        let at_soft_start = row > 0
            && self.rows[row].range.start == offset
            && self.rows[row - 1].paragraph == self.rows[row].paragraph;
        if upstream && at_soft_start {
            row - 1
        } else {
            row
        }
    }

    /// 表示行の先頭のカーソル。
    pub(crate) fn row_start(&self, row: usize) -> Caret {
        Caret {
            offset: self.rows.get(row).map_or(0, |row| row.range.start),
            upstream: false,
        }
    }

    /// 表示行の末尾のカーソル。折り返しの行なら行の末尾に出す。
    pub(crate) fn row_end(&self, row: usize) -> Caret {
        Caret {
            offset: self.rows.get(row).map_or(0, |row| row.range.end),
            upstream: self.is_soft_end(row),
        }
    }

    fn x_in_row(&self, row: &Row, offset: usize) -> Pixels {
        let paragraph = &self.paragraphs[row.paragraph];
        paragraph
            .line
            .unwrapped_layout
            .x_for_index(offset.saturating_sub(paragraph.start))
            - row.start_x
    }

    /// カーソルの左上の位置（文字列の左上が原点）。
    pub(crate) fn position(&self, offset: usize, upstream: bool) -> Point<Pixels> {
        let index = self.row_of(offset, upstream);
        let Some(row) = self.rows.get(index) else {
            return point(px(0.0), px(0.0));
        };
        point(self.x_in_row(row, offset), self.line_height * index)
    }

    /// 位置に最も近いカーソル。文字のない位置は最も近い表示行に収める。
    pub(crate) fn caret_at(&self, position: Point<Pixels>) -> Caret {
        let index = (position.y / self.line_height)
            .floor()
            .clamp(0.0, (self.row_count() - 1) as f32) as usize;
        let Some(row) = self.rows.get(index) else {
            return Caret {
                offset: 0,
                upstream: false,
            };
        };
        if position.x <= px(0.0) {
            return self.row_start(index);
        }
        if position.x >= row.width {
            return self.row_end(index);
        }
        let paragraph = &self.paragraphs[row.paragraph];
        let offset = paragraph.start
            + paragraph
                .line
                .unwrapped_layout
                .closest_index_for_x(position.x + row.start_x);
        let offset = offset.clamp(row.range.start, row.range.end);
        Caret {
            offset,
            upstream: offset == row.range.end && self.is_soft_end(index),
        }
    }

    /// 選択範囲を表示行ごとの `(行の番号, 左の x, 右の x)` にする。
    /// 段落の改行をまたぐ行は `line_break_width` だけ右へ延ばす。
    pub(crate) fn selection_spans(
        &self,
        selection: &Range<usize>,
        line_break_width: Pixels,
    ) -> Vec<(usize, Pixels, Pixels)> {
        let mut spans = Vec::new();
        for (index, row) in self.rows.iter().enumerate() {
            if row.range.start > selection.end || row.range.end < selection.start {
                continue;
            }
            let start = selection.start.max(row.range.start);
            let end = selection.end.min(row.range.end);
            let left = self.x_in_row(row, start);
            let right = self.x_in_row(row, end);
            let crosses_break = selection.end > row.range.end
                && selection.start <= row.range.end
                && !self.is_soft_end(index);
            if crosses_break {
                spans.push((index, left, right + line_break_width));
            } else if start < end {
                spans.push((index, left, right));
            }
        }
        spans
    }

    /// `visible` の高さ（文字列の上端からの距離）にかかる段落を描く。
    pub(crate) fn paint(
        &self,
        origin: Point<Pixels>,
        visible: Range<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for paragraph in &self.paragraphs {
            let top = self.line_height * paragraph.first_row;
            let bottom = top + paragraph.line.size(self.line_height).height;
            if bottom <= visible.start || top >= visible.end {
                continue;
            }
            // 描けなかった行は飛ばす（次のフレームで描き直す）。
            let _ = paragraph.line.paint(
                point(origin.x, origin.y + top),
                self.line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }
    }
}
