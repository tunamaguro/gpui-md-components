// 寸法・色・形の選択は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt
//         (`DropdownMenuPopup`、`DropdownMenuPopupContent`、`DropdownMenuGroup`、`DropdownMenuItem`、
//         `CheckableDropdownMenuItem`、`SelectableDropdownMenuItem`、`DropdownMenuItemContent`、
//         `LabelWithSupportingText`、`shapeByInteraction`、`MenuArrangement`)
//         compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MenuDefaults.kt
//         (`MenuDefaults`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2022, 2025 The Android Open Source Project. Licensed under the Apache License, Version 2.0.
//
// 項目のフォーカスリングを項目の内側に描くことは Material Web から移植している
// (https://github.com/material-components/material-web)
// Source: menu/internal/menuitem/menu-item.ts (`md-focus-ring` の `inward`)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.
// トークンの値と版は md3-tokens の各ファイルにある

//! MD3 Expressive のメニュー

use gpui::{
    App, ClickEvent, Div, Edges, ElementId, FocusHandle, Hsla, InteractiveElement, ParentElement,
    Pixels, Refineable, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, size, svg,
};
use gpui_headless::button::Button as HeadlessButton;
use gpui_headless::choice::CheckedState;
use gpui_headless::menu::{
    Menu as HeadlessMenu, MenuGroup as HeadlessMenuGroup, MenuItem as HeadlessMenuItem,
    MenuItemKind as HeadlessMenuItemKind,
};

use crate::Divider;
use crate::shape::Shaped;
use crate::util;
use md3_icons::StyledIcon;
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{menu, segmented_menu, standard_menu, vibrant_menu};
use md3_theme::md3_tokens::shape::ShapeKey;
use md3_theme::md3_tokens::typography::TypeScale;
use md3_theme::md3_tokens::{focus_ring, state, supplement};
use md3_theme::{SharedTheme, dp};

/// 項目の層が項目の hover を知るための group 名
const GROUP: &str = "md3-menu-item";

type ExpandedChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App)>;
type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// メニューの配色
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuVariant {
    /// 面の色の組に描く
    #[default]
    Standard,
    /// tertiary の容器の色の組に描き、メニューを目立たせる
    Vibrant,
}

/// 配色ごとの色
struct Colors {
    group: ColorRole,
    label: ColorRole,
    leading_icon: ColorRole,
    trailing: ColorRole,
    disabled_label: (ColorRole, f32),
    disabled_leading_icon: (ColorRole, f32),
    disabled_trailing: (ColorRole, f32),
    selected_container: ColorRole,
    selected_label: ColorRole,
    selected_leading_icon: ColorRole,
    selected_trailing: ColorRole,
}

impl MenuVariant {
    // 組の色は `groupStandardContainerColor`・`groupVibrantContainerColor`、項目の色は
    // `defaultMenuSelectableItemColors`・`defaultMenuSelectableItemVibrantColors` に従う。
    // 操作の項目の Compose の既定 (`itemColors`) は ListTokens の色で、Standard の選ばれていない色と同じ値になる。
    // 選ばれていない項目と無効な項目の容器の色は組の色と同じため、容器を描かない
    fn colors(self) -> Colors {
        match self {
            Self::Standard => Colors {
                group: standard_menu::CONTAINER_COLOR,
                label: standard_menu::ITEM_LABEL_TEXT_COLOR,
                leading_icon: standard_menu::ITEM_LEADING_ICON_COLOR,
                trailing: standard_menu::ITEM_TRAILING_ICON_COLOR,
                disabled_label: (
                    standard_menu::ITEM_DISABLED_LABEL_TEXT_COLOR,
                    standard_menu::ITEM_DISABLED_LABEL_TEXT_OPACITY,
                ),
                disabled_leading_icon: (
                    standard_menu::ITEM_DISABLED_LEADING_ICON_COLOR,
                    standard_menu::ITEM_DISABLED_LEADING_ICON_OPACITY,
                ),
                disabled_trailing: (
                    standard_menu::ITEM_DISABLED_TRAILING_ICON_COLOR,
                    standard_menu::ITEM_DISABLED_TRAILING_ICON_OPACITY,
                ),
                selected_container: standard_menu::ITEM_SELECTED_CONTAINER_COLOR,
                selected_label: standard_menu::ITEM_SELECTED_LABEL_TEXT_COLOR,
                selected_leading_icon: standard_menu::ITEM_SELECTED_LEADING_ICON_COLOR,
                selected_trailing: standard_menu::ITEM_SELECTED_TRAILING_ICON_COLOR,
            },
            Self::Vibrant => Colors {
                group: vibrant_menu::CONTAINER_COLOR,
                label: vibrant_menu::ITEM_LABEL_TEXT_COLOR,
                leading_icon: vibrant_menu::ITEM_LEADING_ICON_COLOR,
                trailing: vibrant_menu::ITEM_TRAILING_ICON_COLOR,
                disabled_label: (
                    vibrant_menu::ITEM_DISABLED_LABEL_TEXT_COLOR,
                    vibrant_menu::ITEM_DISABLED_LABEL_TEXT_OPACITY,
                ),
                disabled_leading_icon: (
                    vibrant_menu::ITEM_DISABLED_LEADING_ICON_COLOR,
                    vibrant_menu::ITEM_DISABLED_LEADING_ICON_OPACITY,
                ),
                disabled_trailing: (
                    vibrant_menu::ITEM_DISABLED_TRAILING_ICON_COLOR,
                    vibrant_menu::ITEM_DISABLED_TRAILING_ICON_OPACITY,
                ),
                selected_container: vibrant_menu::ITEM_SELECTED_CONTAINER_COLOR,
                selected_label: vibrant_menu::ITEM_SELECTED_LABEL_TEXT_COLOR,
                selected_leading_icon: vibrant_menu::ITEM_SELECTED_LEADING_ICON_COLOR,
                selected_trailing: vibrant_menu::ITEM_SELECTED_TRAILING_ICON_COLOR,
            },
        }
    }
}

