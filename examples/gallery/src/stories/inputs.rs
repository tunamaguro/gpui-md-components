//! 入力の部品のページ。

use md3_components::inputs::TextFieldState;
use std::collections::HashMap;

use gpui::{AnyElement, App, IntoElement, WeakEntity, Window, div, prelude::*, px};
use md3_components::inputs::{Select, TextField, TextFieldVariant};
use md3_components::theme::TypeScale;
use md3_components::theme::md3_color::ColorRole;
use md3_components::{IconName, Text};

use super::Section;
use crate::{Demo, Gallery};

/// テキストフィールドの中身（描画をまたいで残す）。
#[derive(Default)]
pub struct Fields {
    states: HashMap<String, TextFieldState>,
}

impl Fields {
    /// `key` の中身を返す。なければ `initial` を入れて作る。
    pub fn get(&mut self, key: &str, initial: &str, cx: &mut App) -> TextFieldState {
        self.states
            .entry(key.to_owned())
            .or_insert_with(|| {
                let initial = initial.to_owned();
                TextFieldState::with_text(initial, cx)
            })
            .clone()
    }
}

/// 入力できる見本、エラーの見本、無効な見本の列。
const FIELD_STATES: [&str; 3] = ["Enabled", "Error", "Disabled"];

/// 見出し付きの見本 1 つ。
fn labeled(label: &str, element: impl IntoElement) -> AnyElement {
    div()
        .w(px(296.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            Text::new(TypeScale::LabelMedium, label.to_owned()).color(ColorRole::OnSurfaceVariant),
        )
        .child(element)
        .into_any_element()
}

/// 入力の部品のページ。
pub fn page(
    demo: &Demo,
    fields: &mut Fields,
    view: WeakEntity<Gallery>,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let mut section = |variant: TextFieldVariant, name: &str, fields: &mut Fields| {
        let cells: Vec<AnyElement> = FIELD_STATES
            .iter()
            .map(|&state| {
                let key = format!("{name}-{state}");
                let empty = fields.get(&format!("{key}-empty"), "", cx);
                let filled = fields.get(&format!("{key}-filled"), "入力した文字", cx);
                let make = |id: String, entity: &TextFieldState| {
                    let field = TextField::new(id, entity)
                        .label("ラベル")
                        .variant(variant)
                        .placeholder("例")
                        .supporting_text(if state == "Error" {
                            "エラーの説明"
                        } else {
                            "補助テキスト"
                        });
                    match state {
                        "Error" => field.error(true),
                        "Disabled" => field.disabled(true),
                        _ => field,
                    }
                };
                labeled(
                    state,
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(make(format!("{key}-empty"), &empty))
                        .child(make(format!("{key}-filled"), &filled)),
                )
            })
            .collect();
        Section::new(format!("{name} text field"))
            .child(div().flex().flex_wrap().gap_6().children(cells))
    };
    let filled = section(TextFieldVariant::Filled, "Filled", fields);
    let outlined = section(TextFieldVariant::Outlined, "Outlined", fields);

    let multi_line = {
        let long =
            "長い本文は欄の幅で折り返します。\n上限の 4 行を超えた分は欄の中でスクロールします。\n"
                .repeat(3);
        let notes = fields.get("multi-filled", "", cx);
        let outlined_notes = fields.get("multi-outlined", "", cx);
        let scrolled = fields.get("multi-scrolled", &long, cx);
        let error = fields.get("multi-error", "1 行目\n2 行目", cx);
        let disabled = fields.get("multi-disabled", "1 行目\n2 行目", cx);
        Section::new("Multi-line text field").child(
            div()
                .flex()
                .flex_wrap()
                .gap_6()
                .child(labeled(
                    "Filled",
                    TextField::new("multi-filled", &notes)
                        .label("本文")
                        .placeholder("本文を入力")
                        .multi_line(3, 6),
                ))
                .child(labeled(
                    "Outlined",
                    TextField::new("multi-outlined", &outlined_notes)
                        .label("本文")
                        .variant(TextFieldVariant::Outlined)
                        .leading_icon(IconName::Mail)
                        .multi_line(3, 6),
                ))
                .child(labeled(
                    "上限でスクロール",
                    TextField::new("multi-scrolled", &scrolled)
                        .label("本文")
                        .multi_line(1, 4),
                ))
                .child(labeled(
                    "Error",
                    TextField::new("multi-error", &error)
                        .label("本文")
                        .variant(TextFieldVariant::Outlined)
                        .error(true)
                        .supporting_text("エラーの説明")
                        .multi_line(3, 6),
                ))
                .child(labeled(
                    "Disabled",
                    TextField::new("multi-disabled", &disabled)
                        .label("本文")
                        .disabled(true)
                        .multi_line(3, 6),
                )),
        )
    };

    let name = fields.get("name", "", cx);
    let email = fields.get("email", "user@", cx);
    let price = fields.get("price", "1200", cx);
    let bio = fields.get("bio", "", cx);
    let search = fields.get("search", "", cx);
    let email_text = email.text(cx).to_owned();
    let email_invalid = !email_text.is_empty() && !email_text.contains('.');
    let bio_len = bio.text(cx).chars().count();
    let examples = Section::new("使ってみる（Tab で移動、日本語 IME で入力できる）")
        .heading_id("inputs-examples-heading")
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_6()
                .child(labeled(
                    "Filled",
                    TextField::new("name", &name)
                        .label("名前（必須）")
                        .required(true)
                        .supporting_text("氏名を入力してください")
                        .placeholder("山田 太郎"),
                ))
                .child(labeled(
                    "Outlined・入力の検証",
                    TextField::new("email", &email)
                        .label("メールアドレス")
                        .variant(TextFieldVariant::Outlined)
                        .error(email_invalid)
                        .supporting_text(if email_invalid {
                            "ドメインに . がありません"
                        } else {
                            "user@example.com の形式"
                        }),
                ))
                .child(labeled(
                    "接頭辞・接尾辞",
                    TextField::new("price", &price)
                        .label("価格")
                        .variant(TextFieldVariant::Outlined)
                        .prefix("¥")
                        .suffix("円（税込）"),
                ))
                .child(labeled(
                    "文字数カウンタ",
                    TextField::new("bio", &bio)
                        .label("自己紹介")
                        .max_length(40)
                        .error(bio_len > 40)
                        .supporting_text("40 文字まで"),
                ))
                .child(labeled(
                    "アイコン",
                    TextField::new("search", &search)
                        .label("検索")
                        .leading_icon(IconName::Search)
                        .trailing_icon(IconName::Mail),
                )),
        );

    const COLORS: [&str; 5] = ["赤", "緑", "青", "黄", "紫"];
    let select = |id: &str, variant: TextFieldVariant, selected: Option<usize>| {
        let view = view.clone();
        Select::new(id.to_owned(), COLORS, selected)
            .label("色（必須）")
            .required(true)
            .variant(variant)
            .supporting_text("↓ / Enter で開く")
            .on_change(move |index, _, cx| {
                Gallery::update_demo(&view, cx, |demo| demo.color = Some(index))
            })
    };
    let select_states = |variant: TextFieldVariant, name: &str| {
        let cells: Vec<AnyElement> = FIELD_STATES
            .iter()
            .map(|&state| {
                let field = Select::new(format!("select-{name}-{state}"), COLORS, Some(2))
                    .label("色")
                    .variant(variant);
                let field = match state {
                    "Error" => field.error(true).supporting_text("選んでください"),
                    "Disabled" => field.disabled(true),
                    _ => field,
                };
                labeled(state, field)
            })
            .collect();
        div().flex().flex_wrap().gap_6().children(cells)
    };
    let selects = Section::new("Select")
        .heading_id("inputs-select-heading")
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_6()
                .child(labeled(
                    "Filled",
                    select("select-filled", TextFieldVariant::Filled, demo.color),
                ))
                .child(labeled(
                    "Outlined",
                    select("select-outlined", TextFieldVariant::Outlined, demo.color),
                )),
        )
        .child(select_states(TextFieldVariant::Filled, "filled"))
        .child(select_states(TextFieldVariant::Outlined, "outlined"));

    div()
        .flex()
        .flex_col()
        .gap_8()
        .child(examples)
        .child(selects)
        .child(filled)
        .child(outlined)
        .child(multi_line)
}
