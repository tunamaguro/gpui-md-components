#!/usr/bin/env bash
# Linux（X11）で gallery の各ページの AT-SPI ツリーを書き出す（docs/a11y-audit.md の手順 2 の自動化）。
#
#   nix develop -c tools/a11y-tree/run.sh [出力先] [ページ...]
#
# 専用の D-Bus セッションと AT-SPI のバスを立て、gallery（target/debug/gallery、先に `cargo build -p gallery`）を
# ページごとに起動して、スクロールしながら見えている部品のツリーを `<出力先>/<ページ>.txt` に書く
# （ログは `<ページ>.log`）。
# 道具（at-spi2-core、pyatspi、xdotool、dbus）は nixpkgs から取得する。プロジェクトの依存ではない。
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
out=$(realpath -m "${1:-$root/target/a11y-tree}")
shift || true
pages=("$@")
if [ ${#pages[@]} -eq 0 ]; then
  pages=(foundations buttons selection inputs overlays navigation containment)
fi
mkdir -p "$out"

build() { nix build --no-link --print-out-paths "$@"; }
atspi=$(build nixpkgs#at-spi2-core^out)
dbus=$(build nixpkgs#dbus^out)
xdotool=$(build nixpkgs#xdotool^out)
python=$(build --impure --expr \
  'let p = import (builtins.getFlake "nixpkgs") {}; in p.python3.withPackages (ps: [ ps.pyatspi ps.pygobject3 ])')
typelibs=$atspi/lib/girepository-1.0:$(build nixpkgs#gobject-introspection^out)/lib/girepository-1.0:$(build nixpkgs#glib^out)/lib/girepository-1.0

# 1 ページ分。dbus-run-session の中で動く。
if [ "${A11Y_TREE_PAGE:-}" != "" ]; then
  page=$A11Y_TREE_PAGE
  export PATH=$xdotool/bin:$dbus/bin:$PATH GI_TYPELIB_PATH=$typelibs
  # IsEnabled は gsettings に保存されるので、dconf の無い環境でも保てるようメモリに置く。
  GSETTINGS_BACKEND=memory "$atspi/libexec/at-spi-bus-launcher" --launch-immediately &
  sleep 1.5
  dbus-send --session --print-reply --dest=org.a11y.Bus /org/a11y/bus \
    org.freedesktop.DBus.Properties.Set string:org.a11y.Status string:IsEnabled variant:boolean:true >/dev/null
  # レジストリを先に起こしておく（gallery が起動時に登録できるように）。
  "$python/bin/python3" -I -c 'import pyatspi; pyatspi.Registry.getDesktop(0)'
  (cd "$root" && env -u WAYLAND_DISPLAY MD3_GALLERY_PAGE="$page" ./target/debug/gallery >/dev/null 2>&1 &)
  sleep 12
  window=$(xdotool search --name 'md3 gallery' | head -1)
  : >"$out/$page.txt"
  for step in $(seq 0 8); do
    echo "=== scroll $step" >>"$out/$page.txt"
    "$python/bin/python3" -I "$here/dump.py" >>"$out/$page.txt" 2>&1
    xdotool mousemove --window "$window" 600 600
    for _ in $(seq 10); do xdotool click 5; done
    sleep 1.5
  done
  xdotool windowkill "$window"
  exit 0
fi

for page in "${pages[@]}"; do
  A11Y_TREE_PAGE=$page "$dbus/bin/dbus-run-session" --dbus-daemon="$dbus/bin/dbus-daemon" --config-file="$dbus/share/dbus-1/session.conf" -- "$here/run.sh" "$out" >"$out/$page.log" 2>&1
  echo "$page: $(grep -c '^ *\[' "$out/$page.txt") 行 → $out/$page.txt"
done
