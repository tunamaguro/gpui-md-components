# GPUI の注意点（gpui @ 37faef8）

実装中に踏んだ GPUI の挙動と、その回避策。rev を上げたら各項目をソースで確認し直す。
API はいつも固定 rev のソース（`~/.cargo/git/checkouts/zed-*/37faef8/crates/gpui/src`）を grep して確かめる。

## スクロール

- **div のホイールは伝播を止めない。** `elements/div.rs` の `paint_scroll_listener` は、マウスの下にある
  スクロール領域すべてに差分を適用する（端の判定もない）。入れ子にすると外側も一緒に動く。
  → 内側のスクロール領域には `crates/md3-components/src/scroll.rs` の `NestedScroll` を付ける
  （`capture()` を子に、`on_scroll()` を領域に）。内側が動いたら `stop_propagation`、端なら外へ渡す。
  縦・横どちらの領域にも効く（横だけの領域では縦ホイールが x に振り替えられるので、両軸の差を見る）。
  公開 API（`md3_components::NestedScroll`）なので、gallery や利用者の素の div にも付けられる。
  精密スクロール（トラックパッド）は `TouchPhase::Started` から `Ended` まで内側が持つ。
- **リスナーの順序。** gpui は「利用者のリスナー → 既定処理 → 子」の順に登録し、bubble は逆順に呼ぶ。
  そのため利用者の `on_scroll` が呼ばれる時点で、内側の既定処理（offset の更新）は終わっていて、祖先はまだ動いていない。
  `uniform_list` も同じ `Interactivity::paint` を使う。
- `ScrollHandle::bounds()` は表示領域（ウィンドウ座標）、`max_offset()` は最大距離、`offset()` は負の値。
  `UniformListScrollHandle` の中身は `.0.borrow().base_handle`。
- 部品の中に新しいスクロール領域を作ったら `NestedScroll` と `Scrollbar`（`crates/md3-components/src/scrollbar.rs`）を付ける。
  オーバーレイの本文なら `scroll::content(..)` がまとめてやる（Dialog / Sheet が例）。
  バーは領域の最後の子に置く（`ScrollbarSource` を実装すれば独自の仮想リストにも付く）。

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

- Linux の gallery は Wayland と X11 の両方で動く。xdotool で操作・撮影するときは `WAYLAND_DISPLAY` を外して X11 で起動する。
- nix の外で起動すると「failed to resolve font」で panic することがある。`nix develop -c` の中で起動する。
- Web はシステムフォントを読めない（`MD3_WEB_FONTS`、guide.md の「Web」）。wasm では AccessKit が無い。
- Linux（AT-SPI）では AccessKit が一部の役割で disabled を伝えない（`docs/a11y-audit.md`）。
