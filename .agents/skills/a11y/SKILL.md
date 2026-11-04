---
name: a11y
description: このリポジトリの UI のアクセシビリティ要件や入力イベントを調査するときに使う。同梱の WAI-ARIA、APG、WCAG、UI Events、WCAG2ICT の一次資料から、対象の部品・達成基準・イベント・非 Web ソフトウェアへの適用に関する根拠を探す。
---

# アクセシビリティの参照資料

パスはこの `SKILL.md` のあるディレクトリを基準に解決する。調査対象に対応する資料から読み、各リポジトリの README で文書の構成を確認する。

| 資料                                        | 場所                                                   | 調べるもの                                                                     |
| ------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------ |
| WAI-ARIA                                    | [references/aria](references/aria)                     | ロール・状態・プロパティ、アクセシブルな名前などの仕様                         |
| WAI-ARIA Authoring Practices Guide (APG)    | [references/aria-practices](references/aria-practices) | 部品の役割・状態・キーボード操作、アクセシブルな名前やフォーカス管理のパターン |
| Web Content Accessibility Guidelines (WCAG) | [references/wcag](references/wcag)                     | 達成基準、その解説、関連する達成方法                                           |
| UI Events                                   | [references/uievents](references/uievents)             | キーボード・フォーカス・composition などの入力イベント                         |
| WCAG2ICT                                    | [references/wcag2ict](references/wcag2ict)             | WCAG を非 Web 文書・ソフトウェアへ適用するときのガイダンス                     |
| Material Design 3                           | https://m3.material.io/foundations/overview/principles | Foundations > Accessibility と各コンポーネントの Accessibility タブ            |

## 参照時の確認

- 使用する submodule のコミットと作業ツリーの変更を確認し、根拠となるファイル・項目・コミットを記録する。リビジョン更新の依頼がない限り、固定された参照元を使う。
- 対象の文書の版、公開状態、適用範囲を確認する。WAI-ARIA の仕様本文、APG のパターン、WCAG の達成基準本文と解説・達成方法、WCAG2ICT の適用ガイダンスを区別する。
- Web の DOM / ARIA / イベントに関する記述を GPUI に適用するときは、利用する概念と GPUI での実現方法を区別して説明する。

## 優先順位

各資料で食い違いがある場合、WCAG 2.2(満たすべき基準)を最優先し、振る舞いは MD3、役割・状態・キー操作は APG に合わせる
