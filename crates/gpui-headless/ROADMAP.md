# gpui-headless ロードマップ

コンポーネントの数ではなく、新しく必要になる操作の仕組みでマイルストーンを区切る。
後のマイルストーンは前のマイルストーンで作った仕組みを使う

1. 状態の持ち主 (呼び出し側が持つか、部品が内部で持つか)
1. グループ内の矢印キー移動 (roving tabindex)
1. オーバーレイ (Escape や外側クリックで閉じる、フォーカスの閉じ込めと戻し先)
1. 値の範囲 (min / max / step)
1. テキスト入力 (IME、選択)

各マイルストーンは 1 PR とし、コンポーネントは参照する APG に従って、キーボード操作と支援技術に公開する role・状態・アクションのテストを書く。

## マイルストーン

- [x] M0: Button
- [ ] M1: Toggle 系
- [ ] M2: md3-components で M0・M1 を使う
- [x] M3: 調査 (関連付け・テスト・モーダル・フォーカス)
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

テストは次の 3 段階に分けて書く。1・2 段目は `#[gpui::test]` で書く。
3 段目はテストでは確かめられないため (M3)、対応する MD3 部品を gallery に載せるときに [a11y-tree スキル](../../.agents/skills/a11y-tree/SKILL.md) の手順で確かめる

1. 入力と状態遷移
1. 要素が書く a11y ノードの属性
1. 実際の a11y ツリー (親子関係・子の内容から求まる名前) と支援技術からの操作の配送

## M0: Button

参照する APG:

- [Button](../../.agents/skills/a11y/references/aria-practices/content/patterns/button/button-pattern.html)

## M1: Toggle 系

対象: Checkbox (中間状態を含む)、Switch

押下状態を持つ Toggle Button は、ToggleGroup の項目と合わせて M4 で作る

決めたこと:

- 部品は内部で状態を持たず、呼び出し側が持つ API だけを用意する
- Button の内部 (フォーカス、無効状態、クリックの扱い) は共有せず、各部品に複製することを許容する。将来は 1 つにまとめたい
- ラベルとグループは呼び出し側が組み立てる
  - Checkbox の名前は子孫に置いた id を持つテキスト (`gpui::text!`) か `aria_label` で、Switch の名前は `aria_label` で与える。別の要素を参照する `labelled_by` は作れない (M3)
  - ラベルのクリックで切り替えるのは要件ではない。必要なら呼び出し側がラベルの `on_click` で状態を更新する
  - Checkbox のグループに求められるのは role `group` と名前だけで、キー操作の要件は無い。`div().role(Role::Group).aria_label(..)` で作る
- Mixed の親の状態の算出と、子をまとめて更新するのは呼び出し側が行う。Mixed で操作されたときは `CheckedState::Checked` を要求する
- Checkbox と Switch は role と状態の型だけが違うため、`Choice<K>` の型エイリアスとして実装する。Toggle Button は `Choice` に含めない
- Checkbox と Switch は Enter では操作しない (APG では Checkbox は定義なし、Switch は任意)

気を付けること:

- AccessKit は Switch の名前を子の内容から求めない (accesskit_consumer の `Node::labelled_by`)。名前は `aria_label` で与える必要がある
- Switch は Mixed を持てない (WAI-ARIA は mixed を false として扱う)
- Switch は、状態によってラベルを変えない

参照する APG:

- [Checkbox](../../.agents/skills/a11y/references/aria-practices/content/patterns/checkbox/checkbox-pattern.html)
- [Switch](../../.agents/skills/a11y/references/aria-practices/content/patterns/switch/switch-pattern.html)

## M2: md3-components で M0・M1 を使う

対象: MD3 の Button (各バリアント)、Icon button (toggle のバリアントを除く)、Checkbox、Switch

headless 部品だけを先に増やすと、装飾側で使ったときに API の不足が見つかり作り直しになるため、早い段階で一度 MD3 まで作る。
以降のマイルストーンでも、対応する MD3 部品を作って装飾側の要求を確かめる

完了条件:

