{
  lib,
  pkgs,
  crane,
  rustToolchainFor,
  cargoPackage ? null,
  profile ? "release",
  bin ? null,
  pname ? null,
  cargoBuildExtraArgs ? "",
  rustFlags ? null,
  doCheck ? true,
  dontStrip ? false,
  doInstallCargoArtifacts ? false,
  installCargoArtifactsMode ? "use-zstd",
}:
let
  craneLib = crane.overrideToolchain rustToolchainFor;
  rustToolchain = rustToolchainFor pkgs;

  # gpui の Linux バックエンド（wayland / x11 / wgpu / font-kit）が要求するシステムライブラリ。
  # wayland / vulkan / xkbcommon は実行時に dlopen されるので devShell で LD_LIBRARY_PATH にも載せる。
  linuxRuntimeLibs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (
    with pkgs;
    [
      fontconfig
      freetype
      libxkbcommon
      wayland
      vulkan-loader
      libx11
      libxcb
      libxcursor
      libxi
      libxrandr
    ]
  );

  commonArgs = {
    # Cargo のソースに加えて、`include_bytes!` / `include_str!` で埋め込むファイル
    # （md3-icons の SVG、xtask web の index.html）を含める。
    src = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        (craneLib.fileset.commonCargoSources ../.)
        ../crates/md3-icons/assets
        ../xtask/web
      ];
    };
    cargoLock = ../Cargo.lock;
    strictDeps = true;
    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = linuxRuntimeLibs;
    inherit doCheck dontStrip;

    env = {
      CARGO_PROFILE = profile;
    }

    // lib.optionalAttrs (rustFlags != null) {
      RUSTFLAGS = rustFlags;
    };
  };

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      cargoExtraArgs = lib.escapeShellArgs [
        "--locked"
        "--workspace"
      ];
      inherit cargoBuildExtraArgs;
      # Keep dev/test dependencies in the shared workspace artifact set even
      # when a final package explicitly disables its own checks.
      doCheck = true;
    }
  );

  packageCargoExtraArgs = lib.escapeShellArgs (
    [ "--locked" ]
    ++ lib.optionals (cargoPackage != null) [
      "-p"
      cargoPackage
    ]
    ++ lib.optionals (bin != null) [
      "--bin"
      bin
    ]
  );

  crateInfo = craneLib.crateNameFromCargoToml {
    inherit (commonArgs) src;
  };

  resolvedPname =
    if pname != null then
      pname
    else if cargoPackage != null then
      cargoPackage
    else
      crateInfo.pname;

  resolvedMainProgram =
    if bin != null then
      bin
    else if cargoPackage != null then
      cargoPackage
    else
      crateInfo.pname;

  packageArgs = commonArgs // {
    inherit
      cargoArtifacts
      cargoBuildExtraArgs
      doInstallCargoArtifacts
      installCargoArtifactsMode
      ;
    pname = resolvedPname;
    cargoExtraArgs = packageCargoExtraArgs;
  };
in
craneLib.buildPackage (
  packageArgs
  // {
    passthru = {
      inherit
        cargoArtifacts
        commonArgs
        craneLib
        linuxRuntimeLibs
        rustToolchain
        ;
      mainProgram = resolvedMainProgram;
    };

    meta.mainProgram = resolvedMainProgram;
  }
)
