{
  description = "A Linux face authentication software built in Rust";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }: (
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        packages.default = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix {inherit pkgs;};
        devShells.default = import ./nix/shell.nix {inherit pkgs;};
      }
    )
  );
}
