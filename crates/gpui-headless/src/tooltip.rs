//! 見た目を持たないツールチップ

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Div, Element, ElementId, Entity, FocusHandle,
    GlobalElementId, HoverListenerMode, InspectorElementId, InteractiveElement, IntoElement,
    LayoutId, ParentElement, Pixels, SharedString, Stateful, StatefulInteractiveElement, Styled,
    Subscription, Window, accesskit, div, point, px, size,
};

use crate::is_focus_visible;

/// ツールチップをほかのポップアップより手前に描くための、遅延描画の優先度
const PRIORITY: usize = usize::MAX;

/// トリガーに付ける、見た目を持たないツールチップ
///
/// [`Tooltip::attach`] で付けたトリガーについて、次のいずれかの間、ツールチップの内容をトリガーの上に左右中央をそろえて表示する
///
/// - トリガーにマウスが乗っている
/// - トリガーのフォーカスを表示すべき状態 ([`crate::is_focus_visible`]) にある
/// - 表示中のツールチップにマウスが乗っている。トリガーとツールチップの間隔の上を通っても消えない
///
/// マウスが乗っているかは、キー入力の後も、マウスを動かさずに配置が変わった後も、最後に伝わったマウスの位置で決める。
/// マウスが移動を伝えずにウィンドウから出た場合も、その位置に残っているものとして扱う。
/// 表示中のツールチップの内容と、その周りの間隔の幅の帯の上では、クリックや hover は背後の要素に届かず、スクロールは届く
///
/// 内容に与える幅はウィンドウの幅までにする。
/// 上にウィンドウの端まで収まらなければ下に表示し、ウィンドウの左右と下の端からはみ出さないようずらす。
/// 表示中に Escape を押すと、フォーカスとマウスを動かさずに閉じる。その Escape はほかの要素やキー割り当てに届かない。
/// 修飾キー付きの Escape では閉じず、ほかの要素に届く。
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
    /// `text` はツールチップが伝える内容を、`content` の文字列と同じ意味の文字列で渡す。
    /// `content` はウィンドウからはみ出さないよう、固定の幅を持たせず、与えられた幅で折り返すようにすること
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
    /// Escape で閉じてから、次の描画で表示の条件を決めるまで `true`
    just_dismissed: bool,
    /// 表示中だけ持つ、Escape を受ける購読
    escape: Option<Subscription>,
    /// 最後に描いたトリガーの範囲
    trigger_bounds: Bounds<Pixels>,
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
        .block_mouse_except_scroll()
        // 既定ではキー入力の後に hover が外れ、Escape 以外のキーでも消えるため、キー入力に関わらず位置で決める
        .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
        .on_hover(move |hovered, _, cx| {
            state.update(cx, |state, cx| {
                state.popup_hovered = *hovered;
                cx.notify();
            });
        })
        .child(content)
}

/// トリガーの子に置き、トリガーと同じ範囲でマウスの出入りを `State::trigger_hovered` に記録し、表示中のツールチップを配置する要素
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
    /// 状態と、トリガーと同じ範囲でマウスの出入りを受ける要素と、表示中のツールチップ
    type RequestLayoutState = (Entity<State>, AnyElement, Option<AnyElement>);
    type PrepaintState = ();

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
        let mouse_position = window.mouse_position();
        let visible = state.update(cx, |state, cx| {
            let requested = state.trigger_hovered || state.popup_hovered || focus_visible;
            // ツールチップがトリガーを覆っていた場合、閉じた直後の描画ではトリガーの hover がまだ届いていないため、マウスの位置で代える。
            // その描画でトリガーの hover が届けば、次の描画からは hover で決まる
            let pointer_on_trigger = std::mem::take(&mut state.just_dismissed)
                && state.trigger_bounds.contains(&mouse_position);
            if !requested && !pointer_on_trigger {
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
                        state.just_dismissed = true;
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
            let max_width = window.viewport_size().width + self.spacing * 2.;
            popup(state.clone(), self.text.clone(), self.spacing, content)
                .max_w(max_width)
                .into_any_element()
        });

        // gpui の on_hover は、マウスを動かさずに配置が変わった場合も描画のときに決め直して呼ばれる
        let area_state = state.clone();
        let mut area = div()
            .id("tooltip-trigger")
            .absolute()
            .inset_0()
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(move |hovered, _, cx| {
                area_state.update(cx, |state, cx| {
                    state.trigger_hovered = *hovered;
                    cx.notify();
                });
            })
            .into_any_element();
        let layout_id = area.request_layout(window, cx);
        (layout_id, (state, area, popup))
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (state, area, popup): &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        state.update(cx, |state, _| state.trigger_bounds = bounds);
        area.prepaint(window, cx);
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
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        (_, area, _): &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        area.paint(window, cx);
    }
}

