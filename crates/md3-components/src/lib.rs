//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "button")]
mod button;
#[cfg(feature = "checkbox")]
mod checkbox;
#[cfg(feature = "divider")]
mod divider;
#[cfg(feature = "icon-button")]
mod icon_button;
// 使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(feature = "checkbox")]
mod interaction;
#[cfg(feature = "progress-indicator")]
mod progress_indicator;
#[cfg(feature = "radio-button")]
mod radio_button;
// 形状を使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(feature = "badge", feature = "button"))]
mod shape;
#[cfg(feature = "switch")]
mod switch;
// 使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(
    feature = "button",
    feature = "checkbox",
    feature = "radio-button",
    feature = "switch"
))]
mod util;

#[cfg(feature = "badge")]
pub use badge::{Badge, BadgeColors, BadgedBox};
#[cfg(feature = "button")]
pub use button::{Button, ButtonColors, ButtonShape, ButtonSize, ButtonVariant};
#[cfg(feature = "checkbox")]
pub use checkbox::{Checkbox, CheckboxIndicator, CheckboxRow, CheckboxState};
#[cfg(feature = "divider")]
pub use divider::{Divider, DividerOrientation};
pub use gpui;
pub use gpui_platform;
#[cfg(feature = "icon-button")]
pub use icon_button::{
    IconButton, IconButtonVariant, IconButtonWidth, IconToggleButton, IconToggleButtonColors,
};
#[cfg(feature = "checkbox")]
pub use interaction::InteractionSource;
#[cfg(feature = "progress-indicator")]
pub use progress_indicator::{
    Progress, ProgressIndicator, ProgressIndicatorColors, ProgressIndicatorVariant,
};
#[cfg(feature = "radio-button")]
pub use radio_button::{RadioButton, RadioButtonIndicator, RadioGroup};
#[cfg(feature = "switch")]
pub use switch::Switch;

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
