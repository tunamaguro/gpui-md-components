# 概要

## 目的

GPUI 単体では見た目の統一とアクセシビリティが弱い。そこで既存のデザインシステムを移植し、
Windows / macOS / Linux / Web で同じ見た目・同じ操作性・スクリーンリーダー対応の UI 部品群を提供する。

## 選定: Material Design 3（ベースライン）

比較候補: Material Design 3、Fluent 2、Carbon、shadcn/Radix 系、Apple HIG、Primer、Ant Design、libadwaita。

MD3 を選んだ理由:

1. ガイドライン・画像・資料が Apache 2.0 または CC BY 4.0。Material Symbols も Apache 2.0。トークン値の転記と配色アルゴリズムの移植に法的な曖昧さがない。
2. OS 固有の見た目を真似ないので、4 プラットフォームどこでも成立する。
3. HCT 色空間による配色生成でコントラストを保証でき、標準 / 中 / 高の 3 段階コントラストを持つ。
4. トークンが機械可読（Compose の `*Tokens.kt`、material-web の tokens）で、正解を参照しやすい。

不採用の主な理由: Fluent 2 は Windows 色が強く Segoe UI が再配布不可。Carbon は業務アプリ特化で固定 4 テーマ。
Apple HIG は移植用途の許諾がない。libadwaita は GTK 前提。

## スコープ

### v1.0 でやる

- MD3 ベースライン（2021〜2024 年の M3 仕様）の配色・トークン・主要コンポーネント
- デスクトップ向けの密度調整（既定は密度 -1）
- a11y（役割・名前・状態・アクション・キーボード操作・フォーカス管理）
- Web（wasm32）での表示と基本操作

### v1.0 でやらない

- M3 Expressive（バネ物理モーション、形状モーフ）→ M8（任意）。M8 でバネのモーションと ToggleButton / ButtonGroup は実装済み、形状モーフは未実装
- フォント同梱
- モバイル（iOS / Android）
- リップル以外の装飾的エフェクト

## GPUI の前提（2026 年 10 月時点）

- 上流 Zed の `crates/gpui` を git rev 固定で使う。crates.io 版や非公式フォーク（gpui-ce, gpui-unofficial, gpui-box 等）は使わない。混ぜると型が別物になる。
- Zed は 2026 年 2 月に Zed 以外向けの GPUI 開発を後回しにすると表明している。汎用機能の上流追加は期待せず、足りないものは自前で持つ。
- AccessKit 統合は 2026 年 5 月にマージ。macOS / Windows / Linux / Web 対応だが Zed 内でも実験的扱い。
- Web（wasm32-unknown-unknown）プラットフォームは 2026 年 2 月にマージ。アルファ品質。ブロッキング処理不可。
- Linux の描画は wgpu、Windows は DirectX 11 + DirectWrite。
