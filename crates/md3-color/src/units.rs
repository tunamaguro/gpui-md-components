//! 公開 API で使う色の単位

use crate::utils::{color, math};

/// 不透明度を含む 32 bit の色 (`0xAARRGGBB`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Argb(u32);

impl Argb {
    /// `0xAARRGGBB` 形式の値から作る
    pub const fn new(argb: u32) -> Self {
        Self(argb)
    }

    /// 8 bit の RGB から不透明な色を作る
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(0xff00_0000 | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// `0xAARRGGBB` 形式の値
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// 不透明度 (0〜255)
    pub const fn alpha(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// 赤 (0〜255)
    pub const fn red(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// 緑 (0〜255)
    pub const fn green(self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// 青 (0〜255)
    pub const fn blue(self) -> u8 {
        self.0 as u8
    }

    /// 不透明度が 255 なら `true`
    pub const fn is_opaque(self) -> bool {
        self.alpha() == 255
    }

    /// L*a*b* の明度 L*。HCT のトーンと同じ尺度で、不透明度は無視する
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

/// 色相 (度)。常に `[0, 360)` に収まる
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Hue(f64);

impl Hue {
    /// 角度から作る。範囲外の値は `[0, 360)` に回し込み、NaN と無限大は 0 にする
    pub fn new(degrees: f64) -> Self {
        if degrees.is_finite() {
            Self(math::sanitize_degrees_double(degrees))
        } else {
            Self(0.0)
        }
    }

    /// 角度 (度)
    pub const fn degrees(self) -> f64 {
        self.0
    }
}

/// HCT の彩度。0 以上の有限値に収まる
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Chroma(f64);

impl Chroma {
    /// 値から作る。負の値と NaN は 0、正の無限大は `f64::MAX` にする
    ///
    /// 彩度に上限はない。色域を超える値を指定した HCT は、色域内で最も近い色になる
    pub fn new(chroma: f64) -> Self {
        if chroma.is_nan() {
            Self(0.0)
        } else {
            Self(chroma.clamp(0.0, f64::MAX))
        }
    }

    /// 値
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// HCT のトーン (L*)。`[0, 100]` に収まり、0 が黒、100 が白
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Tone(f64);

impl Tone {
    /// 値から作る。`[0, 100]` に丸め、NaN は 0 にする
    pub fn new(tone: f64) -> Self {
        if tone.is_nan() {
            Self(0.0)
        } else {
            Self(tone.clamp(0.0, 100.0))
        }
    }

    /// 値
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// 配色のコントラストレベル。`[-1, 1]` に収まり、0 が標準
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ContrastLevel(f64);

impl ContrastLevel {
    /// 低コントラスト (-1)
    pub const REDUCED: Self = Self(-1.0);
    /// 標準 (0)
    pub const STANDARD: Self = Self(0.0);
    /// 中コントラスト (0.5)
    pub const MEDIUM: Self = Self(0.5);
    /// 高コントラスト (1)
    pub const HIGH: Self = Self(1.0);

    /// 値から作る。`[-1, 1]` に丸め、NaN は 0 にする
    ///
    /// 配色の計算は -1 以下を -1、1 以上を 1 と同じに扱うため、丸めても結果は変わらない
    pub fn new(level: f64) -> Self {
        if level.is_nan() {
            Self::STANDARD
        } else {
            Self(level.clamp(-1.0, 1.0))
        }
    }

    /// 値
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl Default for ContrastLevel {
    fn default() -> Self {
        Self::STANDARD
    }
}
