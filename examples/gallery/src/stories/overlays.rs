//! オーバーレイの部品のページ。

use gpui::{AnyElement, App, Entity, IntoElement, WeakEntity, Window, div, prelude::*, px};
use md3_components::buttons::{Button, ButtonVariant, IconButton};
use md3_components::inputs::TextField;
use md3_components::inputs::TextFieldState;
use md3_components::overlays::{
    BottomSheet, Dialog, DialogAction, Menu, MenuItem, PlainTooltip, RichTooltip, SideSheet,
    SideSheetKind, Snackbar, SnackbarDuration, SnackbarHost, SnackbarMessage, SnackbarPolicy,
    SnackbarResult,
};
use md3_components::theme::TypeScale;
use md3_components::theme::md3_color::ColorRole;
use md3_components::{IconName, Text};

use super::Section;
use crate::{Demo, Gallery};

/// 見本の小見出しと中身を縦に並べる。
fn sample(label: &str, element: impl IntoElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            Text::new(TypeScale::LabelMedium, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        )
        .child(element)
        .into_any_element()
}

/// メニューの項目（見本と実物で同じものを使う）。
fn menu_items(view: &WeakEntity<Gallery>) -> Vec<MenuItem> {
    let pick = |label: &'static str| {
        let view = view.clone();
        move |_: &mut Window, cx: &mut App| {
            Gallery::update_demo(&view, cx, |demo| demo.last_action = Some(label))
        }
    };
    vec![
        MenuItem::new("切り取り")
            .leading_icon(IconName::ContentCopy)
            .trailing_text("Ctrl+X")
            .on_select(pick("切り取り")),
        MenuItem::new("コピー")
            .leading_icon(IconName::ContentCopy)
            .trailing_text("Ctrl+C")
            .on_select(pick("コピー")),
        MenuItem::new("貼り付け（無効）")
            .leading_icon(IconName::Edit)
            .disabled(true),
        MenuItem::new("共有")
            .leading_icon(IconName::Share)
            .on_select(pick("共有")),
        MenuItem::new("削除")
            .leading_icon(IconName::Delete)
            .on_select(pick("削除")),
    ]
}

/// ダイアログの見本（実物と、その場に描く見本で共有）。
/// 見本とモーダルで ID と入力欄の状態を分ける（同じだとフォーカスハンドルを 2 つの要素が共有する）。
fn dialog(id: &str, view: &WeakEntity<Gallery>, note: &TextFieldState) -> Dialog {
    let close = |action: &'static str| {
        let view = view.clone();
        move |_: &mut Window, cx: &mut App| {
            Gallery::update_demo(&view, cx, |demo| {
                demo.dialog_open = false;
                demo.last_action = Some(action);
            })
        }
    };
    Dialog::new(id.to_owned())
        .headline("下書きを破棄しますか？")
        .icon(IconName::Delete)
        .supporting_text("保存していない変更は失われます。理由を書き残せます。")
        .child(
            div()
                .pt_4()
                .child(TextField::new(format!("{id}-note"), note).label("メモ")),
        )
        .action(DialogAction::new("キャンセル", close("キャンセル")))
        .action(DialogAction::new("破棄", close("破棄")))
        .on_dismiss(close("閉じる（Esc / スクリム）"))
}

