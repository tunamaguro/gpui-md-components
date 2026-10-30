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
mod dialog;
mod focus;
/// キー操作と割り当て。
pub mod keys;
mod menu;
mod navigation;
/// オーバーレイのフォーカスと重ね描き。
pub mod overlay;
/// 押下状態の追跡。
pub mod press;
mod radio_group;
mod scroll;
mod segmented_button;
mod select;
mod slider;
mod snackbar;
mod switch;
mod tabs;
mod tooltip;

pub use a11y::{A11yExt, A11yNode, RangeValue, toggled};
pub use button::Button;
pub use button_group::{ButtonGroup, ButtonItem};
pub use card::{Card, CardBase};
pub use checkbox::{Checkbox, CheckboxState};
pub use chip::Chip;
pub use control::{ControlState, control, step};
pub use dialog::Dialog;
pub use focus::{VisibleFocus, clear_focus_visible, focus_handle, is_focus_visible};
pub use keys::init;
pub use menu::{Menu, MenuItem, MenuKind, MenuPopup, MenuRow};
pub use navigation::{Navigation, NavigationBase, NavigationItem};
pub use radio_group::{RadioGroup, RadioItem};
pub use scroll::{NestedScroll, ScrollScope, reveal_offset, reveal_on_focus, scroll_handle};
pub use segmented_button::{SegmentItem, SegmentedButton};
pub use select::{Select, SelectState};
pub use slider::{Slider, SliderState};
pub use snackbar::{
    Snackbar, SnackbarEntry, SnackbarFront, SnackbarOwner, SnackbarPolicy, SnackbarQueue,
    SnackbarResult,
};
pub use switch::{Switch, SwitchState};
pub use tabs::{TabItem, Tabs};
pub use tooltip::{DescribedTooltip, Tooltip};
