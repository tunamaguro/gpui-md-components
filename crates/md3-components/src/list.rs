// 寸法・色・形の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt
//         (`ListItem`、`InteractiveListItem`、`InteractiveListItemLayout`、`LeadingDecorator`、
//         `TrailingDecorator`、`ListItemType`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItemDefaults.kt
//         (`ListItemDefaults`、`defaultListItemColors`、`defaultListItemShapes`、`shapeForInteraction`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 項目のフォーカスリングを項目の内側に描くことは Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: list/internal/listitem/list-item.ts (`md-focus-ring` の `inward`)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のリスト

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder, px, size, svg,
};
use gpui_headless::choice::CheckedState;
use gpui_headless::list::{
    List as HeadlessList, ListItem as HeadlessListItem, ListItemKind as HeadlessListItemKind,
};
use gpui_headless::text::StaticText;

use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::list;
use md3_theme::md3_tokens::{focus_ring, state};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 項目の層が項目の hover を知るための group 名
const GROUP: &str = "md3-list-item";

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 項目を縦に並べるリスト
///
/// 項目は [`List::item`] で、項目でない要素 (区切り線など) は `child` で加え、加えた順に並ぶ。
/// 操作できる項目の間は、上下の矢印キーと Home・End でフォーカスを移せ、Tab で到達できるのはそのうち 1 つだけになる。
/// 支援技術には role `list` として公開される。大きさと配置は [`Styled`] で付ける
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    aria_label: Option<SharedString>,
    children: Vec<ListChild>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

enum ListChild {
    Item(Box<ListItem>),
    Other(AnyElement),
}

impl List {
    /// 項目を持たないリストを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            aria_label: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ListItem) -> Self {
        self.children.push(ListChild::Item(Box::new(item)));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ListItem>) -> Self {
        self.children.extend(
            items
                .into_iter()
                .map(|item| ListChild::Item(Box::new(item))),
        );
        self
    }

    /// 支援技術に公開するリストの名前を設定する
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 描画に使うテーマを設定する。指定しない場合はウィンドウのテーマを使う
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let mut list = HeadlessList::new(self.id.clone())
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .flex()
            .flex_col();
        list.style().refine(&self.style);
        for child in self.children {
            list = match child {
                ListChild::Item(item) => {
                    let focus_handle = item.focus_handle(&self.id, window, cx);
                    list.item(item.into_headless(&theme, &focus_handle, window, cx))
                }
                ListChild::Other(element) => list.child(element),
            };
        }
        list
    }
}

impl Styled for List {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for List {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(ListChild::Other));
    }
}

/// リストの項目の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ListItemKind {
    /// 操作できない項目
    #[default]
    Text,
    /// 操作を実行する項目
    Action,
    /// オンとオフを切り替える項目。値はオンかどうか。オンの項目は選ばれた見た目になる
    Checkbox(bool),
    /// 同じリストの中から 1 つを選ぶ項目。値は選ばれているかどうか。選ばれている項目は選ばれた見た目になる
    Radio(bool),
}

/// [`List`] の項目
///
/// 見出しの文を中心に、上の小さな文 (overline)・下の補足の文・先頭と末尾のアイコン・末尾の補足の文を並べる。
/// 高さは文の行の数 (見出しに overline と補足の文のどちらかが加わると 2 行、両方なら 3 行) に応じた最小の高さを持つ。
/// 補足の文が折り返しても 3 行の高さにはしない
///
/// 操作できる項目 ([`ListItemKind::Text`] 以外) は、マウスのクリック・支援技術からのクリック操作・
/// フォーカス中の Space で [`ListItem::on_click`] のハンドラを呼ぶ。Enter では [`ListItemKind::Action`] の項目だけが呼ぶ。
/// 操作できる項目のアクセシブルな名前は、項目の文をつなげたものになる。
/// 幅は [`Styled`] で付ける。指定しない場合はリストの幅に広がる
pub struct ListItem {
    id: ElementId,
    headline: SharedString,
    kind: ListItemKind,
    overline: Option<SharedString>,
    supporting_text: Option<SharedString>,
    trailing_supporting_text: Option<SharedString>,
    leading_icon: Option<StyledIcon>,
    trailing_icon: Option<StyledIcon>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    focus_handle: Option<FocusHandle>,
    style: StyleRefinement,
}

