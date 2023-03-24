// RUN: mogglo-rust --only-matching '${{rx("^a\\d", t)}}' %s 2>&1 | uncom | FileCheck %s

// CHECK: a1
// CHECK-NEXT: a2
let a1 = a2;

// CHECK-EMPTY:
// CHECK-NOT: {{.+}}
let a = b2;
let 0 = 1;
let Some(a) = a;
let Ok(a) = Ok(a);
