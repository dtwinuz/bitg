{
  description = "bitg: Uzbek input method for the 2026 Latin alphabet (CLI, IBus engine, Fcitx5 addon)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;
      forAllSystems = lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          # Same recipe as nixpkgs, but from this checkout: Cargo.lock instead of a cargoHash
          bitg = (pkgs.callPackage ./packaging/nix/package.nix { }).overrideAttrs {
            version = (lib.importTOML ./Cargo.toml).workspace.package.version;
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./crates
                ./linux
                ./tools/winres
                ./web/icon.svg
              ];
            };
            cargoDeps = pkgs.rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };
          };
        in
        {
          inherit bitg;
          fcitx5-bitg = pkgs.callPackage ./packaging/nix/fcitx5-bitg.nix { inherit bitg; };
          default = bitg;
        }
      );

      # `nix flake check` paketlarni ham yig'adi (cargo test, installCheck bilan)
      checks = forAllSystems (system: {
        inherit (self.packages.${system}) bitg fcitx5-bitg;
      });
    };
}
