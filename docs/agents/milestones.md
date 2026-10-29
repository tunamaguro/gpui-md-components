# マイルストーン

v1.0 は M0〜M7。M8 は任意。日付は未設定。
各マイルストーンは「完了条件」を満たす PR のマージで完了とする。

## 状態（2026-10-29）

| | PR | 状態 |
| --- | --- | --- |
| M0 基盤 | #1 | 完了 |
| M1 配色エンジン | #2 | 完了 |
| M2 トークン | #3 | 完了 |
| M3 テーマ・基礎部品 | #4 | 完了 |
| M4 操作・選択 | #5 | 完了（#15 で Switch・RadioButton のモーションを追加） |
| M5 入力・オーバーレイ | #6 | 完了（#15 で Snackbar の入れ替え） |
| M6 ナビ・コンテナ | #7 | 完了（#13 で入れ子スクロール、#14 で Scrollbar） |
| M7 a11y 監査・Web・v1.0 | #8 | 実装は完了。**読み上げ確認・Web 版の公開・v1.0 タグが人間待ち** |
| M8 Expressive | #9 | 完了（バネは自作せず gpui の `SpringAnimation` を使った。形状モーフは未実装） |

残りの作業と未対応の候補は [`handoff.md`](handoff.md)。以下は計画時の定義（履歴として残す）。

依存順:

```
M0 ─┬─> M1 配色 ──┐
    └─> M2 トークン ┴─> M3 テーマ・基礎部品 ─┬─> M4 操作・選択      ─┐
                                              ├─> M5 入力・オーバーレイ ┼─> M7 a11y 監査・Web・v1.0 ─> (M8 Expressive)
                                              └─> M6 ナビ・コンテナ    ─┘
```

M1 と M2、M4 / M5 / M6 は独立しており並行で進められる。

---

## M0 基盤

- 成果物
  - Cargo workspace（`crates/` の 5 crate + `gallery` + `xtask`）。中身は空でよい
  - `[workspace.dependencies]` で gpui / gpui_platform の rev 固定
  - `LICENSE`（Apache-2.0）、`docs/TOKENS_SOURCE.md`（空テンプレート）
  - CI（GitHub Actions 想定）: Windows / macOS / Linux で fmt・clippy・test、Linux で `wasm32-unknown-unknown` ビルド
  - `xtask` の骨組み（サブコマンド `web`, `check-tokens` は空実装でよい）
- 完了条件
  - `cargo run -p gallery` でデスクトップ 3 OS に空ウィンドウが出る
  - CI が緑

## M1 配色エンジン（md3-color）

- 成果物
  - `Argb`、`Cam16`、`ViewingConditions`、`Hct`（HCT ソルバ含む）
  - `TonalPalette`、`CorePalette`
  - `DynamicScheme` と各バリアント: TonalSpot（既定）、Neutral、Vibrant、Expressive、Fidelity、Content、Monochrome
  - `MaterialDynamicColors` 相当（全カラーロールの算出）、コントラスト 標準 / 中 / 高
  - `contrast` モジュール（WCAG コントラスト比）
- 完了条件
  - material-color-utilities の公式テストを Rust に移植し全件一致（ARGB 完全一致）
  - `unwrap` / `expect` / `panic!` / `unsafe` なし
  - 移植元のファイルとバージョン（コミット）を各ファイル先頭に記載

## M2 トークン（md3-tokens）

- 成果物
  - タイプスケール 15 種（Display / Headline / Title / Body / Label × Large / Medium / Small）
  - 角丸スケール（none, extra-small, small, medium, large, extra-large, full）
  - 高さ（elevation）レベル 0〜5
  - モーション: イージング（emphasized, standard 等）と時間（short1〜extra-long4）
  - 状態レイヤー不透明度（hover / focus / pressed / dragged）、disabled の不透明度
  - 密度スケール（0, -1, -2）
  - `xtask check-tokens` の実装
- 完了条件
  - 全定数に出典（ファイル名と行）が `docs/TOKENS_SOURCE.md` にある
  - gpui に依存しない

## M3 テーマと基礎部品（md3-theme、md3-components の基礎）

- 成果物
  - `Theme`（gpui `Global`）と `Md3ThemeExt`（`cx.md3()`）
  - ライト / ダークの OS 外観追従
  - 利用者による上書き: シード色、個別ロール色、フォントファミリ、タイプスケール、密度、角丸（architecture.md 参照）
  - `Theme::check_contrast()`
  - 基礎部品: `Surface`、`StateLayer`、`FocusRing`、`Icon`（md3-icons）、`Text`（タイプスケール適用）
  - キーボードフォーカス順序の方針を `docs/agents/a11y.md` に追記
  - gallery: シード色・コントラスト・外観・密度の切り替え UI、全カラーロールの見本
- 完了条件
  - gallery で上記切り替えが動く
  - Tab で移動するとフォーカスリングが見える

## M4 操作・選択（feature: buttons, selection）

- 対象: Button（5 種）、IconButton（4 種）、FAB（各サイズ）、SegmentedButton、Checkbox、RadioButton、Switch、Chip（Assist / Filter / Input / Suggestion）、Slider、Badge、Progress（線 / 円）
- 完了条件
  - 各部品で enabled / hover / focused / pressed / disabled（該当すれば selected / error）が仕様どおり
  - 役割・名前・状態が a11y ツリーに出て、APG どおりのキー操作ができることを UI テストで検証

## M5 入力・オーバーレイ（feature: inputs, overlays）

- 対象: TextField（Filled / Outlined、IME、エラー、補助テキスト、文字数カウンタ）、Menu、Select 相当、Dialog、Tooltip（Plain / Rich）、Snackbar、BottomSheet / SideSheet
- 完了条件
  - Dialog: フォーカストラップ、Esc で閉じる、閉じた後に呼び出し元へフォーカス復帰
  - Menu: 矢印キー移動、Esc で閉じる、項目の役割が menuitem
  - TextField: 日本語 IME で変換中文字が正しく表示・確定される
  - Snackbar: 表示時にスクリーンリーダーへ通知される（GPUI の a11y API で可能な範囲。不可なら制限として記録）

## M6 ナビゲーション・コンテナ（feature: navigation, containment）

- 対象: TopAppBar（各種）、NavigationRail、NavigationDrawer、NavigationBar、Tabs（Primary / Secondary）、List（仮想化対応）、Card（Elevated / Filled / Outlined）、Divider、Search
- 完了条件
  - ウィンドウ幅クラス（compact / medium / expanded 以上）でナビが切り替わる
  - 矢印キーでタブ・リストを移動できる

## M7 a11y 監査・Web・v1.0

- 作業
  - `docs/agents/a11y.md` の手順 3（読み上げ確認）を全部品で実施
  - 全バリアント × 全コントラストで WCAG 2.2 のコントラスト比を自動チェックするテスト
  - `cargo xtask web` で wasm 版 gallery をビルドし公開
  - README、利用ガイド、既知の制限（Web の未対応機能など）を整備
- 完了条件
  - `docs/a11y-audit.md` に結果を記録し、重大指摘ゼロ
  - v1.0 タグ

## M8 Expressive（任意）

- 対象: バネ物理モーション（自作スプリング積分）、形状モーフ、Expressive のボタングループ等。参照は Compose 実装
- 完了条件: OS の「視差効果を減らす」相当の設定でモーションを無効化できる
