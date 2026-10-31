//! ButtonGroup（押された項目が広がり、隣が縮むボタンの並び）。

use gpui::{
    App, ElementId, Entity, IntoElement, Pixels, RenderOnce, TextRun, Window, prelude::*, px,
};
use md3_theme::Md3ThemeExt;
use md3_theme::md3_tokens::component::{
    button_group_small, button_small, connected_button_group_small, filled_tonal_button,
};

use super::press::press_state;
use super::toggle_button::{GroupSlot, ToggleButton};

/// [`ToggleButton`] を横に並べたグループ（M3 Expressive）。
///
/// 項目を押すと、その項目の幅が 15% 広がり、隣の項目が縮む（テーマのモーションスキームのバネで動く）。
/// [`ButtonGroup::connected`] にすると項目の間を詰め、両端の外側だけを丸くして、内側の角を押下・選択で変える。
///
/// 各項目は独立したトグルボタンで、Tab で 1 つずつ止まる。選択を 1 つに限るときは、利用者が
/// `on_change` で他の項目の選択を外す（読み上げで「1 つを選ぶ」と伝えたいときは SegmentedButton を使う）。
///
/// ```ignore
/// ButtonGroup::new("align").aria_label("配置")
///     .connected(true)
///     .child(ToggleButton::new("left", "左").checked(align == 0).on_change(…))
///     .child(ToggleButton::new("center", "中央").checked(align == 1).on_change(…))
/// ```
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    name: gpui_headless::AccessibleName,
    connected: bool,
    items: Vec<ToggleButton>,
}

impl ButtonGroup {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: gpui_headless::AccessibleName::default(),
            connected: false,
            items: Vec::new(),
        }
    }

    accessible_name!();

    /// 項目の間を詰めてつなげる（connected button group）。
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// 項目を加える。
    pub fn child(mut self, item: ToggleButton) -> Self {
        self.items.push(item);
        self
    }

    /// 項目をまとめて加える。
    pub fn children(mut self, items: impl IntoIterator<Item = ToggleButton>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.items.len();
        let gap = px(if self.connected {
            connected_button_group_small::BETWEEN_SPACE
        } else {
            button_group_small::BETWEEN_SPACE
        });

        // 押されていないときの各項目の幅をラベルの文字幅から求める（幅の変化の基準）。
        let label_style = cx.md3().text_style(filled_tonal_button::LABEL_TEXT_FONT);
        let mut text_style = window.text_style();
        text_style.font_weight = label_style.font_weight;
        if let Some(family) = &label_style.font_family {
            text_style.font_family = family.clone();
        }
        let font = text_style.font();
        let naturals: Vec<Pixels> = self
            .items
            .iter()
            .map(|item| {
                let run = TextRun {
                    len: item.label.len(),
                    font: font.clone(),
                    color: text_style.color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window.text_system().shape_line(
                    item.label.clone(),
                    label_style.font_size,
                    &[run],
                    None,
                );
                item.natural_width(line.width.ceil())
            })
            .collect();

        // 最後に押された項目。押下を離した後も、幅が戻るまでその隣を伸び縮みさせる。
        let presses: Vec<_> = self
            .items
            .iter()
            .map(|item| press_state(&item.id, window, cx))
            .collect();
        let last: Entity<Option<usize>> = window.use_keyed_state(
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "last".into()),
            cx,
            |_, _| None,
        );
        if let Some(index) = presses.iter().position(|p| p.read(cx).is_pressed()) {
            last.update(cx, |last, _| *last = Some(index));
        }
        let last = *last.read(cx);

        let total = naturals.iter().fold(px(0.0), |sum, width| sum + *width)
            + gap * count.saturating_sub(1) as f32;
        let padding = px(button_small::LEADING_SPACE + button_small::TRAILING_SPACE);
        let compressible: Vec<Pixels> = naturals
            .iter()
            .map(|&n| compressible_width(n, padding))
            .collect();
        let mut group = gpui_headless::ButtonGroup::new(self.id.clone());
        if let Some(label) = self.name.resolve() {
            group = group.aria_label(label);
        }
        let mut items = Vec::with_capacity(count);
        for (index, ((mut item, natural), press)) in self
            .items
            .into_iter()
            .zip(naturals)
            .zip(presses)
            .enumerate()
        {
            item.press = Some(press.clone());
            item.slot = Some(GroupSlot {
                natural,
                flexible: last.is_some_and(|last| last.abs_diff(index) == 1),
                min_width: natural - compressible[index],
                capacity: neighbor_capacity(&compressible, index),
                connected: self.connected.then_some((index == 0, index + 1 == count)),
            });
            let (inner, paint) = item.prepare(&press);
            group = group.item(inner);
            items.push((paint, press));
        }
        let mut items = items.into_iter();
        group
            .build(window, cx, move |item, _, cx| {
                let (button, press) = items.next().expect("item counts match");
                button.paint(item.base, item.state, press, cx)
            })
            .flex()
            .flex_none()
            .items_center()
            .gap(gap)
            .w(total)
    }
}

/// 項目が縮められる幅（左右の余白の分まで。Compose の `compressionLimit` の既定と同じ）。
fn compressible_width(natural: Pixels, padding: Pixels) -> Pixels {
    padding.min(natural).max(px(0.0))
}

/// `index` の項目が押されたときに、両隣が縮んで吸収できる幅の合計。
fn neighbor_capacity(compressible: &[Pixels], index: usize) -> Pixels {
    let before = index
        .checked_sub(1)
        .and_then(|i| compressible.get(i))
        .copied()
        .unwrap_or(px(0.0));
    let after = compressible.get(index + 1).copied().unwrap_or(px(0.0));
    before + after
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_counts_both_neighbors_and_none_for_a_single_item() {
        let compressible = [px(32.0), px(32.0), px(32.0)];
        assert_eq!(neighbor_capacity(&compressible, 0), px(32.0));
        assert_eq!(neighbor_capacity(&compressible, 1), px(64.0));
        assert_eq!(neighbor_capacity(&compressible, 2), px(32.0));
        assert_eq!(neighbor_capacity(&[px(32.0)], 0), px(0.0));
    }
}
