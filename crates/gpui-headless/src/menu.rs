//! ドロップダウンメニューの役割、フォーカス、キー操作。

use crate::keys::{Dismiss, MENU_CONTEXT};
use crate::overlay::{
    DismissHandler, InitialFocus, MENU_PRIORITY, dismiss_and_restore, overlay_focus,
};
use crate::{
    A11yExt, A11yNode, ControlState, NestedScroll, ScrollScope, control, focus_handle,
    scroll_handle, step,
};
use gpui::{
    AnchoredPositionMode, AnyElement, App, Bounds, Div, Edges, ElementId, Entity, FocusHandle,
    IntoElement, Pixels, Role, ScrollHandle, SharedString, Stateful, Window, anchored, canvas,
    deferred, div, prelude::*, px,
};
use std::rc::Rc;
use std::sync::Arc;

/// メニューの項目。
pub struct MenuItem {
    name: crate::AccessibleName,
    description: Option<SharedString>,
    disabled: bool,
    on_select: Option<DismissHandler>,
}

impl MenuItem {
    /// ラベルから作る。
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            name: crate::AccessibleName::new(label),
            description: None,
            disabled: false,
            on_select: None,
        }
    }

    /// 補足の説明を指定する。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    accessible_name!();

    /// 無効にする。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 選ばれたときの処理。この後にメニューの `on_dismiss` が呼ばれる。
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

/// メニューの使われ方。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuKind {
    /// 操作の一覧（`menu` / `menuitem`）。
    #[default]
    Actions,
    /// 値の選択肢（`listbox` / `option`）。
    Options,
}

/// 開いている間だけ描くメニュー。
///
/// 開くと選択中か最初の項目にフォーカスし、上下の矢印・Home・End で項目を移る。Enter / Space で選び、
/// Esc・Tab・外側の押下で閉じる。キーボードで閉じたときは、開く前の要素にフォーカスを戻す。
pub struct Menu {
    id: ElementId,
    name: crate::AccessibleName,
    items: Vec<MenuItem>,
    selected: Option<usize>,
    on_dismiss: Option<DismissHandler>,
    kind: MenuKind,
    match_anchor_width: bool,
    inline: bool,
    window_margin: Edges<Pixels>,
    reveal_margin: Pixels,
    group_prefix: SharedString,
}

/// メニューの項目の要素と描画に使う状態。
pub struct MenuRow {
    /// 操作を付けた土台。
    pub base: Stateful<Div>,
    /// 共通の操作状態。
    pub state: ControlState,
    /// 項目の ID。
    pub id: ElementId,
    /// 項目の位置。
    pub index: usize,
    /// 選択された選択肢か。
    pub selected: bool,
}

struct Placement {
    anchor_bounds: Entity<Option<Bounds<Pixels>>>,
    window_margin: Edges<Pixels>,
    scroll: ScrollHandle,
}

/// 組み立てたメニュー。[`MenuPopup::place`] で見た目を足して配置する。
pub struct MenuPopup {
    container: Stateful<Div>,
    /// コンテナのスクロール。開いたメニューがスクロールバーを描くのに使う。
    pub scroll: ScrollHandle,
    /// 基準の要素の幅に揃えるときの幅。
    pub anchor_width: Option<Pixels>,
    placement: Option<Placement>,
}

impl MenuPopup {
    /// 開いたメニューか（その場に描く見本なら `false`）。
    pub fn is_floating(&self) -> bool {
        self.placement.is_some()
    }

    /// `style` で見た目を足し、基準の要素の下に重ねて描く要素を返す。
    pub fn place(self, style: impl FnOnce(Stateful<Div>) -> Stateful<Div>) -> AnyElement {
        let container = style(self.container);
        let Some(Placement {
            anchor_bounds,
            window_margin,
            scroll,
        }) = self.placement
        else {
            return container.into_any_element();
        };
        // 親（基準の要素と同じ大きさ）を覆う層の下端に、メニューを重ね描きする。
        div()
            .absolute()
            .inset_0()
            .child(
                canvas(
                    move |bounds, _, cx| {
                        anchor_bounds.update(cx, |stored, _| *stored = Some(bounds))
                    },
                    |_, _, _, _| {},
                )
                .size_full(),
            )
            .child(
                div().absolute().top_full().left_0().child(
                    deferred(
                        anchored()
                            .position_mode(AnchoredPositionMode::Window)
                            .snap_to_window_with_margin(window_margin)
                            .child(ScrollScope::new(scroll, container)),
                    )
                    .with_priority(MENU_PRIORITY),
                ),
            )
            .into_any_element()
    }
}

