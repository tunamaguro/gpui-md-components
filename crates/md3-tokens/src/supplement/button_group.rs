// Ported from Jetpack Compose Material 3 (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt
//         (`ButtonGroupDefaults.ExpandedRatio`、`EnlargeOnPressNode`) @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2024 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! ボタングループの値のうち、Compose の実装が固定値として持つもの

/// 押したボタンが広がる幅の、ボタンの幅に対する割合
///
/// 端のボタンはこの割合だけ隣を縮め、間のボタンは左右の隣をこの割合の半分ずつ縮める
pub const EXPANDED_RATIO: f32 = 0.15;

/// 押すのをやめたときに、元の幅へ戻り始めるまでに待つ押下の動きの進み
///
/// 短く押しても広がる動きが見えるよう、押下の動きがここまで進んでから戻す
pub const RELEASE_PROGRESS: f32 = 0.75;
