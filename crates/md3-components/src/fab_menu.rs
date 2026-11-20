// 配置・余白・色は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButtonMenu.kt
//         (`FloatingActionButtonMenu`、`FloatingActionButtonMenuItem`、`ToggleFloatingActionButton`、
//         `ToggleFloatingActionButtonDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive の FAB メニュー

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, Refineable,
    RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::disclosure::DisclosurePanel;
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::Dp;
use md3_theme::md3_tokens::component::fab_menu_baseline as tokens;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::fab::{self, Fab, FabColor};
use crate::util;

/// メニューの左右と、開閉ボタンの下の余白 (Compose の `FabMenuPaddingHorizontal` と `FabMenuButtonPaddingBottom`)
const PADDING: Dp = Dp(16.0);

type ToggleHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// MD3 Expressive の FAB メニュー (Compose の `FloatingActionButtonMenu` と `ToggleFloatingActionButton`)
///
/// 開閉ボタン (FAB) の上に、開いている間だけ項目を右端に揃えて縦に並べる。
/// 開閉ボタンは、閉じている間は PrimaryContainer の角丸の FAB にアイコンを描き、
/// 開いている間は Primary の丸い FAB に閉じるアイコンを描く。項目は PrimaryContainer の丸い容器にアイコンとラベルを並べる。
/// メニューは左右と下に 16 dp の余白を持ち、置いた場所の右下に置くことを想定する。項目の出入りと開閉ボタンの変形は動かさずに切り替える。
/// メニューの高さは親の高さまでで、項目が収まらない場合は項目の列が縦にスクロールし、開閉ボタンは常に表示する。
/// フォーカスが表示範囲の外の項目に移っても、項目の列はスクロールしない。
/// 支援技術から表示範囲の外の項目を押すと、項目の位置にある別の要素 (開閉ボタンなど) が押されることがある。
/// 開いているかどうかは呼び出し側が持ち、[`FabMenu::new`] に毎回渡す。
/// 開閉は [`FabMenu::on_toggle`] で求め、項目を押しても閉じないため、閉じるには項目のハンドラで状態を更新すること
///
/// - 開閉ボタンのマウスのクリック・フォーカス中の Enter と Space で、開閉を求める
/// - Tab の順は、開閉ボタン、上の項目から下の項目の順になる
/// - 項目にフォーカスがあるまま閉じると、開閉ボタンへフォーカスを移す。項目が消えてもフォーカスを失わない
///
/// 支援技術には、開閉ボタンが [`FabMenu::new`] の `label` を名前とする role `button` と、開いているかの状態 (expanded) として公開される。
/// 項目は、ラベルを名前とする role `button` として公開される。
/// [`Styled`] で指定したスタイルはメニューの外側の箱に付く
#[derive(IntoElement)]
pub struct FabMenu {
    id: ElementId,
    expanded: bool,
    icon: StyledIcon,
    label: SharedString,
    items: Vec<FabMenuItem>,
    on_toggle: Option<ToggleHandler>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl FabMenu {
    /// `expanded` なら項目を開いた状態で表示する FAB メニューを作る
    ///
    /// `icon` は閉じている間の開閉ボタンのアイコン、`label` は開閉ボタンのアクセシブルな名前にする。`label` は画面には表示しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(
        id: impl Into<ElementId>,
        expanded: bool,
        icon: impl Into<StyledIcon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            expanded,
            icon: icon.into(),
            label: label.into(),
            items: Vec::new(),
            on_toggle: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 開閉ボタンが押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、求められた開いているかの値 (今の値の反対) を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`FabMenu::new`] に渡すこと
    pub fn on_toggle(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// 末尾 (開閉ボタンに近い側) に項目を加える
    pub fn item(mut self, item: FabMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl Styled for FabMenu {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let toggle_id = ElementId::from("toggle");
        // 項目の id はメニューの中でだけ一意なので、フォーカスはメニューの id で分ける
        let toggle_focus = window.with_element_namespace(self.id.clone(), |window| {
            util::keyed_focus_handle(&toggle_id, window, cx)
        });
        let expanded = self.expanded;
        let toggle = Fab::new(
            toggle_id,
            if expanded {
                StyledIcon::from(md3_icons::CLOSE)
            } else {
                self.icon
            },
            self.label,
        )
        .color(if expanded {
            FabColor::Primary
        } else {
            FabColor::PrimaryContainer
        })
        .menu_toggle(expanded)
        .track_focus(&toggle_focus)
        .theme(theme.clone())
        .when_some(self.on_toggle, |this, on_toggle| {
            this.on_click(move |_, window, cx| on_toggle(&!expanded, window, cx))
        });

        let mut label_font = theme.text_style(TypeScale::TitleMedium);
        label_font.color = Some(theme.color(FabColor::PrimaryContainer.roles().1));
        let items = expanded.then(|| {
            div()
                .flex()
                .flex_col()
                .flex_none()
                .items_end()
                .gap(dp(tokens::LIST_ITEM_BETWEEN_SPACE))
                // 開閉ボタンとの間隔。閉じている間は空になる箱なので、flex の gap ではなく項目の列に付ける
                .pb(dp(tokens::CLOSE_BUTTON_BETWEEN_SPACE))
                .children(self.items.into_iter().map(|item| {
                    // 描画する要素と同じく、項目のフォーカスも項目の列 (`items`) の下に置き、開閉ボタンと分ける
                    let focus_handle = window.with_element_namespace(self.id.clone(), |window| {
                        window.with_element_namespace("items", |window| {
                            util::keyed_focus_handle(&item.id, window, cx)
                        })
                    });
                    let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
                    let mut label = div();
                    label.text_style().refine(&label_font);
                    HeadlessButton::new(item.id)
                        .track_focus(&focus_handle)
                        .group(fab::GROUP)
                        .relative()
                        .flex()
                        .items_center()
                        .gap(dp(tokens::LIST_ITEM_ICON_LABEL_SPACE))
                        .min_w(dp(tokens::LIST_ITEM_CONTAINER_HEIGHT))
                        .h(dp(tokens::LIST_ITEM_CONTAINER_HEIGHT))
                        .pl(dp(tokens::LIST_ITEM_LEADING_SPACE))
                        .pr(dp(tokens::LIST_ITEM_TRAILING_SPACE))
                        .child(fab::container_layer(
                            &theme,
                            FabColor::PrimaryContainer,
                            tokens::LIST_ITEM_CONTAINER_SHAPE,
                            focus_visible,
                            true,
                        ))
                        .child(
                            svg()
                                .data(item.icon.svg().as_bytes())
                                .relative()
                                .flex_none()
                                .size(dp(tokens::LIST_ITEM_ICON_SIZE))
                                .text_color(theme.color(FabColor::PrimaryContainer.roles().1)),
                        )
                        .child(label.relative().child(item.label.clone()))
                        .when(focus_visible, |this| {
                            this.child(fab::focus_ring(&theme, tokens::LIST_ITEM_CONTAINER_SHAPE))
                        })
                        .aria_label(item.label)
                        .when_some(item.on_click, |this, on_click| this.on_click(on_click))
                }))
        });

        let panel = DisclosurePanel::new("items", expanded, &toggle_focus)
            .flex()
            .flex_col()
            .items_end()
            .min_h_0()
            .overflow_y_scroll()
            .children(items);
        // 開閉ボタンを先に置き、Tab の順を開閉ボタン、上の項目の順にする。見た目は逆順に並べて開閉ボタンを下にする
        let mut menu = div()
            .id(self.id)
            .flex()
            .flex_col_reverse()
            .items_end()
            .max_h_full()
            .px(dp(PADDING))
            .pb(dp(PADDING))
            .child(div().flex_none().child(toggle))
            .child(panel);
        menu.style().refine(&self.style);
        menu
    }
}

/// [`FabMenu`] の項目
///
/// 支援技術には role `button` として公開され、ラベルがアクセシブルな名前になる
pub struct FabMenuItem {
    id: ElementId,
    icon: StyledIcon,
    label: SharedString,
    on_click: Option<ClickHandler>,
}

impl FabMenuItem {
    /// `icon` と `label` を並べた項目を作る
    ///
    /// `id` はメニューの中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(
        id: impl Into<ElementId>,
        icon: impl Into<StyledIcon>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: label.into(),
            on_click: None,
        }
    }

