// Ported from material-color-utilities
// (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/temperature/temperature_cache.ts @ bd811e4ca2ab94c4ce3cadc89c19b1820b9704b3
// Copyright 2023 Google LLC. Licensed under the Apache License, Version 2.0.

//! 色温度に基づく補色・類似色

use std::f64::consts::PI;

use crate::hct::Hct;
use crate::utils::{color, math, math::round_js};

/// 入力色と同じ彩度・トーンを持つ全色相の色と、その色温度
///
/// 色温度は Ou, Woodcock, Wright らの暖色・寒色の研究に基づく。生成時に 361 色を計算する
#[derive(Clone, Debug)]
pub struct TemperatureCache {
    input: Hct,
    input_temp: f64,
    /// 色相 0〜360 (両端を含む) の色
    hcts_by_hue: Vec<Hct>,
    temps_by_hue: Vec<f64>,
    coldest: (Hct, f64),
    warmest: (Hct, f64),
}

impl TemperatureCache {
    /// `input` の色温度を計算する
    pub fn new(input: Hct) -> Self {
        let hcts_by_hue: Vec<Hct> = (0..=360)
            .map(|hue| Hct::from_hct_raw(f64::from(hue), input.chroma_raw(), input.tone_raw()))
            .collect();
        let temps_by_hue: Vec<f64> = hcts_by_hue.iter().map(raw_temperature).collect();
        let input_temp = raw_temperature(&input);

        // 移植元は色相順の 361 色の末尾に入力色を加え、色温度で安定ソートした両端を使う。
        // 同じ色温度の色が並ぶときは、最も寒い色は先頭側、最も暖かい色は末尾側が選ばれる
        let mut coldest = (input, input_temp);
        let mut warmest = (input, input_temp);
        let mut first = true;
        for (hct, temp) in hcts_by_hue
            .iter()
            .copied()
            .zip(temps_by_hue.iter().copied())
            .chain(std::iter::once((input, input_temp)))
        {
            if first {
                coldest = (hct, temp);
                warmest = (hct, temp);
                first = false;
                continue;
            }
            if temp < coldest.1 {
                coldest = (hct, temp);
            }
            if temp >= warmest.1 {
                warmest = (hct, temp);
            }
        }

        Self {
            input,
            input_temp,
            hcts_by_hue,
            temps_by_hue,
            coldest,
            warmest,
        }
    }

    /// 入力色
    pub fn input(&self) -> Hct {
        self.input
    }

    /// 最も暖かい色
    pub fn warmest(&self) -> Hct {
        self.warmest.0
    }

    /// 最も寒い色
    pub fn coldest(&self) -> Hct {
        self.coldest.0
    }

    /// 入力色の補色
    ///
    /// 色相環を最も寒い色と最も暖かい色で 2 つの弧に分け、入力色を含まない側の弧から、
    /// 相対的な色温度が `1 - 入力色の相対的な色温度` に最も近い色を選ぶ
    pub fn complement(&self) -> Hct {
        let coldest_hue = self.coldest.0.hue_raw();
        let coldest_temp = self.coldest.1;
        let warmest_hue = self.warmest.0.hue_raw();
        let warmest_temp = self.warmest.1;
        let range = warmest_temp - coldest_temp;
        let start_hue_is_coldest_to_warmest =
            is_between(self.input.hue_raw(), coldest_hue, warmest_hue);
        let start_hue = if start_hue_is_coldest_to_warmest {
            warmest_hue
        } else {
            coldest_hue
        };
        let end_hue = if start_hue_is_coldest_to_warmest {
            coldest_hue
        } else {
            warmest_hue
        };
        let direction_of_rotation = 1.0;
        let mut smallest_error = 1000.0;
        let mut answer = self.by_rounded_hue(self.input.hue_raw());

        let complement_relative_temp = 1.0 - self.input_relative_temperature();
        // 色相環を 1 度ずつ回り、補色の色温度に最も近い色を探す
        let mut hue_addend = 0.0;
        while hue_addend <= 360.0 {
            let hue = math::sanitize_degrees_double(start_hue + direction_of_rotation * hue_addend);
            hue_addend += 1.0;
            if !is_between(hue, start_hue, end_hue) {
                continue;
            }
            let possible_answer = self.by_rounded_hue(hue);
            let relative_temp = (possible_answer.1 - coldest_temp) / range;
            let error = (complement_relative_temp - relative_temp).abs();
            if error < smallest_error {
                smallest_error = error;
                answer = possible_answer;
            }
        }
        answer.0
    }

