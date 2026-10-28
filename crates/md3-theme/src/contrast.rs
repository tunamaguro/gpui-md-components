//! テーマのコントラスト検査（WCAG 2.2）。

use md3_color::contrast::ratio_of_tones;
use md3_color::{Argb, ColorRole};

/// 検査の種類と基準。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContrastKind {
    /// 本文の文字（4.5:1）。
    Text,
    /// UI 部品の境界や図形（3:1）。
    Ui,
}

impl ContrastKind {
    /// 必要なコントラスト比。
    pub const fn required(self) -> f64 {
        match self {
            ContrastKind::Text => 4.5,
            ContrastKind::Ui => 3.0,
        }
    }
}

/// 基準を下回ったロールの組。
#[derive(Clone, Debug, PartialEq)]
pub struct ContrastIssue {
    /// ダークの配色で見つかったか。
    pub is_dark: bool,
    /// 前景のロール。
    pub foreground: ColorRole,
    /// 背景のロール。
    pub background: ColorRole,
    /// 検査の種類。
    pub kind: ContrastKind,
    /// 実際のコントラスト比。
    pub ratio: f64,
}

use ColorRole::*;

/// 前景と背景の組。MD3 で前景が背景の上に置かれる組み合わせ。
///
/// Primary / Secondary / Tertiary / Error は Text ボタン・タブ・エラー文などで文字色にもなるので、
/// 面の上では文字の基準（4.5:1）で検査する。
const PAIRS: &[(ColorRole, ColorRole, ContrastKind)] = &[
    (OnPrimary, Primary, ContrastKind::Text),
    (OnPrimaryContainer, PrimaryContainer, ContrastKind::Text),
    (OnSecondary, Secondary, ContrastKind::Text),
    (OnSecondaryContainer, SecondaryContainer, ContrastKind::Text),
    (OnTertiary, Tertiary, ContrastKind::Text),
    (OnTertiaryContainer, TertiaryContainer, ContrastKind::Text),
    (OnError, Error, ContrastKind::Text),
    (OnErrorContainer, ErrorContainer, ContrastKind::Text),
    (OnBackground, Background, ContrastKind::Text),
    (OnSurface, SurfaceDim, ContrastKind::Text),
    (OnSurface, SurfaceBright, ContrastKind::Text),
    (OnSurfaceVariant, SurfaceDim, ContrastKind::Text),
    (OnSurfaceVariant, SurfaceBright, ContrastKind::Text),
    (InverseOnSurface, InverseSurface, ContrastKind::Text),
    (InversePrimary, InverseSurface, ContrastKind::Ui),
    (OnPrimaryFixed, PrimaryFixed, ContrastKind::Text),
    (OnPrimaryFixed, PrimaryFixedDim, ContrastKind::Text),
    (OnPrimaryFixedVariant, PrimaryFixed, ContrastKind::Text),
    (OnPrimaryFixedVariant, PrimaryFixedDim, ContrastKind::Text),
    (OnSecondaryFixed, SecondaryFixed, ContrastKind::Text),
    (OnSecondaryFixed, SecondaryFixedDim, ContrastKind::Text),
    (OnSecondaryFixedVariant, SecondaryFixed, ContrastKind::Text),
    (
        OnSecondaryFixedVariant,
        SecondaryFixedDim,
        ContrastKind::Text,
    ),
    (OnTertiaryFixed, TertiaryFixed, ContrastKind::Text),
    (OnTertiaryFixed, TertiaryFixedDim, ContrastKind::Text),
    (OnTertiaryFixedVariant, TertiaryFixed, ContrastKind::Text),
    (OnTertiaryFixedVariant, TertiaryFixedDim, ContrastKind::Text),
    (Primary, SurfaceDim, ContrastKind::Text),
    (Primary, SurfaceBright, ContrastKind::Text),
    (Secondary, SurfaceDim, ContrastKind::Text),
    (Secondary, SurfaceBright, ContrastKind::Text),
    (Tertiary, SurfaceDim, ContrastKind::Text),
    (Tertiary, SurfaceBright, ContrastKind::Text),
    (Error, SurfaceDim, ContrastKind::Text),
    (Error, SurfaceBright, ContrastKind::Text),
    (Outline, SurfaceDim, ContrastKind::Ui),
    (Outline, SurfaceBright, ContrastKind::Ui),
];

/// `color` で引いた配色の組を検査する。
pub(crate) fn check(is_dark: bool, color: impl Fn(ColorRole) -> Argb) -> Vec<ContrastIssue> {
    PAIRS
        .iter()
        .filter_map(|&(foreground, background, kind)| {
            let ratio = ratio_of_tones(color(foreground).lstar(), color(background).lstar());
            (ratio < kind.required()).then_some(ContrastIssue {
                is_dark,
                foreground,
                background,
                kind,
                ratio,
            })
        })
        .collect()
}
