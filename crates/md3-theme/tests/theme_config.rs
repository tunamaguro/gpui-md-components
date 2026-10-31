//! テーマの設定を組み立て直しても保つ。

use gpui::px;
use md3_theme::md3_color::ColorRole;
use md3_theme::{Appearance, Argb, Contrast, Density, MotionScheme, SchemeVariant, Theme};

#[test]
fn rebuilding_preserves_all_settings() {
    let seed = Argb::from_rgb(0x12, 0x34, 0x56);
    let override_color = Argb::from_rgb(0xAB, 0xCD, 0xEF);
    let theme = Theme::from_seed(seed)
        .variant(SchemeVariant::Vibrant)
        .contrast(Contrast::High)
        .appearance(Appearance::System)
        .system_is_dark(true)
        .override_color(ColorRole::Primary, override_color)
        .font_family("Test font")
        .type_scale(|scale| scale.body_large.font_size = px(23.0))
        .density(Density::Minus2)
        .text_scale(1.5)
        .shapes(|shapes| shapes.small = px(13.0))
        .motion_scheme(MotionScheme::Expressive)
        .build();
    let rebuilt = theme.to_builder().build();

    assert_eq!(rebuilt.seed(), seed);
    assert_eq!(rebuilt.variant(), SchemeVariant::Vibrant);
    assert_eq!(rebuilt.contrast(), Contrast::High);
    assert_eq!(rebuilt.appearance(), Appearance::System);
    assert!(rebuilt.is_dark());
    assert_eq!(
        rebuilt.font_family().map(|family| family.as_ref()),
        Some("Test font")
    );
    assert_eq!(rebuilt.type_scale(), theme.type_scale());
    assert_eq!(rebuilt.type_scale().body_large.font_size, px(23.0));
    assert_eq!(rebuilt.density(), Density::Minus2);
    assert_eq!(rebuilt.text_scale(), 1.5);
    assert_eq!(rebuilt.shapes(), theme.shapes());
    assert_eq!(rebuilt.shapes().small, px(13.0));
    assert_eq!(rebuilt.motion_scheme(), MotionScheme::Expressive);
    for is_dark in [false, true] {
        for role in ColorRole::ALL {
            assert_eq!(
                rebuilt.argb_for(role, is_dark),
                theme.argb_for(role, is_dark)
            );
        }
        assert_eq!(
            rebuilt.argb_for(ColorRole::Primary, is_dark),
            override_color
        );
    }
    let cleared = rebuilt.to_builder().clear_color_overrides().build();
    assert_ne!(cleared.argb_for(ColorRole::Primary, false), override_color);
    assert_ne!(cleared.argb_for(ColorRole::Primary, true), override_color);
}

#[test]
fn rebuilding_uses_current_appearance_as_system_appearance() {
    for (appearance, is_dark) in [(Appearance::Light, false), (Appearance::Dark, true)] {
        let theme = Theme::default()
            .to_builder()
            .system_is_dark(!is_dark)
            .appearance(appearance)
            .build();
        let rebuilt = theme.to_builder().appearance(Appearance::System).build();
        assert_eq!(rebuilt.is_dark(), is_dark);
    }
}
