// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/temperature/temperature_cache.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色の温度（暖色・寒色の度合い）に基づく類似色・補色の計算。

use std::f64::consts::PI;

use crate::hct::Hct;
use crate::units::Argb;
use crate::utils::{color, math};

/// 入力色を基準に、同じ彩度・トーンで色相を一周させた色の温度を扱う。
///
/// 生成時に 361 色ぶんの HCT と温度を計算する。
#[derive(Clone, Debug)]
pub struct TemperatureCache {
    input: Hct,
    /// 色相 0〜360（両端を含む）の 361 色。
    hcts_by_hue: Vec<Hct>,
    /// `hcts_by_hue` と入力色を温度の昇順に安定ソートしたもの。
    hcts_by_temp: Vec<(Hct, f64)>,
}

impl TemperatureCache {
    /// 入力色から作る。
    pub fn new(input: Hct) -> Self {
        let chroma = input.chroma().value();
        let tone = input.tone().value();
        let hcts_by_hue: Vec<Hct> = (0..=360)
            .map(|hue| Hct::from_hct(f64::from(hue), chroma, tone))
            .collect();
        let mut hcts_by_temp: Vec<(Hct, f64)> = hcts_by_hue
            .iter()
            .chain(std::iter::once(&input))
            .map(|&hct| (hct, raw_temperature(&hct)))
            .collect();
        // 移植元の Array.prototype.sort と同じく安定ソート。
        hcts_by_temp.sort_by(|a, b| a.1.total_cmp(&b.1));
        Self {
            input,
            hcts_by_hue,
            hcts_by_temp,
        }
    }

    /// 入力色。
    pub fn input(&self) -> Hct {
        self.input
    }

    fn hct_at_hue(&self, hue: f64) -> Hct {
        let index = math::js_round(hue);
        if index < 0.0 {
            return self.input;
        }
        self.hcts_by_hue
            .get(index as usize)
            .copied()
            .unwrap_or(self.input)
    }

    fn coldest(&self) -> (Hct, f64) {
        self.hcts_by_temp
            .first()
            .copied()
            .unwrap_or((self.input, raw_temperature(&self.input)))
    }

    fn warmest(&self) -> (Hct, f64) {
        self.hcts_by_temp
            .last()
            .copied()
            .unwrap_or((self.input, raw_temperature(&self.input)))
    }

    /// 最も暖かい色。
    pub fn warmest_color(&self) -> Hct {
        self.warmest().0
    }

    /// 最も冷たい色。
    pub fn coldest_color(&self) -> Hct {
        self.coldest().0
    }

    /// 入力色に似た色を `count` 個返す。
    ///
    /// 色相の円を温度で `divisions` 等分し、入力色を中心に並べる。
    /// 移植元の既定値は `count = 5`、`divisions = 12`。
    pub fn analogous(&self, count: usize, divisions: usize) -> Vec<Hct> {
        let start_hue = math::js_round(self.input.hue().degrees()) as i64;
        let start_hct = self.hct_at_hue(start_hue as f64);
        let mut last_temp = self.relative_temperature(&start_hct);
        let mut all_colors = vec![start_hct];

        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = math::sanitize_degrees_int(start_hue + i);
            let hct = self.hct_at_hue(hue as f64);
            let temp = self.relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            last_temp = temp;
            absolute_total_temp_delta += temp_delta;
        }

        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / divisions as f64;
        let mut total_temp_delta = 0.0;
        last_temp = self.relative_temperature(&start_hct);
        while all_colors.len() < divisions {
            let hue = math::sanitize_degrees_int(start_hue + hue_addend);
            let hct = self.hct_at_hue(hue as f64);
            let temp = self.relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            total_temp_delta += temp_delta;

            let desired_total_temp_delta_for_index = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
            let mut index_addend = 1;
            // Keep adding this hue to the answers until its temperature is
            // insufficient. This ensures consistent behavior when there aren't
            // [divisions] discrete steps between 0 and 360 in hue with [tempStep]
            // delta in temperature between them.
            //
            // For example, white and black have no analogues: there are no other
            // colors at T100/T0. Therefore, they should just be added to the array
            // as answers.
            while index_satisfied && all_colors.len() < divisions {
                all_colors.push(hct);
                let desired_total_temp_delta_for_index =
                    (all_colors.len() + index_addend) as f64 * temp_step;
                index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;

            if hue_addend > 360 {
                while all_colors.len() < divisions {
                    all_colors.push(hct);
                }
                break;
            }
        }

        let len = all_colors.len();
        let pick = |offset: isize| -> Hct {
            let index = offset.rem_euclid(len.max(1) as isize) as usize;
            all_colors.get(index).copied().unwrap_or(self.input)
        };

        let mut answers = std::collections::VecDeque::from([self.input]);

        // First, generate analogues from rotating counter-clockwise.
        let increase_hue_count = count.saturating_sub(1) / 2;
        for i in 1..=increase_hue_count {
            answers.push_front(pick(-(i as isize)));
        }

        // Second, generate analogues from rotating clockwise.
        let decrease_hue_count = count.saturating_sub(increase_hue_count + 1);
        for i in 1..=decrease_hue_count {
            answers.push_back(pick(i as isize));
        }

        answers.into()
    }

