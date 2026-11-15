// 入力欄に置くアイコン、一覧の余白・色・書体・形は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt
//         (`ExposedDropdownMenuBox`、`ExposedDropdownMenu`、`ExposedDropdownMenuDefaults.TrailingIcon`)、
//         Menu.kt (`DropdownMenuContent`、`DropdownMenuItemContent`)
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2021, 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 選ばれている項目の容器の色と、内側に描く項目のフォーカスリングは Compose が扱わないため Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: select/internal/selectoption/select-option.ts、menu/internal/menuitem/menu-item.ts (`md-focus-ring` の `inward`)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある。入力欄の見た目は field.rs にある

//! MD3 のセレクト

use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Refineable, RenderOnce,
    SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder, size,
};
use gpui_headless::select::{
    Select as HeadlessSelect, SelectListbox, SelectOption as HeadlessSelectOption,
};
use md3_icons::StyledIcon;

use crate::field::{Field, FieldVariant};
use crate::shape::Shaped;
use crate::util;
use md3_theme::md3_tokens::component::{list, menu};
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, ThemeExt, dp};

/// 入力欄の層が入力欄の hover を知るための group 名
const GROUP: &str = "md3-select";
/// 項目の層が項目の hover を知るための group 名
const OPTION_GROUP: &str = "md3-select-option";

type ChangeHandler<T> = Box<dyn Fn(&T, &mut Window, &mut App)>;
type ExpandedChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;

/// セレクトの入力欄の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SelectVariant {
    /// 容器を塗り、下端に線を引く
    #[default]
    Filled,
    /// 容器を塗らず、枠線で囲む
    Outlined,
}

/// MD3 のセレクト
///
/// 読み取り専用の入力欄に選ばれている項目の名前を表示し、押すと項目の一覧を開く。
/// 選ばれている値と開いているかどうかは呼び出し側が持ち、[`Select::new`] に毎回渡す。
/// セレクト自身は状態を変えず、選択は [`Select::on_change`] で、開閉は [`Select::on_expanded_change`] で求める。
/// 項目は [`Select::option`] で加え、加えた順に一覧に並ぶ
///
/// 閉じている間
///
/// - クリック・Enter・Space・下矢印キーで開き、選ばれている項目 (無ければ最初の項目) にフォーカスを移す
/// - 上矢印キーと Home で開いて最初の項目に、End で開いて最後の項目にフォーカスを移す
///
/// 開いている間の一覧
///
/// - 上下の矢印キーで前後の項目へ移り、端では止まる。Home と End では最初と最後の項目へ、
///   PageUp と PageDown では 10 個前と後の項目へ移る。無効な項目は飛ばす
/// - 項目にマウスが乗ると、その項目にフォーカスを移す
/// - 項目のクリック・Enter・Space・Alt+上矢印キーで、その項目を選ぶよう求めて閉じ、フォーカスをセレクトへ戻す
/// - Tab と Shift+Tab で、フォーカスのある項目を選ぶよう求めて閉じ、セレクトの次と前の要素へフォーカスを移す
/// - Escape で選ばずに閉じ、フォーカスをセレクトへ戻す
/// - 一覧とセレクトの外でマウスのボタンを押すと閉じる
///
/// 入力欄は高さ 56dp、幅 280dp 以上で、[`Styled`] で指定した幅に広がる。
/// [`Select::label`] のラベルは、値が無く閉じていてフォーカスも無い間は入力欄の中に、それ以外は入力欄の上端に小さく表示する。
/// 一覧は入力欄の下に入力欄以上の幅で開き、ウィンドウの左右の端から 8dp、上下の端から 48dp の内側に収まらなければ上に移す。
/// ウィンドウより大きければ縦にスクロールする
///
/// 支援技術には role `combobox` として、開いているかと、選ばれている項目の名前を値として公開し、
/// 一覧は role `listbox`、項目は role `option` として公開する。
/// 名前はラベルか [`Select::aria_label`] で与えること。補足の文は説明として公開する
#[derive(IntoElement)]
pub struct Select<T: PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    expanded: bool,
    variant: SelectVariant,
    label: Option<SharedString>,
    aria_label: Option<SharedString>,
    leading_icon: Option<StyledIcon>,
    supporting_text: Option<SharedString>,
    error: bool,
    disabled: bool,
    on_change: Option<ChangeHandler<T>>,
    on_expanded_change: Option<ExpandedChangeHandler>,
    options: Vec<SelectOption<T>>,
    style: StyleRefinement,
    theme: Option<SharedTheme>,
}

