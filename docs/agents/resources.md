# 資料の場所（何を「正」とするか）

**仕様は m3.material.io、数値は Compose のトークン生成ファイル、配色は material-color-utilities のテストを正とする。**
食い違ったら 数値: Compose > material-web、振る舞い: m3.material.io > 各実装 の順で優先し、判断を `docs/TOKENS_SOURCE.md` に残す。

| 用途 | 資料 | 場所 | ライセンス | 使い方 |
| --- | --- | --- | --- | --- |
| 仕様（正） | MD3 ガイドライン | https://m3.material.io の Foundations / Styles / Components（各コンポーネントの Specs タブ） | Apache 2.0 / CC BY 4.0 | 寸法・状態・振る舞い・a11y 要件の一次資料 |
| トークン値（正） | Compose Material3 トークン | `androidx/androidx` の `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/` | Apache 2.0 | `*Tokens.kt` は公式トークン DB から生成された値。コンポーネント単位で Rust の `const` に転記 |
| トークン値（副） | material-web | https://github.com/material-components/material-web の `tokens/` 配下（保守モード） | Apache 2.0 | Web 表現（px、状態不透明度）との突き合わせ |
| 配色アルゴリズム | material-color-utilities | https://github.com/material-foundation/material-color-utilities（TypeScript / Kotlin / Dart 版とテスト） | Apache 2.0 | HCT・CAM16・TonalPalette・DynamicScheme を移植。公式テストを Rust に移す |
| 配色の Rust 先行例 | material-colors / material-color-utils | https://docs.rs/material-colors 、https://docs.rs/material-color-utils | MIT / Apache 2.0 | **依存しない**。型設計とテスト期待値の比較だけに使う |
| アイコン | Material Symbols | https://github.com/google/material-design-icons の `symbols/web/<name>/materialsymbolsoutlined/*.svg` | Apache 2.0 | 使うアイコンだけ `crates/md3-icons/assets/` にコピーし `include_bytes!` |
| GPUI 本体 | gpui / gpui_platform / gpui_web | https://github.com/zed-industries/zed の `crates/gpui`、`crates/gpui_platform`、`crates/gpui/examples/`（a11y 例を含む） | Apache 2.0 | API の正。**固定した rev のソースを grep する** |
| GPUI a11y | Accessibility ガイド | gpui の `_accessibility` モジュール doc と `src/window/a11y.rs` | Apache 2.0 | `role` / `aria_label` / `on_a11y_action` / `a11y_synthetic_children` の使い方 |
| 振る舞いの参照実装 | gpui-kit の gpui-base | https://github.com/longbridge/gpui-kit の `crates/base`、`docs/ARCHITECTURE.md` | Apache 2.0 | オーバーレイ・フォーカストラップ・IME 入力・WASM 対応の実装例。**依存しない**。コードをコピーしたらファイル先頭に出典コメントを書く |
| a11y 仕様 | WAI-ARIA APG | https://www.w3.org/WAI/ARIA/apg/patterns/ | W3C 文書 | 各コンポーネントの役割とキーボード操作の正 |
| a11y 基準 | WCAG 2.2 | https://www.w3.org/TR/WCAG22/ | W3C 文書 | コントラスト比（文字 4.5:1、大きい文字・UI 部品 3:1）、フォーカス可視 |
| a11y 基盤 | AccessKit | https://accesskit.dev | MIT / Apache 2.0 | GPUI が内部で使う。ロール一覧の確認用 |

## 転記ルール

1. 値を書く前に、上表の一次資料の該当ファイル・行を開く。記憶や推測で書かない。
2. `docs/TOKENS_SOURCE.md` に次の形式で 1 行追記する。

```
| Rust の定数 | 値 | 出典ファイル | 行 / セクション | 取得したコミットまたは日付 |
| md3_tokens::button::filled::CONTAINER_HEIGHT | 40 | FilledButtonTokens.kt | ContainerHeight | androidx@<commit> |
```

3. 移植したコード（配色アルゴリズムなど）はファイル先頭に次のコメントを書く。`NOTICE` ファイルは置かないので、出典と著作権表示はこのコメントが唯一の記録になる。

```rust
// Ported from material-color-utilities (https://github.com/material-foundation/material-color-utilities)
// Source: typescript/hct/hct_solver.ts @ <commit>
// Copyright 2021 Google LLC. Licensed under the Apache License, Version 2.0.
```

4. "Material" は Google の商標。crate 名・型名に使わない。doc コメントで「Material Design 3 の仕様に基づく」と書くのは可。
