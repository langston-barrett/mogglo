-- RUN: mogglo-haskell --dry-run --only-matching --replace 5 '${{ focus:kind() == "exp_literal" }}' %s 2>&1 | uncom | FileCheck %s

-- CHECK: x :: ()
x :: ()
-- CHECK: x = 5
x = ()
