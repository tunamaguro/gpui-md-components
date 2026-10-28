//! オーバーレイの部品（feature `overlays`）。
//!
//! Dialog・シート・メニューは開いている間だけ描く。開いたときに中へフォーカスを移し、
//! 閉じたら開く前の要素へ戻す。キー操作は [`crate::init`] で割り当てる。

mod dialog;
mod layer;
mod menu;
mod sheet;
mod snackbar;
mod tooltip;

pub use dialog::{Dialog, DialogAction};
pub(crate) use menu::MenuKind;
pub use menu::{Menu, MenuItem};
pub use sheet::{BottomSheet, SideSheet, SideSheetKind};
pub use snackbar::{Snackbar, SnackbarDuration, SnackbarHost, SnackbarMessage};
pub use tooltip::{PlainTooltip, RichTooltip, TooltipExt};
