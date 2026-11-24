//! 1 行の文字列を編集する、見た目を持たない部品

use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, DispatchPhase, Div, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, GlobalElementId, Hsla,
    InspectorElementId, InteractiveElement, Interactivity, IntoElement, KeyBinding, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, ParentElement, Pixels,
    Point, RenderOnce, ShapedLine, SharedString, Stateful, StatefulInteractiveElement, Style,
    StyleRefinement, Styled, TextAlign, UTF16Selection, UnderlineStyle, Window, accesskit, actions,
    div, fill, point, prelude::FluentBuilder, px, relative, size,
};

use crate::{Availability, InteractionSource, accessibility::A11yInfoExt};

/// [`TextField`] のキー割り当てを有効にする key context
const KEY_CONTEXT: &str = "TextField";

/// カーソルの幅
const CARET_WIDTH: Pixels = px(2.);

/// 支援技術に公開する 1 つの TextRun の最大の文字数。AccessKit は TextRun の中の文字の位置を u8 で表す
const A11Y_RUN_LEN: usize = 255;

actions!(
    gpui_headless,
    [
        /// カーソルの前の 1 文字か、選択している範囲を消す
        DeleteBackward,
        /// カーソルの後の 1 文字か、選択している範囲を消す
        DeleteForward,
        /// カーソルの前の 1 語か、選択している範囲を消す
        DeleteWordBackward,
        /// カーソルの後の 1 語か、選択している範囲を消す
        DeleteWordForward,
        /// カーソルを 1 文字左へ移す。選択していれば選択の始まりへ移す
        MoveLeft,
        /// カーソルを 1 文字右へ移す。選択していれば選択の終わりへ移す
        MoveRight,
        /// カーソルを前の語の始まりへ移す
        MoveWordLeft,
        /// カーソルを次の語の終わりへ移す
        MoveWordRight,
        /// カーソルを先頭へ移す
        MoveToStart,
        /// カーソルを末尾へ移す
        MoveToEnd,
        /// 選択を 1 文字左へ広げる
        SelectLeft,
        /// 選択を 1 文字右へ広げる
        SelectRight,
        /// 選択を前の語の始まりまで広げる
        SelectWordLeft,
        /// 選択を次の語の終わりまで広げる
        SelectWordRight,
        /// 選択を先頭まで広げる
        SelectToStart,
        /// 選択を末尾まで広げる
        SelectToEnd,
        /// すべての文字列を選択する
        SelectAll,
        /// 選択している範囲をクリップボードに写す
        Copy,
        /// 選択している範囲をクリップボードに写して消す
        Cut,
        /// クリップボードの文字列を、選択している範囲に置き換えて入れる
        Paste,
        /// 入力した文字列を確定する
        Submit,
    ]
);

/// [`TextField`] のキー割り当てを登録する。[`crate::init`] から 1 回呼ぶ
pub(crate) fn init(cx: &mut App) {
    let context = Some(KEY_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("enter", Submit, context),
        KeyBinding::new("backspace", DeleteBackward, context),
        KeyBinding::new("shift-backspace", DeleteBackward, context),
        KeyBinding::new("delete", DeleteForward, context),
        KeyBinding::new("left", MoveLeft, context),
        KeyBinding::new("right", MoveRight, context),
        KeyBinding::new("home", MoveToStart, context),
        KeyBinding::new("end", MoveToEnd, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("shift-home", SelectToStart, context),
        KeyBinding::new("shift-end", SelectToEnd, context),
    ]);
    // macOS は Command を主な修飾キーにし、語の単位の移動に Option を使う
    let bindings = if cfg!(target_os = "macos") {
        vec![
            KeyBinding::new("alt-backspace", DeleteWordBackward, context),
            KeyBinding::new("alt-delete", DeleteWordForward, context),
            KeyBinding::new("alt-left", MoveWordLeft, context),
            KeyBinding::new("alt-right", MoveWordRight, context),
            KeyBinding::new("alt-shift-left", SelectWordLeft, context),
            KeyBinding::new("alt-shift-right", SelectWordRight, context),
            KeyBinding::new("cmd-left", MoveToStart, context),
            KeyBinding::new("cmd-right", MoveToEnd, context),
            KeyBinding::new("cmd-shift-left", SelectToStart, context),
            KeyBinding::new("cmd-shift-right", SelectToEnd, context),
            KeyBinding::new("cmd-a", SelectAll, context),
            KeyBinding::new("cmd-c", Copy, context),
            KeyBinding::new("cmd-x", Cut, context),
            KeyBinding::new("cmd-v", Paste, context),
        ]
    } else {
        vec![
            KeyBinding::new("ctrl-backspace", DeleteWordBackward, context),
            KeyBinding::new("ctrl-delete", DeleteWordForward, context),
            KeyBinding::new("ctrl-left", MoveWordLeft, context),
            KeyBinding::new("ctrl-right", MoveWordRight, context),
            KeyBinding::new("ctrl-shift-left", SelectWordLeft, context),
            KeyBinding::new("ctrl-shift-right", SelectWordRight, context),
            KeyBinding::new("ctrl-a", SelectAll, context),
            KeyBinding::new("ctrl-c", Copy, context),
            KeyBinding::new("ctrl-x", Cut, context),
            KeyBinding::new("ctrl-v", Paste, context),
        ]
    };
    cx.bind_keys(bindings);
}

type ChangeHandler = Rc<dyn Fn(&TextEdit, &mut Window, &mut App)>;
type SubmitHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// [`TextField::on_edit`] に渡す 1 回の編集
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEdit {
    /// 編集する前の文字列
    pub old_text: SharedString,
    /// `old_text` のうち置き換えた範囲。UTF-8 のバイトの位置
    pub range: Range<usize>,
    /// `range` に入れた文字列。消しただけなら空
    pub inserted: SharedString,
    /// 編集した後の文字列
    pub new_text: SharedString,
    /// 編集した後の文字列が、IME で変換中の文字列を含むか
    pub composing: bool,
}

/// 1 行の文字列を編集する、見た目を持たないテキストフィールド
///
/// 文字列は呼び出し側が持ち、[`TextField::new`] に毎回渡す。部品自身は文字列を変えない。
/// 次の操作で、[`TextField::on_change`] のハンドラに編集した後の文字列を求める。
/// IME で変換中の文字列も、編集した後の文字列に含めて求め、変換を確定したときにも求める
///
/// - 文字の入力と IME による入力。改行は入れない
/// - Backspace・Delete で 1 文字、Ctrl (macOS では Option) と合わせて 1 語を消す。選択していれば選択している範囲を消す
/// - クリップボードとの切り取り・コピー・貼り付け (Ctrl または Command と X・C・V)。貼り付ける改行は空白に変える
///
/// 次の操作ではカーソルと選択を部品が内部で変える。文字列は変えない
///
/// - 左右の矢印キーで 1 文字、Ctrl (macOS では Option) と合わせて 1 語、Home・End で先頭と末尾へカーソルを移す。
///   Shift と合わせると選択を広げる。Ctrl (macOS では Command) と A ですべてを選択する
/// - マウスの左ボタンを押した位置へカーソルを移し、押したまま動かすと部品の外でも選択を広げる。
///   Shift と合わせると選択を広げ、2 回続けて押すと語を、3 回続けて押すとすべてを選択する
/// - 支援技術からの選択の設定 (SetTextSelection)
///
/// [`TextField::on_submit`] を設定すると、IME で変換中でない間の修飾キーなしの Enter で、そのハンドラに文字列を渡す
///
/// 文字列が部品の幅に収まらない場合は、カーソルが見える位置まで横にずらして描く。
/// 文字の移動と消去は Unicode のスカラー値の単位で行い、書記素クラスタを考慮しない
///
/// 支援技術には role `textbox` と、文字列・選択として公開される。
/// アクセシブルな名前は `aria_label` で与えること。
/// 文字列とカーソル・選択は部品が描き、書体と文字の色と、文字列が部品の幅に収まる間の横の揃えは、継承した text style に従う。
/// ほかの見た目は [`Styled`] と [`InteractiveElement`] で呼び出し側が付ける
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    inner: Stateful<Div>,
    value: SharedString,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
    placeholder: Option<SharedString>,
    error: Option<SharedString>,
    read_only: bool,
    availability: Availability,
    provided_focus_handle: Option<FocusHandle>,
    group: Option<SharedString>,
    selection_color: Option<Hsla>,
    caret_color: Option<Hsla>,
}

