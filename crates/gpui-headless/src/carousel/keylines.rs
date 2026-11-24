// キーラインの計算は Jetpack Compose Material 3 から移植している
// (https://android.googlesource.com/platform/frameworks/support)
// Source: compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/
//         Arrangement.kt、KeylineList.kt、Keylines.kt、Strategy.kt、KeylineSnapPosition.kt
//         @ 2a941dd11aabd9bb7d2016ed159764b220117c20
// Copyright 2023 The Android Open Source Project. Licensed under the Apache License, Version 2.0.

//! カルーセルの項目の大きさと位置を決めるキーラインの計算。長さはすべてピクセル

/// 見えない端のキーラインの大きさ (Compose の `CarouselDefaults.AnchorSize`)
pub const ANCHOR_SIZE: f32 = 10.0;
/// 中くらいの項目が大きい項目にどこまで近づけるか (Compose の `CarouselDefaults.MediumLargeItemDiffThreshold`)
const MEDIUM_LARGE_ITEM_DIFF_THRESHOLD: f32 = 0.85;
/// 中くらいの項目の大きさを変えてよい割合 (Compose の `Arrangement.MediumItemFlexPercentage`)
const MEDIUM_ITEM_FLEX_PERCENTAGE: f32 = 0.1;

fn lerp(start: f32, end: f32, fraction: f32) -> f32 {
    start + (end - start) * fraction
}

/// `value` を入力の範囲から出力の範囲へ写す。入力の範囲の外は出力の端にする
fn lerp_range(output_min: f32, output_max: f32, input_min: f32, input_max: f32, value: f32) -> f32 {
    if value <= input_min {
        output_min
    } else if value >= input_max {
        output_max
    } else {
        lerp(
            output_min,
            output_max,
            (value - input_min) / (input_max - input_min),
        )
    }
}

/// 小・中・大の項目の数と大きさの組み合わせ
#[derive(Clone, Copy, Debug, PartialEq)]
struct Arrangement {
    priority: usize,
    small_size: f32,
    small_count: usize,
    medium_size: f32,
    medium_count: usize,
    large_size: f32,
    large_count: usize,
}

/// 項目の大きさの目安と、試す数の組み合わせ
struct ArrangementTarget<'a> {
    available_space: f32,
    item_spacing: f32,
    target_small_size: f32,
    min_small_size: f32,
    max_small_size: f32,
    small_counts: &'a [usize],
    target_medium_size: f32,
    medium_counts: &'a [usize],
    target_large_size: f32,
    large_counts: &'a [usize],
}

impl Arrangement {
    fn is_valid(&self) -> bool {
        if self.large_count > 0 && self.small_count > 0 && self.medium_count > 0 {
            self.large_size > self.medium_size && self.medium_size > self.small_size
        } else if self.large_count > 0 && self.small_count > 0 {
            self.large_size > self.small_size
        } else {
            true
        }
    }

    fn cost(&self, target_large_size: f32) -> f32 {
        if !self.is_valid() {
            return f32::MAX;
        }
        (target_large_size - self.large_size).abs() * self.priority as f32
    }

    fn item_count(&self) -> usize {
        self.large_count + self.medium_count + self.small_count
    }

    fn find_lowest_cost(target: &ArrangementTarget) -> Option<Self> {
        let mut lowest: Option<Self> = None;
        let mut priority = 1;
        for &large_count in target.large_counts {
            for &medium_count in target.medium_counts {
                for &small_count in target.small_counts {
                    let arrangement =
                        Self::fit(priority, target, small_count, medium_count, large_count);
                    if lowest.is_none_or(|lowest| {
                        arrangement.cost(target.target_large_size)
                            < lowest.cost(target.target_large_size)
                    }) {
                        lowest = Some(arrangement);
                        if arrangement.cost(target.target_large_size) == 0.0 {
                            return lowest;
                        }
                    }
                    priority += 1;
                }
            }
        }
        lowest
    }

    fn fit(
        priority: usize,
        target: &ArrangementTarget,
        small_count: usize,
        medium_count: usize,
        large_count: usize,
    ) -> Self {
        let total_count = large_count + medium_count + small_count;
        let available = target.available_space - (total_count as f32 - 1.0) * target.item_spacing;
        let mut small_size = target
            .target_small_size
            .clamp(target.min_small_size, target.max_small_size);
        let taken = target.target_large_size * large_count as f32
            + target.target_medium_size * medium_count as f32
            + small_size * small_count as f32;
        let delta = available - taken;
        if small_count > 0 && delta > 0.0 {
            small_size += (delta / small_count as f32).min(target.max_small_size - small_size);
        } else if small_count > 0 && delta < 0.0 {
            small_size += (delta / small_count as f32).max(target.min_small_size - small_size);
        }
        if small_count == 0 {
            small_size = 0.0;
        }
        let mut large_size = (available
            - (small_count as f32 + medium_count as f32 / 2.0) * small_size)
            / (large_count as f32 + medium_count as f32 / 2.0);
        let mut medium_size = (large_size + small_size) / 2.0;
        if medium_count > 0 && large_size != target.target_large_size {
            let adjustment = (target.target_large_size - large_size) * large_count as f32;
            let flex = medium_size * MEDIUM_ITEM_FLEX_PERCENTAGE * medium_count as f32;
            let distribute = adjustment.abs().min(flex);
            if adjustment > 0.0 {
                medium_size -= distribute / medium_count as f32;
                large_size += distribute / large_count as f32;
            } else {
                medium_size += distribute / medium_count as f32;
                large_size -= distribute / large_count as f32;
            }
        }
        Self {
            priority,
            small_size,
            small_count,
            medium_size,
            medium_count,
            large_size,
            large_count,
        }
    }
}

