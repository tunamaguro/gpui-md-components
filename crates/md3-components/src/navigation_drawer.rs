// 寸法・色・項目の中の配置は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/NavigationDrawer.kt
//         (`PermanentDrawerSheet`、`DrawerSheet`、`NavigationDrawerItem`、`NavigationDrawerItemDefaults`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
// Compose の項目はフォーカスを状態レイヤーだけで示す。この実装はそれに、ほかの部品と同じ外向きのフォーカスリングを
// インジケーターの形で加える。押している間の ripple は描かない
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 のナビゲーションドロワー

use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Refineable, RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas, div, prelude::FluentBuilder, svg,
};
use gpui_headless::tabs::{Tab as HeadlessTab, TabList, TabListOrientation};
use md3_icons::StyledIcon;
use md3_theme::md3_tokens::Dp;
use md3_theme::md3_tokens::component::navigation_drawer as tokens;
use md3_theme::md3_tokens::{focus_ring, state};
use md3_theme::{SharedTheme, ThemeExt, dp};

use crate::shape::Shaped;
use crate::util;

/// 項目の中の層が項目の hover を知るための group 名
const GROUP: &str = "md3-navigation-drawer-item";

/// シートの左右の端と項目の間の余白 (`NavigationDrawerItemDefaults.ItemPadding`)
const ITEM_HORIZONTAL_PADDING: Dp = Dp(12.0);
/// 項目の先頭側の余白
const ITEM_LEADING_PADDING: Dp = Dp(16.0);
/// 項目の末尾側の余白
const ITEM_TRAILING_PADDING: Dp = Dp(24.0);
/// アイコンとラベルの間、ラベルとバッジの間
const ITEM_CONTENT_SPACE: Dp = Dp(12.0);

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;

