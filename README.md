# md-components

[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)（Zed の UI フレームワーク）で使う、
Material Design 3（ベースライン）の UI 部品集。Windows / macOS / Linux / Web（wasm32）で同じ見た目と操作で動き、
キーボード操作とスクリーンリーダー向けの役割・状態を備え、表示文字や `.aria_label()` で名前を付けられる。

- モーション: Standard / Expressive のモーションスキーム（バネ）。OS の「視差効果を減らす」相当の設定
  （gpui の `App::reduce_motion`）で動きを止められる。
- 配色: シード色から HCT 色空間でライト / ダークの配色を作る（7 バリアント × 標準 / 中 / 高コントラスト）。
  生成した配色は全組み合わせで WCAG 2.2 のコントラスト比（文字 4.5:1、UI 部品 3:1）を満たすことをテストしている。
- トークン: 寸法・色の割り当て・形・高さ・タイプスケールは一次資料（androidx Compose Material 3 の `*Tokens.kt` など）から
  転記し、出典を [`docs/TOKENS_SOURCE.md`](docs/TOKENS_SOURCE.md) に記録している（`cargo xtask check-tokens` で突き合わせる）。
- スクロールバー: 縦・横、ドラッグ / 表示のみ、静止後のフェード。リスト・メニュー・ダイアログなどに標準で表示する（[仕様](docs/specs/scrollbar.md)）。
- 依存は `gpui` だけ（上流 Zed の git rev を固定）。フォントは同梱しない。

> 非公式の移植です。Google および Material Design チームとは関係ありません。

## crate

| crate            | 内容                                                                                     |
| ---------------- | ---------------------------------------------------------------------------------------- |
| `gpui-headless`  | 見た目を持たない操作・アクセシビリティ基盤（[独自テーマガイド](docs/headless-guide.md)） |
| `md3-color`      | HCT 色空間・配色アルゴリズム（material-color-utilities の移植。GPU 不要）                |
| `md3-tokens`     | デザイントークンの定数                                                                   |
| `md3-icons`      | Material Symbols のアイコン（よく使う 30 個）                                            |
| `md3-theme`      | テーマ（gpui の `Global`）、gpui の型への変換、OS のライト / ダークへの追従              |
| `md3-components` | 部品。feature で分割（下表）                                                             |
| `gallery`        | 全部品・全バリアントの操作できる見本アプリ                                               |
| `mail-sample`    | 部品を組み合わせたメールアプリの見本                                                     |
| `xtask`          | 開発用コマンド（std のみ）                                                               |

| feature       | 部品                                                                                                 |
| ------------- | ---------------------------------------------------------------------------------------------------- |
| `buttons`     | Button（5 種）、IconButton、FAB / ExtendedFAB、SegmentedButton                                       |
| `selection`   | Checkbox、RadioButton、Switch、Chip（4 種）、Slider、Progress（線形 / 円形）、Badge                  |
| `inputs`      | TextField（Filled / Outlined、IME 対応）、Select、SearchBar                                          |
| `overlays`    | Menu、Dialog、Tooltip、Snackbar、BottomSheet / SideSheet                                             |
| `navigation`  | TopAppBar（4 種）、NavigationBar / Rail / Drawer、NavigationSuite、Tabs                              |
| `containment` | Card（3 種）、List / LazyList、Divider                                                               |
| `expressive`  | M3 Expressive の ToggleButton、ButtonGroup（Standard / Connected）。押下・選択で形と幅がバネで変わる |

TextField / Select の必須状態は `.required(true)`、メニューボタンの開閉状態は `.menu_expanded(open)` で伝える。静的な `Text` を読み上げ可能にするには一意な `.id()` を指定する。

Checkbox の状態は `md3_components::selection::CheckboxState`、テキスト編集の状態は `md3_components::inputs::TextFieldState` を使う。Button 系、Checkbox、Switch、Chip、Card、RadioGroup、SegmentedButton、ButtonGroup、Tabs、Navigation、Slider、List / LazyList の見た目を持たない操作と a11y は `gpui-headless` が担う。

feature に関係なく使える共通部品: Scrollbar、ScrollArea、`control`。
アプリのスクロール領域に ScrollArea を使うと、Tab などで移った操作部品を見える位置へスクロールする（[利用ガイド](docs/guide.md#%E3%82%AD%E3%83%BC%E3%83%9C%E3%83%BC%E3%83%89%E3%83%95%E3%82%A9%E3%83%BC%E3%82%AB%E3%82%B9%E3%82%92%E8%A6%8B%E3%81%88%E3%82%8B%E4%BD%8D%E7%BD%AE%E3%81%B8%E7%A7%BB%E3%81%99)）。
既定ではすべて有効。各部品の仕様とキー操作は [`docs/specs/`](docs/specs/) にある。

## 使い方

ワークスペースの `Cargo.toml`（gpui は md-components と同じ rev にそろえる。型が別物になるため）:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "37faef81f74fc1bb08be9939c4cffc8ede6ced2f" }
md3-components = { git = "https://github.com/tunamaguro/gpui-md-components" }
```

rev を自分で揃えたくなければ、再公開を使う。`md3_components::gpui` は常に、
`md3_components::gpui_platform` は feature `platform`（既定では無効）を付けたときに使える。

```toml
[dependencies]
md3-components = { git = "https://github.com/tunamaguro/gpui-md-components", features = ["platform"] }
```

```rust
use gpui::{App, KeyBinding, Window, actions, div, prelude::*};
use md3_components::buttons::{Button, ButtonVariant};
use md3_components::theme::{self, Argb, Contrast, SchemeVariant, SetMd3ThemeExt, Theme};

