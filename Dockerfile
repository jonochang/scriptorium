FROM rust:1.91-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p bookstore-web

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 scriptorium

WORKDIR /app

COPY --from=builder /app/target/release/bookstore-web /usr/local/bin/bookstore-web

ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

USER scriptorium

EXPOSE 8080

CMD ["bookstore-web"]
