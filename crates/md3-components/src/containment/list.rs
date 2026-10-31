//! List（1〜3 行の項目、仮想化した LazyList）。

use std::rc::Rc;

use gpui::{
    App, Div, ElementId, IntoElement, RenderOnce, SharedString, Stateful, UniformListScrollHandle,
    Window, div, prelude::*, px,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::list as tokens;
use md3_theme::md3_tokens::layout;

use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};
use gpui_headless::ListRow;

type SelectHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// 項目の先頭に置くもの。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListLeading {
    /// アイコン（24dp）。
    Icon(IconName),
    /// 頭文字のアバター（40dp の丸）。
    Avatar(SharedString),
}

/// 項目の末尾に置くもの。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListTrailing {
    /// アイコン（24dp）。
    Icon(IconName),
    /// 短い文字（件数、時刻など）。
    Text(SharedString),
}

/// リストの 1 項目。[`List`] か [`LazyList`] に入れて使う。
///
/// 行数は内容から決まる。上書き（overline）と補助テキストの両方があれば 3 行（88dp）、
/// どちらか一方なら 2 行（72dp）、見出しだけなら 1 行（56dp）。
#[derive(Clone)]
pub struct ListItem {
    headline: SharedString,
    aria_label: Option<Option<SharedString>>,
    overline: Option<SharedString>,
    supporting_text: Option<SharedString>,
    leading: Option<ListLeading>,
    trailing: Option<ListTrailing>,
    selected: bool,
    disabled: bool,
    on_select: Option<SelectHandler>,
    // LazyList の中では全項目が同じ高さなので、折り返さずに省略する。
    fixed_height: bool,
}

impl ListItem {
    /// アクセシビリティ情報。
    pub fn a11y_node(&self) -> gpui_headless::A11yNode {
        let mut node = self.headless().a11y_node(0, 1);
        let description = [self.overline.as_deref(), self.supporting_text.as_deref()]
            .into_iter()
            .flatten()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("、");
        node = node.description(description);
        node
    }

