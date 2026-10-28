{ inputs, ... }:
pkgs:
let
  # rust-toolchain.toml の設定に、ローカル開発用の components（CI では不要）を足す。
  toolchainFile = (builtins.fromTOML (builtins.readFile ../rust-toolchain.toml)).toolchain;
  devToolchain = toolchainFile // {
    components = (toolchainFile.components or [ ]) ++ [
      "rust-analyzer"
      "rust-src"
    ];
  };
  rustToolchainFor = p: (inputs.rust-overlay.lib.mkRustBin { } p).fromRustupToolchain devToolchain;
in
pkgs.callPackage ./build.nix {
  crane = inputs.crane.mkLib pkgs;
  inherit rustToolchainFor;
}
