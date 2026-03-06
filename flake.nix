{
  description = "scriptorium - church bookstore platform (CLI + web + mobile)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    untangle.url = "github:jonochang/untangle";
    crucible.url = "github:jonochang/crucible";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, untangle, crucible }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" ];
        };

        untangleBin = pkgs.writeShellScriptBin "untangle" ''
          exec nix run github:jonochang/untangle -- "$@"
        '';
        crucibleBin = pkgs.writeShellScriptBin "crucible" ''
          exec nix run github:jonochang/crucible -- "$@"
        '';
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "scriptorium-cli";
          version = "0.3.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        packages.crucible = crucibleBin;

        apps.crucible = flake-utils.lib.mkApp {
          drv = crucibleBin;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            untangleBin
            crucibleBin

            pkgs.pkg-config
            pkgs.cmake
            pkgs.openssl
            pkgs.libiconv

            pkgs.cargo-nextest
            pkgs.cargo-deny
            pkgs.cargo-llvm-cov
            pkgs.cargo-mutants
            pkgs.cargo-audit

            pkgs.just
            pkgs.git
          ];

          shellHook = ''
            echo "Scriptorium dev shell ready"
          '';
        };
      }
    );
}
