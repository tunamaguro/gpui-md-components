//! Select（テキストフィールドの見た目のドロップダウン。MD3 の exposed dropdown menu）。

use std::rc::Rc;
use std::sync::Arc;

use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::filled_text_field as tokens;
use md3_theme::md3_tokens::layout;

use super::field::{FieldColors, FieldFrame, FieldState, TextFieldVariant, tint};
use crate::overlays::{Menu, MenuItem, MenuKind};
use crate::{Icon, IconName};

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// 選択肢から 1 つを選ぶ入力。ラベルは読み上げ用の名前も兼ねる。
///
/// 押すか、Enter / Space / ↓ で選択肢のメニューを開く。開閉の状態は部品が持つ。
#[derive(IntoElement)]
pub struct Select {
    scrollbar: bool,
    id: ElementId,
    inner: gpui_headless::Select,
    label: Option<SharedString>,
    options: Vec<SharedString>,
    selected: Option<usize>,
    variant: TextFieldVariant,
    supporting_text: Option<SharedString>,
    leading_icon: Option<IconName>,
    on_change: Option<ChangeHandler>,
}

impl Select {
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        self.inner.a11y_node()
    }

    /// 補助テキストのアクセシビリティ情報。
    pub fn supporting_a11y_node(&self) -> Option<(gpui::ElementId, gpui_headless::A11yNode)> {
        self.supporting_text.as_ref().map(|text| {
            gpui_headless::supporting_status(&self.id, text.clone(), self.inner.a11y_node().invalid)
        })
    }

    /// 必須の入力か指定する。
    pub fn required(mut self, required: bool) -> Self {
        self.inner = self.inner.required(required);
        self
    }

    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }

    /// ID、選択肢、選ばれている位置から作る。
    pub fn new(
        id: impl Into<ElementId>,
        options: impl IntoIterator<Item = impl Into<SharedString>>,
        selected: Option<usize>,
    ) -> Self {
        let id = id.into();
        Self {
            inner: gpui_headless::Select::new(id.clone()),
            scrollbar: true,
            id,
            label: None,
            options: options.into_iter().map(Into::into).collect(),
            selected,
            variant: TextFieldVariant::default(),
            supporting_text: None,
            leading_icon: None,
            on_change: None,
        }
    }

    /// 表示するラベルを設定し、既定の名前にする。
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.inner = self.inner.label(text.clone());
        self.label = Some(text);
        self
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }

    /// 種類を変える。
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 下に出す補助テキスト。
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

    /// 先頭のアイコン（装飾）。
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    /// 選んだときの処理。引数は選んだ位置。
    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn value(&self) -> Option<SharedString> {
        self.selected
            .and_then(|index| self.options.get(index).cloned())
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = self.value();
        let parent = Arc::new(self.id.clone());
        let node = self.inner.a11y_node();
        let supporting_a11y = self.supporting_a11y_node();
        let (base, select_state) = self
            .inner
            .value(value.clone())
            .reveal_margin(crate::focus::reveal_margin())
            .group_prefix("md3")
            .build(window, cx);
        let is_open = select_state.open;
        let focused = select_state.focused;
        let state = FieldState {
            focused,
            hovered: select_state.hovered,
            error: node.invalid,
            disabled: select_state.disabled,
        };
        let theme = &cx.md3().clone();
        let colors = FieldColors::of(self.variant, state);
        let arrow = if is_open {
            IconName::ArrowDropUp
        } else {
            IconName::ArrowDropDown
        };
        let (mut container, supporting) = FieldFrame {
            variant: self.variant,
            state,
            label: self.label.clone(),
            floated: value.is_some() || focused,
            leading_icon: self.leading_icon,
            trailing: Some(
                Icon::new(arrow)
                    .size(px(tokens::TRAILING_ICON_SIZE))
                    .tint(tint(theme, colors.trailing_icon))
                    .into_any_element(),
            ),
            content: div()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .children(value.clone())
                .into_any_element(),
            supporting_text: self.supporting_text.clone(),
            supporting_a11y,
            counter: None,
        }
        .render(cx);

        if is_open {
            let on_change = self.on_change.clone();
            let close = select_state.dismiss_handler();
            let menu = Menu::new(ElementId::NamedChild(parent.clone(), "menu".into()))
                .when_some(node.label, |menu, label| menu.aria_label(label))
                .scrollbar(self.scrollbar)
                .kind(MenuKind::Options)
                .selected(self.selected)
                .match_anchor_width()
                .items(self.options.iter().enumerate().map(|(index, option)| {
                    let on_change = on_change.clone();
                    MenuItem::new(option.clone()).on_select(move |window, cx| {
                        if let Some(on_change) = &on_change {
                            on_change(index, window, cx);
                        }
                    })
                }))
                .on_dismiss(move |window, cx| close(window, cx));
            container = container.child(menu);
        }
        // テキストフィールドと同じく、フォーカスはフォーカスリングではなく下線・枠線とラベルの色で示す。
        let _ = &theme;

        base.min_w(px(layout::TEXT_FIELD_MIN_WIDTH))
            .flex()
            .flex_col()
            .child(container)
            .children(supporting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_shows_the_selected_option() {
        let select = Select::new("s", ["赤", "青"], Some(1)).label("色");
        assert_eq!(select.value().as_deref(), Some("青"));
        assert_eq!(Select::new("s", ["赤"], Some(3)).label("色").value(), None);
    }
}
