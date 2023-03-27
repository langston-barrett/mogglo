// RUN: mogglo-rust --only-matching 'while $i <= $buf.len() { ${{rec("$buf.get($i)")}} }' %s 2>&1 | uncom | FileCheck %s

let buf = [0; 5];
let i = 0;
// CHECK: while i
while i <= buf.len() {
    j += buf.get(i);
}
