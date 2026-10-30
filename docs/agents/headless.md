# headless crate への分離

`md3-components` から、見た目を持たない `gpui-headless` を切り出す計画と規約。

## 目的

- MD3 以外の独自テーマを、a11y とキー操作を作り直さずに作れるようにする。
- a11y の責務を持つ場所を 1 つの crate に集める。

利用者は作者だけなので、公開 API の破壊的変更は構わない。互換用のラッパーや旧パスの再公開は作らず、最も適した構造を選ぶ。

## 依存の向き

```
gallery ──> md3-components ──> gpui-headless ──> gpui
                 └──> md3-theme / md3-icons
独自テーマ ──────────────────> gpui-headless
```

`gpui-headless` が依存してよいのは `gpui` だけ。`md3-*` には依存しない。

## 責務の分け方

| 持つもの                                                    | `gpui-headless` | `md3-components` |
| ----------------------------------------------------------- | --------------- | ---------------- |
| 役割・名前・状態（`A11yNode`）                              | ○               |                  |
| フォーカス、Tab の止まり場所、矢印キー、Esc、閉じ込めと復帰 | ○               |                  |
| 値の計算（Slider の刻み、Checkbox の次の状態など）          | ○               |                  |
| 押下・ドラッグ・開閉・テキスト編集の状態                    | ○               |                  |
| 色、寸法、角丸、影、フォント、アイコン                      |                 | ○                |
| トークン、`StateLayer`、`FocusRing`、モーションの値         |                 | ○                |

見た目に関わる値が振る舞いに必要なときは、`gpui-headless` に定数を置かず引数で受け取る
（例: `reveal_on_focus` のリング分の余白、スクリムの色、押下の最短時間、Snackbar の表示時間）。

## API の形

### 単体の部品

部品は `build` で、振る舞いを付けた要素と描画用の状態を返す。見た目は返された要素に足す。

```rust
let (base, state) = gpui_headless::Checkbox::new(id, checked, label)
    .disabled(disabled)
    .on_change(handler)
    .build(window, cx);

base.size(px(40.)).bg(my_color).when(state.focus_visible, |this| this.border_2())
```

`state` には `group`（hover 用のグループ名）、`focus_visible`、`disabled` を持たせる。
ホバー・押下のスタイルは土台の `.hover()` / `.active()`、子要素の
`.group_hover(state.group, ..)` / `.group_active(state.group, ..)` で付ける。
描画の計算に状態自体が必要な Switch などは、共通状態とともに部品固有の状態として返す。

### 項目を持つ部品

RadioGroup、Tabs などは項目ごとのクロージャを受け取る。Tab の止まり場所と矢印キーは crate 側が持つ。

```rust
gpui_headless::RadioGroup::new(id, label)
    .selected(selected)
    .option("A")
    .build(window, cx, |item, window, cx| {
        item.base.child(my_radio(item.selected, item.state.focus_visible)).into_any_element()
    })
```

## 進め方

1 段階 1 PR。各段階の完了条件は共通で、次の 3 つ。

1. テストを弱めない。a11y とキー操作のテストは、部品を移すときに `gpui-headless` へ移し、検証する内容は保つ。
1. `tools/a11y-tree/run.sh` の出力が基準（分離前の main）と一致する。
1. `gallery` の見た目と操作が変わらない。

| 段階 | 内容                                                                                                  |
| ---- | ----------------------------------------------------------------------------------------------------- |
| 1    | crate を作る。`A11yNode`、`control`、フォーカス、`keys`、`NestedScroll` / `ScrollScope` を移す        |
| 2    | 単体の操作部品: Button 系（Button、IconButton、FAB、ToggleButton）、Checkbox、Switch、Chip、Card      |
| 3    | 項目を持つ部品: RadioGroup、SegmentedButton、ButtonGroup、Tabs、NavigationBar / Rail / Drawer、Slider |
| 4    | オーバーレイ: レイヤー、Dialog、Sheet、Menu、Select、Tooltip、Snackbar                                |
| 5    | TextField、SearchBar、List / LazyList、Scrollbar                                                      |
| 6    | MD3 に依存しない独自テーマの見本を gallery に足して目的を検証する。利用者向けガイドを書く             |

## 規約

- `gpui-headless` の `Cargo.toml` の依存が `gpui` だけであることを、crate のテストで確かめる。
- 公開 API には `MD3` の語も `Material` の語も使わない。
- a11y のテストは、移した部品から順に `gpui-headless` 側にも足す。役割・名前・状態・キー操作を見た目なしで検証する。