/// 項目の中心を置く位置と、そこでの項目の大きさ
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyline {
    /// 項目の見えている大きさ
    pub size: f32,
    /// 項目の中心のカルーセルの始まりからの位置
    pub offset: f32,
    /// すべての項目が焦点の大きさのときの中心の位置
    pub unadjusted_offset: f32,
    /// 最も大きい項目を置くキーラインか
    pub is_focal: bool,
    /// 項目が見えない、両端の外のキーラインか
    pub is_anchor: bool,
    /// 位置を決める基準にしたキーラインか
    pub is_pivot: bool,
    /// カルーセルの端からはみ出した大きさ
    pub cutoff: f32,
}

fn lerp_keyline(start: &Keyline, end: &Keyline, fraction: f32) -> Keyline {
    let pick = |start: bool, end: bool| if fraction < 0.5 { start } else { end };
    Keyline {
        size: lerp(start.size, end.size, fraction),
        offset: lerp(start.offset, end.offset, fraction),
        unadjusted_offset: lerp(start.unadjusted_offset, end.unadjusted_offset, fraction),
        is_focal: pick(start.is_focal, end.is_focal),
        is_anchor: pick(start.is_anchor, end.is_anchor),
        is_pivot: pick(start.is_pivot, end.is_pivot),
        cutoff: lerp(start.cutoff, end.cutoff, fraction),
    }
}

/// 始まりから順に並べたキーライン
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeylineList(Vec<Keyline>);

/// キーラインを並べる基準
#[derive(Clone, Copy, PartialEq, Eq)]
enum Alignment {
    Start,
    Center,
}

impl KeylineList {
    /// 始まりから順のキーライン
    pub fn keylines(&self) -> &[Keyline] {
        &self.0
    }

    fn min_size(&self) -> f32 {
        self.0.iter().map(|k| k.size).fold(f32::MAX, f32::min)
    }

    fn max_size(&self) -> f32 {
        self.0.iter().map(|k| k.size).fold(0.0, f32::max)
    }

    fn pivot_index(&self) -> usize {
        self.0.iter().position(|k| k.is_pivot).unwrap_or(0)
    }

    fn first_non_anchor_index(&self) -> usize {
        self.0.iter().position(|k| !k.is_anchor).unwrap_or(0)
    }

    fn last_non_anchor_index(&self) -> usize {
        self.0.iter().rposition(|k| !k.is_anchor).unwrap_or(0)
    }

    fn first_focal_index(&self) -> usize {
        self.0.iter().position(|k| k.is_focal).unwrap_or(0)
    }

    fn last_focal_index(&self) -> usize {
        self.0.iter().rposition(|k| k.is_focal).unwrap_or(0)
    }

    fn first_focal(&self) -> Keyline {
        self.0[self.first_focal_index()]
    }

    fn last_focal(&self) -> Keyline {
        self.0[self.last_focal_index()]
    }

    fn focal_count(&self) -> usize {
        self.last_focal_index() - self.first_focal_index() + 1
    }

    fn is_first_focal_item_at_start(&self) -> bool {
        let first_focal = self.first_focal();
        first_focal.offset - first_focal.size / 2.0 >= 0.0
            && self.first_focal_index() == self.first_non_anchor_index()
    }

    fn is_last_focal_item_at_end(&self, main_axis_size: f32) -> bool {
        let last_focal = self.last_focal();
        last_focal.offset + last_focal.size / 2.0 <= main_axis_size
            && self.last_focal_index() == self.last_non_anchor_index()
    }

    fn first_index_after_focal_range_with_size(&self, size: f32) -> usize {
        let last = self.0.len() - 1;
        (self.last_focal_index()..=last)
            .find(|&i| self.0[i].size == size)
            .unwrap_or(last)
    }

    fn last_index_before_focal_range_with_size(&self, size: f32) -> usize {
        (0..self.first_focal_index())
            .rev()
            .find(|&i| self.0[i].size == size)
            .unwrap_or(0)
    }

    /// `unadjusted_offset` より前にある最後のキーライン。無ければ最初のキーライン
    pub fn keyline_before(&self, unadjusted_offset: f32) -> Keyline {
        self.0
            .iter()
            .rev()
            .find(|k| k.unadjusted_offset < unadjusted_offset)
            .copied()
            .unwrap_or(self.0[0])
    }

