# トークン出典対応表

`md3-tokens` に転記したすべての値の出典。`cargo xtask check-tokens` がこの表と定数を突き合わせる。

## 取得元

- `androidx@b8256a52`: androidx/androidx `b8256a524b9fe4bb6cd0fc7c72aad9dd08877ca6` の `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/`（`FontWeight.kt` は `compose/ui/ui-text/src/commonMain/kotlin/androidx/compose/ui/text/font/`）
- `material-web@86cb0d48`: material-components/material-web `86cb0d484d0cda1193f09bb3ee70b8e2ed8e6a79`
- `material-components-web@6ceadb81`: material-components/material-components-web `6ceadb81df508ac08ab007c52c4d3dc1d811cc15`

## 判断の記録

- 数値は Compose を正とする（`docs/agents/resources.md`）。状態レイヤーの不透明度（hover 0.08 / focus 0.1 / pressed 0.1 / dragged 0.16）は material-web の `_md-sys-state.scss` とも一致することを確認した。
- タイプスケールは基本の 15 種だけを転記し、Expressive 用の `*Emphasized*` は含めない。
- 無効状態のコンテナの不透明度はコンポーネントごとに違う（0.1 / 0.12 など）ので、共通のトークンにはしない。内容（文字・アイコン）の 0.38 は全コンポーネントで共通で、material-web の `md.sys.state.disabled.state-layer-opacity` と同じ値。
- 密度は Compose と material-web にトークンがない。material-components-web の `mdc-density` を出典とし、1 段 4dp ずつ小さくする計算（`_density.scss` L111）を `Density::apply` に移した。

## 対応表

