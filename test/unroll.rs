// RUN: mogglo-rust --dry-run --only-matching --where 'h_num = tonumber(h); return h_num ~= nil and h_num % 4 == 0' --replace 'for $i in 0..${{ string.format("%.0f", h / 4) }} { $b; $b; $b; $b; }' 'for $i in 0..$h { $b; }' %s 2>&1 | uncom | FileCheck %s

let mut x = 0;
// CHECK: for j in 0..6
for j in 0..24 {
    x += j.pow(2);
}
