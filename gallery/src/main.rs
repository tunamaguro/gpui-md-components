//! md3-components の全部品の見本アプリ。
//!
//! M0 時点では描画できていることを確かめるため、使用中の GPU 名を表示するだけ。wasm 版のビルド・配信は M7（`cargo xtask web`）で整備する。
//!
//! `cargo run -p gallery -- --probe` で起動すると、最初のフレームを描いた後に
//! 使用中の GPU を標準出力に書いて終了する（描画バックエンドの動作確認用）。

#![cfg_attr(target_family = "wasm", no_main)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use gpui::{
    App, Bounds, Context, TitlebarOptions, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};

/// ギャラリーのルートビュー。
struct Gallery;

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // テーマ（M3）ができるまでの仮の見た目。MD3 のトークン値ではない。
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(rgb(0xffffff))
            .text_color(rgb(0x000000))
            .child("md3 gallery")
            .child(gpu_description(window))
    }
}

fn run(probe: bool) {
    gpui_platform::application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(720.)), cx);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("md3 gallery".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Gallery),
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
