# gpui-headless ロードマップ

コンポーネントの数ではなく、新しく必要になる操作の仕組みでマイルストーンを区切る。
後のマイルストーンは前のマイルストーンで作った仕組みを使う

1. 状態の持ち主 (呼び出し側が持つか、部品が内部で持つか)
2. グループ内の矢印キー移動 (roving tabindex)
3. オーバーレイ (Escape や外側クリックで閉じる、フォーカスの閉じ込めと戻し先)
4. 値の範囲 (min / max / step)
5. テキスト入力 (IME、選択)

各マイルストーンは 1 PR とし、完了条件には APG のキーボード操作と、支援技術に公開する role・状態・アクションのテストを含める。

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

- [x] role `button`、Enter / Space / クリック / 支援技術のクリック操作で `on_click` を呼ぶ
- [x] 無効状態 (`Disabled` / `DisabledFocusable`)

## M1: Toggle 系

対象: Checkbox (中間状態を含む)、Switch、押下状態を持つ Toggle Button

決めること:

- [ ] 部品が内部で状態を持つ API も用意するか。用意するなら初期値と、ID の寿命による初期化の条件
- [ ] Button の内部 (フォーカス、無効状態、クリックの扱い) をどう共有するか

完了条件:

- [ ] Checkbox: role `checkbox`、Space で切り替え、`aria_toggled` で true / false / mixed を公開する
- [ ] Switch: role `switch`、Space で切り替え (Enter は任意)、on / off を公開する。mixed は公開しない
- [ ] Toggle Button: role `button` に押下状態を公開し、Enter / Space で切り替える
- [ ] Switch と Toggle Button は、状態が変わっても名前を変えない
- [ ] 中間状態の Checkbox を操作したときの次の状態を呼び出し側が決められる
- [ ] 無効状態は Button と同じ契約に従う

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

完了条件:

- [ ] グループ全体が Tab 1 回で到達でき、到達先は選択中の項目 (無ければ先頭)
- [ ] 矢印キーで移動し、端で折り返す
- [ ] 無効な項目の扱い (飛ばすか、フォーカスはできるか) が APG に従う
- [ ] RadioGroup: role `radiogroup` / `radio`、各 radio の checked 状態、グループと各項目の名前。Space で選び、矢印キーの移動で選択も移る
- [ ] ToggleGroup: 各項目の押下状態を公開し、矢印キーの移動では選択を変えない

## M5: Slider

対象: Slider (単一の値)

M4 の矢印キーの扱いを使う。値の公開は gpui の `aria_numeric_value` など、支援技術からの操作は `on_a11y_action` で実現できる

完了条件:

- [ ] role `slider`、名前・現在値・最小値・最大値・向きを公開する。読み上げ用の値の表現を指定できる
- [ ] 矢印キーで step ずつ、Home / End で最小・最大に変える。PageUp / PageDown で大きく変える (任意)
- [ ] ドラッグとトラックのクリックで値を変える
- [ ] 支援技術の Increment / Decrement / SetValue に応える
- [ ] キーボード・ドラッグ・支援技術の操作が同じ値の更新規則 (範囲への丸め、step) に従う

## M6: Tabs

対象: Tabs

決めること:

- [ ] フォーカスの移動と同時に選ぶか (自動) 、Enter / Space で選ぶか (手動)。自動にするのは、パネルの表示に目立つ遅延が無い場合に限る

完了条件:

- [ ] role `tablist` / `tab` / `tabpanel`、tablist の名前、各 tab の selected 状態
- [ ] tab から panel への `controls`、panel から tab への `labelled_by` を公開する (M3 の結果に従う)
- [ ] 横向き・縦向きで矢印キーが変わる。Home / End で先頭・末尾に移動する (任意)
- [ ] 選ばれていない panel は表示せず、操作の対象にしない
- [ ] panel の先頭にフォーカスできる要素が無いとき、panel 自体に Tab で到達できる

## M7: 非モーダルのオーバーレイ基盤と Tooltip

対象: アンカーに合わせて配置し開閉する基盤 (`anchored` / `deferred`)、Tooltip

Menu と Select がこの基盤を使う。Dialog の完成を待たない

