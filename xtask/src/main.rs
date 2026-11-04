//! md-components の開発用コマンド
//!
//! - `cargo xtask gen-tokens`: Compose Material3 の生成トークン (`tokens/*Tokens.kt`) を
//!   `crates/md3-tokens/src/generated/` に転記する
//! - `cargo xtask gen-tokens --check`: 転記結果が最新かを確かめ、差分があれば失敗する

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::{env, fs, io::Write as _};

const ANDROIDX: &str = ".agents/skills/material-design-3-port/references/androidx";
const TOKENS_DIR: &str =
    "compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens";
const OUT_DIR: &str = "crates/md3-tokens/src/generated";

/// 転記しないトークンファイルと理由
///
/// - 色の固定値 (`ColorLightTokens` / `ColorDarkTokens` / `PaletteTokens`) は `md3-color` の動的配色で求める
/// - `ColorSchemeKeyTokens` は `md3_color::ColorRole` に対応させる
/// - `TypographyTokens` は `TypeScaleTokens` の値を `TextStyle` にまとめるだけなので、まとめた結果を `type_scale` に出力する
const SKIPPED: &[&str] = &[
    "ColorDarkTokens",
    "ColorLightTokens",
    "ColorSchemeKeyTokens",
    "PaletteTokens",
    "TypographyTokens",
];

/// コンポーネントではなく system のトークンとして `generated/` の直下に置くファイル
const SYSTEM: &[&str] = &[
    "ElevationTokens",
    "ExpressiveMotionTokens",
    "MotionSchemeKeyTokens",
    "MotionTokens",
    "ShapeKeyTokens",
    "ShapeTokens",
    "StandardMotionTokens",
    "StateTokens",
    "TypeScaleTokens",
    "TypefaceTokens",
    "TypographyKeyTokens",
];

/// キーだけを列挙するファイルと、出力する enum の名前
const KEY_ENUMS: &[(&str, &str)] = &[
    ("MotionSchemeKeyTokens", "MotionSchemeKey"),
    ("ShapeKeyTokens", "ShapeKey"),
    ("TypographyKeyTokens", "TypeScale"),
];

/// `TypeScaleTokens` の 1 つの書体を構成する値の接尾辞と、`TypeStyle` のフィールド名
const TYPE_STYLE_FIELDS: &[(&str, &str)] = &[
    ("Font", "font"),
    ("Weight", "weight"),
    ("Size", "size"),
    ("LineHeight", "line_height"),
    ("Tracking", "tracking"),
];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["gen-tokens"] => gen_tokens(false),
        ["gen-tokens", "--check"] => gen_tokens(true),
        _ => Err("usage: cargo xtask gen-tokens [--check]".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    // xtask/ の親がワークスペースのルート
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn gen_tokens(check: bool) -> Result<(), String> {
    let root = workspace_root();
    let androidx = root.join(ANDROIDX);
    let commit = git_head(&androidx)?;
    let tokens_dir = androidx.join(TOKENS_DIR);

    let mut names: Vec<String> = fs::read_dir(&tokens_dir)
        .map_err(|e| format!("failed to read {}: {e}", tokens_dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_suffix(".kt"))
                .map(str::to_owned)
        })
        .filter(|name| name.ends_with("Tokens") && !SKIPPED.contains(&name.as_str()))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err(format!(
            "no token files in {} (is the submodule checked out?)",
            tokens_dir.display()
        ));
    }

    let mut outputs: Vec<(PathBuf, String)> = Vec::new();
    let mut system_modules = Vec::new();
    let mut component_modules = Vec::new();
    for name in &names {
        let path = tokens_dir.join(format!("{name}.kt"));
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let file = parse_file(name, &source).map_err(|e| format!("{name}.kt: {e}"))?;
        let module = snake_case(name.trim_end_matches("Tokens"));
        let is_system = SYSTEM.contains(&name.as_str());
        let body = render_file(&file, &commit).map_err(|e| format!("{name}.kt: {e}"))?;
        let relative = if is_system {
            system_modules.push((module.clone(), name.clone()));
            format!("{module}.rs")
        } else {
            component_modules.push((module.clone(), name.clone()));
            format!("component/{module}.rs")
        };
        outputs.push((PathBuf::from(relative), body));
    }

    let mut generated_mod = String::from(GENERATED_HEADER);
    generated_mod.push_str("\n//! Compose Material3 の生成トークンの転記\n\n");
    generated_mod.push_str("/// コンポーネントごとのトークン\npub mod component;\n");
    for (module, name) in &system_modules {
        let _ = writeln!(generated_mod, "/// `{name}` の転記\npub mod {module};");
    }
    outputs.push((PathBuf::from("mod.rs"), generated_mod));

    let mut component_mod = String::from(GENERATED_HEADER);
    component_mod.push_str("\n//! コンポーネントごとの生成トークンの転記\n\n");
    for (module, name) in &component_modules {
        let _ = writeln!(component_mod, "/// `{name}` の転記\npub mod {module};");
    }
    outputs.push((PathBuf::from("component/mod.rs"), component_mod));

    let out_dir = root.join(OUT_DIR);
    let mut expected = BTreeSet::new();
    let mut stale = Vec::new();
    for (relative, body) in &outputs {
        let formatted = rustfmt(&root, body)?;
        let path = out_dir.join(relative);
        expected.insert(path.clone());
        let current = fs::read_to_string(&path).ok();
        if current.as_deref() == Some(formatted.as_str()) {
            continue;
        }
        if check {
            stale.push(relative.display().to_string());
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        fs::write(&path, formatted)
            .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    }

    for dir in [out_dir.clone(), out_dir.join("component")] {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") && !expected.contains(&path) {
                if check {
                    stale.push(path.display().to_string());
                } else {
                    fs::remove_file(&path)
                        .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
                }
            }
        }
    }

    if !stale.is_empty() {
        return Err(format!(
            "generated tokens are out of date; run `cargo xtask gen-tokens`:\n  {}",
            stale.join("\n  ")
        ));
    }
    Ok(())
}

