# gpui-headless の規約

`gpui-headless` は見た目を持たない操作基盤、`md3-components` はそれに MD3 の見た目を付ける crate。この 2 つの責務の分け方と規約をまとめる。使い方は [`docs/headless-guide.md`](../headless-guide.md)。

## 目的

- MD3 以外の独自テーマを、a11y とキー操作を作り直さずに作れるようにする。
- a11y の責務を持つ場所を 1 つの crate に集める。

利用者は作者だけなので、公開 API の破壊的変更は構わない。互換用のラッパーや旧パスの再公開は作らず、最も適した構造を選ぶ。

## 責務の分け方

| 持つもの                                                    | `gpui-headless` | `md3-components` |
| ----------------------------------------------------------- | --------------- | ---------------- |
| 役割・名前・状態・説明（`A11yNode`）                        | ○               |                  |
| フォーカス、Tab の止まり場所、矢印キー、Esc、閉じ込めと復帰 | ○               |                  |
| 値の計算（Slider の刻み、Checkbox の次の状態など）          | ○               |                  |
| 押下・ドラッグ・開閉・テキスト編集の状態                    | ○               |                  |
| 色、寸法、角丸、影、フォント、アイコン                      |                 | ○                |
| トークン、`StateLayer`、`FocusRing`、モーションの値         |                 | ○                |

見た目に関わる値が振る舞いに必要なときは、`gpui-headless` に定数を置かず引数で受け取る
（例: `reveal_on_focus` のリング分の余白、スクリムの色、押下の最短時間、Snackbar の表示時間）。

名前は API 上任意とし、表示文字を既定として `.aria_label()` で指定・上書きする。空文字は名前なしとして扱う。

名前を持つ headless 部品は `name: AccessibleName` を持ち、impl 内で `accessible_name!()` を使って setter と、名前をまとめて受け取る `name()` を定義する。表示文字が変わる setter は `set_visible` を呼び、ノードには `.with_name(&self.name)` で付ける。ラッパーは `delegate_accessible_name!(0)` のように委譲する。項目を MD3 から渡すときは `name()` で上書きの有無も渡す。

## API の形

- 単体の部品は `build(window, cx)` で、振る舞いを付けた要素と描画用の状態を返す。見た目は返された要素に足す。
- 状態には `group`（hover 用のグループ名）、`focus_visible`、`disabled` を持たせる。描画の計算に状態自体が必要な Switch などは、部品固有の状態も返す。
- 項目を持つ部品（RadioGroup、Tabs など）は項目ごとのクロージャを受け取る。Tab の止まり場所と矢印キーは crate 側が持つ。
- Menu は `build` で `MenuPopup` を返し、`place` に見た目を足す関数を渡す。Snackbar のキューは持ち主のビューが `SnackbarOwner` を実装して持つ。

## 規約

- `md3-components` の公開 API に `gpui-headless` の型を出さない。再公開せず、アプリが書く型は `md3-components` に定義する。列挙型は `From` で変換する。状態は `TextFieldState` のように包む。`crates/md3-components/tests/public_api.rs` で確かめる。
- `gpui-headless` の依存は `gpui` だけ。`md3-*` には依存しない。crate のテストで確かめる。
- 公開 API には `MD3` の語も `Material` の語も使わない。
- MD3 の部品は headless の部品を `inner` に持って設定を委譲し、同じ状態を自分のフィールドに持たない。種類から役割を導く SideSheet と、行ごとに見た目と振る舞いが混ざる Menu / List の項目は写す関数を残す。
- MD3 の部品は役割・名前・状態・説明を headless に渡し、描画とテストの両方で同じ `a11y_node()` を通す。任意の要素に付ける `md3_focus_tooltip` は例外。
- a11y のテストは、振る舞いを持つ `gpui-headless` 側に置く。役割・名前・状態・キー操作を見た目なしで検証する。テストを弱めない。
- 振る舞いを動かすだけの変更では、gallery と a11y ツリーを変更前と比較する。