/// MD3 Expressive のメニュー
///
/// [`IconButton::menu`](crate::IconButton::menu) などに渡し、そのボタンから開く。
/// 開いているかどうかは呼び出し側が持ち、[`Menu::new`] に毎回渡す。
/// メニュー自身は状態を変えず、開閉は [`Menu::on_expanded_change`] で求める。
/// 項目は [`MenuGroup`] にまとめて [`Menu::group`] で加え、組ごとに影の付いた容器に描く。
/// ボタンのテーマで描き、テーマの密度 ([`ThemeSettings::density`](md3_theme::ThemeSettings::density)) は反映しない
///
/// ボタン
///
/// - クリック・Enter・Space で、閉じていれば開くよう、開いていれば閉じるよう求める
/// - 下矢印キーで開くよう求め、上矢印キーで開くよう求めて最後の項目にフォーカスを移す
/// - 支援技術には、メニューを開くボタンであることと、開いているかどうかを公開する
///
/// 開いている間のメニュー
///
/// - 開いたとき、最初の項目 (上矢印キーで開いたときは最後の項目) にフォーカスを移す
/// - 上下の矢印キーで前後の項目へ、組をまたいでフォーカスを移し、端では反対の端に戻る。
///   Home と End では最初と最後の項目へ移る
/// - 項目にマウスが乗ると、その項目にフォーカスを移す
/// - Escape で閉じるよう求め、フォーカスをボタンへ戻す
/// - Tab で閉じるよう求め、フォーカスをボタンの次の要素へ移す。Shift+Tab で閉じるよう求め、フォーカスをボタンへ移す
/// - メニューとボタンの外でマウスのボタンを押すと、閉じるよう求める
///
/// メニューはボタンの下に、左端をボタンの左端にそろえて置く。
/// ウィンドウの左右の端から 8dp、上下の端から 48dp の内側に収まらなければ、ボタンの上や右端にそろえた位置に移す。
/// ウィンドウより大きければ縦にスクロールする
///
/// 支援技術には role `menu` として公開され、名前はボタンのアクセシブルな名前になる
/// ([`IconButton::new`](crate::IconButton::new) の `label`、[`Button::aria_label`](crate::Button::aria_label))。
/// ボタンの名前を子の内容から決める場合は [`Menu::aria_label`] で名前を与えること
pub struct Menu {
    id: ElementId,
    pub(crate) expanded: bool,
    on_expanded_change: Option<ExpandedChangeHandler>,
    variant: MenuVariant,
    groups: Vec<MenuGroup>,
    aria_label: Option<SharedString>,
}

