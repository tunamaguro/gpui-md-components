//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント。
//!
//! 基礎部品（[`Surface`], [`StateLayer`], [`FocusRing`], [`Icon`], [`Text`]）と各コンポーネントを提供する。
//! コンポーネント群は feature（`buttons`, `selection`, `inputs`, `overlays`, `navigation`, `containment`）で分割する。
//!
//! テーマは [`theme`]（`md3-theme` の再公開）で設定する。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod focus;
mod icon;
mod state_layer;
mod style;
mod surface;
mod text;

pub use focus::{FocusRing, focus_handle, is_focus_visible};
pub use icon::Icon;
pub use md3_icons::IconName;
pub use md3_theme as theme;
pub use state_layer::StateLayer;
pub use style::StyledExt;
pub use surface::Surface;
pub use text::Text;
