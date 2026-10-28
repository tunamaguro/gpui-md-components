# 利用ガイド

md3-components を GPUI のアプリで使うための手引き。部品ごとの仕様（寸法・状態・キー操作）は
[`docs/specs/`](specs/) にある。

## 1. 準備

- `gpui` は md-components と**同じ git rev** を使う（`Cargo.toml` の `[workspace.dependencies]` を参照）。
  rev が違うと gpui の型が別物になり、部品を要素として渡せない。
- 必要な部品群だけを feature で選べる（`buttons` / `selection` / `inputs` / `overlays` / `navigation` / `containment`、既定は全部）。
  `inputs` と `navigation` は `overlays` を、`overlays` は `buttons` を自動で有効にする。

起動時に 1 回:

```rust
md3_components::init(cx); // 既定のテーマの設定と、部品のキー操作の割り当て
```

`init` はテキスト編集・メニュー・ダイアログのキー（矢印、Home / End、Esc、Tab の閉じ込めなど）を
部品の key context に割り当てる。アプリの Tab / Shift-Tab（`window.focus_next` / `focus_prev`）は
**`init` より前に**割り当てる。後から割り当てた方が優先されるため、順序が逆だとダイアログの中で Tab が外に出る。

## 2. テーマ

```rust
use md3_components::theme::{Appearance, Argb, Contrast, Density, SchemeVariant, SetMd3ThemeExt, Theme};

cx.set_md3_theme(
    Theme::from_seed(Argb::from_rgb(0x00, 0x6A, 0x6A))
        .variant(SchemeVariant::TonalSpot) // Monochrome / Neutral / TonalSpot / Vibrant / Expressive / Fidelity / Content
        .contrast(Contrast::Standard)      // Standard / Medium / High
        .appearance(Appearance::System)    // System / Light / Dark
        .density(Density::Minus1)          // デスクトップ向けの密度
        .font_family("Noto Sans JP")       // 省略時はシステムフォント
        .build(),
);
```

- `Appearance::System` のときは `theme::follow_window_appearance(window, cx)` の返り値を保持している間、
  OS のライト / ダークに追従する。
- 部品の描画からは `cx.md3()` でテーマを参照する（`theme.color(ColorRole::Primary)` など）。
- 生成した配色はどの組み合わせでも WCAG 2.2 のコントラスト比を満たす（`crates/md3-theme/tests/contrast.rs`）。
  `ThemeBuilder::override_color` で個別の色を差し替えたときは崩れることがあるので、開発時に
  `theme.check_contrast()` で確かめる（基準を下回るロールの組を返す）。
- 高コントラストを望む利用者のために、`Contrast` を設定から選べるようにすることを勧める。

## 3. 部品の使い方の共通ルール

- **操作できる部品は ID と名前を必ず取る。** `Button::new("save", "保存")` の第 1 引数が `ElementId`、
  第 2 引数が見た目のラベル兼スクリーンリーダーが読む名前。アイコンだけの部品（`IconButton` など）も名前が必須。
- **ID はフレームをまたいで安定させる。** リストの項目はデータのキーから ID を作る（index にしない）。
  ID が変わるとフォーカスと状態（開閉・押下など）が失われ、読み上げ位置も飛ぶ。
- **同じ ID・同じ `TextFieldState` を同時に 2 か所に描かない。** フォーカスハンドルを 2 つの要素が共有し、
  アクセシビリティが有効なときに gpui のデバッグビルドの検査で panic する（リリースビルドでは警告）。
- 部品は値を持たない。選択・入力値・開閉は利用者が持ち、`selected(..)` などで渡して `on_change` などで受け取る。
  例外は TextField（`TextFieldState` を `Entity` で持つ）と、フォーカスや一時的な表示状態（部品が内部で保持）。
- 無効な部品（`.disabled(true)`）は Tab で止まらず、操作できない。
- 見本用の `forced(..)` / `inline()` は gallery 向けで、アプリでは使わない（`#[doc(hidden)]`）。

## 4. キーボード操作

全部品の割り当ては [`docs/agents/a11y.md`](agents/a11y.md) の表にある。要点:

