{pkgs ? import <nixpkgs> {}, ...}:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang
    fontconfig
    pkg-config
    libxkbcommon
    libGL

    wayland

    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libX11

    vulkan-headers
    vulkan-loader
  ];
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  RUST_BACKTRACE = 1;
}
