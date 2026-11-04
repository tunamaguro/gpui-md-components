//! 移植元の TypeScript 実装で生成した配色 (fixtures/dynamic_schemes.txt) との照合
//!
//! 期待値の生成手順は fixtures/generate_schemes.mjs を参照

use md3_color::{
    Argb, ColorRole, ContrastLevel, DynamicScheme, Hct, Platform, SpecVersion, Variant,
};

const FIXTURE: &str = include_str!("fixtures/dynamic_schemes.txt");

fn variant(name: &str) -> Variant {
    match name {
        "monochrome" => Variant::Monochrome,
        "neutral" => Variant::Neutral,
        "tonal_spot" => Variant::TonalSpot,
        "vibrant" => Variant::Vibrant,
        "expressive" => Variant::Expressive,
        "fidelity" => Variant::Fidelity,
        "content" => Variant::Content,
        "rainbow" => Variant::Rainbow,
        "fruit_salad" => Variant::FruitSalad,
        "cmf" => Variant::Cmf,
        other => unreachable!("未知の variant: {other}"),
    }
}

fn spec_version(name: &str) -> SpecVersion {
    match name {
        "2021" => SpecVersion::Spec2021,
        "2025" => SpecVersion::Spec2025,
        "2026" => SpecVersion::Spec2026,
        other => unreachable!("未知の仕様: {other}"),
    }
}

fn platform(name: &str) -> Platform {
    match name {
        "phone" => Platform::Phone,
        "watch" => Platform::Watch,
        other => unreachable!("未知の機器: {other}"),
    }
}

fn parse_argb(hex: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(hex, 16)
}

#[test]
fn 列の順序が_color_role_all_と一致する() {
    let header = FIXTURE.lines().next().unwrap();
    let columns: Vec<&str> = header
        .split("columns: ")
        .nth(1)
        .unwrap()
        .split(' ')
        .collect();
    let expected: Vec<String> = ColorRole::ALL
        .iter()
        .map(|role| {
            // snake_case を移植元のメソッド名 (lowerCamelCase) に直す
            let mut name = String::new();
            let mut upper = false;
            for c in role.name().chars() {
                if c == '_' {
                    upper = true;
                } else if upper {
                    name.extend(c.to_uppercase());
                    upper = false;
                } else {
                    name.push(c);
                }
            }
            name
        })
        .collect();
    assert_eq!(columns, expected);
}

#[test]
fn 移植元の配色と一致する() {
    let mut mismatches = Vec::new();
    let mut schemes = 0;
    for line in FIXTURE.lines().skip(1) {
        let (config, values) = line.split_once(" : ").unwrap();
        let config: Vec<&str> = config.split(' ').collect();
        let [variant_name, spec, platform_name, mode, contrast, sources] = config[..] else {
            unreachable!("行の形式が不正: {line}");
        };
        let sources: Vec<Hct> = sources
            .split(',')
            .map(|hex| Hct::from_argb(Argb::new(parse_argb(hex).unwrap())))
            .collect();
        let scheme = DynamicScheme::builder(sources[0], variant(variant_name))
            .additional_source_colors(sources[1..].iter().copied())
            .dark(mode == "dark")
            .contrast_level(ContrastLevel::new(contrast.parse().unwrap()))
            .spec_version(spec_version(spec))
            .platform(platform(platform_name))
            .build()
            .unwrap();
        schemes += 1;
        let values: Vec<&str> = values.split(' ').collect();
        assert_eq!(values.len(), ColorRole::ALL.len(), "列の数が不正: {line}");
        for (role, expected) in ColorRole::ALL.iter().zip(values) {
            let expected = 0xff00_0000 | parse_argb(expected).unwrap();
            let actual = scheme.argb(*role).to_u32();
            if actual != expected {
                mismatches.push(format!(
                    "{} {spec} {platform_name} {mode} {contrast} {:?}: {} expected {expected:08x} actual {actual:08x}",
                    variant_name,
                    scheme.source_color_hcts().iter().map(|h| format!("{:08x}", h.to_argb().to_u32())).collect::<Vec<_>>(),
                    role.name()
                ));
            }
        }
    }
    assert!(schemes > 1000, "配色の数が少ない: {schemes}");
    assert!(
        mismatches.is_empty(),
        "{} 件の不一致:\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