- [ ] state layer、ripple、focus ring を headless の `Styled` / `InteractiveElement` 経由で付けられる
- [ ] 装飾のために headless 側へ追加した API があれば、M1 の部品にも反映する
- [ ] gallery に表示する

## M3: 調査 (関連付け・テスト・モーダル・フォーカス)

後続のマイルストーンの前提を確かめた。固定版の gpui (zed@37faef81) と、zed@cebd5fc6 (2026-11-09 の main) で同じ結論になる

### 要素どうしの関連付け

`controls`、`labelled_by`、`described_by` は、gpui が保証する方法では作れない。

- 関連先の `NodeId` を得る `GlobalElementId::accesskit_node_id` は `pub(crate)` で、`Element::write_a11y_info` も自分の `NodeId` を受け取らない
- `GlobalElementId` の `Hash` は `NodeId` と同じ内部のハッシュ値を書くため、独自の `Element::prepaint` で受け取った id を独自の `Hasher` に通せば `NodeId` が得られる。
  ただし `Hash` の実装は文書化された契約ではなく、gpui の更新で黙って壊れる。
  関連先がそのフレームのツリーに無いと accesskit_consumer が panic し (iterators.rs の `node_by_id(..).unwrap()`)、gpui の `repair_tree_update` は子の参照しか直さない。このため採らない

決めたこと:

- 名前と説明は、関連付けではなく文字列 (`aria_label`、`aria_description`) で与える
- 関連付けが要る部品は、関連先の名前を文字列で写して代える。gpui に関連付けの API が入ったら見直す
- 子の内容から名前を求めるのは、AccessKit では Button、CheckBox、Link、MenuItem 系、RadioButton などに限られる (accesskit_consumer の `Node::labelled_by`)。Switch、Tab、ListBoxOption などの名前は `aria_label` で与える

### 3 段目の検証

テスト用の platform は a11y を有効にできないため、`#[gpui::test]` では確かめられない。

- テスト用のウィンドウは `PlatformWindow::a11y_init` を実装しておらず (既定の空実装)、a11y ツリーを組み立てない
- `TestPlatform` は `pub(crate)` で、`TestAppContext` の platform は差し替えられない
- `Application::with_platform` に独自の `Platform` を渡せば TreeUpdate を受け取れるが、描画やテキストを含む `Platform` と `PlatformWindow` をすべて実装する必要があり、採らない

決めたこと:

- 3 段目は、実機 (Linux の AT-SPI) で gallery を動かし、ツリーの書き出しと操作の送信で確かめる。手順は [a11y-tree スキル](../../.agents/skills/a11y-tree/SKILL.md)
- headless の部品は gallery に直接載せないため、対応する MD3 部品の PR で確かめる

この手順で次のことを確かめた:

- 支援技術からの click は配送され、ハンドラが呼ばれる。ただし部品が `on_a11y_action` で Click を受けない限り、gpui はノードの中央へのマウスの押下と離しとして配送する (`Window::handle_a11y_action`)。
  そのため、ほかの要素に覆われた部品やスクロールで隠れた部品には届かず、その位置の別の要素が押される。入力の種類もマウスになり、focus-visible の表示が消える
- `&str`・`String`・`SharedString` をそのまま子にしたテキストは a11y ノードを持たず、親の名前にもならない。gallery のボタンはすべて名前が空だった。`gpui::text!` で作ると名前になる
- AT-SPI のアダプタは disabled を role によって変えて渡す。checkbox などは read-only になるが、button は sensitive のままで、Linux では無効と伝わらない。
  手掛かりは Click 操作が無いことと、`Availability::Disabled` ではフォーカスできないことだけになる。AccessKit のアダプタの制約として受け入れる
- `.role(Role::GenericContainer)` は debug ビルドで panic する (`StatefulInteractiveElement::role` の検査)。独自の `Element::a11y_role` で返すと panic せず、AccessKit はそのノードを飛ばして子を親に付ける

確かめた結果から決めたこと:

- 部品は支援技術の Click を `on_a11y_action` で受け、位置に依存しないようにする。M0・M1 の部品にも入れる (随時)

### モーダル