/// オーバーレイの部品のページ。
pub fn page(
    demo: &Demo,
    note: &TextFieldState,
    sample_note: &TextFieldState,
    snackbar: &Entity<SnackbarHost>,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let set = |f: fn(&mut Demo)| {
        let view = view.clone();
        move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| Gallery::update_demo(&view, cx, f)
    };
    let last = Text::new(
        TypeScale::BodyMedium,
        format!("最後の操作: {}", demo.last_action.unwrap_or("なし")),
    )
    .color(ColorRole::OnSurfaceVariant);

    // メニュー: 基準のボタンと同じ relative な親に置く。
    let menu_anchor = div()
        .relative()
        .child(
            Button::new("menu-open", "メニューを開く")
                .variant(ButtonVariant::Outlined)
                .icon(IconName::MoreVert)
                .menu_expanded(demo.menu_open)
                .on_click(set(|demo| demo.menu_open = !demo.menu_open)),
        )
        .when(demo.menu_open, |this| {
            let view = view.clone();
            this.child(
                Menu::new("menu")
                    .aria_label("編集")
                    .items(menu_items(&view))
                    .on_dismiss(move |_, cx| {
                        Gallery::update_demo(&view, cx, |demo| demo.menu_open = false)
                    }),
            )
        });
    let icon_menu_anchor = div()
        .relative()
        .child(
            IconButton::new("icon-menu-open", IconName::MoreVert)
                .aria_label("編集メニュー")
                .menu_expanded(demo.icon_menu_open)
                .on_click(set(|demo| demo.icon_menu_open = !demo.icon_menu_open)),
        )
        .when(demo.icon_menu_open, |this| {
            let view = view.clone();
            this.child(
                Menu::new("icon-menu")
                    .aria_label("編集")
                    .items(menu_items(&view))
                    .on_dismiss(move |_, cx| {
                        Gallery::update_demo(&view, cx, |demo| demo.icon_menu_open = false)
                    }),
            )
        });
    let menu_states = sample(
        "Enabled / Disabled",
        div().w(px(240.0)).child(
            Menu::new("menu-sample-Enabled / Disabled")
                .aria_label("編集")
                .items(menu_items(&view))
                .inline(),
        ),
    );
    let menus = Section::new("Menu（↑↓ で移動、Enter で選択、Esc / Tab で閉じる）")
        .heading_id("overlays-menu-heading")
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(menu_anchor)
                .child(icon_menu_anchor)
                .child(last),
        )
        .child(menu_states);

    let dialogs = Section::new("Dialog（中で Tab が回り、Esc で閉じ、閉じると開いたボタンに戻る）")
        .child(
            div().flex().gap_4().child(
                Button::new("dialog-open", "ダイアログを開く")
                    .on_click(set(|demo| demo.dialog_open = true)),
            ),
        )
        .child(sample(
            "見本（その場に描画）",
            dialog("dialog-sample", &view, sample_note).inline(),
        ))
        .when(demo.dialog_open, |this| {
            this.child(dialog("dialog", &view, note))
        });

    let tooltips = Section::new("Tooltip（ポインタを重ねる、Tab でフォーカスする）")
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    IconButton::new("tooltip-favorite", IconName::Favorite)
                        .aria_label("お気に入り")
                        .tooltip("お気に入りに追加"),
                )
                .child(
                    IconButton::new("tooltip-share", IconName::Share)
                        .aria_label("共有")
                        .tooltip("リンクをコピーして共有"),
                )
                .child(
                    div()
                        .id("tooltip-rich")
                        .child(
                            Button::new("tooltip-rich-button", "Rich tooltip")
                                .variant(ButtonVariant::Text),
                        )
                        .hoverable_tooltip(
                            RichTooltip::new("保存した項目はすべての端末で同期されます。")
                                .subhead("同期")
                                .action("詳しく", |_, _| {})
                                .builder(),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_start()
                .gap_6()
                .child(sample(
                    "Plain",
                    gpui::AnyView::from(cx.new(|_| PlainTooltip::new("お気に入りに追加"))),
                ))
                .child(sample(
                    "Rich",
                    gpui::AnyView::from(cx.new(|_| {
                        RichTooltip::new("保存した項目はすべての端末で同期されます。")
                            .subhead("同期")
                            .action("詳しく", |_, _| {})
                    })),
                )),
        );

    let show = |message: fn() -> SnackbarMessage| {
        let snackbar = snackbar.clone();
        move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
            snackbar.update(cx, |host, cx| {
                host.show(message(), cx);
            })
        }
    };
    let policy_host = snackbar.clone();
    let queue = snackbar.read(cx).current_policy() == SnackbarPolicy::Queue;
    let snackbars = Section::new("Snackbar（ウィンドウの下に出る）")
        .child(snackbar_policy_toggle(policy_host, queue))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_4()
                .child(
                    Button::new("snackbar-short", "短い知らせ")
                        .variant(ButtonVariant::FilledTonal)
                        .on_click(show(|| SnackbarMessage::new("写真を保存しました"))),
                )
                .child(
                    Button::new("snackbar-action", "操作付き")
                        .variant(ButtonVariant::FilledTonal)
                        .on_click(show(|| {
                            SnackbarMessage::new("メールを削除しました")
                                .action("元に戻す", |_, _| {})
                                .dismissible(true)
                        })),
                )
                .child(
                    Button::new("snackbar-long", "長い知らせ")
                        .variant(ButtonVariant::FilledTonal)
                        .on_click(show(|| {
                            SnackbarMessage::new(
                                "接続が切れました。オンラインに戻ると、変更は自動で送信されます。",
                            )
                            .duration(SnackbarDuration::Long)
                        })),
                )
                .child(snackbar_follow_up(snackbar.clone())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .w(px(600.0))
                .child(Snackbar::new("snackbar-sample-1", "静的見本：保存完了"))
                .child(
                    Snackbar::new("snackbar-sample-2", "静的見本：削除を取り消せます")
                        .action("元に戻す", |_, _| {})
                        .dismiss_action("閉じる", |_, _| {}),
                )
                .child(Snackbar::new(
                    "snackbar-sample-3",
                    "静的見本：オフラインで編集した内容は、次回接続したときに同期されます。変更は端末に保存されています。",
                )),
        );

    let share_items = |prefix: &str| {
        div().flex().flex_col().px_4().pb_6().gap_2().children(
            [
                ("mail", "メールで送る", IconName::Mail),
                ("copy", "リンクをコピー", IconName::ContentCopy),
            ]
            .map(|(id, label, icon)| {
                Button::new(format!("{prefix}-{id}"), label)
                    .variant(ButtonVariant::Text)
                    .icon(icon)
            }),
        )
    };
    let close_sheet = |f: fn(&mut Demo)| {
        let view = view.clone();
        move |_: &mut Window, cx: &mut App| Gallery::update_demo(&view, cx, f)
    };
    let sheets = Section::new("Sheet")
        .child(
            div()
                .flex()
                .gap_4()
                .child(
                    Button::new("bottom-sheet-open", "ボトムシート")
                        .variant(ButtonVariant::Outlined)
                        .on_click(set(|demo| demo.bottom_sheet_open = true)),
                )
                .child(
                    Button::new("side-sheet-toggle", "サイドシート（Standard）")
                        .variant(ButtonVariant::Outlined)
                        .on_click(set(|demo| demo.side_sheet_open = !demo.side_sheet_open)),
                )
                .child(
                    Button::new("modal-side-sheet-open", "サイドシート（Modal）")
                        .variant(ButtonVariant::Outlined)
                        .on_click(set(|demo| demo.modal_side_sheet_open = true)),
                ),
        )
        .child(
            div()
                .flex()
                .items_start()
                .gap_6()
                .child(sample(
                    "Bottom sheet（見本）",
                    div().w(px(360.0)).child(
                        BottomSheet::new("bottom-sheet-sample")
                            .aria_label("共有")
                            .handle_label("シートを閉じる")
                            .inline()
                            .child(share_items("bottom-sample")),
                    ),
                ))
                .child(sample(
                    "Side sheet Modal（見本）",
                    div().h(px(240.0)).child(
                        SideSheet::new("side-sheet-sample")
                            .headline("フィルタ")
                            .kind(SideSheetKind::Modal)
                            .close_button("閉じる")
                            .inline()
                            .child(Text::new(TypeScale::BodyMedium, "条件を選ぶ")),
                    ),
                )),
        )
        .when(demo.bottom_sheet_open, |this| {
            this.child(
                BottomSheet::new("bottom-sheet")
                    .aria_label("共有")
                    .handle_label("シートを閉じる")
                    .on_dismiss(close_sheet(|demo| demo.bottom_sheet_open = false))
                    .child(share_items("bottom")),
            )
        })
        .when(demo.modal_side_sheet_open, |this| {
            this.child(
                SideSheet::new("modal-side-sheet")
                    .headline("フィルタ")
                    .kind(SideSheetKind::Modal)
                    .close_button("閉じる")
                    .on_dismiss(close_sheet(|demo| demo.modal_side_sheet_open = false))
                    .child(Text::new(TypeScale::BodyMedium, "条件を選ぶ")),
            )
        });

    let content = div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_8()
        .child(menus)
        .child(dialogs)
        .child(tooltips)
        .child(snackbars)
        .child(sheets);
    div()
        .flex()
        .items_start()
        .gap_6()
        .child(content)
        .when(demo.side_sheet_open, |this| {
            this.child(
                div().h(px(480.0)).child(
                    SideSheet::new("side-sheet")
                        .headline("詳細")
                        .close_button("閉じる")
                        .on_dismiss(close_sheet(|demo| demo.side_sheet_open = false))
                        .child(Text::new(
                            TypeScale::BodyMedium,
                            "画面の一部として並ぶシート",
                        )),
                ),
            )
        })
}

