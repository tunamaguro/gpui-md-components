//! 部品が使うキー操作（gpui のアクション）と、その割り当て。
//!
//! Tab / Shift-Tab でフォーカスを移し、部品内の操作にはキーコンテキストを付ける。
//! - テキストフィールドの編集操作（[`TEXT_FIELD_CONTEXT`]）。改行と上下の移動は複数行の欄だけ
//!   （[`MULTI_LINE_FIELD_CONTEXT`]）で、1 行の欄では親の要素へ渡る
//! - Dialog / モーダルなシートの Esc と Tab。フォーカスを中に閉じ込める（[`MODAL_CONTEXT`]）
//! - メニューの Esc と Tab。メニューを閉じる（[`MENU_CONTEXT`]）
//! - フォーカスで出たツールチップの Esc（[`FOCUS_TOOLTIP_CONTEXT`]）
//!
//! gpui はコンテキストのない割り当てを「最も深いコンテキスト」と同じ優先度として扱い、同じ優先度では
//! 後から足した割り当てが勝つ。Tab を変更するアプリは [`init`] の後で割り当てる。
//! モーダルの中にさらに内側のコンテキストがあると、モーダルだけの割り当ては浅くなるので、
//! モーダルの Esc と Tab は最も内側のコンテキストでも効かせる。

use gpui::{App, KeyBinding, Window, actions};

/// テキストフィールドのキーコンテキスト。
pub const TEXT_FIELD_CONTEXT: &str = "HeadlessTextField";
/// 複数行のテキストフィールドのキーコンテキスト。[`TEXT_FIELD_CONTEXT`] と一緒に付く。
pub const MULTI_LINE_FIELD_CONTEXT: &str = "HeadlessMultiLineField";
/// Dialog とモーダルなシートのキーコンテキスト。
pub const MODAL_CONTEXT: &str = "HeadlessModal";
/// メニューのキーコンテキスト。
pub const MENU_CONTEXT: &str = "HeadlessMenu";
/// フォーカスで出たツールチップの基準の要素のキーコンテキスト。
pub const FOCUS_TOOLTIP_CONTEXT: &str = "HeadlessFocusTooltip";

actions!(
    headless,
    [
        /// 開いているオーバーレイ（メニュー、ダイアログ、シート）を閉じる。
        Dismiss,
        /// フォーカスで出たツールチップを閉じる。
        DismissTooltip,
        /// 次の要素にフォーカスを移す。
        FocusNext,
        /// 前の要素にフォーカスを移す。
        FocusPrev,
        /// オーバーレイの中で次の要素にフォーカスを移す。
        FocusNextWithin,
        /// オーバーレイの中で前の要素にフォーカスを移す。
        FocusPrevWithin,
        /// 前の 1 文字を消す。
        Backspace,
        /// 後ろの 1 文字を消す。
        Delete,
        /// カーソルを 1 文字左へ。
        Left,
        /// カーソルを 1 文字右へ。
        Right,
        /// 選択を 1 文字左へ広げる。
        SelectLeft,
        /// 選択を 1 文字右へ広げる。
        SelectRight,
        /// すべて選択する。
        SelectAll,
        /// 表示行の先頭へ。
        Home,
        /// 表示行の末尾へ。
        End,
        /// 表示行の先頭まで選択する。
        SelectHome,
        /// 表示行の末尾まで選択する。
        SelectEnd,
        /// 改行を入れる。
        Newline,
        /// カーソルを 1 行上へ。
        Up,
        /// カーソルを 1 行下へ。
        Down,
        /// 選択を 1 行上へ広げる。
        SelectUp,
        /// 選択を 1 行下へ広げる。
        SelectDown,
        /// カーソルを欄に見えている行数だけ上へ。
        PageUp,
        /// カーソルを欄に見えている行数だけ下へ。
        PageDown,
        /// 選択を欄に見えている行数だけ上へ広げる。
        SelectPageUp,
        /// 選択を欄に見えている行数だけ下へ広げる。
        SelectPageDown,
        /// 文字列の先頭へ。
        DocumentStart,
        /// 文字列の末尾へ。
        DocumentEnd,
        /// 文字列の先頭まで選択する。
        SelectDocumentStart,
        /// 文字列の末尾まで選択する。
        SelectDocumentEnd,
        /// 貼り付ける。
        Paste,
        /// 切り取る。
        Cut,
        /// コピーする。
        Copy,
    ]
);

