// 寸法・色の選択・状態ごとの elevation・選択の印の出し入れは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Chip.kt
//         (`AssistChip`、`FilterChip`、`SuggestionChip`、`SelectableChip`、`ChipContent`、`AnimatingChipContent`、
//         `AssistChipDefaults`、`FilterChipDefaults`、`SuggestionChipDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// フォーカス中の枠線の色と、Filter の選択の印を選ばれていないときのアイコンと置き換える振る舞いは、
// Compose にないため Material Web から移植している (https://github.com/material-components/material-web)
// Source: chips/internal/_shared.scss (`:where(:focus) .outline`)、chips/internal/filter-chip.ts (`renderLeadingIcon`)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のチップ

use gpui::{
    AnimationExt, AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, Hsla,
    InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce, SharedString,
    SpringAnimation, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::toggle::ToggleButton as HeadlessToggleButton;

use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{assist_chip, filter_chip, suggestion_chip};
use md3_theme::md3_tokens::elevation::Elevation;
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 容器の層がチップの hover を知るための group 名
const GROUP: &str = "md3-chip";

/// 押すと操作を実行するチップの用途。用途によってラベルの色が決まる
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// 状況に合わせて提示する、アプリの操作を手助けするチップ (Assist chip)。予定の追加や経路の表示などに使う
    #[default]
    Assist,
    /// 入力や返信の候補を示すチップ (Suggestion chip)。押すと候補をそのまま入力する場合などに使う
    Suggestion,
}

/// チップの容器の見た目
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChipVariant {
    /// 枠線で囲んだ平らな容器
    #[default]
    Flat,
    /// 影で浮かせた容器。画像の上など、背景と区別する必要がある場合に使う
    Elevated,
}

/// 枠線の太さと、通常とフォーカスを示すときの色
struct Outline {
    width: Dp,
    color: Hsla,
    focused: Hsla,
}

/// 状態に合わせて解決したチップの色と影
struct Look {
    /// `None` は透明
    container: Option<Hsla>,
    outline: Option<Outline>,
    label: Hsla,
    icon: Hsla,
    /// 通常・hover・フォーカスを示すときの影の高さ。無効なときは使わない
    elevations: [Elevation; 3],
}

/// `chip` にチップの容器・先頭の要素・ラベル・フォーカスリングを加え、配置と書体を付ける
fn decorate<E>(
    chip: E,
    look: Look,
    leading: Option<AnyElement>,
    label: SharedString,
    disabled: bool,
    focus_visible: bool,
    theme: &SharedTheme,
) -> E
where
    E: Styled + ParentElement + InteractiveElement + FluentBuilder,
{
    // 状態レイヤーはラベルの色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
    let state_layer = |opacity: f32| -> Hsla {
        let layer = look.label.opacity(opacity);
        match look.container {
            Some(container) => container.blend(layer),
            None => layer,
        }
    };
    let [rest, hovered, focused] = look.elevations.map(|elevation| theme.shadow(elevation));
    // 角丸は描画時の大きさで決まるため、背景・枠線・影は Shaped で包んだ層に描き、チップ自身は透明な箱にする
    let container = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .when_some(look.container, |this, container| this.bg(container))
        .when_some(look.outline, |this, outline| {
            this.border(dp(outline.width))
                .border_color(if focus_visible && !disabled {
                    outline.focused
                } else {
                    outline.color
                })
        })
        .when(!disabled, |this| {
            let this = if focus_visible {
                this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                    .shadow(focused)
            } else {
                this.shadow(rest)
            };
            // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
            this.group_hover(GROUP, |style| {
                style
                    .bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                    .shadow(hovered)
            })
        });

    // 高さ・形・書体・アイコンの大きさは Assist・Suggestion・Filter のトークンで同じ値のため、AssistChipTokens の値を使う
    let mut chip = chip
        .group(GROUP)
        .relative()
        .flex_grow_1()
        // 外側の箱を狭めたときに、ラベルを省略して収める
        .min_w_0()
        .flex()
        .items_center()
        .min_h(dp(assist_chip::CONTAINER_HEIGHT))
        .px(dp(supplement::chip::CONTENT_PADDING))
        .text_color(look.label)
        .child(Shaped::new(
            container,
            theme.clone(),
            assist_chip::CONTAINER_SHAPE,
        ))
        .children(leading)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .mx(dp(supplement::chip::HORIZONTAL_SPACING))
                .truncate()
                .child(label),
        )
        .when(focus_visible, |this| {
            let outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);
            let ring = div()
                .absolute()
                .top(-outset)
                .left(-outset)
                .right(-outset)
                .bottom(-outset)
                .border(dp(focus_ring::WIDTH))
                .border_color(theme.color(focus_ring::COLOR));
            this.child(
                Shaped::new(ring, theme.clone(), assist_chip::CONTAINER_SHAPE).outset(outset),
            )
        });
    chip.text_style()
        .refine(&theme.text_style(assist_chip::LABEL_TEXT_FONT));
    chip
}

