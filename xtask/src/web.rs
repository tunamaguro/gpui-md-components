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
            other => return Err(format!("不明な引数: {other}")),
        }
    }

    let fonts: Vec<PathBuf> = std::env::var_os(FONTS_ENV)
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    if fonts.is_empty() {
        return Err(format!(
            "環境変数 {FONTS_ENV} に UI の文字に使うフォント（TTF / OTF / TTC）のパスを並べる（`nix develop` では設定済み）"
        ));
    }

    let expected = locked_version(workspace_root, "wasm-bindgen")?;
    let installed = wasm_bindgen_version()?;
    if installed != expected {
        return Err(format!(
            "wasm-bindgen の CLI は {installed}、Cargo.lock は {expected}。同じ版の CLI を使う（`nix develop` の環境には入っている）"
        ));
    }

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
    let wasm = workspace_root
        .join("target/wasm32-unknown-unknown")
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
        .map_err(|error| format!("{} を書けない: {error}", index.display()))?;
    copy_fonts(&fonts, &out_dir.join(FONTS_DIR))?;
    Ok(out_dir.display().to_string())
}

/// フォントを `dir` に `0.ttf` のような名前で写し、ファイル名の一覧を `list.txt` に書く。
fn copy_fonts(fonts: &[PathBuf], dir: &Path) -> Result<(), String> {
    // 前回の出力が残らないよう作り直す。
    if dir.exists() {
        std::fs::remove_dir_all(dir)
            .map_err(|error| format!("{} を消せない: {error}", dir.display()))?;
    }
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("{} を作れない: {error}", dir.display()))?;
    let mut list = String::new();
    for (index, font) in fonts.iter().enumerate() {
        let extension = font
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("ttf");
        let name = format!("{index}.{extension}");
        // ストアのファイルは読み取り専用なので、権限ごと写す `fs::copy` ではなく中身だけを書く。
        let bytes = std::fs::read(font)
            .map_err(|error| format!("{} を読めない: {error}", font.display()))?;
        std::fs::write(dir.join(&name), bytes)
            .map_err(|error| format!("{} を書けない: {error}", dir.join(&name).display()))?;
        list.push_str(&name);
        list.push('\n');
    }
    let list_path = dir.join("list.txt");
    std::fs::write(&list_path, list)
        .map_err(|error| format!("{} を書けない: {error}", list_path.display()))
}

/// Cargo.lock に記録された crate の版。
fn locked_version(workspace_root: &Path, name: &str) -> Result<String, String> {
    let path = workspace_root.join("Cargo.lock");
    let lock = std::fs::read_to_string(&path)
        .map_err(|error| format!("{} を読めない: {error}", path.display()))?;
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
    Err(format!("Cargo.lock に {name} が無い"))
}

/// PATH にある wasm-bindgen の CLI の版。
fn wasm_bindgen_version() -> Result<String, String> {
    let output = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .map_err(|error| {
            format!("wasm-bindgen の CLI を起動できない（`nix develop` の環境で実行する）: {error}")
        })?;
    // 出力は `wasm-bindgen 0.2.129`。
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .nth(1)
        .map(str::to_owned)
        .ok_or_else(|| "wasm-bindgen --version の出力を読めない".to_owned())
}

/// コマンドを実行し、失敗したらエラーにする。
fn run_command(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("{command:?} を起動できない: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} が失敗した（{status}）"))
    }
}
