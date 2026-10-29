#!/usr/bin/env bash
# gallery のウィンドウに対して xdotool を実行する。座標はウィンドウ内（スクリーンショットの座標と同じ）。
#   xdo.sh move X Y            ポインタを移す（ホバー）
#   xdo.sh click X Y           左クリック
#   xdo.sh scroll X Y N        (X,Y) でホイール N 段（正で下、負で上）
#   xdo.sh drag X1 Y1 X2 Y2    左ボタンでドラッグ
#   xdo.sh key KEY...          キー入力（例: Tab, space, Escape, shift+Tab）
set -euo pipefail
nix shell nixpkgs#xdotool -c bash -c '
  set -euo pipefail
  id=$(xdotool search --name "md3 gallery" | head -1)
  cmd=$1; shift
  case "$cmd" in
    move)   xdotool mousemove --window "$id" "$1" "$2" ;;
    click)  xdotool mousemove --window "$id" "$1" "$2" click 1 ;;
    scroll) xdotool mousemove --window "$id" "$1" "$2"; n=$3; b=5
            if [ "$n" -lt 0 ]; then b=4; n=$((-n)); fi
            for _ in $(seq 1 "$n"); do xdotool click "$b"; sleep 0.05; done ;;
    drag)   xdotool mousemove --window "$id" "$1" "$2" mousedown 1
            for i in 1 2 3 4 5; do
              xdotool mousemove --window "$id" $(( $1 + ($3 - $1) * i / 5 )) $(( $2 + ($4 - $2) * i / 5 )); sleep 0.1
            done
            xdotool mouseup 1 ;;
    key)    xdotool windowfocus "$id" 2>/dev/null || true; xdotool key --delay 80 "$@" ;;
    *) echo "unknown: $cmd" >&2; exit 2 ;;
  esac
' _ "$@"
