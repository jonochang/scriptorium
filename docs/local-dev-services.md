# Local Dev Services

Use [`bin/dev-services`](/Users/jonochang/projects/lib/jc/scriptorium/bin/dev-services) to start local Postgres and MinIO for staging-oriented development work.

## Prerequisites

- `initdb`
- `pg_ctl`
- `psql`
- `createuser`
- `createdb`
- `minio`

On first run the helper:

- creates a local data directory under `~/.scriptorium` by default
- initializes a Postgres data directory if needed
- starts Postgres on `127.0.0.1:${POSTGRES_PORT:-5432}`
- creates the configured local database and user if missing
- starts MinIO on `http://127.0.0.1:${MINIO_PORT:-9000}`
- prints the environment variables needed by `bookstore-web`

## Commands

Start the service stack and leave it running:

```sh
just dev-services
```

Start with logs visible:

```sh
./bin/dev-services --verbose
```

Start services and then launch the app:

```sh
./bin/dev-services -- env \
  DATABASE_URL=postgresql://scriptorium:scriptorium@127.0.0.1:5432/scriptorium \
  SCRIPTORIUM_OBJECT_STORAGE_ENDPOINT=http://127.0.0.1:9000 \
  SCRIPTORIUM_OBJECT_STORAGE_REGION=us-east-1 \
  SCRIPTORIUM_OBJECT_STORAGE_ACCESS_KEY=scriptorium \
  SCRIPTORIUM_OBJECT_STORAGE_SECRET_KEY=scriptorium-dev \
  SCRIPTORIUM_OBJECT_STORAGE_BUCKET=scriptorium-dev \
  cargo run -p bookstore-web
```

## Configurable Environment

- `SCRIPTORIUM_DEV_DATA_DIR` defaults to `~/.scriptorium`
- `SCRIPTORIUM_POSTGRES_DATA_DIR` defaults to `$SCRIPTORIUM_DEV_DATA_DIR/postgres`
- `SCRIPTORIUM_MINIO_DATA_DIR` defaults to `$SCRIPTORIUM_DEV_DATA_DIR/minio`
- `SCRIPTORIUM_POSTGRES_SOCKET_DIR` defaults to `$SCRIPTORIUM_DEV_DATA_DIR/run`
- `POSTGRES_HOST` defaults to `127.0.0.1`
- `POSTGRES_PORT` defaults to `5432`
- `SCRIPTORIUM_POSTGRES_DB` defaults to `scriptorium`
- `SCRIPTORIUM_POSTGRES_USER` defaults to `scriptorium`
- `SCRIPTORIUM_POSTGRES_PASSWORD` defaults to `scriptorium`
- `MINIO_PORT` defaults to `9000`
- `MINIO_CONSOLE_PORT` defaults to `9001`
- `MINIO_ROOT_USER` defaults to `scriptorium`
- `MINIO_ROOT_PASSWORD` defaults to `scriptorium-dev`

The helper only starts supporting services. The application still needs its own env vars such as `HOST`, `PORT`, and admin bootstrap settings.
