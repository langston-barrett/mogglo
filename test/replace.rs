// RUN: mogglo-rust --dry-run --only-matching --replace 'let $y = $x;' 'let $x = $y;' %s 2>&1 | uncom | FileCheck %s

// CHECK: let a = a;
let a = a;
// CHECK: let b = a;
let a = b;