    /// 補色。温度の上で入力色と反対側にある色。
    pub fn complement(&self) -> Hct {
        let (coldest, coldest_temp) = self.coldest();
        let (warmest, warmest_temp) = self.warmest();
        let coldest_hue = coldest.hue().degrees();
        let warmest_hue = warmest.hue().degrees();
        let range = warmest_temp - coldest_temp;
        let start_hue_is_coldest_to_warmest =
            is_between(self.input.hue().degrees(), coldest_hue, warmest_hue);
        let (start_hue, end_hue) = if start_hue_is_coldest_to_warmest {
            (warmest_hue, coldest_hue)
        } else {
            (coldest_hue, warmest_hue)
        };
        let direction_of_rotation = 1.0;
        let mut smallest_error = 1000.0;
        let mut answer = self.hct_at_hue(self.input.hue().degrees());

        let complement_relative_temp = 1.0 - self.input_relative_temperature();
        // Find the color in the other section, closest to the inverse percentile
        // of the input color. This is the complement.
        for hue_addend in 0..=360 {
            let hue = math::sanitize_degrees_double(
                start_hue + direction_of_rotation * f64::from(hue_addend),
            );
            if !is_between(hue, start_hue, end_hue) {
                continue;
            }
            let possible_answer = self.hct_at_hue(hue);
            let relative_temp = (raw_temperature(&possible_answer) - coldest_temp) / range;
            let error = (complement_relative_temp - relative_temp).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible_answer;
            }
        }
        answer
    }

    /// 最も冷たい色を 0、最も暖かい色を 1 とした `hct` の温度。
    pub fn relative_temperature(&self, hct: &Hct) -> f64 {
        let (_, coldest_temp) = self.coldest();
        let (_, warmest_temp) = self.warmest();
        let range = warmest_temp - coldest_temp;
        let difference_from_coldest = raw_temperature(hct) - coldest_temp;
        // Handle when there's no difference in temperature between warmest and
        // coldest: for example, at T100, only one color is available, white.
        if range == 0.0 {
            return 0.5;
        }
        difference_from_coldest / range
    }

    /// 入力色の相対的な温度。
    pub fn input_relative_temperature(&self) -> f64 {
        self.relative_temperature(&self.input)
    }
}

fn is_between(angle: f64, a: f64, b: f64) -> bool {
    if a < b {
        a <= angle && angle <= b
    } else {
        a <= angle || angle <= b
    }
}

/// 色の温度（L*a*b* に基づく）。暖色ほど大きく、おおよそ -0.5〜3 の範囲。
pub fn raw_temperature(color: &Hct) -> f64 {
    raw_temperature_of_argb(color.to_argb())
}

fn raw_temperature_of_argb(argb: Argb) -> f64 {
    let lab = color::lab_from_argb(argb.to_u32());
    let hue = math::sanitize_degrees_double(lab[2].atan2(lab[1]) * 180.0 / PI);
    let chroma = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    -0.5 + 0.02 * chroma.powf(1.07) * (math::sanitize_degrees_double(hue - 50.0) * PI / 180.0).cos()
}

#[cfg(test)]
mod tests {
    // Ported from typescript/temperature/temperature_cache_test.ts

    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 0.0005, "{actual} != {expected}");
    }

    fn hct(argb: u32) -> Hct {
        Hct::from_argb(Argb::new(argb))
    }

    #[test]
    fn computes_raw_temperatures_correctly() {
        assert_close(raw_temperature(&hct(0xff0000ff)), -1.393);
        assert_close(raw_temperature(&hct(0xffff0000)), 2.351);
        assert_close(raw_temperature(&hct(0xff00ff00)), -0.267);
        assert_close(raw_temperature(&hct(0xffffffff)), -0.5);
        assert_close(raw_temperature(&hct(0xff000000)), -0.5);
    }

    #[test]
    fn relative_temperature() {
        let relative = |argb| TemperatureCache::new(hct(argb)).input_relative_temperature();
        assert_close(relative(0xff0000ff), 0.0);
        assert_close(relative(0xffff0000), 1.0);
        assert_close(relative(0xff00ff00), 0.467);
        assert_close(relative(0xffffffff), 0.5);
        assert_close(relative(0xff000000), 0.5);
    }

    #[test]
    fn complement() {
        let complement = |argb| TemperatureCache::new(hct(argb)).complement().to_argb();
        assert_eq!(complement(0xff0000ff), Argb::new(0xff9d0002));
        assert_eq!(complement(0xffff0000), Argb::new(0xff007bfc));
        assert_eq!(complement(0xff00ff00), Argb::new(0xffffd2c9));
        assert_eq!(complement(0xffffffff), Argb::new(0xffffffff));
        assert_eq!(complement(0xff000000), Argb::new(0xff000000));
    }

    #[test]
    fn analogous() {
        let analogous = |argb| -> Vec<u32> {
            TemperatureCache::new(hct(argb))
                .analogous(5, 12)
                .iter()
                .map(|h| h.to_argb().to_u32())
                .collect()
        };
        assert_eq!(
            analogous(0xff0000ff),
            [0xff00590c, 0xff00564e, 0xff0000ff, 0xff6700cc, 0xff81009f]
        );
        assert_eq!(
            analogous(0xffff0000),
            [0xfff60082, 0xfffc004c, 0xffff0000, 0xffd95500, 0xffaf7200]
        );
        assert_eq!(
            analogous(0xff00ff00),
            [0xffcee900, 0xff92f500, 0xff00ff00, 0xff00fd6f, 0xff00fab3]
        );
        assert_eq!(analogous(0xff000000), [0xff000000; 5]);
        assert_eq!(analogous(0xffffffff), [0xffffffff; 5]);
    }
}