/// 先頭のアイコン
fn icon_svg(icon: StyledIcon, color: Hsla) -> gpui::Svg {
    svg()
        .data(icon.svg().as_bytes())
        .flex_none()
        .size(dp(assist_chip::ICON_SIZE))
        .text_color(color)
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// 押すと操作を実行する MD3 のチップ (Assist chip と Suggestion chip)
///
/// `label` を 1 行で描き、収まらない部分は省略記号にする。
/// 支援技術には role `button` として公開され、`label` がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で [`Chip::on_click`] のハンドラを呼ぶ。
///
/// [`Styled`] で指定したスタイルはチップの外側の箱に付く。大きさや外側の余白、flex の指定に使い、
/// 箱を広げるとチップも箱いっぱいに広がる。
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    outer: Div,
    label: SharedString,
    kind: ChipKind,
    variant: ChipVariant,
    icon: Option<StyledIcon>,
    on_click: Option<ClickHandler>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Chip {
    /// `label` を描く、[`ChipKind::Assist`] で [`ChipVariant::Flat`] のチップを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Chip::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            label: label.into(),
            kind: ChipKind::default(),
            variant: ChipVariant::default(),
            icon: None,
            on_click: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// チップの用途を設定する
    pub fn kind(mut self, kind: ChipKind) -> Self {
        self.kind = kind;
        self
    }

    /// 容器の見た目を設定する
    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// ラベルの前にアイコンを置く
    ///
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。スタイルや塗りは [`StyledIcon`] で指定する。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// チップを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なチップは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのチップのフォーカスとして使う
    ///
    /// 指定しない場合はチップが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let disabled = self.disabled;
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはチップの外側に描くため、チップ自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        // Assist と Suggestion のトークンはラベルの色だけが異なる
        let label = match self.kind {
            ChipKind::Assist => assist_chip::LABEL_TEXT_COLOR,
            ChipKind::Suggestion => suggestion_chip::LABEL_TEXT_COLOR,
        };
        let disabled_content = |role| {
            theme
                .color(role)
                .opacity(assist_chip::DISABLED_LABEL_TEXT_OPACITY)
        };
        // `defaultAssistChipColors`、`defaultElevatedAssistChipColors`、`assistChipBorder`、
        // `assistChipElevation`、`elevatedAssistChipElevation` (Suggestion も同じ値)
        let look = Look {
            container: match (self.variant, disabled) {
                (ChipVariant::Flat, _) => None,
                (ChipVariant::Elevated, false) => {
                    Some(theme.color(assist_chip::ELEVATED_CONTAINER_COLOR))
                }
                (ChipVariant::Elevated, true) => Some(
                    theme
                        .color(assist_chip::ELEVATED_DISABLED_CONTAINER_COLOR)
                        .opacity(assist_chip::ELEVATED_DISABLED_CONTAINER_OPACITY),
                ),
            },
            outline: (self.variant == ChipVariant::Flat).then(|| Outline {
                width: assist_chip::FLAT_OUTLINE_WIDTH,
                color: if disabled {
                    theme
                        .color(assist_chip::FLAT_DISABLED_OUTLINE_COLOR)
                        .opacity(assist_chip::FLAT_DISABLED_OUTLINE_OPACITY)
                } else {
                    theme.color(assist_chip::FLAT_OUTLINE_COLOR)
                },
                focused: theme.color(match self.kind {
                    ChipKind::Assist => assist_chip::FLAT_FOCUS_OUTLINE_COLOR,
                    ChipKind::Suggestion => suggestion_chip::FLAT_FOCUS_OUTLINE_COLOR,
                }),
            }),
            label: if disabled {
                disabled_content(assist_chip::DISABLED_LABEL_TEXT_COLOR)
            } else {
                theme.color(label)
            },
            icon: if disabled {
                disabled_content(assist_chip::DISABLED_ICON_COLOR)
            } else {
                theme.color(assist_chip::ICON_COLOR)
            },
            elevations: match self.variant {
                ChipVariant::Flat => [assist_chip::FLAT_CONTAINER_ELEVATION; 3],
                ChipVariant::Elevated => [
                    assist_chip::ELEVATED_CONTAINER_ELEVATION,
                    assist_chip::ELEVATED_HOVER_CONTAINER_ELEVATION,
                    assist_chip::ELEVATED_FOCUS_CONTAINER_ELEVATION,
                ],
            },
        };

        let leading = self
            .icon
            .map(|icon| icon_svg(icon, look.icon).into_any_element());
        let chip = HeadlessButton::new(self.id)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .aria_label(self.label.clone())
            .when_some(self.on_click, |this, on_click| this.on_click(on_click));
        self.outer.child(decorate(
            chip,
            look,
            leading,
            self.label,
            disabled,
            focus_visible,
            &theme,
        ))
    }
}

