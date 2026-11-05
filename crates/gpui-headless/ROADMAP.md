# gpui-headless ロードマップ

コンポーネントの数ではなく、新しく必要になる操作の仕組みでマイルストーンを区切る。
後のマイルストーンは前のマイルストーンで作った仕組みを使う

1. 状態の持ち主 (呼び出し側が持つか、部品が内部で持つか)
2. グループ内の矢印キー移動 (roving tabindex)
3. オーバーレイ (Escape や外側クリックで閉じる、フォーカスの閉じ込めと戻し先)
4. 値の範囲 (min / max / step)
5. テキスト入力 (IME、選択)

各マイルストーンは 1 PR とし、コンポーネントは参照する APG に従って、キーボード操作と支援技術に公開する role・状態・アクションのテストを書く。

## マイルストーン

- [x] M0: Button
- [ ] M1: Toggle 系
- [ ] M2: md3-components で M0・M1 を使う
- [ ] M3: 調査 (関連付け・テスト・モーダル・フォーカス)
- [ ] M4: 矢印キー移動の基盤と RadioGroup / ToggleGroup
- [ ] M5: Slider
- [ ] M6: Tabs
- [ ] M7: 非モーダルのオーバーレイ基盤と Tooltip
- [ ] M8: Menu
- [ ] M9: Dialog
- [ ] M10: Select (Listbox)
- [ ] M11: TextField

## 共通の完了条件

すべてのマイルストーンで満たす

- キーボードフォーカスを装飾側が可視化できる (複合部品ではアクティブな項目にも付けられる)
- フォーカスの移動先がスクロール領域の外に隠れない
- 部品に名前を与える手段があり、テストで確かめる
- 状態を変える部品は、呼び出し側が状態を持つ API を基本とする
  - `on_change` は変更の要求であり、表示と公開する状態は渡された値に従う
  - 外部から値を変えたときはコールバックを呼ばない
  - 1 回の操作で要求を 1 回だけ通知する
- 「フォーカスできる」「Tab で到達できる」「操作できる」を別々に扱う

テストは次の 3 段階に分けて書く。M3 で 3 段目を検証する手段を決めるまでは、1・2 段目までとし、3 段目は実環境での再現手順を残す

1. 入力と状態遷移
2. 要素が書く a11y ノードの属性
3. 実際の a11y ツリー (親子関係・関連先) と支援技術からの操作の配送

## M0: Button

参照する APG:

- [Button](../../.agents/skills/a11y/references/aria-practices/content/patterns/button/button-pattern.html)

## M1: Toggle 系

対象: Checkbox (中間状態を含む)、Switch、押下状態を持つ Toggle Button

決めること:

- [ ] 部品が内部で状態を持つ API も用意するか。用意するなら初期値と、ID の寿命による初期化の条件
- [ ] Button の内部 (フォーカス、無効状態、クリックの扱い) をどう共有するか

参照する APG:

- [Checkbox](../../.agents/skills/a11y/references/aria-practices/content/patterns/checkbox/checkbox-pattern.html)
- [Switch](../../.agents/skills/a11y/references/aria-practices/content/patterns/switch/switch-pattern.html)
- [Button (Toggle Button)](../../.agents/skills/a11y/references/aria-practices/content/patterns/button/button-pattern.html)

## M2: md3-components で M0・M1 を使う

対象: MD3 の Button (各バリアント)、Icon button、Checkbox、Switch

headless 部品だけを先に増やすと、装飾側で使ったときに API の不足が見つかり作り直しになるため、早い段階で一度 MD3 まで作る。
以降のマイルストーンでも、対応する MD3 部品を作って装飾側の要求を確かめる

完了条件:

- [ ] state layer、ripple、focus ring を headless の `Styled` / `InteractiveElement` 経由で付けられる
- [ ] 装飾のために headless 側へ追加した API があれば、M1 の部品にも反映する
- [ ] gallery に表示する

## M3: 調査 (関連付け・テスト・モーダル・フォーカス)

後続のマイルストーンの前提を確かめる。gpui の公開 API が足りない場合は、回避策か gpui の rev 更新を決め、後続の完了条件を直す

- [ ] 描画される 2 つの要素を安定した `NodeId` で関連付け (`controls`、`labelled_by`、`described_by`)、実際のツリーで確かめられるか。gpui では要素から `NodeId` を得る `GlobalElementId::accesskit_node_id` が `pub(crate)`
- [ ] 実際の a11y ツリーと支援技術からの操作の配送をテストで確かめる手段 (テスト用 platform は a11y を初期化しない)
- [ ] モーダルの間、背景のサブツリー・Tab 移動・ショートカット・支援技術からの操作を止めるために、アプリ側とどう協力するか。gpui の `Window::focus_next` はウィンドウ全体の Tab stop を対象にし、モーダルの範囲に絞れない
- [ ] フォーカスと Tab stop を独立に制御する方法。M0 の Button はフォーカスを持つと Tab で到達できるため、矢印キー移動の項目に使えるか確かめる
- [ ] `aria_active_descendant` を、別のサブツリーに描画した一覧の項目に向けられるか