    /// 見出しから作る。
    pub fn new(headline: impl Into<SharedString>) -> Self {
        Self {
            headline: headline.into(),
            aria_label: None,
            overline: None,
            supporting_text: None,
            leading: None,
            trailing: None,
            selected: false,
            disabled: false,
            on_select: None,
            fixed_height: false,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 見出しの上の小さな文字。
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(text.into());
        self
    }

    /// 見出しの下の補助テキスト。
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 先頭に置くもの。
    pub fn leading(mut self, leading: ListLeading) -> Self {
        self.leading = Some(leading);
        self
    }

    /// 末尾に置くもの。
    pub fn trailing(mut self, trailing: ListTrailing) -> Self {
        self.trailing = Some(trailing);
        self
    }

    /// 選択中にする（`SecondaryContainer` で塗る）。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押したとき（クリック、Enter、Space）の処理。付けると項目がフォーカスを持つ。
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// 行数（1〜3）。
    pub fn line_count(&self) -> usize {
        match (self.overline.is_some(), self.supporting_text.is_some()) {
            (true, true) => 3,
            (false, false) => 1,
            _ => 2,
        }
    }

    fn headless(&self) -> gpui_headless::ListItem {
        let mut item = gpui_headless::ListItem::new(self.headline.clone())
            .selected(self.selected)
            .disabled(self.disabled);
        if let Some(label) = self.aria_label.clone() {
            item = item.aria_label(label.unwrap_or_default());
        }
        if let Some(on_select) = self.on_select.clone() {
            item = item.on_select(move |window, cx| on_select(window, cx));
        }
        item
    }

    fn render(self, row: ListRow, cx: &mut App) -> Stateful<Div> {
        let description = self.a11y_node().description;
        let ListRow {
            base,
            state: control,
            id,
            ..
        } = row;
        let lines = self.line_count();
        let focusable = !control.disabled;
        let theme = cx.md3().clone();
        let disabled = |role: ColorRole, opacity: f32| theme.color(role).opacity(opacity);
        let headline_color = if self.disabled {
            disabled(
                tokens::ITEM_DISABLED_LABEL_TEXT_COLOR,
                tokens::ITEM_DISABLED_LABEL_TEXT_OPACITY,
            )
        } else if self.selected {
            theme.color(tokens::ITEM_SELECTED_LABEL_TEXT_COLOR)
        } else {
            theme.color(tokens::ITEM_LABEL_TEXT_COLOR)
        };
        let secondary_color = |role: ColorRole| {
            if self.disabled {
                disabled(
                    tokens::ITEM_DISABLED_SUPPORTING_TEXT_COLOR,
                    tokens::ITEM_DISABLED_SUPPORTING_TEXT_OPACITY,
                )
            } else {
                theme.color(role)
            }
        };
        let icon_color = |role: ColorRole| {
            if self.disabled {
                disabled(
                    tokens::ITEM_DISABLED_LEADING_ICON_COLOR,
                    tokens::ITEM_DISABLED_LEADING_ICON_OPACITY,
                )
            } else if self.selected {
                theme.color(tokens::ITEM_SELECTED_LEADING_ICON_COLOR)
            } else {
                theme.color(role)
            }
        };
        let (height, vertical) = match lines {
            1 => (
                tokens::ITEM_ONE_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_VERTICAL_SPACE,
            ),
            2 => (
                tokens::ITEM_TWO_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_VERTICAL_SPACE,
            ),
            _ => (
                tokens::ITEM_THREE_LINE_CONTAINER_HEIGHT,
                layout::LIST_ITEM_THREE_LINE_VERTICAL_SPACE,
            ),
        };
        let radius = if self.selected {
            theme.corner(tokens::ITEM_SELECTED_CONTAINER_SHAPE)
        } else {
            theme.corner(tokens::ITEM_CONTAINER_SHAPE)
        };

        let leading = self.leading.map(|leading| {
            let element = match leading {
                ListLeading::Icon(icon) => Icon::new(icon)
                    .size(px(tokens::ITEM_LEADING_ICON_SIZE))
                    .tint(icon_color(tokens::ITEM_LEADING_ICON_COLOR))
                    .into_any_element(),
                ListLeading::Avatar(initials) => div()
                    .size(px(tokens::ITEM_LEADING_AVATAR_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .corner_radius(theme.corner(tokens::ITEM_LEADING_AVATAR_SHAPE))
                    .bg(theme.color(tokens::ITEM_LEADING_AVATAR_COLOR))
                    .text_color(theme.color(tokens::ITEM_LEADING_AVATAR_LABEL_COLOR))
                    .type_style(&theme.text_style(tokens::ITEM_LEADING_AVATAR_LABEL_FONT))
                    .when(self.disabled, |this| {
                        this.opacity(tokens::ITEM_DISABLED_LEADING_ICON_OPACITY)
                    })
                    .child(initials)
                    .into_any_element(),
            };
            div()
                .flex_none()
                .pr(px(layout::LIST_ITEM_LEADING_END_SPACE))
                .child(element)
        });
        let trailing = self.trailing.map(|trailing| {
            let element = match trailing {
                ListTrailing::Icon(icon) => Icon::new(icon)
                    .size(px(tokens::ITEM_TRAILING_ICON_SIZE))
                    .tint(icon_color(tokens::ITEM_TRAILING_ICON_COLOR))
                    .into_any_element(),
                ListTrailing::Text(text) => div()
                    .type_style(&theme.text_style(tokens::ITEM_TRAILING_SUPPORTING_TEXT_FONT))
                    .text_color(secondary_color(tokens::ITEM_TRAILING_SUPPORTING_TEXT_COLOR))
                    .child(text)
                    .into_any_element(),
            };
            div()
                .flex_none()
                .pl(px(layout::LIST_ITEM_TRAILING_START_SPACE))
                .child(element)
        });
        let text = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .when_some(self.overline, |this, overline| {
                this.child(
                    div()
                        .type_style(&theme.text_style(tokens::ITEM_OVERLINE_FONT))
                        .text_color(secondary_color(tokens::ITEM_OVERLINE_COLOR))
                        .child(overline),
                )
            })
            .child(
                div()
                    .type_style(&theme.text_style(tokens::ITEM_LABEL_TEXT_FONT))
                    .text_color(headline_color)
                    .when(self.fixed_height, |this| {
                        this.whitespace_nowrap().overflow_hidden().text_ellipsis()
                    })
                    .child(self.headline),
            )
            .when_some(self.supporting_text, |this, text| {
                this.child(
                    div()
                        .type_style(&theme.text_style(tokens::ITEM_SUPPORTING_TEXT_FONT))
                        .text_color(secondary_color(tokens::ITEM_SUPPORTING_TEXT_COLOR))
                        .when(self.fixed_height && lines < 3, |this| {
                            this.whitespace_nowrap().overflow_hidden().text_ellipsis()
                        })
                        .child(text),
                )
            });

        let mut item = base
            .when_some(description, |base, description| {
                base.aria_description(description)
            })
            .debug_selector(|| format!("md3-list-item-{id}"))
            .w_full()
            .min_h(px(height))
            .flex()
            .when(lines < 3, |this| this.items_center())
            .when(lines == 3, |this| this.items_start())
            .py(px(vertical))
            .pl(px(layout::LIST_ITEM_START_SPACE))
            .pr(px(layout::LIST_ITEM_END_SPACE))
            .corner_radius(radius)
            .bg(theme.color(if self.selected {
                tokens::ITEM_SELECTED_CONTAINER_COLOR
            } else {
                tokens::ITEM_CONTAINER_COLOR
            }))
            .children(leading)
            .child(text)
            .children(trailing);
        if focusable {
            item = item
                .child(
                    StateLayer::new(
                        SharedString::from(format!("{}-layer", control.group)),
                        control.group.clone(),
                        ColorRole::OnSurface,
                    )
                    .radius(radius)
                    .focused(control.focus_visible),
                )
                .child(
                    FocusRing::new(radius)
                        .inward()
                        .visible(control.focus_visible),
                );
        }
        item
    }
}

/// 全項目を描くリスト。項目が多いときは [`LazyList`] を使う。
///
/// 押せる項目（[`ListItem::on_select`]）があれば Tab ではリスト全体で 1 回だけ止まり、
/// 中は ↑↓ / Home / End で移る（選択中の項目、無ければ最初の押せる項目に止まる）。
#[derive(IntoElement)]
pub struct List {
    max_height: Option<gpui::Pixels>,
    scrollbar: bool,
    id: ElementId,
    label: Option<SharedString>,
    items: Vec<(SharedString, ListItem)>,
}
impl List {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            items: Vec::new(),
            max_height: None,
            scrollbar: true,
        }
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// 高さを制限し、超過する項目をスクロール可能にする。
    pub fn max_height(mut self, height: gpui::Pixels) -> Self {
        self.max_height = Some(height.max(px(0.)));
        self
    }
    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }
    /// 安定したキーと項目を足す。
    pub fn item(mut self, key: impl Into<SharedString>, item: ListItem) -> Self {
        self.items.push((key.into(), item));
        self
    }
}
impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut list = gpui_headless::List::new(self.id.clone())
            .reveal_margin(crate::focus::reveal_margin())
            .group_prefix("md3");
        if let Some(label) = self.label {
            list = list.aria_label(label);
        }
        if let Some(height) = self.max_height {
            list = list.max_height(height);
        }
        for (key, item) in &self.items {
            list = list.item(key.clone(), item.headless());
        }
        let items = self.items;
        let id = self.id;
        list.build(window, cx, move |row, _, cx| {
            items[row.index]
                .1
                .clone()
                .render(row, cx)
                .into_any_element()
        })
        .finish(|base, scroll| {
            base.debug_selector(|| format!("md3-list-{id}")).when_some(
                scroll.filter(|_| self.scrollbar),
                |base, scroll| {
                    base.child(crate::Scrollbar::new(
                        ElementId::NamedChild(id.clone().into(), "scrollbar".into()),
                        scroll.clone(),
                    ))
                },
            )
        })
    }
}

