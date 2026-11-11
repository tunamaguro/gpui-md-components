// 寸法と色の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/RadioButton.kt
//         (`RadioButtonImpl`、`RadioButtonDefaults`、`RadioButtonColors`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// hover とフォーカスでの円の色の切り替えと、フォーカスリングの大きさは Compose にないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: radio/internal/_radio.scss @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のラジオボタン

use gpui::{
    AnyElement, App, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, div, prelude::FluentBuilder,
};
use gpui_headless::radio::{Radio as HeadlessRadio, RadioGroup as HeadlessRadioGroup};

use crate::util;
use md3_theme::md3_tokens::component::radio_button;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, Theme, ThemeExt, dp};

/// [`RadioButton`] の中の [`RadioButtonIndicator`] が、ラジオボタンの hover を知るための group 名
const GROUP: &str = "md3-radio-button";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// グループの子。項目と、項目でない要素を渡された順に並べる
enum GroupChild<I> {
    Item(I),
    Other(AnyElement),
}

/// 1 つだけを選ぶ、MD3 のラジオボタンのグループ
///
/// 状態 (選ばれている値) は呼び出し側が持ち、[`RadioGroup::new`] に毎回渡す。グループ自身は状態を変えない。
/// 項目は [`RadioGroup::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初の有効な項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。有効な項目が無ければ Tab で入らない
/// - 上下左右の矢印キーで前後の有効な項目へフォーカスを移し、移った先の項目を選ぶよう求める。
///   端では反対の端に戻る
/// - フォーカス中の Space とマウスのクリックで、項目を選ぶよう求める。Enter では求めない
/// - 操作で選択を解除する手段は持たない
///
/// 支援技術には role `radiogroup` として公開され、[`RadioGroup::aria_label`] がアクセシブルな名前になる。
/// グループ自身は見た目を持たない。項目の並べ方は [`Styled`] で呼び出し側が付ける
///
/// ```ignore
/// RadioGroup::new("fruit", self.fruit)
///     .aria_label("果物")
///     .on_change(cx.listener(|this, fruit: &Fruit, _, cx| {
///         this.fruit = Some(*fruit);
///         cx.notify();
///     }))
///     .flex()
///     .flex_col()
///     .item(RadioButton::new("apple", Fruit::Apple).child(gpui::text!("りんご")))
///     .item(RadioButton::new("pear", Fruit::Pear).child(gpui::text!("なし")))
/// ```
#[derive(IntoElement)]
pub struct RadioGroup<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    aria_label: Option<SharedString>,
    children: Vec<GroupChild<RadioButton<T>>>,
    style: StyleRefinement,
}

