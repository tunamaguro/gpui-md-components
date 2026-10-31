//! 選択・状態表示の部品のページ。

use md3_components::selection::CheckboxState;

use gpui::{App, IntoElement, WeakEntity, Window, div, prelude::*, px};
use md3_components::selection::{
    Badge, Checkbox, Chip, ChipKind, CircularProgress, LinearProgress, RadioGroup, Slider, Switch,
};
use md3_components::theme::TypeScale;
use md3_components::{Icon, IconName, Text};

use super::{Section, state_header, state_row};
use crate::{Demo, Gallery};

/// 有効な見本と無効な見本を 1 行に並べる。
fn states<E: IntoElement>(
    id: String,
    label: &'static str,
    make: impl Fn(String) -> E,
    disable: impl Fn(E) -> E,
) -> impl IntoElement {
    let cells = vec![
        make(format!("{id}-Enabled")).into_any_element(),
        disable(make(format!("{id}-disabled"))).into_any_element(),
    ];
    state_row(label, cells)
}

/// 選択・状態表示の部品のページ。
pub fn page(
    demo: &Demo,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let checkboxes = [
        ("Unchecked", CheckboxState::Unchecked, false),
        ("Checked", CheckboxState::Checked, false),
        ("Indeterminate", CheckboxState::Indeterminate, false),
        ("Error", CheckboxState::Checked, true),
    ]
    .map(|(name, state, error)| {
        states(
            format!("checkbox-{name}"),
            name,
            move |id| Checkbox::new(id, state).aria_label(name).error(error),
            |c| c.disabled(true),
        )
    });
    let switches = [
        ("Off", false, false),
        ("On", true, false),
        ("Icons", true, true),
    ]
    .map(|(name, on, icons)| {
        states(
            format!("switch-{name}"),
            name,
            move |id| Switch::new(id, on).aria_label(name).icons(icons),
            |s| s.disabled(true),
        )
    });
    let chips = [
        ("Assist", ChipKind::Assist, false, false),
        ("Assist elevated", ChipKind::Assist, false, true),
        ("Filter", ChipKind::Filter, false, false),
        ("Filter selected", ChipKind::Filter, true, false),
        ("Input", ChipKind::Input, false, false),
        ("Suggestion", ChipKind::Suggestion, false, false),
    ]
    .map(|(name, kind, selected, elevated)| {
        states(
            format!("chip-{name}"),
            name,
            move |id| {
                let chip = Chip::new(id, kind, "Label")
                    .selected(selected)
                    .elevated(elevated)
                    .remove_label("Label を削除");
                if kind == ChipKind::Assist {
                    chip.icon(IconName::Star)
                } else {
                    chip
                }
            },
            |c| c.disabled(true),
        )
    });

    let parent = demo.parent_check();
    let checks = demo.checks;
    let filters = demo.filters;
    let tags = demo.tags.clone();
    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(
            Section::new("Checkbox")
                .child(state_header())
                .children(checkboxes)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(labeled(
                            Checkbox::new("check-all", parent)
                                .aria_label("すべて")
                                .on_change({
                                    let view = view.clone();
                                    move |next, _, cx| {
                                        let on = next == CheckboxState::Checked;
                                        Gallery::update_demo(&view, cx, |demo| {
                                            demo.checks = [on, on]
                                        })
                                    }
                                }),
                            "すべて",
                        ))
                        .children(["メール", "プッシュ通知"].into_iter().enumerate().map(
                            |(i, name)| {
                                let view = view.clone();
                                div().pl(px(32.0)).child(labeled(
                                    Checkbox::new(format!("check-{i}"), checked(checks[i]))
                                        .aria_label(name)
                                        .on_change(move |next, _, cx| {
                                            Gallery::update_demo(&view, cx, |demo| {
                                                demo.checks[i] = next == CheckboxState::Checked
                                            })
                                        }),
                                    name,
                                ))
                            },
                        )),
                ),
        )
        .child(
            Section::new("Radio button").child(
                RadioGroup::new("radio")
                    .aria_label("配送方法")
                    .option("通常")
                    .option("お急ぎ")
                    .disabled_option("当日（選べません）")
                    .option("店頭受け取り")
                    .selected(demo.radio)
                    .on_change({
                        let view = view.clone();
                        move |index, _, cx| {
                            Gallery::update_demo(&view, cx, |demo| demo.radio = Some(index))
                        }
                    }),
            ),
        )
        .child(
            Section::new("Switch")
                .child(state_header())
                .children(switches)
                .child(labeled(
                    Switch::new("switch-demo", demo.switch)
                        .aria_label("Wi-Fi")
                        .on_change({
                            let view = view.clone();
                            move |next, _, cx| {
                                Gallery::update_demo(&view, cx, |demo| demo.switch = next)
                            }
                        }),
                    "Wi-Fi",
                )),
        )
        .child(
            Section::new("Chip")
                .child(state_header())
                .children(chips)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .children(["未読", "添付あり", "重要"].into_iter().enumerate().map(
                            |(i, name)| {
                                let view = view.clone();
                                Chip::new(format!("filter-{i}"), ChipKind::Filter, name)
                                    .selected(filters[i])
                                    .on_click(move |_, _, cx| {
                                        Gallery::update_demo(&view, cx, |demo| {
                                            demo.filters[i] = !demo.filters[i]
                                        })
                                    })
                            },
                        ))
                        .children(tags.into_iter().map(|tag| {
                            let view = view.clone();
                            Chip::new(format!("tag-{tag}"), ChipKind::Input, tag)
                                .remove_label(format!("{tag} を削除"))
                                .on_remove(move |_, cx| {
                                    Gallery::update_demo(&view, cx, |demo| {
                                        demo.tags.retain(|t| *t != tag)
                                    })
                                })
                        })),
                ),
        )
        .child(
            Section::new("Slider")
                .child(slider_row(
                    "連続",
                    Slider::new("volume", demo.volume)
                        .aria_label("音量")
                        .on_change({
                            let view = view.clone();
                            move |value, _, cx| {
                                Gallery::update_demo(&view, cx, |demo| demo.volume = value)
                            }
                        }),
                    format!("{:.2}", demo.volume),
                ))
                .child(slider_row(
                    "離散",
                    Slider::new("rating", demo.rating)
                        .aria_label("評価")
                        .range(0.0, 5.0)
                        .step(1.0)
                        .on_change(move |value, _, cx| {
                            Gallery::update_demo(&view, cx, |demo| demo.rating = value)
                        }),
                    format!("{}", demo.rating),
                ))
                .child(slider_row(
                    "Disabled",
                    Slider::new("slider-disabled", 0.3)
                        .aria_label("無効")
                        .disabled(true),
                    String::new(),
                )),
        )
        .child(
            Section::new("Badge").child(
                div()
                    .flex()
                    .gap_6()
                    .child(badged(Badge::dot()))
                    .child(badged(Badge::new("3")))
                    .child(badged(Badge::new("999+"))),
            ),
        )
        .child(
            Section::new("Progress")
                .child(
                    div()
                        .w(px(320.0))
                        .child(LinearProgress::new("linear", Some(0.4)).aria_label("ダウンロード")),
                )
                .child(div().w(px(320.0)).child(
                    LinearProgress::new("linear-indeterminate", None).aria_label("読み込み中"),
                ))
                .child(
                    div()
                        .flex()
                        .gap_6()
                        .child(
                            CircularProgress::new("circular", Some(0.7)).aria_label("アップロード"),
                        )
                        .child(
                            CircularProgress::new("circular-indeterminate", None)
                                .aria_label("読み込み中"),
                        ),
                ),
        )
}

fn checked(on: bool) -> CheckboxState {
    if on {
        CheckboxState::Checked
    } else {
        CheckboxState::Unchecked
    }
}

/// 部品の右に見えるラベルを置く。
fn labeled(control: impl IntoElement, label: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(control)
        .child(Text::new(TypeScale::BodyLarge, label))
}

fn slider_row(label: &'static str, slider: Slider, value: String) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_4()
        .child(
            div()
                .w(px(120.0))
                .child(Text::new(TypeScale::LabelLarge, label)),
        )
        .child(div().w(px(320.0)).child(slider))
        .child(Text::new(TypeScale::BodyMedium, value))
}

/// アイコンの右上にバッジを重ねる。
fn badged(badge: Badge) -> impl IntoElement {
    div()
        .relative()
        .size(px(24.0))
        .child(Icon::new(IconName::Home))
        .child(div().absolute().top(px(-4.0)).left(px(14.0)).child(badge))
}