| 部品 | 操作 |
| --- | --- |
| ボタン類・Checkbox・Switch・Chip・Card | Tab で移動、Enter / Space で押す |
| RadioGroup・SegmentedButton・Tabs | Tab でグループに入り、矢印で移って選ぶ（Home / End） |
| NavigationBar / Rail / Drawer | 矢印でフォーカスだけを移し、Enter / Space で選ぶ |
| List / LazyList | 矢印で移る（端で止まる）、Enter / Space で選ぶ |
| Slider | 矢印で 1 段、Home / End で端 |
| Menu・Select | 開くと最初の項目にフォーカス、矢印で移動、Enter で選ぶ、Esc で閉じて元に戻る |
| Dialog・モーダルな Sheet / Drawer | 開くと中にフォーカスし、Tab は中に閉じ込める。Esc で閉じ、開いた要素にフォーカスを戻す |

## 5. Web（wasm32）

- ブラウザでは gpui がシステムフォントを読めない。起動時に `cx.text_system().add_fonts(..)` でフォントの
  データを渡す。日本語などを表示するなら、その字を持つフォントも渡す
  （gpui の既定の候補に入っている「Noto Sans」などを選ぶと、テーマでフォント名を指定しなくてよい）。
  gallery の実装（`gallery/src/main.rs` の `start` と `xtask/web/index.html`）が例になる。
- 描画は WebGPU、使えなければ WebGL2。`gpui_platform::application_with_web_backend` で指定できる。
- 共有メモリ（atomics）を使わない通常のビルドなら、配信に COOP / COEP ヘッダーは要らない。
- 編集のショートカットは OS を区別できないため Ctrl（macOS のブラウザでも Ctrl）。

## 既知の制限

### アクセシビリティ（GPUI / AccessKit 側の制約）

- **Web にはアクセシビリティツリーが無い。** gpui は wasm32 で AccessKit を組み込まない（`gpui/src/window.rs` の
  `#[cfg(not(target_family = "wasm"))]`）。Web ではスクリーンリーダーが部品を読めない。キーボード操作と
  フォーカスリングは動く。
- 無効状態はツリーに出している（AccessKit の `disabled`）が、Linux（AT-SPI）では AccessKit がボタン・タブ・リスト項目など
  一部の役割で無効を伝えない（`accesskit_atspi_common` が読み取り専用を持てる役割だけ `enabled` を外す）。
  Checkbox・RadioButton・Switch・Slider・TextField・Select では伝わる。
- Snackbar は live region（polite）として出すが、表示時に読み上げられるかはスクリーンリーダー次第。
- ツールチップはポインタを重ねたときだけ出る。`md3_tooltip` は同じ文を基準の要素の説明（description）にも入れる。
- TextField の入力エラーは invalid として出すが、gpui に「入力必須」の印はない。エラーの理由は補助テキストに書く。
- Dialog・モーダルな Sheet / Drawer には modal の印を付けるが、後ろの要素はツリーに残る（フォーカスとポインタは届かない）。
- gpui はフォーカスされた要素で Enter と Space の両方をクリックとして扱うため、Checkbox / Switch は Enter でも切り替わる。

### 表示・入力

- テキストフィールドの書記素の境界は簡易判定（結合文字・絵文字の ZWJ 列・国旗は扱う。依存を増やさないため
  Unicode の全規則ではない）。
- Web ではフォントをアプリが渡す必要がある（上記）。渡したフォントに無い字は描けない（豆腐になる）。
- M3 Expressive（バネのモーション、形のモーフ）、モバイル（iOS / Android）は対象外。
- フォントは同梱しない。既定はシステムフォント。

### 環境

- Linux の描画は Vulkan（wgpu）。GPU ドライバが無い環境では mesa の lavapipe（CPU 描画）で動く（遅い）。
- Web はブラウザの WebGPU / WebGL2 に依存する。ソフトウェア描画（SwiftShader）の WebGPU では
  描画デバイスが失われることがあり、その場合は `?renderer=webgl`（gallery）で WebGL2 を使う。