impl Menu {
    /// `expanded` なら開いた状態で表示するメニューを作る
    ///
    /// `id` は描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, expanded: bool) -> Self {
        Self {
            id: id.into(),
            expanded,
            on_expanded_change: None,
            variant: MenuVariant::default(),
            groups: Vec::new(),
            aria_label: None,
        }
    }

    /// 開くか閉じるよう求められたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には、開くよう求められたら `true` を、閉じるよう求められたら `false` を渡す。
    /// 表示を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`Menu::new`] に渡すこと。
    /// 設定しない場合、ボタンは押されてもメニューの開閉を求めない
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(Box::new(handler));
        self
    }

    /// メニューの配色を設定する
    pub fn variant(mut self, variant: MenuVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 末尾に項目の組を加える
    ///
    /// 組の容器の形は、メニューの中の組の数と位置で決まる
    pub fn group(mut self, group: MenuGroup) -> Self {
        self.groups.push(group);
        self
    }

    /// ボタンのアクセシブルな名前の代わりに `label` をメニューの名前にする
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// `trigger` をこのメニューを開くボタンにして返す
    ///
    /// `trigger_id` は `trigger` の id、`trigger_label` は `trigger` のアクセシブルな名前にすること
    pub(crate) fn attach(
        self,
        trigger: HeadlessButton,
        trigger_id: &ElementId,
        trigger_label: Option<&SharedString>,
        theme: &SharedTheme,
        window: &mut Window,
        cx: &mut App,
    ) -> HeadlessButton {
        let mut headless = HeadlessMenu::new(self.id.clone(), self.expanded).window_margin(size(
            dp(supplement::menu::HORIZONTAL_MARGIN),
            dp(supplement::menu::VERTICAL_MARGIN),
        ));
        if let Some(handler) = self.on_expanded_change {
            headless = headless.on_expanded_change(handler);
        }
        if let Some(label) = self.aria_label.or_else(|| trigger_label.cloned()) {
            headless = headless.aria_label(label);
        }
        if !self.expanded {
            return headless.attach(trigger);
        }

        let colors = self.variant.colors();
        let shadow = theme.shadow(menu::CONTAINER_ELEVATION);
        // 組の影はメニューの外に出て、余白が無いとスクロールの範囲で切られるため、影の分の余白を取る。
        // gpui は影を、形の縁から、ぼかしの半径の 3 倍と広がりを足した幅だけ外側まで描く
        let margin = shadow
            .iter()
            .fold(Edges::<Pixels>::default(), |margin, shadow| {
                let extent = shadow.blur_radius * 3.0 + shadow.spread_radius;
                Edges {
                    top: margin.top.max(extent - shadow.offset.y),
                    right: margin.right.max(extent + shadow.offset.x),
                    bottom: margin.bottom.max(extent + shadow.offset.y),
                    left: margin.left.max(extent - shadow.offset.x),
                }
            });
        headless = headless
            .paint_margin(margin)
            .flex()
            .flex_col()
            .gap(dp(segmented_menu::SEGMENTED_GAP))
            // メニューはボタンの子として描かれ、ボタンの文字のスタイルを受け継ぐため、折り返しを戻す
            .whitespace_normal();
        // Compose は項目と組のラベルの文を labelLarge で描く。トークン (`ItemLabelTextFont`) は bodyLarge だが、
        // Compose はトークンに置き換える TODO を残して labelLarge を使っている
        headless
            .text_style()
            .refine(&theme.text_style(TypeScale::LabelLarge));

        let group_count = self.groups.len();
        for (group_ix, group) in self.groups.into_iter().enumerate() {
            // `MenuDefaults.groupShape`。上の 2 つの角と下の 2 つの角の形
            let (top, bottom) = if group_count == 1 {
                (
                    segmented_menu::CONTAINER_SHAPE,
                    segmented_menu::CONTAINER_SHAPE,
                )
            } else if group_ix == 0 {
                (ShapeKey::CornerLarge, ShapeKey::CornerSmall)
            } else if group_ix == group_count - 1 {
                (ShapeKey::CornerSmall, ShapeKey::CornerLarge)
            } else {
                (segmented_menu::GROUP_SHAPE, segmented_menu::GROUP_SHAPE)
            };
            let container = div()
                .absolute()
                .inset_0()
                .bg(theme.color(colors.group))
                .shadow(shadow.clone());
            let mut group_element = HeadlessMenuGroup::new(group.id)
                .relative()
                .flex()
                .flex_col()
                .py(dp(supplement::menu::GROUP_VERTICAL_PADDING))
                .child(Shaped::new(container, theme.clone(), top).bottom_corners(bottom));
            if let Some(label) = group.label {
                // `DropdownMenuGroupLabel`。文字の色は組の容器の色に対する内容の色 (Surface の `contentColorFor`)
                let label_element = div()
                    .flex()
                    .items_center()
                    .min_w(dp(supplement::menu::ITEM_MIN_WIDTH))
                    .max_w(dp(supplement::menu::ITEM_MAX_WIDTH))
                    .min_h(dp(supplement::menu::GROUP_LABEL_MIN_HEIGHT))
                    .pl(dp(supplement::menu::GROUP_LABEL_START_PADDING))
                    .pr(dp(supplement::menu::GROUP_LABEL_END_PADDING))
                    .text_color(theme.color(colors.label))
                    .child(label.clone());
                group_element = group_element.aria_label(label).child(label_element);
            }

            let item_count = group
                .entries
                .iter()
                .filter(|entry| matches!(entry, GroupEntry::Item(_)))
                .count();
            let mut item_ix = 0;
            for entry in group.entries {
                group_element = match entry {
                    GroupEntry::Item(item) => {
                        // `MenuDefaults.itemShape`
                        let shape = if item_count == 1 {
                            (segmented_menu::ITEM_SHAPE, segmented_menu::ITEM_SHAPE)
                        } else if item_ix == 0 {
                            (
                                segmented_menu::ITEM_FIRST_CHILD_SHAPE,
                                segmented_menu::ITEM_FIRST_CHILD_INNER_CORNER_CORNER_SIZE,
                            )
                        } else if item_ix == item_count - 1 {
                            (
                                segmented_menu::ITEM_LAST_CHILD_INNER_CORNER_CORNER_SIZE,
                                segmented_menu::ITEM_LAST_CHILD_SHAPE,
                            )
                        } else {
                            (segmented_menu::ITEM_SHAPE, segmented_menu::ITEM_SHAPE)
                        };
                        item_ix += 1;
                        // 項目の id はメニューの中でだけ一意なので、ボタンとメニューの id で分ける
                        let focus_handle =
                            window.with_element_namespace(trigger_id.clone(), |window| {
                                window.with_element_namespace(self.id.clone(), |window| {
                                    util::keyed_focus_handle(&item.id, window, cx)
                                })
                            });
                        group_element.item(item.into_headless(
                            shape,
                            &colors,
                            theme,
                            &focus_handle,
                            window,
                            cx,
                        ))
                    }
                    GroupEntry::Divider => group_element.child(
                        Divider::new()
                            .theme(theme.clone())
                            .mx(dp(supplement::menu::DIVIDER_HORIZONTAL_PADDING))
                            .my(dp(supplement::menu::DIVIDER_VERTICAL_PADDING)),
                    ),
                };
            }
            headless = headless.group(group_element);
        }
        headless.attach(trigger)
    }
}