impl<T: PartialEq + 'static> RadioGroup<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示するグループを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            on_change: None,
            aria_label: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目と無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`RadioGroup::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: RadioButton<T>) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = RadioButton<T>>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for RadioGroup<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let children: Vec<_> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(item) => {
                    let checked = selected.take_if(|selected| **selected == item.value);
                    GroupChild::Item(item.into_headless(checked.is_some(), &self.id, window, cx))
                }
                GroupChild::Other(element) => GroupChild::Other(element),
            })
            .collect();

        let mut group = HeadlessRadioGroup::new(self.id, self.selected)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change));
        *group.style() = self.style;
        children
            .into_iter()
            .fold(group, |group, child| match child {
                GroupChild::Item(item) => group.item(item),
                GroupChild::Other(element) => group.child(element),
            })
    }
}

impl<T: PartialEq + 'static> Styled for RadioGroup<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl<T: PartialEq + 'static> ParentElement for RadioGroup<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

/// [`RadioGroup`] の項目になる、MD3 のラジオボタン
///
/// [`RadioGroup::item`] に渡して使う。項目が表す値が、グループの選ばれている値と等しいときに選ばれた状態になる。
/// [`RadioButtonIndicator`] の後に子の内容を並べ、項目のどこを押しても選ぶよう求める。
/// 項目の hover とフォーカスは [`RadioButtonIndicator`] に表示される
///
/// 支援技術には role `radio` として公開される。アクセシブルな名前は、子のうち支援技術に公開されるテキストか
/// [`RadioButton::aria_label`] で与えること。文字列を直接子にしても支援技術には公開されないため、
/// 名前にするテキストは [`gpui::text!`] で作ること
///
/// 押せず、フォーカスも受けない見た目だけのラジオボタンには [`RadioButtonIndicator`] を使う
///
/// 子を持たない項目の大きさは状態レイヤーの 40dp 四方の箱で、この箱が押せる範囲になる。
/// [`Styled`] で指定したスタイルは、[`RadioButtonIndicator`] と子を横に並べる箱に付く
pub struct RadioButton<T> {
    id: ElementId,
    value: T,
    disabled: bool,
    aria_label: Option<SharedString>,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
    children: Vec<AnyElement>,
    style: StyleRefinement,
}

impl<T> RadioButton<T> {
    /// `value` を表す項目を作る
    ///
    /// `id` と `value` はグループの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`RadioButton::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T) -> Self {
        Self {
            id: id.into(),
            value,
            disabled: false,
            aria_label: None,
            focus_handle: None,
            theme: None,
            children: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は無効の色で描かれ、押されても選ぶよう求めず、Tab や矢印キーでも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// Tab で到達できるかはグループが決める。指定しない場合は項目が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// `checked` の見た目を持つ、グループ `group_id` の項目にする
    fn into_headless(
        self,
        checked: bool,
        group_id: &ElementId,
        window: &mut Window,
        cx: &mut App,
    ) -> HeadlessRadio<T> {
        let focus_handle = self.focus_handle.unwrap_or_else(|| {
            // 項目の id はグループの中でだけ一意なので、グループの id で分ける
            window.with_element_namespace(group_id.clone(), |window| {
                util::keyed_focus_handle(&self.id, window, cx)
            })
        });
        let mut indicator = RadioButtonIndicator::new(checked).disabled(self.disabled);
        indicator.theme = self.theme;
        indicator.interaction = Some((GROUP.into(), focus_handle.clone()));

        // 利用者が Styled で指定した値で上書きできるよう、既定を先に付ける
        let mut radio = HeadlessRadio::new(self.id, self.value)
            .flex()
            .items_center();
        radio.style().refine(&self.style);
        if let Some(label) = self.aria_label {
            radio = radio.aria_label(label);
        }
        radio
            .disabled(self.disabled)
            .track_focus(&focus_handle)
            .group(GROUP)
            .child(indicator)
            .children(self.children)
    }
}

impl<T> Styled for RadioButton<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl<T> ParentElement for RadioButton<T> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// MD3 のラジオボタンの見た目
///
/// 状態を表示するだけで、押せず、フォーカスを受けず、支援技術にも公開されない。
/// 単独で置いて状態を伝える場合は、状態をテキストなどで支援技術にも伝えること。
/// 単独で置いた場合は hover とフォーカスを表示しない
///
/// 大きさは状態レイヤーの 40dp 四方の箱で、フォーカスリングは箱の外側にはみ出して描く
///
/// [`Styled`] で指定したスタイルは外側の箱に付く
#[derive(IntoElement)]
pub struct RadioButtonIndicator {
    selected: bool,
    disabled: bool,
    theme: Option<SharedTheme>,
    /// hover を知る group の名前と、フォーカスを表示するためのフォーカス
    interaction: Option<(SharedString, FocusHandle)>,
    outer: gpui::Div,
}

impl RadioButtonIndicator {
    /// `selected` なら選ばれた状態を表示する見た目を作る
    pub fn new(selected: bool) -> Self {
        Self {
            selected,
            disabled: false,
            theme: None,
            interaction: None,
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
        }
    }

    /// 無効の色で描く。`false` を渡すと有効の色に戻す
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for RadioButtonIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let (group, focus_handle) = self.interaction.unzip();
        let colors = Colors::resolve(
            &theme,
            self.selected,
            self.disabled,
            focus_handle
                .as_ref()
                .is_some_and(|focus_handle| focus_handle.is_focused(window)),
            focus_handle.as_ref().is_some_and(|focus_handle| {
                gpui_headless::is_focus_visible(focus_handle, window, cx)
            }),
        );

        let stroke_width = dp(supplement::radio_button::STROKE_WIDTH);
        // Compose は外側の円を、線の外側の縁がアイコンの大きさに収まる半径で描く
        let circle = div()
            .flex()
            .items_center()
            .justify_center()
            .size(dp(radio_button::ICON_SIZE))
            .rounded_full()
            .border(stroke_width)
            .border_color(colors.icon)
            .when_some(
                group.clone().zip(colors.hover_icon),
                |this, (group, icon)| this.group_hover(group, |style| style.border_color(icon)),
            )
            .when(self.selected, |this| {
                this.child(
                    div()
                        // Compose は内側の円を、半径から線の太さの半分を引いて塗る
                        .size(dp(supplement::radio_button::DOT_SIZE) - stroke_width)
                        .rounded_full()
                        .bg(colors.icon),
                )
            });

        let state_size = dp(radio_button::STATE_LAYER_SIZE);
        let state_layer = div()
            .absolute()
            .top_0()
            .left_0()
            .size(state_size)
            .rounded_full()
            .when_some(colors.focus_state_layer, |this, layer| this.bg(layer))
            .when_some(
                group.zip(colors.hover_state_layer),
                |this, (group, layer)| this.group_hover(group, |style| style.bg(layer)),
            );

        let indicator = div()
            .relative()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .size(state_size)
            .child(state_layer)
            .child(circle)
            .when_some(colors.focus_ring, |this, ring| {
                // Material Web は 44px の円 (状態レイヤーから外側へ 2px) の外側に、太さ 3px のリングを描く
                let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                this.child(
                    div()
                        .absolute()
                        .top(-outset)
                        .left(-outset)
                        .size(state_size + outset * 2.0)
                        .rounded_full()
                        .border(dp(focus_ring::WIDTH))
                        .border_color(ring),
                )
            });

        self.outer.child(indicator)
    }
}