impl ListItem {
    /// 見出しの文が `headline` の、[`ListItemKind::Text`] の項目を作る
    ///
    /// `id` はリストの中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, headline: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            headline: headline.into(),
            kind: ListItemKind::default(),
            overline: None,
            supporting_text: None,
            trailing_supporting_text: None,
            leading_icon: None,
            trailing_icon: None,
            disabled: false,
            on_click: None,
            focus_handle: None,
            style: StyleRefinement::default(),
        }
    }

    /// 項目の種類を設定する。既定は [`ListItemKind::Text`]
    ///
    /// 支援技術には、操作できる項目を種類に応じて role `button`、`checkbox`、`radio` として公開し、
    /// 切り替える項目はその状態を公開する
    pub fn kind(mut self, kind: ListItemKind) -> Self {
        self.kind = kind;
        self
    }

    /// 見出しの上に表示する小さな文を設定する
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(text.into());
        self
    }

    /// 見出しの下に表示する補足の文を設定する
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 末尾に表示する短い補足の文を設定する。末尾のアイコンと両方を設定した場合はアイコンを表示する
    pub fn trailing_supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing_supporting_text = Some(text.into());
        self
    }

    /// 先頭に表示するアイコンを設定する
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// 末尾に表示するアイコンを設定する
    pub fn trailing_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は無効の色で表示され、操作されてもハンドラを呼ばず、Tab や矢印キーでも到達できない
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// [`ListItemKind::Text`] の項目では呼ばない。
    /// 切り替える項目の状態を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ListItem::kind`] に渡すこと
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab で到達できるかはリストが決める。
    /// 指定しない場合は項目が内部でフォーカスを持つ。[`ListItemKind::Text`] の項目では使わない
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, list_id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.focus_handle.clone().unwrap_or_else(|| {
            // 項目の id はリストの中でだけ一意なので、リストの id で分ける
            window.with_element_namespace(list_id.clone(), |window| {
                util::keyed_focus_handle(&self.id, window, cx)
            })
        })
    }

    /// 見た目を付けた headless の項目にする
    fn into_headless(
        self,
        theme: &SharedTheme,
        focus_handle: &FocusHandle,
        window: &Window,
        cx: &App,
    ) -> HeadlessListItem {
        let interactive = self.kind != ListItemKind::Text;
        let selected = matches!(
            self.kind,
            ListItemKind::Checkbox(true) | ListItemKind::Radio(true)
        );
        let disabled = self.disabled;
        let focus_visible =
            interactive && gpui_headless::is_focus_visible(focus_handle, window, cx);

        // `ListItemColors` の各色。無効なときは、選ばれていても無効の色にする
        let with_opacity = |role: ColorRole, opacity: f32| theme.color(role).opacity(opacity);
        let [container, content, leading, trailing, overline, supporting] = if disabled {
            [
                theme.color(list::ITEM_CONTAINER_COLOR),
                with_opacity(
                    list::ITEM_DISABLED_LABEL_TEXT_COLOR,
                    list::ITEM_DISABLED_LABEL_TEXT_OPACITY,
                ),
                with_opacity(
                    list::ITEM_DISABLED_LEADING_ICON_COLOR,
                    list::ITEM_DISABLED_LEADING_ICON_OPACITY,
                ),
                with_opacity(
                    list::ITEM_DISABLED_TRAILING_ICON_COLOR,
                    list::ITEM_DISABLED_TRAILING_ICON_OPACITY,
                ),
                with_opacity(
                    list::ITEM_DISABLED_OVERLINE_COLOR,
                    list::ITEM_DISABLED_OVERLINE_OPACITY,
                ),
                with_opacity(
                    list::ITEM_DISABLED_SUPPORTING_TEXT_COLOR,
                    list::ITEM_DISABLED_SUPPORTING_TEXT_OPACITY,
                ),
            ]
        } else if selected {
            [
                list::ITEM_SELECTED_CONTAINER_COLOR,
                list::ITEM_SELECTED_LABEL_TEXT_COLOR,
                list::ITEM_SELECTED_LEADING_ICON_COLOR,
                list::ITEM_SELECTED_TRAILING_ICON_COLOR,
                list::ITEM_SELECTED_OVERLINE_COLOR,
                list::ITEM_SELECTED_SUPPORTING_TEXT_COLOR,
            ]
            .map(|role| theme.color(role))
        } else {
            [
                list::ITEM_CONTAINER_COLOR,
                list::ITEM_LABEL_TEXT_COLOR,
                list::ITEM_LEADING_ICON_COLOR,
                list::ITEM_TRAILING_ICON_COLOR,
                list::ITEM_OVERLINE_COLOR,
                list::ITEM_SUPPORTING_TEXT_COLOR,
            ]
            .map(|role| theme.color(role))
        };

        // `shapeForInteraction`。押している間の形は、押している状態を受け取れないため扱わない
        let shape = if selected {
            list::ITEM_SELECTED_CONTAINER_EXPRESSIVE_SHAPE
        } else if focus_visible {
            list::ITEM_FOCUSED_CONTAINER_EXPRESSIVE_SHAPE
        } else {
            list::ITEM_CONTAINER_EXPRESSIVE_SHAPE
        };
        // 角丸の半径は描画時の大きさで決まるが、hover の形は大きさを知らずに決めるため、割合でない形の半径を使う
        let hovered_radius = theme
            .corner_radii(
                list::ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE,
                size(px(f32::MAX), px(f32::MAX)),
            )
            .top_left;
        // 状態レイヤーは内容の色を重ねる (Compose の ripple は LocalContentColor を使う)
        let state_layer = |opacity: f32| -> Hsla { container.blend(content.opacity(opacity)) };
        let layer =
            div()
                .absolute()
                .inset_0()
                .bg(container)
                .when(interactive && !disabled, |this| {
                    let this = this.when(focus_visible, |this| {
                        this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                    });
                    // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                    this.group_hover(GROUP, move |style| {
                        let style = style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY));
                        if selected || focus_visible {
                            style
                        } else {
                            style.rounded(hovered_radius)
                        }
                    })
                });
        let ring = focus_visible.then(|| {
            let ring = div()
                .absolute()
                .inset_0()
                .border(dp(focus_ring::WIDTH))
                .border_color(theme.color(focus_ring::COLOR));
            Shaped::new(ring, theme.clone(), shape).outset(-dp(focus_ring::INWARD_OFFSET))
        });

        let icon = |icon: StyledIcon, icon_size, color| {
            svg()
                .data(icon.svg().as_bytes())
                .flex_none()
                .size(dp(icon_size))
                .text_color(color)
        };
        // 文字列をそのまま子にしたテキストは支援技術に公開されず、操作の対象の名前にもならないため、label として公開する
        let text = |id: &'static str, text: SharedString, scale, color| {
            let mut element = StaticText::new(id, text.clone())
                .text_color(color)
                .child(text);
            element.text_style().refine(&theme.text_style(scale));
            element
        };
        let lines =
            1 + usize::from(self.overline.is_some()) + usize::from(self.supporting_text.is_some());
        let min_height = match lines {
            1 => list::ITEM_ONE_LINE_CONTAINER_HEIGHT,
            2 => list::ITEM_TWO_LINE_CONTAINER_HEIGHT,
            _ => list::ITEM_THREE_LINE_CONTAINER_HEIGHT,
        };
        let leading = self.leading_icon.map(|leading_icon| {
            div()
                .flex_none()
                .pr(dp(list::ITEM_BETWEEN_SPACE))
                .child(icon(leading_icon, list::ITEM_LEADING_ICON_SIZE, leading))
        });
        let trailing = match (self.trailing_icon, self.trailing_supporting_text) {
            (Some(trailing_icon), _) => Some(
                icon(trailing_icon, list::ITEM_TRAILING_ICON_SIZE, trailing).into_any_element(),
            ),
            (None, Some(trailing_text)) => Some(
                text(
                    "trailing-supporting-text",
                    trailing_text,
                    list::ITEM_TRAILING_SUPPORTING_TEXT_FONT,
                    trailing,
                )
                .into_any_element(),
            ),
            (None, None) => None,
        }
        .map(|content| {
            div()
                .flex_none()
                .pl(dp(list::ITEM_BETWEEN_SPACE))
                .child(content)
        });
        let body = div()
            .flex()
            .flex_col()
            .flex_grow_1()
            .min_w_0()
            .children(self.overline.map(|overline_text| {
                text(
                    "overline",
                    overline_text,
                    list::ITEM_OVERLINE_FONT,
                    overline,
                )
            }))
            .child(text(
                "headline",
                self.headline,
                list::ITEM_LABEL_TEXT_FONT,
                content,
            ))
            .children(self.supporting_text.map(|supporting_text| {
                text(
                    "supporting-text",
                    supporting_text,
                    list::ITEM_SUPPORTING_TEXT_FONT,
                    supporting,
                )
            }));

        let mut item = HeadlessListItem::new(self.id)
            .kind(match self.kind {
                ListItemKind::Text => HeadlessListItemKind::Text,
                ListItemKind::Action => HeadlessListItemKind::Action,
                ListItemKind::Checkbox(checked) => HeadlessListItemKind::Checkbox(if checked {
                    CheckedState::Checked
                } else {
                    CheckedState::Unchecked
                }),
                ListItemKind::Radio(selected) => HeadlessListItemKind::Radio(selected),
            })
            .disabled(disabled)
            .track_focus(focus_handle)
            .group(GROUP)
            .relative()
            .flex()
            .min_h(dp(min_height))
            .pl(dp(list::ITEM_LEADING_SPACE))
            .pr(dp(list::ITEM_TRAILING_SPACE))
            .pt(dp(list::ITEM_TOP_SPACE))
            .pb(dp(list::ITEM_BOTTOM_SPACE))
            .child(Shaped::new(layer, theme.clone(), shape))
            .children(leading)
            .child(body)
            .children(trailing)
            .children(ring);
        // `ListItemDefaults.verticalAlignment`。3 行の高さは中央にそろえる境目を超えるため、上にそろえる
        item = if lines >= 3 {
            item.items_start()
        } else {
            item.items_center()
        };
        item.style().refine(&self.style);
        if let Some(on_click) = self.on_click {
            item = item.on_click(on_click);
        }
        item
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point};

    use super::*;

    /// 左上の (0, 0) に置いた幅 300 のリスト。項目は `items` が返す
    struct ListView {
        items: fn(Rc<RefCell<Vec<&'static str>>>) -> Vec<ListItem>,
        clicks: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for ListView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                List::new("list")
                    .w(px(300.))
                    .items((self.items)(self.clicks.clone())),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        items: fn(Rc<RefCell<Vec<&'static str>>>) -> Vec<ListItem>,
    ) -> (gpui::Entity<ListView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ListView {
                items,
                clicks: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        // ウィンドウをアクティブにすると最初のマウスの移動の後に位置が (0, 0) に戻るため、済むまで待つ
        cx.run_until_parked();
        (view, cx)
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 背景を `color` で塗った、幅 300 の矩形の y と高さ
    fn rows(cx: &mut VisualTestContext, color: Hsla) -> Vec<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| {
                    quad.background == color.into() && quad.bounds.size.width.0 / scale == 300.
                })
                .map(|quad| {
                    (
                        quad.bounds.origin.y.0 / scale,
                        quad.bounds.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    /// 枠線を `color` で描いた矩形の y と高さ。gpui は 1 つの枠線を複数の矩形に分けて描くことがあるため、重複を除く
    fn bordered(cx: &mut VisualTestContext, color: Hsla) -> Vec<(f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut rows: Vec<(f32, f32)> = window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_color == color && quad.border_widths.top.0 > 0.)
                .map(|quad| {
                    (
                        quad.bounds.origin.y.0 / scale,
                        quad.bounds.size.height.0 / scale,
                    )
                })
                .collect();
            rows.dedup();
            rows
        })
    }

    fn clicks(view: &gpui::Entity<ListView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.clicks.borrow().clone())
    }

    fn recording(id: &'static str, clicks: &Rc<RefCell<Vec<&'static str>>>) -> ListItem {
        let clicks = clicks.clone();
        ListItem::new(id, id).on_click(move |_, _, _| clicks.borrow_mut().push(id))
    }

    #[gpui::test]
    fn 項目の高さは文の行の数で決まる(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |_| {
            vec![
                ListItem::new("one", "One"),
                ListItem::new("two", "Two").supporting_text("Supporting"),
                ListItem::new("three", "Three")
                    .overline("Overline")
                    .supporting_text("Supporting"),
            ]
        });
        let surface = color(cx, list::ITEM_CONTAINER_COLOR);
        assert_eq!(
            rows(cx, surface),
            vec![
                (0., list::ITEM_ONE_LINE_CONTAINER_HEIGHT.0),
                (56., list::ITEM_TWO_LINE_CONTAINER_HEIGHT.0),
                (128., list::ITEM_THREE_LINE_CONTAINER_HEIGHT.0),
            ]
        );
    }

    #[gpui::test]
    fn 選ばれている項目とオンの項目は選ばれた容器の色で塗る(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {
            vec![
                ListItem::new("radio", "Radio").kind(ListItemKind::Radio(true)),
                ListItem::new("checkbox", "Checkbox").kind(ListItemKind::Checkbox(true)),
                ListItem::new("off", "Off").kind(ListItemKind::Checkbox(false)),
                ListItem::new("disabled", "Disabled")
                    .kind(ListItemKind::Radio(true))
                    .disabled(true),
            ]
        });
        cx.simulate_mouse_move(point(px(700.), px(500.)), None, Modifiers::none());
        let selected = color(cx, list::ITEM_SELECTED_CONTAINER_COLOR);
        let surface = color(cx, list::ITEM_CONTAINER_COLOR);
        assert_eq!(rows(cx, selected), vec![(0., 56.), (56., 56.)]);
        assert_eq!(
            rows(cx, surface),
            vec![(112., 56.), (168., 56.)],
            "無効な項目は選ばれていても選ばれた容器の色にしないこと"
        );
    }

    #[gpui::test]
    fn 操作できる項目だけがクリックでハンドラを呼ぶ(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, |clicks| {
            vec![
                recording("text", &clicks),
                recording("action", &clicks).kind(ListItemKind::Action),
                recording("disabled", &clicks)
                    .kind(ListItemKind::Action)
                    .disabled(true),
            ]
        });
        for row in 0..3 {
            cx.simulate_click(
                point(px(150.), px(28. + 56. * row as f32)),
                Modifiers::none(),
            );
        }
        assert_eq!(clicks(&view, cx), vec!["action"]);
    }

    #[gpui::test]
    fn 操作できる項目だけが_hover_の状態レイヤーと形を持つ(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {
            vec![
                ListItem::new("text", "Text"),
                ListItem::new("action", "Action").kind(ListItemKind::Action),
            ]
        });
        let surface = color(cx, list::ITEM_CONTAINER_COLOR);
        let content = color(cx, list::ITEM_LABEL_TEXT_COLOR);
        let hovered = surface.blend(content.opacity(state::HOVER_STATE_LAYER_OPACITY));

        cx.simulate_mouse_move(point(px(150.), px(28.)), None, Modifiers::none());
        assert!(rows(cx, hovered).is_empty());

        cx.simulate_mouse_move(point(px(150.), px(84.)), None, Modifiers::none());
        assert_eq!(rows(cx, hovered), vec![(56., 56.)]);
        let radius = cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.background == hovered.into())
                .map(|quad| quad.corner_radii.top_left.0 / scale)
        });
        let expected = cx.update(|_, cx| {
            cx.md3_theme()
                .corner_radii(
                    list::ITEM_HOVERED_CONTAINER_EXPRESSIVE_SHAPE,
                    gpui::size(px(300.), px(56.)),
                )
                .top_left
        });
        assert_eq!(radius, Some(f32::from(expected)));
    }

    #[gpui::test]
    fn キーボードのフォーカスでリングと状態レイヤーを描く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {
            vec![
                ListItem::new("text", "Text"),
                ListItem::new("action", "Action").kind(ListItemKind::Action),
                ListItem::new("radio", "Radio").kind(ListItemKind::Radio(false)),
            ]
        });
        cx.simulate_mouse_move(point(px(700.), px(500.)), None, Modifiers::none());
        let ring = color(cx, focus_ring::COLOR);
        assert!(bordered(cx, ring).is_empty());
        cx.simulate_keystrokes("tab");
        assert_eq!(bordered(cx, ring), vec![(56., 56.)]);

        let surface = color(cx, list::ITEM_CONTAINER_COLOR);
        let content = color(cx, list::ITEM_LABEL_TEXT_COLOR);
        let focused = surface.blend(content.opacity(state::FOCUS_STATE_LAYER_OPACITY));
        assert_eq!(rows(cx, focused), vec![(56., 56.)]);
        cx.simulate_keystrokes("down");
        assert_eq!(rows(cx, focused), vec![(112., 56.)]);
    }
}
