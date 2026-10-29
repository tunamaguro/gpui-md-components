//! モーション: イージング（3 次ベジェ曲線）と時間。

// 値は docs/TOKENS_SOURCE.md に記載の一次資料から転記した。`cargo xtask check-tokens` で突き合わせる。

use std::time::Duration;

/// CSS の `cubic-bezier(x1, y1, x2, y2)` と同じ 3 次ベジェ曲線のイージング。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// 1 つ目の制御点の x。
    pub x1: f32,
    /// 1 つ目の制御点の y。
    pub y1: f32,
    /// 2 つ目の制御点の x。
    pub x2: f32,
    /// 2 つ目の制御点の y。
    pub y2: f32,
}

impl CubicBezier {
    /// 制御点から作る。
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

/// イージング emphasized。
pub const EASING_EMPHASIZED: CubicBezier = CubicBezier::new(0.2, 0.0, 0.0, 1.0);

/// イージング emphasized accelerate。
pub const EASING_EMPHASIZED_ACCELERATE: CubicBezier = CubicBezier::new(0.3, 0.0, 0.8, 0.15);

/// イージング emphasized decelerate。
pub const EASING_EMPHASIZED_DECELERATE: CubicBezier = CubicBezier::new(0.05, 0.7, 0.1, 1.0);

/// イージング legacy。
pub const EASING_LEGACY: CubicBezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0);

/// イージング legacy accelerate。
pub const EASING_LEGACY_ACCELERATE: CubicBezier = CubicBezier::new(0.4, 0.0, 1.0, 1.0);

/// イージング legacy decelerate。
pub const EASING_LEGACY_DECELERATE: CubicBezier = CubicBezier::new(0.0, 0.0, 0.2, 1.0);

/// イージング linear。
pub const EASING_LINEAR: CubicBezier = CubicBezier::new(0.0, 0.0, 1.0, 1.0);

/// イージング standard。
pub const EASING_STANDARD: CubicBezier = CubicBezier::new(0.2, 0.0, 0.0, 1.0);

/// イージング standard accelerate。
pub const EASING_STANDARD_ACCELERATE: CubicBezier = CubicBezier::new(0.3, 0.0, 1.0, 1.0);

/// イージング standard decelerate。
pub const EASING_STANDARD_DECELERATE: CubicBezier = CubicBezier::new(0.0, 0.0, 0.0, 1.0);

/// 時間 short1。
pub const DURATION_SHORT1: Duration = Duration::from_millis(50);

/// 時間 short2。
pub const DURATION_SHORT2: Duration = Duration::from_millis(100);

/// 時間 short3。
pub const DURATION_SHORT3: Duration = Duration::from_millis(150);

/// 時間 short4。
pub const DURATION_SHORT4: Duration = Duration::from_millis(200);

/// 時間 medium1。
pub const DURATION_MEDIUM1: Duration = Duration::from_millis(250);

/// 時間 medium2。
pub const DURATION_MEDIUM2: Duration = Duration::from_millis(300);

/// 時間 medium3。
pub const DURATION_MEDIUM3: Duration = Duration::from_millis(350);

/// 時間 medium4。
pub const DURATION_MEDIUM4: Duration = Duration::from_millis(400);

/// 時間 long1。
pub const DURATION_LONG1: Duration = Duration::from_millis(450);

/// 時間 long2。
pub const DURATION_LONG2: Duration = Duration::from_millis(500);

/// 時間 long3。
pub const DURATION_LONG3: Duration = Duration::from_millis(550);

/// 時間 long4。
pub const DURATION_LONG4: Duration = Duration::from_millis(600);

/// 時間 extra-long1。
pub const DURATION_EXTRA_LONG1: Duration = Duration::from_millis(700);

/// 時間 extra-long2。
pub const DURATION_EXTRA_LONG2: Duration = Duration::from_millis(800);

/// 時間 extra-long3。
pub const DURATION_EXTRA_LONG3: Duration = Duration::from_millis(900);

/// 時間 extra-long4。
pub const DURATION_EXTRA_LONG4: Duration = Duration::from_millis(1000);

/// バネのモーションの指定。Compose の `spring(dampingRatio, stiffness)` と同じ意味で、質量は 1。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSpec {
    /// 減衰比（1 で行き過ぎなし、1 未満で行き過ぎて戻る）。
    pub damping_ratio: f32,
    /// 剛性（大きいほど速く目標に着く）。
    pub stiffness: f32,
}

impl SpringSpec {
    /// 減衰比と剛性から作る。
    pub const fn new(damping_ratio: f32, stiffness: f32) -> Self {
        Self {
            damping_ratio,
            stiffness,
        }
    }
}

/// モーションスキーム standard のバネ default spatial（位置・大きさ・形）。
pub const STANDARD_SPRING_DEFAULT_SPATIAL: SpringSpec = SpringSpec::new(0.9, 700.0);

/// モーションスキーム standard のバネ default effects（色・不透明度）。
pub const STANDARD_SPRING_DEFAULT_EFFECTS: SpringSpec = SpringSpec::new(1.0, 1600.0);

/// モーションスキーム standard のバネ fast spatial（位置・大きさ・形）。
pub const STANDARD_SPRING_FAST_SPATIAL: SpringSpec = SpringSpec::new(0.9, 1400.0);

/// モーションスキーム standard のバネ fast effects（色・不透明度）。
pub const STANDARD_SPRING_FAST_EFFECTS: SpringSpec = SpringSpec::new(1.0, 3800.0);

/// モーションスキーム standard のバネ slow spatial（位置・大きさ・形）。
pub const STANDARD_SPRING_SLOW_SPATIAL: SpringSpec = SpringSpec::new(0.9, 300.0);

/// モーションスキーム standard のバネ slow effects（色・不透明度）。
pub const STANDARD_SPRING_SLOW_EFFECTS: SpringSpec = SpringSpec::new(1.0, 800.0);

/// モーションスキーム expressive のバネ default spatial（位置・大きさ・形）。
pub const EXPRESSIVE_SPRING_DEFAULT_SPATIAL: SpringSpec = SpringSpec::new(0.8, 380.0);

/// モーションスキーム expressive のバネ default effects（色・不透明度）。
pub const EXPRESSIVE_SPRING_DEFAULT_EFFECTS: SpringSpec = SpringSpec::new(1.0, 1600.0);

/// モーションスキーム expressive のバネ fast spatial（位置・大きさ・形）。
pub const EXPRESSIVE_SPRING_FAST_SPATIAL: SpringSpec = SpringSpec::new(0.6, 800.0);

/// モーションスキーム expressive のバネ fast effects（色・不透明度）。
pub const EXPRESSIVE_SPRING_FAST_EFFECTS: SpringSpec = SpringSpec::new(1.0, 3800.0);

/// モーションスキーム expressive のバネ slow spatial（位置・大きさ・形）。
pub const EXPRESSIVE_SPRING_SLOW_SPATIAL: SpringSpec = SpringSpec::new(0.8, 200.0);

/// モーションスキーム expressive のバネ slow effects（色・不透明度）。
pub const EXPRESSIVE_SPRING_SLOW_EFFECTS: SpringSpec = SpringSpec::new(1.0, 800.0);