    /// `unadjusted_offset` 以降にある最初のキーライン。無ければ最後のキーライン
    pub fn keyline_after(&self, unadjusted_offset: f32) -> Keyline {
        self.0
            .iter()
            .find(|k| k.unadjusted_offset >= unadjusted_offset)
            .copied()
            .unwrap_or(self.0[self.0.len() - 1])
    }

    fn lerp(from: &Self, to: &Self, fraction: f32) -> Self {
        Self(
            from.0
                .iter()
                .zip(&to.0)
                .map(|(from, to)| lerp_keyline(from, to, fraction))
                .collect(),
        )
    }

    /// `sizes` (大きさと見えない端か) を並べ、最も大きい項目を焦点として、`alignment` で置く
    fn with_alignment(
        main_axis_size: f32,
        item_spacing: f32,
        alignment: Alignment,
        sizes: &[(f32, bool)],
    ) -> Self {
        let (first_focal, focal_size) = focal(sizes);
        let Some(first_focal) = first_focal else {
            return Self::default();
        };
        let last_focal = last_focal(sizes, first_focal, focal_size);
        let focal_count = (last_focal - first_focal) as f32;
        let pivot_offset = match alignment {
            Alignment::Center => {
                let split = if item_spacing == 0.0 || (last_focal - first_focal).is_multiple_of(2) {
                    0.0
                } else {
                    item_spacing / 2.0
                };
                let spaces = ((last_focal - first_focal) / 2) as f32 * item_spacing;
                main_axis_size / 2.0 - focal_size / 2.0 * focal_count - split - spaces
            }
            Alignment::Start => focal_size / 2.0,
        };
        Self::with_pivot_inner(
            main_axis_size,
            item_spacing,
            first_focal,
            pivot_offset,
            first_focal,
            last_focal,
            focal_size,
            sizes,
        )
    }

    /// `sizes` を並べ、`pivot_index` の項目の中心を `pivot_offset` に置く
    fn with_pivot(
        main_axis_size: f32,
        item_spacing: f32,
        pivot_index: usize,
        pivot_offset: f32,
        sizes: &[(f32, bool)],
    ) -> Self {
        let (first_focal, focal_size) = focal(sizes);
        let Some(first_focal) = first_focal else {
            return Self::default();
        };
        let last_focal = last_focal(sizes, first_focal, focal_size);
        Self::with_pivot_inner(
            main_axis_size,
            item_spacing,
            pivot_index,
            pivot_offset,
            first_focal,
            last_focal,
            focal_size,
            sizes,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Compose の createKeylinesWithPivot と同じ引数"
    )]
    fn with_pivot_inner(
        main_axis_size: f32,
        item_spacing: f32,
        pivot_index: usize,
        pivot_offset: f32,
        first_focal: usize,
        last_focal: usize,
        item_size: f32,
        sizes: &[(f32, bool)],
    ) -> Self {
        if sizes.is_empty() || pivot_index >= sizes.len() {
            return Self::default();
        }
        let cutoff_left =
            |size: f32, offset: f32| offset - size / 2.0 < 0.0 && offset + size / 2.0 > 0.0;
        let cutoff_right = |size: f32, offset: f32| {
            offset - size / 2.0 < main_axis_size && offset + size / 2.0 > main_axis_size
        };
        let focal_range = first_focal..=last_focal;
        let (pivot_size, pivot_anchor) = sizes[pivot_index];
        let pivot_cutoff = if cutoff_left(pivot_size, pivot_offset) {
            pivot_offset - pivot_size / 2.0
        } else if cutoff_right(pivot_size, pivot_offset) {
            pivot_offset + pivot_size / 2.0 - main_axis_size
        } else {
            0.0
        };
        let mut before = Vec::new();
        let mut offset = pivot_offset - item_size / 2.0 - item_spacing;
        let mut unadjusted = pivot_offset - item_size / 2.0 - item_spacing;
        for index in (0..pivot_index).rev() {
            let (size, is_anchor) = sizes[index];
            let keyline_offset = offset - size / 2.0;
            let cutoff = if cutoff_left(size, keyline_offset) {
                (keyline_offset - size / 2.0).abs()
            } else {
                0.0
            };
            before.push(Keyline {
                size,
                offset: keyline_offset,
                unadjusted_offset: unadjusted - item_size / 2.0,
                is_focal: focal_range.contains(&index),
                is_anchor,
                is_pivot: false,
                cutoff,
            });
            offset -= size + item_spacing;
            unadjusted -= item_size + item_spacing;
        }
        before.reverse();
        before.push(Keyline {
            size: pivot_size,
            offset: pivot_offset,
            unadjusted_offset: pivot_offset,
            is_focal: focal_range.contains(&pivot_index),
            is_anchor: pivot_anchor,
            is_pivot: true,
            cutoff: pivot_cutoff,
        });
        let mut offset = pivot_offset + item_size / 2.0 + item_spacing;
        let mut unadjusted = pivot_offset + item_size / 2.0 + item_spacing;
        for (index, &(size, is_anchor)) in sizes.iter().enumerate().skip(pivot_index + 1) {
            let keyline_offset = offset + size / 2.0;
            let cutoff = if cutoff_right(size, keyline_offset) {
                keyline_offset + size / 2.0 - main_axis_size
            } else {
                0.0
            };
            before.push(Keyline {
                size,
                offset: keyline_offset,
                unadjusted_offset: unadjusted + item_size / 2.0,
                is_focal: focal_range.contains(&index),
                is_anchor,
                is_pivot: false,
                cutoff,
            });
            offset += size + item_spacing;
            unadjusted += item_size + item_spacing;
        }
        Self(before)
    }

