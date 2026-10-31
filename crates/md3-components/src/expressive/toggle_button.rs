//! ToggleButton（押下と選択で形が変わるトグルボタン）。

use gpui::{
    AnimationExt, App, ClickEvent, Corners, ElementId, Entity, IntoElement, Pixels, RenderOnce,
    SharedString, SpringAnimation, Window, prelude::*, px,
};
use md3_theme::md3_tokens::component::{
    button_small as tokens, connected_button_group_small as connected, filled_button,
    filled_tonal_button,
};
use md3_theme::md3_tokens::layout;
use md3_theme::md3_tokens::shape::Shape;
use md3_theme::{CornerRadius, Md3ThemeExt, MotionToken, Theme};

use super::press::{Press, press_state, track_press};
use crate::{FocusRing, Icon, IconName, StateLayer, StyledExt};

/// 押された項目が広がる割合（Compose の `ButtonGroupDefaults.ExpandedRatio`）。
pub(crate) const EXPANDED_RATIO: f32 = 0.15;

/// ButtonGroup の中での並び（角の形と幅の変化を決める）。
#[derive(Clone, Copy, Debug)]
pub(crate) struct GroupSlot {
    /// 押されていないときの幅。
    pub natural: Pixels,
    /// 隣の項目が押されたときに縮んでよいか（押された項目の隣だけ伸び縮みする）。
    pub flexible: bool,
    /// 縮められる最小の幅。
    pub min_width: Pixels,
    /// 押されたときに隣の項目が縮んで吸収できる幅の合計（広がりの上限）。
    pub capacity: Pixels,
    /// connected の並びの両端か（左端、右端）。connected でなければ `None`。
    pub connected: Option<(bool, bool)>,
}

/// 押下と選択で形が変わるトグルボタン（M3 Expressive）。
///
/// 選択されていないときは丸い形、選択中は角の丸い四角、押下中はさらに角の小さい四角に、
/// テーマのモーションスキームのバネで変わる。
///
/// ```ignore
/// ToggleButton::new("bold", "太字").checked(bold).on_change(|checked, _, cx| { … })
/// ```
#[derive(IntoElement)]
pub struct ToggleButton {
    pub(crate) id: ElementId,
    pub(crate) label: SharedString,
    inner: gpui_headless::Button,
    pub(crate) icon: Option<IconName>,
    pub(crate) press: Option<Entity<Press>>,
    pub(crate) slot: Option<GroupSlot>,
}

impl ToggleButton {
    /// ID とラベルから作る。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        let label = label.into();
        Self {
            inner: gpui_headless::Button::new(id.clone(), label.clone())
                .toggled(Some(false))
                .group(format!("md3-{id}"))
                .reveal_margin(crate::focus::reveal_margin()),
            id,
            label,
            icon: None,
            press: None,
            slot: None,
        }
    }

    /// 読み上げる名前を指定する。表示文字を優先し、表示ラベルがある場合は名前に含める（WCAG 2.5.3）。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }

    /// 選択中か。
    pub fn checked(mut self, checked: bool) -> Self {
        self.inner = self.inner.toggled(Some(checked));
        self
    }

    /// ラベルの前にアイコンを置く。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    /// 押されたときの処理（クリック、Enter、Space）。引数は押された後の選択の状態。
    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_toggle(handler);
        self
    }

    /// 押されていないときの内容の幅（左右の余白・アイコン・ラベル）。
    pub(crate) fn natural_width(&self, label_width: Pixels) -> Pixels {
        let icon = if self.icon.is_some() {
            px(tokens::ICON_SIZE + tokens::ICON_LABEL_SPACE)
        } else {
            px(0.0)
        };
        (px(tokens::LEADING_SPACE + tokens::TRAILING_SPACE) + icon + label_width)
            .max(px(layout::BUTTON_MIN_WIDTH))
    }
}

/// 形の変化の目標（角丸の半径）。connected の並びでは内側の角の半径。
pub(crate) fn target_radius(
    theme: &Theme,
    height: Pixels,
    pressed: bool,
    checked: bool,
    connected: bool,
) -> Pixels {
    let full = height / 2.0;
    let radius = |shape: Shape| match theme.corner(shape) {
        CornerRadius::Fixed(radius) => radius,
        CornerRadius::Full => full,
    };
    if connected {
        if pressed {
            radius(connected::PRESSED_INNER_CORNER_CORNER_SIZE)
        } else if checked {
            height * (connected::SELECTED_INNER_CORNER_CORNER_SIZE_PERCENT / 100.0)
        } else {
            radius(connected::INNER_CORNER_CORNER_SIZE)
        }
    } else if pressed {
        radius(tokens::PRESSED_CONTAINER_SHAPE)
    } else if checked {
        radius(tokens::SELECTED_CONTAINER_SHAPE_SQUARE)
    } else {
        radius(tokens::CONTAINER_SHAPE_ROUND)
    }
}

