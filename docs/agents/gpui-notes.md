# GPUI の注意点（gpui @ 37faef8）

実装中に踏んだ GPUI の挙動と、その回避策。rev を上げたら各項目をソースで確認し直す。
API はいつも固定 rev のソース（`~/.cargo/git/checkouts/zed-*/37faef8/crates/gpui/src`）を grep して確かめる。

## スクロール

- **div のホイールは伝播を止めない。** `elements/div.rs` の `paint_scroll_listener` は、マウスの下にある
  スクロール領域すべてに差分を適用する（端の判定もない）。入れ子にすると外側も一緒に動く。
  → 内側のスクロール領域には `crates/gpui-headless/src/scroll.rs` の `NestedScroll` を付ける
  （`capture()` を子に、`on_scroll()` を領域に）。内側が動いたら `stop_propagation`、端なら外へ渡す。
  縦・横どちらの領域にも効く（横だけの領域では縦ホイールが x に振り替えられるので、両軸の差を見る）。
  公開 API（`gpui_headless::NestedScroll`）なので、gallery や利用者の素の div にも付けられる。
  精密スクロール（トラックパッド）は `TouchPhase::Started` から `Ended` まで内側が持つ。
- **リスナーの順序。** gpui は「利用者のリスナー → 既定処理 → 子」の順に登録し、bubble は逆順に呼ぶ。
  そのため利用者の `on_scroll` が呼ばれる時点で、内側の既定処理（offset の更新）は終わっていて、祖先はまだ動いていない。
  `uniform_list` も同じ `Interactivity::paint` を使う。
- `ScrollHandle::bounds()` は表示領域（ウィンドウ座標）、`max_offset()` は最大距離、`offset()` は負の値。
  `UniformListScrollHandle` の中身は `.0.borrow().base_handle`。
- 部品の中に新しいスクロール領域を作ったら `NestedScroll` と `Scrollbar`（`crates/md3-components/src/scrollbar.rs`）を付ける。
  オーバーレイの本文なら `scroll::content(..)` がまとめてやる（Dialog / Sheet が例）。
  バーは領域の最後の子に置く（`ScrollbarSource` を実装すれば独自の仮想リストにも付く）。
- **`Window::focus` も Tab の移動もスクロールしない。** `ScrollScope::new(handle.clone(), div().overflow_y_scroll().track_scroll(&handle))` で領域を包む。操作部品は共通の `control()`、Card、
  TextField / SearchBar で `reveal_on_focus` を持ち、キーボードで移ったときだけ祖先の領域を内側から順に調整する
  （WCAG 2.2 の 2.4.11）。独自のフォーカス可能要素には `.relative()` と
  `.child(reveal_on_focus(&id, focus.clone(), window, cx, margin))` を付ける。margin はリングの太さと外側の間隔の和を渡す。マウスのクリックと、フォーカスが
  変わらない再描画では動かさない。`uniform_list` は `UniformListScrollHandle::scroll_to_item` を使う（LazyList）。
- **描画中の `Window::refresh()` は何もしない。** `window.rs` の `invalidator.not_drawing()` で弾かれる。
  prepaint でオフセットを変えたら `window.on_next_frame(|window, _| window.refresh())` で次のフレームを予約する。
  その場の refresh だけでは、次の入力まで新しい位置が表示されない。
- `ScrollScope` の Global のスタックは同期的な子の prepaint 中だけ push / pop する。遅延描画のオーバーレイは
  pop 後に prepaint されるので、背後のページのスコープを引き継がない。箱を足さず子の LayoutId を返すため、
  flex の寸法や a11y の親子関係も変えない。
- テスト用プラットフォームは次フレームのコールバックを自動配信しない。`tests/navigation.rs` は入力前に
  `simulate_next_frame` の配信だけをタスクに予約し、入力後は `run_until_parked()` だけで検証する。
  手動の refresh / draw を足すと、描画中の refresh が無効になる不具合を隠してしまう。

## レイアウト

- `.h()` の要素は親の高さが足りないと縮む。`.min_h()` に変えると縮まなくなり、100% の寸法が変わることがある（Medium の TopAppBar の上の行）。変えたら部品全体の高さをテストで固定する。
- `.flex()` の中の文字は親の `max_w` で折り返さないことがある（ツールチップの根の要素で確認）。折り返す文字は flex にせず Block のままにする（Plain Tooltip）。
- 定まった幅のない親の中で折り返す文字は、幅が中身に合わせて広がるだけで折り返さない。折り返すなら親の幅を `.w()` で決める（NavigationRail）。
- `uniform_list` は 1 項目の高さを全行に使う。項目が折り返すと重なるので、LazyList の項目は 1 行に省略する。

## モーション

- バネは gpui の `SpringConfig::new(stiffness, damping, mass)`、`SpringAnimation::new(config).to(target)`、
  `AnimationExt::with_spring(id, anim, |el, v| ...)`、`SpringAnimationElement::map_element` を使う。
  `SpringTarget` は f32 / Pixels / bool。設定値は `Theme::spring(MotionToken)`（md3-theme の `motion.rs`）から取る。
- `with_spring` と `with_animation` は `cx.reduce_motion()` が true なら最終値に飛ぶ。
  独自のタイマー・フェード（例: Scrollbar のフェード）は自分で `reduce_motion` を見る。
