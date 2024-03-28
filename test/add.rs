// RUN: mogglo-rust --dry-run --only-matching --replace '${{ tonumber(meta("x")) + tonumber(meta("y")) }}' '$x + $y' %s 2>&1 | uncom | FileCheck %s

// CHECK: let a = 5;
let a = 2 + 3;

let b = c;