## M4: 矢印キー移動の基盤と RadioGroup / ToggleGroup

対象: RadioGroup、ToggleGroup (複数選択。MD3 の Segmented button の複数選択で使う)

決めること:

- [ ] 矢印キー移動の共通部品の形 (グループがフォーカスを持つ項目を管理する)
- [ ] 項目の追加・削除・無効化と、すべて無効なときのフォーカスの移動先
- [ ] 選択数の上限、選択の解除を許すか、フォーカスの移動で選択を変えるか (RadioGroup と ToggleGroup で別々に決める)

参照する APG:

- [Keyboard Interface (グループ内のフォーカス移動)](../../.agents/skills/a11y/references/aria-practices/content/practices/keyboard-interface/keyboard-interface-practice.html)
- [Radio Group](../../.agents/skills/a11y/references/aria-practices/content/patterns/radio/radio-group-pattern.html)
- [Button (ToggleGroup の各項目)](../../.agents/skills/a11y/references/aria-practices/content/patterns/button/button-pattern.html)

## M5: Slider

対象: Slider (単一の値)

M4 の矢印キーの扱いを使う。値の公開は gpui の `aria_numeric_value` など、支援技術からの操作は `on_a11y_action` で実現できる

参照する APG:

- [Slider](../../.agents/skills/a11y/references/aria-practices/content/patterns/slider/slider-pattern.html)

## M6: Tabs

対象: Tabs

決めること:

- [ ] フォーカスの移動と同時に選ぶか (自動) 、Enter / Space で選ぶか (手動)。自動にするのは、パネルの表示に目立つ遅延が無い場合に限る

参照する APG:

- [Tabs](../../.agents/skills/a11y/references/aria-practices/content/patterns/tabs/tabs-pattern.html)

## M7: 非モーダルのオーバーレイ基盤と Tooltip

対象: アンカーに合わせて配置し開閉する基盤 (`anchored` / `deferred`)、Tooltip

Menu と Select がこの基盤を使う。Dialog の完成を待たない

参照する APG:

- [Tooltip](../../.agents/skills/a11y/references/aria-practices/content/patterns/tooltip/tooltip-pattern.html)

## M8: Menu

対象: Menu Button + Menu

M4 の矢印キー移動と M7 のオーバーレイ基盤を組み合わせる

参照する APG:

- [Menu Button](../../.agents/skills/a11y/references/aria-practices/content/patterns/menu-button/menu-button-pattern.html)
- [Menu and Menubar](../../.agents/skills/a11y/references/aria-practices/content/patterns/menubar/menu-and-menubar-pattern.html)

## M9: Dialog

対象: Dialog (モーダル)

M7 の基盤に、M3 で決めた背景の操作を止める仕組みを組み合わせる

参照する APG:

- [Dialog (Modal)](../../.agents/skills/a11y/references/aria-practices/content/patterns/dialog-modal/dialog-modal-pattern.html)

## M10: Select (Listbox)

対象: Select (APG の Select-Only Combobox)

参照する APG:

- [Combobox](../../.agents/skills/a11y/references/aria-practices/content/patterns/combobox/combobox-pattern.html)
- [Select-Only Combobox の例](../../.agents/skills/a11y/references/aria-practices/content/patterns/combobox/examples/combobox-select-only.html)

## M11: TextField

gpui の `EntityInputHandler` を前提にする。最初は単一行に限る

決めること (M4 までに決める):

- [ ] 文字列・選択・IME・クリップボードを扱う編集モデルと、位置・範囲・描画の対応を渡す境界
- [ ] 支援技術にテキストと選択を公開する方法 (gpui の `TextRun` / `TextSelection`)

参照する APG:

- 対応する APG のパターンは無い

## 随時

操作がほとんど無く、他のマイルストーンに依存しないもの。手が空いたときに入れる

- [ ] Progress indicator (対応する APG のパターンは無い)
- [ ] Link ([APG](../../.agents/skills/a11y/references/aria-practices/content/patterns/link/link-pattern.html))
- [ ] Disclosure ([APG](../../.agents/skills/a11y/references/aria-practices/content/patterns/disclosure/disclosure-pattern.html))

## 後回し

必要になった時点でマイルストーンにする

- [ ] RTL: 方向の指定方法、項目の順序と左右キーの対応、popup の start / end 配置
- [ ] 独立した Listbox (複数選択を含む)
- [ ] Toolbar (中の radio は矢印キーの移動で選択を変えない)
- [ ] Accordion
- [ ] Field: ラベル・説明・エラーと required / invalid の公開
- [ ] Status / Alert: フォーカスを奪わない通知 (Snackbar で使う)
