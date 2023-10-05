{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {

  buildInputs = [
    pkgs.rustup
    pkgs.pkgconfig
    pkgs.openssl
    pkgs.openssl.dev
  ];
}