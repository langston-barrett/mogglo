# Guide

Mogglo searches for *patterns* in code. Mogglo patterns consist of code
augmented with *metavariables* and embedded Lua code.

## Metavariables

Metavariables match nodes in the syntax tree. For example, the pattern
`let $x = ();` finds pointless assignments of the unit value `()`; the
metavariable `$x` matches any expression.

Multiple uses of the same metavariable imply equality. For example the pattern
`let $x = $x;` finds pointless assignments of an identifier to itself.

The special metavariable `$_` matches any syntax node, and multiple uses don't
imply equality. For example, `$_ == $_` finds an equality comparison between
any two expressions.

The special metavariable `$..` (read "ellipsis") can match any number of
sibling nodes in the AST. For example, here's how to find the main function:
```
fn main() $.. { $.. }
```

<!-- TODO: Make this true

The semantics of `$..` are as follows. Let `P` be any other part of a pattern,
e.g., a metavariable or piece of syntax. Then `$.. P` matches (in the language
of parsers, consumes) nodes until the first node that `P` matches. Ellipses
function like the non-greedy regular expression `.*?P`.

-->

## Lua

Lua code is written between curly braces: `${{lua code goes here}}`.

### Contexts

Lua code is evaluated in three different contexts:

- Patterns: Lua code embedded in `${{}}` when *matching* code
- Replacments: Lua code embedded in `${{}}` when *replacing* code
- Where clauses: Lua code passed to `--where`.

The value produced by code evaluated in patterns and where clauses is treated
as a boolean. If code in a pattern evaluates to `false` or `nil`, the node
is not matched; if the code evaluates to anything else, it is. For example,
`${{true}}` is equivalent to `$_`. Code evaluated in a replacement is treated
as a string.

The APIs available to code in patterns differ from those available to
replacements and where clauses. For example, code in patterns can write to
metavariables; code in replacements and where clauses can only read them.
In the rest of this guide, (P) denotes an API available only to patterns,
(A) denotes an API available in all contexts.

### Globals

(P) Lua code has access to a global variable `t` that holds the text of the
syntax node in question. For example, this pattern finds let-bound variables
that contain the letter `x`:
```
let ${{string.find(t, "x")}} = $_;
```

(A) All other metavariables are bound to globals; the pattern author is
responsible for not clobbering other important globals.

### Functions

Conventions:

- `Option<T>` means `T` or `nil`.
- If the return type is omitted, it is `nil`.

Functions:

- `bind(String)`, (P): Binds a metavariable to the current node

  - 1st argument: Metavariable name (without the `$`)
  - Example: `${{bind("x")}}` is equivalent to `$x` if `$x` is not yet bound
  - Note: This function can overwrite existing bindings; use with care

- `match(String) -> bool`, (P): Matches the current node against a pattern

  - 1st argument: A pattern
  - Returns: Whether or not the current node matches the pattern
  - Example: Patterns can be negated with `match`: `${{not match("<pattern>")}}`,
    e.g., `${{not match("${{false}}")}}` is equivalent to `${{true}}`.
  - Note: Metavariables in the argument pattern are inherited from the overall
    pattern; variables bound inside the argument pattern are not bound outside
    of it.

- `meta(String) -> Option<String>`, (A): Returns the binding for a metavariable

  - 1st argument: Metavariable name (without the `$`)
  - Returns: Value of the metavariable, or `nil` if there is none
  - Example: `${{meta("x") == t}}` is roughly equivalent to `$x` if `$x` is
    already bound (though not exactly: it matches *textually* instead of
    *structurally*)

- `rec(String) -> bool`, (P): Recursively matches all descendants of the current
  node against a pattern

  - 1st argument: A pattern
  - Returns: Whether or not some descendant of the current node matches the
    pattern
  - Example: `let x = ${{rec("$x")}} + $y;` matches `let a = (b + a) + c;`
  - Note: Metavariables in the argument pattern are inherited from the overall
    pattern; variables bound inside the argument pattern are not bound outside
    of it.

- `rx(String, String) -> bool` (A): Returns whether its first argument is a
  regular expression that matches its second.

  - 1st argument: Regular expression
  - 2nd argument: String to be matched
  - Returns: Whether the regex matched the string

### Nodes

In addition to the "textual" API given by the `t` variable, Lua code has
access to a "structured" API for AST nodes. The type of node objects is denoted
`Node`. The "current node" is stored in the global `focus`.

`Node` methods:

- `child(int) -> Option<Node>`:
  [Upstream docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html#method.child)
- `child_count() -> int`:
  [Upstream docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html#method.child_count)
- `kind() -> String`:
  [Upstream docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html#method.kind)
- `parent() -> Option<Node>`:
  [Upstream docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html#method.parent)

### State and evaluation order

When matching against a single node, Lua snippets in a pattern share the same
global state. Therefore, they can interact via global variables. For example,
the following pattern is functionally equivalent to
`let $_ = $_;`:
```
let ${{foo = "bar"; return true}} = ${{foo == "bar"}};
```
Evaluation order is depth-first, left-to-right. But be careful! It's hard to
tell when and how many times a given snippet will execute.

### Speed

Regular expressions are slow. Don't use them if string matching will do.

## Usage

By default, matches are non-recursive:
```sh
echo 'let a = { let b = c; c };' | mogglo-rust 'let $x = $y;' -
```
```
   ╭─[-:1:1]
   │
 1 │ let a = { let b = c; c };
   │ ────────────┬────────────
   │             ╰────────────── Match
───╯
```
The `--recursive` flag requests recursive matches, it will additionally print:
```
Match:
   ╭─[-:1:11]
   │
 1 │ let a = { let b = c; c };
   │           ─────┬────
   │                ╰────── Match
───╯
```
