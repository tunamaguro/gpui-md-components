//! gpui 用の見た目を持たない操作基盤。
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod a11y;
mod control;
mod focus;
/// キー操作と割り当て。
pub mod keys;
mod press;
mod scroll;

pub use a11y::{A11yExt, A11yNode, RangeValue, toggled};
pub use control::{ControlState, control, step};
pub use focus::{VisibleFocus, clear_focus_visible, focus_handle, is_focus_visible};
pub use keys::init;
pub use press::{Press, press_state, track_press};
pub use scroll::{NestedScroll, ScrollScope, reveal_offset, reveal_on_focus, scroll_handle};
