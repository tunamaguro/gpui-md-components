# `cargo xtask web` が使う wasm-bindgen の CLI。
#
# CLI の版は Cargo.lock の `wasm-bindgen` と完全に一致していなければならない。固定している nixpkgs に
# 同じ版が無いため、nixpkgs の `wasm-bindgen-cli_0_2_*` と同じ形でここで組み立てる。
# Cargo.lock の版を上げたら version と 2 つの hash を更新する（hash は lib.fakeHash にして
# `nix develop` のエラーに出る値を写す）。
{
  buildWasmBindgenCli,
  fetchCrate,
  rustPlatform,
}:

buildWasmBindgenCli rec {
  src = fetchCrate {
    pname = "wasm-bindgen-cli";
    version = "0.2.129";
    hash = "sha256-pcecKQd7E8Opw6bkFoE569epUi7gh5qpQF1e5PJY6V8=";
  };

  cargoDeps = rustPlatform.fetchCargoVendor {
    inherit src;
    inherit (src) pname version;
    hash = "sha256-vmUrWVU7kPJJxO5qIVeAkwQyWDELO1Z4Z5gitz2kco8=";
  };
}