/// MD3 のナビゲーションドロワー (Compose の `PermanentDrawerSheet` に `NavigationDrawerItem` を並べたもの)
///
/// 行き先の項目を縦に並べたシートを、画面の端に常に表示しておく。シートは 360dp の幅で、親がそれより狭ければ親の幅に縮む。
/// 高さは親の高さいっぱいに広がる。項目はシートの左右に 12dp の余白を取った幅で、高さは少なくとも 56dp。
/// 選ばれている項目は角の丸いインジケーターで示す
///
/// 状態 (選ばれている項目の値) は呼び出し側が持ち、毎回渡す。ドロワー自身は状態を変えない
///
/// - Tab で到達できる項目は 1 つだけで、選ばれている項目、最初のフォーカスできる項目の順に選ぶ
/// - 上下の矢印キーで隣の項目へフォーカスを移し、移った先の項目を選ぶよう求める。端では反対の端に戻る。
///   Home と End では最初と最後の項目へ移り、移った先を選ぶよう求める
/// - フォーカス中の Enter と Space・マウスのクリックで、項目を選ぶよう求める
///
/// 矢印キーでも選ぶよう求めるため、行き先の内容は [`NavigationDrawer::on_change`] を受けた次の描画で遅れなく差し替えること。
/// ドロワーは行き先の内容 (role `tabpanel`) を公開しない。選ばれた項目と、表示している行き先の内容を一致させるのは呼び出し側の責務になる
///
/// 支援技術には項目の並びが role `tablist` と縦向きとして公開され、[`NavigationDrawer::aria_label`] がアクセシブルな名前になる。
/// 項目は role `tab` と選ばれているかどうかの状態として公開され、ラベルが名前になる。
/// ヘッダーは並びの外に置き、並びの前の兄弟として公開される。
/// 項目がシートの高さに収まらないときは並びが縦にスクロールし、フォーカスのある項目を表示範囲に入れる
///
/// [`Styled`] で指定したスタイルはシートの箱に付く
#[derive(IntoElement)]
pub struct NavigationDrawer<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    items: Vec<NavigationDrawerItem<T>>,
    on_change: Option<ChangeHandler<T>>,
    header: Option<AnyElement>,
    aria_label: Option<SharedString>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> NavigationDrawer<T> {
    /// `selected` の値を持つ項目を選ばれた状態で表示するドロワーを作る。`None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>) -> Self {
        Self {
            id: id.into(),
            selected,
            items: Vec::new(),
            on_change: None,
            header: None,
            aria_label: None,
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 項目の上に `header` を置く。アプリの名前やドロワーの見出しを置くときに使う
    ///
    /// ヘッダーはシートの幅でシートの上端に置き、項目の並びがスクロールしても動かない
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// `label` を項目の並びのアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`NavigationDrawer::new`] に渡すこと。
    /// 設定しない場合、項目は操作されても何も呼ばない
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: NavigationDrawerItem<T>) -> Self {
        self.items.push(item);
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = NavigationDrawerItem<T>>) -> Self {
        self.items.extend(items);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for NavigationDrawer<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let label_font = theme.text_style(tokens::LABEL_TEXT_FONT);
        let badge_font = theme.text_style(tokens::LARGE_BADGE_LABEL_FONT);
        let icon_size = dp(tokens::ICON_SIZE);
        let shape = tokens::ACTIVE_INDICATOR_SHAPE;
        let ring_outset = dp(focus_ring::OUTWARD_OFFSET) + dp(focus_ring::WIDTH);

        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let mut selected = self.selected.as_ref();
        let mut focused = None;
        let items: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = selected
                    .take_if(|selected| **selected == item.value)
                    .is_some();
                let focus_handle = item.focus_handle.clone().unwrap_or_else(|| {
                    // 項目の id はドロワーの中でだけ一意なので、ドロワーの id で分ける
                    window.with_element_namespace(self.id.clone(), |window| {
                        util::keyed_focus_handle(&item.id, window, cx)
                    })
                });
                if focus_handle.is_focused(window) {
                    focused = Some(index);
                }
                let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);

                // 状態レイヤーは内容の色で描く (`Surface` の ripple)
                let (icon_color, label_color, state_color) = if is_selected {
                    (
                        theme.color(tokens::ACTIVE_ICON_COLOR),
                        theme.color(tokens::ACTIVE_LABEL_TEXT_COLOR),
                        theme.color(tokens::ACTIVE_HOVER_LABEL_TEXT_COLOR),
                    )
                } else {
                    (
                        theme.color(tokens::INACTIVE_ICON_COLOR),
                        theme.color(tokens::INACTIVE_LABEL_TEXT_COLOR),
                        theme.color(tokens::INACTIVE_HOVER_LABEL_TEXT_COLOR),
                    )
                };
                let indicator = is_selected.then(|| {
                    Shaped::new(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .right_0()
                            .bg(theme.color(tokens::ACTIVE_INDICATOR_COLOR)),
                        theme.clone(),
                        shape,
                    )
                });
                let state_layer = Shaped::new(
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
                );
                let ring = focus_visible.then(|| {
                    Shaped::new(
                        div()
                            .absolute()
                            .top(-ring_outset)
                            .left(-ring_outset)
                            .right(-ring_outset)
                            .bottom(-ring_outset)
                            .border(dp(focus_ring::WIDTH))
                            .border_color(theme.color(focus_ring::COLOR)),
                        theme.clone(),
                        shape,
                    )
                    .outset(ring_outset)
                });
                let icon = item.icon.map(|icon| {
                    svg()
                        .data(icon.svg().as_bytes())
                        .flex_none()
                        .size(icon_size)
                        .mr(dp(ITEM_CONTENT_SPACE))
                        .text_color(icon_color)
                });
                let mut label = div().flex_1().min_w_0().text_color(label_color);
                label.text_style().refine(&label_font);
                let label = label.child(item.label.clone());
                let badge = item.badge.clone().map(|badge| {
                    let mut element = div()
                        .flex_none()
                        .ml(dp(ITEM_CONTENT_SPACE))
                        .text_color(label_color);
                    element.text_style().refine(&badge_font);
                    element.child(badge)
                });

                // Compose の項目は子の意味をまとめて公開するため、バッジも名前に含まれる
                let name = match &item.badge {
                    Some(badge) => SharedString::from(format!("{} {}", item.label, badge)),
                    None => item.label.clone(),
                };
                HeadlessTab::new(item.id, item.value)
                    .track_focus(&focus_handle)
                    .aria_label(name)
                    .group(GROUP)
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .w_full()
                    .min_h(dp(tokens::ACTIVE_INDICATOR_HEIGHT))
                    .pl(dp(ITEM_LEADING_PADDING))
                    .pr(dp(ITEM_TRAILING_PADDING))
                    .children(indicator)
                    .child(state_layer)
                    .children(icon)
                    .child(label)
                    .children(badge)
                    .children(ring)
            })
            .collect();

        let header = self.header.map(|header| div().flex_none().child(header));

        let (scroll_handle, last_focused) =
            window.with_element_namespace(self.id.clone(), |window| {
                (
                    window.use_keyed_state("scroll", cx, |_, _| ScrollHandle::new()),
                    window.use_keyed_state("last-focused", cx, |_, _| {
                        util::FocusedPlacement::default()
                    }),
                )
            });
        let scroll_handle = scroll_handle.read(cx).clone();
        // フォーカスリングはインジケーターの外に張り出すため、スクロール領域の上下にその分の余白を取る。
        // シートの上端に接していても先頭の項目のリングが切れないよう、上の余白は打ち消さない。左右はシートの余白の中に収まる
        let list = TabList::new(self.id.clone(), self.selected)
            .orientation(TabListOrientation::Vertical)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.on_change, |this, on_change| this.on_change(on_change))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .py(ring_outset)
            .px(dp(ITEM_HORIZONTAL_PADDING))
            .overflow_y_scroll()
            .track_scroll(&scroll_handle)
            .items(items);
        // キーボードでフォーカスが移った項目を、リングの張り出しも含めて表示範囲に入れる。
        // 並びの後に置いた canvas の prepaint で、この描画のレイアウトで決まった範囲を使う
        let measure = canvas(
            move |_, window, cx| {
                util::scroll_focused_into_view(
                    &scroll_handle,
                    &last_focused,
                    focused,
                    ring_outset,
                    window,
                    cx,
                );
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();

        // Compose は 240dp (`MinimumDrawerWidth`) から 360dp の幅を親の幅の中に収めて取り、
        // 項目が最大の幅に広がる (`fillMaxWidth`) ため、360dp と親の幅の狭い方になる
        let mut sheet = div()
            .id((self.id, "sheet"))
            .relative()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(dp(tokens::CONTAINER_WIDTH))
            .max_w_full()
            .h_full()
            // 角丸は付けない (`PermanentDrawerSheet` の `RectangleShape`)
            .bg(theme.color(tokens::STANDARD_CONTAINER_COLOR))
            .children(header)
            .child(list)
            .child(measure);
        sheet.style().refine(&self.style);
        sheet
    }
}

