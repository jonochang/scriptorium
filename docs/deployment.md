# Deployment Notes

## Runtime Shapes

Current local development shape:

- one Rust web service binary
- one SQLite database file by default

Current staging target:

- one Fly.io app
- one Rust web service container
- one Postgres database referenced through `DATABASE_URL`
- Fly-managed TLS and public ingress

## Environment

- `HOST`
  - example: `0.0.0.0`
- `PORT`
  - example: `8080`
- `DATABASE_URL`
  - example: `sqlite://scriptorium.db?mode=rwc`
  - Postgres example: `postgresql://scriptorium:secret@127.0.0.1:5432/scriptorium`
- `RUST_LOG`
  - example: `info,bookstore_web=debug`
- `SCRIPTORIUM_ADMIN_USERNAME`
- `SCRIPTORIUM_ADMIN_PASSWORD`
- `SCRIPTORIUM_DEFAULT_TENANT_ID`
- `SCRIPTORIUM_ISBN_LOOKUP_BASE_URL`
- `SCRIPTORIUM_OBJECT_STORAGE_ENDPOINT`
- `SCRIPTORIUM_OBJECT_STORAGE_REGION`
- `SCRIPTORIUM_OBJECT_STORAGE_ACCESS_KEY`
- `SCRIPTORIUM_OBJECT_STORAGE_SECRET_KEY`
- `SCRIPTORIUM_OBJECT_STORAGE_BUCKET`

## Local Start

```bash
cargo run -p bookstore-web
```

The service binds to `127.0.0.1:8080` by default. Set `HOST=0.0.0.0` for container or Fly.io-style deployment.

`/health` is a liveness check. `/ready` is a readiness check and returns `503 Service Unavailable` if the configured database connection is not available.

`DATABASE_URL` currently supports SQLite and Postgres bootstrap paths.

## Staging Start

The current staging target is Fly.io with Postgres.

Baseline staging assets:

- [`Dockerfile`](/Users/jonochang/projects/lib/jc/scriptorium/Dockerfile)
- [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml)
- [`docs/staging-deploy.md`](/Users/jonochang/projects/lib/jc/scriptorium/docs/staging-deploy.md)

## Production Checklist

1. Build the release binary:
   - `cargo build --release -p bookstore-web`
2. For local or host-managed deployment, provision a writable directory if SQLite is still in use.
3. Set `DATABASE_URL` for the intended environment.
4. For Fly staging, set secrets and deploy with `flyctl`.
5. For non-Fly deployment, put the service behind a reverse proxy that terminates TLS.
6. Enable process supervision or platform-managed restarts.
7. Run the web BDD suite before promotion.
8. Capture a pre-deploy backup for the active database backend.

## Static Assets

Current frontend pages are server-rendered HTML with inline CSS/JS and external Google Fonts / HTMX / Preact CDNs where used. No separate asset pipeline is required for the MVP binary.

## Rollback

1. Stop the service.
2. Restore the last known-good binary.
3. Restore the active database backend from backup if data rollback is required.
4. Restart and verify `/health` plus key BDD flows.
