# RadioButton / RadioGroup 仕様要約

- 出典: Jetpack Compose Material 3 の実装（`RadioButton.kt`）とトークン。m3.material.io の本文はこの PR では参照していない（Compose の実装に合わせた）。
- トークン出典: `compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/RadioButtonTokens.kt` @ b8256a52
- マイルストーン: M4

## バリアント

- 縦並び（既定）/ 横並び

## 構成要素（アナトミー）

1. 状態レイヤー（40dp の円）
2. 20dp の円（線 2dp）
3. 選択時の中の点（12dp）
4. ラベル（Body Large）

## 寸法

| 項目 | 値 | トークン |
| --- | --- | --- |
| 状態レイヤー | 40dp | `radio_button::STATE_LAYER_SIZE` |
| 円 | 20dp | `ICON_SIZE` |
| 線 / 点 | 2dp / 12dp | `layout::RADIO_BUTTON_*` |

## 色（状態別）

| 要素 | enabled | hover | focused | pressed | disabled |
| --- | --- | --- | --- | --- | --- |
| 円（選択） | Primary | 同 | 同 | 同 | OnSurface 38% |
| 円（非選択） | OnSurfaceVariant | OnSurface | OnSurface | OnSurface | OnSurface 38% |

## 振る舞い

- 矢印キーで選択が移る（無効な項目は飛ばし、端で反対側へ回る）。
- Tab で止まるのは選択中の項目（無ければ最初の有効な項目）だけ。

## アクセシビリティ

- 役割: RadioGroup + RadioButton
- 名前: グループのラベル、各項目のラベル
- 状態: selected、位置（n / 全体）
- キー操作: 矢印で移動と選択、Tab でグループ外へ

## デスクトップ向け調整（密度など）

- 高さのある操作要素はテーマの密度（`Theme::dense`）で 1 段 4dp ずつ小さくなる。

## 既知の制限・仕様からの逸脱

- 単体の RadioButton は公開せず、RadioGroup だけを提供する（APG のキー操作をグループで保証するため）。
- 無効は AccessKit の disabled としてツリーに出し、Tab でも止まらない。ただし Linux（AT-SPI）では役割によって無効が伝わらない（[`docs/a11y-audit.md`](../a11y-audit.md)）。
