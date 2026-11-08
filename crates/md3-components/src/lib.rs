//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(feature = "button")]
mod button;
// 形状を使う部品の feature がすべて無効なときに未使用にならないよう、それらの feature で囲む
#[cfg(any(feature = "button", feature = "switch"))]
mod shape;
#[cfg(feature = "switch")]
mod switch;

#[cfg(feature = "button")]
pub use button::{Button, ButtonColors, ButtonShape, ButtonSize, ButtonVariant};
pub use gpui;
pub use gpui_platform;
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
