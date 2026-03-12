# Staging Plan

## Goal

Get Scriptorium deployed to a public staging environment so it can be used for product feedback and testing with minimal operational overhead.

This plan is focused on:

- getting to a usable staging site quickly
- keeping costs low
- preserving a path to a more durable production deployment later

This plan assumes the default staging target is:

- **Fly.io**

That is the default because staging usage is expected to be very low, and Fly.io is a simpler and cheaper first step than provisioning a dedicated OCI VM.

## Current Constraints

The current app and repo shape impose a few practical constraints:

1. The app is a single Rust web service.
2. The current persistence model is SQLite-first.
3. The web server currently binds to `127.0.0.1:8080`.
4. Deployment is currently documented as manual.
5. The current health endpoint is too weak for deployment gating.

These constraints mean the first staging environment should be treated as a single-instance deployment.

## Platform Assumption

The immediate staging target is:

- **Fly.io**
- one app
- one region
- one persistent volume
- one public staging hostname

OCI + NixOS remains a valid later path, but it is not the default path for first staging.

## Step-By-Step Plan

## Phase 1: Make The App Staging-Ready

### Step 1. Add configurable bind address and port

Change the web service so it can read host and port from environment variables.

Target outcome:

- local development still works easily
- staging can bind to the interface required by Fly.io

Suggested env vars:

- `HOST`
- `PORT`

### Step 2. Add a readiness endpoint

Add a `/ready` endpoint that verifies:

- the app has started
- the database is reachable

If object storage becomes required for staging flows, readiness can later verify that dependency too.

### Step 3. Clarify runtime env vars

Document and standardize the environment variables needed for staging:

- `DATABASE_URL`
- `RUST_LOG`
- `SCRIPTORIUM_ADMIN_USERNAME`
- `SCRIPTORIUM_ADMIN_PASSWORD`
- `SCRIPTORIUM_DEFAULT_TENANT_ID`
- object storage env vars if cover upload is enabled

### Step 4. Choose the first staging database strategy

There are two possible directions:

#### Option A: Keep SQLite for first staging

Use:

- one Fly app
- one persistent volume
- one app instance

Pros:

- fastest path from the current codebase
- no immediate database migration work

Cons:

- staging remains tied to one volume and one region
- rollback and backup stay somewhat manual
- future hosting flexibility is reduced

#### Option B: Add Postgres support before first staging

Use:

- hosted or managed Postgres
- app configured entirely through `DATABASE_URL`

Pros:

- easier future movement between Fly.io, Railway, Render, or OCI
- better long-term infrastructure flexibility
- no dependency on local persistent disk for the main database

Cons:

- more engineering scope before the first public staging deploy
- requires parallel migration and bootstrap work

### Recommendation for the first release

If the immediate goal is collecting feedback:

- keep **SQLite** for the first staging deployment

If the immediate goal is hosting flexibility:

- prioritize **Postgres support** before first launch

## Phase 2: Package The App For Fly.io

### Step 5. Add a deployable build path for `bookstore-web`

Package the web service so it can be built and deployed consistently to Fly.io.

Target outcome:

- a repeatable deploy artifact
- a predictable runtime command

Possible approaches:

- a `Dockerfile`
- Fly builder configuration
- optional Nix-based build later if desired

### Step 6. Add Fly.io configuration

Add the configuration files needed for staging deployment.

Likely files:

- `fly.toml`
- build and process configuration
- volume mount configuration

### Step 7. Define runtime process behavior

The deployed app should have:

- a stable startup command
- environment variable support
- an explicit data path for SQLite and backups
- visible logs in Fly.io

Suggested runtime data path:

- `/var/lib/scriptorium`

## Phase 3: Provision The Staging Environment

### Step 8. Create the Fly.io app

Create a dedicated staging app in Fly.io.

This should define:

- app name
- deployment region
- internal application port
- public HTTP and HTTPS access

### Step 9. Create a persistent volume

For SQLite staging, create a persistent Fly volume and mount it at the application data path.

Suggested use of that path:

