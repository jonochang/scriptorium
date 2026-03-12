# Staging Deploy

This runbook covers the first manual Fly.io staging deploy with Neon as the Postgres provider.

## Assumptions

- Fly.io is the staging platform.
- [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml) is the baseline app config.
- The application image is built from [`Dockerfile`](/Users/jonochang/projects/lib/jc/scriptorium/Dockerfile).
- Neon provides the Postgres database and exposes it to the app through `DATABASE_URL`.

## Before First Deploy

1. Install and authenticate `flyctl`.
2. Review the app name and region in [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml).
3. Provision a staging Neon project and database.
4. Collect the connection string and any required SSL parameters.

## Required Secrets

Set these before the first deploy:

```sh
fly secrets set \
  DATABASE_URL='postgresql://scriptorium:secret@ep-example.ap-southeast-1.aws.neon.tech/scriptorium?sslmode=require' \
  SCRIPTORIUM_ADMIN_USERNAME='admin' \
  SCRIPTORIUM_ADMIN_PASSWORD='change-me' \
  SCRIPTORIUM_DEFAULT_TENANT_ID='church-a'
```

Optional secrets if object storage is enabled:

```sh
fly secrets set \
  SCRIPTORIUM_OBJECT_STORAGE_ENDPOINT='https://minio.example.com' \
  SCRIPTORIUM_OBJECT_STORAGE_REGION='us-east-1' \
  SCRIPTORIUM_OBJECT_STORAGE_ACCESS_KEY='scriptorium' \
  SCRIPTORIUM_OBJECT_STORAGE_SECRET_KEY='change-me' \
  SCRIPTORIUM_OBJECT_STORAGE_BUCKET='scriptorium-staging'
```

## First Deploy

Create the app if it does not exist yet:

```sh
fly apps create scriptorium-staging
```

Deploy:

```sh
fly deploy
```

The app boot path runs database migrations during startup based on `DATABASE_URL`.

For Neon, keep the provider-required SSL mode in the connection string.

## Verification

After deploy, verify:

```sh
fly status
curl -fsS https://scriptorium-staging.fly.dev/health
curl -fsS https://scriptorium-staging.fly.dev/ready
```

Then verify one product and admin flow against the staging URL.

## Low-Cost Settings

The baseline [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml) is set to:

- expose one HTTP service on port `8080`
- autostop idle machines
- autostart on the next request
- allow scale-to-zero with `min_machines_running = 0`

This keeps staging cost low at the expense of cold starts after idle periods.
