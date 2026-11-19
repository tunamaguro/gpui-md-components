// 項目の形と間隔は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt
//         (`ButtonGroupDefaults` の `ConnectedSpaceBetween`、`connectedLeadingButtonShapes`、
//         `connectedMiddleButtonShapes`、`connectedTrailingButtonShapes`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive の Connected button group

use gpui::{
    App, BorderStyle, Corners, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, prelude::FluentBuilder, quad, transparent_black,
};
use gpui_headless::radio::{Radio as HeadlessRadio, RadioGroup as HeadlessRadioGroup};
use gpui_headless::toggle::{ToggleButton as HeadlessToggleButton, ToggleGroup};

use crate::button::RingSlot;
use crate::util;
use crate::{ToggleButton, ToggleButtonVariant};
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::connected_button_group_small as tokens;
use md3_theme::md3_tokens::focus_ring;
use md3_theme::md3_tokens::shape::{CornerShape, ShapeKey};
use md3_theme::{SharedTheme, ThemeExt, dp};

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;
type ToggleHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// 項目に共通の内容
struct Item {
    id: ElementId,
    label: SharedString,
    icon: Option<StyledIcon>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
}

impl Item {
    fn new(id: ElementId, label: SharedString) -> Self {
        // ラベルは 1 行で描くが、gpui のテキストは改行で行を分けるため、改行を空白に置き換える
        let label = if label.contains('\n') {
            label.lines().collect::<Vec<_>>().join(" ").into()
        } else {
            label
        };
        Self {
            id,
            label,
            icon: None,
            disabled: false,
            focus_handle: None,
        }
    }

    fn focus_handle(&self, group_id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.focus_handle.clone().unwrap_or_else(|| {
            // 項目の id はグループの中でだけ一意なので、グループの id で分ける
            window.with_element_namespace(group_id.clone(), |window| {
                util::keyed_focus_handle(&self.id, window, cx)
            })
        })
    }

    /// 並びの `index` 番目 (全 `count` 個) の項目の見た目を `item` に付ける
    #[expect(
        clippy::too_many_arguments,
        reason = "項目の見た目を決める値をまとめて受け取る"
    )]
    fn decorate<E>(
        self,
        item: E,
        checked: bool,
        index: usize,
        count: usize,
        variant: ToggleButtonVariant,
        focus_handle: &FocusHandle,
        ring: &RingSlot,
        theme: &SharedTheme,
        window: &mut Window,
        cx: &mut App,
    ) -> E
    where
        E: ParentElement + Styled + InteractiveElement,
    {
        // リングは項目の外側に描くため、項目自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(focus_handle, window, cx);
        let mut button = ToggleButton::new(self.id, checked)
            .variant(variant)
            .disabled(self.disabled)
            .child(gpui::text!(self.label));
        if let Some(icon) = self.icon {
            button = button.icon(icon);
        }
        let mut look = button.look(theme, focus_visible);
        // 端の項目は外側の角を丸く、内側の角を小さく丸める。間の項目は Small の形にする。
        // Compose のサンプルと同じく、1 つだけの項目は先頭の形にする。
        // 選ばれるとどの項目も丸い形に変わる (`connectedButtonCheckedShape`)
        let full = ShapeKey::CornerFull.resolve(&theme.settings().shapes);
        let middle = ShapeKey::CornerSmall.resolve(&theme.settings().shapes);
        let inner = tokens::INNER_CORNER_CORNER_SIZE;
        look.corner_shape = Some(match (index == 0, index + 1 == count) {
            (true, _) => CornerShape {
                top_start: full.top_start,
                bottom_start: full.bottom_start,
                top_end: inner,
                bottom_end: inner,
            },
            (false, true) => CornerShape {
                top_start: inner,
                bottom_start: inner,
                top_end: full.top_end,
                bottom_end: full.bottom_end,
            },
            (false, false) => middle,
        });
        // 項目の間は 2dp で、項目の外側にはみ出すリングは隣の項目に隠れるため、並びがすべての項目の後に描く
        look.ring_slot = Some(ring.clone());
        if let Some(morph) = look.morph.as_mut() {
            morph.target = tokens::CONTAINER_SHAPE;
        }
        // 項目はラベルに合わせた幅にし、並びの幅に広げない
        look.apply(item).flex_none()
    }
}

