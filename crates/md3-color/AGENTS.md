# md3-color

## 役割

HCT / CAM16、パレット、動的配色、コントラストなどの色計算を提供する。
UI や描画フレームワークに依存しない配色エンジン

## crate 固有の規約

- 外部 crate やワークスペース内の他の crate に依存しない
- 配色は material-color-utilities の TypeScript 版の 2021 仕様を対象とする。別の仕様世代を混在させない
- 内部演算は `f64` の精度を維持し、移植元の丸め・正規化・境界条件を保つ
- 通常の実装ではパニックを起こさず、`unwrap`・`expect`・`panic!` を使わない。入力の扱いや失敗条件は公開 API の契約として明確にする
- GPUI の色型への変換、テーマ設定、部品ごとの色の選択は持ち込まない
- 移植元と出典の扱いは [MD3 移植スキル](../../.agents/skills/material-design-3-port/SKILL.md) に従う
