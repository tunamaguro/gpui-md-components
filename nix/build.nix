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

  commonArgs = {
    src = craneLib.cleanCargoSource ../.;
    cargoLock = ../Cargo.lock;
    strictDeps = true;
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
        rustToolchain
        ;
      mainProgram = resolvedMainProgram;
    };

    meta.mainProgram = resolvedMainProgram;
  }
)
