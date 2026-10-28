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

use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, Role, SharedString, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{
    self, Appearance, Argb, Contrast, Density, Elevation, Md3ThemeExt, SchemeVariant,
    SetMd3ThemeExt, Shape, Theme, TypeScale,
};
use md3_components::{
    FocusRing, Icon, IconName, StateLayer, StyledExt, Surface, Text, focus_handle, is_focus_visible,
};

actions!(
    gallery,
    [
        /// 次の要素にフォーカスを移す（Tab）。
        FocusNext,
        /// 前の要素にフォーカスを移す（Shift-Tab）。
        FocusPrev
    ]
);

/// 切り替え UI で選べるシード色（見本用の任意の色）。
const SEEDS: [(&str, Argb); 5] = [
    ("Purple", theme::DEFAULT_SEED),
    ("Blue", Argb::from_rgb(0x00, 0x61, 0xA4)),
    ("Green", Argb::from_rgb(0x38, 0x6A, 0x20)),
    ("Red", Argb::from_rgb(0xB3, 0x26, 0x1E)),
    ("Orange", Argb::from_rgb(0xE8, 0x75, 0x1A)),
];

/// ギャラリーのルートビュー。
struct Gallery {
    focus_handle: FocusHandle,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self { focus_handle }
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
        let seed_options = SEEDS.map(|(name, seed)| {
            let option = Choice::new(format!("seed-{name}"), name, current.seed() == seed);
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().seed(seed).build();
                cx.set_md3_theme(theme);
            })
        });
        let variant_options = SchemeVariant::ALL.map(|variant| {
            let name = format!("{variant:?}");
            let option = Choice::new(
                format!("variant-{name}"),
                name,
                current.variant() == variant,
            );
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().variant(variant).build();
                cx.set_md3_theme(theme);
            })
        });
        let contrast_options = Contrast::ALL.map(|contrast| {
            let name = format!("{contrast:?}");
            let option = Choice::new(
                format!("contrast-{name}"),
                name,
                current.contrast() == contrast,
            );
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().contrast(contrast).build();
                cx.set_md3_theme(theme);
            })
        });
        let appearance_options = Appearance::ALL.map(|appearance| {
            let name = format!("{appearance:?}");
            let option = Choice::new(
                format!("appearance-{name}"),
                name,
                current.appearance() == appearance,
            )
            .icon(match appearance {
                Appearance::System => IconName::Settings,
                Appearance::Light => IconName::LightMode,
                Appearance::Dark => IconName::DarkMode,
            });
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().appearance(appearance).build();
                cx.set_md3_theme(theme);
            })
        });
        let density_options = Density::ALL.map(|density| {
            let name = format!("{density:?}");
            let option = Choice::new(
                format!("density-{name}"),
                name,
                current.density() == density,
            );
            option.render(window, cx, move |cx| {
                let theme = cx.md3().to_builder().density(density).build();
                cx.set_md3_theme(theme);
            })
        });

        let issues = current.check_contrast();
        let contrast_report = if issues.is_empty() {
            "check_contrast: no issues".to_owned()
        } else {
            format!("check_contrast: {} issue(s)", issues.len())
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
                    .child(
                        Section::new("Theme")
                            .child(option_row("Seed", seed_options))
                            .child(option_row("Variant", variant_options))
                            .child(option_row("Contrast", contrast_options))
                            .child(option_row("Appearance", appearance_options))
                            .child(option_row("Density", density_options))
                            .child(
                                Text::new(TypeScale::BodySmall, contrast_report)
                                    .color(ColorRole::OnSurfaceVariant),
                            ),
                    )
                    .child(Section::new("Color roles").child(swatches(&current)))
                    .child(Section::new("Type scale").child(type_scale()))
                    .child(Section::new("Elevation").child(elevations()))
                    .child(Section::new("Icons").child(icons())),
            )
    }
}

/// 見出し付きのまとまり。
#[derive(IntoElement)]
struct Section {
    title: SharedString,
    body: gpui::Div,
}

impl Section {
    fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: div().flex().flex_col().gap_3(),
        }
    }
}

impl ParentElement for Section {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Section {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Text::new(TypeScale::TitleLarge, self.title))
            .child(self.body)
    }
}

