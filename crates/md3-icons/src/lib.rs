//! Material Symbols の SVG を埋め込んだアイコン集
//!
//! 使うアイコンだけを `assets/` に置き、[`IconName`] から SVG のバイト列を引く。
//! 取得元とバリアントは `assets/SOURCE.md` にある

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// 埋め込んでいるアイコン
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    /// `add`
    Add,
    /// `check`
    Check,
    /// `close`
    Close,
    /// `delete`
    Delete,
    /// `edit`
    Edit,
    /// `favorite`
    Favorite,
    /// `more_vert`
    MoreVert,
    /// `search`
    Search,
    /// `send`
    Send,
    /// `settings`
    Settings,
    /// `star`
    Star,
}

impl IconName {
    /// すべてのアイコン
    pub const ALL: [IconName; 11] = [
        IconName::Add,
        IconName::Check,
        IconName::Close,
        IconName::Delete,
        IconName::Edit,
        IconName::Favorite,
        IconName::MoreVert,
        IconName::Search,
        IconName::Send,
        IconName::Settings,
        IconName::Star,
    ];

    /// Material Symbols での名前 (例: `more_vert`)
    pub const fn name(self) -> &'static str {
        match self {
            IconName::Add => "add",
            IconName::Check => "check",
            IconName::Close => "close",
            IconName::Delete => "delete",
            IconName::Edit => "edit",
            IconName::Favorite => "favorite",
            IconName::MoreVert => "more_vert",
            IconName::Search => "search",
            IconName::Send => "send",
            IconName::Settings => "settings",
            IconName::Star => "star",
        }
    }

    /// SVG のバイト列。24x24 の `viewBox` を持つ単色の図形で、色は描画する側が決める
    pub const fn svg(self) -> &'static [u8] {
        match self {
            IconName::Add => include_bytes!("../assets/add.svg"),
            IconName::Check => include_bytes!("../assets/check.svg"),
            IconName::Close => include_bytes!("../assets/close.svg"),
            IconName::Delete => include_bytes!("../assets/delete.svg"),
            IconName::Edit => include_bytes!("../assets/edit.svg"),
            IconName::Favorite => include_bytes!("../assets/favorite.svg"),
            IconName::MoreVert => include_bytes!("../assets/more_vert.svg"),
            IconName::Search => include_bytes!("../assets/search.svg"),
            IconName::Send => include_bytes!("../assets/send.svg"),
            IconName::Settings => include_bytes!("../assets/settings.svg"),
            IconName::Star => include_bytes!("../assets/star.svg"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn すべてのアイコンは名前と同じファイルの_svg_を持つ() {
        for icon in IconName::ALL {
            let svg = std::str::from_utf8(icon.svg()).expect("SVG は UTF-8 であること");
            assert!(svg.starts_with("<svg"), "{}", icon.name());
            let path = format!("{}/assets/{}.svg", env!("CARGO_MANIFEST_DIR"), icon.name());
            let file = std::fs::read(&path).expect("名前と同じファイルがあること");
            assert_eq!(file, icon.svg(), "{}", icon.name());
        }
    }
}
