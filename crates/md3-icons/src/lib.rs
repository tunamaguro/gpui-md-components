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
    /// `arrow_drop_down`
    ArrowDropDown,
    /// `arrow_drop_up`
    ArrowDropUp,
    /// `check`
    Check,
    /// `chevron_right`
    ChevronRight,
    /// `close`
    Close,
    /// `content_copy`
    ContentCopy,
    /// `dark_mode`
    DarkMode,
    /// `delete`
    Delete,
    /// `edit`
    Edit,
    /// `error`
    Error,
    /// `expand_more`
    ExpandMore,
    /// `favorite`
    Favorite,
    /// `home`
    Home,
    /// `image`
    Image,
    /// `inbox`
    Inbox,
    /// `info`
    Info,
    /// `light_mode`
    LightMode,
    /// `mail`
    Mail,
    /// `menu`
    Menu,
    /// `more_vert`
    MoreVert,
    /// `notifications`
    Notifications,
    /// `palette`
    Palette,
    /// `person`
    Person,
    /// `search`
    Search,
    /// `send`
    Send,
    /// `settings`
    Settings,
    /// `share`
    Share,
    /// `star`
    Star,
    /// `star_fill1`（塗りのスター）。
    StarFill1,
}

impl IconName {
    /// すべてのアイコン。
    pub const ALL: [IconName; 31] = [
        IconName::Add,
        IconName::ArrowBack,
        IconName::ArrowDropDown,
        IconName::ArrowDropUp,
        IconName::Check,
        IconName::ChevronRight,
        IconName::Close,
        IconName::ContentCopy,
        IconName::DarkMode,
        IconName::Delete,
        IconName::Edit,
        IconName::Error,
        IconName::ExpandMore,
        IconName::Favorite,
        IconName::Home,
        IconName::Image,
        IconName::Inbox,
        IconName::Info,
        IconName::LightMode,
        IconName::Mail,
        IconName::Menu,
        IconName::MoreVert,
        IconName::Notifications,
        IconName::Palette,
        IconName::Person,
        IconName::Search,
        IconName::Send,
        IconName::Settings,
        IconName::Share,
        IconName::Star,
        IconName::StarFill1,
    ];

    /// Material Symbols での名前（例: `arrow_back`）。
    pub const fn name(self) -> &'static str {
        match self {
            IconName::Add => "add",
            IconName::ArrowBack => "arrow_back",
            IconName::ArrowDropDown => "arrow_drop_down",
            IconName::ArrowDropUp => "arrow_drop_up",
            IconName::Check => "check",
            IconName::ChevronRight => "chevron_right",
            IconName::Close => "close",
            IconName::ContentCopy => "content_copy",
            IconName::DarkMode => "dark_mode",
            IconName::Delete => "delete",
            IconName::Edit => "edit",
            IconName::Error => "error",
            IconName::ExpandMore => "expand_more",
            IconName::Favorite => "favorite",
            IconName::Home => "home",
            IconName::Image => "image",
            IconName::Inbox => "inbox",
            IconName::Info => "info",
            IconName::LightMode => "light_mode",
            IconName::Mail => "mail",
            IconName::Menu => "menu",
            IconName::MoreVert => "more_vert",
            IconName::Notifications => "notifications",
            IconName::Palette => "palette",
            IconName::Person => "person",
            IconName::Search => "search",
            IconName::Send => "send",
            IconName::Settings => "settings",
            IconName::Share => "share",
            IconName::Star => "star",
            IconName::StarFill1 => "star_fill1",
        }
    }

    /// SVG の文字列
    pub const fn svg(self) -> &'static str {
        match self {
            IconName::Add => include_str!("../assets/add.svg"),
            IconName::ArrowBack => include_str!("../assets/arrow_back.svg"),
            IconName::ArrowDropDown => include_str!("../assets/arrow_drop_down.svg"),
            IconName::ArrowDropUp => include_str!("../assets/arrow_drop_up.svg"),
            IconName::Check => include_str!("../assets/check.svg"),
            IconName::ChevronRight => include_str!("../assets/chevron_right.svg"),
            IconName::Close => include_str!("../assets/close.svg"),
            IconName::ContentCopy => include_str!("../assets/content_copy.svg"),
            IconName::DarkMode => include_str!("../assets/dark_mode.svg"),
            IconName::Delete => include_str!("../assets/delete.svg"),
            IconName::Edit => include_str!("../assets/edit.svg"),
            IconName::Error => include_str!("../assets/error.svg"),
            IconName::ExpandMore => include_str!("../assets/expand_more.svg"),
            IconName::Favorite => include_str!("../assets/favorite.svg"),
            IconName::Home => include_str!("../assets/home.svg"),
            IconName::Image => include_str!("../assets/image.svg"),
            IconName::Inbox => include_str!("../assets/inbox.svg"),
            IconName::Info => include_str!("../assets/info.svg"),
            IconName::LightMode => include_str!("../assets/light_mode.svg"),
            IconName::Mail => include_str!("../assets/mail.svg"),
            IconName::Menu => include_str!("../assets/menu.svg"),
            IconName::MoreVert => include_str!("../assets/more_vert.svg"),
            IconName::Notifications => include_str!("../assets/notifications.svg"),
            IconName::Palette => include_str!("../assets/palette.svg"),
            IconName::Person => include_str!("../assets/person.svg"),
            IconName::Search => include_str!("../assets/search.svg"),
            IconName::Send => include_str!("../assets/send.svg"),
            IconName::Settings => include_str!("../assets/settings.svg"),
            IconName::Share => include_str!("../assets/share.svg"),
            IconName::Star => include_str!("../assets/star.svg"),
            IconName::StarFill1 => include_str!("../assets/star_fill1.svg"),
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
            assert!(svg.starts_with("<svg"), "{}", icon.name());
            assert!(svg.ends_with("</svg>"), "{}", icon.name());
        }
    }
}
