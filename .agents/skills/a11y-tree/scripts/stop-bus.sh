#!/usr/bin/env bash
# start-bus.sh で有効にした支援技術の状態を戻し、start-bus.sh が起動した AT-SPI のバスを止める
set -euo pipefail

pid_file="${XDG_RUNTIME_DIR:-/tmp}/a11y-tree-bus.pid"

# 有効なままだと、以降に起動したアプリも a11y ツリーを組み立てる
dbus-send --session --print-reply --dest=org.a11y.Bus /org/a11y/bus \
  org.freedesktop.DBus.Properties.Set string:org.a11y.Status string:IsEnabled variant:boolean:false \
  >/dev/null 2>&1 || true

if [[ -f $pid_file ]]; then
  # start-bus.sh は setsid で起動するため、バスとレジストリも同じプロセスグループに入る
  kill -- "-$(cat "$pid_file")" 2>/dev/null || true
  rm -f "$pid_file"
fi
echo "AT-SPI bus is stopped"