    fn sizes(&self) -> Vec<(f32, bool)> {
        self.0.iter().map(|k| (k.size, k.is_anchor)).collect()
    }
}

/// 見えない端でない最初の最も大きい項目の番号と、その大きさ
fn focal(sizes: &[(f32, bool)]) -> (Option<usize>, f32) {
    let mut first = None;
    let mut focal_size = 0.0;
    for (index, &(size, is_anchor)) in sizes.iter().enumerate() {
        if !is_anchor && size > focal_size {
            first = Some(index);
            focal_size = size;
        }
    }
    (first, focal_size)
}

fn last_focal(sizes: &[(f32, bool)], first_focal: usize, focal_size: f32) -> usize {
    let mut last = first_focal;
    while last + 1 < sizes.len() && sizes[last + 1].0 == focal_size {
        last += 1;
    }
    last
}

fn left_aligned(
    main_axis_size: f32,
    item_spacing: f32,
    left_anchor: f32,
    right_anchor: f32,
    arrangement: &Arrangement,
) -> KeylineList {
    let mut sizes = vec![(left_anchor, true)];
    sizes.extend(std::iter::repeat_n(
        (arrangement.large_size, false),
        arrangement.large_count,
    ));
    sizes.extend(std::iter::repeat_n(
        (arrangement.medium_size, false),
        arrangement.medium_count,
    ));
    sizes.extend(std::iter::repeat_n(
        (arrangement.small_size, false),
        arrangement.small_count,
    ));
    sizes.push((right_anchor, true));
    KeylineList::with_alignment(main_axis_size, item_spacing, Alignment::Start, &sizes)
}

fn center_aligned(
    main_axis_size: f32,
    item_spacing: f32,
    left_anchor: f32,
    right_anchor: f32,
    arrangement: &Arrangement,
) -> KeylineList {
    let small = (arrangement.small_size, false);
    let medium = (arrangement.medium_size, false);
    let mut sizes = vec![(left_anchor, true)];
    sizes.extend(std::iter::repeat_n(small, arrangement.small_count / 2));
    sizes.extend(std::iter::repeat_n(medium, arrangement.medium_count / 2));
    sizes.extend(std::iter::repeat_n(
        (arrangement.large_size, false),
        arrangement.large_count,
    ));
    sizes.extend(std::iter::repeat_n(medium, arrangement.medium_count / 2));
    sizes.extend(std::iter::repeat_n(small, arrangement.small_count / 2));
    sizes.push((right_anchor, true));
    KeylineList::with_alignment(main_axis_size, item_spacing, Alignment::Center, &sizes)
}

/// 大きい項目の後ろに中くらいと小さい項目を並べるキーライン (Compose の `multiBrowseKeylineList`)
pub fn multi_browse(
    main_axis_size: f32,
    preferred_item_size: f32,
    item_spacing: f32,
    item_count: usize,
    min_small_size: f32,
    max_small_size: f32,
) -> KeylineList {
    if main_axis_size == 0.0 || preferred_item_size == 0.0 {
        return KeylineList::default();
    }
    let mut small_counts: &[usize] = &[1];
    let medium_counts = [1, 0];
    let target_large_size = preferred_item_size.min(main_axis_size);
    let target_small_size = (target_large_size / 3.0).clamp(min_small_size, max_small_size);
    let target_medium_size = (target_large_size + target_small_size) / 2.0;
    if main_axis_size < min_small_size * 2.0 {
        small_counts = &[0];
    }
    let min_available_large_space = main_axis_size
        - target_medium_size
        - max_small_size * *small_counts.iter().max().unwrap_or(&0) as f32;
    let min_large_count = ((min_available_large_space / target_large_size).floor() as usize).max(1);
    let max_large_count = (main_axis_size / target_large_size).ceil() as usize;
    let large_counts: Vec<usize> = (min_large_count..=max_large_count.max(min_large_count))
        .rev()
        .collect();
    let mut target = ArrangementTarget {
        available_space: main_axis_size,
        item_spacing,
        target_small_size,
        min_small_size,
        max_small_size,
        small_counts,
        target_medium_size,
        medium_counts: &medium_counts,
        target_large_size,
        large_counts: &large_counts,
    };
    let mut arrangement = Arrangement::find_lowest_cost(&target);
    if let Some(found) = arrangement
        && found.item_count() > item_count
    {
        let mut surplus = found.item_count() - item_count;
        let mut small_count = found.small_count;
        let mut medium_count = found.medium_count;
        while surplus > 0 {
            if small_count > 0 {
                small_count -= 1;
            } else if medium_count > 1 {
                medium_count -= 1;
            }
            surplus -= 1;
        }
        let small = [small_count];
        let medium = [medium_count];
        target.small_counts = &small;
        target.medium_counts = &medium;
        arrangement = Arrangement::find_lowest_cost(&target);
    }
    let Some(arrangement) = arrangement else {
        return KeylineList::default();
    };
    left_aligned(
        main_axis_size,
        item_spacing,
        ANCHOR_SIZE,
        ANCHOR_SIZE,
        &arrangement,
    )
}

