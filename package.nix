{ lib, rustPlatform, gitignore, pkg-config, openssl }:

let

  inherit (gitignore.lib) gitignoreSource;

  src = gitignoreSource ./.;

  cargoTOML = lib.importTOML "${src}/Cargo.toml";
in
rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  inherit src;

  cargoLock = { lockFile = "${src}/Cargo.lock"; };

  meta = {
    inherit (cargoTOML.package) description homepage license;
    maintainers = cargoTOML.package.authors;
    mainProgram = "git-leave";
  };
}
