{
  perSystem =
    { config, ... }:
    let
      rustToolchain = config.packages.release.passthru.rustToolchain;
    in
    {
      treefmt.programs = {
        rustfmt = {
          enable = true;
          package = rustToolchain;
          includes = [ "*.rs" ];
        };

        mdformat = {
          enable = true;
          includes = [ "*.md" ];
          # 表（GFM）と skill の YAML frontmatter を壊さないためのプラグイン。
          plugins = ps: [
            ps.mdformat-gfm
            ps.mdformat-frontmatter
          ];
        };

        taplo = {
          enable = true;
          includes = [ "*.toml" ];
        };

        nixfmt = {
          enable = true;
        };
      };
    };
}
