//! 見た目を持たないツールチップ

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, DispatchPhase, Div, Edges, Element, ElementId, Entity,
    FocusHandle, GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, InteractiveElement,
    IntoElement, LayoutId, MouseMoveEvent, ParentElement, Pixels, Position, SharedString, Stateful,
    StatefulInteractiveElement, Style, Styled, Subscription, Window, accesskit, div, point, px,
    size,
};

use crate::is_focus_visible;

/// ツールチップをほかのポップアップより手前に描くための、遅延描画の優先度
const PRIORITY: usize = usize::MAX;

/// トリガーに付ける、見た目を持たないツールチップ
///
/// [`Tooltip::attach`] で付けたトリガーについて、次のいずれかの間、ツールチップの内容をトリガーの上に左右中央をそろえて表示する。
///
/// - トリガーにマウスが乗っている
/// - トリガーのフォーカスを表示すべき状態 ([`crate::is_focus_visible`]) にある
/// - 表示中のツールチップにマウスが乗っている。トリガーとツールチップの間隔の上を通っても消えない
///
/// 上にウィンドウの端まで収まらなければ下に表示し、ウィンドウの左右と下の端からはみ出さないようずらす。
/// 表示中に Escape を押すと、フォーカスとマウスを動かさずに閉じる。その Escape はほかの要素やキー割り当てに届かない。
/// 閉じた後は、トリガーとツールチップからマウスが外れ、トリガーのフォーカスの表示が消えるまで表示しない
///
/// 支援技術には、[`Tooltip::relation`] に従って `text` をトリガーの説明か名前として公開する。
/// 表示中のツールチップは、role `tooltip` で `text` を名前に持つノードとして、ウィンドウの直下に公開される。
/// ツールチップはフォーカスを受け取らないため、内容に押せる要素を置かないこと
pub struct Tooltip {
    focus_handle: FocusHandle,
    text: SharedString,
    relation: TooltipRelation,
    content: AnyElement,
    spacing: Pixels,
}

/// ツールチップの文字列をトリガーの何として公開するか
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipRelation {
    /// トリガーの説明 (`aria_description`) にする。名前は別に与えること
    #[default]
    Description,
    /// トリガーのアクセシブルな名前 (`aria_label`) にする。トリガーに付けた名前は置き換わる。
    /// 文字列がトリガーの名前と同じ場合に、説明として重ねて読まれないよう使う
    Label,
}

impl Tooltip {
    /// `content` を表示するツールチップを作る
    ///
    /// `focus_handle` は、付けるトリガーが `track_focus` に渡したフォーカスにすること。
    /// `text` はツールチップが伝える内容を、`content` の文字列と同じ意味の文字列で渡す
    pub fn new(
        focus_handle: &FocusHandle,
        text: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        Self {
            focus_handle: focus_handle.clone(),
            text: text.into(),
            relation: TooltipRelation::default(),
            content: content.into_any_element(),
            spacing: px(0.),
        }
    }

    /// ツールチップの文字列をトリガーの何として公開するかを設定する。既定は [`TooltipRelation::Description`]
    pub fn relation(mut self, relation: TooltipRelation) -> Self {
        self.relation = relation;
        self
    }

    /// トリガーとツールチップの内容の間隔を設定する。既定は 0
    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = spacing;
        self
    }

    /// `trigger` にこのツールチップを付けて返す
    ///
    /// `trigger` の子の最後に、レイアウトに影響しない要素を加える。`trigger` 1 つに付けるツールチップは 1 つまでにすること
    pub fn attach<E: StatefulInteractiveElement + ParentElement>(self, trigger: E) -> E {
        let anchor = Anchor {
            focus_handle: self.focus_handle,
            text: self.text.clone(),
            content: Some(self.content),
            spacing: self.spacing,
        };
        match self.relation {
            TooltipRelation::Description => trigger.aria_description(self.text),
            TooltipRelation::Label => trigger.aria_label(self.text),
        }
        .child(anchor)
    }
}

