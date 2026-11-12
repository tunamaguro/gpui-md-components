// 寸法・色・項目の形・選択の印の出し方・項目を重ねる順は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt
//         (`SegmentedButton`、`SingleChoiceSegmentedButtonRow`、`MultiChoiceSegmentedButtonRow`、
//         `SegmentedButtonContentMeasurePolicy`、`SegmentedButtonDefaults`、`SegmentedButtonColors`、
//         `interactionZIndex`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// フォーカスのある項目のリングを隣の項目より前に描くことは Material Web に合わせている
// (https://github.com/material-components/material-web)
// Source: labs/segmentedbutton/internal/_shared.scss (`.md3-segmented-button__focus-ring` の z-index) @ de5470f
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 の Segmented button

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnimationExt, App, BorderStyle, Bounds, Corners, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Refineable, RenderOnce, SharedString,
    SpringAnimation, StatefulInteractiveElement, StyleRefinement, Styled, Transformation, Window,
    canvas, div, fill, point, prelude::FluentBuilder, px, quad, size, svg, transparent_black,
};
use gpui_headless::radio::{Radio as HeadlessRadio, RadioGroup as HeadlessRadioGroup};
use gpui_headless::toggle::{ToggleButton as HeadlessToggleButton, ToggleGroup};

use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{button_small, outlined_segmented_button as tokens};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層が項目の hover を知るための group 名
const GROUP: &str = "md3-segmented-button";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// 項目に共通する値
struct Segment {
    id: ElementId,
    label: SharedString,
    icon: Option<StyledIcon>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
}

/// 並び全体で決まる、項目の見た目の値
struct RowLook {
    theme: SharedTheme,
    /// 項目の数
    count: usize,
    /// 項目の最小の幅。どの項目も、最も広い項目に合わせた同じ幅にする
    width: Pixels,
}

impl RowLook {
    /// `labels` のラベルを持つ項目を並べるときの値
    fn new<'a>(
        theme: SharedTheme,
        labels: impl ExactSizeIterator<Item = &'a SharedString>,
        window: &mut Window,
    ) -> Self {
        let count = labels.len();
        // Compose の行は `IntrinsicSize.Min` の幅で項目に同じ重みを付け、どの項目も最も広い項目の幅にする。
        // 幅は、ラベルを描く書体で測った幅に、余白とアイコンの場所と間隔を足して求める
        let mut text_style = window.text_style();
        text_style.refine(&theme.text_style(tokens::LABEL_TEXT_FONT));
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let width = labels
            .map(|label| {
                let label_width = window
                    .text_system()
                    .shape_line(
                        label.clone(),
                        font_size,
                        &[text_style.to_run(label.len())],
                        None,
                    )
                    .width
                    // gpui のテキストの要素は幅を切り上げてレイアウトする
                    .ceil();
                dp(supplement::segmented_button::HORIZONTAL_PADDING) * 2.0
                    + dp(tokens::ICON_SIZE)
                    + dp(supplement::segmented_button::ICON_SPACING)
                    + label_width
            })
            .fold(dp(supplement::button::MIN_WIDTH), Pixels::max);
        Self {
            theme,
            count,
            width,
        }
    }

    /// `layers` の容器と枠線を描く要素と、フォーカスのリングを描く要素
    ///
    /// 前者は項目より前、後者は項目より後に並びの子として加える。どちらも、項目が記録した位置に描く
    fn layers(&self, layers: Vec<SegmentLayer>) -> (impl IntoElement, impl IntoElement) {
        let layers = Rc::new(layers);
        let theme = self.theme.clone();
        let paint_layers = {
            let layers = layers.clone();
            let theme = theme.clone();
            move |_, _, window: &mut Window, _: &mut App| {
                // 選ばれた項目の容器と枠線を、重なる隣の項目の枠線より前に描く
                // (Compose の `interactionZIndex` は選ばれた項目を前面に置く)
                let (selected, unselected): (Vec<_>, Vec<_>) =
                    layers.iter().partition(|layer| layer.active);
                for layer in unselected.into_iter().chain(selected) {
                    let Some(bounds) = layer.bounds.get() else {
                        continue;
                    };
                    let radii = layer.radii(&theme, bounds, px(0.0));
                    if let Some(container) = layer.container {
                        window.paint_quad(fill(bounds, container).corner_radii(radii));
                    }
                    window.paint_quad(quad(
                        bounds,
                        radii,
                        transparent_black(),
                        dp(tokens::OUTLINE_WIDTH),
                        layer.outline,
                        BorderStyle::Solid,
                    ));
                }
            }
        };
        // 後ろに並ぶ項目に隠れないよう、すべての項目を描いた後に描く。
        // ほかの要素と同じく並びの中で描くため、祖先の切り取りの範囲と不透明度が効く
        let paint_ring = move |_, _, window: &mut Window, _: &mut App| {
            let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
            for layer in layers.iter().filter(|layer| layer.focus_visible) {
                let Some(bounds) = layer.bounds.get() else {
                    continue;
                };
                window.paint_quad(quad(
                    bounds.dilate(outset),
                    layer.radii(&theme, bounds, outset),
                    transparent_black(),
                    dp(focus_ring::WIDTH),
                    theme.color(focus_ring::COLOR),
                    BorderStyle::Solid,
                ));
            }
        };
        (
            canvas(|_, _, _| {}, paint_layers).absolute(),
            canvas(|_, _, _| {}, paint_ring).absolute(),
        )
    }
}

