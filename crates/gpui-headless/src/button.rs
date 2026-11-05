//! 見た目を持たないボタン

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Interactivity,
    IntoElement, ParentElement, RenderOnce, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, accesskit, div, prelude::FluentBuilder,
};

use crate::accessibility::A11yInfoExt;

/// ボタンを操作できるかどうか
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Availability {
    /// 操作でき、Tab で到達できる
    #[default]
    Enabled,
    /// 操作できず、Tab でも到達できない
    ///
    /// フォーカス中にこの状態になった場合は、フォーカスが外れるまでフォーカスを保つ
    Disabled,
    /// 操作できないが、Tab で到達できる
    ///
    /// 無効な理由を伝えたい場合や、周囲から存在を推測できず見つけてもらう必要がある場合に使う
    DisabledFocusable,
}

impl Availability {
    fn is_focusable(self) -> bool {
        matches!(
            self,
            Availability::Enabled | Availability::DisabledFocusable
        )
    }

    fn is_disabled(self) -> bool {
        matches!(
            self,
            Availability::Disabled | Availability::DisabledFocusable
        )
    }
}

/// ボタンが押されたときの処理
pub trait OnClickHandler: 'static {
    /// 押されたときに呼ぶ処理を持つかどうか
    ///
    /// `false` の場合、ボタンは支援技術にクリック操作を公開せず、[`OnClickHandler::call`] も呼ばない
    const CALLABLE: bool;

    /// ボタンが押されたときに呼ばれる
    fn call(&self, _event: &ClickEvent, _window: &mut Window, _cx: &mut App) {}
}

/// 押されても何も呼ばないことを表す [`OnClickHandler`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoHandler;

impl OnClickHandler for NoHandler {
    const CALLABLE: bool = false;
}

impl<F> OnClickHandler for F
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    const CALLABLE: bool = true;
    fn call(&self, event: &ClickEvent, window: &mut Window, cx: &mut App) {
        self(event, window, cx)
    }
}

/// 見た目を持たないボタン
///
/// 支援技術には role `button` として公開される。アクセシブルな名前は、子の内容か `aria_label` で与えること。
/// マウスのクリック・フォーカス中の Enter と Space・支援技術からのクリック操作でハンドラを呼ぶ。
/// 見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct Button<F: OnClickHandler> {
    id: ElementId,
    inner: Stateful<Div>,
    on_click: F,
    provided_focus_handle: Option<FocusHandle>,
    availability: Availability,
}

impl Button<NoHandler> {
    /// 押されても何も呼ばないボタンを作る
    ///
    /// ハンドラを持たないため、支援技術にはクリック操作を公開しない。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。[`Button::track_focus`] を指定しない場合、
    /// `id` が変わるか 1 フレームでも描画されないとフォーカスが外れる
    pub fn new(id: impl Into<ElementId>) -> Self {
        Button::with_handler(id, NoHandler)
    }
}

impl<F> Button<F>
where
    F: OnClickHandler,
{
    /// 押されたときに `on_click` を呼ぶボタンを作る
    pub fn with_handler(id: impl Into<ElementId>, on_click: F) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            on_click,
            provided_focus_handle: None,
            availability: Availability::Enabled,
        }
    }

    /// ボタンを無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`Button::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// ボタンを無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効なボタンは押されてもハンドラを呼ばず、Tab でも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`Button::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// `focus_handle` をこのボタンのフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合はボタンが内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| {
                    cx.focus_handle().tab_index(0).tab_stop(true)
                })
                .read(cx)
                .clone()
        })
    }
}

impl<F> RenderOnce for Button<F>
where
    F: OnClickHandler,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.focus_handle(window, cx);
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_click = self.on_click;
        self.inner
            .role(accesskit::Role::Button)
            .when(focusable, |this| {
                this.track_focus(&focus_handle.tab_stop(true))
            })
            .when(F::CALLABLE && !disabled, |this| {
                this.on_click(move |event, window, cx| on_click.call(event, window, cx))
            })
            .a11y_info(move |node: &mut accesskit::Node| {
                if disabled {
                    node.set_disabled();
                }
            })
    }
}

impl<F: OnClickHandler> Styled for Button<F> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl<F: OnClickHandler> ParentElement for Button<F> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<F: OnClickHandler> InteractiveElement for Button<F> {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl<F: OnClickHandler> StatefulInteractiveElement for Button<F> {}
