//! 公開 API に `gpui_headless` の名前を出さない（再公開とシグネチャ）。

use std::path::Path;

fn check(dir: &Path, leaks: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("src を読める") {
        let path = entry.expect("項目を読める").path();
        if path.is_dir() {
            check(&path, leaks);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = std::fs::read_to_string(&path).expect("ファイルを読める");
            for (index, line) in source.lines().enumerate() {
                let line = line.trim_start();
                let public = ["pub use ", "pub fn ", "pub type ", "pub trait "]
                    .iter()
                    .any(|item| line.starts_with(item));
                if public && line.contains("gpui_headless") {
                    leaks.push(format!("{}:{}: {line}", path.display(), index + 1));
                }
            }
        }
    }
}

#[test]
fn public_items_do_not_name_gpui_headless() {
    let mut leaks = Vec::new();
    check(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut leaks,
    );
    assert!(leaks.is_empty(), "公開 API に出ている: {leaks:#?}");
}
