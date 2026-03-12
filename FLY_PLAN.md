# Fly Plan

## Goal

Set up Scriptorium staging on Fly.io with Neon as the Postgres provider, then hand back the exact app and database values needed to finish deployment.

## Current Repo Status

Already in place:

- `Dockerfile` for `bookstore-web`
- `fly.toml` baseline config
- `HOST` / `PORT` runtime config
- `/health` and `/ready`
- SQLite or Postgres bootstrap via `DATABASE_URL`
- initial Postgres migrations
- local Postgres integration test
- `flyctl` and Postgres tools in the Nix dev shell

## What You Need To Do

### 1. Set up Fly.io

1. Sign up or log in to Fly.io.
2. In the repo shell, authenticate:

```sh
fly auth login
```

3. Confirm auth:

```sh
fly auth whoami
```

4. Choose the Fly app name.

Suggested:

- `scriptorium-staging`

5. Create the app:

```sh
fly apps create scriptorium-staging
```

6. Decide the region.

Suggested:

- `syd`

If you choose a different app name or region, send it back to me so I can align [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml).

### 2. Set up Neon

1. Sign up or log in to Neon.
2. Create a new Neon project for staging.
3. Pick a region reasonably close to the Fly region.
4. Keep the default branch as `main`.
5. Create or confirm the database name.

Suggested:

- database: `scriptorium`

6. Create or confirm the app role and password.

Suggested:

- username: `scriptorium`

7. Copy the Neon connection string for that database and role.

Expected shape:

```text
postgresql://scriptorium:PASSWORD@ep-...ap-southeast-1.aws.neon.tech/scriptorium?sslmode=require
```

Keep the SSL parameter in the final URL.

### 3. Set Fly secrets

Once you have the Neon connection string, set the required app secrets:

```sh
fly secrets set \
  DATABASE_URL='postgresql://scriptorium:PASSWORD@ep-...neon.tech/scriptorium?sslmode=require' \
  SCRIPTORIUM_ADMIN_USERNAME='admin' \
  SCRIPTORIUM_ADMIN_PASSWORD='CHOOSE_A_PASSWORD' \
  SCRIPTORIUM_DEFAULT_TENANT_ID='church-a' \
  -a scriptorium-staging
```

Optional later, if object storage is enabled:

```sh
fly secrets set \
  SCRIPTORIUM_OBJECT_STORAGE_ENDPOINT='...' \
  SCRIPTORIUM_OBJECT_STORAGE_REGION='us-east-1' \
  SCRIPTORIUM_OBJECT_STORAGE_ACCESS_KEY='...' \
  SCRIPTORIUM_OBJECT_STORAGE_SECRET_KEY='...' \
  SCRIPTORIUM_OBJECT_STORAGE_BUCKET='scriptorium-staging' \
  -a scriptorium-staging
```

## What To Send Back To Me

Reply with:

```text
Fly app name:
Fly region:
Neon project:
Neon host:
Neon database:
Neon username:
DATABASE_URL:
Object storage now or later:
```

## What I Will Do After That

Once you send those values, I can:

1. finalize `fly.toml`
2. verify the secret/config assumptions
3. guide or execute the first `fly deploy`
4. help diagnose any startup or health-check failures
5. prepare CI deployment automation

## Related Docs

- [docs/staging-deploy.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/staging-deploy.md)
- [docs/FLY_SETUP_PLAN.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/FLY_SETUP_PLAN.md)
- [docs/STAGING_PLAN.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/STAGING_PLAN.md)
