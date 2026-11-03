# 参照先の探し方

このファイルのリンクは `references/` を基準とする。AndroidX 全体を読み込まず、まず対象モジュールに検索範囲を絞る。

## AndroidX Compose

Material3 の起点は [compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/](androidx/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/)。以下の表の相対パスはこのディレクトリ基準。

| 調べるもの             | 起点                                                                                                              | 追うもの                                                                |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| コンポーネントの値     | `tokens/*Tokens.kt`                                                                                               | 色・形・書体のキー、elevation、不透明度、寸法、ファイルごとの `VERSION` |
| 色ロールと既定色       | `tokens/ColorSchemeKeyTokens.kt`、`tokens/ColorLightTokens.kt`、`tokens/ColorDarkTokens.kt`、`ColorScheme.kt`     | `fromToken`、ライト / ダーク、コンポーネントの colors / defaults        |
| 形                     | `tokens/ShapeKeyTokens.kt`、`tokens/ShapeTokens.kt`、`Shapes.kt`                                                  | 部分的な角丸、start / end、サイズによって決まる Full                    |
| 書体                   | `tokens/TypeScaleTokens.kt`、`tokens/TypefaceTokens.kt`、`Typography.kt`                                          | サイズ、行高、字間、ウェイト、基本スタイルと Emphasized の区別          |
| 状態・高さ             | `tokens/StateTokens.kt`、`tokens/ElevationTokens.kt`、`Surface.kt`                                                | 状態レイヤー、部品固有の disabled 不透明度、描画への適用                |
| モーション             | `tokens/MotionTokens.kt`、`tokens/StandardMotionTokens.kt`、`tokens/ExpressiveMotionTokens.kt`、`MotionScheme.kt` | duration / easing と spring の区別、Spatial / Effects、部品側での選択   |
| トークン外の寸法・分岐 | `<Component>.kt`、`<Component>Defaults.kt` があればそれも                                                         | padding、最小寸法、アイコン配置、状態の優先順位、精密ポインタによる分岐 |
| 操作領域               | `InteractiveComponentSize.kt` と各コンポーネント                                                                  | 表示寸法と最小操作領域の関係                                            |

Material3 の外へ処理が委譲されている場合は、次も読む。

- [animation-core](androidx/compose/animation/animation-core/src/commonMain/kotlin/androidx/compose/animation/core/): `SpringSimulation.kt` の `updateValues`、`FloatAnimationSpec.kt`、`SpringEstimation.kt`、`Easing.kt`。バネの応答、初速、終了判定、イージングの計算元。
- [material-ripple](androidx/compose/material/material-ripple/src/commonMain/kotlin/androidx/compose/material/ripple/): Material3 の `Ripple.kt` から呼び出しをたどり、`RippleAnimation.kt` / `Ripple.kt` / `CommonRipple.kt` で半径・中心・フェード・クリップを確認する。
- [ui-unit](androidx/compose/ui/ui-unit/src/commonMain/kotlin/androidx/compose/ui/unit/): `Density` など、dp / sp の変換が関係するとき。
- テストは対象モジュールの `src/commonTest`、`src/androidHostTest`、`src/androidDeviceTest` など、実在するディレクトリから対象シンボル名で検索する。期待値だけでなく前提・許容誤差も読む。

## material-web

| 調べるもの               | 起点                                                                                                                                       | 注意点                                                                                  |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| 公開トークン             | [tokens/](material-web/tokens/) の `_md-comp-*.scss`、`_md-sys-*.scss`、`_md-ref-*.scss`                                                   | `values()`、`$_default`、CSS カスタムプロパティの fallback を追う                       |
| 生成トークン             | [tokens/versions/v0_192/](material-web/tokens/versions/v0_192/)、[tokens/versions/latest/sass/](material-web/tokens/versions/latest/sass/) | 呼び出し元の import 先を読む。map を返す版と Sass 変数の版を混同しない                  |
| 互換パス                 | [tokens/v0_192/](material-web/tokens/v0_192/)                                                                                              | `@forward` 先を開く                                                                     |
| トークンの加工           | [tokens/internal/](material-web/tokens/internal/) の `_validate.scss`、`_shape.scss`                                                       | supported / unsupported、改名、追加、論理方向の角                                       |
| コンポーネントの振る舞い | `button/internal/`、`slider/internal/` など対象部品のディレクトリ                                                                          | `.ts` の状態・イベントと `.scss` のトークン適用・寸法計算、対応する `*_test.ts`         |
| 影                       | [elevation/internal/\_elevation.scss](material-web/elevation/internal/_elevation.scss)                                                     | レベルごとの y / blur / spread / opacity と補間・クランプ                               |
| リップル                 | [ripple/internal/ripple.ts](material-web/ripple/internal/ripple.ts)                                                                        | 要素寸法、対角長、初期サイズ、scale、入力状態と時間。半径と直径を変数名だけで判断しない |
| アニメーション           | [internal/motion/](material-web/internal/motion/)                                                                                          | Web Animations API に渡す keyframes、duration、easing、完了・取消し                     |
| フォーカス               | [focus/](material-web/focus/)、`tokens/_md-comp-focus-ring.scss`                                                                           | リングの形状、太さ、オフセット、状態別の表示                                            |

