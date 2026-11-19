// 寸法・色・項目の並べ方・項目の中の配置・インジケーターの動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ShortNavigationBar.kt
//         (`ShortNavigationBar`、`ShortNavigationBarItem`、`ShortNavigationBarDefaults`、
//         `ShortNavigationBarItemDefaults`、`EqualWeightContentMeasurePolicy`、`CenteredContentMeasurePolicy`、
//         `calculateCenteredContentHorizontalPadding`、ファイル末尾の余白の値)、
//         NavigationItem.kt (`NavigationItem`、`TopIconOrIconOnlyMeasurePolicy`、`StartIconMeasurePolicy`、
//         `placeLabelAndTopIcon`、`placeLabelAndStartIcon`、`animateIndicatorProgressAsState`、`Indicator`、
//         `IndicatorRipple`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// Compose はフォーカスをインジケーターの形の状態レイヤーだけで示す。この実装はそれに、ほかの部品と同じ外向きのフォーカスリングを
// インジケーターの形で加える。押している間の ripple は描かない
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のナビゲーションバー

use gpui::{
    AnimationExt, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, SharedString, SpringAnimation, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder, relative, svg,
};
use gpui_headless::tabs::{Tab as HeadlessTab, TabList};
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{
    navigation_bar, navigation_bar_horizontal_item as horizontal,
    navigation_bar_vertical_item as vertical,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 項目の中の層が項目の hover を知るための group 名
const GROUP: &str = "md3-navigation-bar-item";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// ナビゲーションバーの項目の並べ方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationBarArrangement {
    /// 項目をバーの幅いっぱいに同じ幅で並べる。縦長の画面に向く
    #[default]
    EqualWeight,
    /// 項目をバーの中央に寄せて並べる。横長の画面に向く。項目が 7 つ以上なら [`Self::EqualWeight`] と同じ
    Centered,
}

/// ナビゲーションバーの項目のアイコンの位置
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationItemIconPosition {
    /// ラベルの上にアイコンを置く
    #[default]
    Top,
    /// ラベルの前にアイコンを置く。横長の画面で [`NavigationBarArrangement::Centered`] と組み合わせる
    Start,
}