/// 同じ大きさの項目を並べ、収まらない分を中くらいの項目にするキーライン (Compose の `uncontainedKeylineList`)
pub fn uncontained(main_axis_size: f32, item_size: f32, item_spacing: f32) -> KeylineList {
    if main_axis_size == 0.0 || item_size == 0.0 {
        return KeylineList::default();
    }
    let large_size = (item_size + item_spacing).min(main_axis_size);
    let large_count = ((main_axis_size / large_size).floor() as usize).max(1);
    let remaining = main_axis_size - large_count as f32 * large_size;
    let medium_count = usize::from(remaining > 0.0);
    // Compose の calculateMediumChildSize
    let mut medium_size = (remaining * 1.5).max(ANCHOR_SIZE);
    let threshold = large_size * MEDIUM_LARGE_ITEM_DIFF_THRESHOLD;
    if medium_size > threshold {
        medium_size = threshold.max(remaining * 1.2).min(large_size);
    }
    let arrangement = Arrangement {
        priority: 0,
        small_size: 0.0,
        small_count: 0,
        medium_size,
        medium_count,
        large_size,
        large_count,
    };
    let left_anchor = ANCHOR_SIZE.min(item_size).max(medium_size * 0.5);
    left_aligned(
        main_axis_size,
        item_spacing,
        left_anchor,
        ANCHOR_SIZE,
        &arrangement,
    )
}

/// 大きい項目の後ろか両側に小さい項目を置くキーライン (Compose の `heroKeylineList`)
pub fn hero(
    main_axis_size: f32,
    preferred_item_size: Option<f32>,
    item_spacing: f32,
    item_count: usize,
    centered: bool,
    min_small_size: f32,
    max_small_size: f32,
) -> KeylineList {
    if main_axis_size == 0.0 {
        return KeylineList::default();
    }
    let should_center = centered && item_count >= 3;
    let mut small_count = if item_count <= 1 {
        0
    } else if should_center {
        2
    } else {
        1
    };
    let target_large_size = preferred_item_size
        .unwrap_or(main_axis_size)
        .min(main_axis_size);
    let target_small_size = (target_large_size / 3.0).clamp(min_small_size, max_small_size);
    let fullscreen_threshold = min_small_size * small_count as f32 + min_small_size * 1.25;
    if main_axis_size < fullscreen_threshold {
        small_count = 0;
    }
    let min_available_large_space = main_axis_size - min_small_size * small_count as f32;
    let min_large_count = ((min_available_large_space / target_large_size).floor() as usize).max(1);
    let max_large_count = (main_axis_size / target_large_size).ceil() as usize;
    let large_counts: Vec<usize> = (min_large_count..=max_large_count.max(min_large_count))
        .rev()
        .collect();
    let Some(arrangement) = Arrangement::find_lowest_cost(&ArrangementTarget {
        available_space: main_axis_size,
        item_spacing,
        target_small_size,
        min_small_size,
        max_small_size,
        small_counts: &[small_count],
        target_medium_size: 0.0,
        medium_counts: &[0],
        target_large_size,
        large_counts: &large_counts,
    }) else {
        return KeylineList::default();
    };
    if should_center && item_count >= arrangement.item_count() {
        center_aligned(
            main_axis_size,
            item_spacing,
            ANCHOR_SIZE,
            ANCHOR_SIZE,
            &arrangement,
        )
    } else {
        left_aligned(
            main_axis_size,
            item_spacing,
            ANCHOR_SIZE,
            ANCHOR_SIZE,
            &arrangement,
        )
    }
}

/// スクロールの位置ごとのキーラインを求める (Compose の `Strategy`)
#[derive(Clone, Debug, PartialEq)]
pub struct Strategy {
    default: KeylineList,
    start_steps: Vec<KeylineList>,
    end_steps: Vec<KeylineList>,
    available_space: f32,
    item_spacing: f32,
    start_shift_distance: f32,
    end_shift_distance: f32,
    start_shift_points: Vec<f32>,
    end_shift_points: Vec<f32>,
    min_item_size: f32,
    max_item_size: f32,
}

