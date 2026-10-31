# アーキテクチャ

## ワークスペース構成

リポジトリ名は `md-components`。その中に以下の crate を置く。

```
md-components/
├── Cargo.toml              # [workspace] と [workspace.dependencies]（gpui の rev 固定はここ1か所）
├── AGENTS.md               # エージェント向けの作業指示（入口）
├── .agents/skills/         # 繰り返す作業の手順（skill）
├── crates/
│   ├── gpui-headless/     # 見た目を持たない a11y・操作基盤（gpui のみ依存）
│   ├── md3-color/          # HCT / CAM16 / TonalPalette / DynamicScheme（純 Rust、依存ゼロ）
│   ├── md3-tokens/         # タイプスケール・角丸・高さ・モーション・状態不透明度・密度（純 Rust）
│   ├── md3-icons/          # 使うアイコンの SVG バイト列と IconName enum（純 Rust）
│   ├── md3-theme/          # Theme（gpui Global）、gpui 型への変換、OS 外観追従
│   └── md3-components/     # StateLayer / FocusRing / 各コンポーネント
├── examples/mail/          # ライブラリ利用者向けメールアプリ（デスクトップ）
├── gallery/                # 全部品の見本アプリ（gpui_platform を起動に使い、wasm-bindgen を使うのはここだけ）
├── xtask/                  # 開発用コマンド（std のみ）: check-tokens、web
├── tools/a11y-tree/        # Linux の AT-SPI ツリーを書き出すスクリプト（nixpkgs の道具を使う。依存ではない）
├── nix/                    # flake のモジュール（devshell、パッケージ）
└── docs/
    ├── agents/             # このディレクトリ
    ├── specs/              # コンポーネントごとの仕様要約（component-workflow.md 参照）
    ├── TOKENS_SOURCE.md    # 転記した値と出典の対応表
    ├── guide.md            # 利用者向けガイド
    ├── headless-guide.md   # 独自テーマの作り方
    ├── references/         # 参照用の資料（androidx、aria-practices、material-web は submodule）
    └── a11y-audit.md       # アクセシビリティの監査記録
```

## 依存の向き

```
mail-sample ──────────> md3-components（platform feature）
gallery ─────────────> md3-components ──> md3-theme ──> md3-tokens ──> md3-color
        │                   │   │              │
        │                   ├──> gpui-headless ──> gpui
        │                   │   └──> md3-icons │
        ├─> gpui_platform   └────────────────> gpui (git rev 固定)
        └─> wasm-bindgen（wasm32 のみ）
```

- `md3-color` / `md3-tokens` / `md3-icons` は **gpui に依存しない**。GPU なし・wasm でもテストが回る。
- ライブラリで gpui に触れるのは `gpui-headless`、`md3-theme` と `md3-components`。サンプルアプリは `md3-components` の再公開を使う。
- ライブラリ crate は `gpui_platform` / `wasm-bindgen` に依存しない。例外として `md3-components` は feature `platform`
  （既定で無効）で `gpui_platform` を持ち、再公開だけする（コードからは使わない）。
- `md3-components` は `gpui` を `md3_components::gpui` として再公開する。利用者が rev を揃えずに使えるようにするため。

## 依存方針

- 使ってよい外部 crate は `AGENTS.md` の「絶対に守るルール」のとおり。`wasm-bindgen` は `#[wasm_bindgen(start)]` のエントリポイントにだけ使う。
- gpui の default features は切る（`Cargo.toml` のコメントを参照）。
- rev の更新は専用 PR で行い、gallery の目視確認と全テスト通過を条件にする。
- 自作するもの: HCT / CAM16 / 配色スキーム、トークン、状態レイヤー合成、フォーカスリング、cubic-bezier イージング、アイコン埋め込み、密度計算、コントラスト比計算。
- コード生成は `xtask` で行い生成物をコミットする（`build.rs` 禁止）。

## Rust の設計規約

1. **不正状態を型で排除する**
   - 操作可能な要素はコンストラクタで `ElementId` を必須にする（a11y の安定 ID になる）。
   - アイコンのみのボタンには読み上げ用の名前を指定する: `IconButton::new(id, icon).aria_label(label)`。
1. **バリアントは enum**
   - `ButtonVariant::{Filled, Tonal, Elevated, Outlined, Text}` など。
   - `ColorRole` は全ロールを列挙し、`Scheme::color(role)` を網羅 `match` で実装する（ワイルドカード `_` 禁止）。
