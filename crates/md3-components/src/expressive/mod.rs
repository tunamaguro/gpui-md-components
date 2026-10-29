//! M3 Expressive の部品（ToggleButton、ButtonGroup）。
//!
//! 押下・選択に合わせて形（角丸）と幅がバネで変わる。バネはテーマのモーションスキーム
//! （[`crate::theme::MotionScheme`]）から取る。gpui の `App::reduce_motion` が `true` なら、
//! バネは動かずに最終の形で描かれ、押下の形を見せるための待ちもしない。

mod button_group;
mod press;
mod toggle_button;

pub use button_group::ButtonGroup;
pub use toggle_button::ToggleButton;
