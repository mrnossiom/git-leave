{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, gitignore }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      nativeBuildInputs = with pkgs; [ rustToolchain pkg-config act ];
      buildInputs = with pkgs; [ openssl ];
    in
    with pkgs;
    {
      packages = rec {
        git-leave = callPackage ./package.nix { inherit gitignore; };
        default = git-leave;
      };
      apps = rec {
        git-leave = flake-utils.lib.mkApp { drv = self.packages.${system}.git-leave; };
        default = git-leave;
      };

      devShells.default = mkShell {
        inherit buildInputs nativeBuildInputs;
      };
    }
  );
}
