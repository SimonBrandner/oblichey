{
  description = "A facial authentication software for Linux built in Rust.";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {
    nixpkgs,
    self,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    nixosModules.default = import ./nix/module.nix {inherit self system;};
    devShells.${system}.default = import ./nix/shell.nix {inherit pkgs;};
    packages.${system} = {
      default = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix {inherit pkgs;};
    };
  };
}
