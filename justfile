default:
  @just --list

fmt:
  cargo fmt --all

lint:
  cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
  cargo test --workspace

test-postgres:
  cargo test -p bookstore-data --test postgres_integration

bdd:
  cargo test -p bookstore-web --test service_bdd

browser:
  cargo test -p bookstore-web --test browser_e2e

load:
  cargo test -p bookstore-web --test service_load

dev-services:
  ./bin/dev-services

qa:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace
  cargo test -p bookstore-web --test service_bdd
  cargo test -p bookstore-web --test browser_e2e
  cargo test -p bookstore-web --test service_load
  cargo deny check

audit:
  cargo audit

mutants:
  cargo mutants -f crates/bookstore-core/src/lib.rs
