// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/dynamiccolor/material_dynamic_colors.ts (MaterialDynamicColors)
//         @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2022 Google LLC. Licensed under the Apache License, Version 2.0.

/// 配色の色の役割
///
/// 移植元の `MaterialDynamicColors` が提供する色の一覧に対応する。名前は [`ColorRole::name`] で得る
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorRole {
    /// 主色のパレットのキーカラー
    PrimaryPaletteKeyColor,
    /// 第二色のパレットのキーカラー
    SecondaryPaletteKeyColor,
    /// 第三色のパレットのキーカラー
    TertiaryPaletteKeyColor,
    /// 中間色のパレットのキーカラー
    NeutralPaletteKeyColor,
    /// 中間色バリアントのパレットのキーカラー
    NeutralVariantPaletteKeyColor,
    /// エラー色のパレットのキーカラー
    ErrorPaletteKeyColor,
    /// 背景
    Background,
    /// 背景の上の文字・アイコン
    OnBackground,
    /// サーフェス
    Surface,
    /// 暗いサーフェス
    SurfaceDim,
    /// 明るいサーフェス
    SurfaceBright,
    /// 最も低いサーフェスコンテナ
    SurfaceContainerLowest,
    /// 低いサーフェスコンテナ
    SurfaceContainerLow,
    /// サーフェスコンテナ
    SurfaceContainer,
    /// 高いサーフェスコンテナ
    SurfaceContainerHigh,
    /// 最も高いサーフェスコンテナ
    SurfaceContainerHighest,
    /// サーフェスの上の文字・アイコン
    OnSurface,
    /// サーフェスバリアント
    SurfaceVariant,
    /// サーフェスの上の控えめな文字・アイコン
    OnSurfaceVariant,
    /// 反転したサーフェス
    InverseSurface,
    /// 反転したサーフェスの上の文字・アイコン
    InverseOnSurface,
    /// 輪郭線
    Outline,
    /// 控えめな輪郭線
    OutlineVariant,
    /// 影
    Shadow,
    /// スクリム
    Scrim,
    /// サーフェスの色味
    SurfaceTint,
    /// 主色
    Primary,
    /// 暗い主色
    PrimaryDim,
    /// 主色の上の文字・アイコン
    OnPrimary,
    /// 主色のコンテナ
    PrimaryContainer,
    /// 主色のコンテナの上の文字・アイコン
    OnPrimaryContainer,
    /// 反転したサーフェスの上の主色
    InversePrimary,
    /// 第二色
    Secondary,
    /// 暗い第二色
    SecondaryDim,
    /// 第二色の上の文字・アイコン
    OnSecondary,
    /// 第二色のコンテナ
    SecondaryContainer,
    /// 第二色のコンテナの上の文字・アイコン
    OnSecondaryContainer,
    /// 第三色
    Tertiary,
    /// 暗い第三色
    TertiaryDim,
    /// 第三色の上の文字・アイコン
    OnTertiary,
    /// 第三色のコンテナ
    TertiaryContainer,
    /// 第三色のコンテナの上の文字・アイコン
    OnTertiaryContainer,
    /// エラー色
    Error,
    /// 暗いエラー色
    ErrorDim,
    /// エラー色の上の文字・アイコン
    OnError,
    /// エラー色のコンテナ
    ErrorContainer,
    /// エラー色のコンテナの上の文字・アイコン
    OnErrorContainer,
    /// ライトとダークで共通の主色
    PrimaryFixed,
    /// ライトとダークで共通の暗い主色
    PrimaryFixedDim,
    /// 共通の主色の上の文字・アイコン
    OnPrimaryFixed,
    /// 共通の主色の上の控えめな文字・アイコン
    OnPrimaryFixedVariant,
    /// ライトとダークで共通の第二色
    SecondaryFixed,
    /// ライトとダークで共通の暗い第二色
    SecondaryFixedDim,
    /// 共通の第二色の上の文字・アイコン
    OnSecondaryFixed,
    /// 共通の第二色の上の控えめな文字・アイコン
    OnSecondaryFixedVariant,
    /// ライトとダークで共通の第三色
    TertiaryFixed,
    /// ライトとダークで共通の暗い第三色
    TertiaryFixedDim,
    /// 共通の第三色の上の文字・アイコン
    OnTertiaryFixed,
    /// 共通の第三色の上の控えめな文字・アイコン
    OnTertiaryFixedVariant,
}

