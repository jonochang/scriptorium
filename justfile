default:
  @just --list

fmt:
  cargo fmt --all

lint:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
  cargo test --workspace

bdd:
  cargo test -p bookstore-web --test service_bdd

browser:
  cargo test -p bookstore-web --test browser_e2e

qa:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace
  cargo test -p bookstore-web --test service_bdd
  cargo test -p bookstore-web --test browser_e2e
  cargo deny check

audit:
  cargo audit

mutants:
  cargo mutants -f crates/bookstore-core/src/lib.rs
