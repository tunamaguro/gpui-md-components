# md3-components から見た gpui-headless の使いにくさ

M2 (MD3 の Button・Icon button・Checkbox・Switch) を作りながら、装飾側で困ったことを記録する。
headless の改善を検討するときの材料にする

## 操作の状態 (hover・押下・フォーカスの可視化) を読めない

- headless は押下中かどうかを公開しないため、MD3 側で `on_mouse_down` / `on_mouse_up` / `on_mouse_up_out` / `on_key_down` / `on_key_up` / `on_hover` を付けて自前で記録している (`src/indication.rs` の `Interaction`)
  - gpui のキーボードによるクリック (Enter / Space の key down で保留し key up で発火) と同じ判定を、押下の表示のために複製している
  - キーで押している間にフォーカスが外れると key up を受け取れないため、描画のたびにフォーカスを見て押下を解除する処理が要る
- どのキーで操作できるかを MD3 側も知っている必要がある。Button は Enter と Space、Checkbox / Switch は Space だけで、headless の振る舞いと二重管理になっている
- 押下・hover の状態は、Expressive の押下時の形の変化 (バネの目標値。Switch ではつまみの大きさと位置)・状態レイヤーの不透明度・高さ (影) の切り替えに使う。`Styled` の `.hover()` / `.active()` だけでは足りない

## フォーカスを装飾側で判定するために `track_focus` が必須

- headless が内部で持つフォーカスは外から読めないため、フォーカスリングを出すには MD3 側で `FocusHandle` を作って `track_focus` で渡している
- フォーカスが「見えるべきか」(キーボード操作中か) は `window.last_input_was_keyboard()` を MD3 側で組み合わせて判定している。部品ごとに同じ判定が要る

## 無効状態の扱い

- 無効な部品では押下の記録リスナーを付けないよう、MD3 側でも無効かどうかで分岐している
- 押している途中で無効になった場合の押下・hover の解除も MD3 側で行っている

## `on_click` が `StatefulInteractiveElement::on_click` を隠す

- headless の `Button::on_click` は固有メソッドで、gpui の同名のトレイトメソッドを隠す。装飾側でクリックのリスナーを足したいときは、トレイトを明示して呼ぶ必要があり、無効時に呼ばれない保証もなくなる

## エラーの状態を公開できない

- MD3 の Checkbox にはエラーの見た目 (`md.comp.checkbox.*.error.*`) があるが、headless の `Checkbox` は支援技術に invalid を公開する手段を持たない。見た目だけのエラーになるのを避けるため、MD3 側ではエラーの指定を用意していない

## テスト

- MD3 の部品の `render` は `impl IntoElement` を返すため、headless のテストのように a11y ノードを書き出して role・名前・状態を確かめる方法がない。MD3 側では操作の結果だけをテストしている

## gpui 側の制約 (headless の範囲外だが装飾で困ったこと)

- 角丸の形でクリップできず、放射状のグラデーションもないため、リップルは角丸の矩形で切り取った多角形を重ねて近似している
- `PathBuilder` の線の端の形を選べない (`LineCap` が re-export されていない) ため、Checkbox のチェックマークは Compose の `StrokeCap.Square` に合わせて端を太さの半分だけ延ばしている
- バネの目標値に色を渡せず、1 つの `with_spring` の中から別の `with_spring` に値を渡す方法もないため、Checkbox は色・線を引く割合・Mixed への変形のバネを入れ子の `div` に 1 つずつ付けている
- gpui は Tab にフォーカスの移動を割り当てないため、アプリ (gallery) で `tab` / `shift-tab` を `focus_next` / `focus_prev` に割り当てている (M3 の調査対象)
