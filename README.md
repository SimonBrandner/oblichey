# Oblichey

A facial authentication software for Linux built in Rust inspired by
[Howdy](https://github.com/boltgolt/howdy/).

## Notes/disclaimers

- You need a device with an infrared (IR) webcam.
- At the very least until https://github.com/SimonBrandner/oblichey/issues/6 is
  solved, this can be fooled with a printed photo.
- I am relatively new to Rust, Nix and PAM development, so use this at your own
  risk while it is in early stages of development.
- I am a student, and so my availability is somewhat limited depending on the
  time of year etc.
- Contributions are welcome!

## Installation

### Using a Nix flake

Add Oblichey to your flake inputs:

```nix
{
  inputs = {
    oblichey.url = "github:SimonBrandner/oblichey/main";
    ...
  };
  ...
}
```

Add the Oblichey NixOS module to your imports inside your system configuration:

```nix
{inputs, ...}: {
  imports = [
    inputs.oblichey.nixosModules.default
  ];
  ...
}
```

Add an `oblichey` entry to `programs`:

```nix
{inputs, ...}: {
  ...
  programs.oblichey = {
    enable = true;
    settings = {
      camera = {
        path = "/dev/video2"; # Path to your IR camera
      };
    };
    pamServices = ["su" "sudo"]; # List of PAM services (see `/etc/pam.d/`) in which a rule for Oblichey should be added
  };
```

### Other

Currently, NixOS is the only "officially" supported distribution, but it should
be possible to compile from source either by using Nix or without it and
installing manually. I am relatively open to contributions for other packaging
formats.

## Development

For development you are going to need Nix (the package manager not the OS)
which is either part of your NixOS install or can be downloaded
[here](https://nixos.org/download/#nix-install-linux) for other distros. You
are going to need to [enable
flakes](https://nixos.wiki/wiki/Flakes#Enable_flakes_permanently_in_NixOS).
Since the neural network models we are using are quite large, you're going to
need to get [Git LFS](https://git-lfs.com/) too.

Once you have those installed, you can clone the repo and enter the Nix
development environment.

```sh
git clone https://github.com/SimonBrandner/oblichey/
git lfs pull
nix develop
```

Now you can compile both `oblichey-cli` and `oblichey-pam-module` like so.

```sh
cargo build --release
```

The binary and library files can be found in the `target` directory. You can
also use `cargo run --release -p oblichey-cli` to build and run the cli.

To avoid having to type `nix develop` manually every time, you can use
[direnv](https://github.com/direnv/direnv/tree/master).

```sh
cp .envrc.sample .envrc
```

### Notes

- You need to compile with the `--release` flag, otherwise Oblichey is going to
  run super slow due to the neural network models not being optimized.
- If you want to develop on a machine that does not have an IR camera, you can
  do so by compiling with `--features "rgb-webcam"`. This is intended solely for
  development purposes.

## Etymology or where does the name come from?

The word _oblichey_ comes from the Czech word _obličej_ (which sounds sort of
like _oblichey_ when said aloud) meaning _face_.

## Software that was used to build Oblichey

- [The Rust programming language](https://www.rust-lang.org/)
- [Burn: a Rust deep learning framework](https://burn.dev/)
- [The Nix programming language and package manager](https://nixos.org/)
- [FaceONNX: a set of deep neural networks for face recognition and analytics](https://github.com/FaceONNX/FaceONNX)
- [`egui`: an easy-to-use GUI in pure Rust](https://github.com/emilk/egui)
- [`pam-rs`: Rust binding for PAM](https://github.com/anowell/pam-rs)
- [`libv4l-rs`: Rust bindings for `v4l`](https://github.com/raymanfx/libv4l-rs)