    /// 入力色を含む `count` 色の類似色。色相環を色温度で `divisions` 等分した色から選ぶ
    ///
    /// 結果は入力色を中央に置き、前後に色相環上の隣の色を並べる。`count` が 0 のときは入力色だけを返す。
    /// 移植元の既定値は `count = 5`、`divisions = 12`
    pub fn analogous(&self, count: usize, divisions: usize) -> Vec<Hct> {
        if count == 0 {
            // 移植元は前後に加える色の数が負になり、入力色だけを返す
            return vec![self.input];
        }
        let start_hue = round_js(self.input.hue_raw()) as i64;
        let start = self.by_hue_index(start_hue);
        let mut last_temp = self.relative_temperature_of(start.1);

        let mut all_colors = vec![start.0];

        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = math::sanitize_degrees_int(start_hue + i);
            let temp = self.relative_temperature_of(self.by_hue_index(hue).1);
            let temp_delta = (temp - last_temp).abs();
            last_temp = temp;
            absolute_total_temp_delta += temp_delta;
        }

        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / divisions as f64;
        let mut total_temp_delta = 0.0;
        last_temp = self.relative_temperature_of(start.1);
        while all_colors.len() < divisions {
            let hue = math::sanitize_degrees_int(start_hue + hue_addend);
            let (hct, raw_temp) = self.by_hue_index(hue);
            let temp = self.relative_temperature_of(raw_temp);
            let temp_delta = (temp - last_temp).abs();
            total_temp_delta += temp_delta;

            let desired_total_temp_delta_for_index = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
            let mut index_addend = 1;
            // 色温度の変化が大きい色相では、同じ色を複数回加えることがある
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
        let mut answers = vec![self.input];

        let increase_hue_count = (count - 1) / 2;
        for i in 1..=increase_hue_count {
            // 移植元は負の添字に len を足し、len 以上の添字を len で割った余りにする
            let index = (len as i64 - i as i64).rem_euclid(len as i64) as usize;
            if let Some(hct) = all_colors.get(index) {
                answers.insert(0, *hct);
            }
        }

        let decrease_hue_count = count - increase_hue_count - 1;
        for i in 1..=decrease_hue_count {
            let index = i % len;
            if let Some(hct) = all_colors.get(index) {
                answers.push(*hct);
            }
        }

        answers
    }

    /// 入力色の相対的な色温度。最も寒い色が 0、最も暖かい色が 1
    pub fn input_relative_temperature(&self) -> f64 {
        self.relative_temperature_of(self.input_temp)
    }

    /// `hct` の相対的な色温度。最も寒い色が 0、最も暖かい色が 1。範囲外の色温度は 0〜1 を外れる
    ///
    /// 全色相の色温度が等しいときは 0.5
    pub fn relative_temperature(&self, hct: &Hct) -> f64 {
        self.relative_temperature_of(raw_temperature(hct))
    }

    fn relative_temperature_of(&self, temp: f64) -> f64 {
        let range = self.warmest.1 - self.coldest.1;
        let difference_from_coldest = temp - self.coldest.1;
        // 全色相の色温度が等しいとき、範囲が 0 になる
        if range == 0.0 {
            return 0.5;
        }
        difference_from_coldest / range
    }

    /// 色相を四捨五入した添字の色と色温度
    fn by_rounded_hue(&self, hue: f64) -> (Hct, f64) {
        self.by_hue_index(round_js(hue) as i64)
    }

    /// 色相 0〜360 の添字の色と色温度。添字は呼び出し元で範囲内に収めている
    fn by_hue_index(&self, index: i64) -> (Hct, f64) {
        let index = math::clamp_int(0, 360, index) as usize;
        match (self.hcts_by_hue.get(index), self.temps_by_hue.get(index)) {
            (Some(hct), Some(temp)) => (*hct, *temp),
            _ => (self.input, self.input_temp),
        }
    }
}

/// `angle` が `a` から色相の増える向きに `b` まで進む弧の上にあるか (両端を含む)
fn is_between(angle: f64, a: f64, b: f64) -> bool {
    if a < b {
        a <= angle && angle <= b
    } else {
        a <= angle || angle <= b
    }
}

/// 色温度の絶対値。-0.5 より小さければ寒色、大きければ暖色
///
/// 色温度は L*a*b* の色相と彩度から求める。値の範囲はおよそ -0.5〜3.0
pub fn raw_temperature(color: &Hct) -> f64 {
    let lab = color::lab_from_argb(color.argb_raw());
    let hue = math::sanitize_degrees_double(lab[2].atan2(lab[1]) * 180.0 / PI);
    let chroma = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    -0.5 + 0.02 * chroma.powf(1.07) * (math::sanitize_degrees_double(hue - 50.0) * PI / 180.0).cos()
}
