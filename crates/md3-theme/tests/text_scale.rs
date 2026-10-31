//! 文字の拡大率。

use md3_theme::{Theme, TypeScale};

#[test]
fn text_scale_multiplies_font_size_and_line_height() {
    let base = Theme::default().text_style(TypeScale::BodyLarge);
    let theme = Theme::default().to_builder().text_scale(2.0).build();
    let scaled = theme.text_style(TypeScale::BodyLarge);
    assert_eq!(theme.text_scale(), 2.0);
    assert_eq!(scaled.font_size, base.font_size * 2.0);
    assert_eq!(scaled.line_height, base.line_height * 2.0);
    assert_eq!(
        theme.to_builder().build().text_scale(),
        2.0,
        "組み立て直しても保つ"
    );
}
