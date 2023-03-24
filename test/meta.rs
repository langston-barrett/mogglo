// RUN: mogglo-rust --only-matching 'let $x = ${{t == meta("x")}};' %s 2>&1 | uncom | FileCheck %s

// CHECK: let a = a;
let a = a;

// CHECK-EMPTY:
// CHECK-NOT: {{.+}}
let a = b;
let 0 = 1;
let Some(a) = a;
// COM: TODO let Ok(a) = Ok(a);