/// [`Menu`] の項目をまとめる組
///
/// [`Menu::group`] に渡して使う。項目と区切り線は加えた順に縦に並び、組ごとに影の付いた容器に描く。
/// 支援技術には role `group` として公開され、[`MenuGroup::label`] が名前になる
pub struct MenuGroup {
    id: ElementId,
    label: Option<SharedString>,
    entries: Vec<GroupEntry>,
}

/// 組の中に並べるもの
enum GroupEntry {
    Item(Box<MenuItem>),
    Divider,
}

impl MenuGroup {
    /// 空の組を作る
    ///
    /// `id` はメニューの中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            entries: Vec::new(),
        }
    }

    /// 組の先頭に `label` を表示し、組の名前にする
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 末尾に項目を加える
    ///
    /// 項目の容器の形は、組の中の項目の数と位置で決まる
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(GroupEntry::Item(Box::new(item)));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.entries.extend(
            items
                .into_iter()
                .map(|item| GroupEntry::Item(Box::new(item))),
        );
        self
    }

    /// 末尾に横の区切り線を加える
    ///
    /// 区切り線は装飾として扱い、支援技術には公開しない
    pub fn divider(mut self) -> Self {
        self.entries.push(GroupEntry::Divider);
        self
    }
}

/// メニューの項目の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuItemKind {
    /// 操作を実行する項目
    #[default]
    Action,
    /// オンとオフを切り替える項目。値はオンかどうか
    Checkbox(bool),
    /// 同じ組の中から 1 つを選ぶ項目。値は選ばれているかどうか
    Radio(bool),
}

/// 項目の末尾に置くもの
enum Trailing {
    Icon(StyledIcon),
    Shortcut(SharedString),
}

