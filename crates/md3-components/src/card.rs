// 色の選択・状態ごとの elevation・無効時の色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Card.kt
//         (`Card`、`ElevatedCard`、`OutlinedCard`、`CardDefaults`、`CardElevation`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のカード

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Refineable, RenderOnce, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_headless::button::Button as HeadlessButton;

use crate::shape::Shaped;
use crate::util;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{elevated_card, filled_card, outlined_card};
use md3_theme::md3_tokens::elevation::Elevation;
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::{Dp, focus_ring, state, supplement};
use md3_theme::{SharedTheme, Theme, ThemeExt, dp};

/// 容器の層がカードの hover を知るための group 名
const GROUP: &str = "md3-card";

/// カードの種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardVariant {
    /// 背景と異なる色で塗ったカード
    #[default]
    Filled,
    /// 影で浮かせたカード
    Elevated,
    /// 枠線で囲んだカード
    Outlined,
}

/// 色 `base` に `overlay` の色を不透明度付きで重ねた色。Compose の `compositeOver`
#[derive(Clone, Copy)]
struct Layered {
    base: ColorRole,
    overlay: Option<(ColorRole, f32)>,
}

impl Layered {
    fn resolve(self, theme: &Theme) -> Hsla {
        let base = theme.color(self.base);
        match self.overlay {
            Some((role, opacity)) => base.blend(theme.color(role).opacity(opacity)),
            None => base,
        }
    }
}

/// 状態ごとの影の高さ
struct Elevations {
    default: Elevation,
    hovered: Elevation,
    focused: Elevation,
    disabled: Elevation,
}

/// バリアントごとの色・形・枠線・影
struct VariantTokens {
    container: ColorRole,
    disabled_container: Layered,
    shape: ShapeKey,
    /// 枠線の太さ、色、無効なときの色
    outline: Option<(Dp, ColorRole, Layered)>,
    elevations: Elevations,
}

