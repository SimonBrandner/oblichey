# Oblichey

A facial authentication software for Linux built in Rust inspired by
[Howdy](https://github.com/boltgolt/howdy/). I would highly recommend reading
about [how Oblichey works](/docs/how_does_it_work.md) before using it.

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

### NixOS

#### Using a Nix flake

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

And now you are good to go!

### Other distributions

Install the Nix package manager. It is not to be confused with NixOS. NixOS is
a whole distribution but we only need the package manager which can be
installed on any distribution. It is going to manage dependencies for us. It
can be found [here](https://nixos.org/download/#nix-install-linux).

Enable flakes by following the documentation
[here](https://nixos.wiki/wiki/Flakes#Enable_flakes_permanently_in_NixOS).

Clone the repo and build the flake.

```sh
git clone https://github.com/SimonBrandner/oblichey/
cd oblichey
nix build
```

Once the build process has finished, the resulting binary, library and weights
will be located in `./result/`. You can now add `oblichey-cli` to your `PATH`,
so that the PAM module can use it. It is recommended to move the build output
to a more proper place though.

Now, it is necessary to create a configuration file at `/etc/oblichey.toml`
with the path to your IR camera. It will usually be something like
`/dev/video2`.

```toml
[camera]
path="/path/to/camera"
```

The last step is to add a PAM rule for Oblichey. You can find the configuration
for PAM services at `/etc/pam.d/`. For example, one may want to use Oblichey to
authenticate when using `sudo`, so they would edit `/etc/pam.d/sudo` and add
the following line. It is important to note that this line should be placed
above the other `auth` rules, so that it takes precedence.

```
auth sufficient /path/to/libpam_oblichey.so
```

And now you are good to go!

## Usage

You can use `oblichey-cli help` to see the available commands. Everything
should be straightforward - you scan a new face, (use the test feature to check
everything is fine), and you are good to go.

## Development

Install the Nix package manager. It is not to be confused with NixOS. NixOS is
a whole distribution but we only need the package manager which can be
installed on any distribution. It is going to manage dependencies for us. It
can be found [here](https://nixos.org/download/#nix-install-linux).

Enable flakes by following the documentation
[here](https://nixos.wiki/wiki/Flakes#Enable_flakes_permanently_in_NixOS).

Clone the repo and enter the Nix development environment, this is going to
automagically install all the necessary dependencies.

```sh
git clone https://github.com/SimonBrandner/oblichey/
cd oblichey
nix develop
./scripts/unzip_models.sh
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