1. **単位は newtype**
   - `Tone(f32)`（0〜100 に丸め）、`Hue`、`Chroma`、`Dp`。生の `f32` を公開 API に出さない。
1. **GPUI の慣習に合わせる**
   - 状態を持たない部品: `#[derive(IntoElement)]` + `RenderOnce` + メソッドチェーンのビルダー。
   - 状態を持つ部品（TextField 等）: `Entity` + `Render`。
1. **テーマは拡張トレイトで取得**
   - `trait Md3ThemeExt { fn md3(&self) -> &Theme; }` を `App` に実装し `cx.md3()` で参照。
1. **安全性**
   - 全 crate `#![forbid(unsafe_code)]`、`#![deny(missing_docs)]`。
   - `md3-color` は panic しない設計（入力を丸める、`Option` を返す）。
1. **wasm 互換**
   - 描画経路で `std::thread`、ブロッキング、`Mutex` を使わない。`std::time::Instant` は wasm で使えないので GPUI の時間 API を使う。
1. **feature で分割**
   - `md3-components` は `buttons`, `selection`, `inputs`, `overlays`, `navigation`, `containment`, `expressive` の feature を持つ。default は全部有効。

## 共有部品

部品を作るときはまずこれらを使う。

| モジュール                                      | 役割                                                                                                               |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `state_layer.rs` / `focus.rs`                   | 状態レイヤーとフォーカスリング。角が不揃いな部品は `.corners(Corners)` を渡す                                      |
| `anchor_tooltip.rs`                             | 部品の基準要素に Tooltip を付け、`overlays` feature の分岐をまとめる                                               |
| `style.rs`                                      | `StyledExt`（`corners` など）                                                                                      |
| `motion.rs`                                     | 色をテーマのバネで補間する `motion::color`（`Theme::spring` と組み合わせる）                                       |
| `gpui-headless` の `control` / `focus` / `a11y` | 操作部品の共通処理、フォーカス、アクセシビリティ情報、矢印移動の `step`                                            |
| `gpui-headless` の `control::Roving`            | 複合部品の Tab 停止位置とフォーカス移動を共有する内部型。キーの意味は各部品が決める                                |
| `gpui-headless` の `scroll`                     | 入れ子スクロール（`NestedScroll`）、表示領域（`ScrollScope`）、キーボードで移った要素を見せる（`reveal_on_focus`） |
| `scroll.rs`                                     | オーバーレイ本文（`content`）                                                                                      |
| `gpui-headless` の `scrollbar`                  | スクロール位置の供給元、つまみの幾何とポインタ操作                                                                 |
| `scrollbar.rs`                                  | スクロールバーの色・形・フェード                                                                                   |
| `gpui-headless` の `keys`                       | 編集・オーバーレイのアクションとキー割り当て                                                                       |
| `gpui-headless` の `press`                      | 押下の状態機械（押下・解除）。最短時間は呼び出し側から渡す                                                         |

`control()` と Card は共通の操作処理で、有効な要素に `reveal_on_focus` を付ける。TextField / SearchBar にも付ける。
List（高さ制限あり）・Drawer・Menu・オーバーレイ本文は `ScrollScope` で領域を包む。アプリ側のスクロール領域も同じように包む。
スコープはレイアウトを増やさず、同期的な prepaint の間だけ Global のスタックに存在する。内側から順に両軸を調整し、
実際の移動量を外側へ反映する。フォーカスリングの外周を含め、最大オフセットまでに制限する。
遅延描画するオーバーレイには背後の領域を渡さない。LazyList の仮想化領域は `scroll_to_item` を維持する。

GPUI の挙動で注意する点は [`gpui-notes.md`](gpui-notes.md)。

## 部品の形とテーマ

部品の実装は `gpui-headless` の要素に見た目を足す形にする（`headless.md`）。例は `crates/md3-components/src/buttons/button.rs`。
テーマの作り方と上書きできる項目は [`docs/guide.md`](../guide.md) の「テーマ」。

`Theme` と `ThemeBuilder` の設定は非公開の `ThemeConfig` にまとめ、既定値は `ThemeConfig::new` に置く。
設定項目を足すときはそこに追加し、公開 setter / getter と組み立て直しのテストを更新する。