impl CardVariant {
    fn tokens(self) -> VariantTokens {
        match self {
            // `defaultCardColors`、`cardElevation`
            Self::Filled => VariantTokens {
                container: filled_card::CONTAINER_COLOR,
                disabled_container: Layered {
                    base: filled_card::CONTAINER_COLOR,
                    overlay: Some((
                        filled_card::DISABLED_CONTAINER_COLOR,
                        filled_card::DISABLED_CONTAINER_OPACITY,
                    )),
                },
                shape: filled_card::CONTAINER_SHAPE,
                outline: None,
                elevations: Elevations {
                    default: filled_card::CONTAINER_ELEVATION,
                    hovered: filled_card::HOVER_CONTAINER_ELEVATION,
                    focused: filled_card::FOCUS_CONTAINER_ELEVATION,
                    disabled: filled_card::DISABLED_CONTAINER_ELEVATION,
                },
            },
            // `defaultElevatedCardColors`、`elevatedCardElevation`
            Self::Elevated => VariantTokens {
                container: elevated_card::CONTAINER_COLOR,
                disabled_container: Layered {
                    base: elevated_card::DISABLED_CONTAINER_COLOR,
                    overlay: Some((
                        elevated_card::DISABLED_CONTAINER_COLOR,
                        elevated_card::DISABLED_CONTAINER_OPACITY,
                    )),
                },
                shape: elevated_card::CONTAINER_SHAPE,
                outline: None,
                elevations: Elevations {
                    default: elevated_card::CONTAINER_ELEVATION,
                    hovered: elevated_card::HOVER_CONTAINER_ELEVATION,
                    focused: elevated_card::FOCUS_CONTAINER_ELEVATION,
                    disabled: elevated_card::DISABLED_CONTAINER_ELEVATION,
                },
            },
            // `defaultOutlinedCardColors`、`outlinedCardBorder`、`outlinedCardElevation`。
            // `outlinedCardElevation` は hover とフォーカスにトークンを使わず、既定の高さのままにする
            Self::Outlined => VariantTokens {
                container: outlined_card::CONTAINER_COLOR,
                disabled_container: Layered {
                    base: outlined_card::CONTAINER_COLOR,
                    overlay: None,
                },
                shape: outlined_card::CONTAINER_SHAPE,
                outline: Some((
                    outlined_card::OUTLINE_WIDTH,
                    outlined_card::OUTLINE_COLOR,
                    Layered {
                        base: elevated_card::CONTAINER_COLOR,
                        overlay: Some((
                            outlined_card::DISABLED_OUTLINE_COLOR,
                            outlined_card::DISABLED_OUTLINE_OPACITY,
                        )),
                    },
                )),
                elevations: Elevations {
                    default: outlined_card::CONTAINER_ELEVATION,
                    hovered: outlined_card::CONTAINER_ELEVATION,
                    focused: outlined_card::CONTAINER_ELEVATION,
                    disabled: outlined_card::DISABLED_CONTAINER_ELEVATION,
                },
            },
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 のカード
///
/// 1 つの話題についての内容と操作をまとめる容器。子を縦に並べ、バリアントの容器の色・枠線・影の上に描く。
/// 内容の色は OnSurface で、子のテキストはこの色を受け継ぐ。余白は持たないため、[`Styled`] で付ける。
///
/// [`Card::on_click`] を指定すると押せるカードになり、支援技術には role `button` として公開される。
/// アクセシブルな名前は子孫に置いたテキストか [`Card::aria_label`] で与える。
/// マウスのクリック・フォーカス中の Enter と Space で [`Card::on_click`] のハンドラを呼び、hover とフォーカスを状態レイヤーと影で示す。
/// 押せるカードの子に、ボタンなどの操作できる要素を置かないこと。
/// 指定しない場合は見た目だけの容器で、支援技術には子だけが公開される
///
/// [`Styled`] で指定したスタイルはカードの箱に付く。大きさ・余白・子の並べ方の指定に使う。
/// 色や形は変えられないため、[`Card::variant`] を使う。子は角丸で切り抜かない
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    style: StyleRefinement,
    children: Vec<AnyElement>,
    variant: CardVariant,
    on_click: Option<ClickHandler>,
    aria_label: Option<SharedString>,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    theme: Option<SharedTheme>,
}

impl Card {
    /// [`CardVariant::Filled`] のカードを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。押せるカードで [`Card::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            children: Vec::new(),
            variant: CardVariant::default(),
            on_click: None,
            aria_label: None,
            disabled: false,
            focus_handle: None,
            theme: None,
        }
    }

    /// カードの種類を設定する
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 押せるカードにし、押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 押せるカードで、子の内容の代わりに `label` をアクセシブルな名前にする
    ///
    /// 子孫のテキストをすべて並べた名前が長すぎる場合や、子孫に id を持つテキストが無い場合に使う。
    /// 押せないカードでは使わない
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// カードを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なカードは無効の色と影で描かれる。押せるカードは押されてもハンドラを呼ばず、Tab でも到達できず、
    /// 支援技術には disabled として公開される。フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押せるカードで、`focus_handle` をこのカードのフォーカスとして使う
    ///
    /// 指定しない場合は押せるカードが内部でフォーカスを持つ
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

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let tokens = self.variant.tokens();
        let disabled = self.disabled;
        let content = theme.color(supplement::card::CONTENT_COLOR);
        let content = if disabled {
            content.opacity(supplement::card::DISABLED_CONTENT_OPACITY)
        } else {
            content
        };
        let container = if disabled {
            tokens.disabled_container.resolve(&theme)
        } else {
            theme.color(tokens.container)
        };

