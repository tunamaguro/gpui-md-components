//! TextField（Filled / Outlined）。1 行と複数行の入力欄。

use super::TextFieldState;
use super::field::{FieldFrame, FieldState, TextFieldVariant, text_end_space, tint};
use crate::IconName;
use crate::scrollbar::{Scrollbar, ScrollbarTarget};
use gpui::{
    AnyElement, App, Axis, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString,
    Window, canvas, div, prelude::*, px,
};
use gpui_headless::{ScrollbarSource, TextFieldScroll, TextPaint, grapheme_count};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::filled_text_field as tokens;
use md3_theme::md3_tokens::component::scrollbar as scrollbar_tokens;
use md3_theme::md3_tokens::layout;
use std::cell::Cell;
use std::rc::Rc;

/// カーソルの幅。
pub(crate) const CARET_WIDTH: f32 = 2.0;

/// スクロールバーの操作領域の幅。WCAG 2.5.8 の 24 × 24 CSS px。
const SCROLLBAR_HIT_WIDTH: f32 = 24.0;
/// つまみの不透明度。常に見えるバーは隠れた行があることを示す唯一の印なので、
/// WCAG 1.4.11 の 3:1 を満たす値にする。
const SCROLLBAR_THUMB_OPACITY: f32 = 1.0;

/// 高さと位置はそのままに、右端を入力の右の余白の左端 + 操作領域の幅に合わせたスクロール。
struct GutterScroll {
    inner: TextFieldScroll,
    gutter_left: Rc<Cell<Pixels>>,
}

impl ScrollbarSource for GutterScroll {
    fn viewport(&self) -> Bounds<Pixels> {
        let mut viewport = self.inner.viewport();
        viewport.size.width = self.gutter_left.get() + px(SCROLLBAR_HIT_WIDTH) - viewport.left();
        viewport
    }
    fn max_offset(&self, axis: Axis) -> Pixels {
        self.inner.max_offset(axis)
    }
    fn offset(&self, axis: Axis) -> Pixels {
        self.inner.offset(axis)
    }
    fn set_offset(&self, axis: Axis, offset: Pixels) {
        self.inner.set_offset(axis, offset);
    }
}

/// 複数行の欄のスクロールバー。隠れた行がある間は常に表示し、幅 `gutter` の右の余白の中央に置く。
/// 操作領域は余白の左端から始まるので文字にかからず、バーが出ても折り返しは変わらない。
fn gutter_scrollbar(id: ElementId, scroll: TextFieldScroll, gutter: Pixels) -> AnyElement {
    let gutter_left = Rc::new(Cell::new(px(0.0)));
    let measured = gutter_left.clone();
    let source = GutterScroll {
        inner: scroll,
        gutter_left,
    };
    let cross_inset = px(SCROLLBAR_HIT_WIDTH) - (gutter + px(scrollbar_tokens::THICKNESS)) / 2.0;
    div()
        .size_full()
        .child(
            canvas(
                move |bounds, _, _| measured.set(bounds.left()),
                |_, _, _, _| {},
            )
            .size_full(),
        )
        .child(
            Scrollbar::new(
                ElementId::NamedChild(id.into(), "scrollbar".into()),
                ScrollbarTarget::new(source),
            )
            .fade(false)
            .thumb_opacity(SCROLLBAR_THUMB_OPACITY)
            .placement(cross_inset, px(SCROLLBAR_HIT_WIDTH)),
        )
        .into_any_element()
}

/// テキストフィールド。ラベルは読み上げ用の名前も兼ねる。
///
/// ```ignore
/// let state = TextFieldState::new(cx);
/// TextField::new("name", &state).label("名前").supporting_text("必須")
/// ```
///
/// キー操作は [`crate::init`] で割り当てる。
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    inner: gpui_headless::TextField,
    label: Option<SharedString>,
    variant: TextFieldVariant,
    placeholder: Option<SharedString>,
    supporting_text: Option<SharedString>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    multi_line: bool,
}

impl TextField {
    /// アクセシビリティ情報。
    #[cfg(test)]
    pub(crate) fn a11y_node(&self) -> gpui_headless::A11yNode {
        self.inner.a11y_node()
    }

