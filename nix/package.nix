{
  # Inputs
  fenix,

  # Functions
  makeRustPlatform,
  lib,
  system,
}:
let
  config = builtins.fromTOML (builtins.readFile ../Cargo.toml);

  toolchain = fenix.packages.${system}.fromToolchainFile {
    file = ../rust-toolchain.toml;
    sha256 = "sha256-3aoA7PuH09g8F+60uTUQhnHrb/ARDLueSOD08ZVsWe0=";
  };
  rustPlatform = makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
rustPlatform.buildRustPackage {
  pname = "unipac";
  version = config.workspace.package.version;
  src = ../.;

  cargoLock.lockFile = ../Cargo.lock;

  meta = with lib; {
    description = "An universal package manager";
    homepage = "https://github.com/polnio/unipac";
    license = licenses.mit;
  };
}
