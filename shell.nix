{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    sass
    nodePackages.prettier
    httpie
    cargo
    rustc
    nodePackages.mermaid-cli
    postgresql
    pgformatter
    rust-analyzer
    rustfmt
  ];
}