/// APG の Tooltip パターン (content/patterns/tooltip/tooltip-pattern.html) と、
/// WCAG 1.4.13 Content on Hover or Focus で期待される振る舞い
#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AppContext, Context, Entity, Modifiers, Point, Render, TestAppContext, VisualTestContext,
        prelude::FluentBuilder,
    };

    use super::*;
    use crate::test_window::add_test_window;

    const DESCRIPTION: &str = "Add to favorites";

    /// 40px 四方のトリガーと、黒い内容のツールチップ。フォーカスを移す先の要素を右下に、
    /// クリックを数える要素を上に置く。`cover` ならトリガーの下半分をマウスの操作を遮る要素で覆う
    ///
    /// 内容は 40x20 の箱を 2 つ横に並べた 80x20 で、幅が足りなければ折り返す
    struct TooltipView {
        trigger_origin: Point<Pixels>,
        cover: bool,
        focus_handle: FocusHandle,
        other_focus_handle: FocusHandle,
        escapes: Rc<Cell<usize>>,
        behind_clicks: Rc<Cell<usize>>,
    }

    impl Render for TooltipView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let escapes = self.escapes.clone();
            let behind_clicks = self.behind_clicks.clone();
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
                        div()
                            .flex()
                            .flex_wrap()
                            .bg(gpui::black())
                            .children((0..2).map(|_| div().size(px(40.)).h(px(20.)))),
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
                .when(self.cover, |this| {
                    this.child(
                        div()
                            .id("cover")
                            .absolute()
                            .left(self.trigger_origin.x)
                            .top(self.trigger_origin.y + px(20.))
                            .w(px(40.))
                            .h(px(20.))
                            .occlude(),
                    )
                })
                .child(
                    div()
                        .id("behind")
                        .absolute()
                        .left(px(60.))
                        .top(px(60.))
                        .w(px(120.))
                        .h(px(30.))
                        .on_click(move |_, _, _| behind_clicks.set(behind_clicks.get() + 1)),
                )
        }
    }

    fn setup(
        cx: &mut TestAppContext,
        trigger_origin: Point<Pixels>,
    ) -> (Entity<TooltipView>, &mut VisualTestContext) {
        add_test_window(cx, |_, cx| TooltipView {
            trigger_origin,
            cover: false,
            focus_handle: cx.focus_handle(),
            other_focus_handle: cx.focus_handle(),
            escapes: Rc::default(),
            behind_clicks: Rc::default(),
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
    fn マウスが乗っていればキー入力の後も表示を続ける(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, point(px(100.), px(100.)));
        let other = view.read_with(cx, |view, _| view.other_focus_handle.clone());
        cx.update(|window, cx| other.focus(window, cx));
        move_mouse(cx, 120., 120.);
        cx.simulate_keystrokes("a");
        assert_eq!(popup_bounds(cx), ABOVE, "トリガーの上");
        move_mouse(cx, 120., 86.);
        cx.simulate_keystrokes("shift-escape a");
        assert_eq!(popup_bounds(cx), ABOVE, "ツールチップの上");
    }

    #[gpui::test]
    fn マウスを動かさずにトリガーが動くとマウスの位置で決め直す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, point(px(100.), px(100.)));
        move_mouse(cx, 120., 120.);
        view.update(cx, |view, cx| {
            view.trigger_origin = point(px(300.), px(300.));
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(popup_bounds(cx), None, "マウスの下から外れると消えること");
        view.update(cx, |view, cx| {
            view.trigger_origin = point(px(100.), px(100.));
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(popup_bounds(cx), ABOVE, "マウスの下に来ると表示すること");
    }

    /// クリックを数える要素は (60, 60) から 120x30 の範囲で、ツールチップは間隔を含めて (76, 72) から下にある
    #[gpui::test]
    fn ツールチップの上のクリックは背後に届かない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, point(px(100.), px(100.)));
        let behind_clicks = view.read_with(cx, |view, _| view.behind_clicks.clone());
        move_mouse(cx, 120., 120.);
        cx.simulate_click(point(px(120.), px(86.)), Modifiers::none());
        assert_eq!(popup_bounds(cx), ABOVE);
        assert_eq!(behind_clicks.get(), 0);
        cx.simulate_click(point(px(120.), px(65.)), Modifiers::none());
        assert_eq!(
            behind_clicks.get(),
            1,
            "ツールチップの外のクリックは届くこと"
        );
    }

    /// 幅 60 のウィンドウでは内容の幅を 60 にし、箱を縦に折り返して高さ 40 にする。
    /// トリガーの上端 100 から間隔 4 と高さ 40 を引いた 56 に置く
    #[gpui::test]
    fn 内容の幅はウィンドウの幅までにする(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, point(px(10.), px(100.)));
        cx.simulate_resize(size(px(60.), px(600.)));
        move_mouse(cx, 30., 120.);
        assert_eq!(popup_bounds(cx), Some((0.0, 56.0, 60.0, 40.0)));
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

    /// 高さ 60 のウィンドウで左上が (100, 20) のトリガーには、上にも下にも収まらないため、
    /// 内容を下端に寄せた y 40 に置き、トリガーの下半分を覆う
    #[gpui::test]
    fn トリガーを覆うツールチップを_escape_で閉じてもマウスが離れるまで表示しない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, point(px(100.), px(20.)));
        cx.simulate_resize(size(px(300.), px(60.)));
        let other = view.read_with(cx, |view, _| view.other_focus_handle.clone());
        cx.update(|window, cx| other.focus(window, cx));
        move_mouse(cx, 120., 50.);
        assert_eq!(popup_bounds(cx), Some((80.0, 40.0, 80.0, 20.0)));
        cx.simulate_keystrokes("escape");
        assert_eq!(popup_bounds(cx), None);
        move_mouse(cx, 120., 30.);
        assert_eq!(
            popup_bounds(cx),
            None,
            "トリガーの上で動かしても表示しないこと"
        );
        move_mouse(cx, 250., 10.);
        move_mouse(cx, 120., 30.);
        assert_eq!(popup_bounds(cx), Some((80.0, 40.0, 80.0, 20.0)));
    }

    /// 覆う要素の上のマウスはトリガーの範囲の中にあるが、トリガーには乗っていない
    #[gpui::test]
    fn キーボードで出したツールチップを_escape_で閉じてもフォーカスし直すと表示する(
        cx: &mut TestAppContext,
    ) {
        for (cover, (x, y)) in [(false, (300., 300.)), (true, (120., 130.))] {
            let (view, cx) = setup(cx, point(px(100.), px(100.)));
            view.update(cx, |view, cx| {
                view.cover = cover;
                cx.notify();
            });
            move_mouse(cx, x, y);
            assert_eq!(popup_bounds(cx), None, "cover: {cover}");
            cx.simulate_keystrokes("tab");
            assert_eq!(popup_bounds(cx), ABOVE, "cover: {cover}");
            cx.simulate_keystrokes("escape");
            assert_eq!(popup_bounds(cx), None, "cover: {cover}");
            cx.simulate_keystrokes("tab shift-tab");
            assert_eq!(popup_bounds(cx), ABOVE, "cover: {cover}");
        }
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
