//! NavigationBar / Rail / Drawer が共有する項目と、項目の間のキー操作。

use crate::{IconName, selection::Badge};
use gpui::{App, ElementId, SharedString, Window};
use std::rc::Rc;

/// 行き先を選んだときの処理（選んだ項目の位置を受け取る）。
pub(crate) type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// ナビゲーションの行き先 1 つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationItem {
    pub(crate) icon: IconName,
    pub(crate) label: SharedString,
    name: gpui_headless::AccessibleName,
    pub(crate) disabled: bool,
    pub(crate) badge: Option<Badge>,
    badge_description: Option<SharedString>,
}

impl NavigationItem {
    /// アイコンとラベルから作る。
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            icon,
            name: gpui_headless::AccessibleName::new(label.clone()),
            label,
            disabled: false,
            badge: None,
            badge_description: None,
        }
    }

    accessible_name!();

    /// バッジを付け、文字があれば項目の説明として読み上げる。
    /// 点には [`Self::badge_description`] で読み上げ用の説明を付けること。
    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    /// バッジの文字より優先する読み上げ用の説明を指定する。
    pub fn badge_description(mut self, text: impl Into<SharedString>) -> Self {
        self.badge_description = Some(text.into());
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
    for item in items {
        let mut option = gpui_headless::NavigationOption::new(item.label.clone(), item.disabled)
            .name(item.name.clone());
        if let Some(description) = item
            .badge
            .as_ref()
            .and_then(|badge| item.badge_description.as_ref().or_else(|| badge.label()))
        {
            option = option.description(description.clone());
        }
        nav = nav.destination(option);
    }
    if let Some(on_change) = on_change {
        nav = nav.on_change(move |i, w, cx| on_change(i, w, cx));
    }
    nav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_descriptions_reach_tab_nodes() {
        let items = [
            NavigationItem::new(IconName::Inbox, "受信").badge(Badge::new("3")),
            NavigationItem::new(IconName::Inbox, "受信")
                .badge(Badge::new("3"))
                .badge_description("未読 3 件")
                .aria_label("受信トレイ"),
            NavigationItem::new(IconName::Inbox, "受信").badge(Badge::dot()),
            NavigationItem::new(IconName::Inbox, "受信")
                .badge(Badge::dot())
                .badge_description("新着あり"),
        ];
        let nav = navigation("nav".into(), None, &items, None, None, false);
        let node = |index| nav.item_a11y_node(index).expect("項目がある");
        assert_eq!(node(0).description.as_deref(), Some("3"));
        assert_eq!(node(1).description.as_deref(), Some("未読 3 件"));
        assert_eq!(node(1).label.as_deref(), Some("受信トレイ"));
        assert_eq!(node(2).description, None);
        assert_eq!(node(3).description.as_deref(), Some("新着あり"));
    }
}
