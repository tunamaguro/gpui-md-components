//! Material Design 3 の仕様に基づく配色エンジン。
//!
//! HCT / CAM16 色空間、`TonalPalette`、`DynamicScheme` を純 Rust で提供する（M1 で実装）。
//! gpui に依存せず、panic しない。

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
