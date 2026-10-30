//! gpui 用の見た目を持たない操作基盤。
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod a11y;
mod button;
mod button_group;
mod card;
mod checkbox;
mod chip;
mod control;
mod focus;
/// キー操作と割り当て。
pub mod keys;
mod navigation;
/// 押下状態の追跡。
pub mod press;
mod radio_group;
mod scroll;
mod segmented_button;
mod slider;
mod switch;
mod tabs;

pub use a11y::{A11yExt, A11yNode, RangeValue, toggled};
pub use button::Button;
pub use button_group::{ButtonGroup, ButtonItem};
pub use card::{Card, CardBase};
pub use checkbox::{Checkbox, CheckboxState};
pub use chip::Chip;
pub use control::{ControlState, control, step};
pub use focus::{VisibleFocus, clear_focus_visible, focus_handle, is_focus_visible};
pub use keys::init;
pub use navigation::{Navigation, NavigationBase, NavigationItem};
pub use radio_group::{RadioGroup, RadioItem};
pub use scroll::{NestedScroll, ScrollScope, reveal_offset, reveal_on_focus, scroll_handle};
pub use segmented_button::{SegmentItem, SegmentedButton};
pub use slider::{Slider, SliderState};
pub use switch::{Switch, SwitchState};
pub use tabs::{TabItem, Tabs};
