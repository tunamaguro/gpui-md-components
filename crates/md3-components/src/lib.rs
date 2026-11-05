//! Material Design 3 の仕様に基づく gpui 用 UI コンポーネント
//!
//! 部品は [`md3_theme`] のテーマで色・形・書体を決めるため、描画する前に
//! `md3_theme::ThemeExt::init_md3_theme` か `update_md3_theme` でテーマを用意すること
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(any(feature = "button", feature = "icon-button"))]
pub mod button;
#[cfg(feature = "icon-button")]
pub mod icon_button;
#[cfg(any(feature = "button", feature = "icon-button"))]
mod indication;

pub use gpui;
#[cfg(feature = "platform")]
pub use gpui_platform;
pub use md3_icons::IconName;

#[cfg(any(feature = "button", feature = "icon-button"))]
use gpui::{Hsla, Pixels, Styled as _, Svg, svg};
#[cfg(any(feature = "button", feature = "icon-button"))]
use md3_theme::md3_tokens::Dp;

/// 操作できる範囲の最小の幅と高さ。Compose の `InteractiveComponentSize.kt` の
/// `minimumInteractiveComponentSize` (48.dp) による
#[cfg(any(feature = "button", feature = "icon-button"))]
const MIN_INTERACTIVE_SIZE: Dp = Dp(48.0);

/// 大きさ `size` の正方形に `color` で描くアイコン
#[cfg(any(feature = "button", feature = "icon-button"))]
fn icon(name: IconName, size: Pixels, color: Hsla) -> Svg {
    svg()
        .data(name.svg())
        .flex_none()
        .size(size)
        .text_color(color)
}

#[cfg(test)]
mod test_support {
    use gpui::{KeyDownEvent, KeyUpEvent, Keystroke, TestAppContext, VisualTestContext};
    use md3_theme::ThemeExt as _;

    /// 部品が読むテーマを既定の設定で用意する
    pub(crate) fn init_theme(cx: &mut TestAppContext) {
        cx.update(|cx| cx.update_md3_theme(|_| {}))
            .expect("既定の設定は解決できる");
    }

    /// `key` を押して離す
    pub(crate) fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("キーの表記が正しいこと");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    /// Tab で次の Tab stop へフォーカスを移す
    pub(crate) fn focus_next(cx: &mut VisualTestContext) {
        cx.update(|window, cx| window.focus_next(cx));
        cx.run_until_parked();
    }
}