impl Strategy {
    /// `default` を中ほどのキーラインとし、両端に向けて項目を寄せる段階を作る
    pub fn new(
        default: KeylineList,
        available_space: f32,
        item_spacing: f32,
        before_padding: f32,
        after_padding: f32,
    ) -> Self {
        let start_steps = start_steps(&default, available_space, item_spacing, before_padding);
        let end_steps = end_steps(&default, available_space, item_spacing, after_padding);
        let start_shift_distance =
            start_steps
                .first()
                .zip(start_steps.last())
                .map_or(0.0, |(first, last)| {
                    (last.0[0].unadjusted_offset - first.0[0].unadjusted_offset).max(before_padding)
                });
        let end_shift_distance =
            end_steps
                .first()
                .zip(end_steps.last())
                .map_or(0.0, |(first, last)| {
                    let (first, last) = (first.0[first.0.len() - 1], last.0[last.0.len() - 1]);
                    (first.unadjusted_offset - last.unadjusted_offset).max(after_padding)
                });
        let start_shift_points = shift_points(start_shift_distance, &start_steps, true);
        let end_shift_points = shift_points(end_shift_distance, &end_steps, false);
        let mut min_item_size = default.min_size();
        let mut max_item_size = default.max_size();
        for step in start_steps.iter().chain(&end_steps) {
            min_item_size = min_item_size.min(step.min_size());
            max_item_size = max_item_size.max(step.max_size());
        }
        Self {
            default,
            start_steps,
            end_steps,
            available_space,
            item_spacing,
            start_shift_distance,
            end_shift_distance,
            start_shift_points,
            end_shift_points,
            min_item_size,
            max_item_size,
        }
    }

    /// キーラインがあり、項目の大きさが 0 でないか
    pub fn is_valid(&self) -> bool {
        !self.default.0.is_empty() && self.available_space != 0.0 && self.item_size() != 0.0
    }

    /// 焦点の項目の大きさ。項目はこの大きさで描き、キーラインの大きさで切り抜く
    pub fn item_size(&self) -> f32 {
        if self.default.0.is_empty() {
            0.0
        } else {
            self.default.first_focal().size
        }
    }

    /// 項目の間隔
    pub fn item_spacing(&self) -> f32 {
        self.item_spacing
    }

    /// カルーセルの長さ
    pub fn available_space(&self) -> f32 {
        self.available_space
    }

    /// すべての位置でのキーラインの大きさの最小と最大
    pub fn size_range(&self) -> (f32, f32) {
        (self.min_item_size, self.max_item_size)
    }

    /// `item_count` 個の項目をスクロールできる最大の量
    pub fn max_scroll_offset(&self, item_count: usize) -> f32 {
        let count = item_count as f32;
        (self.item_size() * count + self.item_spacing * (count - 1.0) - self.available_space)
            .max(0.0)
    }

    /// スクロールの位置 `scroll_offset` でのキーライン
    pub fn keylines_for_scroll_offset(
        &self,
        scroll_offset: f32,
        max_scroll_offset: f32,
    ) -> KeylineList {
        let position = scroll_offset.max(0.0);
        let start_shift = self.start_shift_distance;
        let end_shift = (max_scroll_offset - self.end_shift_distance).max(0.0);
        if (start_shift..=end_shift).contains(&position) {
            return self.default.clone();
        }
        let mut interpolation = lerp_range(1.0, 0.0, 0.0, start_shift, position);
        let mut points = &self.start_shift_points;
        let mut steps: Vec<&KeylineList> = self.start_steps.iter().collect();
        if position > end_shift {
            interpolation = lerp_range(0.0, 1.0, end_shift, max_scroll_offset, position);
            points = &self.end_shift_points;
            steps = self.end_steps.iter().collect();
            if end_shift < 0.01 && self.start_steps.len() == 2 && self.end_steps.len() == 2 {
                steps = vec![&self.start_steps[1], &self.end_steps[1]];
            }
        }
        let mut from = 0;
        let mut to = 0;
        let mut fraction = 0.0;
        let mut lower = points[0];
        for (i, &upper) in points.iter().enumerate().take(steps.len()).skip(1) {
            if interpolation <= upper {
                from = i - 1;
                to = i;
                fraction = lerp_range(0.0, 1.0, lower, upper, interpolation);
                break;
            }
            lower = upper;
        }
        KeylineList::lerp(steps[from], steps[to], fraction)
    }

    /// 項目 `index` が止まる位置での、項目の始まりのカルーセルの始まりからの距離 (Compose の `getSnapPositionOffset`)
    pub fn snap_position_offset(&self, index: usize, item_count: usize) -> f32 {
        if !self.is_valid() {
            return 0.0;
        }
        let half = self.item_size() / 2.0;
        let mut offset = (self.default.first_focal().unadjusted_offset - half).round();
        let start_last = self.start_steps.len().saturating_sub(1);
        if index <= start_last && !self.start_steps.is_empty() {
            let step = &self.start_steps[(start_last - index).min(start_last)];
            offset = (step.first_focal().unadjusted_offset - half).round();
        }
        let end_last = self.end_steps.len().saturating_sub(1);
        let last_item = item_count.saturating_sub(1);
        if !self.end_steps.is_empty()
            && index + end_last >= last_item
            && item_count > self.default.focal_count()
        {
            let step = &self.end_steps[end_last.saturating_sub(last_item - index).min(end_last)];
            offset = (step.last_focal().unadjusted_offset - half).round();
        }
        offset
    }
}

