//! Chip（Assist / Filter / Input / Suggestion）。

use gpui::{
    App, ClickEvent, ElementId, IntoElement, KeyDownEvent, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};
use md3_theme::md3_color::ColorRole;
use md3_theme::md3_tokens::component::{assist_chip, filter_chip, input_chip, suggestion_chip};
use md3_theme::md3_tokens::layout;
use md3_theme::{Elevation, Md3ThemeExt};

use crate::a11y::{A11yExt, A11yNode, toggled};
use crate::control::control;
use crate::{FocusRing, ForcedState, Icon, IconName, StateLayer, StyledExt};

/// チップの種類。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChipKind {
    /// 関連する操作を提案する。
    #[default]
    Assist,
    /// 絞り込みの条件を切り替える（選択状態を持つ）。
    Filter,
    /// 利用者が入力した項目を表す（削除できる）。
    Input,
    /// 入力の候補を出す。
    Suggestion,
}

impl ChipKind {
    /// すべての種類。
    pub const ALL: [ChipKind; 4] = [
        ChipKind::Assist,
        ChipKind::Filter,
        ChipKind::Input,
        ChipKind::Suggestion,
    ];
}

/// 種類と状態ごとの見た目（トークンの値）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ChipStyle {
    pub container: Option<ColorRole>,
    pub outline: Option<ColorRole>,
    pub label: ColorRole,
    pub leading_icon: ColorRole,
    pub trailing_icon: ColorRole,
    pub elevation: f32,
    pub hovered_elevation: f32,
}

impl ChipStyle {
    pub(crate) fn of(kind: ChipKind, selected: bool, elevated: bool) -> Self {
        let flat = |outline: ColorRole, label: ColorRole, icon: ColorRole| Self {
            container: None,
            outline: Some(outline),
            label,
            leading_icon: icon,
            trailing_icon: icon,
            elevation: 0.0,
            hovered_elevation: 0.0,
        };
        match kind {
            ChipKind::Assist if elevated => Self {
                container: Some(assist_chip::ELEVATED_CONTAINER_COLOR),
                outline: None,
                elevation: assist_chip::ELEVATED_CONTAINER_ELEVATION,
                hovered_elevation: assist_chip::ELEVATED_HOVER_CONTAINER_ELEVATION,
                ..flat(
                    assist_chip::FLAT_OUTLINE_COLOR,
                    assist_chip::LABEL_TEXT_COLOR,
                    assist_chip::ICON_COLOR,
                )
            },
            ChipKind::Assist => flat(
                assist_chip::FLAT_OUTLINE_COLOR,
                assist_chip::LABEL_TEXT_COLOR,
                assist_chip::ICON_COLOR,
            ),
            ChipKind::Suggestion if elevated => Self {
                container: Some(suggestion_chip::ELEVATED_CONTAINER_COLOR),
                outline: None,
                elevation: suggestion_chip::ELEVATED_CONTAINER_ELEVATION,
                hovered_elevation: suggestion_chip::ELEVATED_HOVER_CONTAINER_ELEVATION,
                ..flat(
                    suggestion_chip::FLAT_OUTLINE_COLOR,
                    suggestion_chip::LABEL_TEXT_COLOR,
                    suggestion_chip::LEADING_ICON_COLOR,
                )
            },
            ChipKind::Suggestion => flat(
                suggestion_chip::FLAT_OUTLINE_COLOR,
                suggestion_chip::LABEL_TEXT_COLOR,
                suggestion_chip::LEADING_ICON_COLOR,
            ),
            ChipKind::Filter if selected => Self {
                container: Some(if elevated {
                    filter_chip::ELEVATED_SELECTED_CONTAINER_COLOR
                } else {
                    filter_chip::FLAT_SELECTED_CONTAINER_COLOR
                }),
                outline: None,
                label: filter_chip::SELECTED_LABEL_TEXT_COLOR,
                leading_icon: filter_chip::SELECTED_LEADING_ICON_COLOR,
                trailing_icon: filter_chip::SELECTED_TRAILING_ICON_COLOR,
                elevation: if elevated {
                    filter_chip::ELEVATED_CONTAINER_ELEVATION
                } else {
                    0.0
                },
                hovered_elevation: if elevated {
                    filter_chip::ELEVATED_HOVER_CONTAINER_ELEVATION
                } else {
                    filter_chip::FLAT_SELECTED_HOVER_CONTAINER_ELEVATION
                },
            },
            ChipKind::Filter => Self {
                container: elevated.then_some(filter_chip::ELEVATED_UNSELECTED_CONTAINER_COLOR),
                outline: (!elevated).then_some(filter_chip::FLAT_UNSELECTED_OUTLINE_COLOR),
                label: filter_chip::UNSELECTED_LABEL_TEXT_COLOR,
                leading_icon: filter_chip::UNSELECTED_LEADING_ICON_COLOR,
                trailing_icon: filter_chip::UNSELECTED_TRAILING_ICON_COLOR,
                elevation: if elevated {
                    filter_chip::ELEVATED_CONTAINER_ELEVATION
                } else {
                    0.0
                },
                hovered_elevation: if elevated {
                    filter_chip::ELEVATED_HOVER_CONTAINER_ELEVATION
                } else {
                    filter_chip::FLAT_UNSELECTED_HOVER_CONTAINER_ELEVATION
                },
            },
            ChipKind::Input if selected => Self {
                container: Some(input_chip::SELECTED_CONTAINER_COLOR),
                outline: None,
                label: input_chip::SELECTED_LABEL_TEXT_COLOR,
                leading_icon: input_chip::SELECTED_LEADING_ICON_COLOR,
                trailing_icon: input_chip::SELECTED_TRAILING_ICON_COLOR,
                elevation: input_chip::CONTAINER_ELEVATION,
                hovered_elevation: input_chip::CONTAINER_ELEVATION,
            },
            ChipKind::Input => Self {
                container: None,
                outline: Some(input_chip::UNSELECTED_OUTLINE_COLOR),
                label: input_chip::UNSELECTED_LABEL_TEXT_COLOR,
                leading_icon: input_chip::UNSELECTED_LEADING_ICON_COLOR,
                trailing_icon: input_chip::UNSELECTED_TRAILING_ICON_COLOR,
                elevation: input_chip::CONTAINER_ELEVATION,
                hovered_elevation: input_chip::CONTAINER_ELEVATION,
            },
        }
    }
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type RemoveHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
type SharedRemoveHandler = std::rc::Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// チップ。
///
/// Filter は押すたびに `on_click` で選択状態を反転させる（呼び出し側が状態を持つ）。
/// Input は Backspace / Delete キーか削除アイコンで `on_remove` を呼ぶ。
/// 削除アイコンは名前付きのボタンとして支援技術に出す（WCAG 2.2 の 4.1.2）。
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    kind: ChipKind,
    label: SharedString,
    icon: Option<IconName>,
    selected: bool,
    elevated: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    on_remove: Option<(SharedString, RemoveHandler)>,
    forced: Option<ForcedState>,
}

