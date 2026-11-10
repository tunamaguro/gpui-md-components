//! Material Symbols の SVG を埋め込んだアイコン集
//!
//! すべての Material Symbols を、Material Symbols での名前を大文字にした定数で提供する
//! (例: `arrow_back` は [`ARROW_BACK`])。数字で始まる名前は先頭に `N` を付ける (例: `10k` は [`N10K`])。
//! 各アイコンは、ウェイト 400、グレード 0、サイズ 24px の SVG を、スタイルと塗りの有無の組み合わせごとに持つ
//!
//! 実行ファイルには、使ったアイコンの SVG だけが入る。[`ALL`] を使うと、すべてのアイコンの SVG が入る
//!
//! ```
//! use md3_icons::{HOME, IconStyle};
//!
//! let svg = HOME.svg(IconStyle::Outlined, false);
//! assert!(svg.starts_with("<svg"));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod generated;

pub use generated::*;

/// アイコンのスタイル
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconStyle {
    /// Outlined
    #[default]
    Outlined,
    /// Rounded
    Rounded,
    /// Sharp
    Sharp,
}

/// Material Symbols のアイコン 1 つ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Icon {
    name: &'static str,
    /// `[スタイル][塗り]` の SVG
    svgs: [[&'static str; 2]; 3],
}

impl Icon {
    /// Material Symbols での名前
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// `style` のアイコンの SVG。`filled` が `true` なら塗りのある形を返す
    ///
    /// 幅と高さは 24 で、色は指定していない
    pub const fn svg(self, style: IconStyle, filled: bool) -> &'static str {
        self.svgs[style as usize][filled as usize]
    }
}

/// スタイルと塗りの有無を決めたアイコン
///
/// [`Icon`] から変換すると、[`IconStyle::Outlined`] の塗りのない形になる
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyledIcon {
    icon: Icon,
    style: IconStyle,
    filled: bool,
}

impl StyledIcon {
    /// `icon` を [`IconStyle::Outlined`] の塗りのない形で使う
    pub const fn new(icon: Icon) -> Self {
        Self {
            icon,
            style: IconStyle::Outlined,
            filled: false,
        }
    }

    /// スタイルを `style` にする
    pub const fn style(mut self, style: IconStyle) -> Self {
        self.style = style;
        self
    }

    /// `filled` が `true` なら塗りのある形にする
    pub const fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// 決めたスタイルと塗りの SVG。幅と高さは 24 で、色は指定していない
    pub const fn svg(self) -> &'static str {
        self.icon.svg(self.style, self.filled)
    }
}

impl From<Icon> for StyledIcon {
    fn from(icon: Icon) -> Self {
        Self::new(icon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STYLES: [IconStyle; 3] = [IconStyle::Outlined, IconStyle::Rounded, IconStyle::Sharp];

    #[test]
    fn すべての組み合わせが_24px_の_svg() {
        for icon in ALL {
            for style in STYLES {
                for filled in [false, true] {
                    let text = icon.svg(style, filled);
                    assert!(
                        text.starts_with("<svg") && text.ends_with("</svg>"),
                        "{} {style:?} {filled}",
                        icon.name()
                    );
                    assert!(
                        text.contains(r#"width="24""#) && text.contains(r#"height="24""#),
                        "{} {style:?} {filled}",
                        icon.name()
                    );
                }
            }
        }
    }

    #[test]
    fn 名前と資産が上流のファイルに対応する() {
        assert_eq!(HOME.name(), "home");
        assert_eq!(N10K.name(), "10k");
        assert_eq!(
            HOME.svg(IconStyle::Outlined, false),
            include_str!("../assets/materialsymbolsoutlined/home_24px.svg")
        );
        assert_eq!(
            HOME.svg(IconStyle::Sharp, true),
            include_str!("../assets/materialsymbolssharp/home_fill1_24px.svg")
        );
        assert_ne!(
            HOME.svg(IconStyle::Rounded, false),
            HOME.svg(IconStyle::Rounded, true)
        );
    }

    #[test]
    fn styled_icon_は決めたスタイルと塗りの_svg_を返す() {
        assert_eq!(
            StyledIcon::from(HOME).svg(),
            HOME.svg(IconStyle::Outlined, false),
            "既定は Outlined の塗りなし"
        );
        assert_eq!(
            StyledIcon::new(HOME)
                .style(IconStyle::Sharp)
                .filled(true)
                .svg(),
            HOME.svg(IconStyle::Sharp, true)
        );
    }
}
