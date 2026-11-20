//! 操作を並べる、見た目を持たないツールバー

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, FocusHandle, InteractiveElement, Interactivity, IntoElement,
    ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement, Styled,
    Window, accesskit, div,
};

use crate::{
    Availability,
    roving::{self, GroupChild, RovingItem},
};

/// ツールバーの項目を並べる向き
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolbarOrientation {
    /// 横に並べる。左右の矢印キーで項目の間を移る
    #[default]
    Horizontal,
    /// 縦に並べる。上下の矢印キーで項目の間を移る
    Vertical,
}

/// [`Toolbar`] の項目
///
/// 項目の要素は呼び出し側が作る。要素は渡した [`FocusHandle`] でフォーカスを受け、
/// Tab で到達できるかはツールバーが決めた値に従うこと
pub struct ToolbarItem {
    focus_handle: FocusHandle,
    availability: Availability,
    build: Box<dyn FnOnce(bool) -> AnyElement>,
}

impl ToolbarItem {
    /// `focus_handle` でフォーカスを受ける項目を作る
    ///
    /// `availability` は項目の要素と同じ値にすること。[`Availability::Disabled`] の項目には Tab でも矢印キーでも移らない。
    /// `build` は描画のたびに 1 回呼ばれ、Tab で到達できるようにするかを受け取って項目の要素を返す。
    /// 要素は `focus_handle` をフォーカスとして使い、受け取った値に従って Tab で到達できるかを設定すること
    /// ([`Button`](crate::button::Button) なら [`Button::track_focus`](crate::button::Button::track_focus) と
    /// [`Button::tab_stop`](crate::button::Button::tab_stop))
    pub fn new(
        focus_handle: FocusHandle,
        availability: Availability,
        build: impl FnOnce(bool) -> AnyElement + 'static,
    ) -> Self {
        Self {
            focus_handle,
            availability,
            build: Box::new(build),
        }
    }
}

/// 操作を並べる、見た目を持たないツールバー
///
/// 項目は [`Toolbar::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。
/// `child` には区切り線のようなフォーカスを持たない要素を渡すこと。フォーカスを持つ要素は矢印キーで移れず、Tab でも別に止まる。
/// 項目の並ぶ向きは [`Toolbar::orientation`] と同じにすること
///
/// - Tab で到達できる項目は 1 つだけで、最初のフォーカスできる項目にする。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。フォーカスできる項目が無ければ Tab で入らない
/// - 修飾キーなしの矢印キー (横なら左右、縦なら上下) で前後の項目へフォーカスを移す。端では反対の端に戻る。
///   [`Availability::Disabled`] の項目は飛ばす。移れる項目がほかに無い場合は、キー入力を祖先に渡す。
///   この矢印キーは項目より先に受け取るため、項目が同じキーを使っていても (上下キーで開くメニューなど) 項目の間の移動になる
///
/// 支援技術には role `toolbar` と向きとして公開される。ツールバーの名前は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Toolbar {
    inner: Stateful<Div>,
    orientation: ToolbarOrientation,
    children: Vec<GroupChild<ToolbarItem>>,
}

impl Toolbar {
    /// 項目を持たない横のツールバーを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            inner: div().id(id),
            orientation: ToolbarOrientation::default(),
            children: Vec::new(),
        }
    }

    /// 項目の並ぶ向きを設定する
    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let items: Rc<[RovingItem]> = self
            .children
            .iter()
            .filter_map(|child| match child {
                GroupChild::Item(item) => Some(RovingItem {
                    focus_handle: item.focus_handle.clone(),
                    availability: item.availability,
                }),
                GroupChild::Other(_) => None,
            })
            .collect();
        let tab_stop = roving::tab_stop(&items, None, window);

        let mut ix = 0;
        let elements: Vec<AnyElement> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(item) => {
                    let element = (item.build)(tab_stop == Some(ix));
                    ix += 1;
                    element
                }
                GroupChild::Other(element) => element,
            })
            .collect();

        let (orientation, next_key, previous_key) = match self.orientation {
            ToolbarOrientation::Horizontal => (accesskit::Orientation::Horizontal, "right", "left"),
            ToolbarOrientation::Vertical => (accesskit::Orientation::Vertical, "down", "up"),
        };
        self.inner
            .role(accesskit::Role::Toolbar)
            .aria_orientation(orientation)
            // メニューのボタンのように同じ矢印キーを使う項目より先に受け取るため、capture で受ける。
            // 項目にフォーカスが無ければ neighbor が None を返し、子の要素のキー入力には触れない
            .capture_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let key = event.keystroke.key.as_str();
                let forward = if key == next_key {
                    true
                } else if key == previous_key {
                    false
                } else {
                    return;
                };
                let Some(ix) = roving::neighbor(&items, forward, window) else {
                    return;
                };
                window.focus(&items[ix].focus_handle, cx);
                cx.stop_propagation();
            })
            .children(elements)
    }
}

