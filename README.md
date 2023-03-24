# Mogglo

Mogglo is a multi-language AST-based code search and rewriting tool. Mogglo
supports embedding [Lua][lua] code in search patterns and replacements.

Mogglo focuses on the following features:

- *Minimal setup*: Mogglo will work right away on any codebase in a
  supported language.
- *Many languages*: 12 and counting!
- *High-quality grammars*: Mogglo is based on [tree-sitter][tree-sitter]
  grammars.
- *Lua*: Mogglo exposes a rich API to embedded Lua snippets.

[lua]: https://www.lua.org/
[tree-sitter]: https://tree-sitter.github.io/tree-sitter/

## Introduction

The following examples give a taste of Mogglo. Here's how to find pointless
assignments of an expression to itself:
```sh
mogglo-rust --detail 'let $x = $x;' ./**/*.rs
```
The `--detail` flag helps you understand why something matched, it produces
fancy output like:
```
   ╭─[./test/nonlinear.rs:4:1]
   │
 4 │ ╭─▶ let a =
   │ │       ┬
   │ │       ╰── $x
 5 │ ├─▶     a;
   │ │       ┬
   │ │       ╰──── $x
   │ │
   │ ╰──────────── let $x = $x;
   │
   │ Note: Multiple occurrences of $x were structurally equal
───╯
```

Lua code is wrapped in braces. The Lua variable `t` holds the text of the AST
node in question. Here's are two ways to find palindromic let-bound variables:
```sh
mogglo-rust 'let ${{ string.reverse(t) == t }} = $y;' ./**/*.rs
mogglo-rust 'let $x = $y;' --where 'string.reverse(x) == x' ./**/*.rs
```

Here's how to [unroll][unroll] a simple loop:
```sh
mogglo-rust \
  'for $i in 0..$h { $b; }' \
  --where 'h_num = tonumber(h); return h_num ~= nil and h_num % 4 == 0' \
  --replace 'for $i in 0..${{ string.format("%.0f", h / 4) }} { $b; $b; $b; $b; }' \
  ./*/**.rs
```
This transformation demonstrates the power of using Lua: it can't be done with
regular expression substitutions and would be very difficult with other codemod
tools.

Lua snippets can match and negate patterns, or even compose new patterns
dynamically! See [the guide](./doc/guide.md) for more detailed explanations,
examples, and features.

[unroll]: https://en.wikipedia.org/wiki/Loop_unrolling

## Supported languages

Mogglo currently ships pre-built executables for the following languages:

- [C](./crates/mogglo-c)
- [CSS](./crates/mogglo-css)
- [Java](./crates/mogglo-java)
- [JavaScript](./crates/mogglo-javascript)
- [Rust](./crates/mogglo-rust)
- [TypeScript](./crates/mogglo-typescript)
- [Swift](./crates/mogglo-swift)

Additionally, the following can be built from source or via Cargo/crates.io:

- [C++](./crates/mogglo-cpp)
- [Haskell](./crates/mogglo-haskell)
- [HTML](./crates/mogglo-html)
- [Python](./crates/mogglo-python)
- [Ruby](./crates/mogglo-ruby)

Languages are very easy to add, so file an issue or a PR if you want a new one!

## Comparison to related tools

Mogglo is not as polished as any of the tools mentioned in this section.

Mogglo is most similar to other multi-language code search and codemod tools.

- Mogglo is similar to [ast-grep][ast-grep], but supports more languages and
  allows embedding Lua in patterns.
- Mogglo is similar to [Comby][comby]. Comby uses lower-fidelity parsers, but
  is much more battle-tested and better documented. Mogglo also embeds Lua in
  patterns.
- Mogglo has less semantic understanding of code (e.g., name resolution) than
  [Semgrep][semgrep] or [CodeQL][codeql], but is much easier to set up.

There are many excellent language-specific code search and codemod tools; these
tend to be more polished but less general than Mogglo.

- [retrie][retrie]
- [weggli][weggli]

[ast-grep]: https://ast-grep.github.io/
[codeql]: https://codeql.github.com/
[comby]: https://comby.dev/
[retrie]: https://github.com/facebookincubator/retrie
[semgrep]: https://semgrep.dev/
[tree-sitter]: https://tree-sitter.github.io/tree-sitter/
[weggli]: https://github.com/weggli-rs/weggli

## Installation

### From a release

Statically-linked Linux binaries are available on the [releases page][releases].

### From crates.io

You can build a released version from [crates.io][crates-io]. You'll need the
Rust compiler and the [Cargo][cargo] build tool. [rustup][rustup] makes it very
easy to obtain these. Then, to install Mogglo for the language `<LANG>`,
run:

```
cargo install mogglo-<LANG>
```

This will install binaries in `~/.cargo/bin` by default.

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

## Test

Run end-to-end tests with `lit` and `FileCheck `. 

```sh
cargo build
lit --path=$PWD/test/bin --path=$PWD/target/debug test/
```

[cargo]: https://doc.rust-lang.org/cargo/
[crates-io]: https://crates.io/
[releases]: https://github.com/langston-barrett/mogglo/releases
[rustup]: https://rustup.rs/
