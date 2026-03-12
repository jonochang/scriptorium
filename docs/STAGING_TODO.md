# Staging Todo

## Goal

Prepare Scriptorium for a public staging deployment on Fly.io with **Postgres as the staging database**.

This todo is intentionally detailed and execution-oriented. The target is not just "a plan", but a concrete list of work items that gets the app from local-only to a usable staging environment for feedback and testing.

## Working Assumptions

- staging platform: **Fly.io**
- staging database: **Postgres**
- local development can continue using **SQLite**
- staging should be low-cost and low-traffic
- the first public deployment can remain single-instance
- OCI + NixOS is deferred and not required for first staging

## Phase 1: Application Runtime Readiness

### Networking

- [x] Make the web server bind address configurable.
- [x] Make the web server port configurable.
- [x] Keep sensible defaults for local development.
- [x] Confirm the app can bind to `0.0.0.0` for container deployment.
- [x] Update runtime docs to describe the new bind env vars.

### Health and readiness

- [x] Add a `/ready` endpoint.
- [x] Make `/ready` verify database connectivity.
- [ ] Decide whether `/ready` should also verify optional object storage when enabled.
- [x] Keep `/health` as a lightweight liveness endpoint.
- [x] Document the intended meaning of `/health` vs `/ready`.

### Runtime configuration

- [x] Audit all environment variables currently used by the app.
- [x] Create a documented staging env var list.
- [ ] Confirm bootstrap admin settings required for staging.
- [ ] Confirm tenant bootstrap settings required for staging.
- [ ] Identify any env vars that should be optional vs required.
- [x] Add a sample staging env file or env var reference doc.

## Phase 2: Postgres Support

### Local startup workflow

- [x] Add a local startup script or helper binary that starts Postgres and MinIO for development.
- [x] Use `/Users/jonochang/projects/product/highlighter/web/bin/highlighter.go` as the reference pattern for process orchestration.
- [x] Decide whether the local startup helper should be written in Rust, Go, or shell.
- [x] Ensure the helper creates local data directories if they do not exist.
- [x] Ensure the helper can initialize a local Postgres data directory on first run.
- [x] Ensure the helper can start Postgres on a configurable local port.
- [x] Ensure the helper can start MinIO on a configurable local port.
- [x] Ensure the helper exports or prints the correct local `DATABASE_URL`.
- [x] Ensure the helper exports or prints the correct local object storage configuration.
- [x] Ensure the helper shuts down child processes cleanly on exit or interrupt.
- [x] Add a verbose mode for local debugging.
- [x] Decide where local runtime data should live, for example under a project-local directory or `~/.scriptorium`.
- [x] Document local prerequisites such as `initdb`, `pg_ctl`, and `minio`.
- [x] Add a documented command for starting the full local stack.

### Database bootstrap

- [x] Introduce a Postgres bootstrap path alongside the existing SQLite bootstrap.
- [x] Make database initialization choose backend based on `DATABASE_URL`.
- [x] Keep SQLite support for local development and tests where useful.
- [x] Ensure startup fails clearly if the configured database cannot be reached.

### Migrations

- [x] Create a Postgres migrations directory parallel to the SQLite migrations.
- [x] Port the existing SQLite schema to Postgres migrations.
- [ ] Review SQL types and constraints for Postgres compatibility.
- [ ] Review auto-increment, default values, timestamps, booleans, and text handling.
- [ ] Review indexes and unique constraints for tenant-scoped correctness.
- [ ] Ensure migrations can run cleanly on an empty Postgres database.

### Repository and query compatibility

- [ ] Audit `bookstore-data` for SQLite-specific SQL or assumptions.
- [ ] Replace SQLite-specific SQL where needed with portable or backend-specific logic.
- [ ] Review transaction handling for Postgres compatibility.
- [ ] Confirm tenant-scoped queries behave identically on Postgres.
- [ ] Confirm reporting queries behave identically on Postgres.
- [ ] Confirm error handling surfaces useful DB connection and migration failures.

### Data model validation

- [ ] Verify that the existing schema assumptions map correctly to Postgres.
- [ ] Review text collation and case-sensitivity assumptions.
- [ ] Review numeric and money-related storage assumptions.
- [ ] Review datetime handling and timezone assumptions.
- [ ] Review any SQLite-specific default behavior that could change under Postgres.

### Test coverage

- [x] Add tests for Postgres bootstrap.
- [x] Add tests that run core repository behavior against Postgres.
- [ ] Add tests that confirm tenant isolation under Postgres.
- [ ] Add tests that confirm reporting behavior under Postgres.
- [ ] Decide whether CI should run both SQLite and Postgres test suites or a reduced Postgres subset.

## Phase 3: Staging Deployment Packaging

### Build and release path

- [ ] Decide how the app will be built for Fly.io deployment.
- [x] Add a `Dockerfile` or equivalent Fly-compatible build path.
- [x] Ensure the release build targets `bookstore-web`, not only the CLI.
- [x] Verify the runtime image contains everything needed for startup.
- [x] Verify the startup command is explicit and stable.

### App configuration for Fly.io

- [x] Add `fly.toml`.
- [x] Set the internal application port correctly.
- [x] Configure HTTP and HTTPS handling.
- [x] Configure autostop/autostart settings for low-cost staging.
- [ ] Decide whether staging should scale to zero.
- [ ] Choose a Fly region for staging.
- [ ] Document the Fly app name and naming convention.

