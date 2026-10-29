# コンポーネント実装の手順

コンポーネント 1 つ（例: Checkbox）を実装するときは、毎回この順で進める。1 コンポーネント = 1 PR を基本とする。

## 1. 仕様の要約を作る

- m3.material.io の該当コンポーネント（Overview / Specs / Guidelines / Accessibility）を読み、`docs/specs/<component>.md` に `docs/specs/_template.md` の形式で要約する。
- 長文の引用はしない。寸法・状態・振る舞いを箇条書きにまとめる。

## 2. トークンを転記する

- Compose の `tokens/<Component>Tokens.kt` から値を `md3-tokens` に `const` として転記する。
- 色はトークン名（`ColorSchemeKeyTokens.Primary` 等）を `ColorRole` に対応させる。値を直書きしない。
- 出典を `docs/TOKENS_SOURCE.md` に追記する（resources.md の転記ルール）。

## 3. 実装する

- `md3-components/src/<category>/<component>.rs` に実装し、対応する feature で囲む。
- architecture.md の設計規約に従う（`ElementId` 必須、enum バリアント、ビルダー、`RenderOnce`）。
- 状態色は `StateLayer` で合成する。直接 hover 色を計算しない。
- a11y は a11y.md の表に従い、`A11yExt` 経由で付与する。
- GPUI の API はその都度、固定 rev のソースで確認する。既知の落とし穴は `gpui-notes.md`。
- スクロール領域を持つなら `NestedScroll` と `Scrollbar` を付ける（`architecture.md` の「共有部品」）。
- 状態の変化に Compose が既定でアニメーションを付けているなら、`Theme::spring` のバネで付ける。`reduce_motion` で止まることを確かめる。
- 利用者のコールバックは Entity の update の中から直接呼ばない（`cx.defer`）。押下状態はフォーカスを失ったときにも解除する。

## 4. gallery に見本を追加する

- `gallery/src/stories/<component>.rs` を追加し、全バリアント × 全状態（enabled / hover / focused / pressed / disabled、該当すれば selected / error）を並べる。
- hover / pressed は静的に再現できるよう、内部向けの「強制状態」指定を用意してよい（公開 API にはしない、`#[doc(hidden)]`）。

## 5. テストを書く

- `tests/a11y_<component>.rs`: 役割・名前・状態・キー操作（a11y.md の手順 1）。
- 見た目のロジック（色の選択、寸法計算）は純関数に切り出して単体テストする。

## 6. 画面で確かめる

- gallery を起動して、追加した見本を撮って確かめる（skill `gallery-check`）。

## 7. 完了チェック

- [ ] AGENTS.md の「完了の定義」を全て満たす
- [ ] `docs/specs/<component>.md` がある
- [ ] `docs/TOKENS_SOURCE.md` に転記した全値がある
- [ ] gallery に全状態の見本がある
- [ ] a11y テストがある
- [ ] PR 説明に、何をなぜ変えたかと、仕様から外れた点・未対応点を書く

## 困ったとき

- 仕様が曖昧: Compose の実装（`androidx/compose/material3/<Component>.kt`）の振る舞いに合わせ、その旨を spec ファイルに書く。
- GPUI で実現できない: 実装を止めず、制限を spec ファイルの「既知の制限」に書いて最小限の代替を実装する。代替が無理なら人間に相談する。
- 依存を追加したくなった: 追加せず人間に相談する。
