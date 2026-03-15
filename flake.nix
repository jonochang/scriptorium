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
        optionalChromium = if pkgs.stdenv.isLinux then [ pkgs.chromium ] else [ ];

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
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
          version = "0.4.4";
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
            pkgs.minio
            pkgs.postgresql
            pkgs.flyctl
          ] ++ optionalChromium ++ [

            pkgs.cargo-nextest
            pkgs.cargo-deny
            pkgs.cargo-llvm-cov
            pkgs.cargo-mutants
            pkgs.cargo-audit

            pkgs.trunk
            pkgs.wasm-bindgen-cli
            pkgs.binaryen

            pkgs.just
            pkgs.git
          ];

          shellHook = ''
            ${if pkgs.stdenv.isLinux then "export CHROME_EXECUTABLE=\"${pkgs.chromium}/bin/chromium\"" else ""}
            echo "Scriptorium dev shell ready"
          '';
        };
      }
    );
}
