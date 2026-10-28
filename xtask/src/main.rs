//! md-components の開発用コマンド。`cargo xtask <サブコマンド>` で実行する。
//!
//! 外部 crate に依存せず std だけで書く（AGENTS.md）。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod check_tokens;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "\
使い方: cargo xtask <サブコマンド>

サブコマンド:
  web           wasm 版ギャラリーをビルドする（M7 で実装）
  check-tokens  docs/TOKENS_SOURCE.md と md3-tokens の定数を突き合わせる
  help          この説明を表示する";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("web") => web(),
        Some("check-tokens") => check_tokens(),
        Some("help" | "-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("不明なサブコマンド: {other}\n\n{USAGE}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// wasm 版ギャラリーをビルドする。
fn web() -> ExitCode {
    eprintln!("xtask web: 未実装（M7 で整備）");
    ExitCode::SUCCESS
}

/// `docs/TOKENS_SOURCE.md` の表と `md3-tokens` の定数を突き合わせる。
fn check_tokens() -> ExitCode {
    // xtask はワークスペース直下の `xtask/` にある。
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    match check_tokens::run(workspace_root) {
        Ok(checked) => {
            println!("check-tokens: {checked} 件の定数が TOKENS_SOURCE.md と一致");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("check-tokens: {message}");
            ExitCode::FAILURE
        }
    }
}
