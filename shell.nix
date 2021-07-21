{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    httpie
    cargo
    nodePackages.mermaid-cli
    postgresql
    pgformatter
    rust-analyzer
    rustfmt
  ];
}
