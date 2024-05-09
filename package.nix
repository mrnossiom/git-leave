{ lib
, rustPlatform
, gitignore

, pkg-config
, openssl
}:

let
  inherit (gitignore.lib) gitignoreSource;

  src = gitignoreSource ./.;
  cargoTOML = lib.importTOML "${src}/Cargo.toml";
in
rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;

  inherit src;

  cargoLock = { lockFile = "${src}/Cargo.lock"; };

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  meta = {
    inherit (cargoTOML.package) description homepage license;
    maintainers = cargoTOML.package.authors;
    mainProgram = "git-leave";
  };
}