/// フレームをまたいで持つツールチップの状態
#[derive(Default)]
struct State {
    trigger_hovered: bool,
    /// 表示中のツールチップにマウスが乗っているか
    popup_hovered: bool,
    /// Escape で閉じた後、表示の条件が一度すべて外れるまで `true`
    dismissed: bool,
    /// 表示中だけ持つ、Escape を受ける購読
    escape: Option<Subscription>,
}

/// `content` を包み、role `tooltip` のノードとしてマウスの出入りを `state` に記録する要素
///
/// 間隔の分だけ内容の周りを広げ、トリガーからツールチップへ動かすマウスが途切れず乗るようにする
fn popup(
    state: Entity<State>,
    text: SharedString,
    spacing: Pixels,
    content: AnyElement,
) -> Stateful<Div> {
    div()
        .id("tooltip-popup")
        .role(accesskit::Role::Tooltip)
        .aria_label(text)
        .p(spacing)
        .on_hover(move |hovered, _, cx| {
            state.update(cx, |state, cx| {
                state.popup_hovered = *hovered;
                cx.notify();
            });
        })
        .child(content)
}

/// トリガーの子に置き、トリガーと同じ範囲でマウスの出入りを受け、表示中のツールチップを配置する要素
struct Anchor {
    focus_handle: FocusHandle,
    text: SharedString,
    content: Option<AnyElement>,
    spacing: Pixels,
}

impl IntoElement for Anchor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Anchor {
    type RequestLayoutState = (Entity<State>, Option<AnyElement>);
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // トリガーの id の下に置くため、トリガーごとに別の状態になる
        let state = window.use_keyed_state("tooltip", cx, |_, _| State::default());
        let focus_visible = is_focus_visible(&self.focus_handle, window, cx);
        let window_id = window.window_handle().window_id();
        let visible = state.update(cx, |state, cx| {
            let requested = state.trigger_hovered || state.popup_hovered || focus_visible;
            if !requested {
                state.dismissed = false;
            }
            let visible = requested && !state.dismissed;
            if !visible {
                state.escape = None;
            } else if state.escape.is_none() {
                let this = cx.weak_entity();
                // フォーカスがほかの要素にあっても閉じられるよう、ウィンドウに届くすべてのキーを先に受ける
                state.escape = Some(cx.intercept_keystrokes(move |event, window, cx| {
                    let keystroke = &event.keystroke;
                    if keystroke.key != "escape"
                        || keystroke.modifiers.modified()
                        || window.window_handle().window_id() != window_id
                    {
                        return;
                    }
                    let Some(this) = this.upgrade() else {
                        return;
                    };
                    this.update(cx, |state, cx| {
                        state.dismissed = true;
                        // 消えたツールチップからは hover の解除が届かないため、ここで外す
                        state.popup_hovered = false;
                        cx.notify();
                    });
                    cx.stop_propagation();
                }));
            }
            visible
        });

        let popup = self.content.take().filter(|_| visible).map(|content| {
            popup(state.clone(), self.text.clone(), self.spacing, content).into_any_element()
        });

        let style = Style {
            position: Position::Absolute,
            inset: Edges::all(px(0.).into()),
            ..Style::default()
        };
        (window.request_layout(style, None, cx), (state, popup))
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (_, popup): &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        if let Some(mut popup) = popup.take() {
            let spacing = self.spacing;
            // トリガーの幅に縛られないよう、ウィンドウのルートと同じく独立に大きさを決める
            let popup_size = popup.layout_as_root(
                size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                window,
                cx,
            );
            let content_size = size(
                popup_size.width - spacing * 2.,
                popup_size.height - spacing * 2.,
            );
            let viewport = window.viewport_size();
            let x = (bounds.center().x - content_size.width / 2.)
                .min(viewport.width - content_size.width)
                .max(px(0.));
            let mut y = bounds.top() - spacing - content_size.height;
            if y < px(0.) {
                y = bounds.bottom() + spacing;
            }
            let y = y.min(viewport.height - content_size.height).max(px(0.));
            window.defer_draw(popup, point(x - spacing, y - spacing), PRIORITY, None);
        }
        hitbox
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        (state, _): &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let state = state.clone();
        let hitbox = hitbox.clone();
        window.on_mouse_event(move |_: &MouseMoveEvent, phase, window, cx| {
            let hovered = hitbox.is_hovered(window);
            if phase == DispatchPhase::Bubble && state.read(cx).trigger_hovered != hovered {
                state.update(cx, |state, cx| {
                    state.trigger_hovered = hovered;
                    cx.notify();
                });
            }
        });
    }
}

