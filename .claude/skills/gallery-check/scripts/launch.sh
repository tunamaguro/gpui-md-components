#!/usr/bin/env bash
# gallery をビルドして X11 で起動する（xdotool で操作・撮影するため Wayland を外す）。
#   launch.sh [ページ] [ログ]   ページ: foundations buttons selection inputs overlays navigation containment expressive
# 既に起動している gallery は止める。起動後、ウィンドウが出るまで待ってから戻る。
set -euo pipefail
root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
page=${1:-foundations}
log=${2:-${TMPDIR:-/tmp}/md3-gallery.log}
cd "$root"
pkill -x gallery || true
nix develop -c cargo build -p gallery 2>&1 | grep -E "^(error|warning: unused)" || true
(env -u WAYLAND_DISPLAY MD3_GALLERY_PAGE="$page" ${MD3_REDUCE_MOTION:+MD3_REDUCE_MOTION=$MD3_REDUCE_MOTION} \
  nix develop -c ./target/debug/gallery >"$log" 2>&1 &)
for _ in $(seq 1 60); do
  if nix shell nixpkgs#xdotool -c xdotool search --name "md3 gallery" >/dev/null 2>&1; then
    sleep 2; echo "gallery: $page (log: $log)"; exit 0
  fi
  sleep 1
done
echo "gallery のウィンドウが出ない。ログ: $log" >&2; tail -20 "$log" >&2; exit 1