impl Styled for Chip {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

/// 選ぶと内容を絞り込む MD3 のチップ (Filter chip)
///
/// 選ばれているかどうかは呼び出し側が持ち、[`FilterChip::new`] に毎回渡す。チップ自身は状態を変えない。
/// 選ばれていると容器を塗り、ラベルの前に選択の印を出す。
/// `label` を 1 行で描き、収まらない部分は省略記号にする。
///
/// 支援技術には role `button` と押されているかどうかの状態 (トグルボタン) として公開され、`label` がアクセシブルな名前になる。
/// マウスのクリック・フォーカス中の Enter と Space で、反転した値を [`FilterChip::on_change`] に渡す。
///
/// [`Styled`] で指定したスタイルはチップの外側の箱に付く。大きさや外側の余白、flex の指定に使い、
/// 箱を広げるとチップも箱いっぱいに広がる。
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない
#[derive(IntoElement)]
pub struct FilterChip {
    id: ElementId,
    outer: Div,
    label: SharedString,
    selected: bool,
    variant: ChipVariant,
    icon: Option<StyledIcon>,
    on_change: Option<ChangeHandler>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl FilterChip {
    /// `label` を描き、`selected` の状態を表示する [`ChipVariant::Flat`] のチップを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`FilterChip::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            label: label.into(),
            selected,
            variant: ChipVariant::default(),
            icon: None,
            on_change: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// 容器の見た目を設定する
    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 選ばれていないときに、ラベルの前にアイコンを置く。選ばれているときはアイコンの代わりに選択の印を出す
    ///
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。スタイルや塗りは [`StyledIcon`] で指定する。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 押されたときに、反転した値で `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`FilterChip::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// チップを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なチップは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `focus_handle` をこのチップのフォーカスとして使う
    ///
    /// 指定しない場合はチップが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for FilterChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let disabled = self.disabled;
        let selected = self.selected;
        let focus_handle = self
            .focus_handle
            .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
        // リングはチップの外側に描くため、チップ自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

        let disabled_color = |role, opacity| theme.color(role).opacity(opacity);
        let elevated = self.variant == ChipVariant::Elevated;
        // `defaultFilterChipColors`、`defaultElevatedFilterChipColors`、`filterChipBorder`、
        // `filterChipElevation`、`elevatedFilterChipElevation`
        let look = Look {
            container: match (elevated, selected, disabled) {
                (false, false, _) => None,
                (false, true, false) => {
                    Some(theme.color(filter_chip::FLAT_SELECTED_CONTAINER_COLOR))
                }
                (false, true, true) => Some(disabled_color(
                    filter_chip::FLAT_DISABLED_SELECTED_CONTAINER_COLOR,
                    filter_chip::FLAT_DISABLED_SELECTED_CONTAINER_OPACITY,
                )),
                (true, false, false) => {
                    Some(theme.color(filter_chip::ELEVATED_UNSELECTED_CONTAINER_COLOR))
                }
                (true, true, false) => {
                    Some(theme.color(filter_chip::ELEVATED_SELECTED_CONTAINER_COLOR))
                }
                (true, _, true) => Some(disabled_color(
                    filter_chip::ELEVATED_DISABLED_CONTAINER_COLOR,
                    filter_chip::ELEVATED_DISABLED_CONTAINER_OPACITY,
                )),
            },
            // 選ばれているときの枠線の太さ (`FlatSelectedOutlineWidth`) は 0 のため、枠線を描かない
            outline: (!elevated && !selected).then(|| Outline {
                width: filter_chip::FLAT_UNSELECTED_OUTLINE_WIDTH,
                color: if disabled {
                    disabled_color(
                        filter_chip::FLAT_DISABLED_UNSELECTED_OUTLINE_COLOR,
                        filter_chip::FLAT_DISABLED_UNSELECTED_OUTLINE_OPACITY,
                    )
                } else {
                    theme.color(filter_chip::FLAT_UNSELECTED_OUTLINE_COLOR)
                },
                focused: theme.color(filter_chip::FLAT_UNSELECTED_FOCUS_OUTLINE_COLOR),
            }),
            label: match (selected, disabled) {
                (_, true) => disabled_color(
                    filter_chip::DISABLED_LABEL_TEXT_COLOR,
                    filter_chip::DISABLED_LABEL_TEXT_OPACITY,
                ),
                (true, false) => theme.color(filter_chip::SELECTED_LABEL_TEXT_COLOR),
                (false, false) => theme.color(filter_chip::UNSELECTED_LABEL_TEXT_COLOR),
            },
            icon: match (selected, disabled) {
                (_, true) => disabled_color(
                    filter_chip::DISABLED_LEADING_ICON_COLOR,
                    filter_chip::DISABLED_LEADING_ICON_OPACITY,
                ),
                (true, false) => theme.color(filter_chip::SELECTED_LEADING_ICON_COLOR),
                (false, false) => theme.color(filter_chip::UNSELECTED_LEADING_ICON_COLOR),
            },
            // Compose の `filterChipElevation` は hover の影 (`FlatSelectedHoverContainerElevation`) を
            // 選ばれていないときにも使うが、トークンの名前に合わせて選ばれているときだけにする
            elevations: match (elevated, selected) {
                (false, false) => [
                    filter_chip::FLAT_CONTAINER_ELEVATION,
                    filter_chip::FLAT_UNSELECTED_HOVER_CONTAINER_ELEVATION,
                    filter_chip::FLAT_UNSELECTED_FOCUS_CONTAINER_ELEVATION,
                ],
                (false, true) => [
                    filter_chip::FLAT_CONTAINER_ELEVATION,
                    filter_chip::FLAT_SELECTED_HOVER_CONTAINER_ELEVATION,
                    filter_chip::FLAT_SELECTED_FOCUS_CONTAINER_ELEVATION,
                ],
                (true, _) => [
                    filter_chip::ELEVATED_CONTAINER_ELEVATION,
                    filter_chip::ELEVATED_HOVER_CONTAINER_ELEVATION,
                    filter_chip::ELEVATED_FOCUS_CONTAINER_ELEVATION,
                ],
            },
        };

        let check = icon_svg(StyledIcon::new(md3_icons::CHECK), look.icon);
        let leading = match self.icon {
            Some(icon) if !selected => icon_svg(icon, look.icon).into_any_element(),
            Some(_) => check.into_any_element(),
            // アイコンが無ければ、選ばれると印の場所を広げながら印を表示し、選ばれなくなると縮めながら消す
            // (`AnimatingChipContent` の `expandHorizontally`・`fadeIn` と `shrinkHorizontally`・`fadeOut`)
            None => {
                let icon_size = dp(filter_chip::ICON_SIZE);
                let (size_spring, opacity_spring) = if selected {
                    (MotionSchemeKey::FastSpatial, MotionSchemeKey::SlowEffects)
                } else {
                    (
                        MotionSchemeKey::DefaultEffects,
                        MotionSchemeKey::FastEffects,
                    )
                };
                div()
                    .flex_none()
                    .child(
                        div()
                            .h(icon_size)
                            .overflow_hidden()
                            .child(check)
                            .with_spring(
                                "check-width",
                                SpringAnimation::new(theme.spring(size_spring)).to(selected),
                                move |this, phase| this.w(icon_size * phase.0.max(0.0)),
                            ),
                    )
                    .with_spring(
                        "check-opacity",
                        SpringAnimation::new(theme.spring(opacity_spring)).to(selected),
                        |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
                    )
                    .into_any_element()
            }
        };
        let chip = HeadlessToggleButton::new(self.id, selected)
            .track_focus(&focus_handle)
            .disabled(disabled)
            .aria_label(self.label.clone())
            .when_some(self.on_change, |this, on_change| this.on_change(on_change));
        self.outer.child(decorate(
            chip,
            look,
            Some(leading),
            self.label,
            disabled,
            focus_visible,
            &theme,
        ))
    }
}

impl Styled for FilterChip {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};
    use md3_theme::md3_color::ColorRole;

