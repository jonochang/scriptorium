# Fly Setup Plan

## Goal

Get Scriptorium running on Fly.io staging against Postgres with the minimum viable operational setup and a clear split between repo work and account-level actions.

## Current Repo Status

Completed by me:

- app binds via `HOST` and `PORT`
- app exposes `/health` and `/ready`
- `DATABASE_URL` selects SQLite or Postgres bootstrap
- initial Postgres migrations exist
- local Postgres + MinIO helper exists
- `Dockerfile` and `fly.toml` exist
- initial staging deploy runbook exists

Still pending in repo:

- verify Postgres integration tests against local Postgres in the Nix shell
- choose and document the real staging Postgres provider
- wire CI deployment to Fly
- run the first real Fly deploy

## Work Split

### Tasks for me

- keep the repo deployable
- add and maintain Postgres integration tests
- keep `fly.toml`, `Dockerfile`, and deploy docs aligned
- add CI steps once you have Fly credentials and the target app/database choice
- help validate the first deploy once the account-side prerequisites exist

### Tasks for you

- create or confirm the Fly.io account
- authenticate `flyctl`
- choose the final Fly app name
- use Neon as the staging Postgres provider
- provide or set the real staging secrets
- confirm the public staging hostname you want to use

## Step-By-Step

### Phase 1: Local Validation

Assigned to me:

- verify `nix develop` includes `flyctl`, `postgres`, and existing Rust tooling
- run the Postgres integration test locally
- keep the local startup helper working with the shell dependencies

Assigned to you:

- none unless the local shell is missing system access you expect

### Phase 2: Fly Account Setup

Assigned to you:

1. Sign up for Fly.io if you have not already.
2. Install or confirm access to `flyctl`.
3. Run `fly auth login`.
4. Confirm the organization/account that should own the staging app.
5. Confirm the target app name.

Suggested defaults:

- app name: `scriptorium-staging`
- region: `syd` if you want AU-adjacent staging, otherwise a cheaper or closer region to your testers

### Phase 3: Neon Setup

Assigned to you:

1. Create a Neon account or log in.
2. Create a Neon project for staging.
3. Keep the default `main` branch unless you already want branch-per-environment behavior.
4. Create or confirm a staging database name.
5. Create or confirm the staging Postgres role/password you want the app to use.
6. Record the final `DATABASE_URL`.

Assigned to me:

- keep docs and config aligned with Neon connection string requirements

### Phase 4: Fly App Provisioning

Assigned to you:

1. Create the Fly app:

```sh
fly apps create scriptorium-staging
```

2. Set required secrets:

```sh
fly secrets set \
  DATABASE_URL='postgresql://...neon.tech/...?...' \
  SCRIPTORIUM_ADMIN_USERNAME='admin' \
  SCRIPTORIUM_ADMIN_PASSWORD='...' \
  SCRIPTORIUM_DEFAULT_TENANT_ID='church-a'
```

3. Set object storage secrets if you want uploads enabled in staging.

Assigned to me:

- adjust [`fly.toml`](/Users/jonochang/projects/lib/jc/scriptorium/fly.toml) if your final app name or region changes
- keep the required secret list documented

### Phase 5: First Deploy

Assigned to me:

- make sure the image and app command are correct
- keep startup and readiness behavior compatible with Fly health checks

Assigned to you:

1. Run the first deploy:

```sh
fly deploy
```

2. Verify status:

```sh
fly status
curl -fsS https://scriptorium-staging.fly.dev/health
curl -fsS https://scriptorium-staging.fly.dev/ready
```

3. Share any startup or health-check failure output if deploy does not stabilize.

### Phase 6: Post-Deploy Verification

Assigned to me:

- help interpret Fly logs and application failures
- patch startup, health, or config issues found during rollout

Assigned to you:

1. Log into the admin UI with the configured bootstrap credentials.
2. Verify at least one storefront flow.
3. Verify at least one admin workflow.
4. Confirm whether cold-start behavior is acceptable with scale-to-zero enabled.

### Phase 7: CI Automation

Assigned to you:

- provide the Fly API token or set it in GitHub Actions secrets

Assigned to me:

1. Add a GitHub Actions staging deploy workflow.
2. Make CI run required tests before deploy.
3. Make CI deploy on your chosen trigger:
   - manual only
   - main branch only
   - tagged release only

## Immediate Next Actions

### Me

1. Verify the Postgres integration test passes in the Nix shell.
2. Verify `flyctl` is present in the dev shell.
3. Keep Fly docs and config aligned with the actual chosen app name/provider.

### You

1. Sign up for Fly.io or confirm the account is ready.
2. Decide the final Fly app name.
3. Create the Neon project, database, and app role.
4. Tell me those choices so I can finish the concrete deploy config and CI path.
