{
  lib,
  rustPlatform,
  clang,
  pkgs ? import <nixpkgs> {},
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage rec {
    inherit (cargoToml.package) version;
    pname = cargoToml.package.name;
    cargoLock = {
      lockFile = ../Cargo.lock;
      outputHashes = {
        "burn-0.14.0" = "sha256-5ssNI+QvMpEeU7JpK1KVtBpTgXKbaqTUYTBePTL+J24=";
        "cubecl-0.1.1" = "sha256-xgwXyHAKSxf4rTF/Svpb61IOvxQUiV9xmOXk5WHDdjY=";
      };
    };
    src = lib.cleanSource ../.;
    nativeBuildInputs = with pkgs; [
      clang
      fontconfig
      pkg-config
      libxkbcommon
      libGL
      cmake

      wayland

      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      xorg.libX11

      vulkan-headers
      vulkan-loader

      rustfmt
    ];
    buildInputs = with pkgs; [
      fontconfig
      vulkan-headers
      vulkan-loader
    ];
    preInstall = ''
      mkdir -p $out/bin
      cp -r target/release/weights $out/bin/weights
    '';
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  }
