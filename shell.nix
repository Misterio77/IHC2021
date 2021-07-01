{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    nodePackages.mermaid-cli
    postgresql
    pgformatter
    rust-analyzer
    rustfmt
  ];
}
