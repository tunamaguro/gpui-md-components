//! examples 以下のクレートで共有するアセットと初期化処理。

use std::borrow::Cow;

use gpui::App;

/// 同梱している IBM Plex Sans JP のライセンス文 (SIL Open Font License 1.1)。
pub const FONT_LICENSE: &str = include_str!("../assets/OFL.txt");

/// [`load_fonts`] で登録されるフォントのファミリー名。
pub const FONT_FAMILY: &str = "IBM Plex Sans JP";

/// IBM Plex Sans JP の Regular / Medium / Bold を `cx` のテキストシステムに登録する。
///
/// 登録後は [`FONT_FAMILY`] をファミリー名に指定して使える。フォントを埋め込んだバイナリには
/// [`FONT_LICENSE`] も含まれる。
///
/// # Panics
///
/// フォントの読み込みに失敗した場合。
pub fn load_fonts(cx: &App) {
    // OFL はフォントとともにライセンス文を配布することを求めるので、未使用として削られないようにする。
    std::hint::black_box(FONT_LICENSE);
    let fonts = [
        include_bytes!("../assets/IBMPlexSansJP-Regular.ttf").as_slice(),
        include_bytes!("../assets/IBMPlexSansJP-Medium.ttf").as_slice(),
        include_bytes!("../assets/IBMPlexSansJP-Bold.ttf").as_slice(),
    ];
    cx.text_system()
        .add_fonts(fonts.into_iter().map(Cow::Borrowed).collect())
        .expect("failed to load fonts");
}