| Rust の定数 | 値 | 出典ファイル | 行 / セクション | 取得したコミットまたは日付 |
| --- | --- | --- | --- | --- |
| md3_tokens::typography::BODY_LARGE_TYPEFACE | Plain | TypeScaleTokens.kt | L24 BodyLargeFont | androidx@b8256a52 |
| md3_tokens::typography::BODY_LARGE_LINE_HEIGHT | 24.0 | TypeScaleTokens.kt | L27 BodyLargeLineHeight | androidx@b8256a52 |
| md3_tokens::typography::BODY_LARGE_SIZE | 16.0 | TypeScaleTokens.kt | L30 BodyLargeSize | androidx@b8256a52 |
| md3_tokens::typography::BODY_LARGE_TRACKING | 0.5 | TypeScaleTokens.kt | L33 BodyLargeTracking | androidx@b8256a52 |
| md3_tokens::typography::BODY_LARGE_WEIGHT | WeightRegular | TypeScaleTokens.kt | L36 BodyLargeWeight | androidx@b8256a52 |
| md3_tokens::typography::BODY_MEDIUM_TYPEFACE | Plain | TypeScaleTokens.kt | L39 BodyMediumFont | androidx@b8256a52 |
| md3_tokens::typography::BODY_MEDIUM_LINE_HEIGHT | 20.0 | TypeScaleTokens.kt | L42 BodyMediumLineHeight | androidx@b8256a52 |
| md3_tokens::typography::BODY_MEDIUM_SIZE | 14.0 | TypeScaleTokens.kt | L45 BodyMediumSize | androidx@b8256a52 |
| md3_tokens::typography::BODY_MEDIUM_TRACKING | 0.2 | TypeScaleTokens.kt | L48 BodyMediumTracking | androidx@b8256a52 |
| md3_tokens::typography::BODY_MEDIUM_WEIGHT | WeightRegular | TypeScaleTokens.kt | L51 BodyMediumWeight | androidx@b8256a52 |
| md3_tokens::typography::BODY_SMALL_TYPEFACE | Plain | TypeScaleTokens.kt | L54 BodySmallFont | androidx@b8256a52 |
| md3_tokens::typography::BODY_SMALL_LINE_HEIGHT | 16.0 | TypeScaleTokens.kt | L57 BodySmallLineHeight | androidx@b8256a52 |
| md3_tokens::typography::BODY_SMALL_SIZE | 12.0 | TypeScaleTokens.kt | L60 BodySmallSize | androidx@b8256a52 |
| md3_tokens::typography::BODY_SMALL_TRACKING | 0.4 | TypeScaleTokens.kt | L63 BodySmallTracking | androidx@b8256a52 |
| md3_tokens::typography::BODY_SMALL_WEIGHT | WeightRegular | TypeScaleTokens.kt | L66 BodySmallWeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_LARGE_TYPEFACE | Brand | TypeScaleTokens.kt | L69 DisplayLargeFont | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_LARGE_LINE_HEIGHT | 64.0 | TypeScaleTokens.kt | L72 DisplayLargeLineHeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_LARGE_SIZE | 57.0 | TypeScaleTokens.kt | L75 DisplayLargeSize | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_LARGE_TRACKING | -0.2 | TypeScaleTokens.kt | L78 DisplayLargeTracking | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_LARGE_WEIGHT | WeightRegular | TypeScaleTokens.kt | L81 DisplayLargeWeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_MEDIUM_TYPEFACE | Brand | TypeScaleTokens.kt | L84 DisplayMediumFont | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_MEDIUM_LINE_HEIGHT | 52.0 | TypeScaleTokens.kt | L87 DisplayMediumLineHeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_MEDIUM_SIZE | 45.0 | TypeScaleTokens.kt | L90 DisplayMediumSize | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_MEDIUM_TRACKING | 0.0 | TypeScaleTokens.kt | L93 DisplayMediumTracking | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_MEDIUM_WEIGHT | WeightRegular | TypeScaleTokens.kt | L96 DisplayMediumWeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_SMALL_TYPEFACE | Brand | TypeScaleTokens.kt | L99 DisplaySmallFont | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_SMALL_LINE_HEIGHT | 44.0 | TypeScaleTokens.kt | L102 DisplaySmallLineHeight | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_SMALL_SIZE | 36.0 | TypeScaleTokens.kt | L105 DisplaySmallSize | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_SMALL_TRACKING | 0.0 | TypeScaleTokens.kt | L108 DisplaySmallTracking | androidx@b8256a52 |
| md3_tokens::typography::DISPLAY_SMALL_WEIGHT | WeightRegular | TypeScaleTokens.kt | L111 DisplaySmallWeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_LARGE_TYPEFACE | Brand | TypeScaleTokens.kt | L114 HeadlineLargeFont | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_LARGE_LINE_HEIGHT | 40.0 | TypeScaleTokens.kt | L117 HeadlineLargeLineHeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_LARGE_SIZE | 32.0 | TypeScaleTokens.kt | L120 HeadlineLargeSize | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_LARGE_TRACKING | 0.0 | TypeScaleTokens.kt | L123 HeadlineLargeTracking | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_LARGE_WEIGHT | WeightRegular | TypeScaleTokens.kt | L126 HeadlineLargeWeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_MEDIUM_TYPEFACE | Brand | TypeScaleTokens.kt | L129 HeadlineMediumFont | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_MEDIUM_LINE_HEIGHT | 36.0 | TypeScaleTokens.kt | L132 HeadlineMediumLineHeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_MEDIUM_SIZE | 28.0 | TypeScaleTokens.kt | L135 HeadlineMediumSize | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_MEDIUM_TRACKING | 0.0 | TypeScaleTokens.kt | L138 HeadlineMediumTracking | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_MEDIUM_WEIGHT | WeightRegular | TypeScaleTokens.kt | L141 HeadlineMediumWeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_SMALL_TYPEFACE | Brand | TypeScaleTokens.kt | L144 HeadlineSmallFont | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_SMALL_LINE_HEIGHT | 32.0 | TypeScaleTokens.kt | L147 HeadlineSmallLineHeight | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_SMALL_SIZE | 24.0 | TypeScaleTokens.kt | L150 HeadlineSmallSize | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_SMALL_TRACKING | 0.0 | TypeScaleTokens.kt | L153 HeadlineSmallTracking | androidx@b8256a52 |
| md3_tokens::typography::HEADLINE_SMALL_WEIGHT | WeightRegular | TypeScaleTokens.kt | L156 HeadlineSmallWeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_LARGE_TYPEFACE | Plain | TypeScaleTokens.kt | L159 LabelLargeFont | androidx@b8256a52 |
| md3_tokens::typography::LABEL_LARGE_LINE_HEIGHT | 20.0 | TypeScaleTokens.kt | L162 LabelLargeLineHeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_LARGE_SIZE | 14.0 | TypeScaleTokens.kt | L165 LabelLargeSize | androidx@b8256a52 |
| md3_tokens::typography::LABEL_LARGE_TRACKING | 0.1 | TypeScaleTokens.kt | L168 LabelLargeTracking | androidx@b8256a52 |
| md3_tokens::typography::LABEL_LARGE_WEIGHT | WeightMedium | TypeScaleTokens.kt | L171 LabelLargeWeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_MEDIUM_TYPEFACE | Plain | TypeScaleTokens.kt | L174 LabelMediumFont | androidx@b8256a52 |
| md3_tokens::typography::LABEL_MEDIUM_LINE_HEIGHT | 16.0 | TypeScaleTokens.kt | L177 LabelMediumLineHeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_MEDIUM_SIZE | 12.0 | TypeScaleTokens.kt | L180 LabelMediumSize | androidx@b8256a52 |
| md3_tokens::typography::LABEL_MEDIUM_TRACKING | 0.5 | TypeScaleTokens.kt | L183 LabelMediumTracking | androidx@b8256a52 |
| md3_tokens::typography::LABEL_MEDIUM_WEIGHT | WeightMedium | TypeScaleTokens.kt | L186 LabelMediumWeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_SMALL_TYPEFACE | Plain | TypeScaleTokens.kt | L189 LabelSmallFont | androidx@b8256a52 |
| md3_tokens::typography::LABEL_SMALL_LINE_HEIGHT | 16.0 | TypeScaleTokens.kt | L192 LabelSmallLineHeight | androidx@b8256a52 |
| md3_tokens::typography::LABEL_SMALL_SIZE | 11.0 | TypeScaleTokens.kt | L195 LabelSmallSize | androidx@b8256a52 |
| md3_tokens::typography::LABEL_SMALL_TRACKING | 0.5 | TypeScaleTokens.kt | L198 LabelSmallTracking | androidx@b8256a52 |
| md3_tokens::typography::LABEL_SMALL_WEIGHT | WeightMedium | TypeScaleTokens.kt | L201 LabelSmallWeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_LARGE_TYPEFACE | Brand | TypeScaleTokens.kt | L204 TitleLargeFont | androidx@b8256a52 |
| md3_tokens::typography::TITLE_LARGE_LINE_HEIGHT | 28.0 | TypeScaleTokens.kt | L207 TitleLargeLineHeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_LARGE_SIZE | 22.0 | TypeScaleTokens.kt | L210 TitleLargeSize | androidx@b8256a52 |
| md3_tokens::typography::TITLE_LARGE_TRACKING | 0.0 | TypeScaleTokens.kt | L213 TitleLargeTracking | androidx@b8256a52 |
| md3_tokens::typography::TITLE_LARGE_WEIGHT | WeightRegular | TypeScaleTokens.kt | L216 TitleLargeWeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_MEDIUM_TYPEFACE | Plain | TypeScaleTokens.kt | L219 TitleMediumFont | androidx@b8256a52 |
| md3_tokens::typography::TITLE_MEDIUM_LINE_HEIGHT | 24.0 | TypeScaleTokens.kt | L222 TitleMediumLineHeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_MEDIUM_SIZE | 16.0 | TypeScaleTokens.kt | L225 TitleMediumSize | androidx@b8256a52 |
| md3_tokens::typography::TITLE_MEDIUM_TRACKING | 0.2 | TypeScaleTokens.kt | L228 TitleMediumTracking | androidx@b8256a52 |
| md3_tokens::typography::TITLE_MEDIUM_WEIGHT | WeightMedium | TypeScaleTokens.kt | L231 TitleMediumWeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_SMALL_TYPEFACE | Plain | TypeScaleTokens.kt | L234 TitleSmallFont | androidx@b8256a52 |
| md3_tokens::typography::TITLE_SMALL_LINE_HEIGHT | 20.0 | TypeScaleTokens.kt | L237 TitleSmallLineHeight | androidx@b8256a52 |
| md3_tokens::typography::TITLE_SMALL_SIZE | 14.0 | TypeScaleTokens.kt | L240 TitleSmallSize | androidx@b8256a52 |
| md3_tokens::typography::TITLE_SMALL_TRACKING | 0.1 | TypeScaleTokens.kt | L243 TitleSmallTracking | androidx@b8256a52 |
| md3_tokens::typography::TITLE_SMALL_WEIGHT | WeightMedium | TypeScaleTokens.kt | L246 TitleSmallWeight | androidx@b8256a52 |
| md3_tokens::typography::WEIGHT_BOLD | 700 | TypefaceTokens.kt / FontWeight.kt | L31 WeightBold = FontWeight.Bold; FontWeight.kt L66, L47 | androidx@b8256a52 |
| md3_tokens::typography::WEIGHT_MEDIUM | 500 | TypefaceTokens.kt / FontWeight.kt | L34 WeightMedium = FontWeight.Medium; FontWeight.kt L62, L43 | androidx@b8256a52 |
| md3_tokens::typography::WEIGHT_REGULAR | 400 | TypefaceTokens.kt / FontWeight.kt | L37 WeightRegular = FontWeight.Normal; FontWeight.kt L60, L41 | androidx@b8256a52 |
| md3_tokens::shape::CORNER_NONE | 0.0 | ShapeTokens.kt | L80 CornerValueNone | androidx@b8256a52 |
| md3_tokens::shape::CORNER_EXTRA_SMALL | 4.0 | ShapeTokens.kt | L76 CornerValueExtraSmall | androidx@b8256a52 |
| md3_tokens::shape::CORNER_SMALL | 8.0 | ShapeTokens.kt | L81 CornerValueSmall | androidx@b8256a52 |
| md3_tokens::shape::CORNER_MEDIUM | 12.0 | ShapeTokens.kt | L79 CornerValueMedium | androidx@b8256a52 |
| md3_tokens::shape::CORNER_LARGE | 16.0 | ShapeTokens.kt | L77 CornerValueLarge | androidx@b8256a52 |
| md3_tokens::shape::CORNER_EXTRA_LARGE | 28.0 | ShapeTokens.kt | L74 CornerValueExtraLarge | androidx@b8256a52 |
| md3_tokens::shape::CORNER_FULL | Full | ShapeTokens.kt | L46 CornerFull = CircleShape | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL0 | 0.0 | ElevationTokens.kt | L24 Level0 | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL1 | 1.0 | ElevationTokens.kt | L27 Level1 | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL2 | 3.0 | ElevationTokens.kt | L30 Level2 | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL3 | 6.0 | ElevationTokens.kt | L33 Level3 | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL4 | 8.0 | ElevationTokens.kt | L36 Level4 | androidx@b8256a52 |
| md3_tokens::elevation::LEVEL5 | 12.0 | ElevationTokens.kt | L39 Level5 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_EXTRA_LONG1 | 700 | MotionTokens.kt | L24 DurationExtraLong1 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_EXTRA_LONG2 | 800 | MotionTokens.kt | L25 DurationExtraLong2 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_EXTRA_LONG3 | 900 | MotionTokens.kt | L26 DurationExtraLong3 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_EXTRA_LONG4 | 1000 | MotionTokens.kt | L27 DurationExtraLong4 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_LONG1 | 450 | MotionTokens.kt | L28 DurationLong1 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_LONG2 | 500 | MotionTokens.kt | L29 DurationLong2 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_LONG3 | 550 | MotionTokens.kt | L30 DurationLong3 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_LONG4 | 600 | MotionTokens.kt | L31 DurationLong4 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_MEDIUM1 | 250 | MotionTokens.kt | L32 DurationMedium1 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_MEDIUM2 | 300 | MotionTokens.kt | L33 DurationMedium2 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_MEDIUM3 | 350 | MotionTokens.kt | L34 DurationMedium3 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_MEDIUM4 | 400 | MotionTokens.kt | L35 DurationMedium4 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_SHORT1 | 50 | MotionTokens.kt | L36 DurationShort1 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_SHORT2 | 100 | MotionTokens.kt | L37 DurationShort2 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_SHORT3 | 150 | MotionTokens.kt | L38 DurationShort3 | androidx@b8256a52 |
| md3_tokens::motion::DURATION_SHORT4 | 200 | MotionTokens.kt | L39 DurationShort4 | androidx@b8256a52 |
| md3_tokens::motion::EASING_EMPHASIZED | 0.2, 0.0, 0.0, 1.0 | MotionTokens.kt | L40 EasingEmphasizedCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_EMPHASIZED_ACCELERATE | 0.3, 0.0, 0.8, 0.15 | MotionTokens.kt | L43 EasingEmphasizedAccelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_EMPHASIZED_DECELERATE | 0.05, 0.7, 0.1, 1.0 | MotionTokens.kt | L46 EasingEmphasizedDecelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_LEGACY | 0.4, 0.0, 0.2, 1.0 | MotionTokens.kt | L49 EasingLegacyCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_LEGACY_ACCELERATE | 0.4, 0.0, 1.0, 1.0 | MotionTokens.kt | L52 EasingLegacyAccelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_LEGACY_DECELERATE | 0.0, 0.0, 0.2, 1.0 | MotionTokens.kt | L55 EasingLegacyDecelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_LINEAR | 0.0, 0.0, 1.0, 1.0 | MotionTokens.kt | L58 EasingLinearCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_STANDARD | 0.2, 0.0, 0.0, 1.0 | MotionTokens.kt | L61 EasingStandardCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_STANDARD_ACCELERATE | 0.3, 0.0, 1.0, 1.0 | MotionTokens.kt | L64 EasingStandardAccelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::motion::EASING_STANDARD_DECELERATE | 0.0, 0.0, 0.0, 1.0 | MotionTokens.kt | L67 EasingStandardDecelerateCubicBezier | androidx@b8256a52 |
| md3_tokens::state::DRAGGED_STATE_LAYER_OPACITY | 0.16 | StateTokens.kt | L22 DraggedStateLayerOpacity | androidx@b8256a52 |
| md3_tokens::state::FOCUS_STATE_LAYER_OPACITY | 0.1 | StateTokens.kt | L23 FocusStateLayerOpacity | androidx@b8256a52 |
| md3_tokens::state::HOVER_STATE_LAYER_OPACITY | 0.08 | StateTokens.kt | L24 HoverStateLayerOpacity | androidx@b8256a52 |
| md3_tokens::state::PRESSED_STATE_LAYER_OPACITY | 0.1 | StateTokens.kt | L25 PressedStateLayerOpacity | androidx@b8256a52 |
| md3_tokens::state::DISABLED_CONTENT_OPACITY | 0.38 | material-web tokens/versions/latest/sass/_md-sys-state.scss | L19 md.sys.state.disabled.state-layer-opacity | material-web@86cb0d48 |
| md3_tokens::density::INTERVAL | 4.0 | material-components-web packages/mdc-density/_density.scss | L27 $interval | material-components-web@6ceadb81 |
