//! md3-components を利用するメールアプリの見本。
//!
//! wasm 版は `cargo xtask web` でビルドする。ブラウザではシステムフォントを読めないので、ページが
//! フォントを取得して [`start`] に渡す。

#![cfg_attr(target_family = "wasm", no_main)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod compose;
mod model;

use md3_components::buttons::{Button, ExtendedFab, IconButton, Segment, SegmentedButton};
use md3_components::containment::{List, ListItem, ListTrailing};
use md3_components::gpui;
use md3_components::gpui::{
    App, Bounds, Context, Entity, FocusHandle, KeyBinding, ScrollHandle, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, point, prelude::*, px, size,
};
use md3_components::gpui_platform;
use md3_components::inputs::{SearchBar, TextFieldState};
use md3_components::navigation::{NavigationItem, NavigationSuite, TopAppBar};
use md3_components::overlays::{Menu, MenuItem, SnackbarHost, SnackbarMessage};
use md3_components::scrollbar::ScrollArea;
use md3_components::selection::{Chip, ChipKind, Switch};
use md3_components::theme::md3_color::ColorRole;
use md3_components::theme::{self, Appearance, Md3ThemeExt, SetMd3ThemeExt, TypeScale};
use md3_components::{IconName, StyledExt, Text, WindowSizeClass};
use model::{Folder, Mail};

actions!(
    mail_sample,
    [
        /// 次の要素にフォーカスを移す。
        FocusNext,
        /// 前の要素にフォーカスを移す。
        FocusPrev
    ]
);

struct MailView {
    focus: FocusHandle,
    detail_scroll: ScrollHandle,
    mail: Mail,
    destination: usize,
    selected: Option<u64>,
    unread: bool,
    starred: bool,
    menu: bool,
    search: TextFieldState,
    to: TextFieldState,
    subject: TextFieldState,
    body: TextFieldState,
    compose_open: bool,
    to_error: bool,
    snackbar: Entity<SnackbarHost>,
}

impl MailView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let search = TextFieldState::new(cx);
        search.on_change(cx, |_, cx| cx.notify()).detach();
        Self {
            focus,
            detail_scroll: ScrollHandle::new(),
            mail: Mail::default(),
            destination: 0,
            selected: None,
            unread: false,
            starred: false,
            menu: false,
            search,
            to: TextFieldState::new(cx),
            subject: TextFieldState::new(cx),
            body: TextFieldState::new(cx),
            compose_open: false,
            to_error: false,
            snackbar: cx.new(|_| SnackbarHost::new("Dismiss")),
        }
    }

    fn update(view: &gpui::WeakEntity<Self>, cx: &mut App, f: impl FnOnce(&mut Self)) {
        view.update(cx, |this, cx| {
            f(this);
            cx.notify();
        })
        .ok();
    }

    fn move_message(
        &mut self,
        id: u64,
        folder: Folder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(undo) = self.mail.move_to(id, folder) {
            self.selected = None;
            window.focus(&self.focus, cx);
            let view = cx.entity().downgrade();
            self.snackbar.update(cx, |host, cx| {
                host.show(
                    SnackbarMessage::new(if folder == Folder::Trash {
                        "Message deleted"
                    } else {
                        "Message archived"
                    })
                    .action("Undo", move |_, cx| {
                        Self::update(&view, cx, |this| this.mail.undo(&undo))
                    }),
                    cx,
                )
            });
            cx.notify();
        }
    }
}

