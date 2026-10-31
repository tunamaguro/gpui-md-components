//! NavigationSuite（ウィンドウの幅に合わせて NavigationBar / Rail / Drawer を切り替える）。

use gpui::{
    AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Window, div, prelude::*,
};

use super::bar::destination_builders;
use super::item::{ChangeHandler, NavigationItem};
use super::{NavigationBar, NavigationDrawer, NavigationRail};
use crate::WindowSizeClass;

/// ナビゲーションの形。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationSuiteType {
    /// 下端の NavigationBar。
    Bar,
    /// 左端の NavigationRail。
    Rail,
    /// 左端の Standard の NavigationDrawer。
    Drawer,
}

impl NavigationSuiteType {
    /// 幅のクラスから形を決める。Compact は Bar、Medium は Rail、Expanded 以上は Drawer。
    pub fn for_size_class(class: WindowSizeClass) -> Self {
        match class {
            WindowSizeClass::Compact => Self::Bar,
            WindowSizeClass::Medium => Self::Rail,
            WindowSizeClass::Expanded | WindowSizeClass::Large | WindowSizeClass::ExtraLarge => {
                Self::Drawer
            }
        }
    }
}

/// 行き先のナビゲーションと内容を並べる入れ物。
///
/// 既定ではウィンドウの幅のクラス（[`WindowSizeClass::of`]）から形を選ぶ。
/// 内容は子要素で渡す。選択と行き先は Bar / Rail / Drawer で共通で、形が変わっても保たれる。
#[derive(IntoElement)]
pub struct NavigationSuite {
    id: ElementId,
    name: gpui_headless::AccessibleName,
    items: Vec<NavigationItem>,
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    layout: Option<NavigationSuiteType>,
    header: Option<AnyElement>,
    content: Vec<AnyElement>,
}

impl NavigationSuite {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: gpui_headless::AccessibleName::default(),
            items: Vec::new(),
            selected: None,
            on_change: None,
            layout: None,
            header: None,
            content: Vec::new(),
        }
    }

    accessible_name!();

    /// 形を固定する（ウィンドウの幅で決めない）。
    pub fn layout(mut self, layout: NavigationSuiteType) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Rail と Drawer の項目の上に置く要素（FAB など。Bar では出さない）。
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    destination_builders!();
}

impl ParentElement for NavigationSuite {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements);
    }
}

impl RenderOnce for NavigationSuite {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let layout = self
            .layout
            .unwrap_or_else(|| NavigationSuiteType::for_size_class(WindowSizeClass::of(window)));
        let content = div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .children(self.content);
        // 形が変わってもフォーカスと項目の ID が同じになるよう、ナビゲーションの ID は共通にする。
        let nav_id = ElementId::NamedChild(self.id.clone().into(), "navigation".into());
        let container = div().id(self.id).size_full().flex();
        match layout {
            NavigationSuiteType::Bar => {
                let mut bar = NavigationBar::new(nav_id)
                    .items(self.items)
                    .selected(self.selected);
                if let Some(label) = self.name.resolve() {
                    bar = bar.aria_label(label);
                }
                if let Some(on_change) = self.on_change {
                    bar = bar.on_change(move |index, window, cx| on_change(index, window, cx));
                }
                container.flex_col().child(content).child(bar)
            }
            NavigationSuiteType::Rail => {
                let mut rail = NavigationRail::new(nav_id)
                    .items(self.items)
                    .selected(self.selected);
                if let Some(label) = self.name.resolve() {
                    rail = rail.aria_label(label);
                }
                if let Some(on_change) = self.on_change {
                    rail = rail.on_change(move |index, window, cx| on_change(index, window, cx));
                }
                if let Some(header) = self.header {
                    rail = rail.header(header);
                }
                container.child(rail).child(content)
            }
            NavigationSuiteType::Drawer => {
                let mut drawer = NavigationDrawer::new(nav_id)
                    .items(self.items)
                    .selected(self.selected);
                if let Some(label) = self.name.resolve() {
                    drawer = drawer.aria_label(label);
                }
                if let Some(on_change) = self.on_change {
                    drawer =
                        drawer.on_change(move |index, window, cx| on_change(index, window, cx));
                }
                if let Some(header) = self.header {
                    drawer = drawer.child(header);
                }
                container.child(drawer).child(content)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_follows_the_window_size_class() {
        use NavigationSuiteType::*;
        assert_eq!(
            NavigationSuiteType::for_size_class(WindowSizeClass::Compact),
            Bar
        );
        assert_eq!(
            NavigationSuiteType::for_size_class(WindowSizeClass::Medium),
            Rail
        );
        assert_eq!(
            NavigationSuiteType::for_size_class(WindowSizeClass::Expanded),
            Drawer
        );
        assert_eq!(
            NavigationSuiteType::for_size_class(WindowSizeClass::ExtraLarge),
            Drawer
        );
    }
}
