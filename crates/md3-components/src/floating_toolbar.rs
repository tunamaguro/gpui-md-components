// 配置と色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingToolbar.kt
//         (`HorizontalFloatingToolbarLayout`、`VerticalFloatingToolbarLayout`、`FloatingToolbarDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のフローティングツールバー

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, Refineable, RenderOnce, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_headless::{
    Availability,
    toolbar::{Toolbar, ToolbarItem, ToolbarOrientation},
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::floating_toolbar;
use md3_theme::md3_tokens::supplement::app_bar as supplement;
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::IconButton;

/// フローティングツールバーの色の組み合わせ
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FloatingToolbarColor {
    /// 容器を SurfaceContainer、内容を OnSurface で塗る
    #[default]
    Standard,
    /// 容器を PrimaryContainer、内容を OnPrimaryContainer で塗り、目立たせる
    Vibrant,
}

/// フローティングツールバーの操作を並べる向き
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FloatingToolbarOrientation {
    /// 横に並べる。左右の矢印キーで操作の間を移る
    #[default]
    Horizontal,
    /// 縦に並べる。上下の矢印キーで操作の間を移る
    Vertical,
}

/// MD3 Expressive のフローティングツールバー
///
/// 内容の上に浮かべて置く、角の丸い容器に操作を並べる。容器は内容に合わせた大きさで、
/// 並べる向きの反対側の大きさは少なくとも 64dp。容器の内側に 8dp の余白を取る。
/// [`FloatingToolbar::action`] で置いたアイコンボタンは、少なくとも 48dp 四方の枠の中央に置く。押せる範囲はアイコンボタン自身の大きさのまま。
/// 子の要素は操作の間に、置いた順に並ぶ。子の要素には区切り線のようなフォーカスを持たない要素を渡すこと。
/// 影は描かない。画面の端から 16dp 離して置くことを勧める
///
/// 支援技術には role `toolbar` と向き、[`FloatingToolbar::new`] の `label` を名前として公開される。
/// [`FloatingToolbar::action`] の操作のうち Tab で到達できるのは 1 つだけで、
/// 修飾キーなしの矢印キー (横なら左右、縦なら上下) で前後の操作へフォーカスを移す。端では反対の端に戻り、無効な操作は飛ばす。
/// メニューを持つ操作でも、移れる操作がほかにあればこの矢印キーは移動に使い、メニューは Enter か Space で開く。
/// 子の要素はこの移動の対象にならない
#[derive(IntoElement)]
pub struct FloatingToolbar {
    id: ElementId,
    label: SharedString,
    color: FloatingToolbarColor,
    orientation: FloatingToolbarOrientation,
    children: Vec<Child>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl FloatingToolbar {
    /// 操作の無い、横のツールバーを作る
    ///
    /// `label` はツールバーが何の操作をまとめるかを表す名前にする。画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            color: FloatingToolbarColor::default(),
            orientation: FloatingToolbarOrientation::default(),
            children: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 色の組み合わせを設定する
    pub fn color(mut self, color: FloatingToolbarColor) -> Self {
        self.color = color;
        self
    }

    /// 操作を並べる向きを設定する
    pub fn orientation(mut self, orientation: FloatingToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `icon` を操作として、それまでに置いた操作と子の要素の後に置く
    ///
    /// 有効な `icon` は、[`IconButton::colors`] でアイコンの色を指定すれば、どのバリアントでもその色で描く。
    /// 指定しなければ、Standard と Outlined は [`FloatingToolbarColor`] の内容の色で、Filled と FilledTonal はバリアントの色で描く。
    /// 無効な `icon` は [`IconButton`] の無効時の色で描く。
    /// [`IconButton::theme`] でテーマを指定しなければ、ツールバーのテーマで描く。
    /// [`IconButton::track_focus`] を指定しなければ、フォーカスは `icon` の id とツールバーの id で決まる
    pub fn action(mut self, icon: IconButton) -> Self {
        self.children.push(Child::Icon(Box::new(icon)));
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    ///
    /// [`FloatingToolbar::action`] のアイコンボタンは、それぞれでテーマを指定していなければこのテーマで描く。
    /// 子として置いた要素には渡らないため、子の要素ごとに指定すること
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl RenderOnce for FloatingToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let (container, content) = match self.color {
            FloatingToolbarColor::Standard => (
                floating_toolbar::STANDARD_CONTAINER_COLOR,
                // Compose は容器の色に対応する内容の色 (`contentColorFor`) を使う
                ColorRole::OnSurface,
            ),
            FloatingToolbarColor::Vibrant => (
                floating_toolbar::VIBRANT_CONTAINER_COLOR,
                floating_toolbar::VIBRANT_BUTTON_UNSELECTED_ICON_COLOR,
            ),
        };
        let horizontal = self.orientation == FloatingToolbarOrientation::Horizontal;
        let mut toolbar = Toolbar::new(self.id.clone())
            .orientation(if horizontal {
                ToolbarOrientation::Horizontal
            } else {
                ToolbarOrientation::Vertical
            })
            .aria_label(self.label)
            .flex()
            .flex_none()
            .when(horizontal, |this| {
                this.flex_row()
                    .min_h(dp(floating_toolbar::CONTAINER_HEIGHT))
            })
            .when(!horizontal, |this| {
                this.flex_col()
                    .min_w(dp(floating_toolbar::CONTAINER_HEIGHT))
            })
            .items_center()
            .justify_center()
            .pl(dp(floating_toolbar::CONTAINER_LEADING_SPACE))
            .pt(dp(floating_toolbar::CONTAINER_LEADING_SPACE))
            .pr(dp(floating_toolbar::CONTAINER_TRAILING_SPACE))
            .pb(dp(floating_toolbar::CONTAINER_TRAILING_SPACE))
            // CornerFull は短辺の半分の半径で、テーマによらない
            .rounded_full()
            .bg(theme.color(container));
        for child in self.children {
            match child {
                Child::Icon(icon) => {
                    // ツールバーの外にある同じ id の要素と状態を分けるため、ツールバーの id の下で求める
                    let focus_handle = window.with_element_namespace(self.id.clone(), |window| {
                        icon.resolve_focus_handle(window, cx)
                    });
                    let availability = if icon.is_disabled() {
                        Availability::Disabled
                    } else {
                        Availability::Enabled
                    };
                    let icon = icon
                        .track_focus(&focus_handle)
                        .default_content(content)
                        .default_theme(theme.clone());
                    toolbar = toolbar.item(ToolbarItem::new(
                        focus_handle,
                        availability,
                        move |tab_stop| {
                            div()
                                .flex()
                                .flex_none()
                                .items_center()
                                .justify_center()
                                .min_w(dp(supplement::ICON_TARGET_SIZE))
                                .min_h(dp(supplement::ICON_TARGET_SIZE))
                                .child(icon.tab_stop(tab_stop))
                                .into_any_element()
                        },
                    ));
                }
                Child::Element(element) => toolbar = toolbar.child(element),
            }
        }
        toolbar.style().refine(&self.style);
        toolbar
    }
}

impl Styled for FloatingToolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for FloatingToolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(Child::Element));
    }
}