/// 選択肢を 1 行に並べる。
fn option_row(
    label: &str,
    options: impl IntoIterator<Item = gpui::AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_2()
        .child(div().w(px(96.0)).child(
            Text::new(TypeScale::LabelLarge, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        ))
        .children(options)
}

/// 切り替え UI の 1 つの選択肢（M4 の部品ができるまでの、基礎部品を組み合わせた仮のボタン）。
struct Choice {
    id: SharedString,
    label: SharedString,
    selected: bool,
    icon: Option<IconName>,
}

impl Choice {
    fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected,
            icon: None,
        }
    }

    fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    fn render(
        self,
        window: &mut Window,
        cx: &mut App,
        on_select: impl Fn(&mut App) + 'static,
    ) -> gpui::AnyElement {
        let handle = focus_handle(self.id.clone(), window, cx);
        let focused = is_focus_visible(&handle, window);
        let theme = cx.md3();
        let radius = theme.corner(Shape::Full);
        let (container, content) = if self.selected {
            (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            )
        } else {
            (ColorRole::SurfaceContainerLow, ColorRole::OnSurface)
        };
        let group = self.id.clone();
        div()
            .id(self.id.clone())
            .group(group.clone())
            .relative()
            .track_focus(&handle)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .aria_selected(self.selected)
            .h(theme.dense(40.0))
            .px_4()
            .flex()
            .items_center()
            .gap_2()
            .bg(theme.color(container))
            .text_color(theme.color(content))
            .border_1()
            .border_color(theme.color(if self.selected {
                container
            } else {
                ColorRole::OutlineVariant
            }))
            .corner_radius(radius)
            .cursor_pointer()
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(px(18.0)).color(content))
            })
            .when(self.selected && self.icon.is_none(), |this| {
                this.child(Icon::new(IconName::Check).size(px(18.0)).color(content))
            })
            .child(Text::new(TypeScale::LabelLarge, self.label))
            .child(
                StateLayer::new(format!("{group}-layer"), group, content)
                    .radius(radius)
                    .focused(focused),
            )
            .child(FocusRing::new(radius).visible(focused))
            .on_click(move |_, _, cx| on_select(cx))
            .into_any_element()
    }
}

/// 全カラーロールの見本。
fn swatches(theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_2()
        .children(ColorRole::ALL.map(|role| {
            let argb = theme.argb(role);
            let content = theme::content_color(role)
                .map(|content| theme.color(content))
                .unwrap_or_else(|| {
                    // 対になる前景色が無いロールは、明るさで白黒を選ぶ。
                    if argb.lstar().value() > 50.0 {
                        gpui::black()
                    } else {
                        gpui::white()
                    }
                });
            div()
                .w(px(168.0))
                .h(px(72.0))
                .p_2()
                .flex()
                .flex_col()
                .justify_between()
                .bg(theme.color(role))
                .text_color(content)
                .border_1()
                .border_color(theme.color(ColorRole::OutlineVariant))
                .rounded(px(8.0))
                .child(Text::new(TypeScale::LabelMedium, role.name()))
                .child(Text::new(
                    TypeScale::LabelSmall,
                    format!("#{:06X}", argb.to_u32() & 0x00FF_FFFF),
                ))
        }))
}

/// タイプスケール 15 種の見本。
fn type_scale() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(TypeScale::ALL.map(|scale| Text::new(scale, format!("{scale:?} あいう Aa"))))
}

/// 高さ 6 段階の見本。
fn elevations() -> impl IntoElement {
    let levels = [
        Elevation::Level0,
        Elevation::Level1,
        Elevation::Level2,
        Elevation::Level3,
        Elevation::Level4,
        Elevation::Level5,
    ];
    div()
        .flex()
        .flex_wrap()
        .gap_6()
        .p_2()
        .children(levels.map(|level| {
            Surface::new(ColorRole::SurfaceContainerLow)
                .elevation(level)
                .shape(Shape::Medium)
                .w(px(112.0))
                .h(px(72.0))
                .flex()
                .items_center()
                .justify_center()
                .child(Text::new(TypeScale::LabelLarge, format!("{level:?}")))
        }))
}

/// アイコンの見本。
fn icons() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_4()
        .children(IconName::ALL.map(Icon::new))
}

fn run(probe: bool) {
    gpui_platform::application().run(move |cx: &mut App| {
        theme::init(cx);
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
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
