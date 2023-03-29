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

## Matching nodes with multiple children

Consider that there are several possible readings of the following pattern:
```
{ $f($x); $y + $z; }
```
It might only match blocks with exactly two statements, a call and an addition.
It might match a block that contains any number of statements, as long as there
is call followed *immediately* by an addition. In fact, Mogglo interprets this
pattern as matching any block that contains any number of statements, including
a function call that is followed *at some point* by an addition.

## Lua

Lua code is written between curly braces: `${{lua code goes here}}`.
See [the API reference](./api.md) for details.

## Speed

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
   ╭─[-:1:11]
   │
 1 │ let a = { let b = c; c };
   │           ─────┬────
   │                ╰────── Match
───╯
```
