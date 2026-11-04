#!/usr/bin/env bash
# gallery の Web 版をビルドし、配信する静的ファイル一式を examples/gallery/dist/ に出力する。
#
# wasm-bindgen と wasm-opt (binaryen) は `nix develop` で入る。wasm-bindgen の CLI は
# Cargo.lock の `wasm-bindgen` と同じ版でなければならない。
set -euo pipefail

gallery_dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$gallery_dir/../.." && pwd)
out="$gallery_dir/dist"
wasm="${CARGO_TARGET_DIR:-$root/target}/wasm32-unknown-unknown/release/gallery.wasm"

cargo build --manifest-path "$root/Cargo.toml" --locked --release --target wasm32-unknown-unknown -p gallery

rm -rf "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" "$wasm"
wasm-opt -Oz "$out/gallery_bg.wasm" -o "$out/gallery_bg.wasm"
cp "$gallery_dir/index.html" "$out/"