/// 編集のショートカットの修飾キー（macOS は cmd、それ以外は ctrl）。wasm では OS を区別しない。
macro_rules! shortcut {
    ($key:literal) => {
        if cfg!(target_os = "macos") {
            concat!("cmd-", $key)
        } else {
            concat!("ctrl-", $key)
        }
    };
}

fn with_active_window(cx: &mut App, f: impl FnOnce(&mut Window, &mut App) + 'static) {
    // キー入力の処理中はウィンドウを借りているので、終わってから更新する。
    cx.defer(|cx| {
        if let Some(window) = cx.active_window() {
            window.update(cx, |_, window, cx| f(window, cx)).ok();
        }
    });
}

/// 部品のキー操作を割り当てる。アプリごとに起動時に 1 回だけ呼ぶ。
pub fn init(cx: &mut App) {
    cx.on_action(|_: &FocusNext, cx| with_active_window(cx, |window, cx| window.focus_next(cx)));
    cx.on_action(|_: &FocusPrev, cx| with_active_window(cx, |window, cx| window.focus_prev(cx)));
    let text = Some(TEXT_FIELD_CONTEXT);
    let multi_line = Some(MULTI_LINE_FIELD_CONTEXT);
    let (document_start, document_end, select_document_start, select_document_end) =
        if cfg!(target_os = "macos") {
            ("cmd-up", "cmd-down", "cmd-shift-up", "cmd-shift-down")
        } else {
            ("ctrl-home", "ctrl-end", "ctrl-shift-home", "ctrl-shift-end")
        };
    let modal = Some(MODAL_CONTEXT);
    let menu = Some(MENU_CONTEXT);
    let inside_modal = format!("{MODAL_CONTEXT} > !{MENU_CONTEXT}");
    let inside_modal = Some(inside_modal.as_str());
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrev, None),
        KeyBinding::new("backspace", Backspace, text),
        KeyBinding::new("delete", Delete, text),
        KeyBinding::new("left", Left, text),
        KeyBinding::new("right", Right, text),
        KeyBinding::new("shift-left", SelectLeft, text),
        KeyBinding::new("shift-right", SelectRight, text),
        KeyBinding::new(shortcut!("a"), SelectAll, text),
        KeyBinding::new("home", Home, text),
        KeyBinding::new("end", End, text),
        KeyBinding::new("shift-home", SelectHome, text),
        KeyBinding::new("shift-end", SelectEnd, text),
        KeyBinding::new(shortcut!("v"), Paste, text),
        KeyBinding::new(shortcut!("x"), Cut, text),
        KeyBinding::new(shortcut!("c"), Copy, text),
        KeyBinding::new("enter", Newline, multi_line),
        KeyBinding::new("shift-enter", Newline, multi_line),
        KeyBinding::new("up", Up, multi_line),
        KeyBinding::new("down", Down, multi_line),
        KeyBinding::new("shift-up", SelectUp, multi_line),
        KeyBinding::new("shift-down", SelectDown, multi_line),
        KeyBinding::new("pageup", PageUp, multi_line),
        KeyBinding::new("pagedown", PageDown, multi_line),
        KeyBinding::new("shift-pageup", SelectPageUp, multi_line),
        KeyBinding::new("shift-pagedown", SelectPageDown, multi_line),
        KeyBinding::new(document_start, DocumentStart, multi_line),
        KeyBinding::new(document_end, DocumentEnd, multi_line),
        KeyBinding::new(select_document_start, SelectDocumentStart, multi_line),
        KeyBinding::new(select_document_end, SelectDocumentEnd, multi_line),
        KeyBinding::new("escape", Dismiss, modal),
        KeyBinding::new("tab", FocusNextWithin, modal),
        KeyBinding::new("shift-tab", FocusPrevWithin, modal),
        // 内側のコンテキストがあっても、アプリの割り当てに負けて外へ漏らさない。
        KeyBinding::new("escape", Dismiss, inside_modal),
        KeyBinding::new("tab", FocusNextWithin, inside_modal),
        KeyBinding::new("shift-tab", FocusPrevWithin, inside_modal),
        // メニューは Tab でも閉じる（WAI-ARIA の menu パターン）。モーダルより内側なのでこちらが優先される。
        KeyBinding::new("escape", Dismiss, menu),
        KeyBinding::new("tab", Dismiss, menu),
        KeyBinding::new("shift-tab", Dismiss, menu),
        // `inside_modal` と同じ深さになるので、後に足して Esc でダイアログより先にツールチップを閉じる。
        KeyBinding::new("escape", DismissTooltip, Some(FOCUS_TOOLTIP_CONTEXT)),
    ]);
}
