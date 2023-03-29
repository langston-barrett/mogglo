# Developer's guide

## Build

To build from source, you'll need the Rust compiler and the [Cargo][cargo] build
tool. [rustup][rustup] makes it very easy to obtain these. Then, get the source:

```bash
git clone https://github.com/langston-barrett/mogglo
cd mogglo
```

Finally, build everything:

```bash
cargo build --release
```

You can find binaries in `target/release`. Run tests with `cargo test`.

[cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/

## Docs

HTML documentation can be built with `mdbook`:

```sh
cd doc
mdbook build
```

## Test

Run end-to-end tests with `lit` and `FileCheck `:

```sh
cargo build
lit --path=$PWD/test/bin --path=$PWD/target/debug test/
```
