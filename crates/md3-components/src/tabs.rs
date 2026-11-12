// 寸法・色・タブの内容の配置・インジケーターの大きさと動き・内容の色の切り替えは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TabRow.kt
//         (`PrimaryTabRow`、`SecondaryTabRow`、`TabRowImpl`、`TabRowDefaults`、`TabIndicatorOffsetNode`)、
//         Tab.kt (`Tab`、`TabTransition`、`TabBaselineLayout`、`placeTextAndIcon`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// hover とフォーカスでの選ばれていないタブの内容の色と、内側に描くフォーカスリングは Compose にないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: tabs/internal/_tab.scss、tabs/internal/tab.ts (`md-focus-ring` の `inward`) @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のタブ

use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, Hsla, InteractiveElement, Interactivity,
    IntoElement, ParentElement, Pixels, Refineable, RenderOnce, SharedString, SpringAnimation,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, canvas, div,
    prelude::FluentBuilder, px, relative, size, svg,
};
use gpui_headless::tabs::{Tab as HeadlessTab, TabList, TabPanel as HeadlessTabPanel};

use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{
    divider, primary_navigation_tab as primary, secondary_navigation_tab as secondary,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp, shape_radii};

/// タブの中の層がタブの hover を知るための group 名
const GROUP: &str = "md3-tab";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// タブの並びの種類。インジケーターの形と、選ばれたタブの内容の色が決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TabRowVariant {
    /// 画面の主な内容を切り替える並び。選ばれたタブを Primary の色と、ラベルの幅のインジケーターで示す
    #[default]
    Primary,
    /// 内容の中の関連するまとまりを切り替える並び。選ばれたタブを、タブの幅いっぱいのインジケーターで示す
    Secondary,
}

