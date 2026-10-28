//! `cargo xtask check-tokens`: `docs/TOKENS_SOURCE.md` の表と `md3-tokens` の定数を突き合わせる。
//!
//! - 表の各行の定数が `crates/md3-tokens/src` に存在し、値が一致すること
//! - 値を直接書いた `pub const` がすべて表に載っていること
//!
//! を確認する。値の比較は次のとおり。
//!
//! - 定数の式に数値リテラルがあれば、数値の並びを表の値と比べる（例: `CubicBezier::new(0.2, 0.0, 0.0, 1.0)` と `0.2, 0.0, 0.0, 1.0`）
//! - 数値がなければ、式の最後のパス要素を大文字小文字と `_` を無視して比べる（例: `Typeface::Brand` と `Brand`）
//! - 式に `{` か `[` を含む定数は、他の定数を組み合わせたものとみなして対象外にする
//!
//! モジュールはファイル単位（`foo.rs` / `foo/mod.rs`）で書き、テストはファイル末尾の
//! `#[cfg(test)]` 以降に置く前提で解析する。

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const CRATE_NAME: &str = "md3_tokens";

/// 表の 1 行。
#[derive(Debug, Clone, PartialEq)]
struct Row {
    value: String,
    line: usize,
}

/// ソース中の定数。
#[derive(Debug, Clone, PartialEq)]
struct Const {
    expr: String,
    file: PathBuf,
}

/// 突き合わせの結果。
#[derive(Debug, Default)]
struct Report {
    checked: usize,
    errors: Vec<String>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "- {error}")?;
        }
        Ok(())
    }
}

/// ワークスペースの根から突き合わせを行う。成功なら確認した定数の数を返す。
pub fn run(workspace_root: &Path) -> Result<usize, String> {
    let table_path = workspace_root.join("docs/TOKENS_SOURCE.md");
    let table = fs::read_to_string(&table_path)
        .map_err(|e| format!("{} を読めません: {e}", table_path.display()))?;
    let src_dir = workspace_root.join("crates/md3-tokens/src");
    let mut consts = BTreeMap::new();
    collect_consts(&src_dir, &src_dir, &mut consts)?;

    let mut report = Report::default();
    let rows = parse_table(&table, &mut report.errors);
    compare(&rows, &consts, &mut report);
    if report.errors.is_empty() {
        Ok(report.checked)
    } else {
        Err(format!(
            "{} 件の不一致があります:\n{report}",
            report.errors.len()
        ))
    }
}

/// 表から `md3_tokens::` で始まる行を読む。
fn parse_table(table: &str, errors: &mut Vec<String>) -> BTreeMap<String, Row> {
    let mut rows = BTreeMap::new();
    for (index, line) in table.lines().enumerate() {
        let line_no = index + 1;
        let Some(rest) = line.trim().strip_prefix('|') else {
            continue;
        };
        let cells: Vec<&str> = rest
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let Some(name) = cells.first() else {
            continue;
        };
        if !name.starts_with(&format!("{CRATE_NAME}::")) {
            continue;
        }
        if cells.len() != 5 || cells.iter().any(|c| c.is_empty()) {
            errors.push(format!(
                "TOKENS_SOURCE.md:{line_no}: 列は 5 つ（定数・値・出典ファイル・行・コミット）で、空欄は不可"
            ));
            continue;
        }
        let row = Row {
            value: cells[1].to_string(),
            line: line_no,
        };
        if let Some(previous) = rows.insert(name.to_string(), row) {
            errors.push(format!(
                "TOKENS_SOURCE.md:{line_no}: {name} が重複している（{} 行目にもある）",
                previous.line
            ));
        }
    }
    rows
}

/// `dir` 以下の `.rs` から `pub const` を集める。
fn collect_consts(
    src_root: &Path,
    dir: &Path,
    consts: &mut BTreeMap<String, Const>,
) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{} を読めません: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_consts(src_root, &path, consts)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path)
                .map_err(|e| format!("{} を読めません: {e}", path.display()))?;
            let module = module_path(src_root, &path);
            for (name, expr) in parse_consts(&source) {
                let key = if module.is_empty() {
                    format!("{CRATE_NAME}::{name}")
                } else {
                    format!("{CRATE_NAME}::{module}::{name}")
                };
                consts.insert(
                    key,
                    Const {
                        expr,
                        file: path.clone(),
                    },
                );
            }
        }
    }
    Ok(())
}

/// ファイルのパスからモジュールパス（`foo::bar`）を求める。
fn module_path(src_root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(src_root).unwrap_or(file);
    let mut parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = parts.pop() {
        let stem = last.trim_end_matches(".rs");
        if stem != "lib" && stem != "mod" {
            parts.push(stem.to_string());
        }
    }
    parts.join("::")
}

