// 寸法・色・項目の並べ方・項目の中の配置・幅とインジケーターの動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/WideNavigationRail.kt
//         (`WideNavigationRail`、`WideNavigationRailLayout`、`WideNavigationRailItem`、`WideNavigationRailDefaults`、
//         `WideNavigationRailItemDefaults`、ファイル末尾の余白の値)、
//         ComponentStyles.kt (`NavigationRailStyle.Default`、`NavigationRailItemStyle.Default`)、
//         NavigationItem.kt (`TopIconOrIconOnlyMeasurePolicy`、`StartIconMeasurePolicy`、
//         `animateIndicatorProgressAsState`、`IndicatorRipple`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// Compose の既定の設定はフォーカスをインジケーターの形の状態レイヤーだけで示す。この実装はそれに、ほかの部品と同じ外向きのフォーカスリングを
// インジケーターの形で加える。押している間の ripple は描かない。
// Compose は展開と折りたたみでアイコンの位置も動かすが、この実装はレールの幅だけを動かし、項目の配置はすぐに切り替える
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のナビゲーションレール

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, SpringAnimation,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder, px,
    relative, svg,
};
use gpui_headless::tabs::{Tab as HeadlessTab, TabList, TabListOrientation};
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{
    navigation_rail_baseline_item as baseline, navigation_rail_collapsed as collapsed,
    navigation_rail_color as color, navigation_rail_expanded as expanded,
    navigation_rail_horizontal_item as horizontal, navigation_rail_vertical_item as vertical,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 項目の中の層が項目の hover を知るための group 名
const GROUP: &str = "md3-navigation-rail-item";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// ナビゲーションレールの項目の縦の並べ方
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NavigationRailArrangement {
    /// ヘッダーの下に、上から詰めて並べる
    #[default]
    Top,
    /// レールの高さの中央に並べる。ヘッダーは上端に置き、項目の位置には影響しない
    Center,
    /// 下端に詰めて並べる
    Bottom,
}

/// MD3 Expressive のナビゲーションレール (Compose の `WideNavigationRail`)
///
/// アプリの主な行き先を画面の左端に縦に並べ、選ばれている行き先をアイコンの後ろのインジケーターで示す。
/// 折りたたむとアイコンの下にラベルを置いた 96dp の幅になり、展開するとアイコンの右にラベルを置いた
/// 220dp から 360dp の幅になる。幅は DefaultSpatial のバネで動く。
/// 状態 (選ばれている項目の値と、展開しているか) は呼び出し側が持ち、毎回渡す。レール自身は状態を変えない
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初のフォーカスできる項目の順に選ぶ
/// - 上下の矢印キーで隣の項目へフォーカスを移し、移った先の項目を選ぶよう求める。端では反対の端に戻る。
///   Home と End では最初と最後の項目へ移り、移った先を選ぶよう求める。無効な項目は飛ばす
/// - フォーカス中の Enter と Space・マウスのクリックで、項目を選ぶよう求める
///
/// 矢印キーでも選ぶよう求めるため、行き先の内容は [`NavigationRail::on_change`] を受けた次の描画で遅れなく差し替えること。
/// レールは行き先の内容 (role `tabpanel`) を公開しない。選ばれた項目と、表示している行き先の内容を一致させるのは呼び出し側の責務になる
///
/// 支援技術には role `tablist` と縦向きとして公開され、[`NavigationRail::aria_label`] がアクセシブルな名前になる。
/// 項目は role `tab` と選ばれているかどうかの状態として公開され、ラベルが名前になる。
/// ヘッダーは項目の前に、並びの子として公開される
///
/// [`Styled`] で指定したスタイルはレールの箱に付く。既定では親の高さいっぱいに広がり、幅は上の範囲で内容に合わせる
#[derive(IntoElement)]
pub struct NavigationRail<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    items: Vec<NavigationRailItem<T>>,
    on_change: Option<ChangeHandler<T>>,
    expanded: bool,
    header: Option<AnyElement>,
    arrangement: NavigationRailArrangement,
    aria_label: Option<SharedString>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> NavigationRail<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示する、折りたたんだレールを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            items: Vec::new(),
            on_change: None,
            expanded: false,
            header: None,
            arrangement: NavigationRailArrangement::default(),
            aria_label: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// `true` なら展開し、`false` なら折りたたむ
    ///
    /// 値が変わると、レールの幅は前の幅からバネで動く
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// 項目の上に `header` を置く。メニューボタンや FAB を置くときに使う
    ///
    /// ヘッダーと項目の間は 40dp 空ける。ヘッダーの横の配置と大きさは `header` 自身のスタイルで決める
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// 項目の縦の並べ方を変える
    pub fn arrangement(mut self, arrangement: NavigationRailArrangement) -> Self {
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
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`NavigationRail::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: NavigationRailItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = NavigationRailItem<T>>) -> Self {
        self.items.extend(items);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for NavigationRail<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let spatial = theme.spring(MotionSchemeKey::DefaultSpatial);
        let state_color = theme.color(supplement::navigation_rail::STATE_LAYER_COLOR);
        let disabled_opacity = supplement::color_scheme::DISABLED_ALPHA;
        // 折りたたむとアイコンをラベルの上に、展開するとラベルの前に置く (`iconPositionFor`)
        let top = !self.expanded;
        let label_font = theme.text_style(if top {
            vertical::LABEL_TEXT_FONT
        } else {
            horizontal::LABEL_TEXT_FONT
        });
        let icon_size = dp(baseline::ICON_SIZE);

        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let items: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = selected
                    .take_if(|selected| **selected == item.value)
                    .is_some();
                let focus_handle = item.focus_handle.clone().unwrap_or_else(|| {
                    // 項目の id はレールの中でだけ一意なので、レールの id で分ける
                    window.with_element_namespace(self.id.clone(), |window| {
                        util::keyed_focus_handle(&item.id, window, cx)
                    })
                });
                // フォーカスのある項目が無効になってもフォーカスは残るため、リングは無効な項目でも描く
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

                // 色 (`NavigationRailItemStyle.Default`)。無効な項目は選ばれていない色を薄くする。
                // ラベルの前にアイコンを置くと、選ばれたラベルはアイコンと同じ色にする
                let (icon_color, label_color) = if item.disabled {
                    (
                        theme
                            .color(color::ITEM_INACTIVE_ICON)
                            .opacity(disabled_opacity),
                        theme
                            .color(color::ITEM_INACTIVE_LABEL_TEXT)
                            .opacity(disabled_opacity),
                    )
                } else if is_selected {
                    (
                        theme.color(color::ITEM_ACTIVE_ICON),
                        theme.color(if top {
                            color::ITEM_ACTIVE_LABEL_TEXT
                        } else {
                            color::ITEM_ACTIVE_ICON
                        }),
                    )
                } else {
                    (
                        theme.color(color::ITEM_INACTIVE_ICON),
                        theme.color(color::ITEM_INACTIVE_LABEL_TEXT),
                    )
                };
                let indicator_color = theme.color(color::ITEM_ACTIVE_INDICATOR);
                let shape = baseline::ACTIVE_INDICATOR_SHAPE;

                let icon = if is_selected {
                    item.selected_icon.unwrap_or(item.icon)
                } else {
                    item.icon
                };
                let icon = svg()
                    .data(icon.svg().as_bytes())
                    .flex_none()
                    .size(icon_size)
                    .text_color(icon_color);
                let mut label = div().min_w_0().text_color(label_color);
                label.text_style().refine(&label_font);
                let label = label.child(item.label.clone());

                // インジケーターは選ばれると幅が中央から広がり、同時に濃くなる。
                // 幅と不透明度は DefaultSpatial のバネの進みで、0 から 1 に収める
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

                // インジケーターはアイコン (と展開したときのラベル) の周りに余白を取った大きさにする
                // (`IndicatorCollapsedPadding`、`IndicatorExpandedPadding`)
                let pill = div()
                    .relative()
                    .flex()
                    .flex_none()
                    .max_w_full()
                    .items_center();
                let (pill, inline_label, below_label) = if top {
                    let pill = pill
                        .justify_center()
                        .w(dp(vertical::ACTIVE_INDICATOR_WIDTH))
                        .py((dp(vertical::ACTIVE_INDICATOR_HEIGHT) - icon_size) / 2.0);
                    let label = label
                        .max_w_full()
                        .text_center()
                        .mt(dp(vertical::ICON_LABEL_SPACE));
                    (pill, None, Some(label))
                } else {
                    let pill = pill
                        .py((dp(horizontal::ACTIVE_INDICATOR_HEIGHT) - icon_size) / 2.0)
                        .pl(dp(horizontal::FULL_WIDTH_LEADING_SPACE))
                        .pr(dp(horizontal::FULL_WIDTH_TRAILING_SPACE))
                        .gap(dp(horizontal::ICON_LABEL_SPACE));
                    (pill, Some(label), None)
                };
                let pill = pill
                    .child(indicator)
                    .children(state_layer)
                    .child(icon)
                    .children(inline_label)
                    .children(ring);

                let headless = HeadlessTab::new(item.id, item.value)
                    .track_focus(&focus_handle)
                    .aria_label(item.label)
                    .disabled(item.disabled)
                    .group(GROUP)
                    .flex()
                    .flex_col()
                    .justify_center()
                    .flex_none();
                let headless = if top {
                    // 折りたたんだ項目は 96dp の幅で、内容を左右 20dp の内側の中央に置き、
                    // 高さは 64dp を最小とする。項目の間は 4dp 空ける
                    headless
                        .items_center()
                        .w(dp(collapsed::CONTAINER_WIDTH))
                        .px(dp(
                            supplement::navigation_rail::TOP_ICON_ITEM_HORIZONTAL_PADDING,
                        ))
                        .min_h(dp(baseline::CONTAINER_HEIGHT))
                        .mt(if index > 0 {
                            dp(collapsed::ITEM_VERTICAL_SPACE)
                        } else {
                            px(0.0)
                        })
                } else {
                    // 展開した項目は内容の幅で、レールの内容の幅を上限とする
                    headless.items_start().max_w_full()
                };
                headless.child(pill).children(below_label)
            })
            .collect();

        let has_items = !items.is_empty();
        let top_space = dp(if self.expanded {
            expanded::TOP_SPACE
        } else {
            collapsed::TOP_SPACE
        });
        let center = self.arrangement == NavigationRailArrangement::Center;
        let header = self.header.map(|header| {
            // ヘッダーはレールの幅に引き伸ばさず、`header` 自身の幅で描く
            let header = div().flex().flex_none().child(header);
            if center {
                // 項目はヘッダーを除かずにレールの高さの中央に置くため、ヘッダーは並びから外して上端に置く
                header.absolute().top(top_space).left_0()
            } else {
                header.when(has_items, |this| {
                    this.mb(dp(baseline::HEADER_SPACE_MINIMUM))
                })
            }
        });

        let mut rail = TabList::new(self.id, self.selected)
            .orientation(TabListOrientation::Vertical)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .h_full()
            // 幅が動いている間に内容がはみ出さないよう、レールの形で切る
            .overflow_hidden()
            .bg(theme.color(collapsed::CONTAINER_COLOR));
        rail = match self.arrangement {
            NavigationRailArrangement::Top => rail.justify_start().pt(top_space),
            NavigationRailArrangement::Center => rail.justify_center(),
            NavigationRailArrangement::Bottom => rail.justify_end().pt(top_space),
        };
        rail.style().refine(&self.style);
        let rail = rail.children(header).items(items);

        // 幅は折りたたんだ 96dp と、展開したときの最小 220dp・最大 360dp の間をバネで動き、範囲の中で内容に合わせる。
        // 展開すると内容の左右に 20dp を空ける
        let collapsed_width = dp(collapsed::CONTAINER_WIDTH);
        let min_width = dp(expanded::CONTAINER_WIDTH_MINIMUM);
        let max_width = dp(expanded::CONTAINER_WIDTH_MAXIMUM);
        let padding = dp(supplement::navigation_rail::EXPANDED_HORIZONTAL_PADDING);
        rail.with_spring(
            "width",
            SpringAnimation::new(spatial).to(self.expanded),
            move |rail, phase| {
                // 行き過ぎで折りたたんだ幅より狭くならないよう、0 を下限にする
                let progress = phase.0.max(0.0);
                rail.min_w(collapsed_width + (min_width - collapsed_width) * progress)
                    .max_w(collapsed_width + (max_width - collapsed_width) * progress)
                    .px(padding * progress.min(1.0))
            },
        )
    }
}

impl<T: PartialEq + 'static> Styled for NavigationRail<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`NavigationRail`] の項目
///
/// [`NavigationRail::item`] に渡して使う。項目が表す値が、レールの選ばれている値と等しいときに選ばれた状態になる
pub struct NavigationRailItem<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    icon: StyledIcon,
    selected_icon: Option<StyledIcon>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
}

impl<T> NavigationRailItem<T> {
    /// `value` を表し、`icon` と `label` を描く項目を作る
    ///
    /// `label` はアクセシブルな名前にもなり、アイコンは装飾として名前に含めない。
    /// `id` と `value` はレールの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`NavigationRailItem::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
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
            disabled: false,
            focus_handle: None,
        }
    }

    /// 選ばれているあいだは `icon` を描く。塗りつぶしたアイコンで選ばれていることを示すときに使う
    pub fn selected_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.selected_icon = Some(icon.into());
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
    /// Tab で到達できるかはレールが決める。指定しない場合は項目が内部でフォーカスを持つ
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
            div().flex().w(px(800.0)).h(px(600.0)).child((self.0)())
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

    fn rail(expanded: bool, arrangement: NavigationRailArrangement) -> AnyElement {
        NavigationRail::new("rail", Some(0))
            .expanded(expanded)
            .arrangement(arrangement)
            .aria_label("Main")
            .items([
                NavigationRailItem::new("home", 0, HOME, "Home"),
                NavigationRailItem::new("search", 1, SEARCH, "Search"),
                NavigationRailItem::new("settings", 2, SETTINGS, "Settings"),
            ])
            .into_any_element()
    }

    #[gpui::test]
    fn 折りたたんだレールは_96dp_の幅でインジケーターを項目の中央に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || rail(false, NavigationRailArrangement::Top));
        let container = quads_of(cx, collapsed::CONTAINER_COLOR);
        assert!(
            container.contains(&(0.0, 0.0, 96.0, 600.0)),
            "レールは 96dp の幅で親の高さいっぱい: {container:?}"
        );
        // 上端から 44dp の後の 64dp の項目の中で、56×32 のインジケーターとラベル (4dp + 16dp) を上下中央に置く
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(indicator, vec![(20.0, 44.0 + 6.0, 56.0, 32.0)]);
    }

    #[gpui::test]
    fn 展開したレールは_220dp_を最小の幅にしてラベルの前にアイコンを置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || rail(true, NavigationRailArrangement::Top));
        let container = quads_of(cx, collapsed::CONTAINER_COLOR);
        assert!(
            container.contains(&(0.0, 0.0, 220.0, 600.0)),
            "内容が狭ければ 220dp: {container:?}"
        );
        // 左右 20dp の内側で、高さ 56dp のインジケーターがアイコン・間隔・ラベルを左右 16dp の余白で囲む
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(indicator.len(), 1);
        let (x, y, width, height) = indicator[0];
        assert_eq!((x, y, height), (20.0, 44.0, 56.0));
        assert!(
            width > 16.0 + 24.0 + 8.0 + 16.0,
            "ラベルの幅も囲む: {width}"
        );
    }

    #[gpui::test]
    fn 中央の並びはヘッダーがあっても項目をレールの高さの中央に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationRail::new("rail", Some(0))
                .arrangement(NavigationRailArrangement::Center)
                .header(div().size(px(56.0)))
                .item(NavigationRailItem::new("home", 0, HOME, "Home"))
                .into_any_element()
        });
        // 64dp の項目を 600dp の中央に置く
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(indicator, vec![(20.0, 268.0 + 6.0, 56.0, 32.0)]);
    }

    #[gpui::test]
    fn 上の並びはヘッダーと項目の間を_40dp_空ける(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            NavigationRail::new("rail", Some(0))
                .header(div().size(px(56.0)))
                .item(NavigationRailItem::new("home", 0, HOME, "Home"))
                .into_any_element()
        });
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(
            indicator,
            vec![(20.0, 44.0 + 56.0 + 40.0 + 6.0, 56.0, 32.0)]
        );
    }

    #[gpui::test]
    fn クリックと下の矢印キーで項目を選ぶよう求め_無効な項目では求めない(
        cx: &mut TestAppContext,
    ) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let recorder = changes.clone();
        let cx = open(cx, move || {
            let recorder = recorder.clone();
            NavigationRail::new("rail", Some(0))
                .on_change(move |value, _, _| recorder.borrow_mut().push(*value))
                .items([
                    NavigationRailItem::new("home", 0, HOME, "Home"),
                    NavigationRailItem::new("search", 1, SEARCH, "Search"),
                    NavigationRailItem::new("settings", 2, SETTINGS, "Settings").disabled(true),
                    NavigationRailItem::new("more", 3, SETTINGS, "More"),
                ])
                .into_any_element()
        });
        // 2 つ目と 3 つ目の項目の中央 (上端 44dp、項目 64dp、間隔 4dp)
        for y in [44.0 + 68.0 + 32.0, 44.0 + 136.0 + 32.0] {
            cx.simulate_click(point(px(48.0), px(y)), Modifiers::none());
        }
        assert_eq!(
            *changes.borrow(),
            vec![1],
            "無効な項目のクリックは選ぶよう求めない"
        );
        cx.simulate_keystrokes("down");
        assert_eq!(
            *changes.borrow(),
            vec![1, 3],
            "クリックでフォーカスした項目から、無効な項目を飛ばして次の項目へ移る"
        );
    }
}