/// MD3 のタブの並び
///
/// タブを同じ幅で横に並べ、選ばれているタブをインジケーターと内容の色で示す。
/// 選ばれたタブの内容は、呼び出し側が [`TabPanel`] に入れて並びの外に表示する。
/// 状態 (選ばれているタブの値) は呼び出し側が持ち、[`TabRow::new`] に毎回渡す。並び自身は状態を変えない。
/// タブは [`TabRow::tab`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できるタブは 1 つだけで、選ばれているタブ、最初のタブの順に選ぶ。
///   タブにフォーカスがある間は、そのタブだけが Tab で到達できる
/// - 左右の矢印キーで隣のタブへフォーカスを移し、移った先のタブを選ぶよう求める。端では反対の端に戻る。
///   Home と End では最初と最後のタブへ移り、移った先を選ぶよう求める
/// - フォーカス中の Enter と Space・マウスのクリックで、タブを選ぶよう求める
///
/// 支援技術には role `tablist` として公開され、[`TabRow::aria_label`] がアクセシブルな名前になる。
/// タブは role `tab` と選ばれているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルは並びの箱に付く。既定では親の幅いっぱいに広がり、
/// 高さは 48dp で、アイコンを持つタブがあれば 72dp になる。ラベルが折り返すタブがあれば、その高さまで伸びる。
/// 並びの下端に区切り線を引く
#[derive(IntoElement)]
pub struct TabRow<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    variant: TabRowVariant,
    on_change: Option<ChangeHandler<T>>,
    aria_label: Option<SharedString>,
    tabs: Vec<Tab<T>>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> TabRow<T> {
    /// `selected` の値を持つタブを選ばれた状態で表示する [`TabRowVariant::Primary`] の並びを作る。
    /// `None` ならどのタブも選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            variant: TabRowVariant::default(),
            on_change: None,
            aria_label: None,
            tabs: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 並びの種類を `variant` にする
    pub fn variant(mut self, variant: TabRowVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// タブを選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められたタブの値を渡す。選ばれているタブでは呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`TabRow::new`] に渡すこと。
    /// 設定しない場合、タブは操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾にタブを加える
    pub fn tab(mut self, tab: Tab<T>) -> Self {
        self.tabs.push(tab);
        self
    }

    /// 末尾に `tabs` のタブを順に加える
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab<T>>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for TabRow<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        // 選ばれたタブ、選ばれていないタブ、hover かフォーカスのある選ばれていないタブの内容の色。
        // アイコンとフォーカスの色のトークンは、それぞれラベルと hover の色と同じロールのため、まとめて扱う
        let (active, inactive, inactive_hover, container) = match self.variant {
            TabRowVariant::Primary => (
                primary::ACTIVE_LABEL_TEXT_COLOR,
                primary::INACTIVE_LABEL_TEXT_COLOR,
                primary::INACTIVE_HOVER_LABEL_TEXT_COLOR,
                primary::CONTAINER_COLOR,
            ),
            TabRowVariant::Secondary => (
                secondary::ACTIVE_LABEL_TEXT_COLOR,
                secondary::INACTIVE_LABEL_TEXT_COLOR,
                secondary::HOVER_LABEL_TEXT_COLOR,
                secondary::CONTAINER_COLOR,
            ),
        };
        let count = self.tabs.len();
        // Compose の並びは最も高いタブの高さになる。タブはアイコンとラベルの両方を持てば `LargeTabHeight`、
        // そうでなければ 48dp を最小とし、ラベルが折り返すと伸びる
        let min_height = if self.tabs.iter().any(|tab| tab.icon.is_some()) {
            dp(supplement::tabs::LARGE_TAB_HEIGHT)
        } else {
            dp(primary::CONTAINER_HEIGHT)
        };
        let label_font = theme.text_style(primary::LABEL_TEXT_FONT);
        let mut text_style = window.text_style();
        text_style.refine(&label_font);
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = text_style.line_height_in_pixels(window.rem_size());
        // タブの幅はレイアウトで決まるため、前の描画で測った幅を使う。
        // ラベルが折り返すかと、インジケーターの幅の上限に使う。最初の描画ではまだ測っていない
        let tab_width = window.with_element_namespace(self.id.clone(), |window| {
            window.use_keyed_state("tab-width", cx, |_, _| None::<Pixels>)
        });
        let label_space = tab_width
            .read(cx)
            .map(|width| width - dp(supplement::tabs::HORIZONTAL_TEXT_PADDING) * 2.0);

        // 同じ値のタブが複数あっても、支援技術に公開する状態と同じく最初のタブだけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        // 選ばれたタブの位置と、ラベルの幅
        let mut indicator = None;
        let tabs: Vec<_> = self
            .tabs
            .into_iter()
            .enumerate()
            .map(|(index, tab)| {
                let is_selected = selected
                    .take_if(|selected| **selected == tab.value)
                    .is_some();
                let line = window.text_system().shape_line(
                    tab.label.clone(),
                    font_size,
                    &[text_style.to_run(tab.label.len())],
                    None,
                );
                let wrapped = label_space.is_some_and(|space| line.width > space);
                if is_selected {
                    // gpui のテキストの要素は幅を切り上げてレイアウトし、折り返すと置ける幅いっぱいに広がる。
                    // Compose はタブの内容の幅に 24dp の最小値を課す (`TabRowImpl`)
                    let width = line.width.ceil();
                    let width = label_space.map_or(width, |space| width.min(space));
                    indicator = Some((
                        index,
                        width.max(dp(supplement::tabs::PRIMARY_INDICATOR_MIN_WIDTH)),
                    ));
                }
                let focus_handle = tab.focus_handle(&self.id, window, cx);
                let focused = focus_handle.is_focused(window);
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let active_color = theme.color(active);
                let rest_color = theme.color(if focused { inactive_hover } else { inactive });
                let hover_color = theme.color(if is_selected { active } else { inactive_hover });

                // アイコンとラベルの両方を持つタブは、ラベルの最後の基準線をタブの下端から、アイコンの下端をラベルの
                // 最初の基準線から決まった距離に置く (`placeTextAndIcon`)。下端からの距離はラベルが折り返すと変わる。
                // gpui は行の高さの中で文字の上下の余白を等しくして基準線を置く
                let baseline = (line_height - line.ascent - line.descent) / 2.0 + line.ascent;
                let last_baseline = dp(if wrapped {
                    supplement::tabs::DOUBLE_LINE_TEXT_BASELINE_WITH_ICON
                } else {
                    supplement::tabs::SINGLE_LINE_TEXT_BASELINE_WITH_ICON
                }) + dp(primary::ACTIVE_INDICATOR_HEIGHT);
                let label_bottom = (last_baseline - (line_height - baseline)).max(px(0.));
                let icon_distance = theme.sp(supplement::tabs::ICON_DISTANCE_FROM_BASELINE);
                let icon_gap = icon_distance - baseline;
                // Compose はタブを、アイコンとラベルの高さに `IconDistanceFromBaseline` を足した高さまで伸ばす。
                // そのときアイコンの上に空く距離
                let icon_top = (line_height - last_baseline).max(px(0.));
                let label = div()
                    .debug_selector(|| format!("md3-tab-label-{index}"))
                    .px(dp(supplement::tabs::HORIZONTAL_TEXT_PADDING))
                    .text_center()
                    .child(tab.label.clone());
                let icon = tab.icon;
                // 内容の色は、選ばれると DefaultEffects、選ばれなくなると FastEffects のバネで変える (`TabTransition`)
                let effects = theme.spring(if is_selected {
                    MotionSchemeKey::DefaultEffects
                } else {
                    MotionSchemeKey::FastEffects
                });
                let content = div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .group_hover(GROUP, |style| style.text_color(hover_color))
                    .with_spring(
                        "content-color",
                        SpringAnimation::new(effects).to(is_selected),
                        move |this, phase| {
                            let color =
                                Hsla::from(phase.interpolate_clamped(
                                    rest_color.to_rgb(),
                                    active_color.to_rgb(),
                                ));
                            let this = this.text_color(color);
                            match icon {
                                // Compose は内容の高さのタブを並びの高さの中で上下中央に置く
                                Some(icon) => this
                                    .min_h(dp(supplement::tabs::LARGE_TAB_HEIGHT))
                                    .justify_end()
                                    .pt(icon_top)
                                    .pb(label_bottom)
                                    .child(
                                        svg()
                                            .data(icon.svg().as_bytes())
                                            .flex_none()
                                            .size(dp(primary::ICON_SIZE))
                                            .mb(icon_gap)
                                            .text_color(color)
                                            .group_hover(GROUP, |style| {
                                                style.text_color(hover_color)
                                            }),
                                    ),
                                // Compose はアイコンの無いタブも、ラベルの高さに `IconDistanceFromBaseline` を足した高さまで伸ばす
                                None => {
                                    this.flex_grow(1.0).justify_center().py(icon_distance / 2.0)
                                }
                            }
                            .child(label)
                        },
                    );

                // 状態レイヤーは選ばれたタブの内容の色を重ねる (Compose の Tab の ripple は selectedContentColor を使う)
                let state_layer = div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .when(focus_visible, |this| {
                        this.bg(active_color.opacity(state::FOCUS_STATE_LAYER_OPACITY))
                    })
                    // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                    .group_hover(GROUP, |style| {
                        style.bg(active_color.opacity(state::HOVER_STATE_LAYER_OPACITY))
                    });

                let mut headless = HeadlessTab::new(tab.id, tab.value)
                    .track_focus(&focus_handle)
                    .aria_label(tab.label)
                    .group(GROUP)
                    .relative()
                    .flex_none()
                    .w(relative(1.0 / count as f32))
                    .flex()
                    .flex_col()
                    .justify_center();
                headless.text_style().refine(&label_font);
                // Material Web はリングをタブの内側に描き、選ばれたタブではインジケーターの上まで下端を上げる
                let ring = focus_visible.then(|| {
                    let inset = dp(focus_ring::INWARD_OFFSET);
                    let bottom = if is_selected {
                        dp(primary::ACTIVE_INDICATOR_HEIGHT)
                            + dp(supplement::tabs::FOCUS_RING_ACTIVE_BOTTOM_GAP)
                    } else {
                        inset
                    };
                    div()
                        .absolute()
                        .top(inset)
                        .left(inset)
                        .right(inset)
                        .bottom(bottom)
                        .rounded(dp(supplement::tabs::FOCUS_RING_CORNER_RADIUS))
                        .border(dp(focus_ring::WIDTH))
                        .border_color(theme.color(focus_ring::COLOR))
                });
                headless.child(state_layer).child(content).children(ring)
            })
            .collect();

        let mut list = TabList::new(self.id, self.selected)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .relative()
            .flex()
            .w_full()
            .min_h(min_height)
            .bg(theme.color(container));
        list.style().refine(&self.style);
        let divider = div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(dp(divider::THICKNESS))
            .bg(theme.color(divider::COLOR));
        let measure = canvas(
            move |bounds, window, cx| {
                let width = Some(bounds.size.width);
                if *tab_width.read(cx) != width {
                    tab_width.update(cx, |tab_width, _| *tab_width = width);
                    // 測った幅で描き直す
                    window.request_animation_frame();
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();
        let indicator = indicator.map(|(index, label_width)| {
            let indicator_height = dp(primary::ACTIVE_INDICATOR_HEIGHT);
            let color = theme.color(primary::ACTIVE_INDICATOR_COLOR);
            // インジケーターは選ばれたタブの下端に置き、位置と幅を DefaultSpatial のバネで動かす
            let spatial = theme.spring(MotionSchemeKey::DefaultSpatial);
            let slot = div()
                .absolute()
                .bottom_0()
                .w(relative(1.0))
                .h(indicator_height);
            let slot = match self.variant {
                // Compose の Primary のインジケーターは、タブの幅から左右の余白を除いた幅を上限とするラベルの幅で、
                // 最小 24dp にしてタブの中央に置く
                TabRowVariant::Primary => {
                    let min_width = dp(supplement::tabs::PRIMARY_INDICATOR_MIN_WIDTH);
                    let radii = shape_radii(
                        primary::ACTIVE_INDICATOR_SHAPE,
                        size(min_width, indicator_height),
                    );
                    let bar = div()
                        .flex_none()
                        .h_full()
                        .min_w(min_width)
                        .max_w_full()
                        .rounded_tl(radii.top_left)
                        .rounded_tr(radii.top_right)
                        .rounded_br(radii.bottom_right)
                        .rounded_bl(radii.bottom_left)
                        .bg(color);
                    // 幅は上限に収めてからバネに渡す。タブの幅をまだ測っていない最初の描画では、上限をレイアウトで課し、
                    // バネは測った後の描画で上限に収めた幅から始める
                    let bar = if label_space.is_some() {
                        bar.with_spring(
                            "indicator-width",
                            SpringAnimation::new(spatial).to(label_width),
                            |this, width| this.w(width),
                        )
                        .into_any_element()
                    } else {
                        bar.w(label_width).into_any_element()
                    };
                    slot.flex()
                        .justify_center()
                        .px(dp(supplement::tabs::HORIZONTAL_TEXT_PADDING))
                        .child(bar)
                }
                TabRowVariant::Secondary => slot.bg(color),
            };
            slot.with_spring(
                "indicator-position",
                SpringAnimation::new(spatial).to(index as f32),
                |this, position| this.left(relative(position)),
            )
        });
        // タブはどれも同じ幅のため、測る要素とインジケーターは最初のタブに置き、タブの幅を単位に位置を決める。
        // こうすると並びに padding を付けても、padding の内側に並ぶタブと同じ基準になる
        let mut tabs = tabs.into_iter();
        let first = tabs
            .next()
            .map(|first| first.child(measure).children(indicator));
        list.items(first.into_iter().chain(tabs)).child(divider)
    }
}

impl<T: PartialEq + 'static> Styled for TabRow<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`TabRow`] のタブ
///
/// [`TabRow::tab`] に渡して使う。タブが表す値が、並びの選ばれている値と等しいときに選ばれた状態になる
pub struct Tab<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    icon: Option<StyledIcon>,
    focus_handle: Option<FocusHandle>,
}

