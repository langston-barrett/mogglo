// RUN: mogglo-rust --only-matching 'let $x = ${{rec("let $y = $z;")}};' %s 2>&1 | FileCheck %s

// CHECK: let a = { let b = c; b };
let a = { let b = c; b };

// CHECK-EMPTY:
// CHECK-NOT: {{.+}}
let a = b;
let 0 = 1;
let Some(a) = a;
// COM: The LHS is a pattern, so is parsed differently
let Ok(a) = Ok(a);