/// [`Menu`] の項目
///
/// [`MenuGroup::item`] に渡して使う。`label` の文と、指定したアイコンや補足の文を 1 行に並べる。
/// フォーカス中の Enter と Space・マウスのクリックで [`MenuItem::on_click`] のハンドラを呼ぶ。
/// 続けて、[`MenuItemKind::Action`] の項目ではメニューを閉じるよう求め、切り替える項目では Enter のときだけ閉じるよう求める。
/// オンか選ばれている項目は、容器を塗って形を変える
///
/// 支援技術には、種類に応じて role `menuitem`、`menuitemcheckbox`、`menuitemradio` として公開し、
/// 切り替える項目はその状態を公開する。`label` が名前に、補足の文が説明になる
pub struct MenuItem {
    id: ElementId,
    label: SharedString,
    kind: MenuItemKind,
    leading_icon: Option<StyledIcon>,
    selected_leading_icon: Option<StyledIcon>,
    trailing: Option<Trailing>,
    supporting_text: Option<SharedString>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl MenuItem {
    /// `label` を表示し、アクセシブルな名前にした [`MenuItemKind::Action`] の項目を作る
    ///
    /// `id` はメニューの中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: MenuItemKind::default(),
            leading_icon: None,
            selected_leading_icon: None,
            trailing: None,
            supporting_text: None,
            disabled: false,
            on_click: None,
        }
    }

    /// 項目の種類を設定する
    pub fn kind(mut self, kind: MenuItemKind) -> Self {
        self.kind = kind;
        self
    }

    /// 文の前にアイコンを置く
    ///
    /// [`md3_icons::Icon`] を渡すと Outlined の塗りのない形で描く。アイコンは装飾として扱い、名前には含めない
    pub fn leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }

    /// オンか選ばれている間、文の前に [`MenuItem::leading_icon`] の代わりに `icon` を置く
    ///
    /// [`MenuItem::leading_icon`] が無ければ、オンか選ばれている間だけ `icon` を置く。
    /// 置かない間も、文の前にアイコンとの間隔を空ける
    pub fn selected_leading_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.selected_leading_icon = Some(icon.into());
        self
    }

    /// 項目の末尾にアイコンを置く。[`MenuItem::shortcut`] の文は置かなくなる
    ///
    /// アイコンは装飾として扱い、名前には含めない
    pub fn trailing_icon(mut self, icon: impl Into<StyledIcon>) -> Self {
        self.trailing = Some(Trailing::Icon(icon.into()));
        self
    }

    /// 項目の末尾に、項目の操作を実行するキー操作を表す文 (「Ctrl+X」など) を置く。
    /// [`MenuItem::trailing_icon`] のアイコンは置かなくなる
    ///
    /// 支援技術には項目のキーボードショートカットとして公開する。キー操作そのものは登録しない
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.trailing = Some(Trailing::Shortcut(shortcut.into()));
        self
    }

    /// `label` の下に補足の文を置く
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な項目は無効の色で描かれ、操作されてもハンドラを呼ばずメニューも閉じない。
    /// 矢印キーとマウスではフォーカスできる
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// 切り替える項目の状態を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`MenuItem::kind`] に渡すこと
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 見た目を付けた headless の項目にする
    ///
    /// `shape` は選ばれていないときの上の 2 つの角と下の 2 つの角の形、`focus_handle` は項目のフォーカスにする
    fn into_headless(
        self,
        shape: (ShapeKey, ShapeKey),
        colors: &Colors,
        theme: &SharedTheme,
        focus_handle: &FocusHandle,
        window: &Window,
        cx: &App,
    ) -> HeadlessMenuItem {
        let selected = matches!(
            self.kind,
            MenuItemKind::Checkbox(true) | MenuItemKind::Radio(true)
        );
        let disabled = self.disabled;
        // リングと状態レイヤーは容器の形に合わせて描くため、項目自身のスタイルでは表せない
        let focus_visible = gpui_headless::is_focus_visible(focus_handle, window, cx);
        let with_opacity = |(role, opacity): (ColorRole, f32)| theme.color(role).opacity(opacity);
        // 無効なときは、選ばれていても無効の色にする (`SelectableMenuItemColors`)
        let (label_color, leading_color, trailing_color) = if disabled {
            (
                with_opacity(colors.disabled_label),
                with_opacity(colors.disabled_leading_icon),
                with_opacity(colors.disabled_trailing),
            )
        } else if selected {
            (
                theme.color(colors.selected_label),
                theme.color(colors.selected_leading_icon),
                theme.color(colors.selected_trailing),
            )
        } else {
            (
                theme.color(colors.label),
                theme.color(colors.leading_icon),
                theme.color(colors.trailing),
            )
        };
        let container = (selected && !disabled).then(|| theme.color(colors.selected_container));
        // `shapeByInteraction`。選ばれている項目は、組の中の位置によらず同じ形にする
        let (top, bottom) = if selected {
            (
                segmented_menu::ITEM_SELECTED_SHAPE,
                segmented_menu::ITEM_SELECTED_SHAPE,
            )
        } else {
            shape
        };
        let shaped = |inner: Div| Shaped::new(inner, theme.clone(), top).bottom_corners(bottom);
        // 状態レイヤーは文の色を重ねる (Compose の Surface の ripple は、容器の色に対する内容の色を使う)
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
                this.group_hover(GROUP, |style| {
                    style.bg(state_layer(state::HOVER_STATE_LAYER_OPACITY))
                })
            });

        let icon = |icon: StyledIcon, size, color| {
            svg()
                .data(icon.svg().as_bytes())
                .flex_none()
                .size(dp(size))
                .text_color(color)
        };
        // Compose は先頭のアイコンが選ばれている間だけのとき、選ばれていない間はアイコンの箱を 0 の幅にし、
        // 文の前の間隔は残す
        let has_leading = self.leading_icon.is_some() || self.selected_leading_icon.is_some();
        let leading_icon = if selected {
            self.selected_leading_icon.or(self.leading_icon)
        } else {
            self.leading_icon
        };
        let leading = has_leading.then(|| {
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .when_some(leading_icon, |this, leading_icon| {
                    this.min_w(dp(segmented_menu::ITEM_LEADING_ICON_SIZE))
                        .child(icon(
                            leading_icon,
                            segmented_menu::ITEM_LEADING_ICON_SIZE,
                            leading_color,
                        ))
                })
        });
        let mut text = div()
            .flex()
            .flex_col()
            .flex_grow_1()
            .min_w_0()
            .child(self.label.clone());
        if let Some(supporting_text) = &self.supporting_text {
            // `LabelWithSupportingText`。色は文と同じ
            let mut supporting = div().child(supporting_text.clone());
            supporting
                .text_style()
                .refine(&theme.text_style(TypeScale::BodyMedium));
            text = text.child(supporting);
        }
        let shortcut = match &self.trailing {
            Some(Trailing::Shortcut(shortcut)) => Some(shortcut.clone()),
            _ => None,
        };
        // `DropdownMenuItemTrailingLabel` の文は、項目の文と同じ書体で末尾の内容の色にする
        let trailing = self.trailing.map(|content| {
            let slot = div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .min_w(dp(segmented_menu::ITEM_TRAILING_ICON_SIZE))
                .text_color(trailing_color);
            match content {
                Trailing::Icon(trailing_icon) => slot.child(icon(
                    trailing_icon,
                    segmented_menu::ITEM_TRAILING_ICON_SIZE,
                    trailing_color,
                )),
                Trailing::Shortcut(shortcut) => slot.child(shortcut),
            }
        });
        // Material Web の項目はリングを内側に描く
        let ring = focus_visible.then(|| {
            let ring = div()
                .absolute()
                .inset_0()
                .border(dp(focus_ring::WIDTH))
                .border_color(theme.color(focus_ring::COLOR));
            shaped(ring).outset(-dp(focus_ring::INWARD_OFFSET))
        });

        let mut item = HeadlessMenuItem::new(self.id)
            .kind(match self.kind {
                MenuItemKind::Action => HeadlessMenuItemKind::Action,
                MenuItemKind::Checkbox(checked) => HeadlessMenuItemKind::Checkbox(if checked {
                    CheckedState::Checked
                } else {
                    CheckedState::Unchecked
                }),
                MenuItemKind::Radio(selected) => HeadlessMenuItemKind::Radio(selected),
            })
            .disabled(disabled)
            .track_focus(focus_handle)
            .aria_label(self.label)
            .group(GROUP)
            .relative()
            .flex()
            .items_center()
            .gap(dp(supplement::menu::ICON_TEXT_SPACING))
            .mx(dp(supplement::menu::ITEM_OUTER_HORIZONTAL_PADDING))
            .min_w(dp(supplement::menu::ITEM_MIN_WIDTH))
            .max_w(dp(supplement::menu::ITEM_MAX_WIDTH))
            .min_h(dp(segmented_menu::ITEM))
            .px(dp(supplement::menu::ITEM_HORIZONTAL_PADDING))
            .py(dp(supplement::menu::ITEM_VERTICAL_PADDING))
            .text_color(label_color)
            .child(shaped(layer))
            .children(leading)
            .child(text)
            .children(trailing)
            .children(ring);
        if let Some(supporting_text) = self.supporting_text {
            item = item
                .my(dp(
                    supplement::menu::ITEM_WITH_SUPPORTING_TEXT_OUTER_VERTICAL_PADDING,
                ))
                .aria_description(supporting_text);
        }
        if let Some(shortcut) = shortcut {
            item = item.aria_keyshortcuts(shortcut);
        }
        if let Some(on_click) = self.on_click {
            item = item.on_click(on_click);
        }
        item
    }
}