- 背景を支援技術から隠す: a11y ノードに hidden を書くと、AccessKit は子孫ごとツリーから除く (実機で確認)。ただしフォーカスのあるノードは hidden の中でも除かれないため、開くときにフォーカスをモーダルの中に移す。
  gpui の公開 API には hidden を書く手段が無いため、headless が背景を包む要素を用意する。包む要素の role を `GenericContainer` にすれば、閉じている間に余計なノードを出さない
- モーダル自体: `deferred` で描いた要素は、a11y ツリーではウィンドウの直下に入り、背景の hidden の影響を受けない。ノードには role `Dialog` と `modal` を書く (UIA が modal を渡すのは Dialog と AlertDialog だけ)
- 背景のポップアップ: 背景から `deferred` で描いた要素も、ウィンドウの直下に入るため hidden の外に出る。モーダルを開く前に閉じる
- Tab 移動: `Window::focus_next` はウィンドウ全体の Tab stop を対象にする。モーダルの要素に key context を付け、その context で Tab と Shift+Tab に別の action を割り当て、`focus_next` / `focus_prev` を、モーダルの `FocusHandle::contains_focused` が真になるまで繰り返す。一周しても入らなければ動かさない (M9 で確かめる)
- マウス: ウィンドウ全体を覆う背景の幕に `occlude` を付けて、背景の hover とクリックを止める。幕は `deferred` の priority を背景のポップアップより高くして描く
- ショートカット: キー入力はフォーカスのある要素から祖先へ届く。`deferred` の要素の祖先は `deferred` を書いた位置の祖先になるため、モーダルはルートの近くに書く。アプリ全体の `on_action` と `intercept_keystrokes` はモーダルの間も届くため、止めるのはアプリ側の責任とする
- 支援技術からの操作: hidden の子孫は支援技術から見えず、操作の対象にならない

### フォーカスと Tab stop

独立に制御できる。

- Tab の順に入るかは、`track_focus` に渡した `FocusHandle` の複製の `tab_stop` で決まる。`tab_stop(false)` でも `Window::focus`、クリック、支援技術の Focus 操作ではフォーカスできる
- M0 の Button と M1 の Choice は描画のたびに `tab_stop(true)` を書くため、矢印キー移動の項目には使えない。M4 の項目は Tab stop をグループが決める

### aria_active_descendant

`aria_active_descendant` は、要素が自分を active descendant として申告する仕組みで、a11y ツリーの祖先にフォーカスのあるノードがある場合だけ効く。

- `deferred` で描いた一覧はウィンドウの直下に入るため、一覧の外 (ボタンなど) にフォーカスがあると効かない
- フォーカスのあるノードの子孫で 2 つ以上の要素が申告するか、フォーカスのあるノード自身が申告すると、debug ビルドで panic する

## M4: 矢印キー移動の基盤と RadioGroup / ToggleGroup

対象: RadioGroup、ToggleGroup (複数選択。MD3 の Segmented button の複数選択で使う)、Toggle Button (押下状態を持つボタン。単体でも使い、ToggleGroup の項目にもなる)

決めること:

- [ ] 矢印キー移動の共通部品の形 (グループがフォーカスを持つ項目を管理する)
- [ ] 項目の追加・削除・無効化と、すべて無効なときのフォーカスの移動先
- [ ] 選択数の上限、選択の解除を許すか、フォーカスの移動で選択を変えるか (RadioGroup と ToggleGroup で別々に決める)

気を付けること:

- Toggle Button は状態によってラベルを変えない。ラベルを変える場合は押下状態を公開しない
- Toggle Button は M1 の `Choice` に含めない

参照する APG:

- [Keyboard Interface (グループ内のフォーカス移動)](../../.agents/skills/a11y/references/aria-practices/content/practices/keyboard-interface/keyboard-interface-practice.html)
- [Radio Group](../../.agents/skills/a11y/references/aria-practices/content/patterns/radio/radio-group-pattern.html)
- [Button (Toggle Button と ToggleGroup の各項目)](../../.agents/skills/a11y/references/aria-practices/content/patterns/button/button-pattern.html)

## M5: Slider

対象: Slider (単一の値)

