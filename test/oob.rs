// RUN: mogglo-rust --only-matching 'for $i in 0..=$buf.len() { ${{rec("$buf.get($i)") }} }' %s 2>&1 | uncom | FileCheck %s

let buf = [0; 5];
// CHECK: for i in
for i in 0..=buf.len() {
    j += buf.get(i);
}
