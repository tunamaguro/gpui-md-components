//! 依存の範囲を検証する。

#[test]
fn only_gpui_is_a_dependency() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml");
    let section = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("dependencies");
    let section = section.split('[').next().expect("section");
    let names: Vec<_> = section
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split('=')
                .next()
                .expect("dependency")
                .trim()
                .split('.')
                .next()
                .expect("name")
        })
        .collect();
    assert_eq!(names, ["gpui"]);
}
