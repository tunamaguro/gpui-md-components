//! TextField（Filled / Outlined）。1 行の入力欄。

use super::field::{FieldFrame, FieldState, TextFieldVariant, tint};
use crate::IconName;
use gpui::{
    App, ElementId, Entity, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px,
};
use gpui_headless::{TextFieldState, TextPaint, grapheme_count};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::filled_text_field as tokens;
use md3_theme::md3_tokens::layout;

/// カーソルの幅。
pub(crate) const CARET_WIDTH: f32 = 2.0;

/// テキストフィールド。ラベルは読み上げ用の名前も兼ねる。
///
/// ```ignore
/// let state = cx.new(TextFieldState::new);
/// TextField::new("name", &state).label("名前").supporting_text("必須")
/// ```
///
/// キー操作は [`crate::init`] で割り当てる。
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    state: Entity<TextFieldState>,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    variant: TextFieldVariant,
    placeholder: Option<SharedString>,
    supporting_text: Option<SharedString>,
    error: bool,
    required: bool,
    max_length: Option<usize>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    disabled: bool,
}

impl TextField {
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        self.headless().a11y_node()
    }

    /// 補助テキストのアクセシビリティ情報。
    pub fn supporting_a11y_node(&self) -> Option<(gpui::ElementId, gpui_headless::A11yNode)> {
        self.supporting_text.as_ref().map(|text| {
            (
                ElementId::NamedChild(self.id.clone().into(), "supporting-text".into()),
                // macOS の AccessKit は live の読み上げに value を使う。
                gpui_headless::A11yNode::new(gpui::Role::Status)
                    .label(text.clone())
                    .value(text.clone())
                    .live(self.error),
            )
        })
    }

    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    fn headless(&self) -> gpui_headless::TextField {
        let mut field = gpui_headless::TextField::new(self.id.clone(), &self.state)
            .error(self.error)
            .disabled(self.disabled)
            .required(self.required);
        if let Some(label) = self.aria_label.clone().or_else(|| self.label.clone()) {
            field = field.aria_label(label);
        }
        if let Some(text) = &self.supporting_text {
            field = field.description(text.clone());
        }
        field
    }

    /// ID と中身から作る。
    pub fn new(id: impl Into<ElementId>, state: &Entity<TextFieldState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            aria_label: None,
            variant: TextFieldVariant::default(),
            placeholder: None,
            supporting_text: None,
            error: false,
            required: false,
            max_length: None,
            leading_icon: None,
            trailing_icon: None,
            prefix: None,
            suffix: None,
            disabled: false,
        }
    }

    /// 表示するラベルを設定し、既定の名前にする。
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
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
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut headless = self.headless().reveal_margin(crate::focus::reveal_margin());
        if let Some(placeholder) = self.placeholder.clone() {
            headless = headless.placeholder(placeholder);
        }
        if let Some(max_length) = self.max_length {
            headless = headless.max_length(max_length);
        }
        let (base, control) = headless.build(window, cx);
        let text = &control.text;
        let focused = control.focused;
        let field_state = FieldState {
            focused,
            hovered: control.hovered,
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
            supporting_text: self.supporting_text.clone(),
            supporting_a11y: self.supporting_a11y_node(),
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
