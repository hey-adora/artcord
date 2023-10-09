let
  unstableTarball = fetchTarball https://github.com/NixOS/nixpkgs-channels/archive/nixos-unstable.tar.gz;
  pkgs = import <nixpkgs> {}; 
  unstable = import unstableTarball {};

  shell = pkgs.mkShell {
    buildInputs = [
        unstable.pkgconfig
        unstable.openssl
        unstable.openssl.dev
        unstable.cargo
        unstable.rustc
        unstable.rust-analyzer
    ];
  };  
in shell