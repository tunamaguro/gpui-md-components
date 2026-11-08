//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "button")]
mod button;

#[cfg(feature = "button")]
pub use button::{Button, ButtonSize, ButtonVariant};
pub use gpui;
pub use gpui_platform;

/// この crate の部品が使うキー割り当てと操作を登録する。アプリの起動時に、ウィンドウを開く前に 1 回呼ぶ
pub fn init(cx: &mut gpui::App) {
    gpui_headless::init(cx);
}
