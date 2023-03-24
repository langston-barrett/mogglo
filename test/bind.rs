// RUN: mogglo-rust --only-matching 'let ${{bind("x"); return true}} = $x;' %s 2>&1 | FileCheck %s

// CHECK: let a = a;
let a = a;

// CHECK-EMPTY:
// CHECK-NOT: {{.+}}
let a = b;
let 0 = 1;
let Some(a) = a;
// COM: The LHS is a pattern, so is parsed differently
let Ok(a) = Ok(a);