type ItemBuilder = Rc<dyn Fn(usize, &mut App) -> (SharedString, ListItem)>;
/// 見えている項目だけを描くリスト（gpui の `uniform_list`）。項目の高さはすべて同じにすること。
/// 長い文は折り返さずに省略する。
///
/// 大きさ（高さ）は呼び出し側が指定する。キー操作は [`List`] と同じで、
/// 見えていない項目へ移るときはその項目までスクロールしてからフォーカスする。
/// Tab で止まるのは最後にフォーカスした項目で、それが見えていなければ最初に見えている押せる項目。
#[derive(IntoElement)]
pub struct LazyList {
    scrollbar: bool,
    id: ElementId,
    label: Option<SharedString>,
    count: usize,
    scroll: UniformListScrollHandle,
    build: ItemBuilder,
    base: Div,
}
impl LazyList {
    /// ID、項目数、スクロール状態、項目を作る関数から作る。
    ///
    /// `build(index)` は項目のキーと中身を返す。キーはデータの安定したキーにする。
    pub fn new(
        id: impl Into<ElementId>,
        count: usize,
        scroll: &UniformListScrollHandle,
        build: impl Fn(usize, &mut App) -> (SharedString, ListItem) + 'static,
    ) -> Self {
        Self {
            scrollbar: true,
            id: id.into(),
            label: None,
            count,
            scroll: scroll.clone(),
            build: Rc::new(build),
            base: div(),
        }
    }
    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.label = (!label.is_empty()).then_some(label);
        self
    }

    /// スクロールバーの表示を切り替える（既定は表示）。
    pub fn scrollbar(mut self, visible: bool) -> Self {
        self.scrollbar = visible;
        self
    }
}
impl gpui::Styled for LazyList {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}
impl RenderOnce for LazyList {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let build = self.build;
        let mut list = gpui_headless::LazyList::new(
            self.id.clone(),
            self.count,
            &self.scroll,
            move |index, cx| {
                let (key, mut item) = build(index, cx);
                item.fixed_height = true;
                let behaviour = item.headless();
                (key, behaviour, item)
            },
        )
        .reveal_margin(crate::focus::reveal_margin())
        .group_prefix("md3");
        if let Some(label) = self.label {
            list = list.aria_label(label);
        }
        let mut base = list.build(window, cx, |row, item, _, cx| {
            item.render(row, cx).into_any_element()
        });
        *base.style() = self.base.style().clone();
        base.flex().flex_col().when(self.scrollbar, |base| {
            base.child(crate::Scrollbar::new(
                ElementId::NamedChild(self.id.into(), "scrollbar".into()),
                self.scroll,
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_count_follows_the_content() {
        assert_eq!(ListItem::new("a").line_count(), 1);
        assert_eq!(ListItem::new("a").supporting_text("b").line_count(), 2);
        assert_eq!(ListItem::new("a").overline("o").line_count(), 2);
        assert_eq!(
            ListItem::new("a")
                .overline("o")
                .supporting_text("b")
                .line_count(),
            3
        );
    }
}
