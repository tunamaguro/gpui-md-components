//! ナビゲーションの部品（feature `navigation`）。
//!
//! 行き先の並び（NavigationBar / Rail / Drawer）は Tab では選択中の項目で 1 回だけ止まり、
//! 中は矢印キーでフォーカスを移して Enter / Space で選ぶ。Tabs は矢印キーで選択も移る。

mod app_bar;
mod bar;
mod drawer;
mod item;
mod suite;
mod tabs;

pub use app_bar::{TopAppBar, TopAppBarVariant};
pub use bar::{NavigationBar, NavigationRail};
pub use drawer::{NavigationDrawer, NavigationDrawerKind};
pub use item::NavigationItem;
pub use suite::{NavigationSuite, NavigationSuiteType};
pub use tabs::{Tab, Tabs, TabsVariant};
