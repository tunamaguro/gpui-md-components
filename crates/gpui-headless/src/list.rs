//! 項目を縦に並べる、見た目を持たないリスト

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, KeyboardButton, KeyboardClickEvent, MouseButton, ParentElement, RenderOnce,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, accesskit, div,
    prelude::FluentBuilder,
};

use crate::{
    Availability,
    accessibility::A11yInfoExt,
    choice::CheckedState,
    roving::{self, GroupChild, RovingItem},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 項目を縦に並べる、見た目を持たないリスト
///
/// 項目は [`List::item`] で、項目でない要素は `child` で加え、加えた順に並ぶ。項目は上から下へ並べること。
/// `child` には区切り線のように支援技術に公開しない要素だけを置くこと。
/// 公開する要素 (見出しなど) を置くと role `list` の直下に `listitem` 以外のノードが並ぶため、リストの外に置くこと
///
/// - 操作できる項目 ([`ListItemKind::Text`] 以外) のうち、Tab で到達できるのは 1 つだけで、最初のフォーカスできる項目にする。
///   項目にフォーカスがある間は、その項目だけが Tab で到達できる。フォーカスできる項目が無ければ Tab で入らない
/// - 修飾キーなしの上下の矢印キーで前後の操作できる項目へ、Home と End で最初と最後の操作できる項目へフォーカスを移す。
///   項目の状態は変えない。上下の矢印キーは端で反対の端に戻る。
///   [`Availability::Disabled`] の項目は飛ばし、[`Availability::DisabledFocusable`] の項目にはフォーカスを移す。
///   移る先がフォーカスのある項目自身になる場合は、キー入力を祖先に渡す
///
/// 支援技術には role `list` として公開される。名前を与える場合は `aria_label` で与えること。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    inner: Stateful<Div>,
    children: Vec<GroupChild<ListItem>>,
}

impl List {
    /// 項目を持たないリストを作る
    ///
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            children: Vec::new(),
        }
    }

    /// 末尾に項目を加える
    pub fn item(mut self, item: ListItem) -> Self {
        self.children.push(GroupChild::Item(item));
        self
    }

    /// 末尾に `items` の項目を順に加える
    pub fn items(mut self, items: impl IntoIterator<Item = ListItem>) -> Self {
        self.children
            .extend(items.into_iter().map(GroupChild::Item));
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut items = Vec::new();
        let children: Vec<GroupChild<(ListItem, Option<usize>)>> = self
            .children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item(item) if item.kind == ListItemKind::Text => {
                    GroupChild::Item((item, None))
                }
                GroupChild::Item(item) => {
                    items.push(RovingItem {
                        // 項目の id はリストの中でだけ一意なので、リストの id で分ける
                        focus_handle: window.with_element_namespace(self.id.clone(), |window| {
                            item.focus_handle(window, cx)
                        }),
                        availability: item.availability,
                    });
                    GroupChild::Item((item, Some(items.len() - 1)))
                }
                GroupChild::Other(element) => GroupChild::Other(element),
            })
            .collect();
        let items: Rc<[RovingItem]> = items.into();
        let tab_stop = roving::tab_stop(&items, None, window);

        let elements: Vec<AnyElement> = children
            .into_iter()
            .map(|child| match child {
                GroupChild::Item((item, ix)) => {
                    let id = item.id.clone();
                    let body = item_body(
                        item,
                        ix.map(|ix| (&items[ix], tab_stop == Some(ix))),
                        window,
                    );
                    // 操作の対象を listitem の子にする (HTML の `<li><button>` と同じ構造)
                    div()
                        .id(id)
                        .role(accesskit::Role::ListItem)
                        .flex()
                        .flex_col()
                        .child(body)
                        .into_any_element()
                }
                GroupChild::Other(element) => element,
            })
            .collect();

        self.inner
            .role(accesskit::Role::List)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let target = match event.keystroke.key.as_str() {
                    "down" => roving::neighbor(&items, true, window),
                    "up" => roving::neighbor(&items, false, window),
                    "home" | "end"
                        if items
                            .iter()
                            .any(|item| item.focus_handle.is_focused(window)) =>
                    {
                        let focusable = |item: &RovingItem| item.availability.is_focusable();
                        if event.keystroke.key == "home" {
                            items.iter().position(focusable)
                        } else {
                            items.iter().rposition(focusable)
                        }
                        .filter(|&ix| !items[ix].focus_handle.is_focused(window))
                    }
                    _ => return,
                };
                let Some(ix) = target else {
                    return;
                };
                window.focus(&items[ix].focus_handle, cx);
                cx.stop_propagation();
            })
            .children(elements)
    }
}