### Secrets and environment in Fly.io

- [ ] Define the full list of Fly secrets required.
- [ ] Add `DATABASE_URL` for Postgres staging.
- [ ] Add admin bootstrap credentials as Fly secrets.
- [ ] Add tenant bootstrap values as Fly secrets.
- [ ] Add object storage secrets if needed.
- [ ] Decide which env vars belong in `fly.toml` vs Fly secrets.

## Phase 4: Postgres Hosting Decision

### Choose the Postgres provider

- [ ] Choose whether staging Postgres lives on Fly.io or an external managed provider.
- [ ] Compare the cost of Fly Postgres vs Neon vs Supabase vs Railway Postgres.
- [ ] Choose the cheapest acceptable option for a low-traffic staging environment.
- [ ] Confirm connection limits, region alignment, and cold-start behavior.
- [ ] Confirm backup and restore options for the chosen Postgres provider.

### Database operations

- [ ] Define how the staging database is created.
- [ ] Define how migrations are applied during first deploy.
- [ ] Define how migrations are applied on later deploys.
- [ ] Decide whether migration execution happens in app startup, CI, or a separate release command.
- [ ] Decide how staging database resets should work.

## Phase 5: Deployment Automation

### Local deploy workflow

- [x] Document how to deploy staging manually the first time.
- [x] Document how to set Fly secrets.
- [ ] Document how to provision the Postgres database.
- [x] Document how to run migrations before the first successful boot.

### CI deploy workflow

- [ ] Add a CI workflow for staging deploys.
- [ ] Make CI run the required test suite before deployment.
- [ ] Make CI build the deploy artifact.
- [ ] Make CI authenticate to Fly.io securely.
- [ ] Make CI deploy to the staging app.
- [ ] Make CI verify the deployment after rollout.

### Release safety

- [ ] Decide whether staging deploys happen on every merge or only manually.
- [ ] Decide whether database migrations block deployment on failure.
- [ ] Add smoke checks after deploy.
- [ ] Add rollback guidance for application deploy failures.
- [ ] Add rollback guidance for schema or migration failures.

## Phase 6: Verification And Testing

### Smoke tests

- [ ] Define the required smoke-test endpoints and flows.
- [ ] Verify `/health`.
- [ ] Verify `/ready`.
- [ ] Verify storefront page load.
- [ ] Verify admin login.
- [ ] Verify one admin workflow.
- [ ] Verify one POS workflow.

### Regression gates

- [ ] Decide which existing checks are mandatory before staging deploy:
  - `cargo test`
  - `just bdd`
  - `just browser`
  - `just load`
- [ ] Decide whether Postgres-specific tests must run before every staging deploy.
- [ ] Decide whether heavy browser/load checks should run on every deploy or on a schedule.

### Staging usability

- [ ] Confirm seeded or bootstrap data is sufficient for review.
- [ ] Confirm tester login path is documented.
- [ ] Confirm staging URL is stable and shareable.
- [ ] Confirm feedback collection expectations are clear.

## Phase 7: Documentation

### Deployment docs

- [x] Update deployment docs to describe Fly.io as the staging target.
- [x] Update deployment docs to describe Postgres as the staging database.
- [x] Update docs to distinguish local SQLite from staging Postgres.
- [x] Add docs for Fly.io app creation and environment configuration.
- [x] Add docs for staging rollout and verification.

### Operational docs

- [ ] Add a runbook for failed staging deploys.
- [ ] Add a runbook for Postgres connection or migration failures.
- [ ] Add a runbook for resetting or reseeding staging.
- [ ] Add a runbook for rotating staging secrets.

## Phase 8: Nice-To-Have Follow-Up

- [ ] Add structured readiness reporting with failure reasons.
- [ ] Add a dedicated staging seed command or fixture loader.
- [ ] Add a staging banner or visible environment marker in the UI.
- [ ] Add staging-only observability or tracing configuration.
- [ ] Revisit OCI + NixOS only after staging feedback proves the need for a more host-managed environment.

## Recommended Order Of Execution

The practical order should be:

1. make host and port configurable
2. add `/ready`
3. add the local startup helper for Postgres and MinIO
4. implement Postgres bootstrap selection from `DATABASE_URL`
5. add Postgres migrations
6. make core repository paths work on Postgres
7. add Postgres-focused tests
8. add Fly.io deploy packaging and `fly.toml`
9. choose the Postgres provider
10. provision the staging database
11. provision the Fly app
12. configure secrets and env vars
13. perform the first manual staging deploy
14. run smoke tests
15. add CI automation
16. update docs and runbooks

## Minimum Viable Staging Checklist

This is the shortest acceptable path to a usable staging deployment:

- [ ] app binds to configurable host and port
- [ ] app exposes `/ready`
- [ ] app can start against Postgres
- [ ] Postgres migrations run successfully
- [ ] Fly app exists
- [ ] staging Postgres exists
- [ ] secrets are configured
- [ ] first deploy succeeds
- [ ] smoke checks pass
- [ ] staging URL is shareable with testers

## Deferred For Later

These items should not block the first staging deployment unless they become necessary:

- [ ] OCI + NixOS deployment work
- [ ] production-grade HA or multi-region work
- [ ] large-scale observability setup
- [ ] full environment parity with future production
- [ ] replacing all SQLite usage in local development
