//! md3-components の全部品の見本アプリ。
//!
//! テーマの切り替え（シード色・バリアント・コントラスト・外観・密度）と、全カラーロール・タイプスケール・
//! 高さ・アイコンの見本を表示する。Tab / Shift-Tab でフォーカスを移すとフォーカスリングが出る。
//! wasm 版のビルド・配信は M7（`cargo xtask web`）で整備する。
//!
//! `cargo run -p gallery -- --probe` で起動すると、最初のフレームを描いた後に
//! 使用中の GPU を標準出力に書いて終了する（描画バックエンドの動作確認用）。

#![cfg_attr(target_family = "wasm", no_main)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod stories;

use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, size,
};
use md3_components::buttons::{Segment, SegmentedButton};
use md3_components::selection::CheckboxState;
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
}

impl Page {
    const ALL: [Page; 3] = [Page::Foundations, Page::Buttons, Page::Selection];
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
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let page = match std::env::var("MD3_GALLERY_PAGE").as_deref() {
            Ok("buttons") => Page::Buttons,
            Ok("selection") => Page::Selection,
            _ => Page::Foundations,
        };
        Self {
            focus_handle,
            page,
            demo: Demo::default(),
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

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = cx.md3().clone();
        let view = cx.entity().downgrade();
        let page = self.page;
        let pages = SegmentedButton::new("pages", "ページ")
            .segment(Segment::new("Foundations").icon(IconName::Palette))
            .segment(Segment::new("Buttons").icon(IconName::Add))
            .segment(Segment::new("Selection").icon(IconName::Star))
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
            });
        let content = match page {
            Page::Foundations => stories::theme::foundations(&current).into_any_element(),
            Page::Buttons => {
                stories::buttons::page(&self.demo, view, window, cx).into_any_element()
            }
            Page::Selection => {
                stories::selection::page(&self.demo, view, window, cx).into_any_element()
            }
        };

        div()
            .id("gallery")
            .key_context("Gallery")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::focus_next))
            .on_action(cx.listener(Self::focus_prev))
            .size_full()
            .overflow_y_scroll()
            .bg(current.color(ColorRole::Surface))
            .text_color(current.color(ColorRole::OnSurface))
            .type_style(&current.text_style(TypeScale::BodyMedium))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .p_6()
                    .child(Text::new(TypeScale::HeadlineMedium, "md3 gallery"))
                    .child(
                        Text::new(TypeScale::BodySmall, gpu_description(window))
                            .color(ColorRole::OnSurfaceVariant),
                    )
                    .child(stories::theme::controls(window, cx))
                    .child(div().w(px(480.0)).child(pages))
                    .child(content),
            )
    }
}

fn run(probe: bool) {
    gpui_platform::application().run(move |cx: &mut App| {
        theme::init(cx);
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
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
    run(probe);
}

/// wasm 版のエントリポイント。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run(false);
}
