# アーキテクチャ

## ワークスペース構成

リポジトリ名は `md-components`。その中に以下の crate を置く。

```
md-components/
├── Cargo.toml              # [workspace] と [workspace.dependencies]（gpui の rev 固定はここ1か所）
├── AGENTS.md               # エージェント向けの作業指示（入口）
├── .agents/skills/         # 繰り返す作業の手順（skill）
├── crates/
│   ├── md3-color/          # HCT / CAM16 / TonalPalette / DynamicScheme（純 Rust、依存ゼロ）
│   ├── md3-tokens/         # タイプスケール・角丸・高さ・モーション・状態不透明度・密度（純 Rust）
│   ├── md3-icons/          # 使うアイコンの SVG バイト列と IconName enum（純 Rust）
│   ├── md3-theme/          # Theme（gpui Global）、gpui 型への変換、OS 外観追従
│   └── md3-components/     # StateLayer / FocusRing / 各コンポーネント
├── gallery/                # 全部品の見本アプリ（gpui_platform を起動に使い、wasm-bindgen を使うのはここだけ。examples/ は未作成）
├── xtask/                  # 開発用コマンド（std のみ）: check-tokens、web
├── tools/a11y-tree/        # Linux の AT-SPI ツリーを書き出すスクリプト（nixpkgs の道具を使う。依存ではない）
├── nix/                    # flake のモジュール（devshell、パッケージ）
└── docs/
    ├── agents/             # このディレクトリ
    ├── specs/              # コンポーネントごとの仕様要約（component-workflow.md 参照）
    ├── TOKENS_SOURCE.md    # 転記した値と出典の対応表
    ├── guide.md            # 利用者向けガイド
    └── a11y-audit.md       # アクセシビリティの監査記録
```

## 依存の向き

```
gallery / examples ──> md3-components ──> md3-theme ──> md3-tokens ──> md3-color
        │                   │   │              │
        │                   │   └──> md3-icons │
        ├─> gpui_platform   └────────────────> gpui (git rev 固定)
        └─> wasm-bindgen（wasm32 のみ）
```

- `md3-color` / `md3-tokens` / `md3-icons` は **gpui に依存しない**。GPU なし・wasm でもテストが回る。
- gpui に触れるのは `md3-theme` と `md3-components` だけ。GPUI の API 変更の影響はこの 2 crate に閉じる。
- ライブラリ crate は `gpui_platform` / `wasm-bindgen` に依存しない。例外として `md3-components` は feature `platform`
  （既定で無効）で `gpui_platform` を持ち、再公開だけする（コードからは使わない）。
- `md3-components` は `gpui` を `md3_components::gpui` として再公開する。利用者が rev を揃えずに使えるようにするため。

## 依存方針

- 外部 crate: `gpui` のみ（`md3-components` の feature `platform` は `gpui_platform` の再公開に限る。`gallery` / `examples` は `gpui_platform` と、wasm32 ターゲットに限り `wasm-bindgen` も可。`#[wasm_bindgen(start)]` のエントリポイントにだけ使う）。
- `Cargo.toml` の例:

```toml
[workspace.dependencies]
# default features は切る（gpui は windows-manifest で build 時に embed-resource を使い、
# gpui_platform は multithreaded で wasm に nightly を要求するため）
gpui = { git = "https://github.com/zed-industries/zed", rev = "<固定する rev>", default-features = false, features = ["font-kit", "wayland", "x11"] }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "<同じ rev>", default-features = false, features = ["font-kit", "wayland", "x11"] }
wasm-bindgen = "<上流 zed と同じ要求版>"
```

- rev の更新は専用 PR で行い、gallery の目視確認と全テスト通過を条件にする。
- 自作するもの: HCT / CAM16 / 配色スキーム、トークン、状態レイヤー合成、フォーカスリング、cubic-bezier イージング、アイコン埋め込み、密度計算、コントラスト比計算。
- コード生成は `xtask` で行い生成物をコミットする（`build.rs` 禁止）。

## Rust の設計規約

1. **不正状態を型で排除する**
   - 操作可能な要素はコンストラクタで `ElementId` を必須にする（a11y の安定 ID になる）。
   - アイコンのみのボタンはラベル必須: `IconButton::new(id, icon, label)`。
1. **バリアントは enum**
   - `ButtonStyle::{Filled, Tonal, Elevated, Outlined, Text}` など。
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

## md3-components の共有部品

部品を作るときはまずこれらを使う。

| モジュール                    | 役割                                                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `state_layer.rs` / `focus.rs` | 状態レイヤーとフォーカスリング。角が不揃いな部品は `.corners(Corners)` を渡す                                             |
| `style.rs`                    | `StyledExt`（`corners` など）                                                                                             |
| `motion.rs`                   | 色をテーマのバネで補間する `motion::color`（`Theme::spring` と組み合わせる）                                              |
| `control.rs`                  | 操作部品の共通処理（フォーカス・hover・押下、矢印移動の `step`）                                                          |
| `scroll.rs`                   | 入れ子スクロール（公開の `NestedScroll`）、オーバーレイ本文（`content`）、フォーカスした項目を見せる（`reveal_on_focus`） |
| `scrollbar.rs`                | `Scrollbar` と `ScrollbarSource`                                                                                          |
| `expressive/press.rs`         | 押下の状態機械（押下・解除・キャンセル）                                                                                  |

GPUI の挙動で注意する点は [`gpui-notes.md`](gpui-notes.md)。

## コンポーネント実装例（API 名は固定 rev で要確認）

```rust
/// ボタンの見た目の種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonStyle { #[default] Filled, Tonal, Elevated, Outlined, Text }

/// MD3 のボタン。
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    style: ButtonStyle,
    leading_icon: Option<IconName>,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self { /* 既定値 */ }
    pub fn style(mut self, style: ButtonStyle) -> Self { self.style = style; self }
    pub fn disabled(mut self, disabled: bool) -> Self { self.disabled = disabled; self }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.md3().button(self.style); // 状態別の色をトークンから算出
        div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .h(cx.md3().density().button_height())
            .rounded_full()
            .bg(colors.container)
            // hover / pressed / focus は StateLayer で合成
    }
}
```

## テーマとフォントのカスタマイズ

テーマは「シード色から生成 → 利用者が部分的に上書き」の 2 段構え。フォントはライブラリが持たない。

```rust
let theme = Theme::from_seed(Argb::from_rgb(0x67, 0x50, 0xA4))
    .variant(SchemeVariant::TonalSpot)
    .contrast(Contrast::Standard)
    .appearance(Appearance::System)          // OS のライト / ダークに追従
    .font_family("Noto Sans JP")            // 未指定ならシステムフォント
    .type_scale(|s| s.body_large.size(px(15.)))
    .override_color(ColorRole::Primary, Argb::from_rgb(0x00, 0x5F, 0xAF))
    .density(Density::Compact)               // -1 相当
    .build();

cx.set_md3_theme(theme); // 以後 cx.md3() で参照、変更時は再描画
```

上書き可能な項目: シード色、スキームのバリアント、コントラスト、外観、フォントファミリ、タイプスケール、個別ロール色、密度、角丸。
`Theme::check_contrast()` は WCAG 比を下回るロールの組を返す（開発時の警告用）。
