# 基本ルール

- 常に日本語を使うこと
- 全角と半角の間には半角スペースを入れること

## コードコメント

- 新規コメントの言語は既存ファイルの慣習に合わせる。明確な慣習がなければ日本語を優先する
  - 同じ段落で文が続く場合は、文の区切りに句点 `。` を付ける。段落末尾とリスト項目の末尾には付けない
  - 全角括弧より半角括弧 `()` を優先して使用する
- コメントは必要なときだけ追加し、コードを読めば分かる説明で埋めない
- コメントは、今のコードに実在するもの (コードの行やシンボル、呼び出している外部 API、根拠となる仕様) を対象にする。採用しなかった案、消した値、変更予定、過去の状態のように、今のコードに無いものを対象にしない
- 変更履歴を書かない。「以前は」「従来は」「旧実装では」「もともと」「かつて」のように、今のコードを読んでも確認できない過去の状態を説明しない
- モジュールの責務を越えた説明を書かない。あるモジュールが知り得ない別モジュールの実装 (具体名・広告内容・分岐など) をそのモジュールのコメントに書かない
- repo 固有の制約、生成手順、非自明な境界条件を優先してコメントする
- 一時的な作業メモや agent 向け指示はコードコメントに残さず、必要なら `AGENTS.md`、`.agents/rules/`、`.agents/skills/` に昇格する
- ログやエラーメッセージは英語で記述する。テストで期待する結果（`assert!` や `expect` のメッセージなど）は日本語でよい
- 外部に公開されるコメントは、外部仕様を主として説明する。可視性にかかわらず、その項目を利用する側 (クレート内の呼び出し元も含む) から見た仕様を書く
  - 外部仕様とは、入力と前提、戻り値・エラー・None の条件、順序、有効期間、副作用などの契約を指す
  - 内部動作 (呼び出している外部 API、処理の順序、データ構造の選択) や他の項目との関係 (実装している型の列挙、引数が別の型のどの値に対応するか) は補足としてのみ使い、仕様の説明の主軸としない
  - 主軸にしない内部動作の理由は、doc ではなくコード行のコメントに書くこと

## コーディングスタイル

- Prioritise, in this order: stability, simplicity, performance.
- Keep things in one function unless composable or reusable
- Do not extract single-use helpers preemptively. Inline the logic at the call site unless the helper is reused, hides a genuinely complex boundary, or has a clear independent name that improves the caller.
- Before adding complexity for a speculative or vanishingly unlikely race or security edge case, explain the concrete failure mode, likelihood, and complexity cost to the user and get their buy-in. Do not silently expand scope for theoretical robustness.

## チェック

- 作業が完了したらコミット前に`nix fmt`, `nix flake check`を実行すること