impl Menu {
    /// ID から作る。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: crate::AccessibleName::default(),
            items: Vec::new(),
            selected: None,
            on_dismiss: None,
            kind: MenuKind::Actions,
            match_anchor_width: false,
            inline: false,
            window_margin: Edges::all(px(0.0)),
            reveal_margin: px(0.0),
            group_prefix: "control".into(),
        }
    }

    accessible_name!();

    /// 項目を足す。
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// 閉じるときの処理（項目を選んだとき、Esc、Tab、外側の押下）。
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// 使われ方を指定する。
    pub fn kind(mut self, kind: MenuKind) -> Self {
        self.kind = kind;
        self
    }

    /// 選択中の項目を指定する。
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// 基準の要素の幅を [`MenuPopup::anchor_width`] で受け取る。
    pub fn match_anchor_width(mut self, enabled: bool) -> Self {
        self.match_anchor_width = enabled;
        self
    }

    /// その場に描く（フォーカスは動かさない）。
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// ウィンドウの端から空ける余白。
    pub fn window_margin(mut self, margin: Edges<Pixels>) -> Self {
        self.window_margin = margin;
        self
    }

    /// フォーカス位置を表示する際の余白を指定する。
    pub fn reveal_margin(mut self, margin: Pixels) -> Self {
        self.reveal_margin = margin;
        self
    }

    /// 項目のホバーなどに使うグループ名の接頭辞。
    pub fn group_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.group_prefix = prefix.into();
        self
    }

    /// アクセシビリティツリーに出す情報。
    pub fn a11y_node(&self) -> A11yNode {
        let role = match self.kind {
            MenuKind::Actions => Role::Menu,
            MenuKind::Options => Role::ListBox,
        };
        A11yNode::new(role).with_name(&self.name)
    }

    /// 項目のアクセシビリティツリーに出す情報。
    pub fn item_a11y_node(&self, index: usize) -> Option<A11yNode> {
        let item = self.items.get(index)?;
        let mut node = match self.kind {
            MenuKind::Actions => A11yNode::new(Role::MenuItem),
            MenuKind::Options => {
                A11yNode::new(Role::ListBoxOption).selected(self.selected == Some(index))
            }
        };
        if let Some(description) = &item.description {
            node = node.description(description.clone());
        }
        Some(
            node.with_name(&item.name)
                .position(index, self.items.len())
                .disabled(item.disabled),
        )
    }

    /// 項目の描画を受け取り、組み立てたメニューを返す。
    pub fn build(
        self,
        window: &mut Window,
        cx: &mut App,
        mut render: impl FnMut(MenuRow, &mut Window, &mut App) -> AnyElement,
    ) -> MenuPopup {
        window.with_element_namespace(self.id.clone(), |window| {
            self.build_in_namespace(window, cx, &mut render)
        })
    }

    fn build_in_namespace(
        self,
        window: &mut Window,
        cx: &mut App,
        render: &mut dyn FnMut(MenuRow, &mut Window, &mut App) -> AnyElement,
    ) -> MenuPopup {
        let parent = Arc::new(self.id.clone());
        let item_ids: Vec<ElementId> = (0..self.items.len())
            .map(|index| ElementId::NamedChild(parent.clone(), format!("item-{index}").into()))
            .collect();
        let enabled: Vec<bool> = self.items.iter().map(|item| !item.disabled).collect();
        // control() と同じキーで取り出すので、同じフォーカスハンドルになる。
        let handles: Vec<Option<FocusHandle>> = item_ids
            .iter()
            .zip(&enabled)
            .map(|(id, &enabled)| enabled.then(|| focus_handle(id.clone(), window, cx)))
            .collect();
        let initial = self
            .selected
            .filter(|&index| enabled.get(index).copied().unwrap_or(false))
            .or_else(|| enabled.iter().position(|&enabled| enabled))
            .and_then(|index| handles.get(index).cloned().flatten());
        let focus = (!self.inline).then(|| {
            overlay_focus(
                &self.id,
                move || match initial {
                    Some(handle) => InitialFocus::Handle(handle),
                    None => InitialFocus::Within,
                },
                window,
                cx,
            )
        });
        let anchor_bounds: Entity<Option<Bounds<Pixels>>> = window.use_keyed_state(
            ElementId::NamedChild(parent.clone(), "anchor".into()),
            cx,
            |_, _| None,
        );
        let close = focus
            .as_ref()
            .and_then(|focus| dismiss_and_restore(self.on_dismiss.clone(), focus));

        let node = self.a11y_node();
        let item_nodes: Vec<Option<A11yNode>> = (0..self.items.len())
            .map(|index| self.item_a11y_node(index))
            .collect();

        // 開いたメニューは画面に収まらなければスクロールする。矢印で移った項目を見える範囲に入れる。
        let scroll = scroll_handle(&self.id, window, cx);
        let mut rows = Vec::with_capacity(self.items.len());
        for (index, (item, id)) in self.items.into_iter().zip(item_ids).enumerate() {
            let (base, state) = control(
                &id,
                item.disabled,
                false,
                window,
                cx,
                self.reveal_margin,
                format!("{}-{id}", self.group_prefix).into(),
            );
            let mut base = base.when_some(item_nodes[index].clone(), |this, node| this.a11y(node));
            if !item.disabled {
                let on_select = item.on_select;
                let close = close.clone();
                base = base.on_click(move |_, window, cx| {
                    if let Some(on_select) = &on_select {
                        on_select(window, cx);
                    }
                    if let Some(close) = &close {
                        close(window, cx);
                    }
                });
            }
            rows.push(window.with_element_namespace(id.clone(), |window| {
                render(
                    MenuRow {
                        base,
                        state,
                        id,
                        index,
                        selected: self.kind == MenuKind::Options && self.selected == Some(index),
                    },
                    window,
                    cx,
                )
            }));
        }

        let anchor_width = self
            .match_anchor_width
            .then(|| (*anchor_bounds.read(cx)).map(|bounds| bounds.size.width))
            .flatten();
        let container = div().id(self.id.clone()).a11y(node).children(rows);
        let Some(focus) = focus else {
            return MenuPopup {
                container,
                scroll,
                anchor_width,
                placement: None,
            };
        };

        let handle = focus.read(cx).handle.clone();
        let on_outside = self.on_dismiss.clone();
        let anchor_for_outside = anchor_bounds.clone();
        let wheel = NestedScroll::new(&self.id, scroll.clone(), window, cx);
        let margin = self.window_margin;
        let container = container
            .key_context(MENU_CONTEXT)
            .track_focus(&handle)
            .max_h(window.viewport_size().height - margin.top - margin.bottom)
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .on_scroll(wheel.on_scroll())
            .child(wheel.capture())
            .occlude()
            .on_key_down(move |event, window, cx| {
                let focused = handles
                    .iter()
                    .position(|handle| handle.as_ref().is_some_and(|h| h.is_focused(window)));
                let target = match (event.keystroke.key.as_str(), focused) {
                    ("down", Some(from)) => step(&enabled, from, true),
                    ("up", Some(from)) => step(&enabled, from, false),
                    ("down" | "home", None) | ("home", Some(_)) => {
                        enabled.iter().position(|&enabled| enabled)
                    }
                    ("up" | "end", None) | ("end", Some(_)) => {
                        enabled.iter().rposition(|&enabled| enabled)
                    }
                    _ => return,
                };
                if let Some(handle) = target.and_then(|index| handles.get(index).cloned().flatten())
                {
                    window.focus(&handle, cx);
                }
                cx.stop_propagation();
            })
            .when_some(close, |this, close| {
                this.on_action(move |_: &Dismiss, window, cx| close(window, cx))
            })
            .when_some(on_outside, |this, on_outside| {
                // 外側の押下で閉じる。押した先にフォーカスが移るので、フォーカスは戻さない。
                // 基準の要素の上の押下は無視する（基準の要素の on_click が閉じる）。
                this.on_mouse_down_out(move |event, window, cx| {
                    let on_anchor = anchor_for_outside
                        .read(cx)
                        .is_some_and(|bounds| bounds.contains(&event.position));
                    if !on_anchor {
                        on_outside(window, cx);
                    }
                })
            });
        MenuPopup {
            container,
            scroll: scroll.clone(),
            anchor_width,
            placement: Some(Placement {
                anchor_bounds,
                window_margin: margin,
                scroll,
            }),
        }
    }
}
