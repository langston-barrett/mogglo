// RUN: mogglo-javascript --dry-run --only-matching --replace '${{ tonumber(meta("x")) + tonumber(meta("y")) }}' '$x + $y' %s 2>&1 | uncom | FileCheck %s

// CHECK: 5
2 + 3;