impl Styled for Toolbar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl InteractiveElement for Toolbar {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for Toolbar {}

#[cfg(test)]
mod tests {
    use gpui::{Context, Element, Entity, Render, TestAppContext, VisualTestContext, px};

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    const NAMES: [&str; 3] = ["cut", "copy", "paste"];

    struct ToolbarView {
        orientation: ToolbarOrientation,
        availability: [Availability; 3],
        handles: [FocusHandle; 3],
        after: FocusHandle,
        node: Option<accesskit::Node>,
    }

    impl Render for ToolbarView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let toolbar = NAMES
                .iter()
                .enumerate()
                .fold(
                    Toolbar::new("edit")
                        .orientation(self.orientation)
                        .aria_label("Edit")
                        .flex(),
                    |toolbar, (ix, name)| {
                        let handle = self.handles[ix].clone();
                        let availability = self.availability[ix];
                        toolbar.item(ToolbarItem::new(
                            handle.clone(),
                            availability,
                            move |tab_stop| {
                                Button::new(*name)
                                    .track_focus(&handle)
                                    .tab_stop(tab_stop)
                                    .availability(availability)
                                    .aria_label(*name)
                                    .size(px(20.))
                                    // 上下キーで開くメニューのボタンのように、項目自身も矢印キーを受けて伝播を止める
                                    .on_key_down(|event, _, cx| {
                                        if matches!(
                                            event.keystroke.key.as_str(),
                                            "up" | "down" | "left" | "right"
                                        ) {
                                            cx.stop_propagation();
                                        }
                                    })
                                    .into_any_element()
                            },
                        ))
                    },
                )
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(toolbar.a11y_role().expect("role を持つこと"));
            toolbar.write_a11y_info(&mut node);
            self.node = Some(node);
            div()
                .size_full()
                .child(toolbar)
                .child(Button::new("after").track_focus(&self.after).h(px(20.)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        orientation: ToolbarOrientation,
        availability: [Availability; 3],
    ) -> (Entity<ToolbarView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ToolbarView {
            orientation,
            availability,
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            after: cx.focus_handle(),
            node: None,
        })
    }

    const ALL_ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある項目の名前
    fn focused(view: &Entity<ToolbarView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| {
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| NAMES[ix])
        })
    }

    fn is_after_focused(view: &Entity<ToolbarView>, cx: &mut VisualTestContext) -> bool {
        let after = view.read_with(cx, |view, _| view.after.clone());
        cx.update(|window, _| after.is_focused(window))
    }

    #[gpui::test]
    fn role_toolbar_と向きと名前で公開される(cx: &mut TestAppContext) {
        for (orientation, expected) in [
            (
                ToolbarOrientation::Horizontal,
                accesskit::Orientation::Horizontal,
            ),
            (
                ToolbarOrientation::Vertical,
                accesskit::Orientation::Vertical,
            ),
        ] {
            let (view, cx) = setup(cx, orientation, ALL_ENABLED);
            let node = view
                .read_with(cx, |view, _| view.node.clone())
                .expect("描画済みであること");
            assert_eq!(node.role(), accesskit::Role::Toolbar);
            assert_eq!(node.orientation(), Some(expected));
            assert_eq!(node.label(), Some("Edit"));
        }
    }

    #[gpui::test]
    fn tab_で最初のフォーカスできる項目に入り次の_tab_で外へ出る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            ToolbarOrientation::Horizontal,
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("copy"));
        cx.simulate_keystrokes("tab");
        assert!(
            is_after_focused(&view, cx),
            "ツールバーの中の Tab stop は 1 つだけであること"
        );
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(
            focused(&view, cx),
            Some("copy"),
            "戻るときは最後にフォーカスのあった項目に入ること"
        );
    }

    #[gpui::test]
    fn 横は左右の矢印キーで移り端では反対の端に戻る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ToolbarOrientation::Horizontal, ALL_ENABLED);
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("copy"));
        cx.simulate_keystrokes("right right");
        assert_eq!(focused(&view, cx), Some("cut"));
        cx.simulate_keystrokes("left");
        assert_eq!(focused(&view, cx), Some("paste"));
        cx.simulate_keystrokes("down up shift-right");
        assert_eq!(
            focused(&view, cx),
            Some("paste"),
            "上下と修飾キー付きの矢印キーでは移らないこと"
        );
    }

    #[gpui::test]
    fn 縦は上下の矢印キーで移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ToolbarOrientation::Vertical, ALL_ENABLED);
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("copy"));
        cx.simulate_keystrokes("up up");
        assert_eq!(focused(&view, cx), Some("paste"));
        cx.simulate_keystrokes("right");
        assert_eq!(
            focused(&view, cx),
            Some("paste"),
            "左右の矢印キーでは移らないこと"
        );
    }

    #[gpui::test]
    fn 無効な項目は矢印キーで飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            ToolbarOrientation::Horizontal,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab right");
        assert_eq!(focused(&view, cx), Some("paste"));
    }

    #[gpui::test]
    fn すべての項目が無効なら_tab_で入らない(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            ToolbarOrientation::Horizontal,
            [Availability::Disabled; 3],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None);
        assert!(is_after_focused(&view, cx));
    }
}