impl<T: PartialEq + 'static> Select<T> {
    /// `selected` の値を持つ項目が選ばれ、`expanded` なら一覧を開いた状態で表示する
    /// [`SelectVariant::Filled`] のセレクトを作る。`selected` が `None` ならどの項目も選ばれていない
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, selected: Option<T>, expanded: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            expanded,
            variant: SelectVariant::default(),
            label: None,
            aria_label: None,
            leading_icon: None,
            supporting_text: None,
            error: false,
            disabled: false,
            on_change: None,
            on_expanded_change: None,
            options: Vec::new(),
            style: StyleRefinement::default(),
            theme: None,
        }
    }

    /// 入力欄の種類を `variant` にする
    pub fn variant(mut self, variant: SelectVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 入力欄のラベルを `label` にする。ラベルはアクセシブルな名前にもなる
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// ラベルの代わりに `label` をアクセシブルな名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 入力欄の先頭に `icon` を表示する
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// 入力欄の下に補足の文 `text` を表示する
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// `error` なら、入力欄をエラーの色で表示する
    ///
    /// エラーであることは支援技術に公開しない。理由は [`Select::supporting_text`] で伝えること
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// セレクトを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なセレクトは押されても開かず、Tab でも到達できず、支援技術には disabled として公開される
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 項目を選ぶよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には選ぶよう求められた項目の値を渡す。選ばれている項目と無効な項目では呼ばない。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと
    pub fn on_change(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// 開くか閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、開くよう求められたら `true` を、閉じるよう求められたら `false` を渡す。
    /// 項目を選ぶよう求めるときは、[`Select::on_change`] の後に呼ぶ。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Select::new`] に渡すこと。
    /// 設定しない場合、セレクトは押されても何も呼ばない
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Box::new(handler));
        self
    }

    /// 末尾に項目を加える
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// 末尾に `options` の項目を順に加える
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption<T>>) -> Self {
        self.options.extend(options);
        self
    }

    /// アプリ全体のテーマ ([`ThemeExt::md3_theme`]) の代わりに `theme` で描く
    pub fn theme(mut self, theme: SharedTheme) -> Self {
        self.theme = Some(theme);
        self
    }
}