const GENERATED_HEADER: &str = "// Generated by `cargo xtask gen-tokens`. DO NOT EDIT.\n";

fn git_head(repo: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err(format!("git rev-parse failed in {}", repo.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn rustfmt(root: &Path, source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run rustfmt: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("failed to write to rustfmt: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for rustfmt: {e}"))?;
    if !output.status.success() {
        return Err(format!("rustfmt failed on generated code:\n{source}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

struct KotlinFile {
    file_name: String,
    object: String,
    /// `// VERSION:` の値。MotionSchemeKeyTokens のように版を持たないファイルもある
    version: Option<String>,
    copyright: String,
    declarations: Vec<Declaration>,
}

struct Declaration {
    name: String,
    kotlin_type: Option<String>,
    expr: String,
}

/// `internal object` の中の `val` 宣言を順に読む
///
/// 生成トークンの宣言は `[inline|const] val Name[: Type] = Expr` か、
/// 次の行に `[inline] get() = Expr` が続く形だけを受け付ける。式は次の宣言か `}` の手前まで続く
fn parse_file(file_name: &str, source: &str) -> Result<KotlinFile, String> {
    let version = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("// VERSION:"))
        .map(|v| v.trim().to_owned());
    let copyright = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("* Copyright "))
        .map(|v| v.trim().to_owned())
        .ok_or("missing copyright notice")?;

    // オブジェクト名はファイル名と異なることがある (IconButtonTokens.kt の StandardIconButtonTokens)
    let mut lines = source
        .lines()
        .skip_while(|line| !line.starts_with("internal object "));
    let object = lines
        .next()
        .and_then(|line| line.strip_prefix("internal object "))
        .and_then(|rest| rest.strip_suffix(" {"))
        .ok_or("missing `internal object`")?
        .to_owned();

    let mut statements: Vec<String> = Vec::new();
    for line in lines {
        let line = line.trim();
        if line == "}" {
            break;
        }
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let starts_declaration = ["val ", "inline val ", "const val "]
            .iter()
            .any(|prefix| line.starts_with(prefix));
        match statements.last_mut() {
            Some(last) if !starts_declaration => {
                last.push(' ');
                last.push_str(line);
            }
            _ if starts_declaration => statements.push(line.to_owned()),
            _ => return Err(format!("unexpected line `{line}`")),
        }
    }

    let declarations = statements
        .iter()
        .map(|statement| parse_declaration(statement))
        .collect::<Result<_, _>>()?;
    Ok(KotlinFile {
        file_name: file_name.to_owned(),
        object,
        version,
        copyright,
        declarations,
    })
}

fn parse_declaration(statement: &str) -> Result<Declaration, String> {
    let rest = statement
        .trim_start_matches("inline ")
        .trim_start_matches("const ")
        .strip_prefix("val ")
        .ok_or_else(|| format!("unsupported declaration `{statement}`"))?;
    let name_end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    let (name, rest) = rest.split_at(name_end);
    let rest = rest.trim();
    let (kotlin_type, rest) = match rest.strip_prefix(':') {
        Some(typed) => {
            let typed = typed.trim();
            let end = typed.find(' ').unwrap_or(typed.len());
            (Some(typed[..end].to_owned()), typed[end..].trim())
        }
        None => (None, rest),
    };
    let expr = rest
        .strip_prefix("inline get() =")
        .or_else(|| rest.strip_prefix("get() ="))
        .or_else(|| rest.strip_prefix('='))
        .ok_or_else(|| format!("missing initializer in `{statement}`"))?;
    // 空白と末尾のカンマを除き、`f(a, b,)` を `f(a,b)` の形にそろえる
    let expr: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    let expr = expr.replace(",)", ")");
    Ok(Declaration {
        name: name.to_owned(),
        kotlin_type,
        expr,
    })
}

/// 転記先の Rust の型と式
struct Value {
    rust_type: &'static str,
    expr: String,
}

fn render_file(file: &KotlinFile, commit: &str) -> Result<String, String> {
    let mut out = String::from(GENERATED_HEADER);
    let version = file.version.as_ref().map_or_else(String::new, |version| {
        format!(" (tokens VERSION {version})")
    });
    let _ = write!(
        out,
        "// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)\n\
         // Source: {TOKENS_DIR}/{file_name}.kt\n\
         //         @ {commit}{version}\n\
         // Copyright {copyright}. Licensed under the Apache License, Version 2.0.\n\n\
         //! `{object}` の転記\n\n",
        file_name = file.file_name,
        object = file.object,
        copyright = file.copyright,
    );

    if let Some((_, enum_name)) = KEY_ENUMS.iter().find(|(object, _)| *object == file.object) {
        render_key_enum(&mut out, file, enum_name)?;
        return Ok(out);
    }

    let mut imports = BTreeSet::new();
    let mut body = String::new();
    if file.object == "TypeScaleTokens" {
        render_type_scale(&mut body, file, &mut imports)?;
    } else {
        for declaration in &file.declarations {
            let value = convert(declaration, &mut imports)?;
            if let Some(path) = type_path(value.rust_type) {
                imports.insert(path.to_owned());
            }
            let _ = writeln!(
                body,
                "/// `{}`\npub const {}: {} = {};",
                declaration.name,
                screaming_snake_case(&declaration.name),
                value.rust_type,
                value.expr
            );
        }
    }
    for import in &imports {
        let _ = writeln!(out, "use {import};");
    }
    if !imports.is_empty() {
        out.push('\n');
    }
    out.push_str(&body);
    Ok(out)
}

fn render_key_enum(out: &mut String, file: &KotlinFile, enum_name: &str) -> Result<(), String> {
    let mut variants = Vec::new();
    for (index, declaration) in file.declarations.iter().enumerate() {
        let id = declaration
            .expr
            .split_once('(')
            .and_then(|(_, id)| id.strip_suffix(')'))
            .ok_or_else(|| format!("unsupported key `{}`", declaration.expr))?;
        // Compose は id で識別するため、id が宣言順と一致することを前提に宣言順で列挙する
        if id != index.to_string() {
            return Err(format!(
                "key `{}` has id {id}, expected {index}",
                declaration.name
            ));
        }
        variants.push(declaration.name.as_str());
    }
    let _ = writeln!(
        out,
        "/// `{object}` のキー。宣言順は移植元の id 順\n\
         #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n\
         pub enum {enum_name} {{",
        object = file.object
    );
    for variant in &variants {
        let _ = writeln!(out, "    /// `{variant}`\n    {variant},");
    }
    let _ = writeln!(
        out,
        "}}\n\nimpl {enum_name} {{\n    /// すべてのキー。移植元の id 順\n    pub const ALL: [Self; {len}] = [{list}];\n}}",
        len = variants.len(),
        list = variants
            .iter()
            .map(|v| format!("Self::{v}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

/// `TypeScaleTokens` の値を、`TypographyTokens` と同じ組み合わせで書体ごとにまとめる
fn render_type_scale(
    out: &mut String,
    file: &KotlinFile,
    imports: &mut BTreeSet<String>,
) -> Result<(), String> {
    imports.insert("crate::typography::TypeStyle".to_owned());
    // 書体の名前と、TYPE_STYLE_FIELDS の順に並べた (Kotlin の名前, 値)
    type Fields = Vec<Option<(String, Value)>>;
    let mut styles: Vec<(String, Fields)> = Vec::new();
    for declaration in &file.declarations {
        let (index, prefix) = TYPE_STYLE_FIELDS
            .iter()
            .enumerate()
            .find_map(|(index, (suffix, _))| {
                declaration
                    .name
                    .strip_suffix(suffix)
                    .map(|prefix| (index, prefix))
            })
            .ok_or_else(|| format!("unknown type scale token `{}`", declaration.name))?;
        let value = convert(declaration, imports)?;
        let style = match styles.iter_mut().find(|(name, _)| name == prefix) {
            Some(style) => style,
            None => {
                styles.push((
                    prefix.to_owned(),
                    TYPE_STYLE_FIELDS.iter().map(|_| None).collect(),
                ));
                styles.last_mut().ok_or("unreachable")?
            }
        };
        if style.1[index]
            .replace((declaration.name.clone(), value))
            .is_some()
        {
            return Err(format!("duplicate type scale token `{}`", declaration.name));
        }
    }
    for (name, fields) in &styles {
        let _ = write!(
            out,
            "/// `{name}*`\npub const {}: TypeStyle = TypeStyle {{\n",
            screaming_snake_case(name)
        );
        for ((suffix, field), value) in TYPE_STYLE_FIELDS.iter().zip(fields) {
            let (source, value) = value
                .as_ref()
                .ok_or_else(|| format!("missing type scale token `{name}{suffix}`"))?;
            let _ = writeln!(out, "    // `{source}`\n    {field}: {},", value.expr);
        }
        out.push_str("};\n");
    }
    Ok(())
}

/// Kotlin の式を Rust の式に置き換える。対応しない式はエラーにして、黙って落とさない
fn convert(declaration: &Declaration, imports: &mut BTreeSet<String>) -> Result<Value, String> {
    let expr = declaration.expr.as_str();
    let mut import = |path: &str| {
        imports.insert(path.to_owned());
    };
    let value = if let Some(role) = expr.strip_prefix("ColorSchemeKeyTokens.") {
        import("md3_color::ColorRole");
        Value {
            rust_type: "ColorRole",
            expr: format!("ColorRole::{role}"),
        }
    } else if let Some(key) = expr.strip_prefix("ShapeKeyTokens.") {
        import("crate::shape::ShapeKey");
        Value {
            rust_type: "ShapeKey",
            expr: format!("ShapeKey::{key}"),
        }
    } else if let Some(key) = expr.strip_prefix("TypographyKeyTokens.") {
        import("crate::typography::TypeScale");
        Value {
            rust_type: "TypeScale",
            expr: format!("TypeScale::{key}"),
        }
    } else if let Some(key) = expr.strip_prefix("MotionSchemeKeyTokens.") {
        import("crate::motion::MotionSchemeKey");
        Value {
            rust_type: "MotionSchemeKey",
            expr: format!("MotionSchemeKey::{key}"),
        }
    } else if let Some(level) = expr.strip_prefix("ElevationTokens.") {
        import("crate::generated::elevation");
        Value {
            rust_type: "Dp",
            expr: format!("elevation::{}", screaming_snake_case(level)),
        }
    } else if let Some(shape) = expr.strip_prefix("ShapeTokens.") {
        import("crate::generated::shape");
        let rust_type = if shape.starts_with("CornerValue") {
            "CornerSize"
        } else {
            "CornerShape"
        };
        Value {
            rust_type,
            expr: format!("shape::{}", screaming_snake_case(shape)),
        }
    } else if let Some(typeface) = expr.strip_prefix("TypefaceTokens.") {
        if typeface.starts_with("Weight") {
            import("crate::generated::typeface");
            Value {
                rust_type: "FontWeight",
                expr: format!("typeface::{}", screaming_snake_case(typeface)),
            }
        } else {
            // Brand / Plain はテーマが別々に上書きできるよう、解決せずキーとして残す
            import("crate::typography::Typeface");
            Value {
                rust_type: "Typeface",
                expr: format!("Typeface::{typeface}"),
            }
        }
    } else if let Some(family) = expr.strip_prefix("FontFamily.") {
        import("crate::typography::FontFamily");
        Value {
            rust_type: "FontFamily",
            expr: format!("FontFamily::{family}"),
        }
    } else if let Some(weight) = expr.strip_prefix("FontWeight.") {
        import("crate::typography::FontWeight");
        Value {
            rust_type: "FontWeight",
            expr: format!("FontWeight::{}", screaming_snake_case(weight)),
        }
    } else if let Some(dp) = expr.strip_suffix(".dp") {
        import("crate::units::Dp");
        Value {
            rust_type: "Dp",
            expr: format!("Dp({})", float(dp)?),
        }
    } else if let Some(sp) = expr.strip_suffix(".sp") {
        import("crate::units::Sp");
        Value {
            rust_type: "Sp",
            expr: format!("Sp({})", float(sp)?),
        }
    } else if let Some(args) = call_args(expr, "CubicBezierEasing") {
        import("crate::motion::CubicBezier");
        let args: Vec<String> = args
            .iter()
            .map(|arg| {
                arg.strip_suffix('f')
                    .ok_or_else(|| format!("expected Float in `{expr}`"))
                    .and_then(float)
            })
            .collect::<Result<_, _>>()?;
        Value {
            rust_type: "CubicBezier",
            expr: format!("CubicBezier::new({})", args.join(", ")),
        }
    } else if let Some(args) = call_args(expr, "CornerSize") {
        import("crate::shape::CornerSize");
        let [arg] = args.as_slice() else {
            return Err(format!("unsupported corner size in `{expr}`"));
        };
        Value {
            rust_type: "CornerSize",
            expr: corner_size(arg, expr, &mut import)?,
        }
    } else if let Some(args) = call_args(expr, "RoundedCornerShape") {
        import("crate::shape::CornerShape");
        import("crate::shape::CornerSize");
        let corners = match args.as_slice() {
            [all] => {
                let size = corner_size(all, expr, &mut import)?;
                [size.clone(), size.clone(), size.clone(), size]
            }
            named => {
                let mut corners = [None, None, None, None];
                for arg in named {
                    let (key, value) = arg
                        .split_once('=')
                        .ok_or_else(|| format!("unsupported argument in `{expr}`"))?;
                    let index = ["topStart", "topEnd", "bottomEnd", "bottomStart"]
                        .iter()
                        .position(|corner| *corner == key)
                        .ok_or_else(|| format!("unknown corner `{key}` in `{expr}`"))?;
                    corners[index] = Some(corner_size(value, expr, &mut import)?);
                }
                let [Some(a), Some(b), Some(c), Some(d)] = corners else {
                    return Err(format!("missing corner in `{expr}`"));
                };
                [a, b, c, d]
            }
        };
        let [top_start, top_end, bottom_end, bottom_start] = corners;
        Value {
            rust_type: "CornerShape",
            expr: format!(
                "CornerShape {{ top_start: {top_start}, top_end: {top_end}, bottom_end: {bottom_end}, bottom_start: {bottom_start} }}"
            ),
        }
    } else if expr == "CircleShape" {
        import("crate::shape::CornerShape");
        Value {
            rust_type: "CornerShape",
            expr: "CornerShape::CIRCLE".to_owned(),
        }
    } else if expr == "RectangleShape" {
        import("crate::shape::CornerShape");
        Value {
            rust_type: "CornerShape",
            expr: "CornerShape::RECTANGLE".to_owned(),
        }
    } else if let Some(number) = expr.strip_suffix('f') {
        Value {
            rust_type: "f32",
            expr: float(number)?,
        }
    } else if declaration.name.starts_with("Duration") {
        // MotionTokens の時間はミリ秒の Double
        let millis = expr
            .strip_suffix(".0")
            .filter(|ms| ms.chars().all(|c| c.is_ascii_digit()))
            .ok_or_else(|| format!("unsupported duration `{expr}`"))?;
        import("std::time::Duration");
        Value {
            rust_type: "Duration",
            expr: format!("Duration::from_millis({millis})"),
        }
    } else {
        return Err(format!(
            "unsupported expression `{expr}` for `{}`",
            declaration.name
        ));
    };

    // 宣言された Kotlin の型と、式から決めた Rust の型が対応することを確かめる
    if let Some(kotlin_type) = &declaration.kotlin_type {
        let expected = match kotlin_type.rsplit('.').next().unwrap_or(kotlin_type) {
            "ColorToken" => "ColorRole",
            "ShapeToken" => "ShapeKey",
            "TypographyToken" => "TypeScale",
            "Dp" => "Dp",
            "TextUnit" => "Sp",
            "FontFamily" if value.rust_type == "Typeface" => "Typeface",
            "FontFamily" => "FontFamily",
            "FontWeight" => "FontWeight",
            "CubicBezierEasing" => "CubicBezier",
            other => {
                return Err(format!(
                    "unsupported type `{other}` for `{}`",
                    declaration.name
                ));
            }
        };
        if expected != value.rust_type {
            return Err(format!(
                "`{}` is declared as `{kotlin_type}` but converted to `{}`",
                declaration.name, value.rust_type
            ));
        }
    }
    Ok(value)
}

/// 定数の型注釈に使う型の import パス。組み込みの型は `None`
fn type_path(rust_type: &str) -> Option<&'static str> {
    match rust_type {
        "ColorRole" => Some("md3_color::ColorRole"),
        "ShapeKey" => Some("crate::shape::ShapeKey"),
        "TypeScale" => Some("crate::typography::TypeScale"),
        "MotionSchemeKey" => Some("crate::motion::MotionSchemeKey"),
        "Dp" => Some("crate::units::Dp"),
        "Sp" => Some("crate::units::Sp"),
        "CornerSize" => Some("crate::shape::CornerSize"),
        "CornerShape" => Some("crate::shape::CornerShape"),
        "FontFamily" => Some("crate::typography::FontFamily"),
        "Typeface" => Some("crate::typography::Typeface"),
        "FontWeight" => Some("crate::typography::FontWeight"),
        "CubicBezier" => Some("crate::motion::CubicBezier"),
        "Duration" => Some("std::time::Duration"),
        _ => None,
    }
}

fn corner_size(arg: &str, expr: &str, import: &mut impl FnMut(&str)) -> Result<String, String> {
    let dp = arg
        .strip_suffix(".dp")
        .ok_or_else(|| format!("unsupported corner size in `{expr}`"))?;
    import("crate::units::Dp");
    Ok(format!("CornerSize::Dp(Dp({}))", float(dp)?))
}

fn call_args(expr: &str, function: &str) -> Option<Vec<String>> {
    let inner = expr
        .strip_prefix(function)?
        .strip_prefix('(')?
        .strip_suffix(')')?;
    Some(inner.split(',').map(str::to_owned).collect())
}

/// Kotlin の数値リテラルを Rust の f32 リテラルにする。整数は `.0` を付ける
fn float(literal: &str) -> Result<String, String> {
    let digits = literal.strip_prefix('-').unwrap_or(literal);
    let valid = !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
        && digits.matches('.').count() <= 1
        && !digits.starts_with('.')
        && !digits.ends_with('.');
    if !valid {
        return Err(format!("unsupported number `{literal}`"));
    }
    Ok(if literal.contains('.') {
        literal.to_owned()
    } else {
        format!("{literal}.0")
    })
}

/// `FilledButton` を `filled_button` にする。数字は直前の語に続ける (`Level0` は `level0`)
fn snake_case(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower =
                i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            let prev_upper = i > 0 && chars[i - 1].is_ascii_uppercase();
            if i > 0 && (prev_lower || (prev_upper && next_lower)) {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn screaming_snake_case(name: &str) -> String {
    snake_case(name).to_ascii_uppercase()
}