/// 結果通知から同じホストに次の知らせを追加する見本。
fn snackbar_follow_up(host: Entity<SnackbarHost>) -> impl IntoElement {
    Button::new("snackbar-follow-up", "結果から次の知らせ")
        .variant(ButtonVariant::FilledTonal)
        .on_click(move |_, _, cx| {
            let follow_up = host.downgrade();
            host.update(cx, |host, cx| {
                host.show(
                    SnackbarMessage::new("確認すると、結果から次の知らせを表示します")
                        .action("確認", |_, _| {})
                        .on_result(move |result, cx| {
                            if result == SnackbarResult::ActionPerformed {
                                follow_up
                                    .update(cx, |host, cx| {
                                        host.show(
                                            SnackbarMessage::new("確認の結果を受け取りました"),
                                            cx,
                                        );
                                    })
                                    .ok();
                            }
                        }),
                    cx,
                );
            });
        })
}

fn snackbar_policy_toggle(host: Entity<SnackbarHost>, queue: bool) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            md3_components::selection::Switch::new("snackbar-policy", queue)
                .aria_label("順番を待つ（Queue）")
                .on_change(move |queue, _, cx| {
                    host.update(cx, |host, cx| {
                        host.set_policy(
                            if queue {
                                SnackbarPolicy::Queue
                            } else {
                                SnackbarPolicy::Replace
                            },
                            cx,
                        );
                    })
                }),
        )
        .child(if queue {
            "Queue：現在の知らせが閉じるまで順番待ち"
        } else {
            "Replace（既定）：最新の知らせに置換"
        })
}