/// 押された項目が広がる幅。`expansion` はバネの値（押下で 1、行き過ぎて 1 を超えることがある）。
///
/// 広がりは幅の 15% だが、隣の項目が縮んで吸収できる幅（`capacity`）を超えるとグループの幅から
/// はみ出すため、バネの行き過ぎを含めて `capacity` までに抑える（隣の無い 1 項目のグループは広がらない）。
/// 戻りの行き過ぎで負になるときは狭くなるだけで、隣が伸びて埋めるので抑えない。
pub(crate) fn expanded_width(slot: &GroupSlot, expansion: f32) -> Pixels {
    let extra = (slot.natural * EXPANDED_RATIO).min(slot.capacity) * expansion;
    slot.natural + extra.min(slot.capacity)
}

/// 内側の角の半径 `inner` から 4 つの角を作る。connected の並びの両端は外側の角を丸いままにする。
fn corners_of(inner: Pixels, full: Pixels, connected: Option<(bool, bool)>) -> Corners<Pixels> {
    let inner = inner.max(px(0.0)).min(full);
    let (left, right) = match connected {
        Some((first, last)) => (
            if first { full } else { inner },
            if last { full } else { inner },
        ),
        None => (inner, inner),
    };
    Corners {
        top_left: left,
        bottom_left: left,
        top_right: right,
        bottom_right: right,
    }
}

pub(crate) struct ToggleButtonPaint {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    slot: Option<GroupSlot>,
    checked: bool,
}

impl ToggleButton {
    pub(crate) fn prepare(
        self,
        press: &Entity<Press>,
    ) -> (gpui_headless::Button, ToggleButtonPaint) {
        let checked = self.inner.toggled_state().unwrap_or(false);
        let press = press.clone();
        let inner = self.inner.before_click(move |event, _, cx| {
            if matches!(event, ClickEvent::Keyboard(_)) {
                let hold = super::press::hold(cx);
                press.update(cx, |press, cx| press.pulse(hold, cx));
            }
        });
        (
            inner,
            ToggleButtonPaint {
                id: self.id,
                label: self.label,
                icon: self.icon,
                slot: self.slot,
                checked,
            },
        )
    }
}