例: Filled Button の定義を調べる。以下はスキルのディレクトリで実行する。

```sh
compose=references/androidx/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3
web=references/material-web
rg -n 'ContainerColor|DisabledContainerOpacity|ContainerHeight' "$compose/tokens/FilledButtonTokens.kt"
rg -n 'FilledButtonTokens|contentPadding|MinHeight|defaultTextButtonColors' "$compose/Button.kt"
rg -n '@use|values|new-tokens|renamed-tokens|unsupported-tokens' "$web/tokens/_md-comp-filled-button.scss"
```

## material-color-utilities

起点は同梱の [typescript/](material-color-utilities/typescript/)。`md3-color` の出典コメントと対象仕様を確認してから、以下の実装と同じディレクトリにある `*_test.ts` を読む。Material3 の色ロール定義や Android の動的色取得だけではアルゴリズム全体の根拠にならない。

| 調べるもの         | 起点                                                                                                                            | 追うもの                                                                                                               |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| HCT / CAM16        | [hct/](material-color-utilities/typescript/hct/) の `hct.ts`、`cam16.ts`、`viewing_conditions.ts`                               | ARGB との相互変換、観測条件、許容誤差、往復変換のテスト                                                                |
| HCT ソルバー       | `hct/hct_solver.ts`                                                                                                             | 色域内に収める条件、探索、行列・定数、丸め                                                                             |
| パレット           | [palettes/](material-color-utilities/typescript/palettes/) の `tonal_palette.ts`、`core_palette.ts`                             | 色相・彩度とトーンの対応、キーとなる色                                                                                 |
| 動的スキーム       | [dynamiccolor/](material-color-utilities/typescript/dynamiccolor/)、[scheme/](material-color-utilities/typescript/scheme/)      | `dynamic_scheme.ts`、`dynamic_color.ts`、`material_dynamic_colors.ts`、`color_spec_2021.ts`、各 variant の生成とテスト |
| コントラスト       | [contrast/](material-color-utilities/typescript/contrast/)、`dynamiccolor/contrast_curve.ts`、`dynamiccolor/tone_delta_pair.ts` | 明るさ・トーン・コントラスト比、背景との関係、トーンの差                                                               |
| 色温度・色の補正   | [temperature/](material-color-utilities/typescript/temperature/)、[dislike/](material-color-utilities/typescript/dislike/)      | 補色・類似色、補正条件とテスト                                                                                         |
| 数値・色空間の基礎 | [utils/](material-color-utilities/typescript/utils/) の `color_utils.ts`、`math_utils.ts`                                       | ARGB のビット演算、sRGB の線形化、XYZ / L\*、クランプ、色相の正規化                                                    |

同じ checkout に 2021 / 2025 / 2026 仕様のコードが共存する。`md3-color` は 2021 仕様を対象とするため、`specVersion` と delegate の分岐を確認し、実装とテストの対象仕様を揃える。TypeScript と Kotlin / Java などを比較するときも、対応する仕様・入力・精度を確認する。

## 同梱の submodule の外に根拠があるもの

- 密度計算は `md3-tokens/src/density.rs` の出典を確認する。Compose / material-web 内に同じ密度トークンがあると仮定しない。
- GPUI の API・計算・単位は、ルート `Cargo.toml` が固定する Zed のリビジョンの `crates/gpui/src` を確認する。`md3-theme/src/motion.rs` のように既存の変換処理があれば先に読む。
- アクセシビリティの役割・状態・キー操作は、[a11y スキルの APG](../../a11y/references/aria-practices/content/patterns/) と必要な仕様を確認する。Android のタッチ操作だけからデスクトップのキー操作を決めない。
