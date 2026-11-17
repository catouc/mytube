let
  pkgs = import <nixpkgs> { config = {}; overlays = []; };
in
pkgs.mkShellNoCC {
  packages = with pkgs; [
    cargo
    clippy
    gcc
    rust-analyzer
    rustc
    rustfmt
    sqlite
    litecli
  ];
}
