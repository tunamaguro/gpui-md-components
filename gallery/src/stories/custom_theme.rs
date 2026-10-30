//! headless の振る舞いだけを使う独自テーマの見本。

use std::{rc::Rc, time::Duration};

use gpui::{
    App, Context, Div, Entity, FontWeight, IntoElement, ParentElement, Render, Stateful, Styled,
    Window, div, prelude::*, px, rgb,
};
use gpui_headless::overlay::{
    MODAL_PRIORITY, SNACKBAR_PRIORITY, dismiss_and_restore, modal_container, scrim, window_layer,
};
use gpui_headless::{
    Button, Checkbox, CheckboxState, DescribedTooltip, Dialog, List, ListItem, Menu, MenuItem,
    MenuKind, RadioGroup, Select, Slider, Snackbar, SnackbarEntry, SnackbarOwner, SnackbarQueue,
    Switch, Tabs, TextField, TextFieldState, TextPaint, Tooltip,
};

// この見本だけの値。既存テーマのトークンではない。
struct Look;
impl Look {
    const INK: u32 = 0x17212b;
    const PAPER: u32 = 0xffffff;
    const ACCENT: u32 = 0x005a70;
    const HOVER: u32 = 0xd9eff2;
    const DISABLED: u32 = 0x69777a;
    const SCRIM: u32 = 0x17212b;
    const HEIGHT: f32 = 44.0;
    const WIDTH: f32 = 240.0;
    const GAP: f32 = 12.0;
    const PAGE_PADDING: f32 = 20.0;
    const TRACK_HEIGHT: f32 = 4.0;
    const BORDER: f32 = 1.0;
    const FONT_SIZE: f32 = 16.0;
    const LINE_HEIGHT: f32 = 22.0;
    const FONT: &str = ".SystemUIFont";
}

fn frame(base: Stateful<Div>, focus: bool, disabled: bool) -> Stateful<Div> {
    frame_with_active(base, focus, disabled, true)
}

fn frame_with_active(
    base: Stateful<Div>,
    focus: bool,
    disabled: bool,
    button_active: bool,
) -> Stateful<Div> {
    base.min_h(px(Look::HEIGHT))
        .px(px(Look::GAP))
        .flex()
        .items_center()
        .border_1()
        .border_color(rgb(if focus { Look::ACCENT } else { Look::INK }))
        .bg(rgb(Look::PAPER))
        .text_color(rgb(if disabled { Look::DISABLED } else { Look::INK }))
        .when(focus, |this| {
            this.bg(rgb(Look::HOVER)).text_color(rgb(Look::ACCENT))
        })
        .when(!disabled, |this| {
            this.hover(|s| s.bg(rgb(Look::HOVER))).active(|s| {
                if button_active {
                    s.bg(rgb(Look::ACCENT)).text_color(rgb(Look::PAPER))
                } else {
                    s.bg(rgb(Look::PAPER)).text_color(rgb(Look::INK))
                }
            })
        })
}

/// 見本の状態。
pub struct Custom {
    checked: bool,
    switched: bool,
    radio: usize,
    tab: usize,
    value: f32,
    selected: usize,
    list: usize,
    clicks: u32,
    dialog: bool,
    field: Entity<TextFieldState>,
    disabled_field: Entity<TextFieldState>,
    queue: SnackbarQueue,
}
impl Custom {
    /// 見本を作る。
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            checked: false,
            switched: false,
            radio: 0,
            tab: 0,
            value: 0.4,
            selected: 0,
            list: 0,
            clicks: 0,
            dialog: false,
            field: cx.new(TextFieldState::new),
            disabled_field: cx.new(TextFieldState::new),
            queue: SnackbarQueue::new(),
        }
    }
}
impl SnackbarOwner for Custom {
    fn snackbar_queue(&mut self) -> &mut SnackbarQueue {
        &mut self.queue
    }
}

struct Hint;
impl Render for Hint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Tooltip::new("custom-hint", "説明を表示します")
            .build()
            .font_family(Look::FONT)
            .text_size(px(Look::FONT_SIZE))
            .line_height(px(Look::LINE_HEIGHT))
            .font_weight(FontWeight::NORMAL)
            .p(px(Look::GAP))
            .bg(rgb(Look::PAPER))
            .border_1()
            .border_color(rgb(Look::INK))
            .text_color(rgb(Look::INK))
            .child("説明を表示します")
    }
}