#[cfg(test)]
mod tests {
    use gpui::{
        AnyElement, Context, Hsla, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers,
        Render, TestAppContext, VisualTestContext, point, px,
    };
    use md3_theme::ThemeExt;

    use super::*;
    use crate::Button;

    /// 左上を (100, 100) に置いた、`trigger` が `menu` を付けて作るボタン
    struct MenuView {
        expanded: bool,
        variant: MenuVariant,
        groups: fn() -> Vec<MenuGroup>,
        trigger: fn(Menu) -> AnyElement,
    }

    /// 中身の無い Small のボタン (40x40dp)
    fn button(menu: Menu) -> AnyElement {
        Button::new("button")
            .aria_label("More")
            .menu(menu)
            .into_any_element()
    }

    /// ツールチップを付けた Small のアイコンボタン (40x40dp)
    #[cfg(feature = "icon-button")]
    fn icon_button(menu: Menu) -> AnyElement {
        crate::IconButton::new("button", md3_icons::MORE_VERT, "More")
            .tooltip(crate::Tooltip::new("More options"))
            .menu(menu)
            .into_any_element()
    }

    impl Render for MenuView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let mut menu = Menu::new("menu", self.expanded)
                .variant(self.variant)
                .on_expanded_change(cx.listener(|this, expanded: &bool, _, cx| {
                    this.expanded = *expanded;
                    cx.notify();
                }));
            for group in (self.groups)() {
                menu = menu.group(group);
            }
            let trigger = (self.trigger)(menu);
            crate::root(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_start()
                    .pl(px(100.))
                    .pt(px(100.))
                    .child(trigger),
            )
        }
    }

    fn add_view(
        cx: &mut TestAppContext,
        expanded: bool,
        groups: fn() -> Vec<MenuGroup>,
    ) -> (gpui::Entity<MenuView>, &mut VisualTestContext) {
        cx.update(crate::init);
        cx.add_window_view(|window, cx| {
            cx.init_md3_theme(window);
            MenuView {
                expanded,
                variant: MenuVariant::Standard,
                groups,
                trigger: button,
            }
        })
    }

    fn update(
        view: &gpui::Entity<MenuView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut MenuView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn color(cx: &mut VisualTestContext, role: ColorRole) -> Hsla {
        cx.update(|_, cx| cx.md3_theme().color(role))
    }

    /// 矩形の x, y, 幅, 高さと、左上と左下の角丸の半径
    type Rect = ((f32, f32, f32, f32), (f32, f32));

    /// 背景か枠線を `color` で描いた矩形
    fn painted(cx: &mut VisualTestContext, color: Hsla, border: bool) -> Vec<Rect> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| {
                    if border {
                        quad.border_color == color
                    } else {
                        quad.background == color.into()
                    }
                })
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        (
                            b.origin.x.0 / scale,
                            b.origin.y.0 / scale,
                            b.size.width.0 / scale,
                            b.size.height.0 / scale,
                        ),
                        (
                            quad.corner_radii.top_left.0 / scale,
                            quad.corner_radii.bottom_left.0 / scale,
                        ),
                    )
                })
                .collect()
        })
    }

    fn filled(cx: &mut VisualTestContext, role: ColorRole) -> Vec<Rect> {
        let color = color(cx, role);
        painted(cx, color, false)
    }

    /// フォーカスリングの矩形
    fn ring(cx: &mut VisualTestContext) -> Vec<Rect> {
        let color = color(cx, focus_ring::COLOR);
        let mut rects = painted(cx, color, true);
        // gpui は角丸の枠線を辺ごとの quad に分けて描く
        rects.dedup();
        rects
    }

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    fn two_items() -> Vec<MenuGroup> {
        vec![
            MenuGroup::new("edit")
                .items([MenuItem::new("cut", "Cut"), MenuItem::new("copy", "Copy")]),
        ]
    }

    /// 3 つの項目の組と、オンの項目 1 つの組
    fn two_groups() -> Vec<MenuGroup> {
        vec![
            MenuGroup::new("edit").items([
                MenuItem::new("cut", "Cut"),
                MenuItem::new("copy", "Copy"),
                MenuItem::new("paste", "Paste"),
            ]),
            MenuGroup::new("view")
                .item(MenuItem::new("ruler", "Ruler").kind(MenuItemKind::Checkbox(true))),
        ]
    }

    /// 期待値は Compose の項目の最小の幅 112dp と外側の左右の余白 4dp、
    /// 項目の高さ (SegmentedMenuTokens の Item) 44dp と組の上下の余白 2dp、
    /// 1 つだけの組の形 (SegmentedMenuTokens の ContainerShape) CornerLarge (16dp)、
    /// StandardMenuTokens の ContainerColor (SurfaceContainerLow)。
    /// `MenuAnchorPosition.Below` で、ボタンの下端 140 に上端を、左端 100 に左端をそろえる
    #[gpui::test]
    fn 組の容器をボタンの下に左端をそろえて描く(cx: &mut TestAppContext) {
        let triggers: [fn(Menu) -> AnyElement; _] = [
            button,
            #[cfg(feature = "icon-button")]
            icon_button,
        ];
        for (ix, trigger) in triggers.into_iter().enumerate() {
            let (view, cx) = add_view(cx, false, two_items);
            update(&view, cx, |view| view.trigger = trigger);
            assert_eq!(filled(cx, ColorRole::SurfaceContainerLow), vec![], "{ix}");
            update(&view, cx, |view| view.expanded = true);
            assert_eq!(
                filled(cx, ColorRole::SurfaceContainerLow),
                vec![((100., 140., 120., 92.), (16., 16.))],
                "{ix}"
            );
        }
    }

    /// 期待値は Compose の `MenuDefaults.groupShape` (先頭の組は上が CornerLarge で下が CornerSmall、
    /// 末尾の組はその逆)、`SegmentedGap` の 2dp、選ばれた項目の `ItemSelectedShape` (CornerMedium、12dp) と
    /// StandardMenuTokens の `ItemSelectedContainerColor` (TertiaryContainer)
    #[gpui::test]
    fn 組と選ばれた項目の形を位置と状態で決める(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, true, two_groups);
        cx.run_until_parked();
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainerLow),
            vec![
                ((100., 140., 120., 136.), (16., 8.)),
                ((100., 278., 120., 48.), (8., 16.)),
            ]
        );
        assert_eq!(
            filled(cx, ColorRole::TertiaryContainer),
            vec![((104., 280., 112., 44.), (12., 12.))]
        );
    }

    /// 期待値は Compose の `MenuDefaults.itemShape` (先頭の項目は上が CornerMedium で下が CornerExtraSmall、
    /// 末尾の項目はその逆、間の項目は CornerExtraSmall) と、Material Web の内側のフォーカスリング
    #[gpui::test]
    fn キーで開くと項目の形に合わせて内側にフォーカスリングを描く(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, false, two_groups);
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        press_key(cx, "down");
        assert_eq!(ring(cx), vec![((104., 142., 112., 44.), (12., 4.))]);
        press_key(cx, "down");
        assert_eq!(ring(cx), vec![((104., 186., 112., 44.), (4., 4.))]);
        press_key(cx, "down");
        assert_eq!(ring(cx), vec![((104., 230., 112., 44.), (4., 12.))]);
        press_key(cx, "down");
        assert_eq!(ring(cx), vec![((104., 280., 112., 44.), (12., 12.))]);
    }

    /// メニューの縁の影のための余白は、ボタンの下端 140 から上へ重なる
    #[gpui::test]
    fn 影のための余白に重なるボタンを押すと閉じる(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, false, two_items);
        cx.simulate_click(point(px(120.), px(135.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.expanded));
        cx.simulate_click(point(px(120.), px(135.)), Modifiers::none());
        assert!(!view.read_with(cx, |view, _| view.expanded));
    }

    #[gpui::test]
    fn クリックで開いたときはフォーカスリングを描かない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = add_view(cx, false, two_groups);
        cx.simulate_click(point(px(120.), px(120.)), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.expanded));
        assert_eq!(ring(cx), vec![]);
    }

    /// 期待値は VibrantMenuTokens の `ContainerColor` (TertiaryContainer) と
    /// `ItemSelectedContainerColor` (Tertiary)
    #[gpui::test]
    fn vibrant_は組と選ばれた項目を_tertiary_の色で描く(cx: &mut TestAppContext) {
        let (view, cx) = add_view(cx, true, two_groups);
        update(&view, cx, |view| view.variant = MenuVariant::Vibrant);
        assert_eq!(
            filled(cx, ColorRole::TertiaryContainer)
                .iter()
                .map(|(bounds, _)| bounds.1)
                .collect::<Vec<_>>(),
            vec![140., 278.]
        );
        assert_eq!(
            filled(cx, ColorRole::Tertiary),
            vec![((104., 280., 112., 44.), (12., 12.))]
        );
    }

    /// Compose の `SelectableMenuItemColors` は、無効なときは選ばれていても容器を組の色にする。
    /// 形は `shapeByInteraction` で選ばれた形 (CornerMedium、12dp) のままにする
    #[gpui::test]
    fn 無効な項目は選ばれていても容器を塗らず形は選ばれた形にする(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = add_view(cx, false, || {
            vec![
                MenuGroup::new("view").item(
                    MenuItem::new("ruler", "Ruler")
                        .kind(MenuItemKind::Radio(true))
                        .disabled(true),
                ),
            ]
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_keystrokes("tab");
        press_key(cx, "down");
        assert_eq!(filled(cx, ColorRole::TertiaryContainer), vec![]);
        assert_eq!(ring(cx), vec![((104., 142., 112., 44.), (12., 12.))]);
    }

    /// 期待値は Compose の補足の文を持つ項目の外側の上下の余白 2dp と、labelLarge と bodyMedium の行の高さ 20sp
    #[gpui::test]
    fn 補足の文を持つ項目は外側に上下の余白を取る(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, true, || {
            vec![
                MenuGroup::new("edit")
                    .item(MenuItem::new("cut", "Cut").supporting_text("Move to the clipboard")),
            ]
        });
        cx.run_until_parked();
        let rects = filled(cx, ColorRole::SurfaceContainerLow);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].0.3, 2. + 2. + 12. + 20. + 20. + 12. + 2. + 2.);
    }

    /// 期待値は Compose の項目の最大の幅 280dp と外側の左右の余白 4dp
    #[gpui::test]
    fn 長い文は最大の幅で折り返す(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, true, || {
            vec![MenuGroup::new("edit").item(MenuItem::new(
                "cut",
                "A menu item label that is long enough to wrap onto more than one line",
            ))]
        });
        cx.run_until_parked();
        let rects = filled(cx, ColorRole::SurfaceContainerLow);
        assert_eq!(rects.len(), 1);
        let (_, _, width, height) = rects[0].0;
        assert_eq!(width, 280. + 8.);
        assert!(height > 2. + 44. + 2., "{height}");
    }

    /// 期待値は Compose の `DropdownMenuGroupLabel` の最小の高さ 32dp と、
    /// `MenuDefaults.HorizontalDividerPadding` (左右 12dp、上下 2dp) の区切り線 (OutlineVariant、1dp)
    #[gpui::test]
    fn 組のラベルと区切り線を項目の前に並べる(cx: &mut TestAppContext) {
        let (_, cx) = add_view(cx, true, || {
            vec![
                MenuGroup::new("edit")
                    .label("Edit")
                    .divider()
                    .item(MenuItem::new("cut", "Cut")),
            ]
        });
        cx.run_until_parked();
        assert_eq!(
            filled(cx, ColorRole::SurfaceContainerLow),
            vec![((100., 140., 120., 2. + 32. + 5. + 44. + 2.), (16., 16.))]
        );
        assert_eq!(
            filled(cx, ColorRole::OutlineVariant),
            vec![((112., 176., 96., 1.), (0., 0.))]
        );
    }

    #[cfg(feature = "icon-button")]
    #[gpui::test]
    fn 開いている間はボタンのツールチップを表示しない(
        cx: &mut TestAppContext,
    ) {
        for expanded in [false, true] {
            let (view, cx) = add_view(cx, expanded, two_items);
            update(&view, cx, |view| view.trigger = icon_button);
            cx.simulate_mouse_move(point(px(120.), px(120.)), None, Modifiers::none());
            assert_eq!(
                filled(cx, ColorRole::InverseSurface).len(),
                usize::from(!expanded),
                "{expanded}"
            );
        }
    }
}
