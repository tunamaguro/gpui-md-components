//! 進捗を表す部品が共有する進み具合の型

use gpui_headless::progress::Progress as HeadlessProgress;

/// 進捗インジケーターとローディングインジケーターが表す進み具合
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Progress {
    /// 進み具合が分からない。表示が動き続け、モーション低減の設定では止まる
    Indeterminate,
    /// 全体のうち進んだ割合。0.0 で未着手、1.0 で完了
    ///
    /// 0.0〜1.0 の外の値はその範囲に収め、NaN は 0.0 として扱う
    Determinate(f32),
}

impl From<Progress> for HeadlessProgress {
    fn from(value: Progress) -> Self {
        match value {
            Progress::Indeterminate => HeadlessProgress::Indeterminate,
            Progress::Determinate(fraction) => HeadlessProgress::Determinate(fraction),
        }
    }
}
