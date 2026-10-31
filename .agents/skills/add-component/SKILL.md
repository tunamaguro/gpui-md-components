---
name: add-component
description: md3-components に部品を 1 つ追加する（または既存の部品に状態・バリアントを足す）。仕様の要約、トークンの転記と出典、実装、gallery の見本、a11y テスト、画面確認、PR までを docs/agents/component-workflow.md の順で進める。
---

# 部品を追加する

手順は `docs/agents/component-workflow.md` の 1〜7 に従う。この skill には進め方だけを書く。

1. 読む: `component-workflow.md`、`headless.md`（責務の分け方）、`architecture.md`（共有部品の表）、`a11y.md`（役割とキー操作の表）、
   `gpui-notes.md`、`resources.md`（転記ルール）。
1. 一次資料を開く: `docs/references/androidx/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/` の
   `<Component>.kt` と `tokens/<Component>Tokens.kt`。値は推測しない。
1. 手順を実行する。実装を Codex に任せるなら skill `codex-delegate`、画面の確認は skill `gallery-check`。
1. skill `ship-pr` で出す。
