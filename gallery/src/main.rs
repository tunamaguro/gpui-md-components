//! md3-components の全部品の見本アプリ。
//!
//! テーマの切り替え（シード色・バリアント・コントラスト・外観・密度・モーション）と、全カラーロール・タイプスケール・
//! 高さ・アイコンの見本を表示する。Tab / Shift-Tab でフォーカスを移すとフォーカスリングが出る。
//! wasm 版は `cargo xtask web` でビルドする。ブラウザではシステムフォントを読めないので、ページが
//! フォントを取得して [`start`] に渡す。
//!
//! `cargo run -p gallery -- --probe` で起動すると、最初のフレームを描いた後に
//! 使用中の GPU を標準出力に書いて終了する（描画バックエンドの動作確認用）。

#![cfg_attr(target_family = "wasm", no_main)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod stories;

use gpui::{
    App, Bounds, Context, Entity, FocusHandle, KeyBinding, TitlebarOptions,
    UniformListScrollHandle, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px,
    size,
};
use gpui_headless::CheckboxState;
use md3_components::buttons::{Segment, SegmentedButton};
use md3_components::overlays::SnackbarHost;
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{self, Md3ThemeExt, TypeScale};
use md3_components::{IconName, StyledExt, Text};

actions!(
    gallery,
    [
        /// 次の要素にフォーカスを移す（Tab）。
        FocusNext,
        /// 前の要素にフォーカスを移す（Shift-Tab）。
        FocusPrev
    ]
);

/// gallery のページ。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    /// 色・文字・高さ・アイコン。
    #[default]
    Foundations,
    /// ボタン類。
    Buttons,
    /// 選択・状態表示の部品。
    Selection,
    /// 入力の部品。
    Inputs,
    /// オーバーレイの部品。
    Overlays,
    /// ナビゲーションの部品。
    Navigation,
    /// 入れ物の部品。
    Containment,
    /// M3 Expressive の部品。
    Expressive,
    /// 独自テーマ。
    Custom,
}

impl Page {
    const ALL: [Page; 9] = [
        Page::Foundations,
        Page::Buttons,
        Page::Selection,
        Page::Inputs,
        Page::Overlays,
        Page::Navigation,
        Page::Containment,
        Page::Expressive,
        Page::Custom,
    ];
}

/// 操作できる見本の状態。
#[derive(Clone, Debug)]
pub struct Demo {
    /// ボタンが押された回数。
    pub clicks: u32,
    /// トグルのアイコンボタン。
    pub favorite: bool,
    /// 単一選択の SegmentedButton。
    pub segment: usize,
    /// 複数選択の SegmentedButton。
    pub segments: Vec<usize>,
    /// チェックボックス（親と 2 つの子）。
    pub checks: [bool; 2],
    /// スイッチ。
    pub switch: bool,
    /// ラジオボタン。
    pub radio: Option<usize>,
    /// 連続値のスライダー。
    pub volume: f32,
    /// 離散値のスライダー。
    pub rating: f32,
    /// Filter チップ。
    pub filters: [bool; 3],
    /// Input チップ。
    pub tags: Vec<&'static str>,
    /// Select で選んだ色。
    pub color: Option<usize>,
    /// メニュー・ダイアログで最後に選んだ操作。
    pub last_action: Option<&'static str>,
    /// メニューを開いているか。
    pub menu_open: bool,
    /// アイコンボタンのメニューを開いているか。
    pub icon_menu_open: bool,
    /// ダイアログを開いているか。
    pub dialog_open: bool,
    /// ボトムシートを開いているか。
    pub bottom_sheet_open: bool,
    /// Standard のサイドシートを出しているか。
    pub side_sheet_open: bool,
    /// Modal のサイドシートを開いているか。
    pub modal_side_sheet_open: bool,
    /// ナビゲーションで選んだ行き先。
    pub destination: usize,
    /// Primary のタブ。
    pub tab: usize,
    /// Secondary のタブ。
    pub secondary_tab: usize,
    /// モーダルなドロワーを開いているか。
    pub drawer_open: bool,
    /// リストで選んだ項目。
    pub list_selected: Option<usize>,
    /// LazyList で選んだ連絡先。
    pub contact: Option<usize>,
    /// ToggleButton。
    pub toggle: bool,
    /// Standard の ButtonGroup（太字・斜体・下線）。
    pub formats: [bool; 3],
    /// Connected の ButtonGroup で選んだ期間。
    pub range: usize,
    /// 複数選択の Connected の ButtonGroup。
    pub shown: [bool; 3],
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            clicks: 0,
            favorite: false,
            segment: 0,
            segments: vec![0],
            checks: [true, false],
            switch: true,
            radio: Some(0),
            volume: 0.4,
            rating: 3.0,
            filters: [true, false, false],
            tags: vec!["rust", "gpui", "material"],
            color: None,
            last_action: None,
            menu_open: false,
            icon_menu_open: false,
            dialog_open: false,
            bottom_sheet_open: false,
            side_sheet_open: false,
            modal_side_sheet_open: false,
            destination: 0,
            tab: 0,
            secondary_tab: 0,
            drawer_open: false,
            list_selected: Some(0),
            contact: None,
            toggle: false,
            formats: [true, false, false],
            range: 1,
            shown: [true, false, true],
        }
    }
}

