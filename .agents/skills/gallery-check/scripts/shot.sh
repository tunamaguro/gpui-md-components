#!/usr/bin/env bash
# gallery のウィンドウを PNG に撮る。
#   shot.sh <出力.png> [WxH+X+Y]   2 番目を渡すとその範囲だけ切り出す（ImageMagick の -crop 形式）
set -euo pipefail
out=$1; crop=${2:-}
nix shell nixpkgs#xdotool nixpkgs#xwd nixpkgs#imagemagick -c bash -c '
  set -euo pipefail
  id=$(xdotool search --name "md3 gallery" | head -1)
  tmp=$(mktemp --suffix=.xwd)
  xwd -id "$id" -silent > "$tmp"
  if [ -n "$2" ]; then magick "xwd:$tmp" -crop "$2" +repage "$1"; else magick "xwd:$tmp" "$1"; fi
  rm -f "$tmp"
' _ "$out" "$crop"
echo "$out"