/// `ring` に書かれた矩形の外側にフォーカスリングを描く要素。並びの最後の子に置き、すべての項目の後に描く。
/// ほかの要素と同じく並びの中で描くため、祖先の切り取りの範囲と不透明度が効く
fn paint_ring(ring: RingSlot, theme: SharedTheme) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            let Some((bounds, radii)) = ring.get() else {
                return;
            };
            let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
            window.paint_quad(quad(
                bounds.dilate(outset),
                Corners {
                    top_left: radii.top_left + outset,
                    top_right: radii.top_right + outset,
                    bottom_right: radii.bottom_right + outset,
                    bottom_left: radii.bottom_left + outset,
                },
                transparent_black(),
                dp(focus_ring::WIDTH),
                theme.color(focus_ring::COLOR),
                BorderStyle::Solid,
            ));
        },
    )
    .absolute()
}

/// 1 つだけを選ぶ、MD3 Expressive の Connected button group
///
/// 選択肢を、間を 2dp 空けたトグルボタンとして横に並べる。端の項目は外側の角を丸くし、選ばれた項目は丸い形に変わる。
/// 状態 (選ばれている値) は呼び出し側が持ち、[`SingleChoiceConnectedButtonGroup::new`] に毎回渡す。並び自身は状態を変えない。
/// 項目は [`SingleChoiceConnectedButtonGroup::item`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初の有効な項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。有効な項目が無ければ Tab で入らない
/// - 上下左右の矢印キーで前後の有効な項目へフォーカスを移し、移った先の項目を選ぶよう求める。
///   端では反対の端に戻る
/// - フォーカス中の Space とマウスのクリックで、項目を選ぶよう求める。Enter では求めない
/// - 操作で選択を解除する手段は持たない
///
/// 支援技術には role `radiogroup` として公開され、[`SingleChoiceConnectedButtonGroup::aria_label`] がアクセシブルな名前になる。
/// 項目は role `radio` と選ばれているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルは項目を並べる箱に付く。項目はラベルに合わせた幅で、ラベルは 1 行で描く
#[derive(IntoElement)]
pub struct SingleChoiceConnectedButtonGroup<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    aria_label: Option<SharedString>,
    variant: ToggleButtonVariant,
    items: Vec<ConnectedButton<T>>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> SingleChoiceConnectedButtonGroup<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示する並びを作る。`None` ならどの項目も選ばれていない
    ///
    /// 項目は [`ToggleButtonVariant::Filled`] で描く。`id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            on_change: None,
            aria_label: None,
            variant: ToggleButtonVariant::default(),
            items: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
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
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`SingleChoiceConnectedButtonGroup::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 項目のトグルボタンの種類を設定する
    pub fn variant(mut self, variant: ToggleButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ConnectedButton<T>) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ConnectedButton<T>>) -> Self {
        self.items.extend(items);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for SingleChoiceConnectedButtonGroup<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let count = self.items.len();
        let ring = RingSlot::default();
        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let items: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, ConnectedButton { item, value })| {
                let checked = selected.take_if(|selected| **selected == value).is_some();
                let focus_handle = item.focus_handle(&self.id, window, cx);
                let radio = HeadlessRadio::new(item.id.clone(), value)
                    .disabled(item.disabled)
                    .track_focus(&focus_handle)
                    .aria_label(item.label.clone());
                item.decorate(
                    radio,
                    checked,
                    index,
                    count,
                    self.variant,
                    &focus_handle,
                    &ring,
                    &theme,
                    window,
                    cx,
                )
            })
            .collect();
        let mut group = HeadlessRadioGroup::new(self.id, self.selected)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .flex()
            .items_center()
            .gap(dp(tokens::BETWEEN_SPACE));
        group.style().refine(&self.style);
        group.items(items).child(paint_ring(ring, theme))
    }
}

impl<T: PartialEq + 'static> Styled for SingleChoiceConnectedButtonGroup<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`SingleChoiceConnectedButtonGroup`] の項目
pub struct ConnectedButton<T> {
    item: Item,
    value: T,
}

impl<T> ConnectedButton<T> {
    /// `label` を描き、選ばれると並びに `value` を渡す項目を作る
    ///
    /// `id` は並びの中で一意にし、描画のたびに同じ値を渡すこと。`label` はアクセシブルな名前になる
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            item: Item::new(id.into(), label.into()),
            value,
        }
    }

    /// ラベルの前にアイコンを置く。アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.item.icon = Some(icon.into());
        self
    }

    /// 項目を無効にする。無効な項目は無効の色で描かれ、選べず、Tab と矢印キーでも到達できない
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.item.focus_handle = Some(focus_handle.clone());
        self
    }
}

/// 複数を選べる、MD3 Expressive の Connected button group
///
/// 選択肢を、間を 2dp 空けたトグルボタンとして横に並べる。端の項目は外側の角を丸くし、選ばれた項目は丸い形に変わる。
/// 項目ごとの状態 (選ばれているか) は呼び出し側が持ち、[`ConnectedToggleButton::new`] に毎回渡す。
/// 項目は [`MultiChoiceConnectedButtonGroup::item`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できる項目は 1 つだけで、フォーカスのある項目、最初の有効な項目の順に選ぶ
/// - 左右の矢印キーで前後の有効な項目へフォーカスを移す。選ばれているかは変えない。端では反対の端に戻る
/// - フォーカス中の Enter と Space とマウスのクリックで、項目の選ばれているかを反転するよう求める
///
/// 支援技術には role `toolbar` として公開され、[`MultiChoiceConnectedButtonGroup::aria_label`] がアクセシブルな名前になる。
/// 項目は role `button` と押されているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルは項目を並べる箱に付く。項目はラベルに合わせた幅で、ラベルは 1 行で描く
#[derive(IntoElement)]
pub struct MultiChoiceConnectedButtonGroup {
    id: ElementId,
    aria_label: Option<SharedString>,
    variant: ToggleButtonVariant,
    items: Vec<ConnectedToggleButton>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl MultiChoiceConnectedButtonGroup {
    /// 項目の無い並びを作る
    ///
    /// 項目は [`ToggleButtonVariant::Filled`] で描く。`id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            aria_label: None,
            variant: ToggleButtonVariant::default(),
            items: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 項目のトグルボタンの種類を設定する
    pub fn variant(mut self, variant: ToggleButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ConnectedToggleButton) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ConnectedToggleButton>) -> Self {
        self.items.extend(items);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for MultiChoiceConnectedButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let count = self.items.len();
        let ring = RingSlot::default();
        let items: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(
                |(
                    index,
                    ConnectedToggleButton {
                        item,
                        checked,
                        on_change,
                    },
                )| {
                    let focus_handle = item.focus_handle(&self.id, window, cx);
                    let button = HeadlessToggleButton::new(item.id.clone(), checked)
                        .disabled(item.disabled)
                        .track_focus(&focus_handle)
                        .aria_label(item.label.clone())
                        .when_some(on_change, |this, on_change| this.on_change(on_change));
                    item.decorate(
                        button,
                        checked,
                        index,
                        count,
                        self.variant,
                        &focus_handle,
                        &ring,
                        &theme,
                        window,
                        cx,
                    )
                },
            )
            .collect();
        let mut group = ToggleGroup::new(self.id)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .flex()
            .items_center()
            .gap(dp(tokens::BETWEEN_SPACE));
        group.style().refine(&self.style);
        group.items(items).child(paint_ring(ring, theme))
    }
}

impl Styled for MultiChoiceConnectedButtonGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`MultiChoiceConnectedButtonGroup`] の項目
pub struct ConnectedToggleButton {
    item: Item,
    checked: bool,
    on_change: Option<ToggleHandler>,
}

impl ConnectedToggleButton {
    /// `label` を描き、`checked` の状態を表示する項目を作る
    ///
    /// `id` は並びの中で一意にし、描画のたびに同じ値を渡すこと。`label` はアクセシブルな名前になり、状態によって変えないこと
    pub fn new(id: impl Into<ElementId>, checked: bool, label: impl Into<SharedString>) -> Self {
        Self {
            item: Item::new(id.into(), label.into()),
            checked,
            on_change: None,
        }
    }

    /// 押されたときに、反転した状態で `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ConnectedToggleButton::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// ラベルの前にアイコンを置く。アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.item.icon = Some(icon.into());
        self
    }

    /// 項目を無効にする。無効な項目は無効の色で描かれ、押されてもハンドラを呼ばず、Tab と矢印キーでも到達できない
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.item.focus_handle = Some(focus_handle.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        AnyElement, Context, Modifiers, Render, TestAppContext, VisualTestContext, div, point, px,
    };
    use md3_theme::md3_color::ColorRole;

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // 原点のマウスの位置で hover にならないよう、並びを原点から離す
            div()
                .flex()
                .items_start()
                .pl(px(100.0))
                .pt(px(100.0))
                .child((self.0)())
        }
    }

    fn open(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        // Tab のキー割り当てを登録する
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        // 形のバネを終端まで即座に進める
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.run_until_parked();
        cx
    }

    /// `role` の色で塗った容器の、x・幅・左上と右上の角の半径を左から順に
    fn containers(cx: &mut VisualTestContext, role: ColorRole) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let background: gpui::Background = cx.md3_theme().color(role).into();
            let mut found: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background)
                .map(|quad| {
                    (
                        quad.bounds.origin.x.0 / scale - 100.0,
                        quad.bounds.size.width.0 / scale,
                        quad.corner_radii.top_left.0 / scale,
                        quad.corner_radii.top_right.0 / scale,
                    )
                })
                .collect();
            found.sort_by(|a, b| a.0.total_cmp(&b.0));
            found
        })
    }

    /// 幅 60dp の内容を持つ 3 つの項目。`selected` 番目を選ぶ
    fn single(selected: Rc<Cell<Option<usize>>>) -> AnyElement {
        let on_change = selected.clone();
        SingleChoiceConnectedButtonGroup::new("group", selected.get())
            .on_change(move |value: &usize, window, _| {
                on_change.set(Some(*value));
                window.refresh();
            })
            .items(
                (0..3).map(|index| {
                    ConnectedButton::new(index, index, ["Day", "Week", "Month"][index])
                }),
            )
            .into_any_element()
    }

    fn click(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_click(point(px(100.0 + x), px(120.0)), Modifiers::none());
        cx.simulate_mouse_move(point(px(10.0), px(10.0)), None, Modifiers::none());
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 端の項目は外側の角を丸め_選ばれた項目は丸い形にする(
        cx: &mut TestAppContext,
    ) {
        let selected = Rc::new(Cell::new(Some(1)));
        let state = selected.clone();
        let cx = open(cx, move || single(state.clone()));
        let unchecked = containers(cx, ColorRole::SurfaceContainer);
        let checked = containers(cx, ColorRole::Primary);
        assert_eq!(unchecked.len(), 2);
        assert_eq!(checked.len(), 1);
        assert_eq!(
            (unchecked[0].2, unchecked[0].3),
            (20.0, 8.0),
            "先頭は左を丸くすること"
        );
        assert_eq!(
            (checked[0].2, checked[0].3),
            (20.0, 20.0),
            "選ばれると丸くすること"
        );
        assert_eq!(
            (unchecked[1].2, unchecked[1].3),
            (8.0, 20.0),
            "末尾は右を丸くすること"
        );
        assert_eq!(
            checked[0].0,
            unchecked[0].0 + unchecked[0].1 + 2.0,
            "間を 2dp 空けること"
        );
    }

    #[gpui::test]
    fn 押した項目を選ぶよう求め_間の項目は小さく丸める(
        cx: &mut TestAppContext,
    ) {
        let selected = Rc::new(Cell::new(Some(0)));
        let state = selected.clone();
        let cx = open(cx, move || single(state.clone()));
        let unchecked = containers(cx, ColorRole::SurfaceContainer);
        assert_eq!(
            (unchecked[0].2, unchecked[0].3),
            (8.0, 8.0),
            "間の項目は Small にすること"
        );
        let last = unchecked[1];
        click(cx, last.0 + last.1 / 2.0);
        assert_eq!(selected.get(), Some(2));
        let checked = containers(cx, ColorRole::Primary);
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].0, last.0);
    }

    #[gpui::test]
    fn 複数を選ぶ並びは押した項目だけを反転するよう求める(
        cx: &mut TestAppContext,
    ) {
        let checked = Rc::new([Cell::new(false), Cell::new(true), Cell::new(false)]);
        let state = checked.clone();
        let cx = open(cx, move || {
            MultiChoiceConnectedButtonGroup::new("group")
                .items((0..3).map(|index| {
                    let state = state.clone();
                    ConnectedToggleButton::new(index, state[index].get(), "Item").on_change(
                        move |value, window, _| {
                            state[index].set(*value);
                            window.refresh();
                        },
                    )
                }))
                .into_any_element()
        });
        let first = containers(cx, ColorRole::SurfaceContainer)[0];
        click(cx, first.0 + first.1 / 2.0);
        assert_eq!(
            checked.iter().map(Cell::get).collect::<Vec<_>>(),
            vec![true, true, false]
        );
        assert_eq!(containers(cx, ColorRole::Primary).len(), 2);
    }

    #[gpui::test]
    fn 項目が_1_つなら先頭の形にする(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            SingleChoiceConnectedButtonGroup::new("group", None::<usize>)
                .item(ConnectedButton::new("only", 0, "Only"))
                .into_any_element()
        });
        let unchecked = containers(cx, ColorRole::SurfaceContainer);
        assert_eq!((unchecked[0].2, unchecked[0].3), (20.0, 8.0));
    }

    #[gpui::test]
    fn フォーカスリングは隣の項目より後に描く(cx: &mut TestAppContext) {
        let selected = Rc::new(Cell::new(Some(0)));
        let state = selected.clone();
        let cx = open(cx, move || single(state.clone()));
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let (ring, neighbor) = cx.update(|window, cx| {
            let theme = cx.md3_theme().clone();
            let ring = theme.color(md3_theme::md3_tokens::focus_ring::COLOR);
            let neighbor: gpui::Background = theme.color(ColorRole::SurfaceContainer).into();
            let quads = window.painted_quads();
            (
                quads
                    .iter()
                    .filter(|quad| quad.border_color == ring && quad.border_widths.top.0 > 0.0)
                    .map(|quad| quad.order)
                    .max(),
                // 先頭の項目の右隣
                quads
                    .iter()
                    .filter(|quad| quad.background == neighbor)
                    .min_by(|a, b| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
                    .map(|quad| quad.order),
            )
        });
        let ring = ring.expect("先頭の項目にリングを描くこと");
        assert!(ring > neighbor.expect("隣の項目を描くこと"));
    }

    #[gpui::test]
    fn フォーカスリングに祖先の不透明度が効く(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            div()
                .opacity(0.5)
                .child(single(Rc::new(Cell::new(Some(0)))))
                .into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let alpha = cx.update(|window, cx| {
            let ring = cx
                .md3_theme()
                .color(md3_theme::md3_tokens::focus_ring::COLOR);
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.border_widths.top.0 > 0.0 && quad.border_color.h == ring.h)
                .map(|quad| quad.border_color.a)
        });
        assert_eq!(alpha, Some(0.5));
    }
}
