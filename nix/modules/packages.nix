{ inputs, ... }:
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      mkPackage = import ../toolchain.nix { inherit inputs; };
      app = mkPackage pkgs;
      release = app.override {
        profile = "release";
        dontStrip = false;
      };

      debug = app.override {
        profile = "dev";
        dontStrip = true;
      };

      flamegraph = app.override {
        profile = "flamegraph";
        dontStrip = true;
        doInstallCargoArtifacts = true;
        installCargoArtifactsMode = "use-symlink";
        # perf needs --no-rosegment for accurate stacks with lld/mold on Linux.
        rustFlags =
          "-C force-frame-pointers=yes"
          + lib.optionalString pkgs.stdenv.hostPlatform.isLinux " -C link-arg=-Wl,--no-rosegment";
      };

      flamegraphRunner = pkgs.writeShellApplication {
        name = "profile-flamegraph";
        runtimeInputs = [
          flamegraph.passthru.rustToolchain
          pkgs.stdenv.cc
          pkgs.cargo-flamegraph
          pkgs.coreutils
          pkgs.findutils
        ];
        runtimeEnv = flamegraph.passthru.commonArgs.env // {
          CARGO_BUILD_INCREMENTAL = "false";
          cargoArtifacts = "${flamegraph}";
          doNotLinkInheritedArtifacts = "1";
        };
        text = ''
          export CARGO_TARGET_DIR="''${CARGO_TARGET_DIR:-$PWD/target/nix-flamegraph}/${builtins.baseNameOf flamegraph}"
          if [[ ! -f "$CARGO_TARGET_DIR/.nix-artifacts-ready" ]]; then
            # shellcheck disable=SC1091
            source ${flamegraph.passthru.craneLib.inheritCargoArtifactsHook}/nix-support/setup-hook
            inheritCargoArtifacts
            touch "$CARGO_TARGET_DIR/.nix-artifacts-ready"
          fi
          exec cargo flamegraph --profile "$CARGO_PROFILE" "$@"
        '';
      };

      mkFlakeApp = package: {
        type = "app";
        program = lib.getExe package;
      };
    in
    {
      packages = {
        default = release;
        inherit debug release flamegraph;
      };

      apps = {
        default = mkFlakeApp config.packages.debug;
        debug = mkFlakeApp config.packages.debug;
        release = mkFlakeApp config.packages.release;
        flamegraph = mkFlakeApp flamegraphRunner;
      };
    };
}
