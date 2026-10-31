# コンポーネント実装の手順

コンポーネント 1 つ（例: Checkbox）を実装するときは、毎回この順で進める。1 コンポーネント = 1 PR を基本とする。

## 1. 仕様の要約を作る

- m3.material.io の該当コンポーネント（Overview / Specs / Guidelines / Accessibility）を読み、`docs/specs/<component>.md` に `docs/specs/_template.md` の形式で要約する。
- 本文を読めないときは Compose の実装で代え、読めなかったと spec と PR に書く。
- 長文の引用はしない。寸法・状態・振る舞いを箇条書きにまとめる。

## 2. トークンを転記する

- Compose の `tokens/<Component>Tokens.kt` から値を `md3-tokens` に `const` として転記する。
- 色はトークン名（`ColorSchemeKeyTokens.Primary` 等）を `ColorRole` に対応させる。値を直書きしない。
- 出典を `docs/TOKENS_SOURCE.md` に追記する（resources.md の転記ルール）。

## 3. 実装する

- `md3-components/src/<category>/<component>.rs` に実装し、対応する feature で囲む。
- architecture.md の設計規約に従う（`ElementId` 必須、enum バリアント、ビルダー、`RenderOnce`）。
- 状態色は `StateLayer` で合成する。直接 hover 色を計算しない。角が不揃いなら `.corners()` を渡す。フォーカスは `FocusRing`。
- 名前は表示文字を既定とし、任意の `.aria_label()` で指定・上書きできるようにする。
- 名前 setter は headless の `accessible_name!()`、MD3 の委譲には `delegate_accessible_name!(inner)`、MD3 の項目自身が名前を持つ場合は `accessible_name!()` を使う。追加の説明は `#[doc = "..."]` をマクロに渡す。詳細は `headless.md`。
- a11y とキー操作は a11y.md の表に従い `gpui-headless` に実装し、`A11yExt` 経由で付与する。無効なら Tab で止まらない。MD3 側は headless の部品を `inner` に持って設定を委譲し、返された要素に見た目を付ける（`headless.md`）。
- GPUI の API はその都度、固定 rev のソースで確認する。既知の落とし穴は `gpui-notes.md`。
- スクロール領域を持つなら `ScrollArea` を使う（`architecture.md` の「共有部品」）。
- 状態の変化に Compose が既定でアニメーションを付けているなら、`Theme::spring` のバネで付ける。`reduce_motion` で止まることを確かめる。
- 利用者のコールバックは Entity の update の中から直接呼ばない（`cx.defer`）。押下状態はフォーカスを失ったときにも解除する。
- 文字は `Theme::text_style` を通し、文字を含む容器は `.min_h()` にして、省略せず折り返す（`a11y.md` の「文字の拡大」）。
- 描画経路に thread / Mutex / ブロッキングを入れない（wasm）。

## 4. gallery に見本を追加する

- `examples/gallery/src/stories/<component>.rs` を追加し、全バリアントの操作できる見本と disabled、該当すれば selected / error の見本を並べる。
- 状態を固定する見本や、そのための指定は作らない。

## 5. テストを書く

- `crates/gpui-headless/tests/`: 役割・名前・状態・キー操作（a11y.md の手順 1）。
- 既存の部品のアクセシビリティの問題を直すときは、先に問題を再現するテストを足し、失敗を確かめてから修正する。
- 「出ない」を assert するテストは、その前に「出る」状態を assert する。前提が崩れていても通ってしまう。
- 寸法を変える修正は、定数ではなく描いた結果を `debug_bounds` で確かめる（100% の寸法を含む）。
- 見た目のロジック（色の選択、寸法計算）は純関数に切り出して単体テストする。

## 6. 画面で確かめる

- gallery を起動して、追加した見本を操作し、hover / focused / pressed を撮って確かめる（skill `gallery-check`）。
- Theme の Text scale を 200% にして、文字が切れない・重ならないことを確かめる。

## 7. 完了チェック

- [ ] AGENTS.md の「完了の定義」を全て満たす
- [ ] README の部品表、`docs/guide.md`（使い方が増えたら）、`docs/agents/a11y.md` の表を更新した
- [ ] `docs/specs/<component>.md` がある
- [ ] `docs/TOKENS_SOURCE.md` に転記した全値がある
- [ ] a11y テストがある
- [ ] PR 説明に、何をなぜ変えたかと、仕様から外れた点・未対応点を書く

## 困ったとき

- 仕様が曖昧: Compose の実装（`androidx/compose/material3/<Component>.kt`）の振る舞いに合わせ、その旨を spec ファイルに書く。
- GPUI で実現できない: 実装を止めず、制限を spec ファイルの「既知の制限」に書いて最小限の代替を実装する。代替が無理なら人間に相談する。
- 依存を追加したくなった: 追加せず人間に相談する。
