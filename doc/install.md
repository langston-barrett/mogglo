# Installation

## From a release

Statically-linked Linux binaries are available on the [releases page][releases].

You can download binaries with `curl` like so (replace `X.Y.Z` with a real
version number and `LANG` with a supported language):
```sh
curl -sSL https://github.com/langston-barrett/mogglo/releases/download/vX.Y.Z/mogglo-LANG -o mogglo-LANG
```

## From crates.io

You can build a released version from [crates.io][crates-io]. You'll need the
Rust compiler and the [Cargo][cargo] build tool. [rustup][rustup] makes it very
easy to obtain these. Then, to install Mogglo for the language `<LANG>`,
run:

```
cargo install mogglo-<LANG>
```

This will install binaries in `~/.cargo/bin` by default.

## From source

See the [developer's guide](dev.md).

[cargo]: https://doc.rust-lang.org/cargo/
[crates-io]: https://crates.io/
[releases]: https://github.com/langston-barrett/mogglo/releases
[rustup]: https://rustup.rs/
