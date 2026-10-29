//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント。
//!
//! 基礎部品（[`Surface`], [`StateLayer`], [`FocusRing`], [`Icon`], [`Text`]）と各コンポーネントを提供する。
//! コンポーネント群は feature（`buttons`, `selection`, `inputs`, `overlays`, `navigation`, `containment`）で分割する。
//!
//! 起動時に [`init`] を呼ぶ（テーマの初期化と、部品のキー操作の割り当て）。
//! テーマは [`theme`]（`md3-theme` の再公開）で設定する。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod a11y;
#[cfg(feature = "buttons")]
pub mod buttons;
#[cfg(feature = "containment")]
pub mod containment;
mod control;
#[cfg(feature = "expressive")]
pub mod expressive;
mod focus;
mod icon;
#[cfg(feature = "inputs")]
pub mod inputs;
pub mod keys;
#[cfg(feature = "selection")]
mod motion;
#[cfg(feature = "navigation")]
pub mod navigation;
#[cfg(feature = "overlays")]
pub mod overlays;
#[cfg(any(feature = "containment", feature = "overlays", feature = "navigation"))]
mod scroll;
pub mod scrollbar;
#[cfg(feature = "selection")]
pub mod selection;
mod state_layer;
mod style;
mod surface;
mod text;
mod window_size;

pub use a11y::{A11yNode, RangeValue, toggled};
pub use control::ForcedState;
pub use focus::{FocusRing, focus_handle, is_focus_visible};
pub use icon::Icon;
pub use keys::init;
pub use md3_icons::IconName;
pub use md3_theme as theme;
pub use scrollbar::{Scrollbar, ScrollbarInteraction, ScrollbarSource};
pub use state_layer::StateLayer;
pub use style::StyledExt;
pub use surface::Surface;
pub use text::Text;
pub use window_size::WindowSizeClass;