/// MD3 Expressive のナビゲーションバー
///
/// アプリの主な行き先を画面の下端に並べ、選ばれている行き先をアイコンの後ろのインジケーターで示す。
/// 状態 (選ばれている項目の値) は呼び出し側が持ち、[`NavigationBar::new`] に毎回渡す。バー自身は状態を変えない。
/// 項目は [`NavigationBar::item`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初のフォーカスできる項目の順に選ぶ
/// - 左右の矢印キーで隣の項目へフォーカスを移し、移った先の項目を選ぶよう求める。端では反対の端に戻る。
///   Home と End では最初と最後の項目へ移り、移った先を選ぶよう求める。無効な項目は飛ばす
/// - フォーカス中の Enter と Space・マウスのクリックで、項目を選ぶよう求める
///
/// 矢印キーでも選ぶよう求めるため、行き先の内容は [`NavigationBar::on_change`] を受けた次の描画で遅れなく差し替えること。
/// バーは行き先の内容 (role `tabpanel`) を公開しない。選ばれた項目と、表示している行き先の内容を一致させるのは呼び出し側の責務になる
///
/// 支援技術には role `tablist` として公開され、[`NavigationBar::aria_label`] がアクセシブルな名前になる。
/// 項目は role `tab` と選ばれているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルはバーの箱に付く。既定では親の幅いっぱいに広がり、高さは 64dp を最小とする
#[derive(IntoElement)]
pub struct NavigationBar<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    items: Vec<NavigationBarItem<T>>,
    on_change: Option<ChangeHandler<T>>,
    arrangement: NavigationBarArrangement,
    aria_label: Option<SharedString>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> NavigationBar<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示するバーを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            items: Vec::new(),
            on_change: None,
            arrangement: NavigationBarArrangement::default(),
            aria_label: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 項目の並べ方を変える
    pub fn arrangement(mut self, arrangement: NavigationBarArrangement) -> Self {
        self.arrangement = arrangement;
        self
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目と無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`NavigationBar::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: NavigationBarItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = NavigationBarItem<T>>) -> Self {
        self.items.extend(items);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for NavigationBar<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let count = self.items.len();
        let label_font = theme.text_style(navigation_bar::LABEL_TEXT_FONT);
        let spatial = theme.spring(MotionSchemeKey::DefaultSpatial);
        let state_color = theme.color(supplement::navigation_bar::STATE_LAYER_COLOR);
        let disabled_opacity = supplement::navigation_bar::DISABLED_CONTENT_OPACITY;

        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let items: Vec<_> = self
            .items
            .into_iter()
            .map(|item| {
                let is_selected = selected
                    .take_if(|selected| **selected == item.value)
                    .is_some();
                let focus_handle = item.focus_handle.clone().unwrap_or_else(|| {
                    // 項目の id はバーの中でだけ一意なので、バーの id で分ける
                    window.with_element_namespace(self.id.clone(), |window| {
                        util::keyed_focus_handle(&item.id, window, cx)
                    })
                });
                // フォーカスのある項目が無効になってもフォーカスは残るため、リングは無効な項目でも描く
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let top = item.icon_position == NavigationItemIconPosition::Top;

                // 色 (`ShortNavigationBarItemDefaults.colors`)。無効な項目は選ばれていない色を薄くする。
                // ラベルの前にアイコンを置くと、選ばれたラベルはアイコンと同じ色にする
                let (icon_color, label_color) = if item.disabled {
                    (
                        theme
                            .color(navigation_bar::ITEM_INACTIVE_ICON_COLOR)
                            .opacity(disabled_opacity),
                        theme
                            .color(navigation_bar::ITEM_INACTIVE_LABEL_TEXT_COLOR)
                            .opacity(disabled_opacity),
                    )
                } else if is_selected {
                    (
                        theme.color(navigation_bar::ITEM_ACTIVE_ICON_COLOR),
                        theme.color(if top {
                            navigation_bar::ITEM_ACTIVE_LABEL_TEXT_COLOR
                        } else {
                            navigation_bar::ITEM_ACTIVE_ICON_COLOR
                        }),
                    )
                } else {
                    (
                        theme.color(navigation_bar::ITEM_INACTIVE_ICON_COLOR),
                        theme.color(navigation_bar::ITEM_INACTIVE_LABEL_TEXT_COLOR),
                    )
                };
                let indicator_color = theme.color(navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
                let shape = navigation_bar::ITEM_ACTIVE_INDICATOR_SHAPE;

                let icon = if is_selected {
                    item.selected_icon.unwrap_or(item.icon)
                } else {
                    item.icon
                };
                let icon_size = dp(if top {
                    vertical::ICON_SIZE
                } else {
                    horizontal::ICON_SIZE
                });
                let icon = svg()
                    .data(icon.svg().as_bytes())
                    .flex_none()
                    .size(icon_size)
                    .text_color(icon_color);
                // ラベルは項目の幅に収まらなければ折り返し、インジケーターや項目の高さを広げる
                let mut label = div().min_w_0().text_color(label_color);
                label.text_style().refine(&label_font);
                let label = label.child(item.label.clone());

                // インジケーターは選ばれると幅が中央から広がり、同時に濃くなる。
                // 幅と不透明度は DefaultSpatial のバネの進みで、行き過ぎで負になる分は 0 にする。
                // Compose は項目の幅に収まる範囲で 1 を超える行き過ぎを残すが、インジケーターの幅が項目の幅に達すると
                // この箱では上限を課せないため、1 を上限にする。行き過ぎは 2% 未満で、56dp の幅で 1dp に満たない
                let indicator_theme = theme.clone();
                let indicator = div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .with_spring(
                        "indicator",
                        SpringAnimation::new(spatial).to(is_selected),
                        move |this, phase| {
                            let progress = phase.0.clamp(0.0, 1.0);
                            this.child(Shaped::new(
                                div()
                                    .flex_none()
                                    .h_full()
                                    .w(relative(progress))
                                    .bg(indicator_color.opacity(progress)),
                                indicator_theme.clone(),
                                shape,
                            ))
                        },
                    );
                // 状態レイヤーとフォーカスリングは項目全体ではなくインジケーターの形に描く (`IndicatorRipple`)。
                // 無効な項目には状態レイヤーを描かない
                let state_layer = (!item.disabled).then(|| {
                    Shaped::new(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .right_0()
                            .when(focus_visible, |this| {
                                this.bg(state_color.opacity(state::FOCUS_STATE_LAYER_OPACITY))
                            })
                            // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                            .group_hover(GROUP, |style| {
                                style.bg(state_color.opacity(state::HOVER_STATE_LAYER_OPACITY))
                            }),
                        theme.clone(),
                        shape,
                    )
                });
                let ring = focus_visible.then(|| {
                    let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
                    Shaped::new(
                        div()
                            .absolute()
                            .top(-outset)
                            .left(-outset)
                            .right(-outset)
                            .bottom(-outset)
                            .border(dp(focus_ring::WIDTH))
                            .border_color(theme.color(focus_ring::COLOR)),
                        theme.clone(),
                        shape,
                    )
                    .outset(outset)
                });

                // インジケーターは内容の周りに余白を取った大きさにする。余白はアイコンだけのときに
                // インジケーターの寸法になる値 (`TopIconIndicatorVerticalPadding` などのファイル末尾の値)
                let pill = div()
                    .relative()
                    .flex()
                    .flex_none()
                    .max_w_full()
                    .items_center();
                let pill = if top {
                    // 項目がインジケーターより狭ければインジケーターを項目の幅に縮め、アイコンは中央に残す
                    pill.justify_center()
                        .w(dp(vertical::ACTIVE_INDICATOR_WIDTH))
                        .py((dp(vertical::ACTIVE_INDICATOR_HEIGHT) - icon_size) / 2.0)
                } else {
                    // 上下の余白は、アイコンとラベルの高い方に足す
                    pill.py((dp(horizontal::ACTIVE_INDICATOR_HEIGHT) - icon_size) / 2.0)
                };
                // Compose はラベルを項目の幅からアイコンと間隔だけを除いた幅で測り、インジケーターを項目の幅に収める。
                // 左右の余白は padding ではなく縮む要素で取り、縮める割合を大きくしてラベルより先に縮める
                let side_space =
                    |space| (!top).then(|| div().flex_shrink(1000.0).min_w_0().w(dp(space)));
                let leading = side_space(horizontal::ACTIVE_INDICATOR_LEADING_SPACE);
                let trailing = side_space(horizontal::ACTIVE_INDICATOR_TRAILING_SPACE);
                let (inline_label, below_label) = if top {
                    let label = label.max_w_full().text_center().mt(dp(
                        supplement::navigation_bar::TOP_ICON_INDICATOR_TO_LABEL_PADDING,
                    ));
                    (None, Some(label))
                } else {
                    // ラベルに残る幅が 1 文字にも満たないほど項目が狭いと、ラベルは隣の項目に重ならないよう切れる
                    let label = label
                        .overflow_hidden()
                        .ml(dp(navigation_bar::ITEM_ACTIVE_INDICATOR_ICON_LABEL_SPACE));
                    (Some(label), None)
                };
                let pill = pill
                    .child(indicator)
                    .children(state_layer)
                    .children(leading)
                    .child(icon)
                    .children(inline_label)
                    .children(trailing)
                    .children(ring);

                let headless = HeadlessTab::new(item.id, item.value)
                    .track_focus(&focus_handle)
                    .aria_label(item.label)
                    .disabled(item.disabled)
                    .group(GROUP)
                    .flex()
                    .flex_col()
                    .items_center()
                    .flex_none();
                // Compose の項目の最小の大きさ (`LocalMinimumInteractiveComponentSize`) は、親が項目の大きさを
                // 決めると効かない。項目の幅はバーが割り当てる幅に従うため扱わない
                let headless = match self.arrangement {
                    NavigationBarArrangement::EqualWeight => {
                        headless.w(relative(1.0 / count.max(1) as f32))
                    }
                    // 項目は内容の幅を、余白を除いたバーの幅の等分から、バーの幅の等分までに収める
                    // (`CenteredContentMeasurePolicy`)。項目の割合はバーの幅に対する値で、並びはバーの中央に置く
                    NavigationBarArrangement::Centered => {
                        let padding = if count <= 6 {
                            // バーの幅に対する左右の余白の割合 (`calculateCenteredContentHorizontalPadding`)
                            (100.0 - 10.0 * (count as f32 + 3.0)) / 2.0 / 100.0
                        } else {
                            0.0
                        };
                        let count = count.max(1) as f32;
                        headless
                            .min_w(relative((1.0 - padding * 2.0) / count))
                            .max_w(relative(1.0 / count))
                    }
                };
                // バーが最も高い項目に合わせて伸びても、ラベルの上にアイコンを置く項目の内容は上端に、
                // ラベルの前にアイコンを置く項目の内容は上下中央に置く (`placeLabelAndTopIcon`、`placeLabelAndStartIcon`)
                let headless = if top {
                    headless
                        .justify_start()
                        .py(dp(vertical::CONTAINER_BETWEEN_SPACE))
                } else {
                    headless.justify_center()
                };
                headless.child(pill).children(below_label)
            })
            .collect();

        let mut bar = TabList::new(self.id, self.selected)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .flex()
            .justify_center()
            .w_full()
            .min_h(dp(navigation_bar::CONTAINER_HEIGHT))
            .bg(theme.color(navigation_bar::CONTAINER_COLOR));
        bar.style().refine(&self.style);
        bar.items(items)
    }
}

impl<T: PartialEq + 'static> Styled for NavigationBar<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`NavigationBar`] の項目
///
/// [`NavigationBar::item`] に渡して使う。項目が表す値が、バーの選ばれている値と等しいときに選ばれた状態になる
pub struct NavigationBarItem<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    icon: StyledIcon,
    selected_icon: Option<StyledIcon>,
    icon_position: NavigationItemIconPosition,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
}

impl<T> NavigationBarItem<T> {
    /// `value` を表し、`icon` と `label` を描く項目を作る
    ///
    /// `label` はアクセシブルな名前にもなり、アイコンは装飾として名前に含めない。
    /// `id` と `value` はバーの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`NavigationBarItem::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(
        id: impl Into<ElementId>,
        value: T,
        icon: impl Into<StyledIcon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            icon: icon.into(),
            selected_icon: None,
            icon_position: NavigationItemIconPosition::default(),
            disabled: false,
            focus_handle: None,
        }
    }

    /// 選ばれているあいだは `icon` を描く。塗りつぶしたアイコンで選ばれていることを示すときに使う
    pub fn selected_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.selected_icon = Some(icon.into());
        self
    }

    /// アイコンの位置を変える
    pub fn icon_position(mut self, position: NavigationItemIconPosition) -> Self {
        self.icon_position = position;
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は操作されても選ぶよう求めず、Tab や矢印キーでも到達できず、支援技術には disabled として公開される
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// Tab で到達できるかはバーが決める。指定しない場合は項目が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        AnyElement, Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px,
    };
    use md3_icons::{HOME, SEARCH, SETTINGS};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(400.0)).child((self.0)())
        }
    }

    /// 矩形の論理ピクセルの x, y, 幅, 高さ
    type Rect = (f32, f32, f32, f32);

    fn open(
        cx: &mut TestAppContext,
        element: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        let (_, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(element))
        });
        cx.run_until_parked();
        cx
    }

    /// `role` の色で塗った矩形
    fn quads_of(cx: &mut VisualTestContext, role: ColorRole) -> Vec<Rect> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background = cx.md3_theme().color(role).into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == color)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    fn bar(
        position: NavigationItemIconPosition,
        arrangement: NavigationBarArrangement,
    ) -> AnyElement {
        NavigationBar::new("bar", Some(0))
            .arrangement(arrangement)
            .aria_label("Main")
            .items([
                NavigationBarItem::new("home", 0, HOME, "Home").icon_position(position),
                NavigationBarItem::new("search", 1, SEARCH, "Search").icon_position(position),
                NavigationBarItem::new("settings", 2, SETTINGS, "Settings").icon_position(position),
                NavigationBarItem::new("more", 3, SETTINGS, "More").icon_position(position),
            ])
            .into_any_element()
    }

    #[gpui::test]
    fn 上にアイコンを置く項目は_56x32_のインジケーターを項目の中央上に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            bar(
                NavigationItemIconPosition::Top,
                NavigationBarArrangement::EqualWeight,
            )
        });
        let container = quads_of(cx, navigation_bar::CONTAINER_COLOR);
        assert_eq!(
            container,
            vec![(0.0, 0.0, 400.0, 64.0)],
            "バーは 64dp の高さ"
        );
        // 4 つの項目は 100dp ずつの幅で、インジケーターは上端から 6dp に置く
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator, vec![(22.0, 6.0, 56.0, 32.0)]);
    }

    #[gpui::test]
    fn 前にアイコンを置く項目は高さ_40_のインジケーターでアイコンとラベルを囲む(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            bar(
                NavigationItemIconPosition::Start,
                NavigationBarArrangement::EqualWeight,
            )
        });
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator.len(), 1);
        let (x, y, width, height) = indicator[0];
        assert_eq!((y, height), (12.0, 40.0), "64dp のバーの上下中央");
        // 左右 16dp の余白、24dp のアイコン、4dp の間隔とラベル
        assert!(
            width > 16.0 + 24.0 + 4.0 + 16.0,
            "ラベルの幅も囲む: {width}"
        );
        assert_eq!(x + width / 2.0, 50.0, "項目の中央に置く");
    }

    #[gpui::test]
    fn 中央に寄せる並びは項目の数で決まる割合の余白を左右に取る(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationBar::new("bar", Some(0))
                .arrangement(NavigationBarArrangement::Centered)
                .items([
                    NavigationBarItem::new("home", 0, HOME, "Home"),
                    NavigationBarItem::new("search", 1, SEARCH, "Search"),
                    NavigationBarItem::new("settings", 2, SETTINGS, "Settings"),
                ])
                .into_any_element()
        });
        // 3 項目では左右に幅の (100 - 10 × 6) / 2 = 20% を空け、残りの 240dp を 3 等分する
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(
            indicator,
            vec![(80.0 + (80.0 - 56.0) / 2.0, 6.0, 56.0, 32.0)]
        );
    }

    #[gpui::test]
    fn クリックで項目を選ぶよう求め_無効な項目では求めない(
        cx: &mut TestAppContext,
    ) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let recorder = changes.clone();
        let cx = open(cx, move || {
            let recorder = recorder.clone();
            NavigationBar::new("bar", Some(0))
                .on_change(move |value, _, _| recorder.borrow_mut().push(*value))
                .items([
                    NavigationBarItem::new("home", 0, HOME, "Home"),
                    NavigationBarItem::new("search", 1, SEARCH, "Search"),
                    NavigationBarItem::new("settings", 2, SETTINGS, "Settings").disabled(true),
                ])
                .into_any_element()
        });
        // 400dp を 3 等分した 2 つ目と 3 つ目の項目の中央
        for x in [200.0, 333.0] {
            cx.simulate_click(point(px(x), px(32.0)), Modifiers::none());
        }
        assert_eq!(
            *changes.borrow(),
            vec![1],
            "無効な項目のクリックは選ぶよう求めない"
        );
    }

    #[gpui::test]
    fn 前にアイコンを置く項目のインジケーターは複数行のラベルに合わせて伸びる(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationBar::new("bar", Some(0))
                .item(
                    NavigationBarItem::new("home", 0, HOME, "Long\nLabel\nMultiple\nLines")
                        .icon_position(NavigationItemIconPosition::Start),
                )
                .into_any_element()
        });
        // LabelMedium の行の高さ 16dp の 4 行に、上下 8dp ずつの余白を足す
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator.len(), 1);
        assert_eq!((indicator[0].1, indicator[0].3), (0.0, 80.0));
        let container = quads_of(cx, navigation_bar::CONTAINER_COLOR);
        assert_eq!(
            container,
            vec![(0.0, 0.0, 400.0, 80.0)],
            "バーは最も高い項目に合わせる"
        );
    }

    #[gpui::test]
    fn 中央に寄せる並びの割合は親ではなくバーの幅に対する値(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationBar::new("bar", Some(0))
                .arrangement(NavigationBarArrangement::Centered)
                .items([
                    NavigationBarItem::new("home", 0, HOME, "Home"),
                    NavigationBarItem::new("search", 1, SEARCH, "Search"),
                    NavigationBarItem::new("settings", 2, SETTINGS, "Settings"),
                ])
                .w(px(300.0))
                .into_any_element()
        });
        // 幅 300dp のバーの左右に 20% の 60dp を空け、項目は 60dp ずつになる
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator, vec![(62.0, 6.0, 56.0, 32.0)]);
    }

    #[gpui::test]
    fn 中央に寄せる並びは等分より広い項目を内容の幅まで広げて中央に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            let item = |id: &'static str, value: usize, label: &'static str| {
                NavigationBarItem::new(id, value, HOME, label)
                    .icon_position(NavigationItemIconPosition::Start)
            };
            NavigationBar::new("bar", Some(0))
                .arrangement(NavigationBarArrangement::Centered)
                .items([
                    item("a", 0, "Abc"),
                    item("b", 1, "A"),
                    item("c", 2, "B"),
                    item("d", 3, "C"),
                ])
                .into_any_element()
        });
        // 4 項目では項目の幅を 400dp × 70% の等分の 70dp から、400dp の等分の 100dp までに収める。
        // 2 つ目以降の項目は 70dp で、最初の項目はインジケーターの幅に広がり、並び全体をバーの中央に置く
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator.len(), 1);
        let (x, _, width, _) = indicator[0];
        assert!(
            width > 70.0 && width < 100.0,
            "等分より広く上限より狭い: {width}"
        );
        assert_eq!(x, (400.0 - width - 70.0 * 3.0) / 2.0);
    }

    #[gpui::test]
    fn 前にアイコンを置く項目は狭いときラベルを先に取り左右の余白を縮める(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            let item = |id: &'static str, value: usize| {
                NavigationBarItem::new(id, value, HOME, "Label")
                    .icon_position(NavigationItemIconPosition::Start)
            };
            NavigationBar::new("bar", Some(0))
                .items([
                    item("a", 0),
                    item("b", 1),
                    item("c", 2),
                    item("d", 3),
                    item("e", 4),
                ])
                .w(px(320.0))
                .into_any_element()
        });
        // 64dp の項目からアイコンと間隔を除いた 36dp にラベルを収め、インジケーターは項目の幅を超えない
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator.len(), 1);
        let (x, _, width, _) = indicator[0];
        assert!(
            x >= 0.0 && x + width <= 64.0,
            "項目の中に収める: {x} {width}"
        );
    }

    #[gpui::test]
    fn 上にアイコンを置く項目はバーが伸びてもインジケーターを上端から_6dp_に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationBar::new("bar", Some(0))
                .items([
                    NavigationBarItem::new("home", 0, HOME, "Home"),
                    NavigationBarItem::new("search", 1, SEARCH, "Long\nLabel\nLines"),
                ])
                .into_any_element()
        });
        let container = quads_of(cx, navigation_bar::CONTAINER_COLOR);
        assert_eq!(container[0].3, 96.0, "3 行のラベルでバーが伸びる");
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator, vec![(72.0, 6.0, 56.0, 32.0)]);
    }

    #[gpui::test]
    fn 上にアイコンを置く項目は狭いときインジケーターを項目の幅に縮めて中央に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            let item =
                |id: &'static str, value: usize| NavigationBarItem::new(id, value, HOME, "A");
            NavigationBar::new("bar", Some(1))
                .items([
                    item("a", 0),
                    item("b", 1),
                    item("c", 2),
                    item("d", 3),
                    item("e", 4),
                ])
                .w(px(240.0))
                .into_any_element()
        });
        // 48dp の項目では、56dp のインジケーターを項目の幅に縮める
        let indicator = quads_of(cx, navigation_bar::ITEM_ACTIVE_INDICATOR_COLOR);
        assert_eq!(indicator, vec![(48.0, 6.0, 48.0, 32.0)]);
    }
}