impl<T: PartialEq + 'static> Styled for NavigationDrawer<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// ナビゲーションドロワーの項目
///
/// 支援技術には role `tab` として公開され、ラベルとバッジをこの順に並べた文字列がアクセシブルな名前になる
pub struct NavigationDrawerItem<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    icon: Option<StyledIcon>,
    badge: Option<SharedString>,
    focus_handle: Option<FocusHandle>,
}

impl<T> NavigationDrawerItem<T> {
    /// `label` を表示し、選ばれると `value` を [`NavigationDrawer::on_change`] に渡す項目を作る
    ///
    /// `id` と `value` はドロワーの中で一意にし、`id` は描画のたびに同じ値を渡すこと。
    /// 同じ値の項目が複数あると、そのうち最初の項目だけが選ばれた状態になる。
    /// [`NavigationDrawerItem::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            icon: None,
            badge: None,
            focus_handle: None,
        }
    }

    /// ラベルの前に `icon` を描く
    pub fn icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// ラベルの後ろの末尾側に `badge` を描く。未読の数などを示すときに使う
    ///
    /// `badge` はラベルの後ろに続けてアクセシブルな名前に含める
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// Tab で到達できるかはドロワーが決める。指定しない場合は項目が内部でフォーカスを持つ
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
    use md3_icons::{HOME, INBOX, SEND};
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

    fn drawer() -> NavigationDrawer<usize> {
        NavigationDrawer::new("drawer", Some(1))
            .aria_label("Mail")
            .items([
                NavigationDrawerItem::new("inbox", 0, "Inbox")
                    .icon(INBOX)
                    .badge("24"),
                NavigationDrawerItem::new("sent", 1, "Sent").icon(SEND),
                NavigationDrawerItem::new("home", 2, "Home").icon(HOME),
            ])
    }

    /// 期待値は Compose の `PermanentDrawerSheet` と `NavigationDrawerItem`。シートは 360dp の幅で親の高さに広がり、
    /// 項目は左右 12dp の余白の内側に 56dp ごとに並ぶ。選ばれた 2 つ目の項目にインジケーターを描く。
    /// 項目はフォーカスリングの張り出し (5dp) だけ上端から離れる
    #[gpui::test]
    fn シートは_360dp_の幅で項目を_56dp_ごとに並べる(cx: &mut TestAppContext) {
        let cx = open(cx, || drawer().into_any_element());
        assert_eq!(
            quads_of(cx, tokens::STANDARD_CONTAINER_COLOR),
            vec![(0.0, 0.0, 360.0, 600.0)]
        );
        assert_eq!(
            quads_of(cx, tokens::ACTIVE_INDICATOR_COLOR),
            vec![(12.0, 5.0 + 56.0, 336.0, 56.0)]
        );
    }

    #[gpui::test]
    fn 親が狭ければ親の幅に縮む(cx: &mut TestAppContext) {
        let cx = open(cx, || {
            div()
                .w(px(300.0))
                .h_full()
                .child(drawer())
                .into_any_element()
        });
        assert_eq!(
            quads_of(cx, tokens::STANDARD_CONTAINER_COLOR),
            vec![(0.0, 0.0, 300.0, 600.0)]
        );
    }

    #[gpui::test]
    fn ヘッダーの下から項目を並べる(cx: &mut TestAppContext) {
        let cx = open(cx, || drawer().header(div().h(px(64.0))).into_any_element());
        assert_eq!(
            quads_of(cx, tokens::ACTIVE_INDICATOR_COLOR),
            vec![(12.0, 64.0 + 5.0 + 56.0, 336.0, 56.0)]
        );
    }

    #[gpui::test]
    fn クリックと矢印キーで項目を選ぶよう求める(cx: &mut TestAppContext) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let recorder = changes.clone();
        let cx = open(cx, move || {
            let recorder = recorder.clone();
            drawer()
                .on_change(move |value, _, _| recorder.borrow_mut().push(*value))
                .into_any_element()
        });
        // 1 つ目の項目の中央
        cx.simulate_click(point(px(100.0), px(5.0 + 28.0)), Modifiers::none());
        assert_eq!(*changes.borrow(), vec![0]);
        cx.simulate_keystrokes("up");
        assert_eq!(
            *changes.borrow(),
            vec![0, 2],
            "先頭の前は末尾に戻り、移った先の項目を選ぶよう求める"
        );
    }

    /// フォーカスリングが、切り抜かれる範囲 (スクロール領域の表示範囲) に収まっているか
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
            NavigationDrawer::new("drawer", Some(0))
                .items(
                    (0..12usize)
                        .map(|index| NavigationDrawerItem::new(("item", index), index, "Item")),
                )
                .into_any_element()
        });
        cx.update(|window, _| window.activate_window());
        // スクロールしなければ最後の項目は上端から 5 + 56 × 11 = 621dp にあり、高さ 600dp の表示範囲の外になる
        cx.simulate_keystrokes("tab end");
        // 描いたレイアウトでスクロールを補正し、補正したオフセットで次のフレームを描く
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            ring_fits(cx),
            "End で移った最後の項目のリングが切れないこと"
        );
        cx.simulate_keystrokes("home");
        cx.update(|window, cx| window.simulate_next_frame(cx));
        cx.run_until_parked();
        assert!(
            ring_fits(cx),
            "Home で移った最初の項目のリングが切れないこと"
        );
    }
}