M4 の矢印キーの扱いを使う。値の公開は gpui の `aria_numeric_value` など、支援技術からの操作は `on_a11y_action` で実現できる

参照する APG:

- [Slider](../../.agents/skills/a11y/references/aria-practices/content/patterns/slider/slider-pattern.html)

## M6: Tabs

対象: Tabs

tab から tabpanel への `controls` と、tabpanel の `labelled_by` は作れない (M3)。tabpanel には tab と同じ名前を `aria_label` で与え、`controls` は公開しない。
AccessKit は tab の名前を子の内容から求めないため、tab 自身の名前も `aria_label` で与える

決めること:

- [ ] フォーカスの移動と同時に選ぶか (自動) 、Enter / Space で選ぶか (手動)。自動にするのは、パネルの表示に目立つ遅延が無い場合に限る

参照する APG:

- [Tabs](../../.agents/skills/a11y/references/aria-practices/content/patterns/tabs/tabs-pattern.html)

## M7: 非モーダルのオーバーレイ基盤と Tooltip

対象: アンカーに合わせて配置し開閉する基盤 (`anchored` / `deferred`)、Tooltip

Menu と Select がこの基盤を使う。Dialog の完成を待たない

- `deferred` で描いたポップアップは、a11y ツリーではアンカーではなくウィンドウの直下に入る (M3)
- Tooltip の `aria-describedby` は作れないため、トリガーの `aria_description` に Tooltip の文字列を写す
- 支援技術の Click はノードの中央へのマウス操作として届くため、ポップアップが覆う位置の部品は `on_a11y_action` で Click を受ける (M3)

参照する APG:

- [Tooltip](../../.agents/skills/a11y/references/aria-practices/content/patterns/tooltip/tooltip-pattern.html)

## M8: Menu

対象: Menu Button + Menu

M4 の矢印キー移動と M7 のオーバーレイ基盤を組み合わせる。menu button からメニューへの `controls` は作れない (M3)

参照する APG:

- [Menu Button](../../.agents/skills/a11y/references/aria-practices/content/patterns/menu-button/menu-button-pattern.html)
- [Menu and Menubar](../../.agents/skills/a11y/references/aria-practices/content/patterns/menubar/menu-and-menubar-pattern.html)

## M9: Dialog

対象: Dialog (モーダル)

M7 の基盤に、M3 で決めた背景の操作を止める仕組み (背景を hidden で包む要素、Tab の閉じ込め、`occlude` の幕) を組み合わせる

参照する APG:

- [Dialog (Modal)](../../.agents/skills/a11y/references/aria-practices/content/patterns/dialog-modal/dialog-modal-pattern.html)

## M10: Select (Listbox)

対象: Select (APG の Select-Only Combobox)

APG の例はフォーカスを combobox に残し、`aria-activedescendant` で一覧の項目を指す。gpui ではポップアップの一覧を指せないため (M3)、
開いたら一覧にフォーカスを移し、一覧の中で項目を active descendant にする。combobox の `controls` も作れない。
AccessKit は option の名前を子の内容から求めないため、option の名前は `aria_label` で与える

参照する APG:

- [Combobox](../../.agents/skills/a11y/references/aria-practices/content/patterns/combobox/combobox-pattern.html)
- [Select-Only Combobox の例](../../.agents/skills/a11y/references/aria-practices/content/patterns/combobox/examples/combobox-select-only.html)

## M11: TextField

gpui の `EntityInputHandler` を前提にする。最初は単一行に限る。ラベルへの `labelled_by` は作れないため、名前は `aria_label` で与える (M3)

決めること (M4 までに決める):

- [ ] 文字列・選択・IME・クリップボードを扱う編集モデルと、位置・範囲・描画の対応を渡す境界
- [ ] 支援技術にテキストと選択を公開する方法 (gpui の `TextRun` / `TextSelection`)

参照する APG:

- 対応する APG のパターンは無い

## 随時

操作がほとんど無く、他のマイルストーンに依存しないもの。手が空いたときに入れる

- [ ] 支援技術の Click を `on_a11y_action` で受ける (M0 の Button、M1 の Choice、M4 の部品。M3)
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
