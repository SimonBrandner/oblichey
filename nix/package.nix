{
  lib,
  pkgs ? import <nixpkgs> {},
}: let
  cargoTomlPath = ../Cargo.toml;
  cargoTomlContent = builtins.fromTOML (builtins.readFile cargoTomlPath);
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = "gday";
    version = cargoTomlContent.workspace.package.version;
    src = lib.cleanSource ../.;
    cargoToml = cargoTomlPath;
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
      pam
    ];
    preInstall = ''
      mkdir -p $out/bin
      cp -r target/x86_64-unknown-linux-gnu/release/weights $out/bin/weights
    '';
    postFixup = ''
      patchelf --add-rpath ${with pkgs; lib.makeLibraryPath [libGL libxkbcommon wayland vulkan-loader vulkan-headers]}/lib $out/bin/gday-cli
    '';
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  }
