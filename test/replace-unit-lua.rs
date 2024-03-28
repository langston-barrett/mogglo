// RUN: mogglo-rust --dry-run --only-matching --replace '5' '${{ focus:kind() == "unit_expression" }}'  %s 2>&1 | uncom | FileCheck %s

// CHECK: let a = 5;
let a = ();
// CHECK: let 5 = 5;
let () = ();