impl<T> Tab<T> {
    /// `value` を表し、`label` を描くタブを作る
    ///
    /// `label` はアクセシブルな名前にもなる。`id` と `value` は並びの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値のタブが複数あると、そのうち最初のタブだけが選ばれた状態になる。
    /// [`Tab::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            icon: None,
            focus_handle: None,
        }
    }

    /// ラベルの上に `icon` を描く
    ///
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// `focus_handle` をこのタブのフォーカスとして使う
    ///
    /// Tab で到達できるかは並びが決める。指定しない場合はタブが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, row_id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.focus_handle.clone().unwrap_or_else(|| {
            // タブの id は並びの中でだけ一意なので、並びの id で分ける
            window.with_element_namespace(row_id.clone(), |window| {
                util::keyed_focus_handle(&self.id, window, cx)
            })
        })
    }
}

/// [`TabRow`] で選ばれたタブの内容を入れるパネル
///
/// 支援技術には role `tabpanel` として公開される。名前は `aria_label` で、選ばれたタブのラベルと同じ名前を与えること。
/// パネル自身はフォーカスを持たない。パネルの中に Tab で到達できる要素が無い場合と、最初の内容がフォーカスできない場合は、
/// `tab_stop(true)` にしたフォーカスを `track_focus` で渡し、Tab でパネルに到達できるようにすること。
/// パネル自身は見た目を持たない。見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct TabPanel(HeadlessTabPanel);