- SQLite database file
- backups
- local runtime state that must survive deploys

### Step 10. Configure secrets and environment

Configure application secrets in Fly.io for:

- admin bootstrap credentials
- tenant bootstrap values
- `DATABASE_URL` from Neon
- object storage env vars if needed

### Step 11. Configure public staging access

Set up:

- Fly-managed public hostname
- optional custom staging subdomain
- TLS through Fly.io

This removes the need to run Caddy or Nginx for the first staging release.

## Phase 4: Automate Application Rollout

### Step 12. Add a deploy script or workflow

Add a simple deployment process that:

1. builds the artifact
2. verifies the volume path and env vars
3. deploys the app to Fly.io
4. runs smoke checks

If SQLite remains the first staging database, add a backup step before risky deploys.

### Step 13. Add CI workflow

The CI workflow should:

1. run tests
2. build the release artifact
3. deploy to Fly.io staging
4. verify the deployment

### Step 14. Keep app deploys simple

Avoid overcomplicating the first staging rollout.

For Fly.io staging:

- keep a single instance
- keep a single region
- avoid unnecessary platform abstractions

The goal is a cheap public environment, not a full production platform.

## Phase 5: Verify The Staging Environment

### Step 15. Define staging smoke tests

At minimum, verify:

- `/health`
- `/ready`
- storefront loads
- admin login works
- one key POS flow works

### Step 16. Run regression gates before promotion

Use existing repo checks where feasible:

- `cargo test`
- `just bdd`
- `just browser`
- `just load`

If browser or load checks are too heavy for every staging deploy, that should be an explicit policy choice rather than an accidental omission.

### Step 17. Capture a feedback loop

Once staging is public enough for testers:

- define who uses it
- define what flows should be reviewed
- capture issues in a lightweight feedback process

The deployment is only useful if it produces actionable feedback.

## Suggested Milestones

### Milestone 1: App readiness

Complete:

- configurable bind address
- readiness endpoint
- env var documentation

### Milestone 2: Deploy packaging

Complete:

- deployable web service build path
- Fly.io config
- data path and environment strategy documented

### Milestone 3: Fly.io environment

Complete:

- Fly app created
- persistent volume attached
- secrets configured

### Milestone 4: First successful deploy

Complete:

- CI-built artifact
- deploy process working
- public staging URL
- smoke tests passing

### Milestone 5: Feedback-ready staging

Complete:

- admin credentials set
- seeded tenant available
- testing checklist for reviewers

## Recommended Order Of Execution

If we want the fastest credible path to staging, the order should be:

1. make the app bind address configurable
2. add `/ready`
3. decide SQLite-first vs Postgres-for-staging
4. add deployable packaging for `bookstore-web`
5. add Fly.io configuration
6. provision the Fly app and persistent volume
7. configure secrets and runtime env vars
8. add deploy script and CI workflow
9. deploy and verify the first public staging build

## Risks And Mitigations

### Risk: SQLite becomes a staging bottleneck

Mitigation:

- keep staging single-instance
- use a persistent Fly volume
- back up before risky deploys
- treat Postgres as the next infrastructure milestone if staging use grows

### Risk: SQLite on Fly.io creates operational friction

Mitigation:

- keep the first deployment simple
- use one region
- use one instance
- move to Postgres if the volume model starts slowing us down

### Risk: We over-optimize staging before getting feedback

Mitigation:

- optimize for a working public staging site first
- defer OCI and NixOS host work until there is a stronger reason
- keep deploy automation simple

## Open Questions

These should be resolved early:

1. Do we want first staging on SQLite or do we want to invest in Postgres now?
2. Which Fly.io region should host the staging app?
3. Will object storage be needed in the first staging release?
4. Do we want OCI + NixOS later as a second-phase deployment target?

## Immediate Next Actions

The next concrete actions should be:

1. update the app to support configurable bind address and port
2. add a readiness endpoint
3. decide SQLite-first vs Postgres-for-staging
4. add packaging and config for Fly.io deployment
5. deploy the first low-cost public staging build
