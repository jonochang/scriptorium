# Scriptorium

Church bookstore platform in Rust with four delivery targets:
- CLI app
- Web API
- iOS app (planned, Rust shared core)
- Android app (planned, Rust shared core)

## Workspace

- `bookstore-core`: domain and inventory logic
- `bookstore-cli`: terminal operations for catalog management
- `bookstore-web`: HTTP API and BDD tests (cucumber)
- `bookstore-mobile`: shared Rust API surface for mobile clients

## Development

```sh
nix develop
cargo generate-lockfile
just qa
```

## CLI

```sh
cargo run -p bookstore-cli -- list
cargo run -p bookstore-cli -- add --id bk-200 --title "Mere Christianity" --author "C.S. Lewis" --category Apologetics --price-cents 1799
```

## Web API

```sh
cargo run -p bookstore-web
# GET http://127.0.0.1:8080/health
# GET http://127.0.0.1:8080/books
```

## Quality Tooling

- `clippy`, `rustfmt`
- `cargo-nextest`
- `cargo-deny`
- `cargo-llvm-cov`
- `cargo-mutants`
- `cargo-audit`
- `cucumber` BDD tests
- `untangle` and `crucible` via flake inputs
