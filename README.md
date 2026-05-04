# Phosh First Boot

An app to setup your device. The app is run on first device boot via
[phrog](https://github.com/samcday/phrog).

## Dependencies

On a Debian based system run

```sh
sudo apt-get -y install build-essential
sudo apt-get -y build-dep .
```

For an explicit list of dependencies check the `Build-Depends` entry
in the [debian/control][] file.

## Building

We use the cargo and meson build systems for phosh-first-boot. The
quickest way to get going is to do the following. If you're only
intersted in building the app then just run:

```sh
cargo build
```

## Running

### Running from the source tree

You can run it from the source tree:

```sh
RUST_LOG=trace cargo run
```

## Getting in Touch

- Matrix: <https://matrix.to/#/#phosh:phosh.mobi>

[debian/control]: https://gitlab.gnome.org/World/Phosh/phosh-first-boot/-/blob/main/debian/control