impl Styled for RadioButtonIndicator {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

/// 状態から決まる描画の色
struct Colors {
    /// 外側の円の線と内側の円
    icon: Hsla,
    /// hover 中の外側の円の線。`None` は hover で変えない
    hover_icon: Option<Hsla>,
    /// フォーカスの状態レイヤー。`None` は描かない
    focus_state_layer: Option<Hsla>,
    /// hover の状態レイヤー。`None` は描かない
    hover_state_layer: Option<Hsla>,
    /// フォーカスリング。`None` は描かない
    focus_ring: Option<Hsla>,
}

impl Colors {
    // 有効・無効と選択の色は `RadioButtonColors.radioColor`。hover とフォーカスの色は Compose が使わないため、
    // Material Web の `:host(:hover) .icon` と `:host(:focus-within) .icon` に合わせる。
    // 選択中の hover とフォーカスの色は選択中の色と同じため切り替えない
    fn resolve(
        theme: &Theme,
        selected: bool,
        disabled: bool,
        focused: bool,
        focus_visible: bool,
    ) -> Self {
        let icon = match (selected, disabled) {
            (true, false) => theme.color(radio_button::SELECTED_ICON_COLOR),
            (false, false) => theme.color(if focused {
                // Material Web はリングの有無にかかわらず、フォーカスがあればこの色にする (`:focus-within`)
                radio_button::UNSELECTED_FOCUS_ICON_COLOR
            } else {
                radio_button::UNSELECTED_ICON_COLOR
            }),
            (true, true) => theme
                .color(radio_button::DISABLED_SELECTED_ICON_COLOR)
                .opacity(radio_button::DISABLED_SELECTED_ICON_OPACITY),
            (false, true) => theme
                .color(radio_button::DISABLED_UNSELECTED_ICON_COLOR)
                .opacity(radio_button::DISABLED_UNSELECTED_ICON_OPACITY),
        };
        let (hover_layer, focus_layer) = if selected {
            (
                supplement::radio_button::SELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::radio_button::SELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        } else {
            (
                supplement::radio_button::UNSELECTED_HOVER_STATE_LAYER_COLOR,
                supplement::radio_button::UNSELECTED_FOCUS_STATE_LAYER_COLOR,
            )
        };
        Self {
            icon,
            hover_icon: (!selected && !disabled)
                .then(|| theme.color(radio_button::UNSELECTED_HOVER_ICON_COLOR)),
            focus_state_layer: (!disabled && focus_visible).then(|| {
                theme
                    .color(focus_layer)
                    .opacity(state::FOCUS_STATE_LAYER_OPACITY)
            }),
            hover_state_layer: (!disabled).then(|| {
                theme
                    .color(hover_layer)
                    .opacity(state::HOVER_STATE_LAYER_OPACITY)
            }),
            focus_ring: focus_visible.then(|| theme.color(focus_ring::COLOR)),
        }
    }
}

/// [`RadioGroup`] と [`RadioButton`] と [`RadioButtonIndicator`] を組み合わせたときの振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        Bounds, Context, Entity, Modifiers, Pixels, Render, TestAppContext, VisualTestContext,
        point, px,
    };

