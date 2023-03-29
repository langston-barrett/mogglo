// RUN: mogglo-rust --only-matching  '${{is_descendant_of(focus:kind(), "binary_expression")}}' %s 2>&1 | uncom | FileCheck %s

// CHECK: a + a
let a = a + a;

// CHECK-EMPTY:
// CHECK-NOT: {{.+}}
let a = b;
let 0 = 1;
let Some(a) = a;