impl TextField {
    /// `value` を表示するテキストフィールドを作る
    ///
    /// `value` の改行は空白として描く。最初に描いたときは、カーソルを `value` の末尾に置く。
    /// `id` は同じ親の中で一意にし、描画のたびに同じ値を渡すこと。`id` が変わるか 1 フレームでも描画されないと、
    /// カーソルと選択が失われる。[`TextField::track_focus`] を指定しない場合はフォーカスも外れる
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            inner: div().id(id),
            value: value.into(),
            on_change: None,
            on_submit: None,
            placeholder: None,
            error: None,
            read_only: false,
            availability: Availability::Enabled,
            provided_focus_handle: None,
            group: None,
            selection_color: None,
            caret_color: None,
        }
    }

    /// 文字列を編集する操作をされたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には編集した後の文字列を渡す。表示を変えるには、`handler` で呼び出し側の状態を更新し、
    /// 次の描画で [`TextField::new`] に渡すこと。渡さなかった場合、編集は取り消される。
    /// 編集した後の文字列とも編集する前の文字列とも違う文字列を渡した場合は、選択をやめ、
    /// カーソルを渡した文字列の対応する位置に置く。
    /// 編集した後の文字列と先頭か末尾で一致する部分にあったカーソルは、その部分の中の位置を保ち、
    /// それ以外のカーソルは、渡した文字列の変わった部分の末尾に置く。
    /// 設定しない場合、部品は文字列を編集させず、支援技術には read-only として公開される。
    /// [`TextField::on_edit`] と両方を呼んだ場合は、後に呼んだ方のハンドラだけを呼ぶ
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(move |edit: &TextEdit, window, cx| {
            handler(&edit.new_text, window, cx)
        }));
        self
    }

    /// [`TextField::on_change`] と同じ操作で、編集した後の文字列の代わりに、置き換えた範囲と入れた文字列を含む編集を渡す
    ///
    /// 編集を受け入れなかった場合と、書き換えた文字列を渡した場合のカーソルの扱いは [`TextField::on_change`] と同じ。
    /// 入力位置や貼り付けかどうか、IME で変換中かによって受け入れ方を変える場合に使う。
    /// IME で変換中の編集は `composing` を `true` にして渡す。変換の確定は、変換を始める前の文字列の範囲を
    /// 確定した文字列で置き換えた 1 回の編集として、`composing` を `false` にして渡す。
    /// IME からの空の確定は変換の取り消しとして、変換を始める前の文字列と選択に戻し、文字列を変えない編集を渡す。
    /// 変換中の範囲の外も置き換えた場合は、文字列全体を置き換えた編集として渡す。
    /// 変換中の編集を受け入れなかった場合は変換を終える。
    /// 受け入れなかった編集で文字列が戻る場合は、選択も編集する前に戻す。
    /// [`TextField::on_change`] と両方を呼んだ場合は、後に呼んだ方のハンドラだけを呼ぶ
    pub fn on_edit(mut self, handler: impl Fn(&TextEdit, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// フォーカス中に修飾キーなしの Enter を押されたときに `handler` を呼ぶ。複数回呼んだ場合は最後に渡した `handler` だけを呼ぶ
    ///
    /// `handler` には Enter を押した時点で部品が持つ文字列を渡す。直前の編集を呼び出し側が受け入れていない場合も、
    /// 次の描画までは編集した後の文字列になる。IME で変換中の間と、無効な間は呼ばない。
    /// 読み取り専用の間と、[`TextField::on_change`] を設定していない間も呼ぶ
    pub fn on_submit(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    /// 支援技術に、入力する内容の例やヒントとして `placeholder` を公開する。部品は描かない
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 文字列にエラーがあることを示す。`None` を渡すとエラーのない状態に戻す。既定ではエラーなし
    ///
    /// `Some` の場合、支援技術には invalid として公開され、`message` が説明 (description) の先頭に加わる。
    /// 部品は `message` を表示しないため、エラーの内容は呼び出し側がテキストとしても表示すること
    pub fn error(mut self, message: Option<SharedString>) -> Self {
        self.error = message;
        self
    }

    /// 文字列を編集させないようにする。`false` を渡すと編集できるように戻す
    ///
    /// 読み取り専用の部品も、フォーカス・カーソルの移動・選択・コピーはでき、支援技術には read-only として公開される
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// 部品を無効にするか、無効でも Tab で到達できるようにするかを設定する。既定では有効
    ///
    /// 無効な部品は操作されてもカーソルも選択も変えず、ハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// 支援技術には disabled として公開される。
    /// 有効か無効かだけを切り替える場合は [`TextField::disabled`] を使う
    pub fn availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
        self
    }

    /// 部品を無効にする。`false` を渡すと有効に戻す
    ///
    /// 無効な部品は操作されてもカーソルも選択も変えず、ハンドラを呼ばず、マウスの左ボタンの押下を親に伝えず、
    /// Tab でも到達できず、支援技術には disabled として公開される。
    /// フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ。
    /// 無効でも Tab で到達できるようにしたい場合は [`TextField::availability`] を使う
    pub fn disabled(self, disabled: bool) -> Self {
        self.availability(if disabled {
            Availability::Disabled
        } else {
            Availability::Enabled
        })
    }

    /// 選択している範囲の背景を `color` で塗る。既定では文字の色を不透明度 0.25 にした色
    pub fn selection_color(mut self, color: impl Into<Hsla>) -> Self {
        self.selection_color = Some(color.into());
        self
    }

    /// カーソルを `color` で描く。既定では文字の色
    pub fn caret_color(mut self, color: impl Into<Hsla>) -> Self {
        self.caret_color = Some(color.into());
        self
    }

    /// `focus_handle` をこの部品のフォーカスとして使う
    ///
    /// 呼び出し側から `focus_handle` でフォーカスの移動や判定ができる。`focus_handle` は Tab で到達できるように設定される。
    /// 指定しない場合は部品が内部でフォーカスを持つ
    pub fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.provided_focus_handle = Some(focus_handle.clone());
        self
    }

    /// `source` のフォーカスをこの部品のフォーカスとして使い、`source` の group の名前をこの部品に付ける
    ///
    /// 子孫の要素から、`source` を通してこの部品の hover とフォーカスを参照できる。
    /// [`TextField::track_focus`] と両方を呼んだ場合は、後に呼んだ方のフォーカスを使う
    pub fn interaction_source(mut self, source: &InteractionSource) -> Self {
        self.provided_focus_handle = Some(source.focus_handle().clone());
        self.group = Some(source.group().clone());
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.provided_focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(self.id.clone(), cx, |_, cx| {
                    cx.focus_handle().tab_index(0).tab_stop(true)
                })
                .read(cx)
                .clone()
        });
        // フォーカス中に無効になった場合は、フォーカスが外れるまでフォーカスを保つ
        let focusable = self.availability.is_focusable() || focus_handle.is_focused(window);
        let disabled = self.availability.is_disabled();
        let on_change = self.on_change.filter(|_| !disabled && !self.read_only);
        let editable = on_change.is_some();

        // 最初の描画では、カーソルを文字列の末尾に置く
        let initial_end = self.value.len();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| EditState {
            model: EditModel {
                text: self.value.to_string(),
                selection: initial_end..initial_end,
                ..EditModel::default()
            },
            focus_handle: focus_handle.clone(),
            on_change: None,
            before_edit: None,
            composition: None,
            layout: None,
            scroll: px(0.),
            selecting: false,
        });
        let (synced, model) = state.update(cx, |state, _| {
            state.focus_handle = focus_handle.clone();
            state.on_change = on_change.clone();
            if disabled {
                state.selecting = false;
            }
            let before_edit = state.before_edit.take();
            // 編集した後の文字列と、そのカーソル
            let edited = before_edit
                .as_ref()
                .map(|_| (state.model.text.clone(), state.model.cursor()));
            let synced = state.model.sync(&self.value);
            if synced {
                state.composition = None;
            }
            // 受け入れられなかった編集は、選択も編集する前に戻す
            if let Some(before_edit) =
                before_edit.filter(|before| synced && before.text == self.value.as_ref())
            {
                state.model.selection = before_edit.selection;
                state.model.reversed = before_edit.reversed;
            } else if let Some((edited, cursor)) = edited.filter(|_| synced) {
                // 呼び出し側が書き換えた編集は、カーソルを書き換えた後の文字列の対応する位置へ移す
                let cursor = remap_cursor(&edited, cursor, &self.value);
                state.model.select(cursor, cursor);
            }
            (synced, state.model.clone())
        });
        if synced {
            window.notify_text_input_changed(&focus_handle);
        }

        let runs: Rc<[A11yRun]> = a11y_runs(&model.text).into();
        // 支援技術に公開した TextRun の id。a11y ツリーを組み立てるときに書き、選択の設定で読む
        let run_ids = Rc::new(RefCell::new(Vec::new()));

        let mut inner = self
            .inner
            .role(accesskit::Role::TextInput)
            .key_context(KEY_CONTEXT)
            .when_some(self.group, |this, group| this.group(group))
            .when(focusable, |this| {
                this.track_focus(&focus_handle.clone().tab_stop(true))
            })
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                let focus_handle = focus_handle.clone();
                move |event: &MouseDownEvent, window, cx| {
                    // 選択のドラッグで親の on_click や on_drag を動かさないよう、押下を親に伝えない。
                    // 伝播を止めると track_focus によるクリック時のフォーカス移動も呼ばれないため、ここで移す
                    cx.stop_propagation();
                    if focusable {
                        window.focus(&focus_handle, cx);
                    }
                    if disabled {
                        return;
                    }
                    if editable {
                        window.request_virtual_keyboard();
                    }
                    state.update(cx, |state, cx| {
                        let offset = state.offset_for_x(event.position.x);
                        let text = &state.model.text;
                        match event.click_count {
                            2 => {
                                let word = word_range(text, state.char_at_x(event.position.x));
                                state.model.select(word.start, word.end);
                            }
                            3.. => state.model.select(0, text.len()),
                            _ if event.modifiers.shift => state.model.select_to(offset),
                            _ => state.model.move_to(offset),
                        }
                        state.selecting = true;
                        state.selection_changed(window, cx);
                    });
                }
            });

        if !disabled && let Some(on_submit) = self.on_submit {
            inner = inner.on_action(listener(&state, move |state, _: &Submit, window, cx| {
                // 変換中の Enter は IME が確定に使う
                if state.model.marked.is_none() {
                    on_submit(&state.model.text.clone().into(), window, cx);
                }
            }));
        }
        if !disabled {
            inner = inner
                .cursor(CursorStyle::IBeam)
                .on_action(listener(&state, |state, _: &DeleteBackward, window, cx| {
                    let range = state.deletion(previous_char);
                    state.edit(range, "", window, cx);
                }))
                .on_action(listener(&state, |state, _: &DeleteForward, window, cx| {
                    let range = state.deletion(next_char);
                    state.edit(range, "", window, cx);
                }))
                .on_action(listener(
                    &state,
                    |state, _: &DeleteWordBackward, window, cx| {
                        let range = state.deletion(previous_word);
                        state.edit(range, "", window, cx);
                    },
                ))
                .on_action(listener(
                    &state,
                    |state, _: &DeleteWordForward, window, cx| {
                        let range = state.deletion(next_word);
                        state.edit(range, "", window, cx);
                    },
                ))
                .on_action(listener(&state, |state, _: &MoveLeft, window, cx| {
                    let model = &mut state.model;
                    let offset = if model.selection.is_empty() {
                        previous_char(&model.text, model.cursor())
                    } else {
                        model.selection.start
                    };
                    model.move_to(offset);
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &MoveRight, window, cx| {
                    let model = &mut state.model;
                    let offset = if model.selection.is_empty() {
                        next_char(&model.text, model.cursor())
                    } else {
                        model.selection.end
                    };
                    model.move_to(offset);
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &MoveWordLeft, window, cx| {
                    let model = &mut state.model;
                    model.move_to(previous_word(&model.text, model.cursor()));
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &MoveWordRight, window, cx| {
                    let model = &mut state.model;
                    model.move_to(next_word(&model.text, model.cursor()));
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &MoveToStart, window, cx| {
                    state.model.move_to(0);
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &MoveToEnd, window, cx| {
                    state.model.move_to(state.model.text.len());
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &SelectLeft, window, cx| {
                    let model = &mut state.model;
                    model.select_to(previous_char(&model.text, model.cursor()));
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &SelectRight, window, cx| {
                    let model = &mut state.model;
                    model.select_to(next_char(&model.text, model.cursor()));
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &SelectWordLeft, window, cx| {
                    let model = &mut state.model;
                    model.select_to(previous_word(&model.text, model.cursor()));
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(
                    &state,
                    |state, _: &SelectWordRight, window, cx| {
                        let model = &mut state.model;
                        model.select_to(next_word(&model.text, model.cursor()));
                        state.selection_changed(window, cx);
                    },
                ))
                .on_action(listener(&state, |state, _: &SelectToStart, window, cx| {
                    state.model.select_to(0);
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &SelectToEnd, window, cx| {
                    state.model.select_to(state.model.text.len());
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &SelectAll, window, cx| {
                    state.model.select(0, state.model.text.len());
                    state.selection_changed(window, cx);
                }))
                .on_action(listener(&state, |state, _: &Copy, _, cx| {
                    if !state.model.selection.is_empty() {
                        let text = state.model.text[state.model.selection.clone()].to_owned();
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                    }
                }))
                .on_action(listener(&state, |state, _: &Cut, window, cx| {
                    if state.on_change.is_some() && !state.model.selection.is_empty() {
                        let range = state.model.selection.clone();
                        let text = state.model.text[range.clone()].to_owned();
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                        state.edit(range, "", window, cx);
                    }
                }))
                .on_action(listener(&state, |state, _: &Paste, window, cx| {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        let range = state.model.replacement_range(None);
                        state.edit(range, &text.replace(['\r', '\n'], " "), window, cx);
                    }
                }))
                .on_a11y_action(accesskit::Action::SetTextSelection, {
                    let state = state.clone();
                    let runs = runs.clone();
                    let run_ids = run_ids.clone();
                    let text = model.text.clone();
                    move |data, window, cx| {
                        let Some(accesskit::ActionData::SetTextSelection(selection)) = data else {
                            return;
                        };
                        let ids = run_ids.borrow();
                        let offset = |position: accesskit::TextPosition| {
                            let run = ids.iter().position(|id| *id == position.node)?;
                            Some(a11y_offset(&runs[run], &text, position.character_index))
                        };
                        let (Some(anchor), Some(focus)) =
                            (offset(selection.anchor), offset(selection.focus))
                        else {
                            return;
                        };
                        state.update(cx, |state, cx| {
                            state.model.select(anchor, focus);
                            state.selection_changed(window, cx);
                        });
                    }
                });
        }

        inner
            .child(TextLine {
                state,
                selection_color: self.selection_color,
                caret_color: self.caret_color,
                show_caret: !disabled && focus_handle.is_focused(window),
            })
            .a11y_synthetic_children({
                let text = model.text.clone();
                move |builder| {
                    let mut ids = run_ids.borrow_mut();
                    for (ix, run) in runs.iter().enumerate() {
                        let id = builder.synthetic_node_id(ix);
                        let mut node = accesskit::Node::new(accesskit::Role::TextRun);
                        node.set_value(&text[run.bytes.clone()]);
                        node.set_character_lengths(run.lengths.clone());
                        node.set_word_starts(run.word_starts.clone());
                        builder.push_child(id, node);
                        ids.push(id);
                    }
                    let position = |offset: usize| {
                        let (run, character_index) = a11y_position(&runs, &text, offset);
                        accesskit::TextPosition {
                            node: ids[run],
                            character_index,
                        }
                    };
                    let selection = accesskit::TextSelection {
                        anchor: position(model.anchor()),
                        focus: position(model.cursor()),
                    };
                    builder.parent_node().set_text_selection(selection);
                }
            })
            .a11y_info({
                let placeholder = self.placeholder.clone();
                let error = self.error.clone();
                move |node: &mut accesskit::Node| {
                    if disabled {
                        node.set_disabled();
                    } else if !editable {
                        node.set_read_only();
                    }
                    if let Some(placeholder) = &placeholder {
                        node.set_placeholder(placeholder.to_string());
                    }
                    if let Some(error) = &error {
                        node.set_invalid(accesskit::Invalid::True);
                        // AccessKit のアダプタは invalid や error_message を OS に渡さないため、
                        // メッセージを description に含めて読み上げさせる
                        let description = match node.description() {
                            Some(description) => format!("{error} {description}"),
                            None => error.to_string(),
                        };
                        node.set_description(description);
                    }
                }
            })
    }
}

/// `f` で `state` を更新する action のハンドラ
fn listener<A: gpui::Action>(
    state: &Entity<EditState>,
    f: impl Fn(&mut EditState, &A, &mut Window, &mut Context<EditState>) + 'static,
) -> impl Fn(&A, &mut Window, &mut App) + 'static {
    let state = state.clone();
    move |action, window, cx| state.update(cx, |state, cx| f(state, action, window, cx))
}

impl Styled for TextField {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for TextField {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl StatefulInteractiveElement for TextField {}

/// 編集中の文字列と選択。位置は UTF-8 のバイト数で、常に文字の境界にある
#[derive(Debug, Default, Clone, PartialEq)]
struct EditModel {
    text: String,
    /// 選択している範囲。空ならカーソルの位置
    selection: Range<usize>,
    /// カーソルが選択の始まりの側にあるか
    reversed: bool,
    /// IME で変換中の範囲
    marked: Option<Range<usize>>,
}

impl EditModel {
    fn cursor(&self) -> usize {
        if self.reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    /// 選択を広げても動かない側の端
    fn anchor(&self) -> usize {
        if self.reversed {
            self.selection.end
        } else {
            self.selection.start
        }
    }

    fn move_to(&mut self, offset: usize) {
        self.select(offset, offset);
    }

    /// 選択の動かない側の端を保ち、カーソルを `offset` に移す
    fn select_to(&mut self, offset: usize) {
        self.select(self.anchor(), offset);
    }

    /// `anchor` から `cursor` までを選択する。文字列の外や文字の途中の位置は、その手前の文字の境界にする
    fn select(&mut self, anchor: usize, cursor: usize) {
        let anchor = self.text.floor_char_boundary(anchor);
        let cursor = self.text.floor_char_boundary(cursor);
        self.reversed = cursor < anchor;
        self.selection = anchor.min(cursor)..anchor.max(cursor);
    }

    /// 文字列を `text` にそろえる。変わった場合は変換中の範囲を捨て、`true` を返す
    fn sync(&mut self, text: &str) -> bool {
        if self.text == text {
            return false;
        }
        self.text = text.to_owned();
        self.marked = None;
        self.select(self.anchor(), self.cursor());
        true
    }

    /// 入力で置き換える範囲。`range_utf16` が無ければ変換中の範囲、それも無ければ選択している範囲
    fn replacement_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .map(|range| self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone())
    }

    /// `range` を `new_text` に置き換え、その後ろにカーソルを移す
    fn replace(&mut self, range: Range<usize>, new_text: &str) {
        self.text.replace_range(range.clone(), new_text);
        self.marked = None;
        self.move_to(range.start + new_text.len());
    }

    /// `range` を変換中の `new_text` に置き換え、`new_text` の中の `selected_utf16` を選択する。
    /// `selected_utf16` が無ければ `new_text` の後ろにカーソルを移す
    fn replace_and_mark(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        selected_utf16: Option<Range<usize>>,
    ) {
        self.text.replace_range(range.clone(), new_text);
        self.marked = (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        let selected = selected_utf16
            .map(|selected| {
                utf16_to_offset(new_text, selected.start)..utf16_to_offset(new_text, selected.end)
            })
            .unwrap_or(new_text.len()..new_text.len());
        self.select(range.start + selected.start, range.start + selected.end);
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.text[..offset].encode_utf16().count()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        utf16_to_offset(&self.text, offset)
    }
}

/// `text` の UTF-16 の位置 `offset` に対応する UTF-8 の位置。サロゲートペアの途中なら次の文字の境界にする
fn utf16_to_offset(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (ix, ch) in text.char_indices() {
        if utf16 >= offset {
            return ix;
        }
        utf16 += ch.len_utf16();
    }
    text.len()
}

fn previous_char(text: &str, offset: usize) -> usize {
    text[..offset]
        .char_indices()
        .next_back()
        .map_or(0, |(ix, _)| ix)
}

fn next_char(text: &str, offset: usize) -> usize {
    text[offset..]
        .chars()
        .next()
        .map_or(offset, |ch| offset + ch.len_utf8())
}

/// 語の区切りを決める文字の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Whitespace,
    Word,
    Punctuation,
}

impl From<char> for CharClass {
    fn from(ch: char) -> Self {
        if ch.is_whitespace() {
            CharClass::Whitespace
        } else if ch.is_alphanumeric() || ch == '_' {
            CharClass::Word
        } else {
            CharClass::Punctuation
        }
    }
}

/// `offset` の前の空白を飛ばした先にある語の始まり
fn previous_word(text: &str, offset: usize) -> usize {
    let mut chars = text[..offset]
        .char_indices()
        .rev()
        .skip_while(|(_, ch)| ch.is_whitespace())
        .peekable();
    let Some(&(mut start, first)) = chars.peek() else {
        return 0;
    };
    let class = CharClass::from(first);
    for (ix, ch) in chars {
        if CharClass::from(ch) != class {
            break;
        }
        start = ix;
    }
    start
}

/// `offset` の後の空白を飛ばした先にある語の終わり
fn next_word(text: &str, offset: usize) -> usize {
    let mut chars = text[offset..]
        .char_indices()
        .skip_while(|(_, ch)| ch.is_whitespace())
        .peekable();
    let Some(&(_, first)) = chars.peek() else {
        return text.len();
    };
    let class = CharClass::from(first);
    chars
        .find(|(_, ch)| CharClass::from(*ch) != class)
        .map_or(text.len(), |(ix, _)| offset + ix)
}

/// `offset` の文字 (末尾なら直前の文字) と同じ種類の文字が続く範囲
fn word_range(text: &str, offset: usize) -> Range<usize> {
    let Some(ch) = text[offset..]
        .chars()
        .next()
        .or_else(|| text[..offset].chars().next_back())
    else {
        return offset..offset;
    };
    let class = CharClass::from(ch);
    let start = text[..offset]
        .char_indices()
        .rev()
        .take_while(|(_, ch)| CharClass::from(*ch) == class)
        .last()
        .map_or(offset, |(ix, _)| ix);
    let end = text[offset..]
        .char_indices()
        .find(|(_, ch)| CharClass::from(*ch) != class)
        .map_or(text.len(), |(ix, _)| offset + ix);
    start..end
}

/// 支援技術に公開する 1 つの TextRun
#[derive(Debug, PartialEq)]
struct A11yRun {
    /// 文字列の中の範囲
    bytes: Range<usize>,
    /// 文字ごとの UTF-8 のバイト数
    lengths: Vec<u8>,
    /// 語が始まる文字の位置
    word_starts: Vec<u8>,
}

/// `text` を、[`A11Y_RUN_LEN`] 文字ごとの TextRun に分ける。空の文字列も 1 つの空の TextRun にする
fn a11y_runs(text: &str) -> Vec<A11yRun> {
    let mut runs = vec![A11yRun {
        bytes: 0..0,
        lengths: Vec::new(),
        word_starts: Vec::new(),
    }];
    let mut previous = CharClass::Whitespace;
    for (ix, ch) in text.char_indices() {
        let mut run = runs.last_mut().expect("1 つ以上の TextRun を持つ");
        if run.lengths.len() == A11Y_RUN_LEN {
            runs.push(A11yRun {
                bytes: ix..ix,
                lengths: Vec::new(),
                word_starts: Vec::new(),
            });
            run = runs.last_mut().expect("直前に足した");
        }
        let class = CharClass::from(ch);
        if class != CharClass::Whitespace && class != previous {
            run.word_starts.push(run.lengths.len() as u8);
        }
        previous = class;
        run.lengths.push(ch.len_utf8() as u8);
        run.bytes.end = ix + ch.len_utf8();
    }
    runs
}

/// 文字列の位置 `offset` を、TextRun の番号とその中の文字の位置にする。TextRun の境目は後ろの TextRun の先頭にする
fn a11y_position(runs: &[A11yRun], text: &str, offset: usize) -> (usize, usize) {
    let run = runs
        .iter()
        .position(|run| offset < run.bytes.end)
        .unwrap_or(runs.len() - 1);
    let start = runs[run].bytes.start;
    (run, text[start..offset.max(start)].chars().count())
}

/// `run` の中の文字の位置 `character_index` に対応する文字列の位置
fn a11y_offset(run: &A11yRun, text: &str, character_index: usize) -> usize {
    let offset = run.bytes.start
        + run.lengths[..character_index.min(run.lengths.len())]
            .iter()
            .map(|len| usize::from(*len))
            .sum::<usize>();
    offset.min(text.len())
}

/// IME で変換を始める前の状態
struct Composition {
    /// 変換を始める前の文字列と選択
    start: EditModel,
    /// 変換を始めた編集が `start` の文字列で置き換えた範囲
    range: Range<usize>,
    /// 変換を始めた後の編集が、どれも変換中の範囲だけを置き換えたか
    in_place: bool,
}

/// `edited` の `cursor` の位置を、`edited` を書き換えた `value` の位置に移す
///
/// 先頭と末尾で一致する部分の中なら、その部分の中の位置を保つ。それ以外は `value` の変わった部分の末尾にする
fn remap_cursor(edited: &str, cursor: usize, value: &str) -> usize {
    let prefix: usize = edited
        .chars()
        .zip(value.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum();
    let suffix: usize = edited[prefix..]
        .chars()
        .rev()
        .zip(value[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum();
    if cursor <= prefix {
        cursor
    } else if cursor >= edited.len() - suffix {
        value.len() - (edited.len() - cursor)
    } else {
        value.len() - suffix
    }
}

/// 描画をまたいで保つ、編集の状態
struct EditState {
    model: EditModel,
    focus_handle: FocusHandle,
    /// 編集できない場合は `None`
    on_change: Option<ChangeHandler>,
    /// 最後の描画の後で最初に編集する前の文字列と選択
    before_edit: Option<EditModel>,
    /// IME の変換中に保つ状態。変換中でなければ `None`
    composition: Option<Composition>,
    /// 最後に描いた文字列の形と、その左上の位置と行の高さ
    layout: Option<(ShapedLine, Point<Pixels>, Pixels)>,
    /// 文字列を左にずらして描く量
    scroll: Pixels,
    /// 左ボタンを部品の上で押してから離すまでの間か
    selecting: bool,
}

impl EditState {
    /// 選択していればその範囲、していなければカーソルから `boundary` が返す位置までの範囲
    fn deletion(&self, boundary: impl Fn(&str, usize) -> usize) -> Range<usize> {
        if !self.model.selection.is_empty() {
            return self.model.selection.clone();
        }
        let cursor = self.model.cursor();
        let other = boundary(&self.model.text, cursor);
        cursor.min(other)..cursor.max(other)
    }

    /// 編集できれば `range` を `text` に置き換え、編集した後の文字列を求める
    fn edit(
        &mut self,
        range: Range<usize>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.on_change.is_none() || (range.is_empty() && text.is_empty()) {
            return;
        }
        let before = self.model.clone();
        self.model.replace(range.clone(), text);
        self.text_changed(before, range, text, false, window, cx);
    }

    /// 編集する前の `before` の `range` を `inserted` に置き換えた編集を求める。`ime` は IME からの入力か
    fn text_changed(
        &mut self,
        before: EditModel,
        range: Range<usize>,
        inserted: &str,
        ime: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
        let new_text: SharedString = self.model.text.clone().into();
        let composing = self.model.marked.is_some();
        match &mut self.composition {
            Some(composition) => {
                composition.in_place &= before.marked.as_ref() == Some(&range);
            }
            None if composing => {
                self.composition = Some(Composition {
                    start: before.clone(),
                    range: range.clone(),
                    in_place: true,
                });
            }
            None => {}
        }
        let edit = match (!composing).then(|| self.composition.take()).flatten() {
            // 変換の確定は、変換を始める前の文字列に対する 1 回の編集として渡す
            Some(Composition {
                start,
                range: start_range,
                in_place,
            }) => {
                // 確定を受け入れなかった場合は、変換を始める前の選択へ戻す
                self.before_edit = Some(start.clone());
                if ime && inserted.is_empty() {
                    // IME からの空の確定は変換の取り消しとして、変換を始める前の文字列と選択に戻す
                    self.model = start.clone();
                    TextEdit {
                        old_text: start.text.clone().into(),
                        range: start_range.start..start_range.start,
                        inserted: SharedString::default(),
                        new_text: start.text.into(),
                        composing: false,
                    }
                } else if in_place
                    && let Some(len) =
                        (new_text.len() + start_range.len()).checked_sub(start.text.len())
                {
                    TextEdit {
                        inserted: new_text[start_range.start..start_range.start + len]
                            .to_owned()
                            .into(),
                        old_text: start.text.into(),
                        range: start_range,
                        new_text,
                        composing: false,
                    }
                } else {
                    // 変換中の範囲の外も置き換えた場合は、文字列全体を置き換えた編集とする
                    TextEdit {
                        range: 0..start.text.len(),
                        old_text: start.text.into(),
                        inserted: new_text.clone(),
                        new_text,
                        composing: false,
                    }
                }
            }
            None => TextEdit {
                old_text: before.text.clone().into(),
                range,
                inserted: inserted.to_owned().into(),
                new_text,
                composing,
            },
        };
        self.before_edit.get_or_insert(before);
        if let Some(on_change) = self.on_change.clone() {
            on_change(&edit, window, cx);
        }
    }

    fn selection_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    /// 最後に描いた文字列の形と、その左上の位置と行の高さ。描いた後に文字列が変わっていれば `None`
    fn current_layout(&self) -> Option<&(ShapedLine, Point<Pixels>, Pixels)> {
        let layout = self.layout.as_ref()?;
        let drawn = layout.0.text.as_bytes();
        let text = self.model.text.as_bytes();
        // 描いた文字列は改行を空白に変えている
        let same = drawn.len() == text.len()
            && drawn.iter().zip(text).all(|(drawn, text)| {
                drawn == text || (*drawn == b' ' && matches!(text, b'\r' | b'\n'))
            });
        same.then_some(layout)
    }

    /// 最後に描いた文字列で、ウィンドウの x 座標 `x` にある文字の始まり。文字列より右なら末尾
    fn char_at_x(&self, x: Pixels) -> usize {
        match self.current_layout() {
            Some((line, origin, _)) => line
                .index_for_x(x - origin.x)
                .unwrap_or(self.model.text.len()),
            None => self.model.cursor(),
        }
    }

    /// 最後に描いた文字列で、ウィンドウの x 座標 `x` に最も近い文字の境界
    fn offset_for_x(&self, x: Pixels) -> usize {
        match self.current_layout() {
            Some((line, origin, _)) => line.closest_index_for_x(x - origin.x),
            None => self.model.cursor(),
        }
    }
}

impl EntityInputHandler for EditState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.model.replacement_range(Some(range_utf16));
        adjusted_range.replace(
            self.model.offset_to_utf16(range.start)..self.model.offset_to_utf16(range.end),
        );
        Some(self.model.text[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let selection = &self.model.selection;
        Some(UTF16Selection {
            range: self.model.offset_to_utf16(selection.start)
                ..self.model.offset_to_utf16(selection.end),
            reversed: self.model.reversed,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.model.replacement_range(Some(range_utf16));
        self.model.select(range.start, range.end);
        self.selection_changed(window, cx);
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.model.offset_to_utf16(self.model.text.len()))
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.model.marked.as_ref().map(|range| {
            self.model.offset_to_utf16(range.start)..self.model.offset_to_utf16(range.end)
        })
    }

    fn paste(&mut self, item: ClipboardItem, window: &mut Window, cx: &mut Context<Self>) {
        // Web の貼り付けもキーバインドの Paste と同じく、改行を空白にする
        if let Some(text) = item.text() {
            let range = self.model.replacement_range(None);
            self.edit(range, &text.replace(['\r', '\n'], " "), window, cx);
        }
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.model.clone();
        let marked = self.model.marked.take();
        if self.composition.is_some()
            && let Some(marked) = marked
        {
            let committed = self.model.text[marked.clone()].to_owned();
            self.text_changed(before, marked, &committed, true, window, cx);
        } else {
            window.notify_text_input_changed(&self.focus_handle);
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let explicit_range = range_utf16.is_some();
        let range = self.model.replacement_range(range_utf16);
        // 1 行の部品のため改行は入れない。Enter の入力もここに届くため、改行だけの入力は空にする。
        // ほかの改行 (Web の貼り付けなど) は Paste と同じく空白にする
        let text = if matches!(text, "\n" | "\r" | "\r\n") {
            String::new()
        } else {
            text.replace(['\r', '\n'], " ")
        };
        if text.is_empty() {
            match &self.model.marked {
                // Enter と、範囲を指定しない空の確定では、選択している範囲を消さない
                None if !explicit_range => return,
                Some(_) if range.is_empty() => {
                    self.unmark_text(window, cx);
                    return;
                }
                _ => {}
            }
        }
        if self.on_change.is_none() || (range.is_empty() && text.is_empty()) {
            return;
        }
        let before = self.model.clone();
        self.model.replace(range.clone(), &text);
        self.text_changed(before, range, &text, true, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_text = new_text.replace(['\r', '\n'], "");
        // 確定の後に届く空の変換中の文字列では、文字列を変えない
        if self.on_change.is_none() || (new_text.is_empty() && self.model.marked.is_none()) {
            return;
        }
        let range = self.model.replacement_range(range_utf16);
        let before = self.model.clone();
        self.model
            .replace_and_mark(range.clone(), &new_text, new_selected_range_utf16);
        self.text_changed(before, range, &new_text, true, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let (line, origin, line_height) = self.current_layout()?;
        let range = self.model.replacement_range(Some(range_utf16));
        Some(Bounds::from_corners(
            point(origin.x + line.x_for_index(range.start), origin.y),
            point(
                origin.x + line.x_for_index(range.end),
                origin.y + *line_height,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let (line, origin, _) = self.current_layout()?;
        Some(
            self.model
                .offset_to_utf16(line.closest_index_for_x(point.x - origin.x)),
        )
    }
}

/// 文字列・選択・カーソルを描き、IME の入力と選択のドラッグを受ける要素
struct TextLine {
    state: Entity<EditState>,
    selection_color: Option<Hsla>,
    caret_color: Option<Hsla>,
    show_caret: bool,
}

struct TextLinePrepaint {
    line: ShapedLine,
    origin: Point<Pixels>,
    selection: Option<PaintQuad>,
    caret: Option<PaintQuad>,
}

impl IntoElement for TextLine {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextLine {
    type RequestLayoutState = ();
    type PrepaintState = TextLinePrepaint;

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
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let style = window.text_style();
        let state = self.state.read(cx);
        let model = &state.model;
        // gpui は改行を含む 1 行を描けないため、改行を同じバイト数の空白に変えて描く
        let text = SharedString::from(model.text.replace(['\r', '\n'], " "));
        let run = style.to_run(text.len());
        let runs = match &model.marked {
            Some(marked) => {
                let underline = UnderlineStyle {
                    color: Some(run.color),
                    thickness: px(1.),
                    wavy: false,
                };
                [
                    (marked.start, None),
                    (marked.len(), Some(underline)),
                    (text.len() - marked.end, None),
                ]
                .into_iter()
                .filter(|(len, _)| *len > 0)
                .map(|(len, underline)| gpui::TextRun {
                    len,
                    underline,
                    ..run.clone()
                })
                .collect()
            }
            None => vec![run],
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(text, font_size, &runs, None);

        // カーソルが見えるように、ずらす量をカーソルの位置に合わせる。
        // 右端にはカーソルの幅を空け、文字列の末尾より右へはずらさない
        let cursor_x = line.x_for_index(model.cursor());
        let width = bounds.size.width;
        let max_scroll = (line.width() + CARET_WIDTH - width).max(px(0.));
        let scroll = state
            .scroll
            .min(cursor_x)
            .max(cursor_x + CARET_WIDTH - width)
            .min(max_scroll)
            .max(px(0.));
        // 文字列が収まる間は、継承した text style の揃えに従って置く。収まらない間はカーソルに合わせてずらす
        let free = (width - line.width() - CARET_WIDTH).max(px(0.));
        let align_offset = match style.text_align {
            TextAlign::Left => px(0.),
            TextAlign::Center => free / 2.,
            TextAlign::Right => free,
        };
        let origin = point(bounds.left() - scroll + align_offset, bounds.top());

        let selection = (!model.selection.is_empty()).then(|| {
            let color = self
                .selection_color
                .unwrap_or_else(|| style.color.opacity(0.25));
            fill(
                Bounds::from_corners(
                    point(
                        origin.x + line.x_for_index(model.selection.start),
                        bounds.top(),
                    ),
                    point(
                        origin.x + line.x_for_index(model.selection.end),
                        bounds.bottom(),
                    ),
                ),
                color,
            )
        });
        let caret = (self.show_caret && model.selection.is_empty()).then(|| {
            fill(
                Bounds::new(
                    point(origin.x + cursor_x, bounds.top()),
                    size(CARET_WIDTH, bounds.size.height),
                ),
                self.caret_color.unwrap_or(style.color),
            )
        });

        let height = bounds.size.height;
        let shaped = line.clone();
        self.state.update(cx, |state, _| {
            state.scroll = scroll;
            state.layout = Some((shaped, origin, height));
        });
        TextLinePrepaint {
            line,
            origin,
            selection,
            caret,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.state.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.state.clone()),
            cx,
        );

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = prepaint.selection.take() {
                window.paint_quad(selection);
            }
            // 失敗するのはグリフを描けない場合で、そのときは文字列を描かない
            let _ = prepaint.line.paint(
                prepaint.origin,
                bounds.size.height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
            if let Some(caret) = prepaint.caret.take() {
                window.paint_quad(caret);
            }
        });

        // 押したまま部品の外へ動かしても選択を広げるため、ウィンドウ全体で受ける
        let state = self.state.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble || !state.read(cx).selecting {
                return;
            }
            state.update(cx, |state, cx| {
                // ボタンを離す操作が届かなかった場合は、ボタンを押していない移動で選択を終える
                if event.pressed_button != Some(MouseButton::Left) {
                    state.selecting = false;
                    return;
                }
                let selection = state.model.selection.clone();
                let offset = state.offset_for_x(event.position.x);
                state.model.select_to(offset);
                if state.model.selection != selection {
                    state.selection_changed(window, cx);
                }
            });
        });
        let state = self.state.clone();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble
                && event.button == MouseButton::Left
                && state.read(cx).selecting
            {
                state.update(cx, |state, _| state.selecting = false);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        ClipboardItem, Context, Element, Entity, FocusHandle, IntoElement, Modifiers,
        MouseDownEvent, MouseUpEvent, ParentElement, Render, RenderOnce, SharedString, Styled,
        TestAppContext, VisualTestContext, Window, accesskit, div, point, prelude::FluentBuilder,
        px,
    };

    use super::*;
    use crate::{Availability, button::Button, test_window::add_test_window};

    #[test]
    fn utf_16_の位置と相互に変換する() {
        let model = EditModel {
            text: "a😀b".into(),
            ..EditModel::default()
        };
        assert_eq!(model.offset_to_utf16(5), 3);
        assert_eq!(model.offset_from_utf16(3), 5);
        assert_eq!(
            model.offset_from_utf16(2),
            5,
            "サロゲートペアの途中は次の文字の境界にすること"
        );
        assert_eq!(model.offset_from_utf16(10), 6);
    }

    #[test]
    fn 変換中の文字列を置き換えて確定する() {
        let mut model = EditModel {
            text: "ab".into(),
            selection: 1..1,
            ..EditModel::default()
        };
        let range = model.replacement_range(None);
        model.replace_and_mark(range, "か", None);
        assert_eq!(
            (model.text.as_str(), model.marked.clone()),
            ("aかb", Some(1..4))
        );
        let range = model.replacement_range(None);
        model.replace_and_mark(range, "かん", Some(0..1));
        assert_eq!(
            (model.text.as_str(), model.marked.clone()),
            ("aかんb", Some(1..7))
        );
        assert_eq!(model.selection, 1..4);
        let range = model.replacement_range(None);
        model.replace(range, "漢");
        assert_eq!((model.text.as_str(), model.marked.clone()), ("a漢b", None));
        assert_eq!(model.selection, 4..4);
    }

    #[test]
    fn 外から渡された文字列にそろえると変換中の範囲を捨て選択を収める() {
        let mut model = EditModel {
            text: "abcdef".into(),
            selection: 2..5,
            reversed: true,
            marked: Some(4..6),
        };
        assert!(!model.sync("abcdef"));
        assert!(model.sync("aあ"));
        assert_eq!(model.marked, None);
        assert_eq!(
            (model.selection.clone(), model.reversed),
            (1..4, true),
            "文字の途中は手前の境界に、末尾より後ろは末尾にすること"
        );
    }

    #[test]
    fn 語の境界は空白を飛ばし同じ種類の文字が続く範囲で決める() {
        let text = "foo  bar.baz";
        assert_eq!(previous_word(text, 12), 9);
        assert_eq!(previous_word(text, 9), 8);
        assert_eq!(previous_word(text, 5), 0);
        assert_eq!(next_word(text, 0), 3);
        assert_eq!(next_word(text, 3), 8);
        assert_eq!(next_word(text, 12), 12);
        assert_eq!(word_range(text, 6), 5..8);
        assert_eq!(word_range(text, 12), 9..12);
        assert_eq!(word_range(text, 3), 3..5);
    }

    #[test]
    fn 支援技術に公開する文字列を_255_文字ごとに分ける() {
        assert_eq!(
            a11y_runs(""),
            [A11yRun {
                bytes: 0..0,
                lengths: Vec::new(),
                word_starts: Vec::new()
            }]
        );
        let runs = a11y_runs("ab cあ");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].lengths, [1, 1, 1, 1, 3]);
        assert_eq!(runs[0].word_starts, [0, 3]);

        let text = "a".repeat(300);
        let runs = a11y_runs(&text);
        assert_eq!(
            runs.iter().map(|run| run.bytes.clone()).collect::<Vec<_>>(),
            [0..255, 255..300]
        );
        assert_eq!(a11y_position(&runs, &text, 255), (1, 0));
        assert_eq!(a11y_position(&runs, &text, 300), (1, 45));
        assert_eq!(a11y_offset(&runs[1], &text, 45), 300);
        assert_eq!(a11y_offset(&runs[0], &text, 999), 255);
    }

    const CARET_COLOR: fn() -> Hsla = gpui::red;
    const SELECTION_COLOR: fn() -> Hsla = gpui::blue;

    /// 500x500 の親の (100, 100) に置いた幅 200・高さ 20 のテキストフィールド。親の左上に 10x10 のボタンを置く
    struct FieldView {
        value: SharedString,
        /// `on_change` に渡された値
        changes: Rc<RefCell<Vec<SharedString>>>,
        /// `on_submit` に渡された値
        submits: Rc<RefCell<Vec<SharedString>>>,
        /// `on_change` を設定するか
        editable: bool,
        /// `on_change` の代わりに `on_edit` を設定し、渡された編集を持つ
        edits: Option<Rc<RefCell<Vec<TextEdit>>>>,
        /// `on_change` に渡された値を受け入れるか
        accept: bool,
        /// 受け入れる値を書き換える
        rewrite: Option<fn(&str) -> String>,
        read_only: bool,
        availability: Availability,
        focus: FocusHandle,
        before_focus: FocusHandle,
        node: Option<accesskit::Node>,
        /// 親が受けたマウスの左ボタンの押下の回数
        parent_presses: Rc<RefCell<usize>>,
        /// 部品の編集の状態。IME の変換を直接起こすために持つ
        edit_state: Option<Entity<EditState>>,
        /// 文字列の横の揃え
        align: TextAlign,
        error: Option<SharedString>,
    }

    impl Render for FieldView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let changes = self.changes.clone();
            let submits = self.submits.clone();
            let field = TextField::new("field", self.value.clone())
                .on_submit(move |value, _, _| submits.borrow_mut().push(value.clone()))
                .when(self.editable, |this| {
                    this.on_change(cx.listener(move |this, value: &SharedString, _, cx| {
                        changes.borrow_mut().push(value.clone());
                        if this.accept {
                            this.value = match this.rewrite {
                                Some(rewrite) => rewrite(value).into(),
                                None => value.clone(),
                            };
                        }
                        cx.notify();
                    }))
                })
                .when_some(self.edits.clone(), |this, edits| {
                    this.on_edit(cx.listener(move |this, edit: &TextEdit, _, cx| {
                        edits.borrow_mut().push(edit.clone());
                        // 受け入れない編集は、変換中の文字列を表示していても編集する前に戻す
                        this.value = if this.accept {
                            edit.new_text.clone()
                        } else {
                            edit.old_text.clone()
                        };
                        cx.notify();
                    }))
                })
                .read_only(self.read_only)
                .availability(self.availability)
                .placeholder("Your name")
                .caret_color(CARET_COLOR())
                .selection_color(SELECTION_COLOR())
                .track_focus(&self.focus)
                .aria_label("Name")
                .error(self.error.clone())
                .aria_description("Full name")
                .map(|this| match self.align {
                    TextAlign::Left => this,
                    TextAlign::Center => this.text_center(),
                    // 右揃えは親から継承させる
                    TextAlign::Right => this,
                })
                .absolute()
                .left(px(100.))
                .top(px(100.))
                .w(px(200.))
                .h(px(20.))
                .line_height(px(20.))
                .render(window, cx)
                .into_element();
            // テスト用ウィンドウは支援技術を有効にできず a11y ツリーを組み立てないため、
            // 描画する要素から直接ノードを書き出す
            let mut node = accesskit::Node::new(field.a11y_role().expect("role を持つこと"));
            field.write_a11y_info(&mut node);
            self.node = Some(node);
            // 部品は描画の中で同じ id の keyed state を作るため、ここで同じ状態を得られる
            self.edit_state = Some(
                window.use_keyed_state(ElementId::from("field"), cx, |_, _| {
                    unreachable!("部品が先に状態を作ること")
                }),
            );
            let parent_presses = self.parent_presses.clone();
            div()
                .size(px(500.))
                .when(self.align == TextAlign::Right, |this| this.text_right())
                .on_mouse_down(MouseButton::Left, move |_, _, _| {
                    *parent_presses.borrow_mut() += 1;
                })
                .child(
                    Button::new("before")
                        .track_focus(&self.before_focus)
                        .size(px(10.)),
                )
                .child(field)
        }
    }

    fn setup<'a>(
        cx: &'a mut TestAppContext,
        value: &'static str,
    ) -> (Entity<FieldView>, &'a mut VisualTestContext) {
        add_test_window(cx, |_, cx| FieldView {
            value: value.into(),
            changes: Rc::default(),
            submits: Rc::default(),
            editable: true,
            edits: None,
            accept: true,
            rewrite: None,
            read_only: false,
            availability: Availability::Enabled,
            focus: cx.focus_handle(),
            before_focus: cx.focus_handle(),
            node: None,
            parent_presses: Rc::default(),
            edit_state: None,
            align: TextAlign::Left,
            error: None,
        })
    }

    fn update(
        view: &Entity<FieldView>,
        cx: &mut VisualTestContext,
        f: impl FnOnce(&mut FieldView),
    ) {
        view.update(cx, |view, cx| {
            f(view);
            cx.notify();
        });
        cx.run_until_parked();
    }

    fn value(view: &Entity<FieldView>, cx: &mut VisualTestContext) -> SharedString {
        view.read_with(cx, |view, _| view.value.clone())
    }

    fn changes(view: &Entity<FieldView>, cx: &mut VisualTestContext) -> Vec<SharedString> {
        view.read_with(cx, |view, _| view.changes.borrow().clone())
    }

    fn focused(view: &Entity<FieldView>, cx: &mut VisualTestContext) -> bool {
        let focus = view.read_with(cx, |view, _| view.focus.clone());
        cx.update(|window, _| focus.is_focused(window))
    }

    fn node(view: &Entity<FieldView>, cx: &mut VisualTestContext) -> accesskit::Node {
        view.read_with(cx, |view, _| view.node.clone().unwrap())
    }

    /// 背景を `color` で塗った矩形の x, y, 幅, 高さ
    fn filled(cx: &mut VisualTestContext, color: Hsla) -> Vec<(f32, f32, f32, f32)> {
        cx.update(|window, _| {
            let scale = window.scale_factor();
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == color.into())
                .map(|quad| {
                    let b = quad.bounds;
                    (
                        b.origin.x.0 / scale,
                        b.origin.y.0 / scale,
                        b.size.width.0 / scale,
                        b.size.height.0 / scale,
                    )
                })
                .collect()
        })
    }

    /// Tab でテキストフィールドにフォーカスする
    fn focus(cx: &mut VisualTestContext) {
        cx.simulate_keystrokes("tab tab");
    }

    fn mouse_down(cx: &mut VisualTestContext, x: f32, click_count: usize, modifiers: Modifiers) {
        let position = point(px(x), px(110.));
        cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position,
            modifiers,
            click_count,
            first_mouse: false,
        });
    }

    fn mouse_up(cx: &mut VisualTestContext, x: f32) {
        cx.simulate_event(MouseUpEvent {
            button: MouseButton::Left,
            position: point(px(x), px(110.)),
            modifiers: Modifiers::none(),
            click_count: 1,
        });
    }

    #[gpui::test]
    fn 最初に描いたときはカーソルを末尾に置く(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        focus(cx);
        cx.simulate_input("c");
        assert_eq!(value(&view, cx), "abc");
    }

    #[gpui::test]
    fn 入力した文字をカーソルの位置に入れ_1_文字ごとに求める(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        focus(cx);
        assert!(focused(&view, cx));
        cx.simulate_input("ab");
        cx.simulate_keystrokes("left");
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "axb");
        assert_eq!(changes(&view, cx), ["a", "ab", "axb"]);
    }

    #[gpui::test]
    fn 矢印キーと_home_end_でカーソルを移し文字を消す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "abcd");
        focus(cx);
        cx.simulate_keystrokes("end left backspace");
        assert_eq!(value(&view, cx), "abd");
        cx.simulate_keystrokes("home delete");
        assert_eq!(value(&view, cx), "bd");
        cx.simulate_keystrokes("right right right delete");
        assert_eq!(
            changes(&view, cx),
            ["abd", "bd"],
            "末尾で Delete を押しても求めないこと"
        );
        cx.simulate_keystrokes("home backspace");
        assert_eq!(
            changes(&view, cx).len(),
            2,
            "先頭で Backspace を押しても求めないこと"
        );
    }

    #[gpui::test]
    fn shift_と矢印キーで選択し入力で置き換える(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "abcd");
        focus(cx);
        cx.simulate_keystrokes("end shift-left shift-left");
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "abx");
        cx.simulate_keystrokes("shift-home");
        cx.simulate_keystrokes("right");
        cx.simulate_input("y");
        assert_eq!(
            value(&view, cx),
            "abxy",
            "選択中の右矢印キーは選択の終わりへ移すこと"
        );
        cx.simulate_keystrokes("shift-home left");
        cx.simulate_input("z");
        assert_eq!(
            value(&view, cx),
            "zabxy",
            "選択中の左矢印キーは選択の始まりへ移すこと"
        );
    }

    #[gpui::test]
    fn ctrl_と合わせて語の単位で移し消す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "foo bar baz");
        focus(cx);
        cx.simulate_keystrokes("end ctrl-left ctrl-backspace");
        assert_eq!(value(&view, cx), "foo baz");
        cx.simulate_keystrokes("ctrl-delete");
        assert_eq!(value(&view, cx), "foo ");
        cx.simulate_keystrokes("home ctrl-shift-right");
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "x ");
    }

    #[gpui::test]
    fn すべてを選択して切り取り貼り付ける(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "abc");
        focus(cx);
        cx.simulate_keystrokes("ctrl-a ctrl-c");
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some("abc")
        );
        cx.simulate_keystrokes("ctrl-x");
        assert_eq!(value(&view, cx), "");
        cx.simulate_keystrokes("ctrl-v ctrl-v");
        assert_eq!(value(&view, cx), "abcabc");
        cx.write_to_clipboard(ClipboardItem::new_string("1\n2\r\n3".into()));
        cx.simulate_keystrokes("ctrl-v");
        assert_eq!(
            value(&view, cx),
            "abcabc1 2  3",
            "貼り付ける改行は空白にすること"
        );
    }

    #[gpui::test]
    fn enter_では改行を入れず_文字列を確定する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        focus(cx);
        cx.simulate_keystrokes("end");
        cx.simulate_input("c");
        cx.simulate_keystrokes("enter");
        assert_eq!(changes(&view, cx), ["abc"]);
        assert_eq!(value(&view, cx), "abc");
        assert_eq!(
            *view.read_with(cx, |view, _| view.submits.clone()).borrow(),
            ["abc"]
        );
    }

    #[gpui::test]
    fn 修飾キー付きの_enter_では確定せず_読み取り専用でも確定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        focus(cx);
        cx.simulate_keystrokes("shift-enter ctrl-enter");
        assert!(view.read_with(cx, |view, _| view.submits.borrow().is_empty()));
        update(&view, cx, |view| view.read_only = true);
        cx.simulate_keystrokes("enter");
        assert_eq!(
            *view.read_with(cx, |view, _| view.submits.clone()).borrow(),
            ["ab"]
        );
    }

    #[gpui::test]
    fn ime_で変換中の_enter_では確定せず_確定後の_enter_で確定する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        focus(cx);
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "か", None, window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert!(view.read_with(cx, |view, _| view.submits.borrow().is_empty()));
        state.update_in(cx, |state, window, cx| state.unmark_text(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert_eq!(
            *view.read_with(cx, |view, _| view.submits.clone()).borrow(),
            ["か"]
        );
    }

    #[gpui::test]
    fn 無効な間は_enter_で確定しない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        focus(cx);
        update(&view, cx, |view| view.availability = Availability::Disabled);
        cx.simulate_keystrokes("enter");
        assert!(view.read_with(cx, |view, _| view.submits.borrow().is_empty()));
    }

    #[gpui::test]
    fn 受け入れられなかった編集は次の描画で取り消す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| view.accept = false);
        focus(cx);
        cx.simulate_keystrokes("end");
        cx.simulate_input("x");
        assert_eq!(changes(&view, cx), ["abx"]);
        cx.simulate_input("y");
        assert_eq!(
            changes(&view, cx),
            ["abx", "aby"],
            "取り消した文字列から編集すること"
        );
    }

    #[test]
    fn 書き換えた文字列でも変わらなかった部分に対するカーソルの位置を保つ() {
        // 末尾に足した文字の後ろ
        assert_eq!(remap_cursor("112", 3, "11/2"), 4);
        // 変わらなかった先頭の中
        assert_eq!(remap_cursor("11/2/2026", 3, "11/22/026"), 3);
        // 変わらなかった末尾の中
        assert_eq!(remap_cursor("abXcd", 4, "abYYcd"), 5);
        // 変わった部分の中は、その末尾
        assert_eq!(remap_cursor("abXYZcd", 3, "abQcd"), 3);
        assert_eq!(remap_cursor("aあb", 4, "aいいb"), 7);
    }

    #[gpui::test]
    fn 呼び出し側が書き換えた編集はカーソルを書き換えた文字列の対応する位置に置く(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        // 2 文字の後ろに文字が続くときだけ区切りを入れる
        update(&view, cx, |view| {
            view.rewrite = Some(|value| {
                let digits: String = value.chars().filter(|c| *c != '/').collect();
                match digits.split_at_checked(2) {
                    Some((head, tail)) if !tail.is_empty() => format!("{head}/{tail}"),
                    _ => digits,
                }
            })
        });
        focus(cx);
        for c in ["1", "1", "2", "5"] {
            cx.simulate_input(c);
        }
        assert_eq!(value(&view, cx), "11/25", "区切りの後ろに続けて入れること");
    }

    #[gpui::test]
    fn 受け入れられなかった編集は選択も編集する前に戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| view.accept = false);
        focus(cx);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("x");
        cx.simulate_input("y");
        assert_eq!(
            changes(&view, cx),
            ["x", "y"],
            "すべてを選択した状態に戻すこと"
        );
    }

    #[gpui::test]
    fn on_edit_には置き換えた範囲と入れた文字列と変換中かを渡す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        let edits = Rc::new(RefCell::new(Vec::new()));
        update(&view, cx, |view| view.edits = Some(edits.clone()));
        focus(cx);
        cx.simulate_keystrokes("end");
        cx.simulate_input("x");
        cx.simulate_keystrokes("shift-home backspace");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "か", None, window, cx);
        });
        cx.run_until_parked();
        let edit =
            |old: &str, range: Range<usize>, inserted: &str, new: &str, composing| TextEdit {
                old_text: old.to_owned().into(),
                range,
                inserted: inserted.to_owned().into(),
                new_text: new.to_owned().into(),
                composing,
            };
        assert_eq!(
            *edits.borrow(),
            [
                edit("ab", 2..2, "x", "abx", false),
                edit("abx", 0..3, "", "", false),
                edit("", 0..0, "か", "か", true),
            ]
        );
        assert!(
            changes(&view, cx).is_empty(),
            "on_change は on_edit に置き換わる"
        );
    }

    #[gpui::test]
    fn 変換の確定は変換を始める前からの_1_回の編集として渡す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "09");
        let edits = Rc::new(RefCell::new(Vec::new()));
        update(&view, cx, |view| view.edits = Some(edits.clone()));
        focus(cx);
        cx.simulate_keystrokes("end");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "2", None, window, cx);
        });
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "23", None, window, cx);
        });
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| state.unmark_text(window, cx));
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "か", None, window, cx);
        });
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| {
            state.replace_text_in_range(None, "火", window, cx);
        });
        cx.run_until_parked();
        let edits: Vec<_> = edits
            .borrow()
            .iter()
            .map(|edit| {
                (
                    edit.old_text.to_string(),
                    edit.range.clone(),
                    edit.inserted.to_string(),
                    edit.composing,
                )
            })
            .collect();
        assert_eq!(
            edits,
            [
                ("09".into(), 2..2, "2".into(), true),
                ("092".into(), 2..3, "23".into(), true),
                ("09".into(), 2..2, "23".into(), false),
                ("0923".into(), 4..4, "か".into(), true),
                ("0923".into(), 4..4, "火".into(), false),
            ]
        );
    }

    #[gpui::test]
    fn 変換の取り消しでは変換を始める前の文字列と選択に戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "09");
        focus(cx);
        cx.simulate_keystrokes("ctrl-a");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "あ", None, window, cx);
        });
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "", None, window, cx);
            state.unmark_text(window, cx);
        });
        cx.run_until_parked();
        assert_eq!(value(&view, cx), "09");
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "x", "すべてを選択した状態に戻すこと");
    }

    #[gpui::test]
    fn 受け入れられなかった確定は変換を始める前の選択に戻す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "09");
        let edits = Rc::new(RefCell::new(Vec::new()));
        update(&view, cx, |view| view.edits = Some(edits.clone()));
        focus(cx);
        cx.simulate_keystrokes("ctrl-a");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        // 変換中の文字列は受け入れ、確定を断る
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "あ", None, window, cx);
        });
        cx.run_until_parked();
        update(&view, cx, |view| view.accept = false);
        state.update_in(cx, |state, window, cx| {
            state.replace_text_in_range(Some(0..1), "亜", window, cx);
        });
        cx.run_until_parked();
        assert_eq!(value(&view, cx), "09");
        let last = edits.borrow().last().cloned().expect("確定を渡すこと");
        assert_eq!(
            (last.old_text.as_ref(), last.range, last.composing),
            ("09", 0..2, false)
        );
        update(&view, cx, |view| view.accept = true);
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "x", "すべてを選択した状態に戻すこと");
    }

    #[gpui::test]
    fn 変換を始めたときと違う範囲の確定は文字列全体を置き換えた編集として渡す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "09");
        let edits = Rc::new(RefCell::new(Vec::new()));
        update(&view, cx, |view| view.edits = Some(edits.clone()));
        focus(cx);
        cx.simulate_keystrokes("end");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "あ", None, window, cx);
        });
        cx.run_until_parked();
        state.update_in(cx, |state, window, cx| {
            state.replace_text_in_range(Some(1..3), "23", window, cx);
        });
        cx.run_until_parked();
        let last = edits.borrow().last().cloned().expect("確定を渡すこと");
        assert_eq!(
            (
                last.old_text.as_ref(),
                last.range,
                last.inserted.as_ref(),
                last.new_text.as_ref()
            ),
            ("09", 0..2, "023", "023")
        );
    }

    #[gpui::test]
    fn 範囲の外も置き換えた後の確定と取り消しを変換の前から扱う(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "12");
        let edits = Rc::new(RefCell::new(Vec::new()));
        update(&view, cx, |view| view.edits = Some(edits.clone()));
        focus(cx);
        cx.simulate_keystrokes("home");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        let mark = |range: Option<Range<usize>>, text: &'static str, cx: &mut VisualTestContext| {
            state.update_in(cx, |state, window, cx| {
                state.replace_and_mark_text_in_range(range, text, None, window, cx);
            });
            cx.run_until_parked();
        };
        // 「あ1」を「い」にしてから取り消す
        mark(None, "あ", cx);
        mark(Some(0..2), "い", cx);
        mark(None, "", cx);
        assert_eq!(value(&view, cx), "12");
        let last = edits.borrow().last().cloned().expect("取り消しを渡すこと");
        assert_eq!(
            (last.old_text.as_ref(), last.new_text.as_ref()),
            ("12", "12")
        );
        // 「あ1」を「21」で確定すると、末尾が一致しても全体を置き換えた編集になる
        mark(None, "あ", cx);
        state.update_in(cx, |state, window, cx| {
            state.replace_text_in_range(Some(0..2), "21", window, cx);
        });
        cx.run_until_parked();
        let last = edits.borrow().last().cloned().expect("確定を渡すこと");
        assert_eq!(
            (last.old_text.as_ref(), last.range, last.new_text.as_ref()),
            ("12", 0..2, "212")
        );
    }

    #[gpui::test]
    fn 変換中の切り取りは取り消しにせず文字列を消す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "09");
        focus(cx);
        cx.simulate_keystrokes("end");
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_and_mark_text_in_range(None, "あ", None, window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("ctrl-a ctrl-x");
        assert_eq!(value(&view, cx), "");
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some("09あ")
        );
    }

    #[gpui::test]
    fn web_の貼り付けも改行を空白にする(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "");
        focus(cx);
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.paste(ClipboardItem::new_string("\n".into()), window, cx);
        });
        cx.run_until_parked();
        assert_eq!(changes(&view, cx), [" "]);
    }

    #[gpui::test]
    fn 改行だけの入力は入れず_改行を含む入力は改行を空白にする(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        focus(cx);
        let state = view
            .read_with(cx, |view, _| view.edit_state.clone())
            .expect("描画済みであること");
        state.update_in(cx, |state, window, cx| {
            state.replace_text_in_range(None, "\n", window, cx);
            state.replace_text_in_range(None, "a\nb", window, cx);
        });
        cx.run_until_parked();
        assert_eq!(changes(&view, cx), ["a b"]);
    }

    #[gpui::test]
    fn 外から文字列を変えても求めずカーソルを収める(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "abcd");
        focus(cx);
        cx.simulate_keystrokes("end");
        update(&view, cx, |view| view.value = "ab".into());
        assert_eq!(changes(&view, cx), Vec::<SharedString>::new());
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "abx");
    }

    #[gpui::test]
    fn on_change_が無ければ編集させず_read_only_として公開する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| view.editable = false);
        focus(cx);
        cx.simulate_input("x");
        cx.simulate_keystrokes("backspace");
        assert_eq!(value(&view, cx), "ab");
        assert!(node(&view, cx).is_read_only());
    }

    #[gpui::test]
    fn 読み取り専用でも選択してコピーできるが切り取れない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| view.read_only = true);
        focus(cx);
        cx.simulate_keystrokes("ctrl-a ctrl-x");
        assert_eq!(value(&view, cx), "ab");
        assert_eq!(changes(&view, cx), Vec::<SharedString>::new());
        assert_eq!(
            cx.read_from_clipboard(),
            None,
            "切り取れない場合は写さないこと"
        );
        cx.simulate_keystrokes("ctrl-c");
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some("ab")
        );
        assert!(node(&view, cx).is_read_only());
    }

    #[gpui::test]
    fn 無効なテキストフィールドは_tab_で到達できず編集できない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| view.availability = Availability::Disabled);
        focus(cx);
        assert!(!focused(&view, cx));
        mouse_down(cx, 150., 1, Modifiers::none());
        assert!(!focused(&view, cx));
        let node = node(&view, cx);
        assert!(node.is_disabled());
        assert!(!node.is_read_only());
    }

    #[gpui::test]
    fn 無効でも_tab_で到達できる設定ではフォーカスできるが編集できない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        update(&view, cx, |view| {
            view.availability = Availability::DisabledFocusable
        });
        focus(cx);
        assert!(focused(&view, cx));
        cx.simulate_input("x");
        cx.simulate_keystrokes("ctrl-a backspace");
        assert_eq!(value(&view, cx), "ab");
    }

    #[gpui::test]
    fn クリックした位置にカーソルを移しドラッグで選択する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "abcd");
        mouse_down(cx, 290., 1, Modifiers::none());
        mouse_up(cx, 290.);
        assert!(focused(&view, cx));
        cx.simulate_input("x");
        assert_eq!(
            value(&view, cx),
            "abcdx",
            "文字列より右を押すと末尾に移すこと"
        );

        mouse_down(cx, 100., 1, Modifiers::none());
        cx.simulate_mouse_move(
            point(px(400.), px(300.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        mouse_up(cx, 400.);
        cx.simulate_input("y");
        assert_eq!(
            value(&view, cx),
            "y",
            "部品の外までドラッグしても選択を広げること"
        );

        cx.simulate_keystrokes("home");
        mouse_down(cx, 290., 1, Modifiers::shift());
        mouse_up(cx, 290.);
        cx.simulate_input("z");
        assert_eq!(value(&view, cx), "z", "Shift と押すと選択を広げること");
    }

    #[gpui::test]
    fn 二回続けて押すと語を三回続けて押すとすべてを選択する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "foo bar");
        mouse_down(cx, 101., 2, Modifiers::none());
        mouse_up(cx, 101.);
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "x bar");
        mouse_down(cx, 101., 3, Modifiers::none());
        mouse_up(cx, 101.);
        cx.simulate_input("y");
        assert_eq!(value(&view, cx), "y");
    }

    #[gpui::test]
    fn マウスの左ボタンの押下を親に伝えない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        let presses = view.read_with(cx, |view, _| view.parent_presses.clone());
        mouse_down(cx, 150., 1, Modifiers::none());
        mouse_up(cx, 150.);
        assert_eq!(*presses.borrow(), 0);
        update(&view, cx, |view| view.availability = Availability::Disabled);
        mouse_down(cx, 150., 1, Modifiers::none());
        mouse_up(cx, 150.);
        assert_eq!(*presses.borrow(), 0, "無効でも親に伝えないこと");
        mouse_down(cx, 50., 1, Modifiers::none());
        assert_eq!(*presses.borrow(), 1);
    }

    #[gpui::test]
    fn tab_ではフォーカスを次の要素へ移す(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        focus(cx);
        cx.simulate_keystrokes("tab");
        assert!(!focused(&view, cx));
        assert_eq!(value(&view, cx), "ab");
    }

    #[gpui::test]
    fn textbox_として名前とプレースホルダーを公開する(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "ab");
        let node = node(&view, cx);
        assert_eq!(node.role(), accesskit::Role::TextInput);
        assert_eq!(node.label(), Some("Name"));
        assert_eq!(node.placeholder(), Some("Your name"));
        assert!(!node.is_read_only());
        assert!(!node.is_disabled());
    }

    #[gpui::test]
    fn 中央揃えなら収まる文字列を中央に置き_押した位置にカーソルを移す(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        update(&view, cx, |view| view.align = TextAlign::Center);
        focus(cx);
        let (x, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(
            x, 199.,
            "幅 200 からカーソルの幅を除いた残りの中央に置くこと"
        );
        cx.simulate_input("mm");
        cx.simulate_keystrokes("home");
        let (start, ..) = filled(cx, CARET_COLOR())[0];
        cx.simulate_keystrokes("end");
        let (end, ..) = filled(cx, CARET_COLOR())[0];
        assert!(
            start > 100. && (start + end + 2. - 400.).abs() < 1.,
            "文字列を中央に置くこと"
        );
        // 1 文字目と 2 文字目の境界の少し左を押すと、境界にカーソルを移す。
        // gpui は最後の文字の始まりより右の位置を末尾として扱うため、境界ちょうどは押さない
        mouse_down(cx, (start + end) / 2. - 2., 1, Modifiers::none());
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "mxm");
    }

    #[gpui::test]
    fn 親から継承した右揃えなら収まる文字列を右に置き_はみ出す間はカーソルが見える位置までずらす(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "");
        update(&view, cx, |view| view.align = TextAlign::Right);
        focus(cx);
        let (x, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(x, 298., "右端にカーソルの幅を空けて置くこと");
        cx.simulate_input(&"m".repeat(100));
        cx.simulate_keystrokes("home");
        let (x, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(x, 100., "はみ出す間は左端から見せること");
        // 短くすると右揃えに戻り、押した位置に対応する文字の境界へカーソルを移す
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("mm");
        cx.simulate_keystrokes("home");
        let (start, ..) = filled(cx, CARET_COLOR())[0];
        cx.simulate_keystrokes("end");
        let (end, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(end, 298.);
        assert!(start > 250., "短い文字列は右に寄せること");
        mouse_down(cx, (start + end) / 2. - 2., 1, Modifiers::none());
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "mxm");
    }

    #[gpui::test]
    fn エラーがあれば_invalid_にしてメッセージを説明の先頭に加える(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "ab");
        assert_eq!(node(&view, cx).invalid(), None);
        update(&view, cx, |view| view.error = Some("Required".into()));
        let node = node(&view, cx);
        assert_eq!(node.invalid(), Some(accesskit::Invalid::True));
        assert_eq!(node.description(), Some("Required Full name"));
    }

    #[gpui::test]
    fn フォーカスがある間だけカーソルを描き長い文字列はカーソルが見える位置までずらす(
        cx: &mut TestAppContext,
    ) {
        let (_, cx) = setup(cx, "");
        assert_eq!(filled(cx, CARET_COLOR()), []);
        focus(cx);
        let (x, y, width, height) = filled(cx, CARET_COLOR())[0];
        assert_eq!((x, y, width, height), (100., 100., 2., 20.));
        cx.simulate_input(&"m".repeat(100));
        let (x, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(x, 298., "右端にカーソルの幅を空けて末尾を見せること");
        cx.simulate_keystrokes("home");
        let (x, ..) = filled(cx, CARET_COLOR())[0];
        assert_eq!(x, 100.);
        cx.simulate_keystrokes("shift-end");
        assert_eq!(
            filled(cx, CARET_COLOR()),
            [],
            "選択中はカーソルを描かないこと"
        );
        let (x, y, width, height) = filled(cx, SELECTION_COLOR())[0];
        assert_eq!(
            (x + width, y, height),
            (298., 100., 20.),
            "末尾のカーソルが見える位置までずらすこと"
        );
        cx.simulate_keystrokes("tab");
        assert_eq!(filled(cx, CARET_COLOR()), []);
    }

    #[gpui::test]
    fn 選択中に_enter_を押しても選択している文字列を消さない(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "abc");
        focus(cx);
        cx.simulate_keystrokes("ctrl-a enter");
        assert_eq!(value(&view, cx), "abc");
        assert_eq!(changes(&view, cx), Vec::<SharedString>::new());
    }

    #[gpui::test]
    fn 改行を含む文字列は空白として描き編集できる(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "a\nb");
        focus(cx);
        cx.simulate_keystrokes("end");
        cx.simulate_input("c");
        assert_eq!(value(&view, cx), "a\nbc");
    }

    #[gpui::test]
    fn 二回続けて押すと押した位置の文字を含む語を選択する(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = setup(cx, "foo bar");
        // 3 文字目の右半分を押す。最も近い文字の境界は語の後ろの空白の前になる
        let (fo, foo) = cx.update(|window, _| {
            let mut style = window.text_style();
            style.line_height = px(20.).into();
            let font_size = style.font_size.to_pixels(window.rem_size());
            let width = |text: &'static str| {
                window
                    .text_system()
                    .shape_line(text.into(), font_size, &[style.to_run(text.len())], None)
                    .width
            };
            (width("fo"), width("foo"))
        });
        let x = f32::from(px(100.) + fo + (foo - fo) * 0.75);
        mouse_down(cx, x, 2, Modifiers::none());
        mouse_up(cx, x);
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "x bar");
    }

    #[gpui::test]
    fn ボタンを押していない移動では選択を広げない(cx: &mut TestAppContext) {
        let (view, cx) = setup(cx, "abcd");
        mouse_down(cx, 100., 1, Modifiers::none());
        // 離す操作が届かなかった場合を、離さずにボタンを押していない移動を送って確かめる
        cx.simulate_mouse_move(point(px(400.), px(110.)), None, Modifiers::none());
        cx.simulate_mouse_move(
            point(px(400.), px(110.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        cx.simulate_input("x");
        assert_eq!(value(&view, cx), "xabcd");
    }

    #[gpui::test]
    fn 描いた後に文字列が変わっていれば最後に描いた形を使わない(
        cx: &mut TestAppContext,
    ) {
        let cx = cx.add_empty_window();
        cx.update(|window, cx| {
            let shape = |text: &'static str, window: &mut Window| {
                let style = window.text_style();
                let font_size = style.font_size.to_pixels(window.rem_size());
                window.text_system().shape_line(
                    text.into(),
                    font_size,
                    &[style.to_run(text.len())],
                    None,
                )
            };
            // 同じバイト数でも文字の境界が違う文字列の形は使わない
            let mut state = EditState {
                model: EditModel {
                    text: "あ".into(),
                    ..EditModel::default()
                },
                focus_handle: cx.focus_handle(),
                on_change: None,
                before_edit: None,
                composition: None,
                layout: Some((shape("aaa", window), point(px(0.), px(0.)), px(20.))),
                scroll: px(0.),
                selecting: false,
            };
            assert!(state.current_layout().is_none());
            assert_eq!(state.char_at_x(px(10.)), 0);
            assert_eq!(state.offset_for_x(px(10.)), 0);

            // 改行を空白に変えて描いた形は使う
            state.model.text = "a\nb".into();
            state.layout = Some((shape("a b", window), point(px(0.), px(0.)), px(20.)));
            assert!(state.current_layout().is_some());
        });
    }
}
