//! NavigationBar / Rail / Drawer が共有する項目と、項目の間のキー操作。

use crate::IconName;
use gpui::{App, ElementId, SharedString, Window};
use std::rc::Rc;

/// 行き先を選んだときの処理（選んだ項目の位置を受け取る）。
pub(crate) type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// ナビゲーションの行き先 1 つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationItem {
    pub(crate) icon: IconName,
    pub(crate) label: SharedString,
    aria_label: Option<Option<SharedString>>,
    pub(crate) disabled: bool,
}

impl NavigationItem {
    /// アイコンとラベルから作る。
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            label: label.into(),
            aria_label: None,
            disabled: false,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.aria_label = Some((!label.is_empty()).then_some(label));
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub(crate) fn navigation(
    id: ElementId,
    label: Option<SharedString>,
    items: &[NavigationItem],
    selected: Option<usize>,
    on_change: Option<ChangeHandler>,
    vertical: bool,
) -> gpui_headless::Navigation {
    let mut nav = gpui_headless::Navigation::new(id)
        .selected(selected)
        .vertical(vertical)
        .reveal_margin(crate::focus::reveal_margin())
        .group_prefix("md3");
    if let Some(label) = label {
        nav = nav.aria_label(label);
    }
    for (index, item) in items.iter().enumerate() {
        nav = nav.item(item.label.clone(), item.disabled);
        if let Some(label) = item.aria_label.clone() {
            nav = nav.item_aria_label(index, label.unwrap_or_default());
        }
    }
    if let Some(on_change) = on_change {
        nav = nav.on_change(move |i, w, cx| on_change(i, w, cx));
    }
    nav
}
