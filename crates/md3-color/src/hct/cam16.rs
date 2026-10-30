// Ported from material-color-utilities: typescript/hct/cam16.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3. See NOTICE.

use std::f64::consts::PI;

use super::ViewingConditions;
use crate::units::Argb;
use crate::utils::{color, math};

/// CAM16 色の見えモデルによる色の表現。
///
/// HCT の色相・彩度はこのモデルの値を使う。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cam16 {
    hue: f64,
    chroma: f64,
    j: f64,
    q: f64,
    m: f64,
    s: f64,
    jstar: f64,
    astar: f64,
    bstar: f64,
}

impl Cam16 {
    /// 色相（度）。
    pub fn hue(&self) -> f64 {
        self.hue
    }

    /// 彩度（chroma）。
    pub fn chroma(&self) -> f64 {
        self.chroma
    }

    /// 明度（lightness, J）。
    pub fn j(&self) -> f64 {
        self.j
    }

    /// 明るさ（brightness, Q）。
    pub fn q(&self) -> f64 {
        self.q
    }

    /// カラフルネス（colorfulness, M）。
    pub fn m(&self) -> f64 {
        self.m
    }

    /// 飽和度（saturation, s）。
    pub fn s(&self) -> f64 {
        self.s
    }

    /// CAM16-UCS の J*。
    pub fn jstar(&self) -> f64 {
        self.jstar
    }

    /// CAM16-UCS の a*。
    pub fn astar(&self) -> f64 {
        self.astar
    }

    /// CAM16-UCS の b*。
    pub fn bstar(&self) -> f64 {
        self.bstar
    }

    /// CAM16-UCS 空間での色差。
    pub fn distance(&self, other: &Self) -> f64 {
        let d_j = self.jstar - other.jstar;
        let d_a = self.astar - other.astar;
        let d_b = self.bstar - other.bstar;
        let d_e_prime = (d_j * d_j + d_a * d_a + d_b * d_b).sqrt();
        1.41 * d_e_prime.powf(0.63)
    }