impl<T: PartialEq + 'static> RenderOnce for Select<T> {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = self.theme.unwrap_or_else(|| cx.md3_theme().clone());
        let focus_handle = util::keyed_focus_handle(&self.id, window, cx);
        let value = self
            .options
            .iter()
            .find(|option| Some(&option.value) == self.selected.as_ref())
            .map(|option| option.label.clone());
        let field = Field {
            variant: match self.variant {
                SelectVariant::Filled => FieldVariant::Filled,
                SelectVariant::Outlined => FieldVariant::Outlined,
            },
            group: GROUP,
            label: self.label.clone(),
            leading_icon: self.leading_icon,
            // 開いている間は向きを反転する (`ExposedDropdownMenuDefaults.TrailingIcon`)
            trailing_icon: Some(md3_icons::ARROW_DROP_DOWN.into()),
            flip_trailing_icon: self.expanded,
            prefix: None,
            suffix: None,
            supporting_text: self.supporting_text.clone(),
            error: self.error,
            disabled: self.disabled,
            // 一覧を開いている間はフォーカスが一覧にあるが、Compose と同じく入力欄にフォーカスがあるものとして表示する
            focused: focus_handle.is_focused(window) || self.expanded,
            populated: value.is_some(),
            theme: theme.clone(),
        };

        let name = self.label.clone().or(self.aria_label.clone());
        let mut listbox = SelectListbox::new().window_margin(size(
            dp(supplement::menu::HORIZONTAL_MARGIN),
            dp(supplement::menu::VERTICAL_MARGIN),
        ));
        if let Some(name) = &name {
            listbox = listbox.aria_label(name.clone());
        }
        // 閉じている間も、選ばれている項目の名前をセレクトの値として公開するため、項目を渡す
        listbox = fill_listbox(
            listbox,
            &self.id,
            self.selected.as_ref(),
            std::mem::take(&mut self.options),
            &theme,
            window,
            cx,
        );
        let mut container = HeadlessSelect::new(self.id.clone(), self.selected, self.expanded)
            .listbox(listbox)
            .track_focus(&focus_handle.tab_index(0))
            .disabled(self.disabled)
            .group(GROUP);
        if let Some(name) = name {
            container = container.aria_label(name);
        }
        if let Some(on_change) = self.on_change {
            container = container.on_change(on_change);
        }
        if let Some(on_expanded_change) = self.on_expanded_change {
            container = container.on_expanded_change(on_expanded_change);
        }
        if let Some(text) = &self.supporting_text {
            container = container.aria_description(text.clone());
        }
        let value = value.map(|value| div().child(value).into_any_element());
        let mut outer = field.render(container, value, window);
        outer.style().refine(&self.style);
        outer
    }
}

impl<T: PartialEq + 'static> Styled for Select<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

/// `id` のセレクトの一覧 `listbox` に、容器を描き `options` を並べて返す
fn fill_listbox<T: PartialEq + 'static>(
    mut listbox: SelectListbox<T>,
    id: &ElementId,
    selected: Option<&T>,
    options: Vec<SelectOption<T>>,
    theme: &SharedTheme,
    window: &mut Window,
    cx: &mut App,
) -> SelectListbox<T> {
    let shadow = theme.shadow(menu::CONTAINER_ELEVATION);
    let container = div()
        .absolute()
        .inset_0()
        .bg(theme.color(menu::CONTAINER_COLOR))
        .shadow(shadow.clone());
    listbox = listbox
        .paint_margin(util::shadow_margin(&shadow))
        .relative()
        .flex()
        .flex_col()
        .py(dp(supplement::select::LISTBOX_VERTICAL_PADDING))
        // 一覧はセレクトの子として描かれ、入力欄の文字のスタイルを受け継ぐため、折り返しを戻す
        .whitespace_normal()
        .child(Shaped::new(container, theme.clone(), menu::CONTAINER_SHAPE));
    // Compose は一覧の項目の文を labelLarge で描く (`DropdownMenuItemContent`)
    listbox
        .text_style()
        .refine(&theme.text_style(TypeScale::LabelLarge));

    let mut selected_found = false;
    for option in options {
        // 同じ値の項目が複数あっても、支援技術に公開する状態と同じく最初の項目だけを選ばれた見た目にする
        let is_selected = !selected_found && selected == Some(&option.value);
        selected_found |= is_selected;
        let focus_handle = window.with_element_namespace(id.clone(), |window| {
            window.with_element_namespace("options", |window| {
                util::keyed_focus_handle(&option.id, window, cx)
            })
        });
        let focus_visible = gpui_headless::is_focus_visible(&focus_handle, window, cx);
        let disabled = option.disabled;
        let label_color = if disabled {
            theme
                .color(list::ITEM_DISABLED_LABEL_TEXT_COLOR)
                .opacity(list::ITEM_DISABLED_LABEL_TEXT_OPACITY)
        } else if is_selected {
            theme.color(menu::LIST_ITEM_SELECTED_LABEL_TEXT_COLOR)
        } else {
            theme.color(list::ITEM_LABEL_TEXT_COLOR)
        };
        let container = (is_selected && !disabled)
            .then(|| theme.color(menu::LIST_ITEM_SELECTED_CONTAINER_COLOR));
        // 状態レイヤーは文の色を重ねる (Compose の項目の ripple は内容の色を使う)
        let state_layer = |opacity: f32| -> Hsla {
            let layer = label_color.opacity(opacity);
            match container {
                Some(container) => container.blend(layer),
                None => layer,
            }
        };
        let layer = div()
            .absolute()
            .inset_0()
            .when_some(container, |this, container| this.bg(container))
            .when(!disabled, |this| {
                let this = this.when(focus_visible, |this| {
                    this.bg(state_layer(state::FOCUS_STATE_LAYER_OPACITY))
                });
                // gpui は group_hover を自身のスタイルの後に重ねるため、hover がフォーカスより優先される
                this.group_hover(OPTION_GROUP, |style| {
                    style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                })
            });
        // Material Web の項目はリングを内側に描く
        let ring = focus_visible.then(|| {
            div()
                .absolute()
                .inset(dp(focus_ring::INWARD_OFFSET))
                .border(dp(focus_ring::WIDTH))
                .border_color(theme.color(focus_ring::COLOR))
        });
        listbox = listbox.option(
            HeadlessSelectOption::new(option.id, option.value, option.label.clone())
                .disabled(disabled)
                .track_focus(&focus_handle)
                .group(OPTION_GROUP)
                .relative()
                .flex()
                .items_center()
                .min_h(dp(supplement::select::OPTION_MIN_HEIGHT))
                .px(dp(supplement::select::OPTION_HORIZONTAL_PADDING))
                .text_color(label_color)
                .child(layer)
                .child(div().relative().child(option.label))
                .children(ring),
        );
    }
    listbox
}