impl ColorRole {
    /// すべての役割。移植元の `ColorSpecDelegate` の宣言順
    pub const ALL: [ColorRole; 59] = [
        ColorRole::PrimaryPaletteKeyColor,
        ColorRole::SecondaryPaletteKeyColor,
        ColorRole::TertiaryPaletteKeyColor,
        ColorRole::NeutralPaletteKeyColor,
        ColorRole::NeutralVariantPaletteKeyColor,
        ColorRole::ErrorPaletteKeyColor,
        ColorRole::Background,
        ColorRole::OnBackground,
        ColorRole::Surface,
        ColorRole::SurfaceDim,
        ColorRole::SurfaceBright,
        ColorRole::SurfaceContainerLowest,
        ColorRole::SurfaceContainerLow,
        ColorRole::SurfaceContainer,
        ColorRole::SurfaceContainerHigh,
        ColorRole::SurfaceContainerHighest,
        ColorRole::OnSurface,
        ColorRole::SurfaceVariant,
        ColorRole::OnSurfaceVariant,
        ColorRole::InverseSurface,
        ColorRole::InverseOnSurface,
        ColorRole::Outline,
        ColorRole::OutlineVariant,
        ColorRole::Shadow,
        ColorRole::Scrim,
        ColorRole::SurfaceTint,
        ColorRole::Primary,
        ColorRole::PrimaryDim,
        ColorRole::OnPrimary,
        ColorRole::PrimaryContainer,
        ColorRole::OnPrimaryContainer,
        ColorRole::InversePrimary,
        ColorRole::Secondary,
        ColorRole::SecondaryDim,
        ColorRole::OnSecondary,
        ColorRole::SecondaryContainer,
        ColorRole::OnSecondaryContainer,
        ColorRole::Tertiary,
        ColorRole::TertiaryDim,
        ColorRole::OnTertiary,
        ColorRole::TertiaryContainer,
        ColorRole::OnTertiaryContainer,
        ColorRole::Error,
        ColorRole::ErrorDim,
        ColorRole::OnError,
        ColorRole::ErrorContainer,
        ColorRole::OnErrorContainer,
        ColorRole::PrimaryFixed,
        ColorRole::PrimaryFixedDim,
        ColorRole::OnPrimaryFixed,
        ColorRole::OnPrimaryFixedVariant,
        ColorRole::SecondaryFixed,
        ColorRole::SecondaryFixedDim,
        ColorRole::OnSecondaryFixed,
        ColorRole::OnSecondaryFixedVariant,
        ColorRole::TertiaryFixed,
        ColorRole::TertiaryFixedDim,
        ColorRole::OnTertiaryFixed,
        ColorRole::OnTertiaryFixedVariant,
    ];

    /// [`ColorRole::ALL`] での位置。役割ごとの値を配列に持つときの添字に使う
    pub const fn index(self) -> usize {
        self as usize
    }

    /// 移植元の `DynamicColor.name` と同じ snake_case の名前
    pub const fn name(self) -> &'static str {
        match self {
            ColorRole::PrimaryPaletteKeyColor => "primary_palette_key_color",
            ColorRole::SecondaryPaletteKeyColor => "secondary_palette_key_color",
            ColorRole::TertiaryPaletteKeyColor => "tertiary_palette_key_color",
            ColorRole::NeutralPaletteKeyColor => "neutral_palette_key_color",
            ColorRole::NeutralVariantPaletteKeyColor => "neutral_variant_palette_key_color",
            ColorRole::ErrorPaletteKeyColor => "error_palette_key_color",
            ColorRole::Background => "background",
            ColorRole::OnBackground => "on_background",
            ColorRole::Surface => "surface",
            ColorRole::SurfaceDim => "surface_dim",
            ColorRole::SurfaceBright => "surface_bright",
            ColorRole::SurfaceContainerLowest => "surface_container_lowest",
            ColorRole::SurfaceContainerLow => "surface_container_low",
            ColorRole::SurfaceContainer => "surface_container",
            ColorRole::SurfaceContainerHigh => "surface_container_high",
            ColorRole::SurfaceContainerHighest => "surface_container_highest",
            ColorRole::OnSurface => "on_surface",
            ColorRole::SurfaceVariant => "surface_variant",
            ColorRole::OnSurfaceVariant => "on_surface_variant",
            ColorRole::InverseSurface => "inverse_surface",
            ColorRole::InverseOnSurface => "inverse_on_surface",
            ColorRole::Outline => "outline",
            ColorRole::OutlineVariant => "outline_variant",
            ColorRole::Shadow => "shadow",
            ColorRole::Scrim => "scrim",
            ColorRole::SurfaceTint => "surface_tint",
            ColorRole::Primary => "primary",
            ColorRole::PrimaryDim => "primary_dim",
            ColorRole::OnPrimary => "on_primary",
            ColorRole::PrimaryContainer => "primary_container",
            ColorRole::OnPrimaryContainer => "on_primary_container",
            ColorRole::InversePrimary => "inverse_primary",
            ColorRole::Secondary => "secondary",
            ColorRole::SecondaryDim => "secondary_dim",
            ColorRole::OnSecondary => "on_secondary",
            ColorRole::SecondaryContainer => "secondary_container",
            ColorRole::OnSecondaryContainer => "on_secondary_container",
            ColorRole::Tertiary => "tertiary",
            ColorRole::TertiaryDim => "tertiary_dim",
            ColorRole::OnTertiary => "on_tertiary",
            ColorRole::TertiaryContainer => "tertiary_container",
            ColorRole::OnTertiaryContainer => "on_tertiary_container",
            ColorRole::Error => "error",
            ColorRole::ErrorDim => "error_dim",
            ColorRole::OnError => "on_error",
            ColorRole::ErrorContainer => "error_container",
            ColorRole::OnErrorContainer => "on_error_container",
            ColorRole::PrimaryFixed => "primary_fixed",
            ColorRole::PrimaryFixedDim => "primary_fixed_dim",
            ColorRole::OnPrimaryFixed => "on_primary_fixed",
            ColorRole::OnPrimaryFixedVariant => "on_primary_fixed_variant",
            ColorRole::SecondaryFixed => "secondary_fixed",
            ColorRole::SecondaryFixedDim => "secondary_fixed_dim",
            ColorRole::OnSecondaryFixed => "on_secondary_fixed",
            ColorRole::OnSecondaryFixedVariant => "on_secondary_fixed_variant",
            ColorRole::TertiaryFixed => "tertiary_fixed",
            ColorRole::TertiaryFixedDim => "tertiary_fixed_dim",
            ColorRole::OnTertiaryFixed => "on_tertiary_fixed",
            ColorRole::OnTertiaryFixedVariant => "on_tertiary_fixed_variant",
        }
    }
}