完了条件:

- [ ] Escape と外側クリックで閉じ、閉じた理由を呼び出し側に伝える
- [ ] Tooltip: role `tooltip`、起動元の説明として関連付ける (M3 の結果に従う)。Tooltip にフォーカスを移さない
- [ ] Tooltip: フォーカスとホバーで表示し、Escape で閉じる。ホバーしたまま Tooltip 上にポインタを移しても消えない
- [ ] Tooltip: Escape で閉じた後、同じホバー・フォーカスのままでは再表示しない

## M8: Menu

対象: Menu Button + Menu

M4 の矢印キー移動と M7 のオーバーレイ基盤を組み合わせる

完了条件:

- [ ] Menu Button: Enter / Space / 下矢印で開いて先頭へ、上矢印で開いて末尾へフォーカス。開閉状態と popup の種類を公開する
- [ ] Menu: 上下矢印で移動、Home / End、文字入力で先頭文字が一致する項目へ移動 (typeahead)
- [ ] 無効な項目にも矢印キーでフォーカスできる
- [ ] 閉じたときのフォーカス先を理由ごとに分ける: Escape と項目の実行は Menu Button に戻す、Tab / Shift+Tab は外へ移す、外側の操作ではその操作先を尊重する
- [ ] 実行したコマンドが別の場所 (Dialog など) にフォーカスを移した場合は、それを上書きしない
- [ ] role `menu` / `menuitem` (checkbox / radio の項目を含む)

## M9: Dialog

対象: Dialog (モーダル)

M7 の基盤に、M3 で決めた背景の操作を止める仕組みを組み合わせる

完了条件:

- [ ] 開くとダイアログ内の最初の操作可能な要素か、指定した要素 (静的な見出しも可) にフォーカスする
- [ ] Tab / Shift+Tab がダイアログ内で循環する
- [ ] Escape で閉じ、開く前にフォーカスしていた要素か指定した要素にフォーカスを戻す。その要素が無くなっていた場合の戻し先を決める
- [ ] 背景はマウス・キーボード・支援技術のいずれからも操作できない。背景を視覚的に覆う処理は MD3 側の完了条件にする
- [ ] role `dialog`、modal であること (`set_modal`) と名前を公開する

## M10: Select (Listbox)

対象: Select (APG の Select-Only Combobox)

完了条件:

- [ ] 実際のフォーカスは combobox に残し、候補は `aria_active_descendant` で示す
- [ ] 開いたときは選択中の候補をアクティブにし、見える位置までスクロールする
- [ ] 矢印キーで候補を動かしても値は確定しない。Enter / Space / Tab で確定し、Escape では元の値を保って閉じる
- [ ] role `combobox` / `listbox` / `option`、選択状態と展開状態を公開する

## M11: TextField

gpui の `EntityInputHandler` を前提にする。最初は単一行に限る

決めること (M4 までに決める):

- [ ] 文字列・選択・IME・クリップボードを扱う編集モデルと、位置・範囲・描画の対応を渡す境界
- [ ] 支援技術にテキストと選択を公開する方法 (gpui の `TextRun` / `TextSelection`)

完了条件:

- [ ] read-only / disabled
- [ ] 名前・値・選択状態を公開し、対応する支援技術の操作に応える

## 随時

操作がほとんど無く、他のマイルストーンに依存しないもの。手が空いたときに入れる

- [ ] Progress indicator: role `progressbar`。値が確定していれば範囲と現在値を公開し、不定なら現在値を省く
- [ ] Link: Enter で実行する (Space では実行しない)
- [ ] Disclosure: 開閉ボタンと領域の関連 (M3 の結果に従う)

## 後回し

必要になった時点でマイルストーンにする

- [ ] RTL: 方向の指定方法、項目の順序と左右キーの対応、popup の start / end 配置
- [ ] 独立した Listbox (複数選択を含む)
- [ ] Toolbar (中の radio は矢印キーの移動で選択を変えない)
- [ ] Accordion
- [ ] Field: ラベル・説明・エラーと required / invalid の公開
- [ ] Status / Alert: フォーカスを奪わない通知 (Snackbar で使う)
