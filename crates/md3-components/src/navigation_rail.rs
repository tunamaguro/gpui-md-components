// 寸法・色・項目の並べ方・項目の中の配置・幅とインジケーターの動きは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/WideNavigationRail.kt
//         (`WideNavigationRail`、`WideNavigationRailLayout`、`WideNavigationRailItem`、`WideNavigationRailDefaults`、
//         `WideNavigationRailItemDefaults`、ファイル末尾の余白の値)、
//         ComponentStyles.kt (`NavigationRailStyle.Default`、`NavigationRailItemStyle.Default`)、
//         NavigationItem.kt (`TopIconOrIconOnlyMeasurePolicy`、`StartIconMeasurePolicy`、
//         `animateIndicatorProgressAsState`、`IndicatorRipple`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 (WideNavigationRail.kt、NavigationItem.kt)、2026 (ComponentStyles.kt) The Android Open Source Project.
// Licensed under the Apache License, Version 2.0.
// Compose の既定の設定はフォーカスをインジケーターの形の状態レイヤーだけで示す。この実装はそれに、ほかの部品と同じ外向きのフォーカスリングを
// インジケーターの形で加える。押している間の ripple は描かない。
// Compose は展開と折りたたみでアイコンの位置も動かすが、この実装はレールの幅だけを動かし、項目の配置はすぐに切り替える
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のナビゲーションレール

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, Refineable, RenderOnce, ScrollHandle, SharedString, SpringAnimation,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div,
    prelude::FluentBuilder, px, relative, svg,
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
    /// ヘッダーの下に残る高さの中央に並べる
    Center,
    /// 下端に詰めて並べる。ヘッダーは上端に残る
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
/// ヘッダーは並びの外に置き、並びの前の兄弟として公開される。
/// 項目がレールの高さに収まらないときは並びが縦にスクロールし、フォーカスのある項目を表示範囲に入れる
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
    /// ヘッダーと項目の間は 40dp 空ける。ヘッダーは `header` 自身の大きさで、レールの内容の先頭側の端に置く。
    /// 展開したときの幅は、ヘッダーの幅も含めて決まる
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
        let mut focus_handles = Vec::with_capacity(self.items.len());
        // 展開したときに内容で最小の幅より広がるかを決めるため、展開した項目のインジケーターが要る幅のうち
        // 最も広いものを、ラベルの幅から求める。折りたたんでいる間も求め、展開を始めたときの幅に使う
        let mut expanded_text_style = window.text_style();
        expanded_text_style.refine(&theme.text_style(horizontal::LABEL_TEXT_FONT));
        let expanded_font_size = expanded_text_style.font_size.to_pixels(window.rem_size());
        let widest_item = self
            .items
            .iter()
            .map(|item| {
                // 改行を含むラベルは最も長い行の幅にする
                let label = item
                    .label
                    .split('\n')
                    .map(|line| {
                        window
                            .text_system()
                            .shape_line(
                                SharedString::from(line.to_owned()),
                                expanded_font_size,
                                &[expanded_text_style.to_run(line.len())],
                                None,
                            )
                            .width
                            .ceil()
                    })
                    .fold(px(0.0), Pixels::max);
                dp(horizontal::FULL_WIDTH_LEADING_SPACE)
                    + icon_size
                    + dp(horizontal::ICON_LABEL_SPACE)
                    + label
                    + dp(horizontal::FULL_WIDTH_TRAILING_SPACE)
            })
            .fold(px(0.0), Pixels::max);
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
                focus_handles.push(focus_handle.clone());

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

                // インジケーターは選ばれると幅が先頭側から広がり、同時に濃くなる (`placeAnimatedLabelAndIcon` はインジケーターを
                // 先頭に置く)。幅と不透明度は DefaultSpatial のバネの進みで、0 から 1 に収める
                let indicator_theme = theme.clone();
                let indicator = div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .flex()
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
        // ヘッダーの幅を、レールの幅に引き伸ばされない内側の要素で測る
        let header_width = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("header-width", cx, |_, _| None::<Pixels>)
        });
        let header_measure = {
            let header_width = header_width.clone();
            canvas(
                move |bounds, window, cx| {
                    if *header_width.read(cx) != Some(bounds.size.width) {
                        header_width.update(cx, |width, _| *width = Some(bounds.size.width));
                        // 測った幅でレールの幅を決め直す
                        window.request_animation_frame();
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        };
        // フォーカスリングはインジケーターの外に張り出すため、スクロール領域の上下左右にその分の余白を取る。
        // 上の余白は負のマージンで打ち消し、上から詰める項目の位置を変えない。下の余白は打ち消さず、
        // 下端と中央に寄せた項目はその分だけ上に寄る。最後の項目までスクロールしてもリングが切れない
        let ring_outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
        // ヘッダーは並べ方にかかわらず上端に置き、レールの幅に引き伸ばさず `header` 自身の幅で描く。
        // ヘッダーは項目の並び (tablist) の外に置き、支援技術には並びの前の兄弟として公開する
        let has_header = self.header.is_some();
        let header = self.header.map(|header| {
            div()
                .relative()
                .flex()
                .flex_none()
                .when(has_items, |this| {
                    this.mb(dp(baseline::HEADER_SPACE_MINIMUM))
                })
                // 展開した配置では、項目の並びと同じくレールの余白から引いたリングの張り出し分を空ける
                .when(self.expanded, |this| this.px(ring_outset))
                .child(
                    div()
                        .relative()
                        .flex_none()
                        .child(header)
                        .child(header_measure),
                )
        });

        // 項目がレールの高さに収まらないときは並びを縦にスクロールし、キーボードでフォーカスが移った項目を表示範囲に入れる。
        // 中央と下端に寄せる並べ方は、項目の前後の縮む余白で寄せ、項目があふれたら余白を 0 にしてスクロールする
        let (scroll_handle, last_focused) =
            window.with_element_namespace(self.id.clone(), |window| {
                (
                    window.use_keyed_state("scroll", cx, |_, _| ScrollHandle::new()),
                    window.use_keyed_state("last-focused", cx, |_, _| {
                        (None::<usize>, px(0.0), None::<(Pixels, Pixels)>)
                    }),
                )
            });
        let scroll_handle = scroll_handle.read(cx).clone();
        let leading_space = matches!(
            self.arrangement,
            NavigationRailArrangement::Center | NavigationRailArrangement::Bottom
        );
        let trailing_space = self.arrangement == NavigationRailArrangement::Center;
        let focused = focus_handles
            .iter()
            .position(|handle| handle.is_focused(window));
        // Compose の中央の並べ方は項目をヘッダーを除かずにレールの高さの中央に置くが、
        // この実装はヘッダーの下に残る高さの中央に置く
        let list = TabList::new(self.id.clone(), self.selected)
            .orientation(TabListOrientation::Vertical)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .mt(-ring_outset)
            // 折りたたんだ項目は内側に余白を持つため、左右の余白を負のマージンで打ち消してレールの幅に合わせる。
            // 展開した項目は左に寄るため打ち消さず、その分をレールの左右の余白から引く
            .when(!self.expanded, |this| this.mx(-ring_outset))
            .py(ring_outset)
            .px(ring_outset)
            .overflow_y_scroll()
            .track_scroll(&scroll_handle)
            .when(leading_space, |this| this.child(div().flex_1()))
            .items(items)
            .when(trailing_space, |this| this.child(div().flex_1()));

        // 折りたたむ間の幅は前の描画の幅を上限にするため、描いた幅を記録する。
        // あわせて、展開したときに内容で最小の幅より広がったかを記録する
        let padding = dp(supplement::navigation_rail::EXPANDED_HORIZONTAL_PADDING);
        let collapsed_width = dp(collapsed::CONTAINER_WIDTH);
        let min_width = dp(expanded::CONTAINER_WIDTH_MINIMUM);
        let max_width = dp(expanded::CONTAINER_WIDTH_MAXIMUM);
        let painted_width = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("painted-width", cx, |_, _| None::<Pixels>)
        });
        let collapsing_from = (!self.expanded).then(|| *painted_width.read(cx)).flatten();
        // ヘッダーは前の描画で測った幅を使う。ヘッダーは展開と折りたたみで配置を変えないため、
        // 展開を始めたときには測り終えている
        let header_width = if has_header {
            header_width.read(cx).unwrap_or_default()
        } else {
            px(0.0)
        };
        // 展開したときの幅。内容が最小の幅に収まるなら最小の幅にする
        let content_width = widest_item.max(header_width) + padding * 2.0;
        let widened = content_width > min_width;
        let expanded_width = content_width.clamp(min_width, max_width);
        // フォーカスが移ったとき・表示領域の高さが変わったとき・項目の配置が変わったときに、
        // フォーカスのある項目をリングの張り出しも含めて表示範囲に入れる。
        // 並びの後に置いた canvas の prepaint で、この描画のレイアウトで決まった範囲を使う
        let expanded = self.expanded;
        let focused_child = focused.map(|index| index + usize::from(leading_space));
        let scroll = scroll_handle.clone();
        let measure = canvas(
            move |bounds, window, cx| {
                if *painted_width.read(cx) != Some(bounds.size.width) {
                    painted_width.update(cx, |width, _| *width = Some(bounds.size.width));
                }
                let viewport = scroll.bounds();
                let item = focused_child.and_then(|child| scroll.bounds_for_item(child));
                // 子の範囲はスクロールする前の位置なので、表示領域からの位置はスクロールで変わらず、配置が変わると変わる
                let placement =
                    item.map(|item| (item.top() - viewport.top(), item.bottom() - viewport.top()));
                let key = (focused, viewport.size.height, placement);
                if key == *last_focused.read(cx) {
                    return;
                }
                last_focused.update(cx, |last, _| *last = key);
                let Some(item) = item else {
                    return;
                };
                // 今のオフセットを足して画面上の位置にする
                let mut offset = scroll.offset();
                let top = item.top() + offset.y - ring_outset;
                let bottom = item.bottom() + offset.y + ring_outset;
                if top < viewport.top() {
                    offset.y += viewport.top() - top;
                } else if bottom > viewport.bottom() {
                    offset.y -= bottom - viewport.bottom();
                }
                if offset != scroll.offset() {
                    scroll.set_offset(offset);
                    // 変えたオフセットで描き直す
                    window.request_animation_frame();
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();

        let rail = div()
            .id((self.id.clone(), "rail"))
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .h_full()
            .pt(top_space)
            // 幅が動いている間に内容がはみ出さないよう、レールの形で切る
            .overflow_hidden()
            .bg(theme.color(collapsed::CONTAINER_COLOR))
            .children(header)
            .child(list)
            .child(measure);

        // 幅は折りたたんだ 96dp と、展開したときの最小 220dp・最大 360dp の間をバネで動く。
        // 展開した項目とヘッダーが最小の幅に収まるなら、展開と折りたたみのどちらも最小の幅の動きにする (`actualMinWidth`)。
        // 収まらないなら、展開する間と展開した後は最大の幅の動きを内容の幅で頭打ちにし
        // (`widthFullRange.coerceAtMost(widthConstrain)`)、折りたたむ間は、最大の幅の動きを、
        // 最小の幅の動きと前の描画の幅の間に収めた幅にする (`widthFullRange.coerceIn(actualMinWidth, currentWidth)`)。
        // そのため、どの時点で折りたたみ始めても、描いていた幅から続けて縮む。展開すると内容の左右に 20dp を空ける。
        // 呼び出し側のスタイルは、幅と余白の既定の後に重ねる
        let style = self.style;
        rail.with_spring(
            (self.id, "width"),
            SpringAnimation::new(spatial).to(self.expanded),
            move |rail, phase| {
                // 行き過ぎで折りたたんだ幅より狭くならないよう、0 を下限にする
                let progress = phase.0.max(0.0);
                let min = collapsed_width + (min_width - collapsed_width) * progress;
                // 行き過ぎで展開したときの最大の幅を超えないよう、最大の幅で頭打ちにする
                let max =
                    (collapsed_width + (max_width - collapsed_width) * progress).min(max_width);
                let width = match collapsing_from {
                    _ if !widened => min,
                    Some(from) => max.clamp(min, from.max(min)),
                    None => max.min(expanded_width),
                };
                let rail = rail.w(width);
                // 展開の配置では並びがフォーカスリングの張り出し分を左右に空けるため、その分を除く
                let padding = if expanded {
                    (padding * progress.min(1.0) - ring_outset).max(px(0.0))
                } else {
                    padding * progress.min(1.0)
                };
                let mut rail = rail.px(padding);
                rail.style().refine(&style);
                rail
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
        cx.update(crate::init);
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
    fn 中央と下の並びはヘッダーを上端に残し項目だけを寄せる(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            let rail = |id: &'static str, arrangement| {
                NavigationRail::new(id, Some(0))
                    .arrangement(arrangement)
                    .header(div().size(px(56.0)).bg(gpui::red()))
                    .item(NavigationRailItem::new("home", 0, HOME, "Home"))
            };
            div()
                .flex()
                .size_full()
                .child(rail("center", NavigationRailArrangement::Center))
                .child(rail("bottom", NavigationRailArrangement::Bottom))
                .into_any_element()
        });
        let headers: Vec<(f32, f32)> = cx.update(|window, _| {
            let scale = window.scale_factor();
            let red: gpui::Background = gpui::red().into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == red)
                .map(|quad| {
                    (
                        quad.bounds.origin.x.0 / scale,
                        quad.bounds.origin.y.0 / scale,
                    )
                })
                .collect()
        });
        assert_eq!(
            headers,
            vec![(0.0, 44.0), (96.0, 44.0)],
            "ヘッダーはどちらも上端"
        );
        // ヘッダーと 40dp の間隔の下に残る 460dp から、フォーカスリングのための下の余白 5dp を除いた範囲の
        // 中央と下端に、64dp の項目を置く
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(
            indicator,
            vec![
                (20.0, 140.0 + 195.5 + 6.0, 56.0, 32.0),
                (96.0 + 20.0, 600.0 - 5.0 - 64.0 + 6.0, 56.0, 32.0),
            ]
        );
    }

    #[gpui::test]
    fn 並べたレールは幅のアニメーションを共有しない(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            div()
                .flex()
                .size_full()
                .child(
                    NavigationRail::new("a", Some(0))
                        .item(NavigationRailItem::new("home", 0, HOME, "Home")),
                )
                .child(
                    NavigationRail::new("b", Some(0))
                        .expanded(true)
                        .item(NavigationRailItem::new("home", 0, HOME, "Home")),
                )
                .into_any_element()
        });
        let container = quads_of(cx, collapsed::CONTAINER_COLOR);
        assert!(
            container.contains(&(0.0, 0.0, 96.0, 600.0))
                && container.contains(&(96.0, 0.0, 220.0, 600.0)),
            "最初の描画からそれぞれ 96dp と 220dp: {container:?}"
        );
    }

    /// フォーカスリングが、切り抜きの範囲を合わせた範囲に収まっているか。
    /// 角の形に切り抜くため、1 つのリングを切り抜きの範囲の違う複数の矩形で描く
    fn ring_fits(cx: &mut VisualTestContext) -> bool {
        cx.update(|window, cx| {
            let color = cx.md3_theme().color(focus_ring::COLOR);
            let ring: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_color == color && quad.border_widths.top.0 > 0.)
                .collect();
            let Some(first) = ring.first() else {
                return false;
            };
            let mask = ring.iter().fold(first.content_mask.bounds, |mask, quad| {
                mask.union(&quad.content_mask.bounds)
            });
            let ring = first.bounds;
            ring.top() >= mask.top()
                && ring.bottom() <= mask.bottom()
                && ring.left() >= mask.left()
                && ring.right() <= mask.right()
        })
    }

    #[gpui::test]
    fn 高さに収まらない項目へキーボードで移るとリングまで含めてスクロールして表示する(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationRail::new("rail", Some(0))
                .expanded(true)
                .items(
                    (0..12usize)
                        .map(|index| NavigationRailItem::new(("item", index), index, HOME, "Item")),
                )
                .into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        // スクロールしなければ最後の項目は上端から 44 + 56 × 11 = 660dp にあり、表示範囲の外になる
        cx.simulate_keystrokes("tab end");
        // 描いたレイアウトでスクロールを補正し、補正したオフセットで次のフレームを描く
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            ring_fits(cx),
            "End で移った最後の項目のリングが切れないこと"
        );
        cx.simulate_keystrokes("home");
        // 描いたレイアウトでスクロールを補正し、補正したオフセットで次のフレームを描く
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            ring_fits(cx),
            "Home で移った最初の項目のリングが切れないこと"
        );
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

    #[gpui::test]
    fn 折りたたんで項目の配置が変わってもフォーカスのある項目を表示する(
        cx: &mut TestAppContext,
    ) {
        let expanded = Rc::new(std::cell::Cell::new(true));
        let state = expanded.clone();
        let cx = open(cx, move || {
            NavigationRail::new("rail", Some(0))
                .expanded(state.get())
                .items(
                    (0..12usize)
                        .map(|index| NavigationRailItem::new(("item", index), index, HOME, "Item")),
                )
                .into_any_element()
        });
        // バネを終端まで即座に進める
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab end");
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(ring_fits(cx), "展開した配置で最後の項目を表示すること");
        // 折りたたむと項目の高さが 56dp から 68dp に増え、同じオフセットでは最後の項目が表示範囲の外に出る
        expanded.set(false);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            ring_fits(cx),
            "折りたたんだ配置でも最後の項目を表示すること"
        );
    }

    #[gpui::test]
    fn 項目の高さが変わってもフォーカスのある項目を表示する(
        cx: &mut TestAppContext,
    ) {
        let two_lines = Rc::new(std::cell::Cell::new(false));
        let state = two_lines.clone();
        let cx = open(cx, move || {
            let label = if state.get() { "Item\nItem" } else { "Item" };
            NavigationRail::new("rail", Some(0))
                .expanded(true)
                .items(
                    (0..12usize)
                        .map(|index| NavigationRailItem::new(("item", index), index, HOME, label)),
                )
                .into_any_element()
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab end");
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(ring_fits(cx), "1 行のラベルで最後の項目を表示すること");
        // ラベルを 2 行にすると項目が高くなり、同じオフセットでは最後の項目が表示範囲の外に出る
        two_lines.set(true);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(ring_fits(cx), "2 行のラベルでも最後の項目を表示すること");
    }

    #[gpui::test]
    fn 左右の余白を消しても展開した項目のリングが切れない(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationRail::new("rail", Some(0))
                .expanded(true)
                .px_0()
                .item(NavigationRailItem::new("item", 0usize, HOME, "Item"))
                .into_any_element()
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        assert!(ring_fits(cx), "レールの外枠でリングが切れないこと");
    }

    #[gpui::test]
    fn 展開したレールはヘッダーと項目を左端から_20dp_に置く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            NavigationRail::new("rail", Some(0))
                .expanded(true)
                .header(div().size(px(56.0)).bg(gpui::red()))
                .item(NavigationRailItem::new("home", 0, HOME, "Home"))
                .into_any_element()
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let header_x: Vec<f32> = cx.update(|window, _| {
            let scale = window.scale_factor();
            let red: gpui::Background = gpui::red().into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == red)
                .map(|quad| quad.bounds.origin.x.0 / scale)
                .collect()
        });
        assert_eq!(header_x, vec![20.0], "ヘッダーは左端から 20dp");
        let indicator = quads_of(cx, color::ITEM_ACTIVE_INDICATOR);
        assert_eq!(indicator[0].0, 20.0, "インジケーターは左端から 20dp");
    }

    #[gpui::test]
    fn 展開した幅は内容が最小の幅に収まれば_220dp_で_収まらなければ内容に合わせる(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            let rail = |id: &'static str, label: &'static str| {
                NavigationRail::new(id, Some(0))
                    .expanded(true)
                    .header(div().size(px(56.0)))
                    .item(NavigationRailItem::new("item", 0, HOME, label))
            };
            div()
                .flex()
                .size_full()
                .child(rail("short", "Home"))
                .child(rail("long", "A destination with a long label"))
                .into_any_element()
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let containers = quads_of(cx, collapsed::CONTAINER_COLOR);
        assert_eq!(
            containers[0].2, 220.0,
            "短いラベルとヘッダーのレールは 220dp"
        );
        assert!(
            containers[1].2 > 220.0 && containers[1].2 <= 360.0,
            "長いラベルのレールは内容に合わせて広がること: {}",
            containers[1].2
        );
    }

    #[gpui::test]
    fn 広いヘッダーを測ってレールを広げ_外すと戻す(cx: &mut TestAppContext) {
        let with_header = Rc::new(std::cell::Cell::new(true));
        let state = with_header.clone();
        let cx = open(cx, move || {
            NavigationRail::new("rail", Some(0))
                .expanded(true)
                .when(state.get(), |rail| {
                    rail.header(div().w(px(240.0)).h(px(40.0)))
                })
                .item(NavigationRailItem::new("item", 0, HOME, "Home"))
                .into_any_element()
        });
        cx.update(|_, cx| cx.set_reduce_motion(true));
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        // 最初の描画で測ったヘッダーの幅で、次のフレームを描く
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        let width = |cx: &mut VisualTestContext| quads_of(cx, collapsed::CONTAINER_COLOR)[0].2;
        assert_eq!(width(cx), 280.0, "ヘッダーの幅と左右 20dp に広がること");
        with_header.set(false);
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        assert_eq!(width(cx), 220.0, "ヘッダーを外すと最小の幅に戻ること");
    }
}
