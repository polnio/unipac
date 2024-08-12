{ rustPlatform, lib }:
let
  config = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = "unipac";
  version = config.workspace.package.version;
  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = with lib; {
    description = "An universal package manager";
    homepage = "https://github.com/polnio/unipac";
    license = licenses.mit;
  };
}
