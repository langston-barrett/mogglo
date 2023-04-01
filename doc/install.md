# Installation

## Pre-compiled binaries

Pre-compiled binaries are available on the [releases page][releases].

### Fetching binaries with cURL

You can download binaries with `curl` like so (replace `X.Y.Z` with a real
version number, `LANG` with a supported language, and `TARGET` with your OS):
```sh
curl -sSL https://github.com/langston-barrett/mogglo/releases/download/vX.Y.Z/mogglo-LANG_TARGET -o mogglo-LANG
```

## Build from source

To install from source, you'll need to install Rust and [Cargo][cargo]. Follow
the instructions on the [Rust installation page][install-rust].

[install-rust]: https://www.rust-lang.org/tools/install

### From a release on crates.io

You can build a released version from [crates.io]. To install the latest
release of Mogglo for the language `<LANG>`, run:

```sh
cargo install mogglo-<LANG>
```

This will automatically download the source from [crates.io], build it, and
install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

### From the latest unreleased version on Github

To build and install the very latest unreleased version, run:

```sh
cargo install --git https://github.com/langston-barrett/mogglo.git mogglo-LANG
```

### From a local checkout

See the [developer's guide](dev.md).

### Uninstalling

To uninstall, run `cargo uninstall mogglo-<LANG>`.

[cargo]: https://doc.rust-lang.org/cargo/
[crates.io]: https://crates.io/
[releases]: https://github.com/langston-barrett/mogglo/releases
[rustup]: https://rustup.rs/
