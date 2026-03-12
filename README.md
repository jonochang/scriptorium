# Scriptorium

Church bookstore platform in Rust with modular-monolith boundaries and four delivery targets:
- CLI app
- Web API
- iOS app (planned, Rust shared core)
- Android app (planned, Rust shared core)

## Workspace

- `bookstore-domain` (in `crates/bookstore-core`): domain entities and invariants
- `bookstore-app`: application services/use-cases
- `bookstore-data`: persistence bootstrap and migrations
- `bookstore-cli`: terminal operations for catalog management
- `bookstore-web`: HTTP API, service BDD, and browser E2E tests
- `bookstore-mobile`: shared Rust API surface for mobile clients

## Development

```sh
nix develop
cargo generate-lockfile
just qa
```

Targeted web test commands:

```sh
just bdd
just browser
just load
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
# GET http://127.0.0.1:8080/ready
# GET http://127.0.0.1:8080/books
# GET http://127.0.0.1:8080/context (tenant/locale middleware context)
```

Runtime env vars:

- `HOST` defaults to `127.0.0.1`
- `PORT` defaults to `8080`
- `DATABASE_URL` defaults to `sqlite://scriptorium.db?mode=rwc`

For staging packaging and Fly.io rollout notes, see [docs/staging-deploy.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/staging-deploy.md).

## Quality Tooling

- `clippy`, `rustfmt`
- `cargo-nextest`
- `cargo-deny`
- `cargo-llvm-cov`
- `cargo-mutants`
- `cargo-audit`
- `cucumber` BDD tests
- `chromiumoxide` browser E2E tests
- `untangle` and `crucible` via flake inputs
