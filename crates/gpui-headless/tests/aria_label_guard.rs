//! 名前 setter の直接定義を防ぐ。
use std::fs;
use std::path::Path;

fn check_directory(path: &Path) {
    for entry in fs::read_dir(path).expect("source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            check_directory(&path);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path).expect("Rust source");
            let mut macro_depth = 0_i32;
            for (index, line) in source.lines().enumerate() {
                let starts_macro =
                    macro_depth == 0 && line.trim_start().starts_with("macro_rules!");
                assert!(
                    macro_depth > 0 || starts_macro || !line.contains("pub fn aria_label("),
                    "direct aria_label definition in {}:{}",
                    path.display(),
                    index + 1
                );
                if macro_depth > 0 || starts_macro {
                    macro_depth += line.matches('{').count() as i32;
                    macro_depth -= line.matches('}').count() as i32;
                }
            }
            assert_eq!(macro_depth, 0, "unterminated macro in {}", path.display());
        }
    }
}

#[test]
fn aria_label_is_only_defined_in_macros() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates directory");
    check_directory(&workspace.join("gpui-headless/src"));
    check_directory(&workspace.join("md3-components/src"));
}
