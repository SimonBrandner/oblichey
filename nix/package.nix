{
  lib,
  rustPlatform,
  clang,
  pkgs ? import <nixpkgs> {},
}: let
  cliCargoTomlPath = ../crates/gday-cli/Cargo.toml;
  cliCargoTomlContent = builtins.fromTOML (builtins.readFile cliCargoTomlPath);
  workspaceCargoTomlContent = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  rustPlatform.buildRustPackage rec {
    src = lib.cleanSource ../.;
    cargoToml = cliCargoTomlPath;
    version = workspaceCargoTomlContent.workspace.package.version;
    pname = cliCargoTomlContent.package.name;
    cargoLock = {
      lockFile = ../Cargo.lock;
      outputHashes = {
        "burn-0.14.0" = "sha256-ChBlLKeq0WuINP+6mfZ0vFMYOKWnqT2dEMuM0UZJnbc=";
        "cubecl-0.1.1" = "sha256-tLNC2KRRYrRRKL9HkhTYHg8tvxkJDm9fM8GrSQWNXeM=";
      };
    };
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
      cp -r target/x86_64-unknown-linux-gnu/release/weights $out/bin/weights
    '';
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  }