    /// 補助テキストのアクセシビリティ情報。
    pub(crate) fn supporting_a11y_node(
        &self,
    ) -> Option<(gpui::ElementId, gpui_headless::A11yNode)> {
        self.supporting_text.as_ref().map(|text| {
            gpui_headless::supporting_status(&self.id, text.clone(), self.inner.a11y_node().invalid)
        })
    }

    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.inner = self.inner.required(required);
        self
    }

    /// ID と中身から作る。
    pub fn new(id: impl Into<ElementId>, state: &TextFieldState) -> Self {
        let id = id.into();
        Self {
            inner: gpui_headless::TextField::new(id.clone(), state.entity()),
            id,
            label: None,
            variant: TextFieldVariant::default(),
            placeholder: None,
            supporting_text: None,
            leading_icon: None,
            trailing_icon: None,
            prefix: None,
            suffix: None,
            multi_line: false,
        }
    }

    /// 表示するラベルを設定し、既定の名前にする。
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.inner = self.inner.label(text.clone());
        self.label = Some(text);
        self
    }

    delegate_accessible_name!(inner);

    /// 種類を変える。
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 空のときに出す例（ラベルが上に出ているときだけ見える）。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        let placeholder = placeholder.into();
        self.inner = self.inner.placeholder(placeholder.clone());
        self.placeholder = Some(placeholder);
        self
    }

    /// 下に出す補助テキスト（エラーのときはエラーの説明）。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.inner = self.inner.description(text.clone());
        self.supporting_text = Some(text);
        self
    }

    /// エラー状態にする。
    pub fn error(mut self, error: bool) -> Self {
        self.inner = self.inner.error(error);
        self
    }

    /// 文字数の上限。補助テキストの行に「文字数/上限」を出す（入力は制限しない）。
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.inner = self.inner.max_length(max_length);
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

    /// 複数行の入力欄にする。文字は欄の幅で折り返し、Enter で改行する。
    ///
    /// 高さは `min_lines` 行から `max_lines` 行の間で内容に合わせて変わり、`max_lines` を超えた分は
    /// 欄の中でスクロールする。隠れた行がある間はスクロールバーを出す（トラックが最小長に届かない低い
    /// 欄を除く）。上限なしは `usize::MAX`。
    /// `min_lines` は 1 以上に、`max_lines` は `min_lines` 以上に直す。
    ///
    /// ```ignore
    /// TextField::new("body", &state).label("本文").multi_line(4, 10)
    /// ```
    pub fn multi_line(mut self, min_lines: usize, max_lines: usize) -> Self {
        self.inner = self.inner.multi_line(min_lines, max_lines);
        self.multi_line = true;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.inner.a11y_node();
        let supporting_a11y = self.supporting_a11y_node();
        let (base, control) = self
            .inner
            .reveal_margin(crate::focus::reveal_margin())
            .build(window, cx);
        let gutter = control.scroll(cx).map(|scroll| {
            let width = text_end_space(self.multi_line, self.trailing_icon.is_some());
            gutter_scrollbar(self.id.clone(), scroll, width)
        });
        let text = &control.text;
        let focused = control.focused;
        let field_state = FieldState {
            focused,
            hovered: control.hovered,
            error: node.invalid,
            disabled: node.disabled,
        };
        let floated = focused || !text.is_empty();
        let theme = cx.md3();
        let colors = super::field::FieldColors::of(self.variant, field_state);
        let affix_color = |role: ColorRole| {
            if node.disabled {
                tint(theme, colors.input)
            } else {
                theme.color(role)
            }
        };

        let line = control.input(
            TextPaint {
                text: tint(theme, colors.input),
                placeholder: affix_color(tokens::INPUT_PLACEHOLDER_COLOR),
                caret: theme.color(colors.caret),
                selection: theme
                    .color(ColorRole::Primary)
                    .opacity(layout::TEXT_SELECTION_OPACITY),
                caret_width: px(CARET_WIDTH),
            },
            self.placeholder
                .clone()
                .filter(|_| focused || self.label.is_none()),
        );
        let affix_space = px(layout::TEXT_FIELD_PREFIX_SUFFIX_SPACE);
        // 接頭辞・接尾辞はラベルが上に出ている間だけ描く（複数行は入力を常に置くので、ここで絞る）。
        let shows_affix = floated || self.label.is_none();
        let content = div()
            .flex()
            .when(self.multi_line, |this| this.items_start())
            .when(!self.multi_line, |this| this.items_center())
            .w_full()
            .when_some(
                self.prefix.clone().filter(|_| shows_affix),
                |this, prefix| {
                    this.child(
                        div()
                            .flex_none()
                            .pr(affix_space)
                            .text_color(affix_color(tokens::INPUT_PREFIX_COLOR))
                            .child(prefix),
                    )
                },
            )
            .child(div().flex_1().min_w_0().child(line))
            .when_some(
                self.suffix.clone().filter(|_| shows_affix),
                |this, suffix| {
                    this.child(
                        div()
                            .flex_none()
                            .pl(affix_space)
                            .text_color(affix_color(tokens::INPUT_SUFFIX_COLOR))
                            .child(suffix),
                    )
                },
            )
            .into_any_element();

        let counter = control
            .max_length
            .map(|max| SharedString::from(format!("{}/{max}", grapheme_count(text))));
        let (container, supporting) = FieldFrame {
            variant: self.variant,
            state: field_state,
            label: self.label.clone(),
            floated,
            leading_icon: self.leading_icon,
            trailing: self.trailing_icon.map(|icon| {
                crate::Icon::new(icon)
                    .size(px(tokens::TRAILING_ICON_SIZE))
                    .tint(tint(theme, colors.trailing_icon))
                    .into_any_element()
            }),
            content,
            multi_line: self.multi_line,
            gutter,
            supporting_text: self.supporting_text.clone(),
            supporting_a11y,
            counter,
        }
        .render(cx);

        base.min_w(px(layout::TEXT_FIELD_MIN_WIDTH))
            .flex()
            .flex_col()
            .child(container)
            .children(supporting)
    }
}