/// 削除アイコンの押せる範囲の一辺。WCAG 2.2 の 2.5.8 Target Size (Minimum) の 24 CSS px。
/// アイコン（18dp）より広い分は負の余白で打ち消し、見た目の配置は変えない。
const REMOVE_TARGET_SIZE: f32 = 24.0;

impl Chip {
    /// ID、種類、ラベルから作る。
    pub fn new(id: impl Into<ElementId>, kind: ChipKind, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            icon: None,
            selected: false,
            elevated: false,
            disabled: false,
            on_click: None,
            on_remove: None,
            forced: None,
        }
    }

    /// ラベルの前にアイコンを置く（Filter の選択中はチェックに置き換わる）。
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 選択状態（Filter、Input）。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// 影付き（elevated）にする（Assist、Filter、Suggestion）。
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 押されたときの処理。
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 削除されたときの処理（Input）。`label` は削除アイコンの読み上げ用の名前（「rust を削除」など）。
    pub fn on_remove(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some((label.into(), Box::new(handler)));
        self
    }

    /// 見本用に状態を固定する。
    #[doc(hidden)]
    pub fn forced(mut self, forced: ForcedState) -> Self {
        self.forced = Some(forced);
        self
    }

    /// アクセシビリティツリーに出す情報。Filter はチェックボックス相当、他はボタン。
    pub fn a11y_node(&self) -> A11yNode {
        let node = match self.kind {
            ChipKind::Filter => A11yNode::new(Role::CheckBox).toggled(toggled(self.selected)),
            ChipKind::Assist | ChipKind::Input | ChipKind::Suggestion => {
                A11yNode::new(Role::Button)
            }
        };
        node.label(self.label.clone()).disabled(self.disabled)
    }

    /// 削除アイコンのアクセシビリティ情報。削除できる Input のチップだけにある。
    pub fn remove_a11y_node(&self) -> Option<A11yNode> {
        let (label, _) = self.on_remove.as_ref()?;
        (self.kind == ChipKind::Input).then(|| {
            A11yNode::new(Role::Button)
                .label(label.clone())
                .disabled(self.disabled)
        })
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let node = self.a11y_node();
        let remove_node = self.remove_a11y_node();
        let (base, control) = control(&self.id, self.disabled, true, self.forced, window, cx);
        let theme = cx.md3();
        let style = ChipStyle::of(self.kind, self.selected, self.elevated);
        let radius = theme.corner(assist_chip::CONTAINER_SHAPE);
        let disabled = self.disabled;
        let tinted = |role: ColorRole, opacity: f32| {
            if disabled {
                theme
                    .color(assist_chip::DISABLED_LABEL_TEXT_COLOR)
                    .opacity(opacity)
            } else {
                theme.color(role)
            }
        };
        let label_color = tinted(style.label, assist_chip::DISABLED_LABEL_TEXT_OPACITY);
        let leading_color = tinted(style.leading_icon, assist_chip::DISABLED_ICON_OPACITY);
        let trailing_color = tinted(style.trailing_icon, assist_chip::DISABLED_ICON_OPACITY);
        let leading = if self.kind == ChipKind::Filter && self.selected {
            Some(IconName::Check)
        } else {
            self.icon
        };
        let trailing = (self.kind == ChipKind::Input).then_some(IconName::Close);
        let icon_size = px(assist_chip::ICON_SIZE);

        let mut chip = base
            .a11y(node)
            .h(theme.dense(assist_chip::CONTAINER_HEIGHT))
            .px(px(layout::CHIP_CONTENT_SPACE))
            .flex()
            .items_center()
            .corner_radius(radius)
            .text_color(label_color)
            .type_style(&theme.text_style(assist_chip::LABEL_TEXT_FONT));
        if let Some(container) = style.container {
            chip = chip.bg(if disabled {
                theme
                    .color(assist_chip::ELEVATED_DISABLED_CONTAINER_COLOR)
                    .opacity(assist_chip::ELEVATED_DISABLED_CONTAINER_OPACITY)
            } else {
                theme.color(container)
            });
        }
        if let Some(outline) = style.outline {
            chip = chip
                .border(px(assist_chip::FLAT_OUTLINE_WIDTH))
                .border_color(if disabled {
                    theme
                        .color(assist_chip::FLAT_DISABLED_OUTLINE_COLOR)
                        .opacity(assist_chip::FLAT_DISABLED_OUTLINE_OPACITY)
                } else {
                    theme.color(outline)
                });
        }
        if !disabled && style.hovered_elevation > 0.0 {
            let hovered = theme.shadows(Elevation::from_dp(style.hovered_elevation));
            chip = if self.forced == Some(ForcedState::Hovered) {
                chip.shadow(hovered)
            } else {
                chip.shadow(theme.shadows(Elevation::from_dp(style.elevation)))
                    .hover(move |s| s.shadow(hovered.clone()))
            };
        }

        let on_remove: Option<SharedRemoveHandler> = self
            .on_remove
            .filter(|_| !disabled)
            .map(|(_, handler)| std::rc::Rc::from(handler));
        chip.when_some(leading, |this, icon| {
            this.child(Icon::new(icon).size(icon_size).tint(leading_color))
        })
        .child(div().px(px(layout::CHIP_LABEL_SPACE)).child(self.label))
        .when_some(trailing, |this, icon| {
            let overhang = px((REMOVE_TARGET_SIZE - assist_chip::ICON_SIZE) / 2.0);
            let mut remove = div()
                .id("remove")
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .size(px(REMOVE_TARGET_SIZE))
                .mx(-overhang)
                .my(-overhang)
                .when_some(remove_node, |this, node| this.a11y(node))
                .child(Icon::new(icon).size(icon_size).tint(trailing_color));
            if let Some(on_remove) = on_remove.clone() {
                remove = remove.on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    on_remove(window, cx);
                });
            }
            this.child(remove)
        })
        .child(
            StateLayer::new(
                SharedString::from(format!("{}-layer", control.group)),
                control.group.clone(),
                style.label,
            )
            .radius(radius)
            .focused(control.focus_visible)
            .disabled(disabled)
            .forced(self.forced),
        )
        .child(FocusRing::new(radius).visible(control.focus_visible))
        .when_some(self.on_click.filter(|_| !disabled), |this, handler| {
            this.on_click(handler)
        })
        .when_some(on_remove, |this, on_remove| {
            this.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if matches!(event.keystroke.key.as_str(), "backspace" | "delete") {
                    cx.stop_propagation();
                    on_remove(window, cx);
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Toggled;

    #[test]
    fn selected_filter_chip_is_secondary_container() {
        let style = ChipStyle::of(ChipKind::Filter, true, false);
        assert_eq!(style.container, Some(ColorRole::SecondaryContainer));
        assert_eq!(style.outline, None);
        let unselected = ChipStyle::of(ChipKind::Filter, false, false);
        assert_eq!(unselected.container, None);
        assert!(unselected.outline.is_some());
    }

    #[test]
    fn filter_chip_is_a_checkbox() {
        let node = Chip::new("c", ChipKind::Filter, "未読")
            .selected(true)
            .a11y_node();
        assert_eq!(node.role, Role::CheckBox);
        assert_eq!(node.toggled, Some(Toggled::True));
        let assist = Chip::new("a", ChipKind::Assist, "予定に追加").a11y_node();
        assert_eq!(assist.role, Role::Button);
    }
}
