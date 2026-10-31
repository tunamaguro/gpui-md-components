//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント。
//!
//! 基礎部品（[`Surface`], [`StateLayer`], [`FocusRing`], [`Icon`], [`Text`]）と各コンポーネントを提供する。
//! コンポーネント群は feature（`buttons`, `selection`, `inputs`, `overlays`, `navigation`, `containment`）で分割する。
//!
//! 起動時に [`init`] を呼ぶ（テーマの初期化と、部品のキー操作の割り当て）。
//! テーマは [`theme`]（`md3-theme` の再公開）で設定する。
//! この crate が使う版の gpui は [`gpui`] として再公開する。feature `platform` を有効にすると
//! 同じ版の `gpui_platform` も `gpui_platform` として再公開する。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg_attr(
    not(any(
        feature = "buttons",
        feature = "containment",
        feature = "expressive",
        feature = "selection"
    )),
    expect(
        unused_macros,
        reason = "使う部品の feature がすべて無効だと展開されない"
    )
)]
macro_rules! accessible_name {
    ($(#[$extra:meta])*) => {
        gpui_headless::accessible_name_setter!(self, label, {
            self.name.set_aria_label(label);
            self
        } $(; #[$extra])*);
    };
}

#[cfg_attr(
    not(any(feature = "buttons", feature = "expressive", feature = "selection")),
    expect(
        unused_macros,
        reason = "使う部品の feature がすべて無効だと展開されない"
    )
)]
macro_rules! delegate_accessible_name {
    ($($field:tt).+ $(; #[$extra:meta])*) => {
        gpui_headless::accessible_name_setter!(self, label, {
            self.$($field).+ = self.$($field).+.aria_label(label);
            self
        } $(; #[$extra])*);
    };
}

#[cfg(feature = "buttons")]
mod anchor_tooltip;
#[cfg(feature = "buttons")]
pub mod buttons;
#[cfg(feature = "containment")]
pub mod containment;
#[cfg(feature = "expressive")]
pub mod expressive;
mod focus;
mod icon;
#[cfg(feature = "inputs")]
pub mod inputs;
#[cfg(feature = "selection")]
mod motion;
#[cfg(feature = "navigation")]
pub mod navigation;
#[cfg(feature = "overlays")]
pub mod overlays;
mod scroll;
pub mod scrollbar;
#[cfg(feature = "selection")]
pub mod selection;
mod state_layer;
mod style;
mod surface;
mod text;
mod window_size;

pub use focus::FocusRing;
/// この crate が依存する gpui（固定 rev）。利用者が版を揃えずに使えるよう再公開する。
pub use gpui;
/// この crate と同じ rev の gpui_platform。アプリの起動（`gpui_platform::application()`）に使う。
#[cfg(feature = "platform")]
pub use gpui_platform;
pub use icon::Icon;
pub use md3_icons::IconName;
pub use md3_theme as theme;
pub use scrollbar::Scrollbar;
pub use state_layer::StateLayer;
pub use style::StyledExt;
pub use surface::Surface;
pub use text::Text;
pub use window_size::WindowSizeClass;

/// テーマを初期化し、部品のキー操作を割り当てる。
///
/// コンテキストなしの同じ優先度の割り当ては後から追加したものが勝つため、
/// アプリが Tab などを割り当てる場合はその後に呼ぶ。
pub fn init(cx: &mut gpui::App) {
    md3_theme::init(cx);
    gpui_headless::init(cx);
}