/// [`Select`] の一覧の項目
///
/// [`Select::option`] に渡して使う。項目が表す値が、セレクトの選ばれている値と等しいときに選ばれた状態になる
pub struct SelectOption<T> {
    id: ElementId,
    value: T,
    label: SharedString,
    disabled: bool,
}

impl<T> SelectOption<T> {
    /// `value` を表し、`label` を表示する項目を作る
    ///
    /// `label` は項目の名前と、選ばれたときに入力欄に表示する値になる。
    /// `id` と `value` は一覧の中で一意にし、`id` は描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は選べず、矢印キーでも到達できない
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, point, px};
    use md3_theme::md3_color::ColorRole;
    use md3_theme::md3_tokens::component::{
        filled_text_field as filled, outlined_text_field as outlined,
    };

    use super::*;

    const OPTIONS: [&str; 3] = ["Apple", "Banana", "Cherry"];

    /// (100, 100) に置いた幅 300 のセレクト。前に 10x10 のボタンを置く
    struct SelectView {
        selected: Option<usize>,
        expanded: bool,
        variant: SelectVariant,
        leading_icon: bool,
        error: bool,
        disabled: bool,
        label: &'static str,
        /// 指定した幅。`None` なら幅を指定しない
        width: Option<f32>,
    }

    impl Render for SelectView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Select::new("select", self.selected, self.expanded)
                    .variant(self.variant)
                    .label(self.label)
                    .when(self.leading_icon, |this| {
                        this.leading_icon(md3_icons::NUTRITION)
                    })
                    .error(self.error)
                    .disabled(self.disabled)
                    .on_change(cx.listener(|this, value: &usize, _, cx| {
                        this.selected = Some(*value);
                        cx.notify();
                    }))
                    .on_expanded_change(cx.listener(|this, expanded: &bool, _, cx| {
                        this.expanded = *expanded;
                        cx.notify();
                    }))
                    .options(
                        OPTIONS
                            .iter()
                            .enumerate()
                            .map(|(ix, name)| SelectOption::new(*name, ix, *name)),
                    )
                    .absolute()
                    .left(px(100.))
                    .top(px(100.))
                    .when_some(self.width, |this, width| this.w(px(width))),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut SelectView),
    ) -> (gpui::Entity<SelectView>, &mut VisualTestContext) {
        cx.update(gpui_headless::init);
        let (view, cx) = cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            let mut view = SelectView {
                selected: None,
                expanded: false,
                variant: SelectVariant::Filled,
                leading_icon: false,
                error: false,
                disabled: false,
                label: "Fruit",
                width: Some(300.),
            };
            f(&mut view);
            view
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_resize(size(px(800.), px(600.)));
        // ウィンドウをアクティブにすると最初のマウスの移動の後に位置が (0, 0) に戻るため、済むまで待つ
        cx.run_until_parked();
        (view, cx)
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 背景を `color` で塗った矩形の x, y, 幅, 高さ
    fn filled(cx: &mut VisualTestContext, color: Hsla) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == color.into())
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

    /// 矩形の x, y, 幅, 高さと、上・右・下・左の枠線の太さ
    type BorderedRect = ((f32, f32, f32, f32), [f32; 4]);

    /// 枠線を `color` で描いた矩形。
    /// gpui は枠線だけの矩形を空の内側を避けて分けて描くため、同じ矩形は 1 つにまとめる
    fn bordered(cx: &mut VisualTestContext, color: Hsla) -> Vec<BorderedRect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.border_color == color)
                .map(|quad| {
                    let b = quad.bounds;
                    let w = quad.border_widths;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        [w.top.0, w.right.0, w.bottom.0, w.left.0].map(|w| w / scale),
                    )
                })
                .fold(Vec::new(), |mut rects, rect| {
                    if !rects.contains(&rect) {
                        rects.push(rect);
                    }
                    rects
                })
        })
    }

    // gpui は debug_assertions のあるビルドでだけ debug_selector の位置を記録する
    #[cfg(debug_assertions)]
    fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Option<(f32, f32, f32, f32)> {
        cx.debug_bounds(selector).map(|b| {
            (
                b.origin.x.into(),
                b.origin.y.into(),
                b.size.width.into(),
                b.size.height.into(),
            )
        })
    }

    #[gpui::test]
    fn filled_の入力欄は高さ_56_で容器を塗り下端に線を引く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let container = color(cx, filled::CONTAINER_COLOR);
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
        let line = color(cx, filled::ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, line), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn 幅は_280_を下回らない(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.width = None);
        let container = color(cx, filled::CONTAINER_COLOR);
        assert_eq!(filled(cx, container), [(100., 100., 280., 56.)]);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 値が無く閉じている間はラベルを入力欄の中央に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |_| {});
        let (x, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        // bodyLarge の行の高さは 24
        assert_eq!((x, y, height), (116., 116., 24.));
        assert_eq!(bounds(cx, "md3-field-input"), None);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 値があればラベルを上端から_8_に浮かせその下に値を置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| view.selected = Some(1));
        let (x, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        // bodySmall の行の高さは 16
        assert_eq!((x, y, height), (116., 108., 16.));
        let (x, y, _, height) = bounds(cx, "md3-field-input").unwrap();
        assert_eq!((x, y, height), (116., 124., 24.));
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 先頭のアイコンがあれば文を_48_の箱と_4_の余白の後に置く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.selected = Some(1);
            view.leading_icon = true;
        });
        let (x, ..) = bounds(cx, "md3-field-input").unwrap();
        assert_eq!(x, 152.);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn 開くとラベルを浮かせ線を_primary_で太くし一覧を下に開く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx, |_| {});
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.expanded));
        let (_, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((y, height), (108., 16.));
        let primary = color(cx, filled::FOCUS_ACTIVE_INDICATOR_COLOR);
        assert!(filled(cx, primary).contains(&(100., 154., 300., 2.)));
        // 一覧の容器は入力欄の幅で、上下 8 の余白の中に高さ 48 の項目を 3 つ並べる
        let listbox = color(cx, menu::CONTAINER_COLOR);
        assert_eq!(filled(cx, listbox), [(100., 156., 300., 160.)]);
    }

    #[gpui::test]
    fn 選ばれている項目を_secondary_container_で塗る(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.selected = Some(1);
            view.expanded = true;
        });
        cx.run_until_parked();
        let selected = color(cx, menu::LIST_ITEM_SELECTED_CONTAINER_COLOR);
        assert_eq!(filled(cx, selected), [(100., 212., 300., 48.)]);
    }

    #[gpui::test]
    fn 項目を選ぶと値を表示する(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, |_| {});
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        // 3 つ目の項目は y が 260 から 308 の範囲になる
        cx.simulate_click(point(px(200.), px(284.)), Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| (view.selected, view.expanded)),
            (Some(2), false)
        );
    }

    #[gpui::test]
    fn hover_で線を_on_surface_にする(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |_| {});
        cx.simulate_mouse_move(point(px(200.), px(128.)), None, Modifiers::none());
        let hover = color(cx, filled::HOVER_ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, hover), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn エラーでは線とラベルを_error_の色にする(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.error = true);
        let error = color(cx, filled::ERROR_ACTIVE_INDICATOR_COLOR);
        assert_eq!(filled(cx, error), [(100., 155., 300., 1.)]);
    }

    #[gpui::test]
    fn 無効なセレクトは薄く描き開かない(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, |view| view.disabled = true);
        let container = cx.update(|_, cx| {
            cx.md3_theme()
                .color(filled::DISABLED_CONTAINER_COLOR)
                .opacity(filled::DISABLED_CONTAINER_OPACITY)
        });
        assert_eq!(filled(cx, container), [(100., 100., 300., 56.)]);
        cx.simulate_click(point(px(200.), px(128.)), Modifiers::none());
        assert!(!view.read_with(cx, |view, _| view.expanded));
    }

    #[gpui::test]
    fn outlined_は枠線を上の角と残りの_3_辺に分けて描く(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = SelectVariant::Outlined;
            view.selected = Some(0);
        });
        let outline = color(cx, outlined::OUTLINE_COLOR);
        let mut pieces = bordered(cx, outline);
        pieces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        // 入力欄は浮かせたラベルの半分の高さ 8 だけ下に置く。角の半径は 4
        assert_eq!(
            pieces,
            [
                ((100., 108., 4., 4.), [1., 0., 0., 1.]),
                ((100., 112., 300., 52.), [0., 1., 1., 1.]),
                ((396., 108., 4., 4.), [1., 1., 0., 0.]),
            ]
        );
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_は浮かせたラベルを上辺に置きその左右_4_までの線を空ける(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = SelectVariant::Outlined;
            view.selected = Some(0);
        });
        let (label_x, label_y, label_width, label_height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((label_x, label_y, label_height), (116., 100., 16.));
        let outline = color(cx, outlined::OUTLINE_COLOR);
        let mut segments = filled(cx, outline);
        segments.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let right = label_x + label_width.ceil() + 4.;
        assert_eq!(
            segments,
            [(104., 108., 8., 1.), (right, 108., 396. - right, 1.)]
        );
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_の浮かせたラベルが長ければ左右の余白の内側に収め角の手前の線を残す(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, |view| {
            view.variant = SelectVariant::Outlined;
            view.selected = Some(0);
            view.label = "A very long label that does not fit in the select field at all";
        });
        let (label_x, _, label_width, _) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((label_x, label_width), (116., 268.));
        let outline = color(cx, outlined::OUTLINE_COLOR);
        let mut segments = filled(cx, outline);
        segments.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(segments, [(104., 108., 8., 1.), (388., 108., 8., 1.)]);
    }

    #[gpui::test]
    #[cfg(debug_assertions)]
    fn outlined_の値が無く閉じている間は上辺を空けない(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, |view| view.variant = SelectVariant::Outlined);
        let outline = color(cx, outlined::OUTLINE_COLOR);
        assert_eq!(filled(cx, outline), [(104., 108., 292., 1.)]);
        let (_, y, _, height) = bounds(cx, "md3-field-label").unwrap();
        assert_eq!((y, height), (124., 24.));
    }
}