impl Demo {
    /// 親のチェックボックスの状態（子の状態から決まる）。
    pub fn parent_check(&self) -> CheckboxState {
        match self.checks {
            [true, true] => CheckboxState::Checked,
            [false, false] => CheckboxState::Unchecked,
            _ => CheckboxState::Indeterminate,
        }
    }
}

/// ギャラリーのルートビュー。
pub struct Gallery {
    focus_handle: FocusHandle,
    page: Page,
    demo: Demo,
    fields: stories::inputs::Fields,
    snackbar: Entity<SnackbarHost>,
    contacts: UniformListScrollHandle,
    custom: Entity<stories::custom_theme::Custom>,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let page = match std::env::var("MD3_GALLERY_PAGE").as_deref() {
            Ok("buttons") => Page::Buttons,
            Ok("selection") => Page::Selection,
            Ok("inputs") => Page::Inputs,
            Ok("overlays") => Page::Overlays,
            Ok("navigation") => Page::Navigation,
            Ok("containment") => Page::Containment,
            Ok("expressive") => Page::Expressive,
            Ok("custom") => Page::Custom,
            _ => Page::Foundations,
        };
        Self {
            focus_handle,
            page,
            demo: Demo::default(),
            fields: stories::inputs::Fields::default(),
            snackbar: cx.new(|_| SnackbarHost::new("閉じる")),
            contacts: UniformListScrollHandle::new(),
            custom: cx.new(stories::custom_theme::Custom::new),
        }
    }

    /// 見本の状態を書き換えて再描画する。
    pub fn update_demo(view: &gpui::WeakEntity<Self>, cx: &mut App, f: impl FnOnce(&mut Demo)) {
        view.update(cx, |this, cx| {
            f(&mut this.demo);
            cx.notify();
        })
        .ok();
    }

    fn focus_next(&mut self, _: &FocusNext, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn focus_prev(&mut self, _: &FocusPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

fn page_selector(page: Page, view: gpui::WeakEntity<Gallery>) -> SegmentedButton {
    SegmentedButton::new("pages")
        .aria_label("ページ")
        .segment(Segment::new("Foundations").icon(IconName::Palette))
        .segment(Segment::new("Buttons").icon(IconName::Add))
        .segment(Segment::new("Selection").icon(IconName::Star))
        .segment(Segment::new("Inputs").icon(IconName::Edit))
        .segment(Segment::new("Overlays").icon(IconName::Menu))
        .segment(Segment::new("Navigation").icon(IconName::Home))
        .segment(Segment::new("Containment").icon(IconName::Inbox))
        .segment(Segment::new("Expressive").icon(IconName::Favorite))
        .segment(Segment::new("Custom").icon(IconName::Palette))
        // ページ移動でありラジオグループではないため、全項目を Tab で選べるようにする。
        .multi_select(true)
        .selected([page as usize])
        .on_change({
            let view = view.clone();
            move |index, _, cx| {
                let page = Page::ALL.get(index).copied().unwrap_or_default();
                view.update(cx, |this, cx| {
                    this.page = page;
                    cx.notify();
                })
                .ok();
            }
        })
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = cx.md3().clone();
        let view = cx.entity().downgrade();
        let page = self.page;
        let pages = page_selector(page, view.clone());
        let content = match page {
            Page::Foundations => stories::theme::foundations(&current).into_any_element(),
            Page::Buttons => {
                stories::buttons::page(&self.demo, view, window, cx).into_any_element()
            }
            Page::Selection => {
                stories::selection::page(&self.demo, view, window, cx).into_any_element()
            }
            Page::Inputs => stories::inputs::page(&self.demo, &mut self.fields, view, window, cx)
                .into_any_element(),
            Page::Overlays => {
                let note = self.fields.get("dialog-note", "", cx);
                let sample_note = self.fields.get("dialog-sample-note", "", cx);
                stories::overlays::page(
                    &self.demo,
                    &note,
                    &sample_note,
                    &self.snackbar,
                    view,
                    window,
                    cx,
                )
                .into_any_element()
            }
            Page::Navigation => {
                let search = self.fields.get("search-people", "", cx);
                stories::navigation::page(&self.demo, &search, view, window, cx).into_any_element()
            }
            Page::Containment => div()
                .flex()
                .flex_col()
                .gap_6()
                .child(stories::scrollbar::page(window, cx))
                .child(stories::containment::page(
                    &self.demo,
                    &self.contacts,
                    view,
                    window,
                    cx,
                ))
                .into_any_element(),
            Page::Expressive => {
                stories::expressive::page(&self.demo, view, window, cx).into_any_element()
            }
            Page::Custom => self.custom.clone().into_any_element(),
        };

        let root_scroll = window
            .use_keyed_state("gallery-scroll", cx, |_, _| gpui::ScrollHandle::new())
            .read(cx)
            .clone();
        gpui_headless::ScrollScope::new(
            root_scroll.clone(),
            div()
                .id("gallery")
                .key_context("Gallery")
                .track_focus(&self.focus_handle)
                .on_action(cx.listener(Self::focus_next))
                .on_action(cx.listener(Self::focus_prev))
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&root_scroll)
                .bg(current.color(ColorRole::Surface))
                .text_color(current.color(ColorRole::OnSurface))
                .type_style(&current.text_style(TypeScale::BodyMedium))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_6()
                        .p_6()
                        .child(
                            Text::new(TypeScale::HeadlineMedium, "md3 gallery")
                                .id("gallery-heading"),
                        )
                        .child(
                            Text::new(TypeScale::BodySmall, gpu_description(window))
                                .id("gallery-gpu-description")
                                .color(ColorRole::OnSurfaceVariant),
                        )
                        .child(stories::theme::controls(window, cx))
                        .child(div().w(px(1150.0)).child(pages))
                        .child(content),
                )
                .child(self.snackbar.clone())
                .child(md3_components::Scrollbar::new(
                    "gallery-scrollbar",
                    root_scroll,
                )),
        )
    }
}

