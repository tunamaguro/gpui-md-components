//! `cargo xtask web`: wasm 版ギャラリーをビルドし、静的ファイル一式を `target/web/` に出力する。
//!
//! 1. `cargo build -p gallery --target wasm32-unknown-unknown`（既定は `--release`）
//! 2. `wasm-bindgen --target web` で JS バインディングを作る
//! 3. `index.html` と、UI の文字に使うフォント（`fonts/`）を置く
//!
//! ブラウザでは gpui がシステムフォントを読めないため、ページがフォントを取得して wasm に渡す。
//! ライブラリはフォントを同梱しない方針なので、フォントはリポジトリに置かず、環境変数
//! `MD3_WEB_FONTS`（`PATH` と同じ区切りのパスの並び。`nix develop` では Noto Sans と
//! Noto Sans CJK）のファイルを `fonts/0.ttf` のような名前で写し、一覧を `fonts/list.txt` に書く。
//!
//! wasm-bindgen の CLI は Cargo.lock の `wasm-bindgen` と同じ版でなければならない。開発環境
//! （`nix develop`）には同じ版が入っている。共有メモリ（atomics）は使わないので、配信に
//! COOP / COEP ヘッダーは要らない。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 出力先（ワークスペース直下からの相対パス）。
const OUT_DIR: &str = "target/web";

/// 配信するページ。
const INDEX_HTML: &str = include_str!("../web/index.html");

/// フォントのファイルの並びを指す環境変数。
const FONTS_ENV: &str = "MD3_WEB_FONTS";

/// フォントを置くディレクトリ（出力先からの相対パス）。ページは `list.txt` に並んだファイルを取得する。
const FONTS_DIR: &str = "fonts";

/// ビルドして出力先のパスを返す。`args` は `--debug`（最適化しない）だけを受け付ける。
pub fn run(workspace_root: &Path, args: impl Iterator<Item = String>) -> Result<String, String> {
    let mut release = true;
    for arg in args {
        match arg.as_str() {
            "--debug" => release = false,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let fonts: Vec<PathBuf> = std::env::var_os(FONTS_ENV)
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    if fonts.is_empty() {
        return Err(format!(
            "set {FONTS_ENV} to a list of paths to UI fonts (TTF / OTF / TTC); it is already set in `nix develop`"
        ));
    }

    let expected = locked_version(workspace_root, "wasm-bindgen")?;
    let installed = wasm_bindgen_version()?;
    if installed != expected {
        return Err(format!(
            "wasm-bindgen CLI is {installed}, but Cargo.lock has {expected}; use the matching CLI version (available in `nix develop`)"
        ));
    }

    let target_dir = target_directory(workspace_root)?;
    let profile = if release { "release" } else { "debug" };
    let mut build = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()));
    build
        .current_dir(workspace_root)
        .args(["build", "--locked", "-p", "gallery", "--target"])
        .arg("wasm32-unknown-unknown");
    if release {
        build.arg("--release");
    }
    run_command(&mut build)?;

    let out_dir = workspace_root.join(OUT_DIR);
    let wasm = target_dir
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join("gallery.wasm");
    run_command(
        Command::new("wasm-bindgen")
            .args([
                "--target",
                "web",
                "--no-typescript",
                "--out-name",
                "gallery",
            ])
            .arg("--out-dir")
            .arg(&out_dir)
            .arg(&wasm),
    )?;

    let index = out_dir.join("index.html");
    std::fs::write(&index, INDEX_HTML)
        .map_err(|error| format!("cannot write {}: {error}", index.display()))?;
    copy_fonts(&fonts, &out_dir.join(FONTS_DIR))?;
    Ok(out_dir.display().to_string())
}

/// フォントを `dir` に `0.ttf` のような名前で写し、ファイル名の一覧を `list.txt` に書く。
fn copy_fonts(fonts: &[PathBuf], dir: &Path) -> Result<(), String> {
    // 前回の出力が残らないよう作り直す。
    if dir.exists() {
        std::fs::remove_dir_all(dir)
            .map_err(|error| format!("cannot remove {}: {error}", dir.display()))?;
    }
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let mut list = String::new();
    for (index, font) in fonts.iter().enumerate() {
        let extension = font
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("ttf");
        let name = format!("{index}.{extension}");
        // ストアのファイルは読み取り専用なので、権限ごと写す `fs::copy` ではなく中身だけを書く。
        let bytes = std::fs::read(font)
            .map_err(|error| format!("cannot read {}: {error}", font.display()))?;
        std::fs::write(dir.join(&name), bytes)
            .map_err(|error| format!("cannot write {}: {error}", dir.join(&name).display()))?;
        list.push_str(&name);
        list.push('\n');
    }
    let list_path = dir.join("list.txt");
    std::fs::write(&list_path, list)
        .map_err(|error| format!("cannot write {}: {error}", list_path.display()))
}