/// 並びが項目の位置に描く、項目の容器と枠線とフォーカスのリング
struct SegmentLayer {
    /// 項目の位置。項目の prepaint で記録する
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    container: Option<Hsla>,
    outline: Hsla,
    /// 形の角丸を付ける角
    corners: Corners<bool>,
    active: bool,
    focus_visible: bool,
}

impl SegmentLayer {
    /// 位置 `bounds` の項目の形を外側へ `outset` だけ広げた角丸
    fn radii(
        &self,
        theme: &SharedTheme,
        bounds: Bounds<Pixels>,
        outset: Pixels,
    ) -> Corners<Pixels> {
        let radii = theme.corner_radii(tokens::SHAPE, bounds.size);
        let radius =
            |radius: Pixels, rounded: bool| if rounded { radius } else { px(0.0) } + outset;
        Corners {
            top_left: radius(radii.top_left, self.corners.top_left),
            top_right: radius(radii.top_right, self.corners.top_right),
            bottom_right: radius(radii.bottom_right, self.corners.bottom_right),
            bottom_left: radius(radii.bottom_left, self.corners.bottom_left),
        }
    }
}

impl Segment {
    fn new(id: ElementId, label: SharedString) -> Self {
        Self {
            id,
            label,
            icon: None,
            disabled: false,
            focus_handle: None,
        }
    }

    /// グループ `group_id` の項目として使うフォーカス
    fn focus_handle(&self, group_id: &ElementId, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.focus_handle.clone().unwrap_or_else(|| {
            // 項目の id はグループの中でだけ一意なので、グループの id で分ける
            window.with_element_namespace(group_id.clone(), |window| {
                util::keyed_focus_handle(&self.id, window, cx)
            })
        })
    }

