//! 部品が使うキー操作（gpui のアクション）と、その割り当て。
//!
//! 部品の中だけで効くように、割り当てにはキーコンテキストを付ける。
//! - テキストフィールドの編集操作（[`TEXT_FIELD_CONTEXT`]）
//! - Dialog / モーダルなシートの Esc と Tab。フォーカスを中に閉じ込める（[`MODAL_CONTEXT`]）
//! - メニューの Esc と Tab。メニューを閉じる（[`MENU_CONTEXT`]）
//! - フォーカスで出たツールチップの Esc（[`FOCUS_TOOLTIP_CONTEXT`]）
//!
//! gpui はコンテキストのない割り当てを「最も深いコンテキスト」と同じ優先度として扱い、同じ優先度では
//! 後から足した割り当てが勝つ。アプリが Tab などをコンテキストなしで割り当てる場合は、その後で
//! [`init`] を呼ぶこと。モーダルの中にさらに内側のコンテキストがあると、モーダルだけの割り当ては
//! 浅くなってアプリの割り当てに負けるので、モーダルの Esc と Tab は最も内側のコンテキストでも効かせる。

use gpui::{App, KeyBinding, actions};

/// テキストフィールドのキーコンテキスト。
pub const TEXT_FIELD_CONTEXT: &str = "HeadlessTextField";
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
        /// 行頭へ。
        Home,
        /// 行末へ。
        End,
        /// 行頭まで選択する。
        SelectHome,
        /// 行末まで選択する。
        SelectEnd,
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

/// 部品のキー操作を割り当てる。アプリの起動時に 1 回呼ぶ。
pub fn init(cx: &mut App) {
    let text = Some(TEXT_FIELD_CONTEXT);
    let modal = Some(MODAL_CONTEXT);
    let menu = Some(MENU_CONTEXT);
    let inside_modal = format!("{MODAL_CONTEXT} > !{MENU_CONTEXT}");
    let inside_modal = Some(inside_modal.as_str());
    cx.bind_keys([
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