/// 置いた順に並べる子
enum Child {
    /// [`FloatingToolbar::action`] で置いたアイコンボタン
    Icon(Box<IconButton>),
    /// 子として置いた要素
    Element(AnyElement),
}

#[cfg(test)]
mod tests {
    use gpui::{Context, FocusHandle, Render, TestAppContext, VisualTestContext};

    use super::*;
    use crate::ButtonColors;

    /// 位置を測るため、容器を `role` の色で塗ったアイコンボタン
    fn button(id: &'static str, role: ColorRole) -> IconButton {
        IconButton::new(id, md3_icons::MORE_VERT, id).colors(ButtonColors {
            container: Some(role),
            ..ButtonColors::default()
        })
    }

    struct ToolbarView {
        toolbar: fn(&[FocusHandle; 3]) -> FloatingToolbar,
        /// ツールバーの操作に渡すフォーカス
        handles: [FocusHandle; 3],
        /// ツールバーの後に置いたボタンのフォーカス
        after: FocusHandle,
    }

    impl Render for ToolbarView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_start()
                .child((self.toolbar)(&self.handles))
                .child(
                    IconButton::new("after", md3_icons::MORE_VERT, "After")
                        .track_focus(&self.after),
                )
        }
    }

    fn open(
        cx: &mut TestAppContext,
        toolbar: fn(&[FocusHandle; 3]) -> FloatingToolbar,
    ) -> (gpui::Entity<ToolbarView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(move |window, cx| {
            cx.init_md3_theme(window);
            ToolbarView {
                toolbar,
                handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
                after: cx.focus_handle(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    /// `role` の色で塗った矩形の x・y・幅・高さ。左上から並べる
    fn filled(cx: &mut VisualTestContext, role: ColorRole) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let background: gpui::Background = cx.md3_theme().color(role).into();
            let mut rects: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| quad.background == background)
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect();
            rects.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));
            rects
        })
    }

    /// 期待値は Compose の `HorizontalFloatingToolbar` に 48dp の操作を 2 つ置いた大きさ。
    /// 容器は 8dp の余白と 48dp の枠で 112×64、アイコンボタンの容器 (40dp) は枠の中央にある
    #[gpui::test]
    fn 横は操作を_8dp_の余白の内側に並べる(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, |_| {
            FloatingToolbar::new("toolbar", "Edit")
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
        });
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainer),
            vec![(0.0, 0.0, 112.0, 64.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(12.0, 12.0, 40.0, 40.0), (60.0, 12.0, 40.0, 40.0)]
        );
    }

    #[gpui::test]
    fn 縦は操作を上から並べる(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, |_| {
            FloatingToolbar::new("toolbar", "Edit")
                .orientation(FloatingToolbarOrientation::Vertical)
                .action(button("a", ColorRole::Primary))
                .action(button("b", ColorRole::Primary))
        });
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainer),
            vec![(0.0, 0.0, 64.0, 112.0)]
        );
        assert_eq!(
            filled(cx, ColorRole::Primary),
            vec![(12.0, 12.0, 40.0, 40.0), (12.0, 60.0, 40.0, 40.0)]
        );
    }

    #[gpui::test]
    fn vibrant_は容器を_primary_container_で塗る(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, |_| {
            FloatingToolbar::new("toolbar", "Edit")
                .color(FloatingToolbarColor::Vibrant)
                .action(button("a", ColorRole::Primary))
        });
        assert_eq!(
            filled(cx, ColorRole::PrimaryContainer),
            vec![(0.0, 0.0, 64.0, 64.0)]
        );
    }

    fn three_actions(handles: &[FocusHandle; 3]) -> FloatingToolbar {
        FloatingToolbar::new("toolbar", "Edit")
            .action(IconButton::new("cut", md3_icons::MORE_VERT, "Cut").track_focus(&handles[0]))
            .action(
                IconButton::new("copy", md3_icons::MORE_VERT, "Copy")
                    .track_focus(&handles[1])
                    .disabled(true),
            )
            .action(
                IconButton::new("paste", md3_icons::MORE_VERT, "Paste").track_focus(&handles[2]),
            )
    }

    /// フォーカスのある要素の、ツールバーの中での位置。ツールバーの後のボタンなら `Some(3)`
    fn focused(view: &gpui::Entity<ToolbarView>, cx: &mut VisualTestContext) -> Option<usize> {
        let (handles, after) =
            view.read_with(cx, |view, _| (view.handles.clone(), view.after.clone()));
        cx.update(|window, _| {
            if after.is_focused(window) {
                return Some(3);
            }
            handles.iter().position(|handle| handle.is_focused(window))
        })
    }

    #[gpui::test]
    fn tab_で_1_つの操作に入り矢印キーで無効な操作を飛ばして移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = open(cx, three_actions);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some(0));
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(2), "無効な操作は飛ばすこと");
        cx.simulate_keystrokes("right");
        assert_eq!(focused(&view, cx), Some(0), "末尾の次は先頭に戻ること");
        cx.simulate_keystrokes("tab");
        assert_eq!(
            focused(&view, cx),
            Some(3),
            "ツールバーの中の Tab stop は 1 つだけであること"
        );
    }

    /// [`IconButton::track_focus`] を指定しなくても、矢印キーで移った先の操作がキーの入力を受ける
    #[gpui::test]
    fn track_focus_が無くても矢印キーで移った操作を押せる(
        cx: &mut TestAppContext,
    ) {
        struct ClickView {
            clicked: Vec<&'static str>,
        }
        impl Render for ClickView {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let record = |name: &'static str| {
                    cx.listener(move |this: &mut Self, _: &gpui::ClickEvent, _, _| {
                        this.clicked.push(name)
                    })
                };
                FloatingToolbar::new("toolbar", "Edit")
                    .action(
                        IconButton::new("cut", md3_icons::MORE_VERT, "Cut").on_click(record("cut")),
                    )
                    .action(
                        IconButton::new("copy", md3_icons::MORE_VERT, "Copy")
                            .on_click(record("copy")),
                    )
            }
        }
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            ClickView {
                clicked: Vec::new(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        cx.simulate_keystrokes("tab right");
        // フォーカスを移した後の描画でも、移った先の操作がフォーカスを保つこと
        view.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        // ボタンはキーを離したときに押される
        let enter = gpui::Keystroke::parse("enter").expect("キーを読めること");
        cx.simulate_event(gpui::KeyDownEvent {
            keystroke: enter.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(gpui::KeyUpEvent { keystroke: enter });
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |view, _| view.clicked.clone()),
            vec!["copy"]
        );
    }
}
