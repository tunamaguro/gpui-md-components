//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "button")]
mod button;
#[cfg(feature = "card")]
mod card;
#[cfg(feature = "checkbox")]
mod checkbox;
#[cfg(feature = "chip")]
mod chip;
#[cfg(feature = "dialog")]
mod dialog;
#[cfg(feature = "divider")]
mod divider;
#[cfg(feature = "fab")]
mod fab;
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
#[cfg(feature = "menu")]
mod menu;
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
    feature = "menu",
    feature = "segmented-button",
    feature = "select",
    feature = "slider",
    feature = "text-field"
))]
mod shape;
#[cfg(feature = "slider")]
mod slider;
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
    feature = "radio-button",
    feature = "segmented-button",
    feature = "select",
    feature = "slider",
    feature = "switch",
    feature = "tabs",
    feature = "text-field"
))]
mod util;

#[cfg(feature = "badge")]
pub use badge::{Badge, BadgeColors, BadgedBox};
#[cfg(feature = "button")]
pub use button::{Button, ButtonColors, ButtonShape, ButtonSize, ButtonVariant};
#[cfg(feature = "card")]
pub use card::{Card, CardVariant};
#[cfg(feature = "checkbox")]
pub use checkbox::{Checkbox, CheckboxIndicator, CheckboxRow, CheckboxState};
#[cfg(feature = "chip")]
pub use chip::{Chip, ChipKind, ChipVariant, FilterChip, InputChip};
#[cfg(feature = "dialog")]
pub use dialog::{Dialog, DialogKind, DismissReason};
#[cfg(feature = "divider")]
pub use divider::{Divider, DividerOrientation};
#[cfg(feature = "fab")]
pub use fab::{ExtendedFab, ExtendedFabSize, Fab, FabColor, FabSize};
pub use gpui;
pub use gpui_platform;
#[cfg(feature = "icon-button")]
pub use icon_button::{
    IconButton, IconButtonVariant, IconButtonWidth, IconToggleButton, IconToggleButtonColors,
};
#[cfg(feature = "checkbox")]
pub use interaction::InteractionSource;
#[cfg(feature = "list")]
pub use list::{List, ListItem, ListItemKind};
#[cfg(feature = "menu")]
pub use menu::{Menu, MenuGroup, MenuItem, MenuItemKind, MenuVariant};
#[cfg(feature = "progress-indicator")]
pub use progress_indicator::{
    Progress, ProgressIndicator, ProgressIndicatorColors, ProgressIndicatorVariant,
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
