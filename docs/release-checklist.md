# Release Checklist

Use this checklist before tagging a Scriptorium release.

## Regression Gates

- Run `cargo test`
- Run `just bdd`
- Run `just browser`
- Run `just load`
- Run `cargo audit`
- Run `cargo deny check`

## Product Checks

- Verify POS login, basket, payment, and completion on desktop Chrome.
- Verify storefront catalog, cart, and checkout session creation.
- Verify admin dashboard login, report refresh, order actions, and intake save flow.
- Verify barcode intake fallback messaging when camera access is unavailable.

## Deployment Checks

- Confirm `DATABASE_URL` points at the intended environment.
- Confirm seeded admin bootstrap env vars are set for the target environment:
  - `SCRIPTORIUM_ADMIN_USERNAME`
  - `SCRIPTORIUM_ADMIN_PASSWORD`
  - `SCRIPTORIUM_DEFAULT_TENANT_ID`
- Review [deployment.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/deployment.md).
- Review [sqlite-backup-restore.md](/Users/jonochang/projects/lib/jc/scriptorium/docs/runbooks/sqlite-backup-restore.md).

## Signoff

- Confirm current UX review findings are resolved or explicitly deferred.
- Confirm palette/token review is complete.
- Record release notes in `CHANGELOG.md`.
- Tag and push only from a clean worktree.