- **gpui は OS の「視差効果を減らす」設定を読まない。** `App::set_reduce_motion` をアプリが呼ぶ。
- 時間は `cx.background_executor().now()` を使う（wasm で `std::time::Instant` は使えない）。
  フレームごとに描き直すには paint 中に `window.request_animation_frame()`。終わったら止める。

## 状態とイベント

- **マウス移動で入力方法が Mouse に変わる。** `Window::dispatch_event` は座標が変わらない `MouseMove` でも切り替え、Windows は移動がなくても WM_MOUSEMOVE を送る。`is_focus_visible` はキーボードフォーカスの表示を保持し、フォーカス喪失で解除する。共通の操作要素と Card はクリックでも解除する。gpui は `FocusHandle` の ID を公開しないため、寿命を延ばさない `WeakFocusHandle` を識別に使う。
- **キーの解決の前に描き直される。** `dispatch_event` は KeyDown で入力方法を Keyboard にして refresh し、`dispatch_key_event` はキーを解決する前に描く。そのため `focus_visible` で付けた key context が、その前のフレームで見えていなかったのに同じキーを受ける。ポインタで得たフォーカスは離れるまで状態で覚える（`FocusTooltip`）。
- **キーの割り当ては `on_key_down` より先に動く。** 親のコンテキストの割り当てに勝つには、より深い要素に `key_context` を付ける（ツールチップの Esc と Dialog）。
- **フォーカス中の要素が消えると、キーアクションはルートの dispatch node から配送される。** ルート要素の `on_action` には届かないが、グローバルな `cx.on_action` には届く。
- テストで `window.focus_next` だけを呼んでも入力方法は Keyboard にならない。`support::press` で実際のキーを送る。
- `defer_draw(.., None)` はクリップされない。基準の要素がスクロールで隠れても描かれるので、prepaint で `window.content_mask().bounds` との交差を確かめる。
- `a11y_synthetic_children` は 1 要素に 1 つのクロージャしか持てない。状態と子を両方書くときは `A11yExt::a11y_with_children` を使う。
- `debug_selector` / `debug_bounds` はデバッグビルドで、gpui の `test-support` を有効にしたときだけ記録される。テストに `#![cfg(debug_assertions)]` を付ける。結合テストはライブラリを `cfg(test)` なしでコンパイルするので、`cfg(test)` の分岐は効かない。
- **Entity の更新中に同じ Entity を更新すると panic する**（`cannot update ... while it is already being updated`）。
  利用者のコールバック（例: Snackbar の `on_result`）を Entity の update の中から呼ばない。`cx.defer` で後に回す。
- **フォーカスが移ると key-up が届かない。** Space / Enter の押下で状態を持つ部品は、フォーカスを失ったときにも
  押下を解除する（Switch で修正済み）。
- 部品の内部状態は `window.use_keyed_state(ElementId, cx, init)`。ID は `ElementId::NamedChild(id, "名前")` で派生させる。
- 描画とヒットテストを自前でやるときは `canvas(prepaint, paint)` を `.absolute().size(px(0.))` で置き、
  prepaint で `window.insert_hitbox`、paint で `window.on_mouse_event` を登録する（Scrollbar が例）。
  ドラッグはウィンドウ全体の `MouseMoveEvent` / `MouseUpEvent` を capture で見ると、要素の外に出ても続く。
- 同じ `FocusHandle` を 2 つの要素に付けると、a11y 有効時のデバッグビルドで panic する。

## プラットフォーム

- Windows でアプリから IME の変換を終了するには、入力ハンドラーを残し、フレーム終了時の `accepts_text_input` を false にする。gpui が描画後に `CPS_COMPLETE` を送り、確定文字をハンドラーに返す。`on_next_frame` はそのフレームの描画より先に走るため、false の解除は paint から予約する。
- gpui の `on_blur` はウィンドウの非アクティブ化でも発火する。要素自体のフォーカス移動と区別するには `FocusHandle::is_focused` を確認する。
- `on_blur` は `Window::draw` の最後で走る。フォーカスの移動先はその時点で描画済みなので、移動先に描画で処理させたいときは `window.on_next_frame(|window, _| window.refresh())` で描き直す（`.cached()` のビューは移動元の `cx.notify()` では描かれない）。[描画中の refresh は無効](#%E3%82%B9%E3%82%AF%E3%83%AD%E3%83%BC%E3%83%AB)なので次のフレームまで遅らせる。
- `invalidate_character_coordinates` の次フレームのコールバックは次の描画より先に走る。最新の入力位置を返せるよう、レイアウトを保存した描画後に呼ぶ。
- gpui_web の `update_ime_position` は空実装（`crates/gpui_web/src/window.rs`）、IME 用の隠れた textarea は `top: 0; left: 0` に固定されている（`ime_mirror.rs`）。Web の候補ウィンドウは左上のままになる。
- Linux の gallery は Wayland と X11 の両方で動く。xdotool で操作・撮影するときは `WAYLAND_DISPLAY` を外して X11 で起動する。
- nix の外で起動すると「failed to resolve font」で panic することがある。`nix develop -c` の中で起動する。
- Web はシステムフォントを読めない（`MD3_WEB_FONTS`、guide.md の「Web」）。wasm では AccessKit が無い。
- Linux（AT-SPI）では AccessKit が一部の役割で disabled を伝えない（`docs/a11y-audit.md`）。
