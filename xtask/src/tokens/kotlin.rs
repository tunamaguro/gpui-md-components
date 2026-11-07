//! Compose の生成トークン (Kotlin) の読み取りと、Rust の式への変換

use std::collections::BTreeSet;

use super::screaming_snake_case;

pub(super) struct KotlinFile {
    pub(super) file_name: String,
    pub(super) object: String,
    /// `// VERSION:` の値。MotionSchemeKeyTokens のように版を持たないファイルもある
    pub(super) version: Option<String>,
    pub(super) copyright: String,
    pub(super) declarations: Vec<Declaration>,
}

pub(super) struct Declaration {
    pub(super) name: String,
    pub(super) kotlin_type: Option<String>,
    pub(super) expr: String,
}

/// `internal object` の中の `val` 宣言を順に読む
///
/// 生成トークンの宣言は `[inline|const] val Name[: Type] = Expr` か、
/// 次の行に `[inline] get() = Expr` が続く形だけを受け付ける。式は次の宣言か `}` の手前まで続く
pub(super) fn parse_file(file_name: &str, source: &str) -> Result<KotlinFile, String> {
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
pub(super) struct Value {
    pub(super) rust_type: &'static str,
    pub(super) expr: String,
}

/// Kotlin の式を Rust の式に置き換える。対応しない式はエラーにして、黙って落とさない
pub(super) fn convert(
    declaration: &Declaration,
    imports: &mut BTreeSet<String>,
) -> Result<Value, String> {
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
pub(super) fn type_path(rust_type: &str) -> Option<&'static str> {
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
