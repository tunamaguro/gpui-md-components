//! Material Design 3 の仕様に基づくデザイントークン
//!
//! 値の多くは Jetpack Compose Material3 の生成トークン (`tokens/*Tokens.kt`) を `cargo xtask gen-tokens` で
//! 転記したもので、各ファイルの先頭に出典とコミット、トークンの版を記録している。
//! Compose が値を持たない影・フォーカスリング・リップルは Material Web、ウィンドウのサイズ区分は
//! Jetpack Window、密度は Material Design 3 の仕様から取り、各モジュールに出典を記録している。
//! 色・形・書体はロールやキーとして参照を保ち、解決済みの描画値には置き換えない。
//! 寸法は [`Dp`] / [`Sp`] の論理単位で持ち、GPUI の型への変換は扱わない

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod generated;

pub mod density;
pub mod elevation;
pub mod focus_ring;
pub mod motion;
pub mod ripple;
pub mod shadow;
pub mod shape;
pub mod state;
pub mod typography;
mod units;
pub mod window;

pub use generated::component;
pub use units::{Dp, Sp};
