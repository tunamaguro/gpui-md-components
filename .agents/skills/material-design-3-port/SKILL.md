---
name: material-design-3-port
description: Material Design 3 のトークン値・計算ロジック・コンポーネント仕様を Rust / GPUI に移植するときに使う。同梱の AndroidX Compose、material-web、material-color-utilities の submodule を一次資料として読み、値の依存先、状態別の振る舞い、単位、参照リビジョンを確認する。
---

# Material Design 3 の移植

このリポジトリの MD3 ベースラインの移植に使う。トークン値・寸法・計算式は記憶や見た目から補わず、一次資料の該当箇所を開いて確認する。

## 参照資料

パスはこの `SKILL.md` のあるディレクトリを基準に解決する。

- [references/androidx](references/androidx): Compose Material3 の生成トークンと実装。数値の第一候補。
- [references/material-web](references/material-web): Sass トークンと TypeScript / Sass の実装。Compose に値がない項目や Web 表現の参照先。
- [references/material-color-utilities](references/material-color-utilities): HCT / CAM16 / TonalPalette / DynamicScheme などの配色アルゴリズムと公式テスト。
- [m3.material.io](https://m3.material.io): Foundations,Styles,Componentsを参照する
- [参照先の探し方](references/source-map.md): 対象のトークン・計算ロジックを探すときに読む。
- [移植時の判断と検証](references/porting-notes.md): 値の食い違い、単位変換、状態・形状・モーションを移植するときに読む。

## 参照元を確定する

1. 対象 crate と既存の出典コメントを読み、コンポーネント、バリアント、対象状態、ベースライン / Expressive の範囲を確認する。既存の数値やコメントも、一次資料との照合なしに正としない。

1. submodule の実際のコミットと作業ツリーの変更を確認する。以下はスキルのディレクトリで実行する。

   ```sh
   git -C references/androidx rev-parse HEAD
   git -C references/androidx status --short
   git -C references/material-web rev-parse HEAD
   git -C references/material-web status --short
   git -C references/material-color-utilities rev-parse HEAD
   git -C references/material-color-utilities status --short
   ```

   未初期化なら、リポジトリルートで必要なものだけ初期化する。

   ```sh
   git submodule update --init -- \
     .agents/skills/material-design-3-port/references/androidx \
     .agents/skills/material-design-3-port/references/material-web \
     .agents/skills/material-design-3-port/references/material-color-utilities
   ```

1. Kotlin の `VERSION`、Sass の版・生成コンテキストも確認する。同じコミットでもファイルごとにトークンの世代が異なる。`latest` というディレクトリ名だけで採用を決めない。

1. トークンの数値は対象世代の **Compose > material-web**、振る舞いは **対象の MD3 仕様 > 各実装** を基本にする。Compose に値がない影やフォーカス指標などは material-web を調べる。配色アルゴリズムは material-color-utilities の対象仕様とテストを正とする。世代・プラットフォームが異なる場合は、差と採用理由を記録する。

1. 仕様を確認する必要があれば [m3.material.io](https://m3.material.io/) の該当項目を読む。本文を取得できなければ Compose の実装で補い、仕様本文は未確認と報告する。submodule のリビジョン更新は移植作業と分け、更新依頼がない限り固定値を維持する。

## 値とロジックを追う

- 生成トークンだけでなく、それを使うコンポーネントの defaults・状態分岐・レイアウト計算まで読む。トークンにない余白・最小寸法・アニメーションの選択も実装から探す。
- 色・形・タイポグラフィ・elevation の参照先をたどる。色を固定 RGB に潰さず `ColorRole` として対応させ、ライト / ダークやテーマの上書きを維持する。
- material-web は公開トークンの `@use` / `@forward` 先と、`values()` の依存・追加・改名・除外を確認する。生成ファイルの値と実際のコンポーネントが利用する値を区別する。
- 計算式は入力条件、初期値、クランプ、丸め、時間・長さの単位、終了条件まで読む。Android / DOM 固有の API は GPUI の責務に置き換え、観測できる結果を合わせる。
- 配色計算は material-color-utilities の TypeScript 実装から依存する数式・定数とテストまで追い、`md3-color` の対象である 2021 仕様との対応を確認する。
- 同梱の submodule で根拠が見つからなければ、その範囲を明記する。密度など別の出典が必要な項目を、Compose 由来として記録しない。

## 実装と出典

- 定数は `md3-tokens`、配色計算は `md3-color`、テーマ解決と GPUI 型への変換は `md3-theme`、部品の描画は `md3-components` の責務を基準にする。既存のトークン・変換処理を調べてから配置する。
- コンポーネントを移植するときは、対応する Cargo feature でモジュール宣言と再公開を囲む（`#[cfg(feature = "...")]`）。その feature を無効にした構成でもビルドできるようにする。
- 移植・転記したファイルには、移植元のリポジトリ、ソース内の相対パス、シンボル、コミット、トークンの版を出典として記録する。既存ファイルに異なる出典を追加する場合は、どの値・処理に対応するかを明確にする。
  - 可能な限り移植・転記した箇所と、自身で実装した部分をファイル単位で区別する
- コピー元の著作権表示・ライセンス表示を保持する。説明コメントは日本語で書く。ライセンス文書の扱いは実際のコピー元のファイルを確認する。
- 固定値の転記と、GPUI 向けの変換・調整を区別して説明する。上流にない挙動や制限を、上流と同じ仕様だと書かない。

## 検証と報告

- 計算ロジックは上流のテスト入力・期待値や独立に評価した式と照合し、必要な境界条件を検証する。Rust 側の定数をそのまま期待値に使うだけのテストは根拠にしない。
- 状態選択は対象の enabled / disabled / hovered / focused / pressed、該当する selected / error を確認する。寸法は必要に応じて描画結果でも確認し、定数の一致だけで表示の一致を主張しない。
- 現在の構成で実行できる、変更した crate のテスト・整形・ビルドを行う。上流のビルド全体を動かす必要はない。
- 完了時は採用した出典とコミット、値の不一致と判断、GPUI 向け調整、実施した検証、未確認の範囲を簡潔に報告する。