    /// 並びの `index` 番目の項目の見た目を `item` に付け、並びが描く容器と枠線とリングを返す
    ///
    /// `active` は選ばれているか、`focus_visible` はフォーカスを示すか
    fn decorate<E: Styled + ParentElement + InteractiveElement>(
        self,
        item: E,
        active: bool,
        index: usize,
        focus_visible: bool,
        row: &RowLook,
    ) -> (E, SegmentLayer) {
        let theme = &row.theme;
        let disabled = self.disabled;

        // `defaultSegmentedButtonColors`。無効でも選ばれている項目は容器の色を保つ
        let content = if disabled {
            theme
                .color(tokens::DISABLED_LABEL_TEXT_COLOR)
                .opacity(tokens::DISABLED_LABEL_TEXT_OPACITY)
        } else if active {
            theme.color(tokens::SELECTED_LABEL_TEXT_COLOR)
        } else {
            theme.color(tokens::UNSELECTED_LABEL_TEXT_COLOR)
        };
        let container = active.then(|| theme.color(tokens::SELECTED_CONTAINER_COLOR));
        let outline = if disabled {
            // Compose の `defaultSegmentedButtonColors` は、OnSurface のトークン (DisabledOutlineColor) ではなく
            // Outline に無効の不透明度を付ける
            theme
                .color(tokens::OUTLINE_COLOR)
                .opacity(tokens::DISABLED_OUTLINE_OPACITY)
        } else {
            theme.color(tokens::OUTLINE_COLOR)
        };
        // `SegmentedButtonDefaults.itemShape`。1 つだけなら形のまま、最初は左、最後は右の角だけを丸め、ほかは直角にする
        let first = index == 0;
        let last = index + 1 == row.count;
        let corners = Corners {
            top_left: first,
            top_right: last,
            bottom_right: last,
            bottom_left: first,
        };

        // 容器と枠線は隣の項目と重ねる順を選ぶため並びが描き、項目は記録した位置を並びに渡す
        let bounds = Rc::new(Cell::new(None));
        let probe = {
            let bounds = bounds.clone();
            canvas(move |item, _, _| bounds.set(Some(item)), |_, _, _, _| {})
                .absolute()
                .size_full()
        };
        // 状態レイヤーは内容の色を、枠線の内側に重ねる (Compose の `ripple()` は LocalContentColor を使い、
        // 枠線は内容の上に描く)。角丸は描画時の大きさで決まるため Shaped で包む
        let state_layer = div()
            .absolute()
            .top(dp(tokens::OUTLINE_WIDTH))
            .left(dp(tokens::OUTLINE_WIDTH))
            .right(dp(tokens::OUTLINE_WIDTH))
            .bottom(dp(tokens::OUTLINE_WIDTH))
            .when(!disabled, |this| {
                this.when(focus_visible, |this| {
                    this.bg(content.opacity(state::FOCUS_STATE_LAYER_OPACITY))
                })
                // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                .group_hover(GROUP, |style| {
                    style.bg(content.opacity(state::HOVER_STATE_LAYER_OPACITY))
                })
            });

        let icon_size = dp(tokens::ICON_SIZE);
        let spatial = theme.spring(MotionSchemeKey::FastSpatial);
        let effects = theme.spring(MotionSchemeKey::DefaultEffects);
        let check = svg()
            .data(StyledIcon::new(md3_icons::CHECK).svg().as_bytes())
            .size(icon_size)
            .text_color(content);
        // 選択の印。Compose の `SegmentedButtonDefaults.Icon` に合わせる
        let icon_slot = div().relative().flex_none().size(icon_size).map(|slot| {
            match self.icon {
                // 選ばれていないときのアイコンがあれば、印と重ねて不透明度を入れ替える (`Crossfade`)
                Some(icon) => slot
                    .child(
                        div()
                            .absolute()
                            .child(
                                svg()
                                    .data(icon.svg().as_bytes())
                                    .size(icon_size)
                                    .text_color(content),
                            )
                            .with_spring(
                                "inactive-icon",
                                SpringAnimation::new(effects).to(!active),
                                |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
                            ),
                    )
                    .child(div().absolute().child(check).with_spring(
                        "active-icon",
                        SpringAnimation::new(effects).to(active),
                        |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
                    )),
                // 無ければ、選ばれると印を左下から拡大しながら表示し (`fadeIn` と `scaleIn`)、
                // 選ばれなくなるとすぐに消す (`ExitTransition.None`)
                None => slot.child(
                    div()
                        .absolute()
                        .child(check.with_spring(
                            "check-scale",
                            SpringAnimation::new(spatial).to(active),
                            move |this, phase| {
                                let scale = phase.0.max(0.0);
                                // Transformation は中心を基準に拡大するため、左下が動かないようずらす
                                let shift = icon_size * ((1.0 - scale) / 2.0);
                                this.with_transformation(
                                    Transformation::scale(size(scale, scale))
                                        .with_translation(point(-shift, shift)),
                                )
                            },
                        ))
                        .with_spring(
                            "check-opacity",
                            SpringAnimation::new(effects).to(active),
                            move |this, phase| {
                                this.opacity(if active { phase.0.clamp(0.0, 1.0) } else { 0.0 })
                            },
                        ),
                ),
            }
        });
        let has_icon = self.icon.is_some();
        // 印の場所はアイコンの有無によらず確保する。印もアイコンも無いときは、ラベルを印の場所と間隔の半分だけ左に寄せ、
        // 印が出るときに FastSpatial のバネで戻す (`SegmentedButtonContentMeasurePolicy`)
        let shift = (icon_size + dp(supplement::segmented_button::ICON_SPACING)) / 2.0;
        let label = div().relative().child(self.label).with_spring(
            "label-offset",
            SpringAnimation::new(spatial).to(active),
            move |this, phase| {
                if has_icon {
                    this
                } else {
                    this.left(-shift * (1.0 - phase.0))
                }
            },
        );

        let mut item = item
            .group(GROUP)
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .flex_basis(Pixels::ZERO)
            .flex_grow(1.0)
            .min_w(row.width)
            // `ButtonDefaults.MinHeight`
            .min_h(dp(button_small::CONTAINER_HEIGHT))
            .px(dp(supplement::segmented_button::HORIZONTAL_PADDING))
            .py(dp(supplement::segmented_button::VERTICAL_PADDING))
            .text_color(content)
            .whitespace_nowrap();
        if !first {
            // 隣の項目と枠線を重ねる (`Arrangement.spacedBy(-space)`)
            item = item.ml(-dp(tokens::OUTLINE_WIDTH));
        }
        item.text_style()
            .refine(&theme.text_style(tokens::LABEL_TEXT_FONT));
        let item = item
            .child(probe)
            .child(Shaped::new(state_layer, theme.clone(), tokens::SHAPE).only_corners(corners))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(dp(supplement::segmented_button::ICON_SPACING))
                    .child(icon_slot)
                    .child(label),
            );
        let layer = SegmentLayer {
            bounds,
            container,
            outline,
            corners,
            active,
            focus_visible,
        };
        (item, layer)
    }
}