impl TabPanel {
    /// 中身の無いパネルを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self(HeadlessTabPanel::new(id))
    }
}

impl RenderOnce for TabPanel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.0
    }
}

impl Styled for TabPanel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.0.style()
    }
}

impl ParentElement for TabPanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.0.extend(elements);
    }
}

impl InteractiveElement for TabPanel {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.0.interactivity()
    }
}

impl StatefulInteractiveElement for TabPanel {}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    /// 幅 `width` の親に置いた並びと、比べる用にラベルと同じ書体で描いた選ばれたタブのラベル
    struct RowView {
        selected: usize,
        variant: TabRowVariant,
        labels: &'static [&'static str],
        icons: bool,
        width: f32,
        /// 並びの左右の padding
        padding: f32,
        changes: Rc<RefCell<Vec<usize>>>,
    }

    impl Render for RowView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut probe = div()
                .flex_none()
                .bg(gpui::black())
                .child(self.labels[self.selected]);
            probe
                .text_style()
                .refine(&cx.md3_theme().text_style(primary::LABEL_TEXT_FONT));
            let changes = self.changes.clone();
            let icons = self.icons;
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(100.))
                .child(
                    div().w(px(self.width)).child(
                        TabRow::new("row", Some(self.selected))
                            .variant(self.variant)
                            .px(px(self.padding))
                            .aria_label("Tabs")
                            .on_change(move |value, _, _| changes.borrow_mut().push(*value))
                            .tabs(self.labels.iter().enumerate().map(|(index, label)| {
                                let tab = Tab::new(index, index, *label);
                                if icons {
                                    tab.icon(md3_icons::HOME)
                                } else {
                                    tab
                                }
                            })),
                    ),
                )
                .child(probe)
        }
    }

    fn add_row(
        cx: &mut TestAppContext,
        view: impl FnOnce(Rc<RefCell<Vec<usize>>>) -> RowView,
    ) -> (Rc<RefCell<Vec<usize>>>, &mut VisualTestContext) {
        cx.update(crate::init);
        let changes = Rc::new(RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            view(changes.clone())
        });
        // 状態レイヤーを描かないよう、マウスを並びの外に置く
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        (changes, cx)
    }

    fn row(
        selected: usize,
        labels: &'static [&'static str],
        width: f32,
    ) -> impl FnOnce(Rc<RefCell<Vec<usize>>>) -> RowView {
        move |changes| RowView {
            selected,
            variant: TabRowVariant::Primary,
            labels,
            icons: false,
            width,
            padding: 0.0,
            changes,
        }
    }

    /// 矩形の x, y, 幅, 高さと左上の角丸の半径 (論理ピクセル)
    type Rect = ((f32, f32, f32, f32), f32);

    /// `background` で塗った矩形。左から順に並べる
    fn filled(cx: &mut VisualTestContext, background: gpui::Background) -> Vec<Rect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut rects: Vec<Rect> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background && quad.border_widths.top.0 == 0.0)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        quad.corner_radii.top_left.0 / scale,
                    )
                })
                .collect();
            rects.sort_by(|a, b| a.0.0.total_cmp(&b.0.0));
            rects
        })
    }

    /// `color` の太さ 3dp の枠線で描いた矩形。gpui は塗りの無い枠線を辺ごとに分けて描くため、同じ矩形は 1 つにまとめる
    fn rings(cx: &mut VisualTestContext, color: gpui::Hsla) -> Vec<Rect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut rects: Vec<Rect> = Vec::new();
            for quad in window.painted_quads().into_iter().filter(|quad| {
                quad.border_widths.top.0 / scale == 3.0 && quad.border_color == color
            }) {
                let b = quad.bounds;
                let rect = (
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    ),
                    quad.corner_radii.top_left.0 / scale,
                );
                if !rects.contains(&rect) {
                    rects.push(rect);
                }
            }
            rects
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> gpui::Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    fn probe_width(cx: &mut VisualTestContext) -> f32 {
        filled(cx, gpui::black().into())
            .first()
            .expect("比べる用のラベルを描くこと")
            .0
            .2
    }

    #[gpui::test]
    fn 並びは_surface_の容器の下端に区切り線を引き高さは_48dp(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_row(cx, row(0, &["Flights", "Hotels"], 400.0));
        let surface = color(cx, ColorRole::Surface);
        let outline_variant = color(cx, ColorRole::OutlineVariant);
        assert_eq!(
            filled(cx, surface.into()),
            vec![((0.0, 0.0, 400.0, 48.0), 0.0)]
        );
        assert_eq!(
            filled(cx, outline_variant.into()),
            vec![((0.0, 47.0, 400.0, 1.0), 0.0)]
        );
    }

    #[gpui::test]
    fn アイコンを持つタブがあれば並びの高さは_72dp(cx: &mut TestAppContext) {
        let (_, cx) = add_row(cx, |changes| RowView {
            icons: true,
            ..row(0, &["Flights", "Hotels"], 400.0)(changes)
        });
        let surface = color(cx, ColorRole::Surface);
        let outline_variant = color(cx, ColorRole::OutlineVariant);
        assert_eq!(
            filled(cx, surface.into()),
            vec![((0.0, 0.0, 400.0, 72.0), 0.0)]
        );
        assert_eq!(
            filled(cx, outline_variant.into()),
            vec![((0.0, 71.0, 400.0, 1.0), 0.0)]
        );
    }

    #[gpui::test]
    fn ラベルが折り返すタブがあれば並びが伸びる(cx: &mut TestAppContext) {
        // タブの幅 50dp から左右の余白を除いた 18dp には収まらない
        let (_, cx) = add_row(cx, row(0, &["Specifications", "Reviews"], 100.0));
        let surface = color(cx, ColorRole::Surface);
        let outline_variant = color(cx, ColorRole::OutlineVariant);
        let ((_, _, _, height), _) = filled(cx, surface.into())[0];
        assert!(height > 48.0, "高さ {height} は 48dp より高いこと");
        let ((_, divider_y, _, _), _) = filled(cx, outline_variant.into())[0];
        assert_eq!(
            divider_y,
            height - 1.0,
            "区切り線は伸びた並びの下端に引くこと"
        );
    }

    /// 期待値は Compose の `TabBaselineLayout` の高さ。アイコン (24dp) とラベルの高さに
    /// `IconDistanceFromBaseline` (20sp) を足した高さが `LargeTabHeight` (72dp) を超えると伸びる
    #[gpui::test]
    fn アイコンを持つタブのラベルが折り返すと_compose_と同じ高さまで伸びる(
        cx: &mut TestAppContext,
    ) {
        // タブの幅 100dp から左右の余白を除いた 68dp に収まらず、2 行になる
        let (_, cx) = add_row(cx, |changes| RowView {
            icons: true,
            ..row(0, &["Favorites list", "Music"], 200.0)(changes)
        });
        let (line_height, icon_distance) = cx.update(|window, cx| {
            let theme = cx.md3_theme();
            let mut text_style = window.text_style();
            text_style.refine(&theme.text_style(primary::LABEL_TEXT_FONT));
            (
                f32::from(text_style.line_height_in_pixels(window.rem_size())),
                f32::from(theme.sp(supplement::tabs::ICON_DISTANCE_FROM_BASELINE)),
            )
        });
        let surface = color(cx, ColorRole::Surface);
        let ((_, _, _, height), _) = filled(cx, surface.into())[0];
        assert_eq!(height, 24.0 + line_height * 2.0 + icon_distance);
    }

    /// 期待値は Compose の `placeTextAndIcon`。最後の基準線からインジケーターの上端までの距離は、
    /// 1 行なら `SingleLineTextBaselineWithIcon` (14dp)、2 行以上なら `DoubleLineTextBaselineWithIcon` (6dp)
    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    #[gpui::test]
    fn アイコンを持つタブのラベルが折り返すと最後の基準線を下端に近づける(
        cx: &mut TestAppContext,
    ) {
        // タブの幅 100dp から左右の余白を除いた 68dp に収まらず、2 行になる
        let (_, cx) = add_row(cx, |changes| RowView {
            icons: true,
            ..row(0, &["Favorites list", "Music"], 200.0)(changes)
        });
        let surface = color(cx, ColorRole::Surface);
        let ((_, _, _, height), _) = filled(cx, surface.into())[0];
        let wrapped = cx
            .debug_bounds("md3-tab-label-0")
            .expect("ラベルを描くこと");

        // 同じラベルが 1 行に収まる幅
        let (_, cx) = add_row(cx, |changes| RowView {
            icons: true,
            ..row(0, &["Favorites list", "Music"], 600.0)(changes)
        });
        let single = cx
            .debug_bounds("md3-tab-label-0")
            .expect("ラベルを描くこと");
        // 行の高さと基準線の位置は 1 行目も 2 行目も同じため、ラベルの下端からタブの下端までの距離は
        // 2 つの距離の差だけ縮む
        assert_eq!(
            f32::from(px(72.) - single.bottom()) - f32::from(px(height) - wrapped.bottom()),
            8.0
        );
    }

    #[gpui::test]
    fn primary_のインジケーターは選ばれたタブの中央にラベルの幅で置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_row(cx, row(1, &["Flights", "Hotels"], 400.0));
        let primary = color(cx, ColorRole::Primary);
        let width = probe_width(cx);
        // 高さ 3dp に角丸 3dp を付けると、gpui は高さの半分の半径に収める
        assert_eq!(
            filled(cx, primary.into()),
            vec![((300.0 - width / 2.0, 45.0, width, 3.0), 1.5)]
        );
    }

    #[gpui::test]
    fn primary_のインジケーターは最小_24dp_でタブの幅から余白を除いた幅を超えない(
        cx: &mut TestAppContext,
    ) {
        const LABELS: &[&str] = &["A", "A label much longer than the tab can hold"];
        let (_, cx) = add_row(cx, row(0, LABELS, 400.0));
        let primary = color(cx, ColorRole::Primary);
        let ((x, _, width, _), _) = filled(cx, primary.into())[0];
        assert_eq!((x, width), (88.0, 24.0));

        let (_, cx) = add_row(cx, row(1, LABELS, 400.0));
        let primary = color(cx, ColorRole::Primary);
        let outline_variant = color(cx, ColorRole::OutlineVariant);
        let ((x, y, width, height), _) = filled(cx, primary.into())[0];
        assert_eq!((x, width), (216.0, 168.0));
        let ((_, divider_y, _, _), _) = filled(cx, outline_variant.into())[0];
        assert_eq!(
            y + height,
            divider_y + 1.0,
            "インジケーターは並びの下端に置くこと"
        );
    }

    #[gpui::test]
    fn secondary_のインジケーターはタブの幅いっぱいに置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_row(cx, |changes| RowView {
            variant: TabRowVariant::Secondary,
            ..row(1, &["Overview", "Reviews"], 400.0)(changes)
        });
        let primary = color(cx, ColorRole::Primary);
        assert_eq!(
            filled(cx, primary.into()),
            vec![((200.0, 45.0, 200.0, 3.0), 0.0)]
        );
    }

    #[gpui::test]
    fn インジケーターは並びの_padding_の内側に並ぶタブに合わせる(
        cx: &mut TestAppContext,
    ) {
        // 幅 240dp から左右の padding 20dp を除いた 200dp に、幅 100dp のタブが並ぶ
        let (_, cx) = add_row(cx, |changes| RowView {
            variant: TabRowVariant::Secondary,
            padding: 20.0,
            ..row(1, &["Overview", "Reviews"], 240.0)(changes)
        });
        let primary = color(cx, ColorRole::Primary);
        assert_eq!(
            filled(cx, primary.into()),
            vec![((120.0, 45.0, 100.0, 3.0), 0.0)]
        );
    }

    #[gpui::test]
    fn 矢印キーで隣のタブを選ぶよう求める(cx: &mut TestAppContext) {
        let (changes, cx) = add_row(cx, row(0, &["Flights", "Hotels", "Explore"], 600.0));
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab right");
        assert_eq!(*changes.borrow(), vec![1]);
    }

    #[gpui::test]
    fn フォーカスをタブの内側のリングで示し選ばれたタブではインジケーターの上まで上げる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_row(cx, row(0, &["Flights", "Hotels"], 400.0));
        let ring = color(cx, ColorRole::Secondary);
        assert_eq!(rings(cx, ring), vec![]);
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        // インジケーターの 3dp と隙間の 1dp だけ上げる
        assert_eq!(rings(cx, ring), vec![((0.0, 0.0, 200.0, 44.0), 8.0)]);

        // 並びは選ばれた値を変えないため、フォーカスだけが選ばれていないタブへ移る
        cx.simulate_keystrokes("right");
        assert_eq!(rings(cx, ring), vec![((200.0, 0.0, 200.0, 48.0), 8.0)]);
    }
}
