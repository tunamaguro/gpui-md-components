// Ported from Material Web (https://github.com/material-components/material-web)
// Source: tokens/versions/latest/sass/_md-comp-button.scss, _md-comp-button-text.scss,
//         _md-comp-icon-button-*.scss, _md-comp-checkbox.scss, _md-comp-switch.scss
//         (Design system: Google Material 3, Version 34.0.21)
//         @ de5470f7ed6a653457ca8f5cc0e7f7602c7a8391
// Copyright 2026 Google LLC. Licensed under the Apache License, Version 2.0.

//! Compose の生成トークン (`component`) に無いか、Compose の実装がトークンを上書きしている値
//!
//! Material Web の Expressive 世代のトークンから転記している

/// ボタン (`md.comp.button.*`)
pub mod button {
    use crate::motion::MotionSchemeKey;

    /// 押下中の形へ角の大きさを変えるときのバネ
    /// (`md.comp.button.<size>.pressed.container.corner-size.motion.spring`)。
    /// XS〜XL のすべての大きさで同じ
    ///
    /// Compose の `Button.kt` は跳ねを避けるため `DefaultEffects` を使っている
    pub const PRESSED_CONTAINER_CORNER_SIZE_MOTION: MotionSchemeKey = MotionSchemeKey::FastSpatial;
}

/// テキストボタン (`md.comp.button.text.*`)
pub mod text_button {
    use md3_color::ColorRole;

    /// ラベル・アイコン・状態レイヤーの色 (`md.comp.button.text.label-text.color`)
    ///
    /// Compose の `TextButtonTokens.LabelColor` (v0_11_0) は `OnSurfaceVariant` だが、
    /// `ButtonDefaults.textButtonColors` は `Primary` を使っている
    pub const LABEL_TEXT_COLOR: ColorRole = ColorRole::Primary;
}

/// アイコンボタン (`md.comp.icon-button.*`)
pub mod icon_button {
    use crate::motion::MotionSchemeKey;

    /// 押下中の形へ角の大きさを変えるときのバネ
    /// (`md.comp.icon-button.<size>.pressed.container.corner-size.motion.spring`)。
    /// XS〜XL のすべての大きさで同じ
    pub const PRESSED_CONTAINER_CORNER_SIZE_MOTION: MotionSchemeKey = MotionSchemeKey::FastSpatial;
}

/// チェックボックスの状態レイヤー (`md.comp.checkbox.*.state-layer.color`)
pub mod checkbox {
    use md3_color::ColorRole;

    /// 選択中の hover とフォーカスの色
    pub const SELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
    /// 選択中の押下の色
    pub const SELECTED_PRESSED_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
    /// 未選択の hover とフォーカスの色
    pub const UNSELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
    /// 未選択の押下の色
    pub const UNSELECTED_PRESSED_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
    /// エラーのときの hover・フォーカス・押下の色
    pub const ERROR_STATE_LAYER_COLOR: ColorRole = ColorRole::Error;
}

/// スイッチの状態レイヤー (`md.comp.switch.*.state-layer.color`)
pub mod switch {
    use md3_color::ColorRole;

    /// 選択中の hover・フォーカス・押下の色
    pub const SELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::Primary;
    /// 未選択の hover・フォーカス・押下の色
    pub const UNSELECTED_STATE_LAYER_COLOR: ColorRole = ColorRole::OnSurface;
}