actions!(app, [FocusNext, FocusPrev]);

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // Tab / Shift-Tab でフォーカスを移す（アプリ側で割り当てる）。
        cx.bind_keys([
            KeyBinding::new("tab", FocusNext, None),
            KeyBinding::new("shift-tab", FocusPrev, None),
        ]);
        // テーマの初期化と、部品のキー操作（テキスト編集・メニュー・ダイアログ）の割り当て。
        md3_components::init(cx);
        cx.set_md3_theme(
            Theme::from_seed(Argb::from_rgb(0x67, 0x50, 0xA4))
                .variant(SchemeVariant::TonalSpot)
                .contrast(Contrast::Standard)
                .build(),
        );
        cx.open_window(Default::default(), |window, cx| {
            // OS のライト / ダークの切り替えに追従する。
            theme::follow_window_appearance(window, cx).detach();
            cx.new(|_| Root)
        })
        .unwrap();
    });
}

struct Root;

impl Render for Root {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .on_action(|_: &FocusNext, window, cx| window.focus_next(cx))
            .on_action(|_: &FocusPrev, window, cx| window.focus_prev(cx))
            .child(
                // 操作できる部品には ID（ElementId）を指定する。表示文字を名前とし、必要なら `.aria_label()` を使う。
                Button::new("save", "保存")
                    .variant(ButtonVariant::Filled)
                    .on_click(|_, _, _| println!("saved")),
            )
    }
}
```

詳しくは [利用ガイド](docs/guide.md)。

## 見本（examples）

`examples/` に見本アプリを置いている。`gallery` は各バリアントの操作できる見本と disabled、該当すれば selected / error の見本を並べる。
hover / focused / pressed はマウスやキーボードで操作して確かめる。

```sh
cargo run -p gallery                    # デスクトップ
cargo run -p mail-sample                # メールアプリの見本
MD3_GALLERY_PAGE=inputs cargo run -p gallery   # ページを指定して起動
MD3_GALLERY_LIST_PAGES=1 target/debug/gallery   # ビルド後、ページキーの一覧を表示
MD3_REDUCE_MOTION=1 cargo run -p gallery       # 「視差効果を減らす」を有効にして起動（画面でも切り替えられる）
cargo xtask web                         # 全見本の wasm 版を target/web/ にビルド（`nix develop` の環境で）
python3 -m http.server -d target/web    # 静的ファイルとして配信して http://localhost:8000/ を開く
```

main の Web 版は GitHub Pages で公開している: https://tunamaguro.github.io/gpui-md-components/

開いたページで見本を選ぶ。`gallery/` や `mail/` を直接開いてもよい。WebGPU が使えない環境では見本の URL に `?renderer=webgl` を付けると WebGL2 で描く（例: `gallery/?renderer=webgl`）。

### CI のビルドを試す（動作確認用）

push・PR ごとに GitHub Actions の「Gallery」ワークフローが見本をビルドし、実行結果のページの Artifacts に置く
（動作確認用で配布物ではないので、3 日で消える）。

| Artifact                 | 動かし方                                                                                                        |
| ------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `gallery-linux-x86_64`   | `tar xzf gallery-linux-x86_64.tar.gz && ./gallery`（Vulkan のドライバと libxkbcommon が要る）                   |
| `gallery-macos-arm64`    | `tar xzf gallery-macos-arm64.tar.gz && xattr -d com.apple.quarantine gallery; ./gallery`（署名していないため）  |
| `gallery-windows-x86_64` | `gallery.exe` を実行（署名していないため SmartScreen の警告が出る）                                             |
| `examples-web`           | 展開したフォルダで `python3 -m http.server` を実行し、`http://localhost:8000/` を開く（`file://` では動かない） |

## 既知の制限

GPUI 側の制約によるものが多い。全体は [利用ガイドの「既知の制限」](docs/guide.md#%E6%97%A2%E7%9F%A5%E3%81%AE%E5%88%B6%E9%99%90)、
アクセシビリティの検証結果は [`docs/a11y-audit.md`](docs/a11y-audit.md) にある。主なもの:

- **Web 版にはアクセシビリティツリーが無い。** gpui は wasm で AccessKit を組み込まない（キャンバスに描くだけ）ため、
  Web ではスクリーンリーダーが部品を読めない。キーボード操作とフォーカスリングは動く。
- Web ではシステムフォントを読めないため、アプリがフォントのデータを渡す必要がある。
- M3 Expressive は ToggleButton・ButtonGroup とバネのモーションだけ（多角形の形のモーフなどは未実装）。
  動きは gpui の `App::reduce_motion` に従うが、gpui は OS の「視差効果を減らす」設定を読まないため、アプリが渡す。

## 開発

開発環境は Nix の flake（`nix develop`、または direnv の `use flake`）で用意する。作業の手順と規約は
[`AGENTS.md`](AGENTS.md) と [`docs/agents/`](docs/agents/) にある（未対応事項は [`docs/agents/backlog.md`](docs/agents/backlog.md)）。

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p md3-components --target wasm32-unknown-unknown
cargo xtask check-tokens
```

## ライセンス

Apache License 2.0（[`LICENSE`](LICENSE)）。Material Design のトークン値・配色アルゴリズム・アイコンは
Apache License 2.0 の一次資料から転記・移植しており、各ファイルの先頭に出典を記している。