        // 角丸は描画時の大きさで決まるため、背景・枠線・影は Shaped で包んだ層に描き、カード自身は透明な箱にする。
        // `interactive` は hover とフォーカスを示すか
        let layer = |interactive: bool, focus_visible: bool| {
            // 状態レイヤーは内容の色を重ねる (Compose の `ripple()` は LocalContentColor を使う)
            let state_layer = |opacity: f32| container.blend(content.opacity(opacity));
            let elevations = &tokens.elevations;
            let layer = div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .when_some(tokens.outline, |this, (width, color, disabled_color)| {
                    this.border(dp(width)).border_color(if disabled {
                        disabled_color.resolve(&theme)
                    } else {
                        theme.color(color)
                    })
                })
                .map(|this| {
                    if disabled {
                        this.bg(container).shadow(theme.shadow(elevations.disabled))
                    } else if interactive && focus_visible {
                        this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                            .shadow(theme.shadow(elevations.focused))
                    } else {
                        this.bg(container).shadow(theme.shadow(elevations.default))
                    }
                })
                // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                .when(interactive, |this| {
                    let hovered_shadow = theme.shadow(elevations.hovered);
                    this.group_hover(GROUP, |style| {
                        style
                            .bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                            .shadow(hovered_shadow)
                    })
                });
            Shaped::new(layer, theme.clone(), tokens.shape)
        };

        match self.on_click {
            Some(on_click) => {
                let focus_handle = self
                    .focus_handle
                    .unwrap_or_else(|| util::keyed_focus_handle(&self.id, window, cx));
                // リングはカードの外側に描くため、カード自身のスタイルでは表せない
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                let mut card = HeadlessButton::new(self.id)
                    .track_focus(&focus_handle)
                    .disabled(disabled)
                    .group(GROUP)
                    .relative()
                    .flex()
                    .flex_col()
                    .text_color(content);
                card.style().refine(&self.style);
                card.child(layer(!disabled, focus_visible))
                    .children(self.children)
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
                        this.child(Shaped::new(ring, theme.clone(), tokens.shape).outset(outset))
                    })
                    .when_some(self.aria_label, |this, label| this.aria_label(label))
                    .on_click(on_click)
                    .into_any_element()
            }
            None => {
                let mut card = div()
                    .id(self.id)
                    .relative()
                    .flex()
                    .flex_col()
                    .text_color(content);
                card.style().refine(&self.style);
                card.child(layer(false, false))
                    .children(self.children)
                    .into_any_element()
            }
        }
    }
}

