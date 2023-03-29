# Mogglo

Mogglo is a multi-language AST-based code search and rewriting tool. Mogglo
supports embedding [Lua][lua] code in search patterns and replacements.

Mogglo focuses on the following goals:

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

Lua code is wrapped in braces. Lua can recursively match patterns with `rec`.
Here's a pattern to detect out-of-bounds array accesses:
```sh
mogglo-rust 'while $i <= $buf.len() { ${{ rec("$buf.get($i)") }} }' ./**/*.rs
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

See [the documentation](./doc) for more details!

[cargo]: https://doc.rust-lang.org/cargo/
[crates-io]: https://crates.io/
[releases]: https://github.com/langston-barrett/mogglo/releases
[rustup]: https://rustup.rs/
[unroll]: https://en.wikipedia.org/wiki/Loop_unrolling
