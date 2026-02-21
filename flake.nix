{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      gitignore,
    }:
    let
      inherit (nixpkgs.lib) genAttrs getExe;

      forAllSystems = genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllPkgs = function: forAllSystems (system: function pkgs.${system});

      mkApp = (
        program: {
          type = "app";
          inherit program;
        }
      );

      pkgs = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        }
      );
    in
    {
      formatter = forAllPkgs (pkgs: pkgs.nixfmt-tree);

      packages = forAllPkgs (pkgs: rec {
        default = git-leave;
        git-leave = pkgs.callPackage ./package.nix { inherit gitignore; };
      });
      apps = forAllSystems (system: rec {
        default = git-leave;
        git-leave = mkApp (getExe self.packages.${system}.git-leave);
      });

      devShells = forAllPkgs (
        pkgs:
        let
          file-rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rust-toolchain = file-rust-toolchain.override { extensions = [ "rust-analyzer" ]; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              pkg-config
              rust-toolchain
            ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );
    };
}
