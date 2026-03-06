# Deployment Notes

## MVP Shape

Scriptorium is deployed as:

- one Rust web service binary
- one SQLite database file
- optional reverse proxy for TLS and compression

## Environment

- `DATABASE_URL`
  - example: `sqlite://scriptorium.db?mode=rwc`
- `RUST_LOG`
  - example: `info,bookstore_web=debug`

## Local Start

```bash
cargo run -p bookstore-web
```

The service binds to `127.0.0.1:8080` by default.

## Production Checklist

1. Build the release binary:
   - `cargo build --release -p bookstore-web`
2. Provision a writable directory for the SQLite database and backups.
3. Set `DATABASE_URL` to that persistent path.
4. Put the service behind a reverse proxy that terminates TLS.
5. Enable process supervision with restart-on-failure.
6. Run the web BDD suite before promotion.
7. Capture a pre-deploy SQLite backup.

## Static Assets

Current frontend pages are server-rendered HTML with inline CSS/JS and external Google Fonts / HTMX / Preact CDNs where used. No separate asset pipeline is required for the MVP binary.

## Rollback

1. Stop the service.
2. Restore the last known-good binary.
3. Restore the SQLite file from the backup runbook if data rollback is required.
4. Restart and verify `/health` plus key BDD flows.
