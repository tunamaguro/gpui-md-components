//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "app-bar")]
mod app_bar;
#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "bottom-app-bar")]
mod bottom_app_bar;
#[cfg(feature = "button")]
mod button;
#[cfg(feature = "button-group")]
mod button_group;
#[cfg(feature = "card")]
mod card;
#[cfg(feature = "checkbox")]
mod checkbox;
#[cfg(feature = "chip")]
mod chip;
#[cfg(feature = "connected-button-group")]
mod connected_button_group;
#[cfg(feature = "dialog")]
mod dialog;
#[cfg(feature = "divider")]
mod divider;
#[cfg(feature = "docked-toolbar")]
mod docked_toolbar;
#[cfg(feature = "fab")]
mod fab;
#[cfg(feature = "floating-toolbar")]
mod floating_toolbar;
#[cfg(feature = "icon-button")]
mod icon_button;
// 使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(feature = "checkbox")]
mod interaction;
// 入力欄を使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(feature = "select", feature = "text-field"))]
mod field;
#[cfg(feature = "list")]
mod list;
#[cfg(feature = "loading-indicator")]
mod loading_indicator;
#[cfg(feature = "menu")]
mod menu;
#[cfg(feature = "navigation-bar")]
mod navigation_bar;
#[cfg(feature = "navigation-drawer")]
mod navigation_drawer;
#[cfg(feature = "navigation-rail")]
mod navigation_rail;
// 進み具合を表す部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(feature = "loading-indicator", feature = "progress-indicator"))]
mod progress;
#[cfg(feature = "progress-indicator")]
mod progress_indicator;
#[cfg(feature = "radio-button")]
mod radio_button;
#[cfg(feature = "segmented-button")]
mod segmented_button;
#[cfg(feature = "select")]
mod select;
// 形状を使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(
    feature = "badge",
    feature = "button",
    feature = "card",
    feature = "chip",
    feature = "dialog",
    feature = "fab",
    feature = "icon-button",
    feature = "list",
    feature = "loading-indicator",
    feature = "menu",
    feature = "navigation-bar",
    feature = "navigation-drawer",
    feature = "navigation-rail",
    feature = "segmented-button",
    feature = "select",
    feature = "slider",
    feature = "text-field"
))]
mod shape;
#[cfg(feature = "slider")]
mod slider;
#[cfg(feature = "snackbar")]
mod snackbar;
#[cfg(feature = "split-button")]
mod split_button;
#[cfg(feature = "switch")]
mod switch;
#[cfg(feature = "tabs")]
mod tabs;
#[cfg(feature = "text-field")]
mod text_field;
// ツールチップを付けられる部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(feature = "icon-button")]
mod tooltip;
// 使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(
    feature = "button",
    feature = "card",
    feature = "checkbox",
    feature = "chip",
    feature = "fab",
    feature = "list",
    feature = "menu",
    feature = "navigation-bar",
    feature = "navigation-drawer",
    feature = "navigation-rail",
    feature = "radio-button",
    feature = "segmented-button",
    feature = "select",
    feature = "slider",
    feature = "switch",
    feature = "tabs",
    feature = "text-field"
))]
mod util;