impl ToggleButtonPaint {
    pub(crate) fn paint(
        self,
        base: gpui::Stateful<gpui::Div>,
        control: gpui_headless::ControlState,
        press: Entity<Press>,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let pressed = !control.disabled && press.read(cx).is_pressed();
        let theme = cx.md3();
        let height = theme.dense(tokens::CONTAINER_HEIGHT);
        let full = height / 2.0;
        let connected = self.slot.and_then(|slot| slot.connected);
        let target = target_radius(theme, height, pressed, self.checked, connected.is_some());
        let spring = theme.spring(MotionToken::FastSpatial);

        let (container, content_role) = if self.checked {
            (
                filled_button::SELECTED_CONTAINER_COLOR,
                filled_button::SELECTED_PRESSED_LABEL_TEXT_COLOR,
            )
        } else {
            (
                filled_button::UNSELECTED_CONTAINER_COLOR,
                filled_button::UNSELECTED_PRESSED_LABEL_TEXT_COLOR,
            )
        };
        let (container, content) = if control.disabled {
            (
                theme
                    .color(filled_button::DISABLED_CONTAINER_COLOR)
                    .opacity(filled_button::DISABLED_CONTAINER_OPACITY),
                theme
                    .color(filled_button::DISABLED_LABEL_TEXT_COLOR)
                    .opacity(filled_button::DISABLED_LABEL_TEXT_OPACITY),
            )
        } else {
            (theme.color(container), theme.color(content_role))
        };
        let mut button = base
            .min_h(height)
            .min_w(px(layout::BUTTON_MIN_WIDTH))
            .pl(px(tokens::LEADING_SPACE))
            .pr(px(tokens::TRAILING_SPACE))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(tokens::ICON_LABEL_SPACE))
            .whitespace_nowrap()
            .bg(container)
            .text_color(content)
            .type_style(&theme.text_style(filled_tonal_button::LABEL_TEXT_FONT))
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(px(tokens::ICON_SIZE)).tint(content))
            })
            .child(self.label.clone());
        if !control.disabled {
            button = track_press(button, &press);
        }

        let layer_id = ElementId::from(SharedString::from(format!("{}-layer", control.group)));
        let group = control.group.clone();
        let focus_visible = control.focus_visible;
        let disabled = control.disabled;
        let shape_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "shape".into());
        let shaped = button.with_spring(
            shape_id,
            SpringAnimation::new(spring).to(target),
            move |this, radius| {
                let corners = corners_of(radius, full, connected);
                this.corners(corners)
                    .child(
                        StateLayer::new(layer_id, group, content_role)
                            .corners(corners)
                            .focused(focus_visible)
                            .disabled(disabled),
                    )
                    .child(
                        FocusRing::new(CornerRadius::Full)
                            .corners(corners)
                            .visible(focus_visible),
                    )
            },
        );

        let Some(slot) = self.slot else {
            return shaped.into_any_element();
        };
        // ButtonGroup の中: 押されると幅が広がり、押された項目の隣が縮む（Compose の ButtonGroup）。
        let width_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "width".into());
        let expansion = if pressed { 1.0_f32 } else { 0.0 };
        shaped
            .with_spring(
                width_id,
                SpringAnimation::new(spring).to(expansion),
                move |this, expansion| {
                    this.map_element(|button| {
                        let button = button
                            .flex_basis(expanded_width(&slot, expansion))
                            .min_w(slot.min_width);
                        if slot.flexible {
                            button.flex_grow(1.0).flex_shrink(1.0)
                        } else {
                            button.flex_grow(0.0).flex_shrink(0.0)
                        }
                    })
                },
            )
            .into_any_element()
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let press = self
            .press
            .clone()
            .unwrap_or_else(|| press_state(&self.id, window, cx));
        let (inner, paint) = self.prepare(&press);
        let (base, state) = inner.build(window, cx);
        paint.paint(base, state, press, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_morphs_from_round_to_square_and_smaller_when_pressed() {
        let theme = Theme::default();
        let height = px(40.0);
        let round = target_radius(&theme, height, false, false, false);
        let checked = target_radius(&theme, height, false, true, false);
        let pressed = target_radius(&theme, height, true, true, false);
        assert_eq!(round, px(20.0));
        assert_eq!(checked, px(12.0));
        assert_eq!(pressed, px(8.0));
    }

    #[test]
    fn connected_inner_corners_round_when_checked_and_shrink_when_pressed() {
        let theme = Theme::default();
        let height = px(40.0);
        assert_eq!(target_radius(&theme, height, false, false, true), px(8.0));
        assert_eq!(target_radius(&theme, height, false, true, true), px(20.0));
        assert_eq!(target_radius(&theme, height, true, false, true), px(4.0));
        let corners = corners_of(px(8.0), px(20.0), Some((true, false)));
        assert_eq!(corners.top_left, px(20.0));
        assert_eq!(corners.top_right, px(8.0));
    }

    fn slot(natural: f32, capacity: f32) -> GroupSlot {
        GroupSlot {
            natural: px(natural),
            flexible: false,
            min_width: px(natural - 32.0),
            capacity: px(capacity),
            connected: None,
        }
    }

    #[test]
    fn expansion_is_fifteen_percent_when_neighbors_can_absorb_it() {
        let slot = slot(100.0, 64.0);
        assert_eq!(expanded_width(&slot, 0.0), px(100.0));
        assert_eq!(expanded_width(&slot, 1.0), px(115.0));
    }

    #[test]
    fn expansion_of_a_long_label_is_capped_by_the_neighbors_compression() {
        // 端の項目（隣は 1 つで、縮められるのは余白の 32px まで）。15% は 45px で上限を超える。
        let slot = slot(300.0, 32.0);
        assert_eq!(expanded_width(&slot, 1.0), px(332.0));
        // バネの行き過ぎでも上限を超えない。
        assert_eq!(expanded_width(&slot, 1.3), px(332.0));
        // 戻りの行き過ぎ（負）は狭くなるだけ。
        assert!(expanded_width(&slot, -0.2) < px(300.0));
    }

    #[test]
    fn single_item_group_does_not_expand() {
        let slot = slot(120.0, 0.0);
        assert_eq!(expanded_width(&slot, 1.0), px(120.0));
        assert_eq!(expanded_width(&slot, 1.3), px(120.0));
    }

    #[test]
    fn natural_width_includes_padding_icon_and_label() {
        let plain = ToggleButton::new("a", "A").natural_width(px(40.0));
        let with_icon = ToggleButton::new("b", "B")
            .icon(IconName::Check)
            .natural_width(px(40.0));
        assert_eq!(plain, px(72.0));
        assert_eq!(with_icon, px(100.0));
        let tiny = ToggleButton::new("c", "C").natural_width(px(1.0));
        assert_eq!(tiny, px(layout::BUTTON_MIN_WIDTH));
    }
}
