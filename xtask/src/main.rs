//! md-components の開発用コマンド
//!
//! - `cargo xtask gen-tokens`: Compose Material3 の生成トークン (`tokens/*Tokens.kt`) を
//!   `crates/md3-tokens/src/generated/` に転記する
//! - `cargo xtask gen-icons`: Material Symbols の SVG を `crates/md3-icons/assets/` にコピーし、
//!   `crates/md3-icons/src/generated.rs` にアイコンごとの定数を生成する
//!
//! どちらも `--check` を付けると書き換えずに、生成結果が最新かを確かめ、差分があれば失敗する

mod icons;
mod tokens;

use std::collections::BTreeSet;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const USAGE: &str = "usage: cargo xtask <gen-tokens|gen-icons> [--check]";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = pico_args::Arguments::from_env();
    let command = args.subcommand().map_err(|e| e.to_string())?;
    let check = args.contains("--check");
    let rest = args.finish();
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {rest:?}\n{USAGE}"));
    }
    match command.as_deref() {
        Some("gen-tokens") => tokens::generate(check),
        Some("gen-icons") => icons::generate(check),
        _ => Err(USAGE.to_owned()),
    }
}

fn workspace_root() -> PathBuf {
    // xtask/ の親がワークスペースのルート
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// `outputs` (`out_dir` からの相対パスと内容) を書き出し、`prune_dirs` 直下のほかのファイルを消す
///
/// 内容が違うファイルと余分なファイルのパスを返す。`check` のときは書き換えも削除もしない
fn sync_outputs(
    out_dir: &Path,
    outputs: &[(String, Vec<u8>)],
    prune_dirs: &[PathBuf],
    check: bool,
) -> Result<Vec<String>, String> {
    let mut expected = BTreeSet::new();
    let mut stale = Vec::new();
    for (relative, bytes) in outputs {
        let path = out_dir.join(relative);
        expected.insert(path.clone());
        if fs::read(&path).ok().as_deref() == Some(bytes.as_slice()) {
            continue;
        }
        if check {
            stale.push(relative.clone());
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    }

    for dir in prune_dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if !path.is_file() || expected.contains(&path) {
                continue;
            }
            if check {
                stale.push(path.display().to_string());
            } else {
                fs::remove_file(&path)
                    .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
            }
        }
    }
    Ok(stale)
}

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
