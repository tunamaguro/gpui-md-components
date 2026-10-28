//! 公開 API で使う単位の newtype。
//!
//! 移植元と計算結果を完全一致させるため、内部の精度は `f64` のまま保持する。

use crate::utils::{color, math};

/// 不透明度を含む 32bit の色（`0xAARRGGBB`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Argb(u32);

impl Argb {
    /// `0xAARRGGBB` 形式の値から作る。
    pub const fn new(argb: u32) -> Self {
        Self(argb)
    }

    /// 8bit の RGB から不透明な色を作る。
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(0xff00_0000 | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// `0xAARRGGBB` 形式の値。
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// 不透明度（0〜255）。
    pub const fn alpha(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// 赤（0〜255）。
    pub const fn red(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// 緑（0〜255）。
    pub const fn green(self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// 青（0〜255）。
    pub const fn blue(self) -> u8 {
        self.0 as u8
    }

    /// L*a*b* の明度 L*。HCT のトーンと同じ値。
    pub fn lstar(self) -> Tone {
        Tone::new(color::lstar_from_argb(self.0))
    }
}

impl From<u32> for Argb {
    fn from(argb: u32) -> Self {
        Self(argb)
    }
}

impl From<Argb> for u32 {
    fn from(argb: Argb) -> Self {
        argb.0
    }
}

/// 色相（度）。`[0, 360)` に正規化される。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Hue(f64);

impl Hue {
    /// 角度から作る。範囲外の値は `[0, 360)` に回し込み、NaN は 0 にする。
    pub fn new(degrees: f64) -> Self {
        if degrees.is_nan() {
            Self(0.0)
        } else {
            Self(math::sanitize_degrees_double(degrees))
        }
    }

    /// 角度（度）。
    pub const fn degrees(self) -> f64 {
        self.0
    }

    /// 正規化せずにそのまま包む。移植元と同じ値を保つため内部でだけ使う。
    pub(crate) const fn raw(value: f64) -> Self {
        Self(value)
    }
}

/// 彩度（CAM16 の chroma）。0 以上。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Chroma(f64);

impl Chroma {
    /// 値から作る。負の値と NaN は 0 にする。
    pub fn new(chroma: f64) -> Self {
        if chroma.is_nan() || chroma < 0.0 {
            Self(0.0)
        } else {
            Self(chroma)
        }
    }

    /// 値。
    pub const fn value(self) -> f64 {
        self.0
    }

    /// 正規化せずにそのまま包む。移植元と同じ値を保つため内部でだけ使う。
    pub(crate) const fn raw(value: f64) -> Self {
        Self(value)
    }
}

/// トーン（L*）。`[0, 100]` に丸められる。
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Tone(f64);

impl Tone {
    /// 値から作る。範囲外の値は `[0, 100]` に丸め、NaN は 0 にする。
    pub fn new(tone: f64) -> Self {
        if tone.is_nan() {
            Self(0.0)
        } else {
            Self(math::clamp_double(0.0, 100.0, tone))
        }
    }

    /// 値。
    pub const fn value(self) -> f64 {
        self.0
    }

    /// 正規化せずにそのまま包む。移植元と同じ値を保つため内部でだけ使う。
    pub(crate) const fn raw(value: f64) -> Self {
        Self(value)
    }
}
