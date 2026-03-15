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

wasm:
  cargo build -p bookstore-cart-wasm --release --target wasm32-unknown-unknown
  mkdir -p static/wasm
  wasm-bindgen --target web --out-dir static/wasm --out-name bookstore-cart-wasm --no-typescript target/wasm32-unknown-unknown/release/bookstore_cart_wasm.wasm
  wasm-opt -Oz -o static/wasm/bookstore-cart-wasm_bg.wasm static/wasm/bookstore-cart-wasm_bg.wasm

wasm-dev:
  cargo build -p bookstore-cart-wasm --target wasm32-unknown-unknown
  mkdir -p static/wasm
  wasm-bindgen --target web --out-dir static/wasm --out-name bookstore-cart-wasm --no-typescript target/wasm32-unknown-unknown/debug/bookstore_cart_wasm.wasm

dev-services:
  ./bin/dev-services

qa:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  just wasm
  cargo test --workspace
  cargo test -p bookstore-web --test service_bdd
  cargo test -p bookstore-web --test browser_e2e
  cargo test -p bookstore-web --test service_load
  cargo deny check

audit:
  cargo audit

mutants:
  cargo mutants -f crates/bookstore-core/src/lib.rs