impl Render for MailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let expanded = WindowSizeClass::of(window) >= WindowSizeClass::Expanded;
        let theme = cx.md3().clone();
        let unread_count = self
            .mail
            .filtered(Some(Folder::Inbox), "", true, false)
            .len();
        let items = [
            NavigationItem::new(IconName::Inbox, format!("Inbox ({unread_count})")),
            NavigationItem::new(IconName::Star, "Starred"),
            NavigationItem::new(IconName::Send, "Sent"),
            NavigationItem::new(IconName::Delete, "Trash"),
            NavigationItem::new(IconName::Settings, "Settings"),
        ];
        let mut list = List::new("messages")
            .aria_label("Messages")
            .max_height(window.viewport_size().height);
        let folder = match self.destination {
            0 => Some(Folder::Inbox),
            1 => None,
            2 => Some(Folder::Sent),
            3 => Some(Folder::Trash),
            _ => None,
        };
        let messages = self.mail.filtered(
            folder,
            self.search.text(cx),
            self.unread,
            self.starred || self.destination == 1,
        );
        let empty = messages.is_empty();
        for message in messages {
            let id = message.id;
            let message_read = message.read;
            let row_view = view.clone();
            list = list.item(
                format!("message-{id}"),
                ListItem::new(message.subject.clone())
                    .overline(format!(
                        "{}{}",
                        message.sender,
                        if message.read { "" } else { " · Unread" }
                    ))
                    .supporting_text(message.preview())
                    .trailing(ListTrailing::Text(message.time.clone().into()))
                    .selected(self.selected == Some(id))
                    .on_select(move |window, cx| {
                        row_view
                            .update(cx, |this, cx| {
                                if !expanded || (this.unread && !message_read) {
                                    window.focus(&this.focus, cx);
                                }
                                if this.selected != Some(id) {
                                    this.detail_scroll.set_offset(point(px(0.), px(0.)));
                                }
                                this.mail.open(id);
                                this.selected = Some(id);
                                cx.notify();
                            })
                            .ok();
                    }),
            );
        }
        let unread_view = view.clone();
        let starred_view = view.clone();
        let compose_view = view.clone();
        let list_pane = div()
            .relative()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .child(TopAppBar::new("list-title", "Mail"))
            .child(
                div()
                    .px_4()
                    .child(SearchBar::new("search", &self.search, "Search mail")),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .p_4()
                    .child(
                        Chip::new("unread-filter", ChipKind::Filter, "Unread")
                            .selected(self.unread)
                            .on_click(move |_, _, cx| {
                                Self::update(&unread_view, cx, |this| this.unread = !this.unread)
                            }),
                    )
                    .child(
                        Chip::new("starred-filter", ChipKind::Filter, "Starred")
                            .selected(self.starred)
                            .on_click(move |_, _, cx| {
                                Self::update(&starred_view, cx, |this| this.starred = !this.starred)
                            }),
                    ),
            )
            .when(empty, |el| {
                el.child(
                    div().p_4().child(
                        Text::new(TypeScale::BodyLarge, "No messages match.").id("no-messages"),
                    ),
                )
            })
            .child(list)
            .child(
                div().absolute().right_4().bottom_4().child(
                    ExtendedFab::new("compose", "Compose")
                        .icon(IconName::Edit)
                        .on_click(move |_, _, cx| {
                            compose_view
                                .update(cx, |this, cx| this.open_compose(None, cx))
                                .ok();
                        }),
                ),
            );
        let mut detail = div().flex_1().min_w_0().min_h_0().flex().flex_col();
        if let Some(message) = self.selected.and_then(|id| self.mail.get(id)).cloned() {
            let id = message.id;
            let back_view = view.clone();
            let mut top = TopAppBar::new("detail-title", "Message");
            if !expanded {
                top = top.navigation_icon(
                    IconButton::new("back", IconName::ArrowBack)
                        .aria_label("Back to messages")
                        .on_click(move |_, window, cx| {
                            back_view
                                .update(cx, |this, cx| {
                                    this.selected = None;
                                    window.focus(&this.focus, cx);
                                    cx.notify();
                                })
                                .ok();
                        }),
                );
            }
            let menu_view = view.clone();
            let star_view = view.clone();
            let unread_view = view.clone();
            let dismiss_view = view.clone();
            let menu = div()
                .relative()
                .child(
                    IconButton::new("more", IconName::MoreVert)
                        .aria_label("More actions")
                        .menu_expanded(self.menu)
                        .on_click(move |_, _, cx| {
                            Self::update(&menu_view, cx, |this| this.menu = !this.menu)
                        }),
                )
                .when(self.menu, |el| {
                    el.child(
                        Menu::new("message-menu")
                            .aria_label("Message actions")
                            .item(MenuItem::new("Mark as unread").on_select(move |_, cx| {
                                Self::update(&unread_view, cx, |this| {
                                    this.mail.mark_unread(id);
                                    this.menu = false;
                                })
                            }))
                            .item(
                                MenuItem::new(if message.starred { "Unstar" } else { "Star" })
                                    .on_select(move |_, cx| {
                                        Self::update(&star_view, cx, |this| {
                                            this.mail.toggle_star(id);
                                            this.menu = false;
                                        })
                                    }),
                            )
                            .on_dismiss(move |_, cx| {
                                Self::update(&dismiss_view, cx, |this| this.menu = false)
                            }),
                    )
                });
            let archive_view = view.clone();
            let delete_view = view.clone();
            let reply_view = view.clone();
            detail = detail.child(top).child(
                ScrollArea::new("detail-scroll")
                    .handle(self.detail_scroll.clone())
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                Text::new(TypeScale::HeadlineSmall, message.subject)
                                    .id("message-subject"),
                            )
                            .child(
                                Text::new(
                                    TypeScale::BodyMedium,
                                    format!(
                                        "{} <{}> · {}",
                                        message.sender, message.address, message.time
                                    ),
                                )
                                .id("message-meta"),
                            )
                            .child(Text::new(TypeScale::BodyLarge, message.body).id("message-body"))
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .flex_wrap()
                                    .child(Button::new("reply", "Reply").on_click(
                                        move |_, _, cx| {
                                            reply_view
                                                .update(cx, |this, cx| {
                                                    if let Some(draft) = this.mail.reply(id) {
                                                        this.open_compose(Some(draft), cx);
                                                    }
                                                })
                                                .ok();
                                        },
                                    ))
                                    .child(Button::new("archive", "Archive").on_click(
                                        move |_, window, cx| {
                                            archive_view
                                                .update(cx, |this, cx| {
                                                    this.move_message(
                                                        id,
                                                        Folder::Archive,
                                                        window,
                                                        cx,
                                                    )
                                                })
                                                .ok();
                                        },
                                    ))
                                    .child(Button::new("delete", "Delete").on_click(
                                        move |_, window, cx| {
                                            delete_view
                                                .update(cx, |this, cx| {
                                                    this.move_message(id, Folder::Trash, window, cx)
                                                })
                                                .ok();
                                        },
                                    ))
                                    .child(menu),
                            ),
                    ),
            );
        } else {
            detail = detail.child(div().p_6().child(
                Text::new(TypeScale::BodyLarge, "Select a message to read it.").id("no-selection"),
            ));
        }
        let content = if self.destination == 4 {
            let dark = theme.is_dark();
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_4()
                .p_6()
                .child(Text::new(TypeScale::HeadlineMedium, "Settings").id("settings-title"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            Switch::new("dark-mode", dark)
                                .aria_label("Dark mode")
                                .on_change(|dark, _, cx| {
                                    let theme = cx
                                        .md3()
                                        .to_builder()
                                        .appearance(if dark {
                                            Appearance::Dark
                                        } else {
                                            Appearance::Light
                                        })
                                        .build();
                                    cx.set_md3_theme(theme);
                                }),
                        )
                        .child("Dark mode"),
                )
                .child(
                    SegmentedButton::new("text-scale")
                        .aria_label("Text scale")
                        .segment(Segment::new("100%"))
                        .segment(Segment::new("150%"))
                        .segment(Segment::new("200%"))
                        .selected([if theme.text_scale() == 1.5 {
                            1
                        } else if theme.text_scale() == 2.0 {
                            2
                        } else {
                            0
                        }])
                        .on_change(|index, _, cx| {
                            let scale = [1.0, 1.5, 2.0].get(index).copied().unwrap_or(1.0);
                            let theme = cx.md3().to_builder().text_scale(scale).build();
                            cx.set_md3_theme(theme);
                        }),
                )
        } else if expanded {
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .child(list_pane)
                .child(detail)
        } else if self.selected.is_some() {
            detail
        } else {
            list_pane
        };
        let nav_view = view.clone();
        div()
            .id("mail-root")
            .key_context("Mail")
            .track_focus(&self.focus)
            .size_full()
            .bg(theme.color(ColorRole::Surface))
            .text_color(theme.color(ColorRole::OnSurface))
            .type_style(&theme.text_style(TypeScale::BodyMedium))
            .child(
                NavigationSuite::new("navigation")
                    .aria_label("Mail folders")
                    .items(items)
                    .selected(Some(self.destination))
                    .on_change(move |index, _, cx| {
                        Self::update(&nav_view, cx, |this| {
                            this.destination = index;
                            this.selected = None;
                        })
                    })
                    .child(content),
            )
            .child(self.snackbar.clone())
            .when(self.compose_open, |el| el.child(self.compose_dialog(&view)))
    }
}