fn start_steps(
    default: &KeylineList,
    main_axis_size: f32,
    item_spacing: f32,
    before_padding: f32,
) -> Vec<KeylineList> {
    if default.0.is_empty() {
        return Vec::new();
    }
    let mut steps = vec![default.clone()];
    if default.is_first_focal_item_at_start() {
        if before_padding != 0.0 {
            steps.push(shift_for_padding(
                default,
                main_axis_size,
                item_spacing,
                before_padding,
                default.first_focal(),
                default.first_focal_index(),
            ));
        }
        return steps;
    }
    let start = default.first_non_anchor_index();
    let end = default.first_focal_index();
    if end <= start && default.first_focal().cutoff > 0.0 {
        steps.push(move_keyline(default, 0, 0, main_axis_size, item_spacing));
        return steps;
    }
    for i in 0..end.saturating_sub(start) {
        let prev = steps.last().expect("steps start with the default keylines");
        let original = start + i;
        let dst = if original > 0 {
            prev.first_index_after_focal_range_with_size(default.0[original - 1].size) - 1
        } else {
            default.0.len() - 1
        };
        let next = move_keyline(
            prev,
            default.first_non_anchor_index(),
            dst,
            main_axis_size,
            item_spacing,
        );
        steps.push(next);
    }
    if before_padding != 0.0 {
        let last = steps.pop().expect("steps start with the default keylines");
        steps.push(shift_for_padding(
            &last,
            main_axis_size,
            item_spacing,
            before_padding,
            last.first_focal(),
            last.first_focal_index(),
        ));
    }
    steps
}

fn end_steps(
    default: &KeylineList,
    main_axis_size: f32,
    item_spacing: f32,
    after_padding: f32,
) -> Vec<KeylineList> {
    if default.0.is_empty() {
        return Vec::new();
    }
    let mut steps = vec![default.clone()];
    if default.is_last_focal_item_at_end(main_axis_size) {
        if after_padding != 0.0 {
            steps.push(shift_for_padding(
                default,
                main_axis_size,
                item_spacing,
                -after_padding,
                default.last_focal(),
                default.last_focal_index(),
            ));
        }
        return steps;
    }
    let start = default.last_focal_index();
    let end = default.last_non_anchor_index();
    if end <= start && default.last_focal().cutoff > 0.0 {
        steps.push(move_keyline(default, 0, 0, main_axis_size, item_spacing));
        return steps;
    }
    for i in 0..end.saturating_sub(start) {
        let prev = steps.last().expect("steps start with the default keylines");
        let original = end - i;
        let dst = if original < default.0.len() - 1 {
            prev.last_index_before_focal_range_with_size(default.0[original + 1].size) + 1
        } else {
            0
        };
        let next = move_keyline(
            prev,
            default.last_non_anchor_index(),
            dst,
            main_axis_size,
            item_spacing,
        );
        steps.push(next);
    }
    if after_padding != 0.0 {
        let last = steps.pop().expect("steps start with the default keylines");
        steps.push(shift_for_padding(
            &last,
            main_axis_size,
            item_spacing,
            -after_padding,
            last.last_focal(),
            last.last_focal_index(),
        ));
    }
    steps
}

fn shift_for_padding(
    from: &KeylineList,
    main_axis_size: f32,
    item_spacing: f32,
    padding: f32,
    pivot: Keyline,
    pivot_index: usize,
) -> KeylineList {
    let non_anchor = from.0.iter().filter(|k| !k.is_anchor).count() as f32;
    let reduction = padding / non_anchor;
    let sizes: Vec<(f32, bool)> = from
        .0
        .iter()
        .map(|k| (k.size - reduction.abs(), k.is_anchor))
        .collect();
    let shifted = KeylineList::with_pivot(
        main_axis_size,
        item_spacing,
        pivot_index,
        pivot.offset - reduction / 2.0 + padding,
        &sizes,
    );
    KeylineList(
        shifted
            .0
            .into_iter()
            .zip(&from.0)
            .map(|(k, original)| Keyline {
                unadjusted_offset: original.unadjusted_offset,
                ..k
            })
            .collect(),
    )
}