/// ソースから `pub const NAME: TYPE = EXPR;` を取り出す（`#[cfg(test)]` 以降は除く）。
fn parse_consts(source: &str) -> Vec<(String, String)> {
    let mut code = String::new();
    for line in source.lines() {
        if line.starts_with("#[cfg(test)]") {
            break;
        }
        let line = line.split("//").next().unwrap_or("");
        code.push_str(line);
        code.push('\n');
    }

    let mut consts = Vec::new();
    let mut rest = code.as_str();
    while let Some(start) = rest.find("pub const ") {
        rest = &rest[start + "pub const ".len()..];
        if rest.starts_with("fn ") {
            continue;
        }
        let Some(colon) = rest.find(':') else {
            break;
        };
        let name = rest[..colon].trim().to_string();
        let Some(eq) = rest.find('=') else {
            break;
        };
        let after_eq = &rest[eq + 1..];
        let mut depth = 0i32;
        let mut end = None;
        for (i, c) in after_eq.char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ';' if depth == 0 => {
                    end = Some(i);
                    break;
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            break;
        };
        let expr = after_eq[..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        consts.push((name, expr));
        rest = &after_eq[end..];
    }
    consts
}

/// 文字列中の数値リテラルを順に取り出す。識別子の一部（`LEVEL0` の `0` など）は除く。
fn numbers(text: &str) -> Vec<f64> {
    let chars: Vec<char> = text.chars().collect();
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut result = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        if c.is_ascii_digit() && !prev.is_some_and(|p| is_ident(p) || p == '.') {
            let negative =
                prev == Some('-') && !(i >= 2 && (is_ident(chars[i - 2]) || chars[i - 2] == ')'));
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_')
            {
                i += 1;
            }
            let literal: String = chars[start..i].iter().filter(|&&c| c != '_').collect();
            if let Ok(value) = literal.trim_end_matches('.').parse::<f64>() {
                result.push(if negative { -value } else { value });
            }
        } else {
            i += 1;
        }
    }
    result
}

fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

/// 定数の式と表の値が一致するか。
fn values_match(expr: &str, table_value: &str) -> bool {
    let expr_numbers = numbers(expr);
    if expr_numbers.is_empty() {
        let last = expr.rsplit("::").next().unwrap_or(expr).trim();
        normalize(last) == normalize(table_value)
    } else {
        expr_numbers == numbers(table_value)
    }
}

fn is_composite(expr: &str) -> bool {
    expr.contains('{') || expr.contains('[')
}

fn compare(rows: &BTreeMap<String, Row>, consts: &BTreeMap<String, Const>, report: &mut Report) {
    for (name, row) in rows {
        match consts.get(name) {
            None => report.errors.push(format!(
                "TOKENS_SOURCE.md:{}: {name} がソースにない",
                row.line
            )),
            Some(c) if is_composite(&c.expr) => report.errors.push(format!(
                "TOKENS_SOURCE.md:{}: {name} は他の定数を組み合わせた定数なので表に載せない",
                row.line
            )),
            Some(c) if !values_match(&c.expr, &row.value) => report.errors.push(format!(
                "TOKENS_SOURCE.md:{}: {name} の値が違う（表: {}、{}: {}）",
                row.line,
                row.value,
                c.file.display(),
                c.expr
            )),
            Some(_) => report.checked += 1,
        }
    }
    for (name, c) in consts {
        if !is_composite(&c.expr) && !rows.contains_key(name) {
            report.errors.push(format!(
                "{}: {name} の出典が TOKENS_SOURCE.md にない",
                c.file.display()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_numbers() {
        assert_eq!(
            numbers("CubicBezier::new(0.2, 0.0, 0.8, 0.15)"),
            [0.2, 0.0, 0.8, 0.15]
        );
        assert_eq!(numbers("Duration::from_millis(1_000)"), [1000.0]);
        assert_eq!(numbers("-0.2"), [-0.2]);
        assert_eq!(numbers("LEVEL0"), Vec::<f64>::new());
        assert_eq!(numbers("a - 1"), [1.0]);
        assert_eq!(numbers("x-1"), [1.0]);
    }

    #[test]
    fn parses_consts() {
        let source = "\
/// doc
pub const A: f32 = 4.0; // comment
pub const B: Typeface = Typeface::Brand;
pub const fn f() -> u8 { 0 }
pub const C: TypeStyle = TypeStyle {
    size: A,
};
#[cfg(test)]
mod tests {
    pub const D: f32 = 1.0;
}
";
        assert_eq!(
            parse_consts(source),
            [
                ("A".to_string(), "4.0".to_string()),
                ("B".to_string(), "Typeface::Brand".to_string()),
                ("C".to_string(), "TypeStyle { size: A, }".to_string()),
            ]
        );
    }

    #[test]
    fn matches_values() {
        assert!(values_match("4.0", "4"));
        assert!(values_match("Typeface::Brand", "Brand"));
        assert!(values_match("WEIGHT_REGULAR", "WeightRegular"));
        assert!(values_match("CornerSize::Full", "Full"));
        assert!(!values_match("4.0", "4.5"));
        assert!(!values_match("Typeface::Plain", "Brand"));
    }

    #[test]
    fn module_paths() {
        let root = Path::new("/src");
        assert_eq!(module_path(root, Path::new("/src/lib.rs")), "");
        assert_eq!(module_path(root, Path::new("/src/shape.rs")), "shape");
        assert_eq!(module_path(root, Path::new("/src/button/mod.rs")), "button");
        assert_eq!(
            module_path(root, Path::new("/src/button/filled.rs")),
            "button::filled"
        );
    }

    #[test]
    fn reports_mismatches() {
        let mut errors = Vec::new();
        let rows = parse_table(
            "| md3_tokens::a::X | 1.0 | f.kt | L1 | c |\n| md3_tokens::a::Y | 2.0 | f.kt | L2 | c |\n",
            &mut errors,
        );
        assert!(errors.is_empty());
        let mut consts = BTreeMap::new();
        let c = |expr: &str| Const {
            expr: expr.to_string(),
            file: PathBuf::from("a.rs"),
        };
        consts.insert("md3_tokens::a::X".to_string(), c("1.0"));
        consts.insert("md3_tokens::a::Z".to_string(), c("3.0"));
        let mut report = Report::default();
        compare(&rows, &consts, &mut report);
        assert_eq!(report.checked, 1);
        assert_eq!(report.errors.len(), 2, "{report}");
    }
}
