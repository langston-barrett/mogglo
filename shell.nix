{ pkgs ? import <nixpkgs> { }
, unstable ? import <unstable> { }
}:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.lit
    pkgs.rust-analyzer
    pkgs.rustup
  ];
}