impl Styled for Card {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};

    use super::*;

    struct ElementView(Box<dyn Fn() -> Card>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // ルートの要素はウィンドウいっぱいに広がるため、カードを伸ばさない箱に包む
            div()
                .flex()
                .items_start()
                .child((self.0)().w(px(200.)).h(px(100.)))
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        card: impl Fn() -> Card + 'static,
    ) -> &mut VisualTestContext {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ElementView(Box::new(card))
        });
        // 状態レイヤーを描かないよう、マウスをカードの外に置く
        cx.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
        cx
    }

    /// 最初に描いた矩形 (カードの容器)
    fn container_quad(cx: &mut VisualTestContext) -> gpui::Quad {
        cx.update(|window, _| window.painted_quads()[0])
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 期待値は Compose の FilledCardTokens・ElevatedCardTokens・OutlinedCardTokens の容器の色と、
    /// どれも CornerMedium (12dp) の形
    #[gpui::test]
    fn バリアントごとの容器の色と形で描く(cx: &mut TestAppContext) {
        for (variant, role) in [
            (CardVariant::Filled, ColorRole::SurfaceContainerHighest),
            (CardVariant::Elevated, ColorRole::SurfaceContainerLow),
            (CardVariant::Outlined, ColorRole::Surface),
        ] {
            let cx = add_view(cx, move || Card::new("card").variant(variant));
            let quad = container_quad(cx);
            let scale = cx.update(|window, _| window.scale_factor());
            assert_eq!(quad.background, color(cx, role).into(), "{variant:?}");
            assert_eq!(quad.corner_radii.top_left.0 / scale, 12.0, "{variant:?}");
            assert_eq!(
                (
                    quad.bounds.size.width.0 / scale,
                    quad.bounds.size.height.0 / scale
                ),
                (200.0, 100.0),
                "{variant:?}"
            );
        }
    }

    /// 期待値は Compose の `outlinedCardBorder`。有効なら OutlineVariant、無効なら Outline の 12% を
    /// ElevatedCardTokens の容器の色 (SurfaceContainerLow) に重ねた色で、太さは 1dp
    #[gpui::test]
    fn outlined_は枠線を描き無効では枠線の色を変える(cx: &mut TestAppContext) {
        for disabled in [false, true] {
            let cx = add_view(cx, move || {
                Card::new("card")
                    .variant(CardVariant::Outlined)
                    .disabled(disabled)
            });
            let expected = if disabled {
                color(cx, ColorRole::SurfaceContainerLow)
                    .blend(color(cx, ColorRole::Outline).opacity(0.12))
            } else {
                color(cx, ColorRole::OutlineVariant)
            };
            // gpui は枠線を背景と別の矩形として子の後に描く
            let quad = cx.update(|window, _| {
                window
                    .painted_quads()
                    .into_iter()
                    .find(|quad| quad.border_widths.top.0 > 0.0)
                    .expect("枠線を描くこと")
            });
            let scale = cx.update(|window, _| window.scale_factor());
            assert_eq!(quad.border_color, expected, "disabled: {disabled}");
            assert_eq!(
                quad.border_widths.top.0 / scale,
                1.0,
                "disabled: {disabled}"
            );
        }
    }

    /// 期待値は Compose の `CardDefaults` の無効な容器の色。Filled は SurfaceVariant の 38% を容器の色に重ね、
    /// Elevated は Surface の 38% を Surface に重ね、Outlined は容器の色のまま
    #[gpui::test]
    fn 無効なカードは無効の容器の色で描く(cx: &mut TestAppContext) {
        for (variant, base, overlay) in [
            (
                CardVariant::Filled,
                ColorRole::SurfaceContainerHighest,
                Some((ColorRole::SurfaceVariant, 0.38)),
            ),
            (
                CardVariant::Elevated,
                ColorRole::Surface,
                Some((ColorRole::Surface, 0.38)),
            ),
            (CardVariant::Outlined, ColorRole::Surface, None),
        ] {
            let cx = add_view(cx, move || {
                Card::new("card")
                    .variant(variant)
                    .on_click(|_, _, _| {})
                    .disabled(true)
            });
            let mut expected = color(cx, base);
            if let Some((role, opacity)) = overlay {
                expected = expected.blend(color(cx, role).opacity(opacity));
            }
            assert_eq!(
                container_quad(cx).background,
                expected.into(),
                "{variant:?}"
            );
        }
    }

    /// 状態レイヤーは内容の色 (OnSurface) を 8% 重ねる
    #[gpui::test]
    fn 押せるカードだけが_hover_で状態レイヤーを重ねる(
        cx: &mut TestAppContext,
    ) {
        for clickable in [true, false] {
            let cx = add_view(cx, move || {
                Card::new("card").when(clickable, |this| this.on_click(|_, _, _| {}))
            });
            let container = color(cx, ColorRole::SurfaceContainerHighest);
            let hovered = container.blend(color(cx, ColorRole::OnSurface).opacity(0.08));
            cx.simulate_mouse_move(point(px(100.), px(50.)), None, Modifiers::none());
            let expected = if clickable { hovered } else { container };
            assert_eq!(
                container_quad(cx).background,
                expected.into(),
                "clickable: {clickable}"
            );
        }
    }

    #[gpui::test]
    fn 押せるカードは押すとハンドラを呼び無効なら呼ばない(
        cx: &mut TestAppContext,
    ) {
        for disabled in [false, true] {
            let clicks = Rc::new(Cell::new(0));
            let cx = add_view(cx, {
                let clicks = clicks.clone();
                move || {
                    let clicks = clicks.clone();
                    Card::new("card")
                        .disabled(disabled)
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                }
            });
            cx.simulate_click(point(px(100.), px(50.)), Modifiers::none());
            assert_eq!(clicks.get(), usize::from(!disabled), "disabled: {disabled}");
        }
    }

    #[gpui::test]
    fn 押せるカードだけが_tab_のフォーカスをリングで示す(
        cx: &mut TestAppContext,
    ) {
        for clickable in [true, false] {
            let cx = add_view(cx, move || {
                Card::new("card").when(clickable, |this| this.on_click(|_, _, _| {}))
            });
            cx.update(|window, _| window.activate_window());
            cx.simulate_keystrokes("tab");
            // リングは 200×100 の容器の外側 5px に描く
            let scale = cx.update(|window, _| window.scale_factor());
            let ring = cx.update(|window, _| {
                window.painted_quads().iter().any(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    ) == (-5., -5., 210., 110.)
                })
            });
            assert_eq!(ring, clickable, "clickable: {clickable}");
        }
    }
}
