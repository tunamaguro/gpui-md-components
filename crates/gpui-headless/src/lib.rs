//! gpui 用の見た目を持たない操作基盤。
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// 部品用の名前 setter を定義する。
#[doc(hidden)]
#[macro_export]
macro_rules! accessible_name_setter {
    ($this:ident, $label:ident, $body:block $(; #[$extra:meta])*) => {
        /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
        $(#[$extra])*
        pub fn aria_label(mut $this, $label: impl Into<gpui::SharedString>) -> Self $body
    };
}

macro_rules! accessible_name {
    ($(#[$extra:meta])*) => {
        crate::accessible_name_setter!(self, label, {
            self.name.set_aria_label(label);
            self
        } $(; #[$extra])*);

        /// 読み上げる名前をまとめて指定する。
        pub fn name(mut self, name: crate::AccessibleName) -> Self {
            self.name = name;
            self
        }
    };
}

macro_rules! delegate_accessible_name {
    ($($field:tt).+) => {
        crate::accessible_name_setter!(self, label, {
            self.$($field).+ = self.$($field).+.aria_label(label);
            self
        });
    };
}

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
mod text_layout;
mod tooltip;

pub use a11y::{A11yExt, A11yNode, AccessibleName, RangeValue, supporting_status, toggled};
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
pub use navigation::{Navigation, NavigationBase, NavigationItem, NavigationOption};
pub use progress::Progress;
pub use radio_group::{RadioGroup, RadioItem, RadioOption};
pub use scroll::{NestedScroll, ScrollScope, reveal_offset, reveal_on_focus, scroll_handle};
pub use scrollbar::{
    Scrollbar, ScrollbarGeometry, ScrollbarInteraction, ScrollbarSource, ScrollbarState,
};
pub use segmented_button::{SegmentItem, SegmentOption, SegmentedButton};
pub use select::{Select, SelectState};
pub use slider::{Slider, SliderState};
pub use snackbar::{
    Snackbar, SnackbarEntry, SnackbarFront, SnackbarOwner, SnackbarPolicy, SnackbarQueue,
    SnackbarResult,
};
pub use switch::{Switch, SwitchState};
pub use tabs::{TabItem, TabOption, Tabs};
pub use text_field::{
    SearchField, TextField, TextFieldControl, TextFieldEvent, TextFieldScroll, TextFieldState,
    TextPaint, grapheme_count,
};
pub use tooltip::{DescribedTooltip, FocusTooltip, Tooltip, tooltip_origin};