fn with_active_window(cx: &mut App, f: impl FnOnce(&mut Window, &mut App) + 'static) {
    // キー入力の処理中はウィンドウを借りているので、終わってから更新する。
    cx.defer(|cx| {
        if let Some(window) = cx.active_window() {
            window.update(cx, |_, window, cx| f(window, cx)).ok();
        }
    });
}

/// `fonts` は起動時に読み込むフォントのデータ（デスクトップではシステムフォントを使うので空）。
/// `reduce_motion` は OS・ブラウザの「視差効果を減らす」相当の設定（gpui は OS の設定を読まないので、起動側が渡す）。
fn run(app: gpui::Application, fonts: Vec<Vec<u8>>, reduce_motion: bool) {
    app.run(move |cx: &mut App| {
        cx.set_reduce_motion(reduce_motion);
        if !fonts.is_empty() {
            let fonts = fonts.into_iter().map(std::borrow::Cow::Owned).collect();
            if let Err(error) = cx.text_system().add_fonts(fonts) {
                eprintln!("failed to load the fonts: {error:#}");
            }
        }
        // ペインや Snackbar が消えてフォーカスの行き先がなくなると、要素の on_action には
        // 届かない。どの状態でも Tab が効くよう、グローバルに受ける。
        cx.on_action(|_: &FocusNext, cx| {
            with_active_window(cx, |window, cx| window.focus_next(cx))
        });
        cx.on_action(|_: &FocusPrev, cx| {
            with_active_window(cx, |window, cx| window.focus_prev(cx))
        });
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
        md3_components::init(cx);
        let bounds = Bounds::centered(None, size(px(1200.), px(900.)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Mail sample".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                theme::follow_window_appearance(window, cx).detach();
                cx.new(|cx| MailView::new(window, cx))
            },
        );
        if let Err(error) = result {
            eprintln!("failed to open mail window: {error:#}");
            exit_with_failure(cx);
            return;
        }
        cx.activate(true);
    });
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
    // デスクトップでは OS の設定を読む手段が gpui に無いため、環境変数で指定する。
    let reduce_motion = std::env::var("MD3_REDUCE_MOTION").is_ok_and(|value| value == "1");
    run(gpui_platform::application(), Vec::new(), reduce_motion);
}

/// wasm 版のエントリポイント。ブラウザでは gpui がシステムフォントを読めないため、ページが
/// UI の文字に使うフォントを取得し、1 つの配列 `fonts` につなげて、それぞれの長さ `lengths` と
/// 一緒に渡す。`webgl` が真なら WebGPU を使わず WebGL2 で描く（ページの `?renderer=webgl`）。
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
        split,
        reduce_motion,
    );
}
