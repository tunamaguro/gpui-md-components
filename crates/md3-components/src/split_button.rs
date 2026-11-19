// 寸法・形・色の選び方・モーションは Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SplitButton.kt
//         (`SplitButtonLayout`、`SplitButtonDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive の分割ボタン

use std::cell::Cell;
use std::f32::consts::PI;
use std::rc::Rc;

use gpui::{
    AnimationExt, AnyElement, App, ClickEvent, Div, ElementId, IntoElement, ParentElement,
    RenderOnce, SharedString, SpringAnimation, StatefulInteractiveElement, StyleRefinement, Styled,
    Transformation, Window, canvas, div, prelude::FluentBuilder, radians, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::press::{PressArea, PressScope};
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::component::{
    button_small, split_button_large, split_button_medium, split_button_small,
    split_button_x_large, split_button_x_small,
};
use md3_theme::md3_tokens::motion::MotionSchemeKey;
use md3_theme::md3_tokens::shape::{CornerShape, CornerSize};
use md3_theme::md3_tokens::{Dp, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::button::{Morph, RingSlot, paint_ring};
use crate::{Button, ButtonSize, ButtonVariant, Menu, util};

/// 分割ボタンの種類。強調の度合いが高い順に Filled、FilledTonal、Elevated、Outlined
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SplitButtonVariant {
    /// 塗りつぶしの分割ボタン。重要な操作や流れを完了させる操作に使う
    #[default]
    Filled,
    /// 影で浮かせた分割ボタン。模様のある背景と区別する必要がある場合に使う
    Elevated,
    /// 二次的な色で塗りつぶした分割ボタン。Filled と Outlined の中間の強調に使う
    FilledTonal,
    /// 枠線の分割ボタン。重要だが主ではない操作に使う
    Outlined,
}

impl SplitButtonVariant {
    fn button_variant(self) -> ButtonVariant {
        match self {
            Self::Filled => ButtonVariant::Filled,
            Self::Elevated => ButtonVariant::Elevated,
            Self::FilledTonal => ButtonVariant::FilledTonal,
            Self::Outlined => ButtonVariant::Outlined,
        }
    }
}

/// サイズごとの寸法
struct SizeTokens {
    /// 前のボタンの内容の前と後の余白
    leading_padding: (Dp, Dp),
    /// 後ろのボタンの内容の前と後の余白
    trailing_padding: (Dp, Dp),
    /// 2 つのボタンが向き合う角の大きさ
    inner_corner: CornerSize,
    /// 押している間の、向き合う角の大きさ
    inner_pressed_corner: CornerSize,
    trailing_icon_size: Dp,
    between_space: Dp,
    height: Dp,
}

fn size_tokens(size: ButtonSize) -> SizeTokens {
    macro_rules! tokens {
        ($tokens:ident) => {
            SizeTokens {
                leading_padding: (
                    $tokens::LEADING_BUTTON_LEADING_SPACE,
                    $tokens::LEADING_BUTTON_TRAILING_SPACE,
                ),
                trailing_padding: (
                    $tokens::TRAILING_BUTTON_LEADING_SPACE,
                    $tokens::TRAILING_BUTTON_TRAILING_SPACE,
                ),
                inner_corner: $tokens::INNER_CORNER_CORNER_SIZE,
                inner_pressed_corner: $tokens::INNER_PRESSED_CORNER_CORNER_SIZE,
                trailing_icon_size: $tokens::TRAILING_ICON_SIZE,
                between_space: $tokens::BETWEEN_SPACE,
                height: $tokens::CONTAINER_HEIGHT,
            }
        };
    }
    match size {
        ButtonSize::ExtraSmall => tokens!(split_button_x_small),
        ButtonSize::Small => tokens!(split_button_small),
        ButtonSize::Medium => tokens!(split_button_medium),
        ButtonSize::Large => tokens!(split_button_large),
        ButtonSize::ExtraLarge => tokens!(split_button_x_large),
    }
}

/// 高さ `height` の形の角の大きさ `corner` を dp で
fn corner_dp(corner: CornerSize, height: Dp) -> f32 {
    match corner {
        CornerSize::Dp(value) => value.0,
        CornerSize::Percent(percent) => height.0 * percent / 100.0,
    }
}

/// 外側の角を丸く、2 つのボタンが向き合う角を `inner` にした形
fn shape(leading: bool, inner: CornerSize) -> CornerShape {
    let outer = CornerShape::CIRCLE.top_start;
    if leading {
        CornerShape {
            top_start: outer,
            bottom_start: outer,
            top_end: inner,
            bottom_end: inner,
        }
    } else {
        CornerShape {
            top_start: inner,
            bottom_start: inner,
            top_end: outer,
            bottom_end: outer,
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 Expressive の分割ボタン
///
/// 主な操作をする前のボタンと、関連する操作 (主にメニューを開く) をする後ろのボタンを 2dp 空けて並べる。
/// 前のボタンは子の内容をラベルとして描き、押されると [`SplitButton::on_click`] のハンドラを呼ぶ。
/// 後ろのボタンは下向きの矢印を描き、押されると [`SplitButton::on_trailing_click`] のハンドラを呼び、
/// [`SplitButton::menu`] のメニューの開閉を求める。メニューが開いている間、後ろのボタンは丸い形になり、
/// 内容の色を重ね、矢印を上に向ける。
/// ポインターで押している間は、押しているボタンの向き合う角を大きく丸める
///
/// 支援技術には、2 つのボタンがそれぞれ role `button` として公開される。
/// 前のボタンは子孫に置いたテキストか [`SplitButton::aria_label`] が、後ろのボタンは [`SplitButton::new`] の
/// `trailing_label` がアクセシブルな名前になる。テキストは `gpui::text!` のように id を持つ要素にする。
/// マウスのクリック・フォーカス中の Enter と Space でそれぞれの操作をする。Tab では前、後ろの順に到達する
///
/// [`Styled`] で指定したスタイルは外側の箱に付く。箱を広げると、前のボタンが残りの幅に広がる
///
/// テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない。
/// 大きさは [`SplitButton::size`] で選ぶ
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    outer: Div,
    children: Vec<AnyElement>,
    trailing_label: SharedString,
    variant: SplitButtonVariant,
    size: ButtonSize,
    icon: Option<StyledIcon>,
    aria_label: Option<SharedString>,
    on_click: Option<ClickHandler>,
    on_trailing_click: Option<ClickHandler>,
    menu: Option<Menu>,
    disabled: bool,
    theme: Option<SharedTheme>,
}

impl SplitButton {
    /// [`SplitButtonVariant::Filled`]、[`ButtonSize::Small`] の分割ボタンを作る
    ///
    /// `trailing_label` は後ろのボタンのアクセシブルな名前にする。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// フォーカスと押している状態が失われる
    pub fn new(id: impl Into<ElementId>, trailing_label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            // 利用者が Styled で指定した display を render で上書きしないよう、既定はここで付ける
            outer: div().flex(),
            children: Vec::new(),
            trailing_label: trailing_label.into(),
            variant: SplitButtonVariant::default(),
            size: ButtonSize::default(),
            icon: None,
            aria_label: None,
            on_click: None,
            on_trailing_click: None,
            menu: None,
            disabled: false,
            theme: None,
        }
    }

    /// 分割ボタンの種類を設定する
    pub fn variant(mut self, variant: SplitButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 分割ボタンの大きさを設定する
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// 前のボタンのラベルの前にアイコンを置く
    ///
    /// アイコンはラベルと同じ色になり、大きさとラベルとの間隔はサイズに合わせる。
    /// アイコンは装飾として扱い、アクセシブルな名前には含めない
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 子の内容の代わりに `label` を前のボタンのアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 前のボタンが押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 後ろのボタンが押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// [`SplitButton::menu`] があれば、メニューの開閉を求める前に呼ぶ
    pub fn on_trailing_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_trailing_click = Some(Box::new(handler));
        self
    }

    /// 後ろのボタンを `menu` を開くボタンにする
    ///
    /// メニューの名前は既定で `trailing_label` になる
    pub fn menu(mut self, menu: Menu) -> Self {
        self.menu = Some(menu);
        self
    }

    /// 2 つのボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは無効の色で描かれ、押されてもハンドラを呼ばず、Tab でも到達できない
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

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let tokens = size_tokens(self.size);
        let variant = self.variant.button_variant();
        let disabled = self.disabled;
        let expanded = self.menu.as_ref().is_some_and(|menu| menu.expanded);
        // Compose は形を DefaultEffects のバネで変える
        let effects = theme.spring(MotionSchemeKey::DefaultEffects);

        let id = self.id.clone();
        let (leading_focus, trailing_focus, leading_pressed, trailing_pressed, trailing_height) =
            window.with_element_namespace(id.clone(), |window| {
                (
                    util::keyed_focus_handle(&"leading".into(), window, cx),
                    util::keyed_focus_handle(&"trailing".into(), window, cx),
                    window.use_keyed_state("leading-pressed", cx, |_, _| false),
                    window.use_keyed_state("trailing-pressed", cx, |_, _| false),
                    window
                        .use_keyed_state("trailing-height", cx, |_, _| Rc::new(Cell::new(None)))
                        .read(cx)
                        .clone(),
                )
            });

        // 間は 2dp で、前のボタンの外側にはみ出すリングは後ろのボタンに隠れるため、2 つのボタンの後に描く
        let ring = RingSlot::default();
        let mut leading = Button::new("leading")
            .variant(variant)
            .size(self.size)
            .disabled(disabled)
            .children(self.children);
        if let Some(icon) = self.icon {
            leading = leading.icon(icon);
        }
        let leading_focus_visible = gpui_headless::is_focus_visible(&leading_focus, window, cx);
        let mut look = leading.look(&theme, leading_focus_visible);
        // 向き合う角の半径 (dp) をバネの値にし、0dp の形と 1dp の形の補間で描く。
        // 状態がどう変わっても、今の半径から目標の半径へ同じ速さの動きで変わる
        look.corner_shape = Some(shape(true, CornerSize::ZERO));
        look.morph = Some(Morph {
            shape: shape(true, CornerSize::Dp(Dp(1.0))),
            target: corner_dp(
                if *leading_pressed.read(cx) {
                    tokens.inner_pressed_corner
                } else {
                    tokens.inner_corner
                },
                tokens.height,
            ),
            spring: effects,
            fade_outline: false,
        });
        look.ring_slot = Some(ring.clone());
        // Compose の分割ボタンの枠線は、どのサイズでも Small の太さ (`outlinedButtonBorder`)
        look.outline_width(dp(button_small::OUTLINED_OUTLINE_WIDTH));
        // Compose の分割ボタンは左右にだけ余白を取る
        look.padding = tokens.leading_padding;
        look.vertical_padding = Dp(0.0);
        look.min_width = supplement::split_button::MIN_WIDTH;
        let leading_button = look.apply(
            HeadlessButton::new("leading")
                .track_focus(&leading_focus)
                .disabled(disabled)
                .when_some(self.aria_label, |this, label| this.aria_label(label))
                .when_some(self.on_click, |this, handler| this.on_click(handler)),
        );

        let mut trailing = Button::new("trailing")
            .variant(variant)
            .size(self.size)
            .disabled(disabled);
        let trailing_focus_visible = gpui_headless::is_focus_visible(&trailing_focus, window, cx);
        let mut look = trailing.look(&theme, trailing_focus_visible);
        // 下向きの矢印。メニューが開いている間は上に向ける
        look.child(
            svg()
                .data(
                    StyledIcon::from(md3_icons::KEYBOARD_ARROW_DOWN)
                        .svg()
                        .as_bytes(),
                )
                .flex_none()
                .size(dp(tokens.trailing_icon_size))
                .text_color(look.content())
                .with_spring(
                    "arrow",
                    SpringAnimation::new(theme.spring(MotionSchemeKey::DefaultSpatial))
                        .to(expanded),
                    |this, phase| {
                        this.with_transformation(Transformation::rotate(radians(PI * phase.0)))
                    },
                ),
        );
        // メニューが開いている間は丸い形にし、内容の色を押したときの不透明度で重ねる
        if expanded {
            look.overlay_content(state::PRESSED_STATE_LAYER_OPACITY);
        }
        look.corner_shape = Some(shape(false, CornerSize::ZERO));
        look.morph = Some(Morph {
            shape: shape(false, CornerSize::Dp(Dp(1.0))),
            // 押している間は、開いているかによらず押しているときの形にする。
            // 開いている間は高さの半分の半径で丸い形にする
            target: if *trailing_pressed.read(cx) {
                corner_dp(tokens.inner_pressed_corner, tokens.height)
            } else if expanded {
                // Styled で高さを変えても丸くなるよう、前の描画で測った高さを使う
                trailing_height.get().unwrap_or(tokens.height.0) / 2.0
            } else {
                corner_dp(tokens.inner_corner, tokens.height)
            },
            spring: effects,
            fade_outline: false,
        });
        look.ring_slot = Some(ring.clone());
        look.outline_width(dp(button_small::OUTLINED_OUTLINE_WIDTH));
        look.padding = tokens.trailing_padding;
        look.vertical_padding = Dp(0.0);
        look.min_width = supplement::split_button::MIN_WIDTH;
        let mut trailing_button = look.apply(
            HeadlessButton::new("trailing")
                .track_focus(&trailing_focus)
                .disabled(disabled)
                .aria_label(self.trailing_label.clone())
                .when_some(self.on_trailing_click, |this, handler| {
                    this.on_click(handler)
                }),
        );
        if let Some(menu) = self.menu {
            trailing_button = window.with_element_namespace(id.clone(), |window| {
                menu.attach(
                    trailing_button,
                    &"trailing".into(),
                    Some(&self.trailing_label),
                    &theme,
                    window,
                    cx,
                )
            });
        }

        self.outer.child(
            PressScope::new(id, [leading_pressed.clone(), trailing_pressed.clone()])
                .flex_grow_1()
                .flex()
                .gap(dp(tokens.between_space))
                .child(
                    PressArea::new("leading-press", &leading_pressed)
                        .disabled(disabled)
                        .flex_grow_1()
                        .flex()
                        .child(leading_button),
                )
                .child(
                    PressArea::new("trailing-press", &trailing_pressed)
                        .disabled(disabled)
                        .flex_none()
                        .flex()
                        .child(trailing_button)
                        .child(
                            canvas(
                                move |bounds, window, _| {
                                    let height = Some(f32::from(bounds.size.height));
                                    if trailing_height.replace(height) != height {
                                        // 描画中の refresh は効かないため、次のフレームを求める
                                        window.request_animation_frame();
                                    }
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        ),
                )
                .child(paint_ring(ring, theme.clone())),
        )
    }
}

impl Styled for SplitButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.outer.style()
    }
}

impl ParentElement for SplitButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui::{
        AnyElement, Context, Modifiers, MouseButton, Render, TestAppContext, VisualTestContext,
        div, point, px,
    };
    use md3_theme::md3_color::ColorRole;

    use super::*;
    use crate::{MenuGroup, MenuItem};

    struct ElementView(Box<dyn Fn() -> AnyElement>);

    impl Render for ElementView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            // 原点のマウスの位置で hover にならないよう、ボタンを原点から離す
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

    /// 高さ 40dp の容器の、x・幅・左上と右上の角の半径を左から順に
    fn containers(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let mut found: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.bounds.size.height.0 / scale == 40.0)
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

    /// 前のボタンを押した回数を `clicks` に数え、メニューの開閉の求めを `expanded` に書く分割ボタン
    fn split(clicks: Rc<Cell<usize>>, expanded: Rc<Cell<bool>>) -> AnyElement {
        let request = expanded.clone();
        SplitButton::new("split", "More")
            .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
            .menu(
                Menu::new("menu", expanded.get())
                    .on_expanded_change(move |value: &bool, window, _| {
                        request.set(*value);
                        window.refresh();
                    })
                    .group(MenuGroup::new("group").item(MenuItem::new("edit", "Edit"))),
            )
            .child(div().w(px(60.0)).h(px(10.0)))
            .into_any_element()
    }

    fn move_away(cx: &mut VisualTestContext) {
        cx.simulate_mouse_move(point(px(10.0), px(10.0)), None, Modifiers::none());
        cx.run_until_parked();
    }

    #[gpui::test]
    fn 外側の角を丸め_向き合う角を小さく丸め_間を空ける(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            split(Rc::new(Cell::new(0)), Rc::new(Cell::new(false)))
        });
        let found = containers(cx);
        assert_eq!(found.len(), 2);
        assert_eq!(
            (found[0].2, found[0].3),
            (20.0, 4.0),
            "前は左を丸くすること"
        );
        assert_eq!(
            (found[1].2, found[1].3),
            (4.0, 20.0),
            "後ろは右を丸くすること"
        );
        assert_eq!(
            found[1].0,
            found[0].0 + found[0].1 + 2.0,
            "間を 2dp 空けること"
        );
        assert_eq!(found[1].1, 48.0, "後ろは余白とアイコンの幅にすること");
    }

    #[gpui::test]
    fn 前のボタンは押している間だけ向き合う角を大きく丸め_離すと押されたことを伝える(
        cx: &mut TestAppContext,
    ) {
        let clicks = Rc::new(Cell::new(0));
        let state = clicks.clone();
        let cx = open(cx, move || split(state.clone(), Rc::new(Cell::new(false))));
        let position = point(px(130.0), px(120.0));
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        let pressed = containers(cx);
        assert_eq!(
            (pressed[0].2, pressed[0].3),
            (20.0, 12.0),
            "押している間は大きく丸めること"
        );
        assert_eq!(pressed[1].2, 4.0, "押していない後ろは変えないこと");
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
        move_away(cx);
        assert_eq!(clicks.get(), 1);
        let released = containers(cx);
        assert_eq!(released[0].3, 4.0, "離すと戻すこと");
    }

    #[gpui::test]
    fn 後ろのボタンはメニューを開くよう求め_開いている間は丸い形で内容の色を重ねる(
        cx: &mut TestAppContext,
    ) {
        let expanded = Rc::new(Cell::new(false));
        let state = expanded.clone();
        let cx = open(cx, move || split(Rc::new(Cell::new(0)), state.clone()));
        let trailing = *containers(cx).last().expect("後ろのボタンを描くこと");
        cx.simulate_click(
            point(px(100.0 + trailing.0 + trailing.1 / 2.0), px(120.0)),
            Modifiers::none(),
        );
        move_away(cx);
        assert!(expanded.get(), "開くよう求めること");
        let overlay = cx.update(|_, cx| {
            let theme = cx.md3_theme();
            theme
                .color(ColorRole::OnPrimary)
                .opacity(state::PRESSED_STATE_LAYER_OPACITY)
        });
        let open_trailing = cx.update(|window, _| {
            let background: gpui::Background = overlay.into();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background)
                .count()
        });
        assert_eq!(open_trailing, 1, "内容の色を重ねること");
        let open_trailing = containers(cx);
        assert_eq!(open_trailing[1].0, trailing.0);
        assert_eq!(
            (open_trailing[1].2, open_trailing[1].3),
            (20.0, 20.0),
            "開いている間は丸い形にすること"
        );
    }

    #[gpui::test]
    fn 無効な分割ボタンは押しても伝えず_形を変えない(
        cx: &mut TestAppContext,
    ) {
        let clicks = Rc::new(Cell::new(0));
        let state = clicks.clone();
        let cx = open(cx, move || {
            div()
                .child(
                    SplitButton::new("split", "More")
                        .disabled(true)
                        .on_click({
                            let clicks = state.clone();
                            move |_, _, _| clicks.set(clicks.get() + 1)
                        })
                        .child(div().w(px(60.0)).h(px(10.0))),
                )
                .into_any_element()
        });
        let position = point(px(130.0), px(120.0));
        cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::none());
        cx.run_until_parked();
        let found = containers(cx);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].3, 4.0, "無効なら押しても形を変えないこと");
        cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::none());
        move_away(cx);
        assert_eq!(clicks.get(), 0);
    }

    #[gpui::test]
    fn 前のボタンのフォーカスリングは後ろのボタンより後に描く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            split(Rc::new(Cell::new(0)), Rc::new(Cell::new(false)))
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        let (ring, trailing) = cx.update(|window, cx| {
            let scale = window.scale_factor();
            let ring = cx
                .md3_theme()
                .color(md3_theme::md3_tokens::focus_ring::COLOR);
            let quads = window.painted_quads();
            (
                quads
                    .iter()
                    .filter(|quad| quad.border_color == ring && quad.border_widths.top.0 > 0.0)
                    .max_by_key(|quad| quad.order)
                    .map(|quad| (quad.order, quad.bounds.origin.x.0 / scale - 100.0)),
                quads
                    .iter()
                    .filter(|quad| quad.bounds.size.height.0 / scale == 40.0)
                    .max_by(|a, b| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
                    .map(|quad| quad.order),
            )
        });
        let (ring_order, ring_x) = ring.expect("リングを描くこと");
        assert!(ring_x < 0.0, "前のボタンにリングを描くこと");
        assert!(ring_order > trailing.expect("後ろのボタンを描くこと"));
    }

    #[gpui::test]
    fn 開いている枠線の分割ボタンでも後ろのボタンの_hover_を描く(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            SplitButton::new("split", "More")
                .variant(SplitButtonVariant::Outlined)
                .menu(
                    Menu::new("menu", true)
                        .group(MenuGroup::new("group").item(MenuItem::new("edit", "Edit"))),
                )
                .child(div().w(px(60.0)).h(px(10.0)))
                .into_any_element()
        });
        let trailing = *containers(cx).last().expect("後ろのボタンを描くこと");
        cx.simulate_mouse_move(
            point(px(100.0 + trailing.0 + trailing.1 / 2.0), px(120.0)),
            None,
            Modifiers::none(),
        );
        cx.run_until_parked();
        let (overlay, hover) = cx.update(|window, cx| {
            let content = cx.md3_theme().color(ColorRole::OnSurfaceVariant);
            let overlay: gpui::Background =
                content.opacity(state::PRESSED_STATE_LAYER_OPACITY).into();
            let hover: gpui::Background = content.opacity(state::HOVER_STATE_LAYER_OPACITY).into();
            let scale = window.scale_factor();
            // 後ろのボタンの容器の大きさの層
            let quads: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| {
                    quad.bounds.origin.x.0 / scale - 100.0 == trailing.0
                        && quad.bounds.size.height.0 / scale == 40.0
                })
                .collect();
            (
                quads.iter().any(|quad| quad.background == overlay),
                quads.iter().any(|quad| quad.background == hover),
            )
        });
        assert!(overlay, "開いている間は内容の色を重ねること");
        assert!(hover, "hover の状態レイヤーも描くこと");
    }

    #[gpui::test]
    fn 高さを変えても開いている後ろのボタンは丸くする(
        cx: &mut TestAppContext,
    ) {
        let cx = open(cx, || {
            SplitButton::new("split", "More")
                .h(px(56.0))
                .menu(
                    Menu::new("menu", true)
                        .group(MenuGroup::new("group").item(MenuItem::new("edit", "Edit"))),
                )
                .child(div().w(px(60.0)).h(px(10.0)))
                .into_any_element()
        });
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        let trailing = cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.bounds.size.height.0 / scale == 56.0)
                .max_by(|a, b| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0))
                .map(|quad| {
                    (
                        quad.corner_radii.top_left.0 / scale,
                        quad.corner_radii.top_right.0 / scale,
                    )
                })
        });
        assert_eq!(trailing, Some((24.0, 24.0)), "短辺の半分の半径にすること");
    }
}