    /// 押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{
        Context, KeyDownEvent, KeyUpEvent, Keystroke, Render, TestAppContext, VisualTestContext,
    };

    use super::*;

    /// 開いているかと、押された項目を持ち、右下に FAB メニューを置くビュー
    struct MenuView {
        expanded: bool,
        clicked: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Render for MenuView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let item = |id: &'static str, clicked: Rc<RefCell<Vec<&'static str>>>| {
                FabMenuItem::new(id, md3_icons::EDIT, id).on_click(cx.listener(
                    move |this, _, _, cx| {
                        clicked.borrow_mut().push(id);
                        this.expanded = false;
                        cx.notify();
                    },
                ))
            };
            div().size_full().flex().justify_end().items_end().child(
                FabMenu::new("menu", self.expanded, md3_icons::ADD, "Create")
                    .on_toggle(cx.listener(|this, expanded: &bool, _, cx| {
                        this.expanded = *expanded;
                        cx.notify();
                    }))
                    .item(item("first", self.clicked.clone()))
                    .item(item("second", self.clicked.clone())),
            )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (gpui::Entity<MenuView>, &mut VisualTestContext) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            MenuView {
                expanded: false,
                clicked: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        (view, cx)
    }

    fn press(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        cx.run_until_parked();
    }

    /// 項目の容器の色で塗った矩形 (x, y, 幅, 高さ)
    fn item_quads(cx: &mut VisualTestContext) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, cx| {
            let scale = window.scale_factor();
            let color: gpui::Background = cx
                .md3_theme()
                .color(FabColor::PrimaryContainer.roles().0)
                .into();
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

    #[gpui::test]
    fn 閉じている間は開閉ボタンだけを右下に描き_開くと項目をその上に右端を揃えて並べる(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx);
        let size = cx.update(|window, _| window.viewport_size());
        let (width, height) = (f32::from(size.width), f32::from(size.height));
        assert_eq!(
            item_quads(cx),
            vec![(width - 16.0 - 56.0, height - 16.0 - 56.0, 56.0, 56.0)],
            "閉じている間は PrimaryContainer の開閉ボタンだけを描くこと"
        );

        press(cx, "tab");
        press(cx, "enter");
        let quads = item_quads(cx);
        assert_eq!(
            quads.len(),
            2,
            "開くと開閉ボタンは Primary になり、項目を 2 つ描くこと"
        );
        let toggle_top = height - 16.0 - 56.0;
        for (index, (x, y, w, h)) in quads.iter().enumerate() {
            assert_eq!(x + w, width - 16.0, "項目の右端を揃えること");
            assert_eq!(*h, 56.0);
            let bottom = toggle_top - 8.0 - (1 - index) as f32 * (56.0 + 4.0);
            assert_eq!(y + h, bottom, "{index} 番目の項目の下端");
        }
    }

    #[gpui::test]
    fn tab_は開閉ボタンから上の項目へ進み_項目を押すとフォーカスを開閉ボタンに戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx);
        press(cx, "tab");
        press(cx, "enter");
        assert!(view.read_with(cx, |view, _| view.expanded));
        press(cx, "tab");
        press(cx, "enter");
        assert_eq!(
            *view.read_with(cx, |view, _| view.clicked.clone()).borrow(),
            vec!["first"],
            "開閉ボタンの次は上の項目に移ること"
        );
        assert!(!view.read_with(cx, |view, _| view.expanded));
        press(cx, "enter");
        assert!(
            view.read_with(cx, |view, _| view.expanded),
            "フォーカスが開閉ボタンに戻り、Enter で再び開くこと"
        );
    }

    /// 同じ項目の id を使う 2 つのメニューを並べ、押された (メニュー, 項目) を持つビュー。項目を押しても閉じない
    struct TwoMenusView {
        clicked: Rc<RefCell<Vec<(&'static str, &'static str)>>>,
    }

    impl Render for TwoMenusView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let menu =
                |menu: &'static str, clicked: Rc<RefCell<Vec<(&'static str, &'static str)>>>| {
                    FabMenu::new(menu, true, md3_icons::ADD, menu).item(
                        // 開閉ボタンの内部の id と同じ "toggle" を使っても衝突しないこと
                        FabMenuItem::new("toggle", md3_icons::EDIT, "Edit")
                            .on_click(move |_, _, _| clicked.borrow_mut().push((menu, "edit"))),
                    )
                };
            div()
                .flex()
                .child(menu("left", self.clicked.clone()))
                .child(menu("right", self.clicked.clone()))
        }
    }

    #[gpui::test]
    fn 同じ項目の_id_を使う_2_つのメニューの項目は別にフォーカスされ_開いたままなら項目にフォーカスが残る(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            TwoMenusView {
                clicked: Rc::default(),
            }
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        // Tab の順は左の開閉ボタン、左の項目、右の開閉ボタン、右の項目
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "enter");
        press(cx, "enter");
        press(cx, "tab");
        press(cx, "tab");
        press(cx, "enter");
        assert_eq!(
            *view.read_with(cx, |view, _| view.clicked.clone()).borrow(),
            vec![("left", "edit"), ("left", "edit"), ("right", "edit")]
        );
    }
}