    /// 既定の観察条件で ARGB から作る。
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_argb_in_viewing_conditions(argb, ViewingConditions::standard())
    }

    /// 指定の観察条件で ARGB から作る。
    pub fn from_argb_in_viewing_conditions(argb: Argb, vc: &ViewingConditions) -> Self {
        let red_l = color::linearized(argb.red());
        let green_l = color::linearized(argb.green());
        let blue_l = color::linearized(argb.blue());
        let x = 0.41233895 * red_l + 0.35762064 * green_l + 0.18051042 * blue_l;
        let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
        let z = 0.01932141 * red_l + 0.11916382 * green_l + 0.95034478 * blue_l;

        let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
        let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
        let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;

        let r_d = vc.rgb_d[0] * r_c;
        let g_d = vc.rgb_d[1] * g_c;
        let b_d = vc.rgb_d[2] * b_c;

        let r_af = ((vc.fl * r_d.abs()) / 100.0).powf(0.42);
        let g_af = ((vc.fl * g_d.abs()) / 100.0).powf(0.42);
        let b_af = ((vc.fl * b_d.abs()) / 100.0).powf(0.42);

        let r_a = (math::signum(r_d) * 400.0 * r_af) / (r_af + 27.13);
        let g_a = (math::signum(g_d) * 400.0 * g_af) / (g_af + 27.13);
        let b_a = (math::signum(b_d) * 400.0 * b_af) / (b_af + 27.13);

        let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
        let b = (r_a + g_a - 2.0 * b_a) / 9.0;
        let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
        let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;
        let atan2 = b.atan2(a);
        let atan_degrees = (atan2 * 180.0) / PI;
        let hue = math::sanitize_degrees_double(atan_degrees);
        let hue_radians = (hue * PI) / 180.0;

        let ac = p2 * vc.nbb;
        let j = 100.0 * (ac / vc.aw).powf(vc.c * vc.z);
        let q = (4.0 / vc.c) * (j / 100.0).sqrt() * (vc.aw + 4.0) * vc.fl_root;
        let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
        let e_hue = 0.25 * (((hue_prime * PI) / 180.0 + 2.0).cos() + 3.8);
        let p1 = (50000.0 / 13.0) * e_hue * vc.nc * vc.ncb;
        let t = (p1 * (a * a + b * b).sqrt()) / (u + 0.305);
        let alpha = t.powf(0.9) * (1.64 - 0.29f64.powf(vc.n)).powf(0.73);
        let c = alpha * (j / 100.0).sqrt();
        let m = c * vc.fl_root;
        let s = 50.0 * ((alpha * vc.c) / (vc.aw + 4.0)).sqrt();
        let jstar = ((1.0 + 100.0 * 0.007) * j) / (1.0 + 0.007 * j);
        let mstar = (1.0 / 0.0228) * (1.0 + 0.0228 * m).ln();
        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();

        Self {
            hue,
            chroma: c,
            j,
            q,
            m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// 既定の観察条件で J（明度）・C（彩度）・h（色相）から作る。
    pub fn from_jch(j: f64, c: f64, h: f64) -> Self {
        Self::from_jch_in_viewing_conditions(j, c, h, ViewingConditions::standard())
    }

    /// 指定の観察条件で J・C・h から作る。
    pub fn from_jch_in_viewing_conditions(j: f64, c: f64, h: f64, vc: &ViewingConditions) -> Self {
        let q = (4.0 / vc.c) * (j / 100.0).sqrt() * (vc.aw + 4.0) * vc.fl_root;
        let m = c * vc.fl_root;
        let alpha = c / (j / 100.0).sqrt();
        let s = 50.0 * ((alpha * vc.c) / (vc.aw + 4.0)).sqrt();
        let hue_radians = (h * PI) / 180.0;
        let jstar = ((1.0 + 100.0 * 0.007) * j) / (1.0 + 0.007 * j);
        let mstar = (1.0 / 0.0228) * (1.0 + 0.0228 * m).ln();
        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();
        Self {
            hue: h,
            chroma: c,
            j,
            q,
            m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// 既定の観察条件で CAM16-UCS の J*・a*・b* から作る。
    pub fn from_ucs(jstar: f64, astar: f64, bstar: f64) -> Self {
        Self::from_ucs_in_viewing_conditions(jstar, astar, bstar, ViewingConditions::standard())
    }

    /// 指定の観察条件で CAM16-UCS の J*・a*・b* から作る。
    pub fn from_ucs_in_viewing_conditions(
        jstar: f64,
        astar: f64,
        bstar: f64,
        vc: &ViewingConditions,
    ) -> Self {
        let a = astar;
        let b = bstar;
        let m = (a * a + b * b).sqrt();
        let big_m = ((m * 0.0228).exp() - 1.0) / 0.0228;
        let c = big_m / vc.fl_root;
        let mut h = b.atan2(a) * (180.0 / PI);
        if h < 0.0 {
            h += 360.0;
        }
        let j = jstar / (1.0 - (jstar - 100.0) * 0.007);
        Self::from_jch_in_viewing_conditions(j, c, h, vc)
    }

    /// 既定の観察条件で ARGB にする。
    pub fn to_argb(&self) -> Argb {
        self.viewed(ViewingConditions::standard())
    }

    /// 指定の観察条件で見たときの ARGB。
    pub fn viewed(&self, vc: &ViewingConditions) -> Argb {
        let [x, y, z] = self.xyz_in_viewing_conditions(vc);
        Argb::new(color::argb_from_xyz(x, y, z))
    }

    /// 指定の観察条件で見たときの XYZ から作る。
    pub fn from_xyz_in_viewing_conditions(x: f64, y: f64, z: f64, vc: &ViewingConditions) -> Self {
        let r_c = 0.401288 * x + 0.650173 * y - 0.051461 * z;
        let g_c = -0.250268 * x + 1.204414 * y + 0.045854 * z;
        let b_c = -0.002079 * x + 0.048952 * y + 0.953127 * z;

        let r_d = vc.rgb_d[0] * r_c;
        let g_d = vc.rgb_d[1] * g_c;
        let b_d = vc.rgb_d[2] * b_c;

        let r_af = (vc.fl * r_d.abs() / 100.0).powf(0.42);
        let g_af = (vc.fl * g_d.abs() / 100.0).powf(0.42);
        let b_af = (vc.fl * b_d.abs() / 100.0).powf(0.42);
        let r_a = math::signum(r_d) * 400.0 * r_af / (r_af + 27.13);
        let g_a = math::signum(g_d) * 400.0 * g_af / (g_af + 27.13);
        let b_a = math::signum(b_d) * 400.0 * b_af / (b_af + 27.13);

        let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
        let b = (r_a + g_a - 2.0 * b_a) / 9.0;

        let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
        let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;

        let atan2 = b.atan2(a);
        let atan_degrees = atan2 * 180.0 / PI;
        let hue = if atan_degrees < 0.0 {
            atan_degrees + 360.0
        } else if atan_degrees >= 360.0 {
            atan_degrees - 360.0
        } else {
            atan_degrees
        };
        let hue_radians = hue * PI / 180.0;

        let ac = p2 * vc.nbb;

        let j = 100.0 * (ac / vc.aw).powf(vc.c * vc.z);
        let q = (4.0 / vc.c) * (j / 100.0).sqrt() * (vc.aw + 4.0) * vc.fl_root;

        let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
        let e_hue = (1.0 / 4.0) * ((hue_prime * PI / 180.0 + 2.0).cos() + 3.8);
        let p1 = 50000.0 / 13.0 * e_hue * vc.nc * vc.ncb;
        let t = p1 * (a * a + b * b).sqrt() / (u + 0.305);
        let alpha = t.powf(0.9) * (1.64 - 0.29f64.powf(vc.n)).powf(0.73);
        let c = alpha * (j / 100.0).sqrt();
        let m = c * vc.fl_root;
        let s = 50.0 * ((alpha * vc.c) / (vc.aw + 4.0)).sqrt();

        let jstar = (1.0 + 100.0 * 0.007) * j / (1.0 + 0.007 * j);
        let mstar = (1.0 + 0.0228 * m).ln() / 0.0228;
        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();
        Self {
            hue,
            chroma: c,
            j,
            q,
            m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// 指定の観察条件で見たときの XYZ。
    pub fn xyz_in_viewing_conditions(&self, vc: &ViewingConditions) -> [f64; 3] {
        let alpha = if self.chroma == 0.0 || self.j == 0.0 {
            0.0
        } else {
            self.chroma / (self.j / 100.0).sqrt()
        };

        let t = (alpha / (1.64 - 0.29f64.powf(vc.n)).powf(0.73)).powf(1.0 / 0.9);
        let h_rad = self.hue * PI / 180.0;

        let e_hue = 0.25 * ((h_rad + 2.0).cos() + 3.8);
        let ac = vc.aw * (self.j / 100.0).powf(1.0 / vc.c / vc.z);
        let p1 = e_hue * (50000.0 / 13.0) * vc.nc * vc.ncb;

        let p2 = ac / vc.nbb;

        let h_sin = h_rad.sin();
        let h_cos = h_rad.cos();

        let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * h_cos + 108.0 * t * h_sin);
        let a = gamma * h_cos;
        let b = gamma * h_sin;
        let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
        let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
        let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;

        let r_c_base = f64::max(0.0, (27.13 * r_a.abs()) / (400.0 - r_a.abs()));
        let r_c = math::signum(r_a) * (100.0 / vc.fl) * r_c_base.powf(1.0 / 0.42);
        let g_c_base = f64::max(0.0, (27.13 * g_a.abs()) / (400.0 - g_a.abs()));
        let g_c = math::signum(g_a) * (100.0 / vc.fl) * g_c_base.powf(1.0 / 0.42);
        let b_c_base = f64::max(0.0, (27.13 * b_a.abs()) / (400.0 - b_a.abs()));
        let b_c = math::signum(b_a) * (100.0 / vc.fl) * b_c_base.powf(1.0 / 0.42);
        let r_f = r_c / vc.rgb_d[0];
        let g_f = g_c / vc.rgb_d[1];
        let b_f = b_c / vc.rgb_d[2];

        let x = 1.86206786 * r_f - 1.01125463 * g_f + 0.14918677 * b_f;
        let y = 0.38752654 * r_f + 0.62144744 * g_f - 0.00897398 * b_f;
        let z = -0.01584150 * r_f - 0.03412294 * g_f + 1.04996444 * b_f;

        [x, y, z]
    }
}

#[cfg(test)]
mod tests {
    // Ported from typescript/hct/hct_test.ts ("CAM to ARGB", "CAM to ARGB to CAM")

    use super::*;

    const RED: Argb = Argb::new(0xffff_0000);
    const GREEN: Argb = Argb::new(0xff00_ff00);
    const BLUE: Argb = Argb::new(0xff00_00ff);
    const WHITE: Argb = Argb::new(0xffff_ffff);
    const BLACK: Argb = Argb::new(0xff00_0000);

    fn assert_cam(cam: Cam16, hue: f64, chroma: f64, j: f64, m: f64, s: f64, q: f64) {
        let close = |a: f64, e: f64| (a - e).abs() < 0.0005;
        assert!(close(cam.hue(), hue), "hue {} != {hue}", cam.hue());
        assert!(
            close(cam.chroma(), chroma),
            "chroma {} != {chroma}",
            cam.chroma()
        );
        assert!(close(cam.j(), j), "j {} != {j}", cam.j());
        assert!(close(cam.m(), m), "m {} != {m}", cam.m());
        assert!(close(cam.s(), s), "s {} != {s}", cam.s());
        assert!(close(cam.q(), q), "q {} != {q}", cam.q());
    }

    #[test]
    fn cam_from_argb() {
        assert_cam(
            Cam16::from_argb(RED),
            27.408,
            113.358,
            46.445,
            89.494,
            91.890,
            105.989,
        );
        assert_cam(
            Cam16::from_argb(GREEN),
            142.140,
            108.410,
            79.332,
            85.588,
            78.605,
            138.520,
        );
        assert_cam(
            Cam16::from_argb(BLUE),
            282.788,
            87.231,
            25.466,
            68.867,
            93.675,
            78.481,
        );
        assert_cam(
            Cam16::from_argb(WHITE),
            209.492,
            2.869,
            100.0,
            2.265,
            12.068,
            155.521,
        );
        assert_cam(Cam16::from_argb(BLACK), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    #[test]
    fn cam_round_trips_to_argb() {
        for color in [RED, GREEN, BLUE] {
            assert_eq!(Cam16::from_argb(color).to_argb(), color);
        }
    }
}