/// 1 つだけを選ぶ、MD3 の Segmented button
///
/// 選択肢のラベルを、同じ幅の項目として横に並べ、選ばれている項目を容器の色と印で示す。
/// 状態 (選ばれている値) は呼び出し側が持ち、[`SingleChoiceSegmentedButtonRow::new`] に毎回渡す。並び自身は状態を変えない。
/// 項目は [`SingleChoiceSegmentedButtonRow::segment`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初の有効な項目の順に選ぶ。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。有効な項目が無ければ Tab で入らない
/// - 上下左右の矢印キーで前後の有効な項目へフォーカスを移し、移った先の項目を選ぶよう求める。
///   端では反対の端に戻る
/// - フォーカス中の Space とマウスのクリックで、項目を選ぶよう求める。Enter では求めない
/// - 操作で選択を解除する手段は持たない
///
/// 支援技術には role `radiogroup` として公開され、[`SingleChoiceSegmentedButtonRow::aria_label`] がアクセシブルな名前になる。
/// 項目は role `radio` と選ばれているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルは項目を並べる箱に付く。幅を広げると、項目は同じ幅のまま広がる。
/// 狭めても、項目は最も広いラベルに合わせた幅より縮まず、箱からはみ出す。ラベルは 1 行で描く
#[derive(IntoElement)]
pub struct SingleChoiceSegmentedButtonRow<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    on_change: Option<ChangeHandler<T>>,
    aria_label: Option<SharedString>,
    segments: Vec<SegmentedButton<T>>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> SingleChoiceSegmentedButtonRow<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示する並びを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            on_change: None,
            aria_label: None,
            segments: Vec::new(),
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
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`SingleChoiceSegmentedButtonRow::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn segment(mut self, segment: SegmentedButton<T>) -> Self {
        self.segments.push(segment);
        self
    }

    /// 末尾に `segments` の項目を順に加える
    pub fn segments(mut self, segments: impl IntoIterator<Item = SegmentedButton<T>>) -> Self {
        self.segments.extend(segments);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for SingleChoiceSegmentedButtonRow<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let row = RowLook::new(
            theme,
            self.segments.iter().map(|segment| &segment.segment.label),
            window,
        );
        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let (items, layers): (Vec<_>, Vec<_>) = self
            .segments
            .into_iter()
            .enumerate()
            .map(|(index, SegmentedButton { segment, value })| {
                let active = selected.take_if(|selected| **selected == value).is_some();
                let focus_handle = segment.focus_handle(&self.id, window, cx);
                // リングは項目の外側に描くため、項目自身のスタイルでは表せない
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let radio = HeadlessRadio::new(segment.id.clone(), value)
                    .disabled(segment.disabled)
                    .track_focus(&focus_handle)
                    .aria_label(segment.label.clone());
                segment.decorate(radio, active, index, focus_visible, &row)
            })
            .unzip();

        let (below, above) = row.layers(layers);
        let mut group = HeadlessRadioGroup::new(self.id, self.selected)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .flex()
            .items_center()
            .min_h(dp(tokens::CONTAINER_HEIGHT));
        group.style().refine(&self.style);
        group.child(below).items(items).child(above)
    }
}

impl<T: PartialEq + 'static> Styled for SingleChoiceSegmentedButtonRow<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`SingleChoiceSegmentedButtonRow`] の項目
///
/// 項目が表す値が、並びの選ばれている値と等しいときに選ばれた状態になる
pub struct SegmentedButton<T> {
    segment: Segment,
    value: T,
}

impl<T> SegmentedButton<T> {
    /// `value` を表し、`label` を描く項目を作る
    ///
    /// `label` はアクセシブルな名前にもなる。`id` と `value` は並びの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`SegmentedButton::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            segment: Segment::new(id.into(), label.into()),
            value,
        }
    }

    /// 選ばれていないときに、ラベルの前に `icon` を描く。選ばれると選択の印に入れ替わる
    ///
    /// 指定しない場合、選ばれていないときはラベルだけを描く。アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.segment.icon = Some(icon.into());
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は無効の色で描かれ、押されても選ぶよう求めず、Tab や矢印キーでも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.segment.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// Tab で到達できるかは並びが決める。指定しない場合は項目が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.segment.focus_handle = Some(focus_handle.clone());
        self
    }
}