#[cfg(feature = "app-bar")]
pub use app_bar::{TopAppBar, TopAppBarTitleAlignment, TopAppBarVariant};
#[cfg(feature = "badge")]
pub use badge::{Badge, BadgeColors, BadgedBox};
#[cfg(feature = "bottom-app-bar")]
pub use bottom_app_bar::BottomAppBar;
#[cfg(feature = "button")]
pub use button::{
    Button, ButtonColors, ButtonShape, ButtonSize, ButtonVariant, ToggleButton, ToggleButtonColors,
    ToggleButtonVariant,
};
#[cfg(feature = "button-group")]
pub use button_group::{ButtonGroup, ButtonGroupItem};
#[cfg(feature = "card")]
pub use card::{Card, CardVariant};
#[cfg(feature = "checkbox")]
pub use checkbox::{Checkbox, CheckboxIndicator, CheckboxRow, CheckboxState};
#[cfg(feature = "chip")]
pub use chip::{Chip, ChipKind, ChipVariant, FilterChip, InputChip};
#[cfg(feature = "connected-button-group")]
pub use connected_button_group::{
    ConnectedButton, ConnectedToggleButton, MultiChoiceConnectedButtonGroup,
    SingleChoiceConnectedButtonGroup,
};
#[cfg(feature = "dialog")]
pub use dialog::{Dialog, DialogKind, DismissReason};
#[cfg(feature = "divider")]
pub use divider::{Divider, DividerOrientation};
#[cfg(feature = "docked-toolbar")]
pub use docked_toolbar::{DockedToolbar, DockedToolbarArrangement};
#[cfg(feature = "fab")]
pub use fab::{ExtendedFab, ExtendedFabSize, Fab, FabColor, FabSize};
#[cfg(feature = "floating-toolbar")]
pub use floating_toolbar::{FloatingToolbar, FloatingToolbarColor, FloatingToolbarOrientation};
pub use gpui;
#[cfg(feature = "platform")]
pub use gpui_platform;
#[cfg(feature = "icon-button")]
pub use icon_button::{
    IconButton, IconButtonVariant, IconButtonWidth, IconToggleButton, IconToggleButtonColors,
};
#[cfg(feature = "checkbox")]
pub use interaction::InteractionSource;
#[cfg(feature = "list")]
pub use list::{List, ListItem, ListItemKind};
#[cfg(feature = "loading-indicator")]
pub use loading_indicator::{LoadingIndicator, LoadingIndicatorVariant};
#[cfg(feature = "menu")]
pub use menu::{Menu, MenuGroup, MenuItem, MenuItemKind, MenuVariant};
#[cfg(feature = "navigation-bar")]
pub use navigation_bar::{
    NavigationBar, NavigationBarArrangement, NavigationBarItem, NavigationItemIconPosition,
};
#[cfg(feature = "navigation-drawer")]
pub use navigation_drawer::{ModalNavigationDrawer, NavigationDrawer, NavigationDrawerItem};
// dialog feature が有効なら dialog から公開している
#[cfg(all(feature = "navigation-drawer", not(feature = "dialog")))]
pub use gpui_headless::dialog::DismissReason;
#[cfg(feature = "navigation-rail")]
pub use navigation_rail::{NavigationRail, NavigationRailArrangement, NavigationRailItem};
#[cfg(any(feature = "loading-indicator", feature = "progress-indicator"))]
pub use progress::Progress;
#[cfg(feature = "progress-indicator")]
pub use progress_indicator::{
    ProgressIndicator, ProgressIndicatorColors, ProgressIndicatorVariant,
};
#[cfg(feature = "radio-button")]
pub use radio_button::{RadioButton, RadioButtonIndicator, RadioGroup};
#[cfg(feature = "segmented-button")]
pub use segmented_button::{
    MultiChoiceSegmentedButtonRow, SegmentedButton, SegmentedToggleButton,
    SingleChoiceSegmentedButtonRow,
};
#[cfg(feature = "select")]
pub use select::{Select, SelectOption, SelectVariant};
#[cfg(feature = "slider")]
pub use slider::Slider;
#[cfg(feature = "snackbar")]
pub use snackbar::{
    SnackbarDuration, SnackbarHost, SnackbarHostState, SnackbarMessage, SnackbarResult,
};
#[cfg(feature = "split-button")]
pub use split_button::{SplitButton, SplitButtonVariant};
#[cfg(feature = "switch")]
pub use switch::Switch;
#[cfg(feature = "tabs")]
pub use tabs::{Tab, TabPanel, TabRow, TabRowVariant};
#[cfg(feature = "text-field")]
pub use text_field::{TextField, TextFieldVariant};
#[cfg(feature = "icon-button")]
pub use tooltip::{Tooltip, TooltipVariant};

/// この crate の部品が使うキー割り当てと操作を登録する。アプリの起動時に、ウィンドウを開く前に 1 回呼ぶ
pub fn init(cx: &mut gpui::App) {
    gpui_headless::init(cx);
}

/// `child` をウィンドウのルートとして包む
///
/// この crate の部品が必要とするウィンドウ全体の入力を記録する。部品を使うウィンドウでは、
/// ルートビューの render が返す要素を包むこと。包まない場合、フォーカスリングがマウスの操作で消えない。
/// レイアウトと見た目は `child` のまま
pub fn root(child: impl gpui::IntoElement) -> impl gpui::IntoElement {
    gpui_headless::root(child)
}
