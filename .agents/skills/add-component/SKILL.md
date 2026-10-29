---
name: add-component
description: md3-components に部品を 1 つ追加する（または既存の部品に状態・バリアントを足す）。仕様の要約、トークンの転記と出典、実装、gallery の見本、a11y テスト、画面確認、PR までを docs/agents/component-workflow.md の順で進める。
---

# 部品を追加する

正の手順は `docs/agents/component-workflow.md`。この skill はそれを実行するときのチェックリスト。

1. 読む: `docs/agents/component-workflow.md`、`architecture.md`（共有部品の表）、`a11y.md`（役割とキー操作の表）、
   `gpui-notes.md`、`resources.md`（転記ルール）。
2. 一次資料を開く: Compose の `<Component>.kt` と `tokens/<Component>Tokens.kt`
   （androidx の `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/`）。
   手元に無ければ scratchpad に sparse checkout する。値は推測しない。
3. `docs/specs/<component>.md` を `_template.md` の形式で書く。
4. トークンを `md3-tokens` に転記し、`docs/TOKENS_SOURCE.md` に出典（ファイル・行・androidx のコミット）を足す。
   移植したコードにはファイル先頭に Apache-2.0 の出典コメント。
5. 実装（Codex に任せるなら skill `codex-delegate`）。確認すること:
   - `ElementId` 必須、a11y の役割・名前・状態、APG のキー操作、無効なら Tab で止まらない
   - 状態色は `StateLayer`、角が不揃いなら `.corners()`、フォーカスは `FocusRing`
   - スクロール領域があれば `NestedScroll` と `Scrollbar`
   - Compose が既定で付けているモーションは `Theme::spring` で付け、`reduce_motion` で止まる
   - コールバックを Entity の update の中から呼ばない、押下はフォーカス喪失でも解除する
   - 描画経路に thread / Mutex / ブロッキングがない（wasm）
6. gallery の見本（全バリアント × enabled / hover / focused / pressed / disabled、該当すれば selected / error）。
7. テスト: `crates/md3-components/tests/a11y_<component>.rs`（役割・名前・状態・キー操作）と純関数の単体テスト。
8. 画面で確かめる（skill `gallery-check`）。
9. README の部品表、`docs/guide.md`（使い方が増えたら）、`docs/agents/a11y.md` の表を更新する。
10. skill `ship-pr` で出す。
