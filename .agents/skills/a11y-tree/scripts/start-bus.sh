#!/usr/bin/env bash
# セッションバスに AT-SPI のバスを用意し、支援技術が使われている状態にする。
# `nix develop .#a11y` の中で、確かめるアプリを起動する前に実行する
set -euo pipefail

runtime_dir="${XDG_RUNTIME_DIR:-/tmp}"
pid_file="$runtime_dir/a11y-tree-bus.pid"
log="$runtime_dir/a11y-tree-bus.log"

has_owner() {
  dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus \
    org.freedesktop.DBus.NameHasOwner "string:$1" | grep -q "boolean true"
}

if ! dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus \
  org.freedesktop.DBus.GetId >/dev/null; then
  echo "no D-Bus session bus; run this inside a desktop session" >&2
  exit 1
fi

if ! has_owner org.a11y.Bus; then
  # デスクトップ環境が無いと GSettings を保存できず、IsEnabled が false に戻るため、メモリに保つ
  GSETTINGS_BACKEND=memory setsid "$AT_SPI_BUS_LAUNCHER" --launch-immediately >"$log" 2>&1 </dev/null &
  echo $! >"$pid_file"
  for _ in $(seq 50); do
    has_owner org.a11y.Bus && break
    sleep 0.1
  done
  has_owner org.a11y.Bus || {
    echo "failed to start the AT-SPI bus; see $log" >&2
    exit 1
  }
fi

# アプリは起動時に AT-SPI のバスが無いと、その後に用意されても登録しない。登録するかはこの値の変化を追って決める
dbus-send --session --print-reply --dest=org.a11y.Bus /org/a11y/bus \
  org.freedesktop.DBus.Properties.Set string:org.a11y.Status string:IsEnabled variant:boolean:true >/dev/null

# レジストリは最初に呼ばれたときに起動する。アプリより先に起動しておく
python3 -I -c 'import pyatspi; pyatspi.Registry.getDesktop(0)'
echo "AT-SPI bus is ready"