    use super::*;

    const VALUES: [&str; 3] = ["apple", "pear", "plum"];

    struct GroupView {
        selected: Option<&'static str>,
        disabled: [bool; 3],
        changes: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for GroupView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            // 項目は高さ 40px で、上から 0px、40px、80px に並ぶ。円は左端の 40px 四方、テキストはその右に並ぶ
            RadioGroup::new("fruit", self.selected)
                .aria_label("Fruit")
                .on_change(cx.listener(|this, value: &&'static str, _, cx| {
                    this.changes.borrow_mut().push(*value);
                    this.selected = Some(*value);
                    cx.notify();
                }))
                .flex()
                .flex_col()
                .w(px(200.))
                .items(VALUES.iter().enumerate().map(|(ix, value)| {
                    RadioButton::new(*value, *value)
                        .disabled(self.disabled[ix])
                        .child(gpui::text!(*value))
                }))
        }
    }

    fn setup<'a>(
        cx: &'a mut TestAppContext,
        selected: Option<&'static str>,
        disabled: [bool; 3],
    ) -> (Entity<GroupView>, &'a mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            GroupView {
                selected,
                disabled,
                changes: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        (view, cx)
    }

    fn changes(view: &Entity<GroupView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    /// 描いた矩形の位置と大きさ (論理ピクセル)
    fn quad_bounds(cx: &mut VisualTestContext) -> Vec<Bounds<Pixels>> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .map(|quad| quad.bounds.map(|value| px(value.0 / scale)))
                .collect()
        })
    }

    /// 上から `row` 番目 (0 始まり) の項目に、選ばれていることを示す内側の円を描いたか
    fn dot_painted(cx: &mut VisualTestContext, row: usize) -> bool {
        // 内側の円は直径 10px で、40px 四方の箱の中央に描く
        let origin = point(px(15.), px(15. + 40. * row as f32));
        quad_bounds(cx)
            .into_iter()
            .any(|bounds| bounds.origin == origin && bounds.size.width == px(10.))
    }

    /// 上から `row` 番目 (0 始まり) の項目に、フォーカスリングを描いたか
    fn ring_painted(cx: &mut VisualTestContext, row: usize) -> bool {
        // フォーカスリングは 40px の箱の外側 5px に描く
        let origin = point(px(-5.), px(-5. + 40. * row as f32));
        quad_bounds(cx)
            .into_iter()
            .any(|bounds| bounds.origin == origin && bounds.size.width == px(50.))
    }

    #[gpui::test]
    fn 選ばれている項目だけに内側の円を描く(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, Some("pear"), [false; 3]);
        assert_eq!(
            [0, 1, 2].map(|row| dot_painted(cx, row)),
            [false, true, false]
        );
    }

    #[gpui::test]
    fn テキストを押しても円を押しても一度だけ選ぶよう求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, None, [false; 3]);
        cx.simulate_click(point(px(100.), px(60.)), Modifiers::none());
        assert_eq!(changes(&view, cx), vec!["pear"], "テキスト側");
        assert!(dot_painted(cx, 1), "選んだ項目に内側の円を描くこと");

        cx.simulate_click(point(px(20.), px(100.)), Modifiers::none());
        assert_eq!(changes(&view, cx), vec!["pear", "plum"], "円の側");
        assert_eq!(
            [0, 1, 2].map(|row| dot_painted(cx, row)),
            [false, false, true]
        );
    }

    #[gpui::test]
    fn tab_で選ばれている項目に入りフォーカスリングを描く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("pear"), [false; 3]);
        assert_eq!(
            [0, 1, 2].map(|row| ring_painted(cx, row)),
            [false; 3],
            "フォーカスの前はリングを描かないこと"
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(
            [0, 1, 2].map(|row| ring_painted(cx, row)),
            [false, true, false]
        );
        assert!(changes(&view, cx).is_empty());
    }

    #[gpui::test]
    fn 矢印キーで移った先を選ぶよう求めフォーカスリングも移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, Some("apple"), [false, true, false]);
        cx.simulate_keystrokes("tab down");
        assert_eq!(changes(&view, cx), vec!["plum"], "無効な項目を飛ばすこと");
        assert_eq!(
            [0, 1, 2].map(|row| ring_painted(cx, row)),
            [false, false, true]
        );
        assert_eq!(
            [0, 1, 2].map(|row| dot_painted(cx, row)),
            [false, false, true]
        );
    }

    #[gpui::test]
    fn 無効な項目は押しても選ぶよう求めない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, None, [false, true, false]);
        cx.simulate_click(point(px(100.), px(60.)), Modifiers::none());
        assert!(changes(&view, cx).is_empty());
    }

    /// 左上が (`x`, `y`) で幅が `width` の矩形の、枠線の色と塗り
    fn quad_colors(
        cx: &mut VisualTestContext,
        (x, y, width): (f32, f32, f32),
    ) -> Option<(Hsla, gpui::Background)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window.painted_quads().into_iter().find_map(|quad| {
                let bounds = quad.bounds.map(|value| value.0 / scale);
                ((bounds.origin.x, bounds.origin.y, bounds.size.width) == (x, y, width))
                    .then_some((quad.border_color, quad.background))
            })
        })
    }

    /// 外側の円 (20px) と内側の円 (10px) と状態レイヤー (40px) の、上から 0 番目の項目での位置
    const CIRCLE: (f32, f32, f32) = (10., 10., 20.);
    const DOT: (f32, f32, f32) = (15., 15., 10.);
    const STATE_LAYER: (f32, f32, f32) = (0., 0., 40.);

    // 期待する色は RadioButtonTokens.kt と _md-comp-radio-button.scss、StateTokens.kt の値を直接書く
    #[gpui::test]
    fn 選択と無効の色を_md3_のロールから取る(cx: &mut TestAppContext) {
        use md3_theme::md3_color::ColorRole;
        for (selected, disabled, role, opacity) in [
            (false, false, ColorRole::OnSurfaceVariant, 1.),
            (true, false, ColorRole::Primary, 1.),
            (false, true, ColorRole::OnSurface, 0.38),
            (true, true, ColorRole::OnSurface, 0.38),
        ] {
            let (_, cx) = setup(cx, selected.then_some("apple"), [disabled, false, false]);
            // ウィンドウをアクティブにした直後は (0, 0) の項目が hover の見た目になり、外への最初の移動では消えないため、
            // 一度中に入れてから項目の外に置く
            cx.simulate_mouse_move(point(px(100.), px(20.)), None, Modifiers::none());
            cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
            let expected = cx.update(|_, cx| cx.md3_theme().color(role).opacity(opacity));
            let case = format!("selected={selected} disabled={disabled}");
            assert_eq!(
                quad_colors(cx, CIRCLE).map(|(border, _)| border),
                Some(expected),
                "外側の円 {case}"
            );
            assert_eq!(
                quad_colors(cx, DOT).map(|(_, background)| background),
                selected.then_some(expected.into()),
                "内側の円 {case}"
            );
            assert_eq!(
                quad_colors(cx, STATE_LAYER).map(|(_, background)| background),
                None,
                "hover もフォーカスも無ければ状態レイヤーを描かないこと {case}"
            );
        }
    }

    #[gpui::test]
    fn hover_とフォーカスで円と状態レイヤーの色が変わる(
        cx: &mut TestAppContext,
    ) {
        use md3_theme::md3_color::ColorRole;
        for (selected, layer_role) in [(false, ColorRole::OnSurface), (true, ColorRole::Primary)] {
            let (_, cx) = setup(cx, selected.then_some("apple"), [false; 3]);
            let color = |cx: &mut VisualTestContext, role: ColorRole, opacity: f32| {
                cx.update(|_, cx| cx.md3_theme().color(role).opacity(opacity))
            };
            let circle_role = if selected {
                ColorRole::Primary
            } else {
                ColorRole::OnSurface
            };

            cx.simulate_mouse_move(point(px(100.), px(20.)), None, Modifiers::none());
            assert_eq!(
                quad_colors(cx, CIRCLE).map(|(border, _)| border),
                Some(color(cx, circle_role, 1.)),
                "hover 中の外側の円 selected={selected}"
            );
            assert_eq!(
                quad_colors(cx, STATE_LAYER).map(|(_, background)| background),
                Some(color(cx, layer_role, 0.08).into()),
                "hover の状態レイヤー selected={selected}"
            );

            cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
            cx.simulate_keystrokes("tab");
            assert_eq!(
                quad_colors(cx, CIRCLE).map(|(border, _)| border),
                Some(color(cx, circle_role, 1.)),
                "フォーカス中の外側の円 selected={selected}"
            );
            assert_eq!(
                quad_colors(cx, STATE_LAYER).map(|(_, background)| background),
                Some(color(cx, layer_role, 0.1).into()),
                "フォーカスの状態レイヤー selected={selected}"
            );
            assert_eq!(
                quad_colors(cx, (-5., -5., 50.)).map(|(border, _)| border),
                Some(color(cx, ColorRole::Secondary, 1.)),
                "フォーカスリング selected={selected}"
            );
        }
    }

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    /// `element` だけを描いたときの矩形。ポインターは要素の外に置く
    fn painted_quads(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> String {
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
        cx.update(|window, _| format!("{:?}", window.painted_quads()))
    }

    #[gpui::test]
    fn radio_button_は選択と無効を見た目に渡す(cx: &mut TestAppContext) {
        cx.update(crate::init);
        for selected in [false, true] {
            for disabled in [false, true] {
                let button = painted_quads(cx, move || {
                    RadioGroup::new("group", selected.then_some(1))
                        .aria_label("Fruit")
                        .item(
                            RadioButton::new("radio", 1)
                                .aria_label("Apple")
                                .disabled(disabled),
                        )
                        .into_any_element()
                });
                let indicator = painted_quads(cx, move || {
                    RadioButtonIndicator::new(selected)
                        .disabled(disabled)
                        .into_any_element()
                });
                assert_eq!(
                    button, indicator,
                    "selected={selected} disabled={disabled} で同じ見た目になること"
                );
            }
        }
    }
}