/// 複数を選べる、MD3 の Segmented button
///
/// 選択肢のラベルを、同じ幅の項目として横に並べ、選ばれている項目を容器の色と印で示す。
/// 項目ごとの状態 (選ばれているか) は呼び出し側が持ち、[`SegmentedToggleButton::new`] に毎回渡す。
/// 項目は [`MultiChoiceSegmentedButtonRow::segment`] で加え、加えた順に左から並ぶ
///
/// - Tab で到達できる項目は 1 つだけで、最初の有効な項目にする。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。有効な項目が無ければ Tab で入らない
/// - 左右の矢印キーで前後の有効な項目へフォーカスを移す。選ばれているかは変えない。端では反対の端に戻る
/// - フォーカス中の Enter と Space とマウスのクリックで、項目の状態を反転するよう求める
///
/// 支援技術には role `toolbar` として公開され、[`MultiChoiceSegmentedButtonRow::aria_label`] がアクセシブルな名前になる。
/// 項目は role `button` と押されているかどうかの状態として公開され、ラベルが名前になる
///
/// [`Styled`] で指定したスタイルは項目を並べる箱に付く。幅を広げると、項目は同じ幅のまま広がる。
/// 狭めても、項目は最も広いラベルに合わせた幅より縮まず、箱からはみ出す。ラベルは 1 行で描く
#[derive(IntoElement)]
pub struct MultiChoiceSegmentedButtonRow {
    id: ElementId,
    aria_label: Option<SharedString>,
    segments: Vec<SegmentedToggleButton>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl MultiChoiceSegmentedButtonRow {
    /// 項目を持たない並びを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            aria_label: None,
            segments: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 末尾に項目を加える
    pub fn segment(mut self, segment: SegmentedToggleButton) -> Self {
        self.segments.push(segment);
        self
    }

    /// 末尾に `segments` の項目を順に加える
    pub fn segments(mut self, segments: impl IntoIterator<Item = SegmentedToggleButton>) -> Self {
        self.segments.extend(segments);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for MultiChoiceSegmentedButtonRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let row = RowLook::new(
            theme,
            self.segments.iter().map(|segment| &segment.segment.label),
            window,
        );
        let (items, layers): (Vec<_>, Vec<_>) = self
            .segments
            .into_iter()
            .enumerate()
            .map(
                |(
                    index,
                    SegmentedToggleButton {
                        segment,
                        checked,
                        on_change,
                    },
                )| {
                    let focus_handle = segment.focus_handle(&self.id, window, cx);
                    // リングは項目の外側に描くため、項目自身のスタイルでは表せない
                    let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                    let button = HeadlessToggleButton::new(segment.id.clone(), checked)
                        .disabled(segment.disabled)
                        .track_focus(&focus_handle)
                        .aria_label(segment.label.clone())
                        .when_some(on_change, |this, on_change| this.on_change(on_change));
                    segment.decorate(button, checked, index, focus_visible, &row)
                },
            )
            .unzip();

        let (below, above) = row.layers(layers);
        let mut group = ToggleGroup::new(self.id)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .flex()
            .items_center()
            .min_h(dp(tokens::CONTAINER_HEIGHT));
        group.style().refine(&self.style);
        group.child(below).items(items).child(above)
    }
}

impl Styled for MultiChoiceSegmentedButtonRow {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// [`MultiChoiceSegmentedButtonRow`] の項目
pub struct SegmentedToggleButton {
    segment: Segment,
    checked: bool,
    on_change: Option<ChangeHandler<bool>>,
}

impl SegmentedToggleButton {
    /// `label` を描き、`checked` の状態を表示する項目を作る
    ///
    /// `label` はアクセシブルな名前にもなる。`id` は並びの中で一意にし、描画のたびに同じ値を渡すこと。
    /// [`SegmentedToggleButton::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, checked: bool, label: impl Into<SharedString>) -> Self {
        Self {
            segment: Segment::new(id.into(), label.into()),
            checked,
            on_change: None,
        }
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には操作後に求められる状態として、現在の状態を反転した値を渡す。無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`SegmentedToggleButton::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばず、支援技術にクリック操作を公開しない
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 選ばれていないときに、ラベルの前に `icon` を描く。選ばれると選択の印に入れ替わる
    ///
    /// 指定しない場合、選ばれていないときはラベルだけを描く。アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.segment.icon = Some(icon.into());
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は無効の色で描かれ、押されてもハンドラを呼ばず、Tab や矢印キーでも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.segment.disabled = disabled;
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// Tab で到達できるかは並びが決める。指定しない場合は項目が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.segment.focus_handle = Some(focus_handle.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{AnyElement, Context, Modifiers, Render, TestAppContext, VisualTestContext, point};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    const LABELS: [&str; 3] = ["Day", "Week", "Month"];

    /// 1 つを選ぶ並びと、比べる用にラベルと同じ書体で描いた最も長いラベル
    struct SingleView {
        selected: Option<usize>,
        changes: Rc<RefCell<Vec<usize>>>,
        disabled: Option<usize>,
    }

    impl Render for SingleView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut probe = div().flex_none().bg(gpui::black()).child("Month");
            probe
                .text_style()
                .refine(&cx.md3_theme().text_style(tokens::LABEL_TEXT_FONT));
            let changes = self.changes.clone();
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(100.))
                .child(
                    SingleChoiceSegmentedButtonRow::new("row", self.selected)
                        .aria_label("Period")
                        .on_change(move |value, _, _| changes.borrow_mut().push(*value))
                        .segments(LABELS.iter().enumerate().map(|(index, label)| {
                            SegmentedButton::new(index, index, *label)
                                .disabled(self.disabled == Some(index))
                        })),
                )
                .child(probe)
        }
    }

    fn add_single(
        cx: &mut TestAppContext,
        selected: Option<usize>,
        disabled: Option<usize>,
    ) -> (Rc<RefCell<Vec<usize>>>, &mut VisualTestContext) {
        cx.update(crate::init);
        let changes = Rc::new(RefCell::new(Vec::new()));
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            SingleView {
                selected,
                changes: changes.clone(),
                disabled,
            }
        });
        // 状態レイヤーを描かないよう、マウスを並びの外に置く
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        (changes, cx)
    }

    /// 項目の容器の x, y, 幅, 高さと、左右の角丸の半径と塗り (論理ピクセル)。左から順に並べる。塗りの無い容器は `None`
    type Segment = ((f32, f32, f32, f32), (f32, f32), Option<gpui::Background>);

    fn segments(cx: &mut VisualTestContext) -> Vec<Segment> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let quads = window.painted_quads();
            let rect = |quad: &gpui::Quad| {
                let b = quad.bounds;
                (
                    b.origin.x.0 / scale,
                    b.origin.y.0 / scale,
                    b.size.width.0 / scale,
                    b.size.height.0 / scale,
                )
            };
            // gpui は塗りの無い枠線を辺ごとに分けて描くため、同じ矩形の枠線は 1 つにまとめる。
            // 塗りは枠線と別の矩形として描く。太さ 1dp の枠線だけを項目の枠線とし、フォーカスリングと分ける
            let mut segments: Vec<Segment> = Vec::new();
            for quad in quads
                .iter()
                .filter(|quad| quad.border_widths.top.0 / scale == 1.0)
            {
                if segments.iter().any(|(r, _, _)| *r == rect(quad)) {
                    continue;
                }
                let fill = quads
                    .iter()
                    .find(|fill| fill.border_widths.top.0 == 0.0 && rect(fill) == rect(quad))
                    .map(|fill| fill.background);
                segments.push((
                    rect(quad),
                    (
                        quad.corner_radii.top_left.0 / scale,
                        quad.corner_radii.top_right.0 / scale,
                    ),
                    fill,
                ));
            }
            segments.sort_by(|a, b| a.0.0.total_cmp(&b.0.0));
            segments
        })
    }

    fn probe_width(cx: &mut VisualTestContext) -> f32 {
        let probe = gpui::Background::from(gpui::black());
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == probe)
                .map(|quad| quad.bounds.size.width.0 / scale)
                .expect("比べる用のラベルを描くこと")
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> gpui::Background {
        cx.update(|_, cx| cx.md3_theme().color(role).into())
    }

    /// 期待値は Compose の項目の幅 (`SegmentedButtonDefaults.ContentPadding` の左右の余白 12dp と、
    /// `SegmentedButtonContentMeasurePolicy` が確保するアイコン 18dp・間隔 8dp・ラベルの幅の和) と、
    /// 高さ (OutlinedSegmentedButtonTokens.ContainerHeight の 40dp)、`spacedBy(-1dp)` の重なり
    #[gpui::test]
    fn 項目は最も広いラベルに合わせた同じ幅で枠線を重ねて並ぶ(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_single(cx, None, None);
        let width = 12.0 + 18.0 + 8.0 + probe_width(cx) + 12.0;
        let rects: Vec<_> = segments(cx).into_iter().map(|(rect, _, _)| rect).collect();
        assert_eq!(
            rects,
            vec![
                (0.0, 0.0, width, 40.0),
                (width - 1.0, 0.0, width, 40.0),
                (2.0 * (width - 1.0), 0.0, width, 40.0),
            ]
        );
    }

    /// 期待値は Compose の `SegmentedButtonDefaults.itemShape`。CornerFull の 40dp の高さの半分を、
    /// 最初の項目は左、最後の項目は右の角だけに付ける
    #[gpui::test]
    fn 両端の項目だけが外側の角を丸める(cx: &mut TestAppContext) {
        let (_, cx) = add_single(cx, None, None);
        let radii: Vec<_> = segments(cx)
            .into_iter()
            .map(|(_, radii, _)| radii)
            .collect();
        assert_eq!(radii, vec![(20.0, 0.0), (0.0, 0.0), (0.0, 20.0)]);
    }

    /// 期待値は Compose の `defaultSegmentedButtonColors`。選ばれた項目は SecondaryContainer で塗り、
    /// 無効でも塗りを保つ。選ばれていない項目は塗らない
    #[gpui::test]
    fn 選ばれた項目だけを塗り無効でも塗りを保つ(cx: &mut TestAppContext) {
        for disabled in [None, Some(1)] {
            let (_, cx) = add_single(cx, Some(1), disabled);
            let selected = color(cx, ColorRole::SecondaryContainer);
            let backgrounds: Vec<_> = segments(cx).into_iter().map(|(_, _, bg)| bg).collect();
            assert_eq!(
                backgrounds,
                vec![None, Some(selected), None],
                "disabled: {disabled:?}"
            );
        }
    }

    #[gpui::test]
    fn 押した項目の値を渡し無効な項目と選ばれている項目では渡さない(
        cx: &mut TestAppContext,
    ) {
        let (changes, cx) = add_single(cx, Some(0), Some(2));
        let centers: Vec<_> = segments(cx)
            .into_iter()
            .map(|((x, y, w, h), _, _)| point(px(x + w / 2.0), px(y + h / 2.0)))
            .collect();
        for center in &centers {
            cx.simulate_click(*center, Modifiers::none());
        }
        assert_eq!(*changes.borrow(), vec![1]);
    }

    #[gpui::test]
    fn tab_のフォーカスを項目の外側のリングで示す(cx: &mut TestAppContext) {
        let (_, cx) = add_single(cx, Some(1), None);
        let (x, y, w, h) = segments(cx)[1].0;
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        let scale = cx.update(|window, _| window.scale_factor());
        // リングは選ばれている項目の外側 5px に描く。中の項目の直角の角を外側へ 5px 広げるため、角の半径は 5px になる
        let ring = cx.update(|window, _| {
            window.painted_quads().into_iter().any(|quad| {
                let b = quad.bounds;
                (
                    b.origin.x.0 / scale,
                    b.origin.y.0 / scale,
                    b.size.width.0 / scale,
                    b.size.height.0 / scale,
                    quad.corner_radii.top_left.0 / scale,
                ) == (x - 5.0, y - 5.0, w + 10.0, h + 10.0, 5.0)
            })
        });
        assert!(ring, "選ばれている項目にリングを描くこと");
    }

    /// 複数を選べる並び。押されると状態を反転して描き直す
    struct MultiView {
        checked: [bool; 3],
    }

    impl Render for MultiView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().flex().items_start().child(
                MultiChoiceSegmentedButtonRow::new("row")
                    .aria_label("Days")
                    .segments(LABELS.iter().enumerate().map(|(index, label)| {
                        SegmentedToggleButton::new(index, self.checked[index], *label).on_change(
                            cx.listener(move |this, checked: &bool, _, cx| {
                                this.checked[index] = *checked;
                                cx.notify();
                            }),
                        )
                    })),
            )
        }
    }

    #[gpui::test]
    fn 複数を選べる並びは押した項目の状態を反転する(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            MultiView {
                checked: [true, false, false],
            }
        });
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        let centers: Vec<_> = segments(cx)
            .into_iter()
            .map(|((x, y, w, h), _, _)| point(px(x + w / 2.0), px(y + h / 2.0)))
            .collect();
        cx.simulate_click(centers[0], Modifiers::none());
        cx.simulate_click(centers[2], Modifiers::none());
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| view.checked),
            [false, false, true]
        );
        let selected = color(cx, ColorRole::SecondaryContainer);
        let backgrounds: Vec<_> = segments(cx).into_iter().map(|(_, _, bg)| bg).collect();
        assert_eq!(backgrounds, vec![None, None, Some(selected)]);
    }

    fn hsla(cx: &mut VisualTestContext, role: ColorRole) -> gpui::Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 項目の中心の位置。左から順に並べる
    fn centers(cx: &mut VisualTestContext) -> Vec<gpui::Point<gpui::Pixels>> {
        segments(cx)
            .into_iter()
            .map(|((x, y, w, h), _, _)| point(px(x + w / 2.0), px(y + h / 2.0)))
            .collect()
    }

    /// `build` で作った要素を、伸ばさない箱に入れて描くビュー
    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().flex().flex_col().items_start().child((self.0)())
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        build: impl Fn() -> AnyElement + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(build))
        });
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        cx
    }

    /// 期待値は Compose の `defaultSegmentedButtonColors` の枠線の色。有効なら Outline、
    /// 無効なら Outline に OutlinedSegmentedButtonTokens.DisabledOutlineOpacity (0.12) を付ける
    #[gpui::test]
    fn 枠線は_outline_で無効な項目では不透明度を下げる(cx: &mut TestAppContext) {
        let (_, cx) = add_single(cx, Some(1), Some(2));
        let outline = hsla(cx, ColorRole::Outline);
        let rects: Vec<_> = segments(cx).into_iter().map(|(rect, _, _)| rect).collect();
        let scale = cx.update(|window, _| window.scale_factor());
        let borders: Vec<_> = cx.update(|window, _| {
            rects
                .iter()
                .map(|&(x, ..)| {
                    window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| {
                            quad.border_widths.top.0 > 0.0 && quad.bounds.origin.x.0 / scale == x
                        })
                        .map(|quad| quad.border_color)
                        .expect("枠線を描くこと")
                })
                .collect()
        });
        assert_eq!(borders, vec![outline, outline, outline.opacity(0.12)]);
    }

    /// `index` 番目の項目の枠線の内側に重ねた塗り
    fn state_layer(cx: &mut VisualTestContext, index: usize) -> Option<gpui::Background> {
        let (x, y, w, h) = segments(cx)[index].0;
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .find(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    ) == (x + 1.0, y + 1.0, w - 2.0, h - 2.0)
                })
                .map(|quad| quad.background)
        })
    }

    /// 状態レイヤーは内容の色を枠線の内側に重ねる。選ばれていない項目は OnSurface、選ばれた項目は
    /// OnSecondaryContainer を、hover では 8%、フォーカスでは 10% 重ねる。無効な項目には重ねない
    #[gpui::test]
    fn hover_とフォーカスで状態レイヤーを重ね無効な項目には重ねない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_single(cx, Some(1), Some(2));
        let on_surface = hsla(cx, ColorRole::OnSurface);
        let on_container = hsla(cx, ColorRole::OnSecondaryContainer);
        let centers = centers(cx);
        let expected = [
            Some(on_surface.opacity(0.08).into()),
            Some(on_container.opacity(0.08).into()),
            None,
        ];
        for (index, expected) in expected.into_iter().enumerate() {
            cx.simulate_mouse_move(centers[index], None, Modifiers::none());
            assert_eq!(state_layer(cx, index), expected, "hover: {index}");
        }

        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        assert_eq!(state_layer(cx, 1), Some(on_container.opacity(0.1).into()));
    }

    /// 期待値は Compose の `interactionZIndex`。選ばれた項目は前面に置くため、無効で枠線の薄い選ばれた項目でも、
    /// 隣の項目と重なる枠線は選ばれた項目の容器と枠線で描く
    #[gpui::test]
    fn 選ばれた項目の容器と枠線を隣の項目より前面に描く(
        cx: &mut TestAppContext,
    ) {
        for selected in 0..3 {
            let (_, cx) = add_single(cx, Some(selected), Some(selected));
            let rects: Vec<_> = segments(cx).into_iter().map(|(rect, _, _)| rect).collect();
            let orders: Vec<_> = cx.update(|window, _| {
                let scale = window.scale_factor();
                let quads = window.painted_quads();
                rects
                    .iter()
                    .map(|&(x, ..)| {
                        quads
                            .iter()
                            .filter(|quad| {
                                quad.border_widths.top.0 / scale == 1.0
                                    && quad.bounds.origin.x.0 / scale == x
                            })
                            .map(|quad| quad.order)
                            .max()
                            .expect("枠線を描くこと")
                    })
                    .collect()
            });
            for (index, order) in orders.iter().enumerate() {
                if index.abs_diff(selected) == 1 {
                    assert!(
                        orders[selected] > *order,
                        "selected: {selected}, neighbor: {index}"
                    );
                }
            }
        }
    }

    /// 期待値は Compose の項目の `defaultMinSize` の幅 (`ButtonDefaults.MinWidth` の 58dp)
    #[gpui::test]
    fn ラベルが短くても項目は_58dp_より狭くしない(cx: &mut TestAppContext) {
        let cx = add_view(cx, || {
            SingleChoiceSegmentedButtonRow::new("row", None)
                .segments((0..2).map(|index| SegmentedButton::new(index, index, "")))
                .into_any_element()
        });
        let widths: Vec<_> = segments(cx)
            .into_iter()
            .map(|((_, _, width, _), _, _)| width)
            .collect();
        assert_eq!(widths, vec![58.0, 58.0]);
    }

    // gpui の描画順 (`Quad::order`) は、先に描いた重なる矩形より大きくなる
    #[gpui::test]
    fn フォーカスリングは重なる項目より前面に描く(cx: &mut TestAppContext) {
        let (_, cx) = add_single(cx, Some(1), None);
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        let ring = hsla(cx, ColorRole::Secondary);
        let quads = cx.update(|window, _| window.painted_quads());
        let (rings, others): (Vec<_>, Vec<_>) = quads
            .iter()
            .partition(|quad| quad.border_widths.top.0 > 0.0 && quad.border_color == ring);
        assert!(!rings.is_empty(), "リングを描くこと");
        // gpui は塗りの無い枠線を辺ごとに、同じ矩形を切り取りの範囲を変えて描くため、切り取った範囲で重なりを見る
        let overlaps = |a: &gpui::Quad, b: &gpui::Quad| {
            let a = a.bounds.intersect(&a.content_mask.bounds);
            let b = b.bounds.intersect(&b.content_mask.bounds);
            !a.intersect(&b).is_empty()
        };
        // 両隣の項目の枠線もリングと重なる
        let mut overlapping = 0;
        for ring in &rings {
            for other in others.iter().filter(|other| overlaps(ring, other)) {
                overlapping += 1;
                assert!(
                    ring.order > other.order,
                    "リングを重なる矩形より前面に描くこと: {:?}",
                    other.bounds
                );
            }
        }
        assert!(overlapping > 0);
    }

    #[gpui::test]
    fn フォーカスリングは祖先の切り取りの範囲の中に描く(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, || {
            div()
                .overflow_hidden()
                .child(
                    SingleChoiceSegmentedButtonRow::new("row", Some(0)).segments(
                        LABELS
                            .iter()
                            .enumerate()
                            .map(|(index, label)| SegmentedButton::new(index, index, *label)),
                    ),
                )
                .into_any_element()
        });
        let (x, _, w, h) = *segments(cx).last().map(|(rect, _, _)| rect).unwrap();
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        let ring = hsla(cx, ColorRole::Secondary);
        let scale = cx.update(|window, _| window.scale_factor());
        let masks: Vec<_> = cx.update(|window, _| {
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_widths.top.0 > 0.0 && quad.border_color == ring)
                .map(|quad| {
                    let b = quad.content_mask.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        (b.origin.x.0 + b.size.width.0) / scale,
                        (b.origin.y.0 + b.size.height.0) / scale,
                    )
                })
                .collect()
        });
        assert!(!masks.is_empty(), "リングを描くこと");
        // 切り取る箱は並びと同じ大きさ (0, 0) - (x + w, h)
        for (left, top, right, bottom) in masks {
            assert!(
                left >= 0.0 && top >= 0.0 && right <= x + w && bottom <= h,
                "リングを並びの外に描かないこと: {:?}",
                (left, top, right, bottom)
            );
        }
    }

    #[gpui::test]
    fn フォーカスリングにも祖先の不透明度を付ける(cx: &mut TestAppContext) {
        let cx = add_view(cx, || {
            div()
                .opacity(0.5)
                .child(
                    SingleChoiceSegmentedButtonRow::new("row", Some(0)).segments(
                        LABELS
                            .iter()
                            .enumerate()
                            .map(|(index, label)| SegmentedButton::new(index, index, *label)),
                    ),
                )
                .into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        let ring = hsla(cx, ColorRole::Secondary);
        let scale = cx.update(|window, _| window.scale_factor());
        let colors: Vec<_> = cx.update(|window, _| {
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.border_widths.top.0 / scale == 3.0)
                .map(|quad| quad.border_color)
                .collect()
        });
        assert!(!colors.is_empty(), "リングを描くこと");
        // gpui は塗りの無い枠線を辺ごとに分けて描く
        for color in colors {
            assert_eq!(color, ring.opacity(0.5));
        }
    }
}
