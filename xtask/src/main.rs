//! md-components の開発用コマンド。`cargo xtask <サブコマンド>` で実行する。
//!
//! 外部 crate に依存せず std だけで書く（AGENTS.md）。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod check_tokens;
mod web;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "\
Usage: cargo xtask <subcommand>

Subcommands:
  web [--debug] build the wasm examples in target/web/
  check-tokens  check md3-tokens constants against docs/TOKENS_SOURCE.md
  help          show this help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("web") => web(args),
        Some("check-tokens") => check_tokens(),
        Some("help" | "-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown subcommand: {other}\n\n{USAGE}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// wasm 版の見本をビルドする。
fn web(args: impl Iterator<Item = String>) -> ExitCode {
    match web::run(workspace_root(), args) {
        Ok(out_dir) => {
            println!(
                "web: output to {out_dir}. Serve as static files (e.g. python3 -m http.server -d {out_dir})"
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("web: {message}");
            ExitCode::FAILURE
        }
    }
}

/// ワークスペースの直下。xtask はワークスペース直下の `xtask/` にある。
fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
}

/// `docs/TOKENS_SOURCE.md` の表と `md3-tokens` の定数を突き合わせる。
fn check_tokens() -> ExitCode {
    match check_tokens::run(workspace_root()) {
        Ok(checked) => {
            println!("check-tokens: {checked} constants match TOKENS_SOURCE.md");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("check-tokens: {message}");
            ExitCode::FAILURE
        }
    }
}