impl Styled for List {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for List {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children
            .extend(elements.into_iter().map(GroupChild::Other));
    }
}

impl InteractiveElement for List {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for List {}

/// `item` の、見た目を付ける要素を作る。操作できる項目では操作の対象にする
///
/// `roving` は操作できる項目のときの、矢印キーの移動先としての状態と Tab で到達できるかどうか
fn item_body(
    item: ListItem,
    roving: Option<(&RovingItem, bool)>,
    window: &Window,
) -> impl IntoElement + use<> {
    let kind = item.kind;
    let disabled = roving.is_some_and(|(roving, _)| roving.availability.is_disabled());
    let on_click = item.on_click.filter(|_| !disabled);
    item.inner
        .when_some(roving, |this, (roving, tab_stop)| {
            let focus_handle = roving.focus_handle.clone();
            // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
            let focusable = roving.availability.is_focusable() || focus_handle.is_focused(window);
            this.role(match kind {
                ListItemKind::Text | ListItemKind::Action => accesskit::Role::Button,
                ListItemKind::Checkbox(_) => accesskit::Role::CheckBox,
                ListItemKind::Radio(_) => accesskit::Role::RadioButton,
            })
            .map(|this| match kind {
                ListItemKind::Text | ListItemKind::Action => this,
                ListItemKind::Checkbox(state) => this.aria_toggled(state.into()),
                ListItemKind::Radio(selected) => this.aria_toggled(selected.into()),
            })
            .when(disabled, |this| {
                let focus_handle = focus_handle.clone();
                // 押下を親に伝えず、親の on_click を呼ばせない。
                // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    if focusable {
                        window.focus(&focus_handle, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .when(focusable, |this| {
                this.track_focus(&focus_handle.tab_stop(tab_stop))
            })
            .when_some(on_click, |this, on_click| {
                this.on_click(move |event, window, cx| {
                    // APG は Enter での操作を、Checkbox と Radio Group のパターンでは定義していない
                    if kind != ListItemKind::Action
                        && let ClickEvent::Keyboard(KeyboardClickEvent {
                            button: KeyboardButton::Enter,
                            ..
                        }) = event
                    {
                        return;
                    }
                    on_click(event, window, cx);
                })
            })
        })
        .a11y_info(move |node: &mut accesskit::Node| {
            if disabled {
                node.set_disabled();
            }
        })
}

/// [`List`] の項目
///
/// [`List::item`] に渡して使う。見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける。
/// 支援技術には role `listitem` として公開され、操作できる項目では、その子として種類に応じた role の操作の対象を公開する。
/// 見た目を付ける要素はこの操作の対象で、role `listitem` の要素はその幅をリストに合わせて包む。
/// 操作の対象のアクセシブルな名前は、子の内容か `aria_label` で与えること
///
/// 操作できる項目では、マウスのクリック・支援技術からのクリック操作・フォーカス中の Space で
/// [`ListItem::on_click`] のハンドラを呼ぶ。Enter では [`ListItemKind::Action`] の項目だけが呼ぶ
pub struct ListItem {
    id: ElementId,
    inner: Stateful<Div>,
    kind: ListItemKind,
    availability: Availability,
    on_click: Option<ClickHandler>,
    provided_focus_handle: Option<FocusHandle>,
}

/// リストの項目の種類
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ListItemKind {
    /// 操作できない項目
    #[default]
    Text,
    /// 操作を実行する項目。支援技術には role `button` として公開する
    Action,
    /// オンとオフを切り替える項目。値はチェックの状態。支援技術には role `checkbox` として公開する
    Checkbox(CheckedState),
    /// 組の中から 1 つを選ぶ項目。値は選ばれているかどうか。支援技術には role `radio` として公開する
    ///
    /// 選ばれている項目が組の中で 1 つだけになるよう、呼び出し側が値を渡すこと。
    /// リストは組を支援技術に公開しないため、組として公開するには、名前を持つ role `radiogroup` の要素でリストを包むこと
    Radio(bool),
}

impl ListItem {
    /// [`ListItemKind::Text`] の項目を作る
    ///
    /// `id` はリストの中で一意にし、描画のたびに同じ値を渡すこと。
    /// [`ListItem::track_focus`] を指定しない場合、`id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            kind: ListItemKind::default(),
            availability: Availability::Enabled,
            on_click: None,
            provided_focus_handle: None,
        }
    }

    /// 項目の種類を設定する。既定は [`ListItemKind::Text`]
    pub fn kind(mut self, kind: ListItemKind) -> Self {
        self.kind = kind;
        self
    }

    /// 操作されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// [`ListItemKind::Text`] の項目では呼ばない。設定しない場合、項目は支援技術にクリック操作を公開しない。
    /// 切り替える項目の状態を変えるには、`handler` で呼び出し側の状態を更新し、次の描画で [`ListItem::kind`] に渡すこと
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// 項目を無効にするか、無効でも Tab や矢印キーで到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な操作できる項目は、操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`ListItem::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 項目を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な操作できる項目は、操作されてもハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// Tab や矢印キーでも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも到達できるようにしたい場合は [`ListItem::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこの項目のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。Tab で到達できるかはリストが決める。
    /// 指定しない場合は項目が内部でフォーカスを持つ。[`ListItemKind::Text`] の項目では使わない
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle().tab_index(0))
                .read(cx)
                .clone()
        })
    }
}