    use super::*;

    /// チップと、比べる用にラベルと同じ書体で描いた `probe`
    struct ChipView {
        chip: Box<dyn Fn() -> AnyElement>,
        probe: &'static str,
    }

    impl Render for ChipView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut probe = div().flex_none().bg(gpui::black()).child(self.probe);
            probe
                .text_style()
                .refine(&cx.md3_theme().text_style(assist_chip::LABEL_TEXT_FONT));
            // ルートの要素はウィンドウいっぱいに広がるため、チップを伸ばさない箱に包む
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(100.))
                .child((self.chip)())
                .child(probe)
        }
    }

    fn add_view<'a, E: IntoElement>(
        cx: &'a mut TestAppContext,
        probe: &'static str,
        chip: impl Fn() -> E + 'static,
    ) -> &'a mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ChipView {
                chip: Box::new(move || chip().into_any_element()),
                probe,
            }
        });
        // 状態レイヤーを描かないよう、マウスをチップの外に置く
        cx.simulate_mouse_move(point(px(900.), px(900.)), None, Modifiers::none());
        cx
    }

    /// 矩形の x, y, 幅, 高さ (論理ピクセル)
    type Rect = (f32, f32, f32, f32);

    fn rect(quad: &gpui::Quad, scale: f32) -> Rect {
        let b = quad.bounds;
        (
            b.origin.x.0 / scale,
            b.origin.y.0 / scale,
            b.size.width.0 / scale,
            b.size.height.0 / scale,
        )
    }

    /// `background` で塗った枠線の無い矩形と、その左上の角丸の半径
    fn filled(cx: &mut VisualTestContext, background: Hsla) -> Vec<(Rect, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| {
                    quad.background == background.into() && quad.border_widths.top.0 == 0.0
                })
                .map(|quad| (rect(quad, scale), quad.corner_radii.top_left.0 / scale))
                .collect()
        })
    }

    /// 枠線を描いた矩形と、その太さ・色・左上の角丸の半径。左から順に並べる。
    /// gpui は塗りの無い枠線を辺ごとに分けて描くため、同じ矩形は 1 つにまとめる
    fn outlines(cx: &mut VisualTestContext) -> Vec<(Rect, f32, Hsla, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut outlines = Vec::new();
            for quad in window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_widths.top.0 > 0.0)
            {
                let outline = (
                    rect(quad, scale),
                    quad.border_widths.top.0 / scale,
                    quad.border_color,
                    quad.corner_radii.top_left.0 / scale,
                );
                if !outlines.contains(&outline) {
                    outlines.push(outline);
                }
            }
            outlines.sort_by(|a, b| a.0.0.total_cmp(&b.0.0));
            outlines
        })
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    fn probe_width(cx: &mut VisualTestContext) -> f32 {
        filled(cx, gpui::black())
            .first()
            .expect("比べる用のラベルを描くこと")
            .0
            .2
    }

    /// 期待値は Compose の AssistChipTokens の高さ 32dp・CornerSmall (8dp)・枠線 (OutlineVariant、1dp) と、
    /// `ContentPadding` の 8dp、アイコン 18dp、`ChipArrangement` がラベルの左右に取る 8dp
    #[gpui::test]
    fn assist_は枠線で囲みアイコンとラベルの左右に余白を取る(
        cx: &mut TestAppContext,
    ) {
        for (icon, padding) in [(false, 32.0), (true, 50.0)] {
            let cx = add_view(cx, "Directions", move || {
                let chip = Chip::new("chip", "Directions");
                if icon {
                    chip.icon(md3_icons::DIRECTIONS)
                } else {
                    chip
                }
            });
            let width = probe_width(cx) + padding;
            let outline_variant = color(cx, ColorRole::OutlineVariant);
            assert_eq!(
                outlines(cx),
                vec![((0.0, 0.0, width, 32.0), 1.0, outline_variant, 8.0)],
                "icon: {icon}"
            );
        }
    }

    /// 期待値は Compose の `defaultElevatedAssistChipColors` と `defaultElevatedSuggestionChipColors` の容器の色
    /// (SurfaceContainerLow)。Elevated は枠線を持たない
    #[gpui::test]
    fn elevated_は枠線の代わりに容器を塗る(cx: &mut TestAppContext) {
        for kind in [ChipKind::Assist, ChipKind::Suggestion] {
            let cx = add_view(cx, "Thanks!", move || {
                Chip::new("chip", "Thanks!")
                    .kind(kind)
                    .variant(ChipVariant::Elevated)
            });
            let width = probe_width(cx) + 32.0;
            let container = color(cx, ColorRole::SurfaceContainerLow);
            assert_eq!(
                filled(cx, container),
                vec![((0.0, 0.0, width, 32.0), 8.0)],
                "{kind:?}"
            );
            assert_eq!(outlines(cx), vec![], "{kind:?}");
        }
    }

    /// 期待値は AssistChipTokens の `FlatDisabledOutline*` (OnSurface の 12%) と `ElevatedDisabledContainer*`
    /// (OnSurface の 12%)
    #[gpui::test]
    fn 無効なチップは枠線と容器を無効の色で描く(cx: &mut TestAppContext) {
        let cx = add_view(cx, "Call", || Chip::new("chip", "Call").disabled(true));
        let disabled = color(cx, ColorRole::OnSurface).opacity(0.12);
        let ((_, _, width, _), _, outline, _) = outlines(cx)[0];
        assert_eq!(outline, disabled);

        let cx = add_view(cx, "Call", || {
            Chip::new("chip", "Call")
                .variant(ChipVariant::Elevated)
                .disabled(true)
        });
        assert_eq!(filled(cx, disabled), vec![((0.0, 0.0, width, 32.0), 8.0)]);
    }

    /// 状態レイヤーはラベルの色 (Assist は OnSurface) を 8% 重ねる
    #[gpui::test]
    fn hover_でラベルの色の状態レイヤーを重ねる(cx: &mut TestAppContext) {
        let cx = add_view(cx, "Share", || {
            Chip::new("chip", "Share").variant(ChipVariant::Elevated)
        });
        let container = color(cx, ColorRole::SurfaceContainerLow);
        let hovered = container.blend(color(cx, ColorRole::OnSurface).opacity(0.08));
        cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::none());
        assert_eq!(filled(cx, hovered).len(), 1);
    }

    #[gpui::test]
    fn 押すとハンドラを呼び無効なら呼ばない(cx: &mut TestAppContext) {
        for disabled in [false, true] {
            let clicks = Rc::new(RefCell::new(0));
            let cx = add_view(cx, "Share", {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    Chip::new("chip", "Share")
                        .disabled(disabled)
                        .on_click(move |_, _, _| *clicks.borrow_mut() += 1)
                }
            });
            cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
            assert_eq!(
                *clicks.borrow(),
                usize::from(!disabled),
                "disabled: {disabled}"
            );
        }
    }

    /// リングは容器の外側 5dp に、容器の角丸に 5dp を足した半径で描く。
    /// 枠線の色は Material Web と同じく、フォーカス中は `FlatFocusOutlineColor` (Assist は OnSurface、Suggestion は OnSurfaceVariant) にする
    #[gpui::test]
    fn tab_のフォーカスをリングと枠線の色で示す(cx: &mut TestAppContext) {
        for (kind, focused) in [
            (ChipKind::Assist, ColorRole::OnSurface),
            (ChipKind::Suggestion, ColorRole::OnSurfaceVariant),
        ] {
            let cx = add_view(cx, "Thanks!", move || {
                Chip::new("chip", "Thanks!").kind(kind)
            });
            cx.update(|window, _| window.activate_window());
            cx.simulate_keystrokes("tab");
            let width = probe_width(cx) + 32.0;
            let focused = color(cx, focused);
            let ring = color(cx, ColorRole::Secondary);
            assert_eq!(
                outlines(cx),
                vec![
                    ((-5.0, -5.0, width + 10.0, 42.0), 3.0, ring, 13.0),
                    ((0.0, 0.0, width, 32.0), 1.0, focused, 8.0),
                ],
                "{kind:?}"
            );
        }
    }

    /// 期待値は Compose の `defaultFilterChipColors` と `filterChipBorder`。選ばれると SecondaryContainer で塗り、
    /// 枠線の太さ (`FlatSelectedOutlineWidth`) は 0 になる。ラベルの左の間隔はアイコンの有無で変わらないため、選択の印の 18dp だけ広がる
    #[gpui::test]
    fn 選ばれた_filter_は容器を塗り枠線を描かず印の分だけ広がる(
        cx: &mut TestAppContext,
    ) {
        let cx = add_view(cx, "Vegan", || FilterChip::new("chip", "Vegan", false));
        let width = probe_width(cx) + 32.0;
        let outline_variant = color(cx, ColorRole::OutlineVariant);
        assert_eq!(
            outlines(cx),
            vec![((0.0, 0.0, width, 32.0), 1.0, outline_variant, 8.0)]
        );

        let cx = add_view(cx, "Vegan", || FilterChip::new("chip", "Vegan", true));
        let container = color(cx, ColorRole::SecondaryContainer);
        assert_eq!(
            filled(cx, container),
            vec![((0.0, 0.0, width + 18.0, 32.0), 8.0)]
        );
        assert_eq!(outlines(cx), vec![]);
    }

    /// 期待値は FilterChipTokens の `ElevatedUnselectedContainerColor` (SurfaceContainerLow) と
    /// `ElevatedSelectedContainerColor` (SecondaryContainer)
    #[gpui::test]
    fn elevated_の_filter_は選ばれているかで容器の色を変える(
        cx: &mut TestAppContext,
    ) {
        for (selected, role) in [
            (false, ColorRole::SurfaceContainerLow),
            (true, ColorRole::SecondaryContainer),
        ] {
            let cx = add_view(cx, "Vegan", move || {
                FilterChip::new("chip", "Vegan", selected).variant(ChipVariant::Elevated)
            });
            let container = color(cx, role);
            assert_eq!(filled(cx, container).len(), 1, "selected: {selected}");
            assert_eq!(outlines(cx), vec![], "selected: {selected}");
        }
    }

    /// 期待値は FilterChipTokens の `FlatDisabledSelectedContainer*` と `ElevatedDisabledContainer*` (どちらも OnSurface の 12%)
    #[gpui::test]
    fn 無効な選ばれた_filter_は無効の容器の色で描く(cx: &mut TestAppContext) {
        for variant in [ChipVariant::Flat, ChipVariant::Elevated] {
            let cx = add_view(cx, "Delivery", move || {
                FilterChip::new("chip", "Delivery", true)
                    .variant(variant)
                    .disabled(true)
            });
            let disabled = color(cx, ColorRole::OnSurface).opacity(0.12);
            assert_eq!(filled(cx, disabled).len(), 1, "{variant:?}");
        }
    }

    #[gpui::test]
    fn filter_は押すと反転した値を求め無効なら求めない(
        cx: &mut TestAppContext,
    ) {
        for (selected, disabled, expected) in [
            (false, false, vec![true]),
            (true, false, vec![false]),
            (false, true, vec![]),
        ] {
            let changes = Rc::new(RefCell::new(Vec::new()));
            let cx = add_view(cx, "Vegan", {
                let changes = changes.clone();
                move || {
                    let changes = changes.clone();
                    FilterChip::new("chip", "Vegan", selected)
                        .disabled(disabled)
                        .on_change(move |value, _, _| changes.borrow_mut().push(*value))
                }
            });
            cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
            assert_eq!(
                *changes.borrow(),
                expected,
                "selected: {selected}, disabled: {disabled}"
            );
        }
    }
}
