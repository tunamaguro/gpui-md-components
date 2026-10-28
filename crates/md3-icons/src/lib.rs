//! Material Symbols（Outlined、24px）の SVG を埋め込んだアイコン集。
//!
//! 使うアイコンだけを `assets/` に置き、[`IconName`] から SVG バイト列を引く。
//! 出典は `assets/SOURCE.md` にある。gpui に依存しない。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// アイコンの名前。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    /// `add`
    Add,
    /// `arrow_back`
    ArrowBack,
    /// `check`
    Check,
    /// `close`
    Close,
    /// `dark_mode`
    DarkMode,
    /// `delete`
    Delete,
    /// `edit`
    Edit,
    /// `expand_more`
    ExpandMore,
    /// `favorite`
    Favorite,
    /// `home`
    Home,
    /// `info`
    Info,
    /// `light_mode`
    LightMode,
    /// `menu`
    Menu,
    /// `more_vert`
    MoreVert,
    /// `palette`
    Palette,
    /// `search`
    Search,
    /// `settings`
    Settings,
    /// `star`
    Star,
}

impl IconName {
    /// すべてのアイコン。
    pub const ALL: [IconName; 18] = [
        IconName::Add,
        IconName::ArrowBack,
        IconName::Check,
        IconName::Close,
        IconName::DarkMode,
        IconName::Delete,
        IconName::Edit,
        IconName::ExpandMore,
        IconName::Favorite,
        IconName::Home,
        IconName::Info,
        IconName::LightMode,
        IconName::Menu,
        IconName::MoreVert,
        IconName::Palette,
        IconName::Search,
        IconName::Settings,
        IconName::Star,
    ];

    /// Material Symbols での名前（例: `arrow_back`）。
    pub const fn name(self) -> &'static str {
        match self {
            IconName::Add => "add",
            IconName::ArrowBack => "arrow_back",
            IconName::Check => "check",
            IconName::Close => "close",
            IconName::DarkMode => "dark_mode",
            IconName::Delete => "delete",
            IconName::Edit => "edit",
            IconName::ExpandMore => "expand_more",
            IconName::Favorite => "favorite",
            IconName::Home => "home",
            IconName::Info => "info",
            IconName::LightMode => "light_mode",
            IconName::Menu => "menu",
            IconName::MoreVert => "more_vert",
            IconName::Palette => "palette",
            IconName::Search => "search",
            IconName::Settings => "settings",
            IconName::Star => "star",
        }
    }

    /// SVG のバイト列（viewBox は `0 -960 960 960`、塗りは単色）。
    pub const fn svg(self) -> &'static [u8] {
        match self {
            IconName::Add => include_bytes!("../assets/add.svg"),
            IconName::ArrowBack => include_bytes!("../assets/arrow_back.svg"),
            IconName::Check => include_bytes!("../assets/check.svg"),
            IconName::Close => include_bytes!("../assets/close.svg"),
            IconName::DarkMode => include_bytes!("../assets/dark_mode.svg"),
            IconName::Delete => include_bytes!("../assets/delete.svg"),
            IconName::Edit => include_bytes!("../assets/edit.svg"),
            IconName::ExpandMore => include_bytes!("../assets/expand_more.svg"),
            IconName::Favorite => include_bytes!("../assets/favorite.svg"),
            IconName::Home => include_bytes!("../assets/home.svg"),
            IconName::Info => include_bytes!("../assets/info.svg"),
            IconName::LightMode => include_bytes!("../assets/light_mode.svg"),
            IconName::Menu => include_bytes!("../assets/menu.svg"),
            IconName::MoreVert => include_bytes!("../assets/more_vert.svg"),
            IconName::Palette => include_bytes!("../assets/palette.svg"),
            IconName::Search => include_bytes!("../assets/search.svg"),
            IconName::Settings => include_bytes!("../assets/settings.svg"),
            IconName::Star => include_bytes!("../assets/star.svg"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_an_svg() {
        for icon in IconName::ALL {
            let svg = icon.svg();
            assert!(svg.starts_with(b"<svg"), "{}", icon.name());
            assert!(svg.ends_with(b"</svg>"), "{}", icon.name());
        }
    }
}
