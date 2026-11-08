//! Material Design 3 の仕様に基づくテーマ
//!
//! [`ThemeSettings`] (利用者の設定) と OS の外観から [`Theme`] を解決し、gpui の `Global` として
//! アプリ全体で共有する。[`Theme`] は `md3-tokens` のトークンが参照する色・書体・形・モーションのキーを
//! gpui の値に変換する
//!
//! ```ignore
//! use md3_theme::ThemeExt;
//!
//! cx.update_md3_theme(|settings| {
//!     settings.color.source = Argb::from_rgb(0x00, 0x6A, 0x60);
//! })?;
//!
//! // ウィンドウを開いたときに 1 回
//! cx.init_md3_theme(window);
//!
//! // 描画で
//! div().bg(cx.md3_theme().color(filled_button::CONTAINER_COLOR))
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod global;
mod settings;
mod theme;

pub use global::{SharedTheme, ThemeExt};
pub use settings::{
    AppearanceMode, ColorSettings, CustomColor, Fonts, PaletteOverrides, RoleOverrides, TextScale,
    ThemeSettings,
};
pub use theme::{ContrastIssue, CustomColorRoles, Theme, dp};

pub use md3_color;
pub use md3_tokens;
