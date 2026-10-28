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
      devShells.default = app.passthru.craneLib.devShell (
        {
          checks = config.checks;
          inputsFrom = [ app ];
          packages = [
            config.treefmt.build.wrapper
          ]
          # `vulkaninfo --summary` で Vulkan ドライバが見えているか確認できるようにする。
          ++ lib.optionals isLinux [ pkgs.vulkan-tools ];
        }
        // lib.optionalAttrs isLinux {
          # gpui が実行時に dlopen する wayland / vulkan / xkbcommon などを見つけられるようにする。
          LD_LIBRARY_PATH = lib.makeLibraryPath app.passthru.linuxRuntimeLibs;
          # ホストに Vulkan ドライバが無い環境（WSL や非 NixOS）でも gallery を動かすため、mesa の lavapipe
          # （CPU による描画）をシステムのドライバに「追加」する（置き換えはしない）。gpui は GPU を
          # ソフトウェア描画より優先するので、ホストに GPU ドライバがあればそちらが使われる。
          # dzn（WSL の D3D12 経由）は wgpu がアダプタとして認識しないため入れていない（zed@37faef81 時点）。
          VK_ADD_DRIVER_FILES = "${pkgs.mesa}/share/vulkan/icd.d/lvp_icd.${pkgs.stdenv.hostPlatform.uname.processor}.json";
          # ライブラリはフォントを同梱しない方針なので、システムにフォントが無い環境（WSL 等）でも gallery の
          # 文字を描けるよう開発環境にだけフォントを用意する。Noto Sans は gpui の既定フォールバックに含まれる。
          FONTCONFIG_FILE = pkgs.makeFontsConf {
            fontDirectories = [
              pkgs.noto-fonts
              pkgs.noto-fonts-cjk-sans
            ];
          };
        }
      );
    };
}