/// APG の Tooltip パターン (content/patterns/tooltip/tooltip-pattern.html) と、
/// WCAG 1.4.13 Content on Hover or Focus で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AppContext, Context, Entity, Modifiers, Point, Render, TestAppContext, VisualTestContext,
    };

    use super::*;
    use crate::test_window::add_test_window;

    const DESCRIPTION: &str = "Add to favorites";

    /// 40px 四方のトリガーと、80x20 の黒い内容のツールチップ。フォーカスを移す先の要素を右下に置く
    struct TooltipView {
        trigger_origin: Point<Pixels>,
        focus_handle: FocusHandle,
        other_focus_handle: FocusHandle,
        escapes: Rc<Cell<usize>>,
    }

    impl Render for TooltipView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let escapes = self.escapes.clone();
            let trigger = div()
                .id("trigger")
                .role(accesskit::Role::Button)
                .track_focus(&self.focus_handle.clone().tab_stop(true))
                .absolute()
                .left(self.trigger_origin.x)
                .top(self.trigger_origin.y)
                .size(px(40.));
            div()
                .size_full()
                .on_key_down(move |event, _, _| {
                    if event.keystroke.key == "escape" {
                        escapes.set(escapes.get() + 1);
                    }
                })
                .child(
                    Tooltip::new(
                        &self.focus_handle,
                        DESCRIPTION,
                        div().w(px(80.)).h(px(20.)).bg(gpui::black()),
                    )
                    .spacing(px(4.))
                    .attach(trigger),
                )
                .child(
                    div()
                        .id("other")
                        .track_focus(&self.other_focus_handle.clone().tab_stop(true))
                        .absolute()
                        .left(px(400.))
                        .top(px(400.))
                        .size(px(40.)),
                )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        trigger_origin: Point<Pixels>,
    ) -> (Entity<TooltipView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| TooltipView {
            trigger_origin,
            focus_handle: cx.focus_handle(),
            other_focus_handle: cx.focus_handle(),
            escapes: Rc::default(),
        })
    }

    /// 描いたツールチップの内容の x, y, 幅, 高さ。描いていなければ `None`
    fn popup_bounds(cx: &mut VisualTestContext) -> Option<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .find(|quad| quad.background == gpui::black().into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
        })
    }

    fn move_mouse(cx: &mut VisualTestContext, x: f32, y: f32) {
        cx.simulate_mouse_move(point(px(x), px(y)), None, Modifiers::none());
    }

    /// トリガーの左上が (100, 100) のとき、内容は左右中央の 120 から幅の半分を引いた 80、
    /// 上端から間隔 4 と内容の高さ 20 を引いた 76 に置かれる
    const ABOVE: Option<(f32, f32, f32, f32)> = Some((80.0, 76.0, 80.0, 20.0));

    #[gpui::test]
    fn マウスが乗るとトリガーの上に左右中央をそろえて表示する(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, point(px(100.), px(100.)));
        assert_eq!(popup_bounds(cx), None);
        move_mouse(cx, 120., 120.);
        assert_eq!(popup_bounds(cx), ABOVE);
        move_mouse(cx, 300., 300.);
        assert_eq!(popup_bounds(cx), None, "マウスが外れると消えること");
    }

    /// トリガーの左上が (0, 10) のとき、上に置くと上端が -14 になるため、下端 50 から間隔 4 の 54 に置く。
    /// 左右中央にそろえると左端が -20 になるため、0 にずらす
    #[gpui::test]
    fn 上に収まらなければ下に表示し左右は窓の中に収める(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, point(px(0.), px(10.)));
        move_mouse(cx, 20., 30.);
        assert_eq!(popup_bounds(cx), Some((0.0, 54.0, 80.0, 20.0)));
    }

    #[gpui::test]
    fn トリガーからツールチップへマウスを動かしても表示を続ける(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, point(px(100.), px(100.)));
        move_mouse(cx, 120., 120.);
        // トリガーとツールチップの間隔の上
        move_mouse(cx, 120., 98.);
        assert_eq!(popup_bounds(cx), ABOVE);
        move_mouse(cx, 120., 86.);
        assert_eq!(popup_bounds(cx), ABOVE);
        move_mouse(cx, 300., 300.);
        assert_eq!(popup_bounds(cx), None);
    }

    #[gpui::test]
    fn キーボードでフォーカスすると表示しフォーカスが外れると消す(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, point(px(100.), px(100.)));
        cx.simulate_keystrokes("tab");
        assert_eq!(popup_bounds(cx), ABOVE);
        cx.simulate_keystrokes("tab");
        assert_eq!(popup_bounds(cx), None);
    }

    #[gpui::test]
    fn クリックで得たフォーカスだけでは表示しない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, point(px(100.), px(100.)));
        cx.simulate_click(point(px(120.), px(120.)), Modifiers::none());
        move_mouse(cx, 300., 300.);
        let focus_handle = view.read_with(cx, |view, _| view.focus_handle.clone());
        assert!(cx.update(|window, _| focus_handle.is_focused(window)));
        assert_eq!(popup_bounds(cx), None);
    }

    #[gpui::test]
    fn escape_で閉じると条件が一度外れるまで表示しない(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, point(px(100.), px(100.)));
        move_mouse(cx, 120., 120.);
        cx.simulate_keystrokes("shift-escape");
        assert_eq!(popup_bounds(cx), ABOVE, "修飾キー付きでは閉じないこと");
        cx.simulate_keystrokes("escape");
        assert_eq!(popup_bounds(cx), None);
        move_mouse(cx, 125., 125.);
        assert_eq!(popup_bounds(cx), None, "乗ったまま動かしても表示しないこと");
        move_mouse(cx, 300., 300.);
        move_mouse(cx, 120., 120.);
        assert_eq!(popup_bounds(cx), ABOVE, "乗り直すと表示すること");
    }

    #[gpui::test]
    fn フォーカスがほかにあっても_escape_で閉じその_escape_はほかに届かない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, point(px(100.), px(100.)));
        let other = view.read_with(cx, |view, _| view.other_focus_handle.clone());
        cx.update(|window, cx| other.focus(window, cx));
        move_mouse(cx, 120., 120.);
        cx.simulate_keystrokes("escape");
        assert_eq!(popup_bounds(cx), None);
        let escapes = view.read_with(cx, |view, _| view.escapes.clone());
        assert_eq!(escapes.get(), 0, "閉じた Escape は届かないこと");
        cx.simulate_keystrokes("escape");
        assert_eq!(escapes.get(), 1, "閉じた後の Escape は届くこと");
    }

    #[gpui::test]
    fn トリガーは説明か名前を公開しツールチップは名前を持つ_tooltip_になる(
        cx: &mut TestAppContext,
    ) {
        let focus_handle = cx.update(|cx| cx.focus_handle());
        let trigger = Tooltip::new(&focus_handle, DESCRIPTION, div())
            .attach(div().id("trigger").role(accesskit::Role::Button));
        let mut node = accesskit::Node::new(trigger.a11y_role().expect("role を持つこと"));
        trigger.write_a11y_info(&mut node);
        assert_eq!(node.description(), Some(DESCRIPTION));

        let trigger = Tooltip::new(&focus_handle, DESCRIPTION, div())
            .relation(TooltipRelation::Label)
            .attach(
                div()
                    .id("trigger")
                    .role(accesskit::Role::Button)
                    .aria_label("Favorite"),
            );
        let mut node = accesskit::Node::new(trigger.a11y_role().expect("role を持つこと"));
        trigger.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some(DESCRIPTION));
        assert_eq!(node.description(), None);

        let state = cx.new(|_| State::default());
        let popup = popup(state, DESCRIPTION.into(), px(4.), div().into_any_element());
        let mut node = accesskit::Node::new(popup.a11y_role().expect("role を持つこと"));
        popup.write_a11y_info(&mut node);
        assert_eq!(node.role(), accesskit::Role::Tooltip);
        assert_eq!(node.label(), Some(DESCRIPTION));
    }
}
