// Ported from material-web (https://github.com/material-components/material-web)
// Source: tokens/versions/latest/sass/_md-comp-sheet-side.scss @ 86cb0d48 (Material 3 tokens 34.0.21)
// Copyright 2026 Google LLC. Licensed under the Apache License, Version 2.0.

//! サイドシートのトークン。Compose に tokens がないため material-web から転記した。
//! 値の出典は `docs/TOKENS_SOURCE.md`。

use crate::elevation;
use crate::shape::Shape;
use crate::typography::TypeScale;
use md3_color::ColorRole;

/// `docked-container-width`
pub const DOCKED_CONTAINER_WIDTH: f32 = 256.0;
/// `docked-divider-color`
pub const DOCKED_DIVIDER_COLOR: ColorRole = ColorRole::Outline;
/// `docked-headline-color`
pub const DOCKED_HEADLINE_COLOR: ColorRole = ColorRole::OnSurfaceVariant;
/// `docked-headline-font`
pub const DOCKED_HEADLINE_FONT: TypeScale = TypeScale::TitleLarge;
/// `docked-modal-container-color`
pub const DOCKED_MODAL_CONTAINER_COLOR: ColorRole = ColorRole::SurfaceContainerLow;
/// `docked-modal-container-elevation`
pub const DOCKED_MODAL_CONTAINER_ELEVATION: f32 = elevation::LEVEL1;
/// `docked-modal-container-shape`（開始側の 2 つの角だけに適用する）
pub const DOCKED_MODAL_CONTAINER_SHAPE: Shape = Shape::Large;
/// `docked-standard-container-color`
pub const DOCKED_STANDARD_CONTAINER_COLOR: ColorRole = ColorRole::Surface;
/// `docked-standard-container-elevation`
pub const DOCKED_STANDARD_CONTAINER_ELEVATION: f32 = elevation::LEVEL0;