/// `fonts` は起動時に読み込むフォントのデータ（デスクトップではシステムフォントを使うので空）。
/// `reduce_motion` は OS・ブラウザの「視差効果を減らす」相当の設定（gpui は OS の設定を読まないので、起動側が渡す）。
fn run(app: gpui::Application, probe: bool, fonts: Vec<Vec<u8>>, reduce_motion: bool) {
    app.run(move |cx: &mut App| {
        cx.set_reduce_motion(reduce_motion);
        if !fonts.is_empty() {
            let fonts = fonts.into_iter().map(std::borrow::Cow::Owned).collect();
            if let Err(error) = cx.text_system().add_fonts(fonts) {
                eprintln!("failed to load the fonts: {error:#}");
            }
        }
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
        // 部品の割り当て（ダイアログの中の Tab など）がアプリの Tab より優先されるよう、後から足す。
        md3_components::init(cx);
        let bounds = Bounds::centered(None, size(px(1200.), px(900.)), cx);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("md3 gallery".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                theme::follow_window_appearance(window, cx).detach();
                cx.new(|cx| Gallery::new(window, cx))
            },
        );
        let window = match opened {
            Ok(window) => window,
            Err(error) => {
                eprintln!("failed to open the gallery window: {error:#}");
                exit_with_failure(cx);
                return;
            }
        };
        if probe {
            let scheduled = window.update(cx, |_, window, _| {
                window.on_next_frame(|window, cx| {
                    println!("{}", gpu_description(window));
                    cx.quit();
                });
            });
            if let Err(error) = scheduled {
                eprintln!("failed to schedule the probe: {error:#}");
                exit_with_failure(cx);
                return;
            }
        }
        cx.activate(true);
    });
}

/// 使用中の GPU を 1 行で説明する。
fn gpu_description(window: &Window) -> String {
    match window.gpu_specs() {
        Some(specs) => format!(
            "gpu: {} (driver: {} {}, software: {})",
            specs.device_name, specs.driver_name, specs.driver_info, specs.is_software_emulated
        ),
        None => "gpu: unknown (the platform does not report it)".to_owned(),
    }
}

