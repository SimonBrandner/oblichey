{pkgs ? import <nixpkgs> {}, ...}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    clang
  ];
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}
