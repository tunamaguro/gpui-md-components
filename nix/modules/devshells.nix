{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      app = config.packages.release;
      isLinux = pkgs.stdenv.hostPlatform.isLinux;
    in
    {
      devShells = {
        default = app.passthru.craneLib.devShell (
          {
            checks = config.checks;
            inputsFrom = [ app ];
            packages = [
              config.treefmt.build.wrapper
              # examples/gallery の Web 版のビルド (`examples/gallery/build-web.sh`) に使う。
              (pkgs.callPackage ../wasm-bindgen-cli.nix { })
              pkgs.binaryen
            ]
            # `vulkaninfo --summary` で Vulkan ドライバが見えているか確認できるようにする。
            ++ lib.optionals isLinux [ pkgs.vulkan-tools ];
          }
          // lib.optionalAttrs isLinux {
            # gpui が実行時に dlopen する wayland / vulkan / xkbcommon などを見つけられるようにする。
            LD_LIBRARY_PATH = lib.makeLibraryPath app.passthru.linuxRuntimeLibs;
            # ホストに Vulkan ドライバが無い環境（WSL や非 NixOS）でも GPUI を動かすため、mesa の lavapipe
            # （CPU による描画）をシステムのドライバに「追加」する（置き換えはしない）。gpui は GPU を
            # ソフトウェア描画より優先するので、ホストに GPU ドライバがあればそちらが使われる。
            # dzn（WSL の D3D12 経由）は wgpu がアダプタとして認識しないため入れていない（zed@37faef81 時点）。
            VK_ADD_DRIVER_FILES = "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.${pkgs.stdenv.hostPlatform.uname.processor}.json";
          }
        );
      }
      // lib.optionalAttrs isLinux {
        # 実機の a11y ツリーを AT-SPI から読む (`.agents/skills/a11y-tree`)。gallery は default の shell で動かす。
        a11y = pkgs.mkShell {
          packages = [
            (pkgs.python3.withPackages (ps: [
              ps.pyatspi
              ps.pygobject3
            ]))
            pkgs.at-spi2-core
            pkgs.dbus
            # start-bus.sh の setsid
            pkgs.util-linux
          ];
          AT_SPI_BUS_LAUNCHER = "${pkgs.at-spi2-core}/libexec/at-spi-bus-launcher";
          GI_TYPELIB_PATH = lib.makeSearchPath "lib/girepository-1.0" [
            pkgs.at-spi2-core
            pkgs.gobject-introspection
            pkgs.glib
          ];
        };
      };
    };
}
