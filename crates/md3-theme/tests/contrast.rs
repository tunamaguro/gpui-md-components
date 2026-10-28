//! 全バリアント × 全コントラスト × ライト / ダークで、生成した配色が WCAG 2.2 のコントラスト比を満たすか。
//!
//! 既定のシード色だけでなく、色相を一周するシード色と、無彩色・高彩度・極端な明るさのシード色でも確かめる。
//! 検査するロールの組と基準（文字 4.5:1、UI 部品 3:1）は [`Theme::check_contrast`] と同じ。

use md3_theme::md3_color::{Chroma, Hct, Hue, Tone};
use md3_theme::{Argb, Contrast, DEFAULT_SEED, SchemeVariant, Theme};

/// 検査に使うシード色。
fn seeds() -> Vec<Argb> {
    // 色相を 30° ごとに一周する、彩度の高いシード色。
    let hues = (0..12).map(|step| {
        Hct::new(
            Hue::new(f64::from(step) * 30.0),
            Chroma::new(48.0),
            Tone::new(50.0),
        )
        .to_argb()
    });
    let extremes = [
        DEFAULT_SEED,
        Argb::from_rgb(0x00, 0x00, 0x00),
        Argb::from_rgb(0xFF, 0xFF, 0xFF),
        Argb::from_rgb(0x80, 0x80, 0x80),
        Argb::from_rgb(0xFF, 0x00, 0x00),
        Argb::from_rgb(0x00, 0xFF, 0x00),
        Argb::from_rgb(0x00, 0x00, 0xFF),
        Argb::from_rgb(0xFF, 0xFF, 0x00),
        Argb::from_rgb(0x00, 0xFF, 0xFF),
        Argb::from_rgb(0xFF, 0x00, 0xFF),
    ];
    hues.chain(extremes).collect()
}

#[test]
fn every_generated_scheme_meets_wcag_contrast() {
    let mut failures = Vec::new();
    for seed in seeds() {
        for variant in SchemeVariant::ALL {
            for contrast in Contrast::ALL {
                let theme = Theme::from_seed(seed)
                    .variant(variant)
                    .contrast(contrast)
                    .build();
                for issue in theme.check_contrast() {
                    failures.push(format!(
                        "seed {seed:?} {variant:?} {contrast:?} dark={} {:?} on {:?}: {:.2} < {}",
                        issue.is_dark,
                        issue.foreground,
                        issue.background,
                        issue.ratio,
                        issue.kind.required(),
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