impl Render for Custom {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let mut page = div()
            .flex()
            .flex_col()
            .gap(px(Look::GAP))
            .font_family(Look::FONT)
            .text_size(px(Look::FONT_SIZE))
            .line_height(px(Look::LINE_HEIGHT))
            .font_weight(FontWeight::NORMAL)
            .p(px(Look::PAGE_PADDING))
            .bg(rgb(Look::PAPER))
            .text_color(rgb(Look::INK))
            .child("独自テーマ — 四角い枠と単色のアクセント");
        let (base, state) = Button::new("custom-button", "数える")
            .on_click({
                let view = view.clone();
                move |_, _, cx| {
                    view.update(cx, |v, cx| {
                        v.clicks += 1;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx);
        let (disabled, ds) = Button::new("custom-button-disabled", "無効")
            .disabled(true)
            .build(window, cx);
        page = page.child(
            div()
                .flex()
                .gap(px(Look::GAP))
                .child(
                    frame(base, state.focus_visible, state.disabled)
                        .child(format!("数える: {}", self.clicks)),
                )
                .child(frame(disabled, ds.focus_visible, ds.disabled).child("無効")),
        );
        for disabled in [false, true] {
            let (base, state) = Checkbox::new(
                if disabled { "check-off" } else { "check-on" },
                if self.checked {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                },
                "チェック",
            )
            .disabled(disabled)
            .on_change({
                let view = view.clone();
                move |value, _, cx| {
                    view.update(cx, |v, cx| {
                        v.checked = value == CheckboxState::Checked;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx);
            let (switch, ss) = Switch::new(
                if disabled { "switch-off" } else { "switch-on" },
                self.switched,
                "スイッチ",
            )
            .disabled(disabled)
            .on_change({
                let view = view.clone();
                move |value, _, cx| {
                    view.update(cx, |v, cx| {
                        v.switched = value;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx);
            page = page.child(
                div()
                    .flex()
                    .gap(px(Look::GAP))
                    .child(frame(base, state.focus_visible, disabled).child(format!(
                        "[{}] チェック{}",
                        if self.checked { "✓" } else { " " },
                        if disabled { " (無効)" } else { "" }
                    )))
                    .child(
                        frame(switch, ss.control.focus_visible, disabled).child(format!(
                            "スイッチ: {}{}",
                            if self.switched { "ON" } else { "OFF" },
                            if disabled { " (無効)" } else { "" }
                        )),
                    ),
            );
        }
        let radio = RadioGroup::new("custom-radio", "選択肢")
            .option("赤")
            .option("青")
            .disabled_option("無効")
            .selected(Some(self.radio))
            .on_change({
                let view = view.clone();
                move |i, _, cx| {
                    view.update(cx, |v, cx| {
                        v.radio = i;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx, |item, _, _| {
                frame(item.base, item.state.focus_visible, item.state.disabled)
                    .child(format!(
                        "{} {}",
                        if item.selected { "◆" } else { "◇" },
                        item.label
                    ))
                    .into_any_element()
            });
        page = page.child(radio.flex().gap(px(Look::GAP)));
        let tabs = Tabs::new("custom-tabs", "タブ", self.tab)
            .tab("概要", false)
            .tab("詳細", false)
            .tab("無効", true)
            .on_change({
                let view = view.clone();
                move |i, _, cx| {
                    view.update(cx, |v, cx| {
                        v.tab = i;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx, |item, _, _| {
                frame(item.base, item.state.focus_visible, item.state.disabled)
                    .child(format!(
                        "{}{}",
                        if item.selected { "■ " } else { "" },
                        ["概要", "詳細", "無効"][item.index]
                    ))
                    .into_any_element()
            });
        page = page.child(tabs.flex().gap(px(Look::GAP)));
        for disabled in [false, true] {
            let (base, state) = Slider::new(
                if disabled { "slider-off" } else { "slider-on" },
                self.value,
                "音量",
            )
            .disabled(disabled)
            .on_change({
                let view = view.clone();
                move |value, _, cx| {
                    view.update(cx, |v, cx| {
                        v.value = value;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx);
            page = page.child(
                frame(
                    div()
                        .id(if disabled {
                            "slider-frame-off"
                        } else {
                            "slider-frame-on"
                        })
                        .w(px(Look::WIDTH)),
                    state.control.focus_visible,
                    disabled,
                )
                .child(
                    base.flex()
                        .flex_col()
                        .w_full()
                        .child(format!(
                            "音量: {:.0}%{}",
                            state.fraction * 100.,
                            if disabled { " (無効)" } else { "" }
                        ))
                        .child(
                            div()
                                .h(px(Look::TRACK_HEIGHT))
                                .w_full()
                                .bg(rgb(Look::HOVER))
                                .child(
                                    div()
                                        .h_full()
                                        .w(px((Look::WIDTH - 2. * (Look::GAP + Look::BORDER))
                                            * state.fraction))
                                        .bg(rgb(if disabled {
                                            Look::DISABLED
                                        } else {
                                            Look::ACCENT
                                        })),
                                ),
                        ),
                ),
            );
        }
        for disabled in [false, true] {
            let (base, state) = TextField::new(
                if disabled { "field-off" } else { "field-on" },
                if disabled {
                    &self.disabled_field
                } else {
                    &self.field
                },
                "名前",
            )
            .placeholder("名前を入力")
            .disabled(disabled)
            .build(window, cx);
            let paint = TextPaint {
                text: rgb(Look::INK).into(),
                placeholder: rgb(Look::DISABLED).into(),
                caret: rgb(Look::ACCENT).into(),
                selection: rgb(Look::HOVER).into(),
                caret_width: px(Look::BORDER),
            };
            page = page.child(
                frame_with_active(
                    base.w(px(Look::WIDTH)),
                    state.focus_visible,
                    disabled,
                    false,
                )
                .child(state.input(paint, Some("名前を入力".into()))),
            );
        }
        let labels = ["春", "夏", "秋", "無効"];
        for disabled in [false, true] {
            let (base, state) =
                Select::new(if disabled { "select-off" } else { "select-on" }, "季節")
                    .value(Some(labels[self.selected].into()))
                    .disabled(disabled)
                    .build(window, cx);
            let mut base = frame(
                base.w(px(Look::WIDTH)).relative(),
                state.focus_visible,
                disabled,
            )
            .child(format!(
                "季節: {} ▾{}",
                labels[self.selected],
                if disabled { " (無効)" } else { "" }
            ));
            if state.open {
                let dismiss = state.dismiss_handler();
                let mut menu = Menu::new("custom-menu", "季節")
                    .kind(MenuKind::Options)
                    .selected(Some(self.selected))
                    .on_dismiss({
                        let dismiss = dismiss.clone();
                        move |window, cx| dismiss(window, cx)
                    });
                for (i, label) in labels.iter().enumerate() {
                    let view = view.clone();
                    menu = menu.item(MenuItem::new(*label).disabled(i == 3).on_select(
                        move |_, cx| {
                            view.update(cx, |v, cx| {
                                v.selected = i;
                                cx.notify();
                            })
                            .ok();
                        },
                    ));
                }
                let popup = menu.build(window, cx, |row, _, _| {
                    frame(row.base, row.state.focus_visible, row.state.disabled)
                        .child(format!(
                            "{}{}",
                            if row.selected { "■ " } else { "" },
                            labels[row.index]
                        ))
                        .into_any_element()
                });
                base = base.child(popup.place(|base| {
                    base.flex()
                        .flex_col()
                        .bg(rgb(Look::PAPER))
                        .border_1()
                        .border_color(rgb(Look::INK))
                }));
            }
            page = page.child(base);
        }
        let mut list = List::new("custom-list", "項目");
        for (i, label) in ["項目 A", "項目 B", "無効"].iter().enumerate() {
            let view = view.clone();
            list = list.item(
                *label,
                ListItem::new(*label)
                    .selected(self.list == i)
                    .disabled(i == 2)
                    .on_select(move |_, cx| {
                        view.update(cx, |v, cx| {
                            v.list = i;
                            cx.notify();
                        })
                        .ok();
                    }),
            );
        }
        page = page.child(
            list.build(window, cx, |row, _, _| {
                frame(row.base, row.state.focus_visible, row.state.disabled)
                    .child(format!(
                        "{}{}",
                        if row.selected { "■ " } else { "" },
                        ["項目 A", "項目 B", "無効"][row.index]
                    ))
                    .into_any_element()
            })
            .finish(|base, _| base.w(px(Look::WIDTH))),
        );
        let (base, state) = Button::new("custom-dialog-open", "ダイアログを開く")
            .on_click({
                let view = view.clone();
                move |_, _, cx| {
                    view.update(cx, |v, cx| {
                        v.dialog = true;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .build(window, cx);
        let (note, ns) = Button::new("custom-note", "通知を表示")
            .on_click({
                let view = view.clone();
                move |_, _, cx| {
                    view.update(cx, |v, cx| {
                        v.queue.show(
                            SnackbarEntry::new("保存しました")
                                .duration(Some(Duration::from_secs(5)))
                                .dismissible(true),
                            cx,
                        );
                    })
                    .ok();
                }
            })
            .build(window, cx);
        let (hint, hs) = Button::new("custom-tooltip", "説明").build(window, cx);
        page = page.child(
            div()
                .flex()
                .gap(px(Look::GAP))
                .child(frame(base, state.focus_visible, false).child("ダイアログを開く"))
                .child(frame(note, ns.focus_visible, false).child("通知を表示"))
                .child(
                    frame(hint, hs.focus_visible, false)
                        .described_tooltip("説明を表示します", |_, cx| {
                            cx.new(|_| Hint).into()
                        })
                        .child("説明"),
                ),
        );
        if self.dialog {
            let (base, focus) = Dialog::new("custom-dialog", "確認").build(window, cx);
            if let Some(focus) = focus {
                let close = Rc::new({
                    let view = view.clone();
                    move |_: &mut Window, cx: &mut App| {
                        view.update(cx, |v, cx| {
                            v.dialog = false;
                            cx.notify();
                        })
                        .ok();
                    }
                });
                let dismiss = dismiss_and_restore(Some(close), &focus);
                let (button, state) = Button::new("custom-dialog-close", "閉じる")
                    .on_click({
                        let dismiss = dismiss.clone();
                        move |_, window, cx| {
                            if let Some(dismiss) = &dismiss {
                                dismiss(window, cx);
                            }
                        }
                    })
                    .build(window, cx);
                let body = modal_container(base, &focus, dismiss.clone(), cx)
                    .w(px(Look::WIDTH))
                    .p(px(Look::GAP))
                    .flex()
                    .flex_col()
                    .gap(px(Look::GAP))
                    .bg(rgb(Look::PAPER))
                    .border_1()
                    .border_color(rgb(Look::INK))
                    .child("確認してください")
                    .child(frame(button, state.focus_visible, false).child("閉じる"));
                page = page.child(window_layer(
                    scrim(
                        "custom-scrim".into(),
                        gpui::Hsla::from(rgb(Look::SCRIM)).opacity(0.6),
                        dismiss,
                    )
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(body),
                    MODAL_PRIORITY,
                    window,
                ));
            }
        }
        self.queue.prepare(cx);
        if let Some(front) = self.queue.front(view.clone()) {
            if front.exiting {
                let id = front.id;
                let view = view.clone();
                cx.defer(move |cx| SnackbarQueue::finish_exit(&view, id, cx));
            } else {
                let mut bar = Snackbar::new("custom-snackbar", front.message.clone())
                    .build()
                    .p(px(Look::GAP))
                    .flex()
                    .gap(px(Look::GAP))
                    .items_center()
                    .bg(rgb(Look::PAPER))
                    .border_1()
                    .border_color(rgb(Look::INK))
                    .child(front.message);
                if let Some(dismiss) = front.dismiss {
                    let (base, state) = Button::new("custom-dismiss", "閉じる")
                        .on_click(move |_, w, c| dismiss(w, c))
                        .build(window, cx);
                    bar = bar.child(frame(base, state.focus_visible, false).child("閉じる"));
                }
                page = page.child(window_layer(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .justify_end()
                        .items_center()
                        .p(px(Look::PAGE_PADDING))
                        .child(bar),
                    SNACKBAR_PRIORITY,
                    window,
                ));
            }
        }
        page
    }
}