/// 起動に失敗したことを呼び出し元に伝えて終了する。
///
/// Linux ではウィンドウを開けなかった後に `App::quit` を呼んでもイベントループが止まらないため、
/// デスクトップではプロセスを直接終了する。
fn exit_with_failure(cx: &mut App) {
    #[cfg(not(target_family = "wasm"))]
    {
        let _ = cx;
        std::process::exit(1);
    }
    #[cfg(target_family = "wasm")]
    cx.quit();
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    let probe = std::env::args().skip(1).any(|arg| arg == "--probe");
    // デスクトップでは OS の設定を読む手段が gpui に無いため、環境変数で指定する（gallery の画面でも切り替えられる）。
    let reduce_motion = std::env::var("MD3_REDUCE_MOTION").is_ok_and(|value| value == "1");
    run(
        gpui_platform::application(),
        probe,
        Vec::new(),
        reduce_motion,
    );
}

/// wasm 版のエントリポイント。ブラウザでは gpui がシステムフォントを読めないため、ページが
/// UI の文字に使うフォント（gpui の既定の候補の「Noto Sans」と、日本語の字を持つフォント）を取得し、
/// 1 つの配列 `fonts` につなげて、それぞれの長さ `lengths` と一緒に渡す。
/// `webgl` が真なら WebGPU を使わず WebGL2 で描く（ページの `?renderer=webgl`）。
/// `reduce_motion` はブラウザの `prefers-reduced-motion: reduce`。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(fonts: Vec<u8>, lengths: Vec<u32>, webgl: bool, reduce_motion: bool) {
    use gpui_platform::WebBackendPreference;
    gpui_platform::web_init();
    let mut rest = fonts.as_slice();
    let mut split = Vec::new();
    for length in lengths {
        let (font, tail) = rest.split_at(rest.len().min(length as usize));
        split.push(font.to_vec());
        rest = tail;
    }
    let backend = if webgl {
        WebBackendPreference::WebGl
    } else {
        WebBackendPreference::Auto
    };
    run(
        gpui_platform::application_with_web_backend(backend),
        false,
        split,
        reduce_motion,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn custom_theme_does_not_import_theme_crates() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stories/custom_theme.rs");
        let source = std::fs::read_to_string(path).expect("独自テーマのソースを読む");
        let mut in_use = false;
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("use ") || line.starts_with("pub use ") {
                in_use = true;
            }
            if in_use {
                assert!(
                    !line.to_ascii_lowercase().contains("md3"),
                    "独自テーマの use: {line}"
                );
                if line.contains(';') {
                    in_use = false;
                }
            }
        }
    }

    #[gpui::test]
    fn custom_theme_renders(cx: &mut TestAppContext) {
        cx.update(gpui_headless::init);
        let (_, cx) = cx.add_window_view(|_, cx| stories::custom_theme::Custom::new(cx));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }

    struct SelectorTest {
        root: FocusHandle,
        gallery: Entity<Gallery>,
    }

    impl Render for SelectorTest {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("selector-test")
                .track_focus(&self.root)
                .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
                .child(page_selector(Page::Foundations, self.gallery.downgrade()))
        }
    }

    #[gpui::test]
    fn tab_reaches_every_page_selector_segment(cx: &mut TestAppContext) {
        cx.update(|cx| {
            cx.bind_keys([KeyBinding::new("tab", FocusNext, None)]);
            md3_components::init(cx);
        });
        let (_, cx) = cx.add_window_view(|window, cx| {
            let gallery = cx.new(|cx| Gallery::new(window, cx));
            let root = cx.focus_handle();
            window.focus(&root, cx);
            SelectorTest { root, gallery }
        });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let root = cx.update(|window, cx| window.focused(cx).expect("ルートにフォーカス"));
        let mut focused = Vec::new();
        for _ in 0..Page::ALL.len() {
            cx.simulate_keystrokes("tab");
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let handle = window.focused(cx).expect("ページ項目にフォーカス");
                assert_ne!(handle, root, "Tab でページ項目に移る");
                if !focused.contains(&handle) {
                    focused.push(handle);
                }
            });
        }
        assert_eq!(
            focused.len(),
            Page::ALL.len(),
            "Tab で全ページの項目に移動する"
        );
    }
}
