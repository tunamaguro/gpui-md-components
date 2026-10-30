//! gpui 用の見た目を持たない操作基盤。
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod a11y;
mod app_bar;
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
mod list;
mod menu;
mod navigation;
/// オーバーレイのフォーカスと重ね描き。
pub mod overlay;
/// 押下状態の追跡。
pub mod press;
mod progress;
mod radio_group;
mod scroll;
mod scrollbar;
mod segmented_button;
mod select;
mod slider;
mod snackbar;
mod switch;
mod tabs;
mod text_field;
mod tooltip;

pub use a11y::{A11yExt, A11yNode, RangeValue, toggled};
pub use app_bar::AppBar;
pub use button::Button;
pub use button_group::{ButtonGroup, ButtonItem};
pub use card::{Card, CardBase};
pub use checkbox::{Checkbox, CheckboxState};
pub use chip::Chip;
pub use control::{ControlState, control, step};
pub use dialog::Dialog;
pub use focus::{VisibleFocus, clear_focus_visible, focus_handle, is_focus_visible};
pub use keys::init;
pub use list::{LazyList, List, ListBase, ListItem, ListRow};
pub use menu::{Menu, MenuItem, MenuKind, MenuPopup, MenuRow};
pub use navigation::{Navigation, NavigationBase, NavigationItem};
pub use progress::Progress;
pub use radio_group::{RadioGroup, RadioItem};
pub use scroll::{NestedScroll, ScrollScope, reveal_offset, reveal_on_focus, scroll_handle};
pub use scrollbar::{
    Scrollbar, ScrollbarGeometry, ScrollbarInteraction, ScrollbarSource, ScrollbarState,
};
pub use segmented_button::{SegmentItem, SegmentedButton};
pub use select::{Select, SelectState};
pub use slider::{Slider, SliderState};
pub use snackbar::{
    Snackbar, SnackbarEntry, SnackbarFront, SnackbarOwner, SnackbarPolicy, SnackbarQueue,
    SnackbarResult,
};
pub use switch::{Switch, SwitchState};
pub use tabs::{TabItem, Tabs};
pub use text_field::{
    SearchField, TextField, TextFieldControl, TextFieldEvent, TextFieldState, TextPaint,
    grapheme_count,
};
pub use tooltip::{DescribedTooltip, Tooltip};
