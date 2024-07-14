{
  lib,
  rustPlatform,
  clang,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage {
    inherit (cargoToml.package) version;
    pname = cargoToml.package.name;
    cargoLock.lockFile = ../Cargo.lock;
    src = lib.cleanSource ../.;
    buildInputs = [
      clang
    ];
  }