/// Cargo.lock に記録された crate の版。
fn locked_version(workspace_root: &Path, name: &str) -> Result<String, String> {
    let path = workspace_root.join("Cargo.lock");
    let lock = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let name_line = format!("name = \"{name}\"");
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == name_line
            && let Some(version) = lines
                .next()
                .and_then(|line| line.strip_prefix("version = \""))
                .and_then(|rest| rest.strip_suffix('"'))
        {
            return Ok(version.to_owned());
        }
    }
    Err(format!("Cargo.lock does not contain {name}"))
}

/// PATH にある wasm-bindgen の CLI の版。
fn wasm_bindgen_version() -> Result<String, String> {
    let output = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|error| {
            format!("cannot start wasm-bindgen CLI (run inside `nix develop`): {error}")
        })?;
    // 出力は `wasm-bindgen 0.2.129`。
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .nth(1)
        .map(str::to_owned)
        .ok_or_else(|| "cannot read wasm-bindgen --version output".to_owned())
}

/// コマンドを実行し、失敗したらエラーにする。
fn run_command(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("cannot start {command:?}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} failed ({status})"))
    }
}

/// Cargo の設定・環境変数を反映した出力先を問い合わせる。
fn target_directory(workspace_root: &Path) -> Result<PathBuf, String> {
    let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .current_dir(workspace_root)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
            "--offline",
        ])
        .output()
        .map_err(|error| format!("cannot start cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("cargo metadata output is not UTF-8: {error}"))?;
    metadata_target_directory(json)
        .map(PathBuf::from)
        .ok_or_else(|| "cannot read target_directory from cargo metadata".to_owned())
}

/// 最上位の文字列プロパティだけを読む。依存なしで JSON のエスケープを復元する。
fn metadata_target_directory(json: &str) -> Option<String> {
    let mut chars = json.chars().peekable();
    let mut depth = 0usize;
    while let Some(ch) = chars.next() {
        match ch {
            '{' | '[' => depth += 1,
            '}' | ']' => depth = depth.checked_sub(1)?,
            '"' => {
                let key = json_string(&mut chars)?;
                if depth == 1 && key == "target_directory" {
                    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
                        chars.next();
                    }
                    if chars.next()? != ':' {
                        return None;
                    }
                    while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
                        chars.next();
                    }
                    if chars.next()? != '"' {
                        return None;
                    }
                    return json_string(&mut chars);
                }
            }
            _ => {}
        }
    }
    None
}

/// 開き引用符の直後から JSON の文字列を読む。
fn json_string(chars: &mut impl Iterator<Item = char>) -> Option<String> {
    fn hex(chars: &mut impl Iterator<Item = char>) -> Option<u32> {
        (0..4).try_fold(0, |value, _| Some(value * 16 + chars.next()?.to_digit(16)?))
    }
    let mut text = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(text),
            '\\' => text.push(match chars.next()? {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{8}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    let first = hex(chars)?;
                    let code = if (0xd800..=0xdbff).contains(&first) {
                        if chars.next()? != '\\' || chars.next()? != 'u' {
                            return None;
                        }
                        let second = hex(chars)?;
                        if !(0xdc00..=0xdfff).contains(&second) {
                            return None;
                        }
                        0x10000 + ((first - 0xd800) << 10) + second - 0xdc00
                    } else {
                        first
                    };
                    char::from_u32(code)?
                }
                _ => return None,
            }),
            ch if ch.is_control() => return None,
            ch => text.push(ch),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_paths_decode_escapes_and_ignore_nested_fields() {
        assert_eq!(
            metadata_target_directory(
                r#"{"packages":[{"target_directory":"wrong"}],"target_directory":"/tmp/出力 dir"}"#
            )
            .as_deref(),
            Some("/tmp/出力 dir")
        );
        assert_eq!(
            metadata_target_directory(
                r#"{"target_directory":"C:\\work\\\"dir\"\\\u65e5\ud83d\ude80"}"#
            )
            .as_deref(),
            Some("C:\\work\\\"dir\"\\日🚀")
        );
        assert_eq!(
            metadata_target_directory(r#"{"target_directory":"/tmp/a\nb\tc"}"#).as_deref(),
            Some("/tmp/a\nb\tc")
        );
        assert_eq!(metadata_target_directory(r#"{"packages":[]}"#), None);
        assert_eq!(
            metadata_target_directory(r#"{"target_directory":"\ud800x"}"#),
            None
        );
        assert_eq!(
            metadata_target_directory(r#"{"target_directory":"unfinished}"#),
            None
        );
    }
}