impl Styled for ListItem {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl InteractiveElement for ListItem {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for ListItem {}

/// APG に List のパターンは無いため、Keyboard Interface の roving tabindex
/// (content/practices/keyboard-interface/keyboard-interface-practice.html) と、
/// 項目の種類に対応する Button・Checkbox・Radio Group のパターンで期待される振る舞い
#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::{
        Context, Element, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, Render,
        TestAppContext, VisualTestContext, point, px,
    };

    use super::*;
    use crate::button::Button;
    use crate::test_window::add_test_window;

    /// 操作できる項目の名前。行 0 は操作できない項目で、行 1 から順に並ぶ
    const NAMES: [&str; 3] = ["action", "checkbox", "radio"];

    struct ListView {
        availability: [Availability; 3],
        handles: [FocusHandle; 3],
        after: FocusHandle,
        clicks: Rc<RefCell<Vec<&'static str>>>,
        parent_presses: Rc<RefCell<usize>>,
        /// リストが祖先に渡したキー入力
        parent_keys: Rc<RefCell<Vec<String>>>,
        list_node: Option<accesskit::Node>,
    }

    impl Render for ListView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let kinds = [
                ListItemKind::Action,
                ListItemKind::Checkbox(CheckedState::Checked),
                ListItemKind::Radio(false),
            ];
            let items = NAMES.iter().enumerate().map(|(ix, name)| {
                let clicks = self.clicks.clone();
                ListItem::new(*name)
                    .kind(kinds[ix])
                    .availability(self.availability[ix])
                    .track_focus(&self.handles[ix])
                    .on_click(move |_, _, _| clicks.borrow_mut().push(name))
                    .aria_label(*name)
                    .h(px(20.))
            });
            let list = List::new("list")
                .aria_label("Fruit")
                .flex()
                .flex_col()
                .item(ListItem::new("text").h(px(20.)))
                .items(items)
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(list.a11y_role().expect("role を持つこと"));
            list.write_a11y_info(&mut node);
            self.list_node = Some(node);
            let parent_presses = self.parent_presses.clone();
            let parent_keys = self.parent_keys.clone();
            div()
                .size_full()
                .on_key_down(move |event, _, _| {
                    parent_keys.borrow_mut().push(event.keystroke.key.clone());
                })
                .on_mouse_down(MouseButton::Left, move |_, _, _| {
                    *parent_presses.borrow_mut() += 1;
                })
                .child(list)
                .child(Button::new("after").track_focus(&self.after).h(px(20.)))
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        availability: [Availability; 3],
    ) -> (Entity<ListView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| ListView {
            availability,
            handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            after: cx.focus_handle(),
            clicks: Rc::default(),
            parent_presses: Rc::default(),
            parent_keys: Rc::default(),
            list_node: None,
        })
    }

    const ALL_ENABLED: [Availability; 3] = [Availability::Enabled; 3];

    /// フォーカスのある操作できる項目の名前
    fn focused(view: &Entity<ListView>, cx: &mut VisualTestContext) -> Option<&'static str> {
        let handles = view.read_with(cx, |view, _| view.handles.clone());
        cx.update(|window, _| {
            handles
                .iter()
                .position(|handle| handle.is_focused(window))
                .map(|ix| NAMES[ix])
        })
    }

    fn is_after_focused(view: &Entity<ListView>, cx: &mut VisualTestContext) -> bool {
        let after = view.read_with(cx, |view, _| view.after.clone());
        cx.update(|window, _| after.is_focused(window))
    }

    fn clicks(view: &Entity<ListView>, cx: &mut VisualTestContext) -> Vec<&'static str> {
        view.read_with(cx, |view, _| view.clicks.borrow().clone())
    }

    fn press_key(cx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).expect("テストのキー表記が有効であること");
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
    }

    /// 上から `row` 番目 (0 始まり) の高さ 20px の行をクリックする
    fn click_row(cx: &mut VisualTestContext, row: usize) {
        cx.simulate_click(
            point(px(10.), px(10. + 20. * row as f32)),
            Modifiers::none(),
        );
    }

    /// 項目の見た目を付ける要素だけを描画したときの a11y ノード。`availability` が `None` なら操作できない項目として描く
    fn body_node(
        cx: &mut VisualTestContext,
        item: ListItem,
        availability: Option<Availability>,
    ) -> Option<accesskit::Node> {
        cx.update(|window, cx| {
            let roving = availability.map(|availability| RovingItem {
                focus_handle: cx.focus_handle(),
                availability,
            });
            let element = item_body(
                item.aria_label("Apple"),
                roving.as_ref().map(|roving| (roving, true)),
                window,
            )
            .into_element();
            let role = element.a11y_role()?;
            let mut node = accesskit::Node::new(role);
            element.write_a11y_info(&mut node);
            Some(node)
        })
    }

    #[gpui::test]
    fn リストは_role_list_と名前で公開される(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        let node = view
            .read_with(cx, |view, _| view.list_node.clone())
            .expect("描画済みであること");
        assert_eq!(node.role(), accesskit::Role::List);
        assert_eq!(node.label(), Some("Fruit"));
    }

    #[gpui::test]
    fn 操作できる項目は種類に応じた_role_と状態で公開される(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, ALL_ENABLED);
        for (kind, role, toggled) in [
            (ListItemKind::Action, accesskit::Role::Button, None),
            (
                ListItemKind::Checkbox(CheckedState::Mixed),
                accesskit::Role::CheckBox,
                Some(accesskit::Toggled::Mixed),
            ),
            (
                ListItemKind::Radio(true),
                accesskit::Role::RadioButton,
                Some(accesskit::Toggled::True),
            ),
        ] {
            let item = ListItem::new("item").kind(kind).on_click(|_, _, _| {});
            let node = body_node(cx, item, Some(Availability::Enabled)).expect("role を持つこと");
            assert_eq!(node.role(), role);
            assert_eq!(node.label(), Some("Apple"));
            assert_eq!(node.toggled(), toggled);
            assert!(!node.is_disabled());
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(node.supports_action(accesskit::Action::Focus));
        }
    }

    #[gpui::test]
    fn 操作できない項目は操作の対象を公開しない(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, ALL_ENABLED);
        let role = body_node(cx, ListItem::new("item").on_click(|_, _, _| {}), None)
            .map(|node| node.role());
        assert!(
            role.is_none() || role == Some(accesskit::Role::GenericContainer),
            "操作できない項目は listitem の中に操作の対象を作らないこと: {role:?}"
        );
    }

    #[gpui::test]
    fn ハンドラを持たない項目はクリック操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, ALL_ENABLED);
        let item = ListItem::new("item").kind(ListItemKind::Action);
        let node = body_node(cx, item, Some(Availability::Enabled)).expect("role を持つこと");
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(node.supports_action(accesskit::Action::Focus));
    }

    #[gpui::test]
    fn 無効な項目は_disabled_として公開され操作を公開しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, ALL_ENABLED);
        let item = || {
            ListItem::new("item")
                .kind(ListItemKind::Action)
                .on_click(|_, _, _| {})
        };
        let node = body_node(cx, item(), Some(Availability::Disabled)).expect("role を持つこと");
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));

        let node =
            body_node(cx, item(), Some(Availability::DisabledFocusable)).expect("role を持つこと");
        assert!(node.is_disabled());
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(
            node.supports_action(accesskit::Action::Focus),
            "無効でも到達できる項目はフォーカス操作を公開すること"
        );
    }

    #[gpui::test]
    fn tab_で最初の操作できる項目に入り_tab_stop_は_1_つだけ(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("action"));
        cx.simulate_keystrokes("tab");
        assert!(is_after_focused(&view, cx));
        cx.simulate_keystrokes("shift-tab");
        assert_eq!(focused(&view, cx), Some("action"));
        assert!(clicks(&view, cx).is_empty());
    }

    #[gpui::test]
    fn tab_で入る先は無効な項目を飛ばす(cx: &mut TestAppContext) {
        let (view, cx) = setup(
            cx,
            [
                Availability::Disabled,
                Availability::Enabled,
                Availability::Enabled,
            ],
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), Some("checkbox"));
    }

    #[gpui::test]
    fn すべての操作できる項目が無効なら_tab_で入らない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, [Availability::Disabled; 3]);
        cx.simulate_keystrokes("tab");
        assert_eq!(focused(&view, cx), None);
        assert!(is_after_focused(&view, cx));
    }

    #[gpui::test]
    fn 上下の矢印キーで操作できる項目の間を移り端で戻る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("checkbox"));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("radio"));
        cx.simulate_keystrokes("down");
        assert_eq!(focused(&view, cx), Some("action"));
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("radio"));
        assert!(
            clicks(&view, cx).is_empty(),
            "矢印キーの移動では項目を操作しないこと"
        );
    }

    #[gpui::test]
    fn 左右の矢印キーと修飾キー付きの矢印キーでは移らない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab right left shift-down ctrl-down");
        assert_eq!(focused(&view, cx), Some("action"));
    }

    #[gpui::test]
    fn home_と_end_で最初と最後の操作できる項目へ移る(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab end");
        assert_eq!(focused(&view, cx), Some("radio"));
        cx.simulate_keystrokes("home");
        assert_eq!(focused(&view, cx), Some("action"));
        let parent_keys = view.read_with(cx, |view, _| view.parent_keys.borrow().clone());
        assert!(
            !parent_keys.iter().any(|key| key == "home" || key == "end"),
            "移ったキー入力は祖先に渡さないこと: {parent_keys:?}"
        );
    }

    #[gpui::test]
    fn 移る先がフォーカスのある項目なら_home_と_end_を祖先に渡す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::Disabled,
            ],
        );
        cx.simulate_keystrokes("tab home end");
        assert_eq!(focused(&view, cx), Some("action"));
        let parent_keys = view.read_with(cx, |view, _| view.parent_keys.borrow().clone());
        assert!(parent_keys.contains(&"home".to_string()));
        assert!(parent_keys.contains(&"end".to_string()));
    }

    #[gpui::test]
    fn 移動は無効な項目を飛ばし無効でも到達できる項目には移る(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            [
                Availability::Enabled,
                Availability::Disabled,
                Availability::DisabledFocusable,
            ],
        );
        cx.simulate_keystrokes("tab down");
        assert_eq!(focused(&view, cx), Some("radio"));
        press_key(cx, "space");
        assert!(clicks(&view, cx).is_empty(), "無効な項目は操作できないこと");
        cx.simulate_keystrokes("up");
        assert_eq!(focused(&view, cx), Some("action"));
    }

    #[gpui::test]
    fn space_では種類によらず操作し_enter_では実行する項目だけを操作する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        cx.simulate_keystrokes("tab");
        for _ in NAMES {
            press_key(cx, "enter");
            press_key(cx, "space");
            cx.simulate_keystrokes("down");
        }
        assert_eq!(
            clicks(&view, cx),
            vec!["action", "action", "checkbox", "radio"]
        );
    }

    #[gpui::test]
    fn クリックで項目を操作しフォーカスを移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        click_row(cx, 2);
        assert_eq!(clicks(&view, cx), vec!["checkbox"]);
        assert_eq!(focused(&view, cx), Some("checkbox"));
    }

    #[gpui::test]
    fn 操作できない項目はクリックしても何もしない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, ALL_ENABLED);
        click_row(cx, 0);
        assert!(clicks(&view, cx).is_empty());
        assert_eq!(focused(&view, cx), None);
    }

    #[gpui::test]
    fn 無効な項目のクリックは操作せず押下を親に伝えない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(
            cx,
            [
                Availability::Disabled,
                Availability::DisabledFocusable,
                Availability::Enabled,
            ],
        );
        click_row(cx, 1);
        assert_eq!(focused(&view, cx), None);
        click_row(cx, 2);
        assert_eq!(
            focused(&view, cx),
            Some("checkbox"),
            "無効でも到達できる項目はクリックでフォーカスされること"
        );
        assert!(clicks(&view, cx).is_empty());
        assert_eq!(
            view.read_with(cx, |view, _| *view.parent_presses.borrow()),
            0
        );
    }
}