fn move_keyline(
    from: &KeylineList,
    src: usize,
    dst: usize,
    main_axis_size: f32,
    item_spacing: f32,
) -> KeylineList {
    let forward = src > dst;
    let direction = if forward { 1.0 } else { -1.0 };
    let delta = (from.0[src].size - from.0[src].cutoff + item_spacing) * direction;
    let pivot_index = if forward {
        from.pivot_index() + 1
    } else {
        from.pivot_index().saturating_sub(1)
    };
    let pivot_offset = from.0[from.pivot_index()].offset + delta;
    let mut sizes = from.sizes();
    let moved = sizes.remove(src);
    sizes.insert(dst, moved);
    KeylineList::with_pivot(
        main_axis_size,
        item_spacing,
        pivot_index,
        pivot_offset,
        &sizes,
    )
}

fn shift_points(total: f32, steps: &[KeylineList], shifting_left: bool) -> Vec<f32> {
    let mut points = vec![0.0];
    if total == 0.0 || steps.is_empty() {
        return points;
    }
    for i in 1..steps.len() {
        let (prev, current) = (&steps[i - 1], &steps[i]);
        let shifted = if shifting_left {
            current.0[0].unadjusted_offset - prev.0[0].unadjusted_offset
        } else {
            prev.0[prev.0.len() - 1].unadjusted_offset
                - current.0[current.0.len() - 1].unadjusted_offset
        };
        let point = if i == steps.len() - 1 {
            1.0
        } else {
            points[i - 1] + shifted / total
        };
        points.push(point);
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sizes(list: &KeylineList) -> Vec<f32> {
        list.keylines().iter().map(|k| k.size).collect()
    }

    #[test]
    fn 幅_360_で大きさ_186_を目安にすると大中小を並べる() {
        let list = multi_browse(360.0, 186.0, 8.0, 10, 40.0, 56.0);
        let sizes = sizes(&list);
        assert_eq!(sizes.len(), 5, "見えない端 2 つと大中小: {sizes:?}");
        assert_eq!(sizes[0], ANCHOR_SIZE);
        assert_eq!(sizes[4], ANCHOR_SIZE);
        assert!(sizes[1] > sizes[2] && sizes[2] > sizes[3], "{sizes:?}");
        // 間隔を除いた長さに収まる
        let total: f32 = sizes[1..4].iter().sum::<f32>() + 8.0 * 2.0;
        assert!((total - 360.0).abs() < 0.01, "{total}");
        let first = list.keylines()[1];
        assert!(first.is_focal && first.offset == first.size / 2.0);
    }

    #[test]
    fn 項目が少なければ小さい項目を除いて並べ直す() {
        let list = multi_browse(360.0, 186.0, 8.0, 2, 40.0, 56.0);
        let sizes = sizes(&list);
        // 小さい項目を除くと、大きい項目 2 つと中くらいの項目 1 つが最も目安に近い
        assert_eq!(sizes.len(), 5, "{sizes:?}");
        assert_eq!(sizes[1], sizes[2]);
        assert!(sizes[3] < sizes[2] && sizes[3] > 56.0, "{sizes:?}");
    }

    #[test]
    fn 同じ大きさを並べ残りを中くらいの項目にする() {
        let list = uncontained(360.0, 150.0, 0.0);
        let sizes = sizes(&list);
        // 150 を 2 つと、残り 60 の 1.5 倍の 90 を 1 つ
        assert_eq!(sizes[1..4], [150.0, 150.0, 90.0]);
    }

    #[test]
    fn 中央に寄せたヒーローは両側に小さい項目を置く() {
        let list = hero(360.0, None, 8.0, 5, true, 40.0, 56.0);
        let sizes = sizes(&list);
        assert_eq!(sizes.len(), 5, "{sizes:?}");
        assert_eq!(sizes[1], sizes[3]);
        assert!(sizes[2] > sizes[1]);
        let focal = list.keylines()[2];
        assert!(
            (focal.offset - 180.0).abs() < 0.01,
            "焦点の項目を中央に置くこと: {focal:?}"
        );
    }

    #[test]
    fn スクロールの両端では項目を端に寄せる() {
        let list = multi_browse(360.0, 186.0, 8.0, 10, 40.0, 56.0);
        let strategy = Strategy::new(list.clone(), 360.0, 8.0, 0.0, 0.0);
        let max = strategy.max_scroll_offset(10);
        assert_eq!(strategy.keylines_for_scroll_offset(0.0, max), list);
        let end = strategy.keylines_for_scroll_offset(max, max);
        let last = end
            .keylines()
            .iter()
            .rfind(|k| !k.is_anchor)
            .copied()
            .expect("見えない端でない項目があること");
        assert!(last.is_focal, "最後には焦点の項目を右端に置くこと: {end:?}");
        assert!((last.offset + last.size / 2.0 - 360.0).abs() < 0.01);
    }

    #[test]
    fn 最初と最後の項目は端に止まる() {
        let list = multi_browse(360.0, 186.0, 8.0, 10, 40.0, 56.0);
        let strategy = Strategy::new(list, 360.0, 8.0, 0.0, 0.0);
        assert_eq!(strategy.snap_position_offset(0, 10), 0.0);
        let item = strategy.item_size();
        let last = strategy.snap_position_offset(9, 10);
        assert!((last - (360.0 - item)).abs() < 1.0, "{last} {item}");
    }
}
