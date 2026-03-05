# SQLite/Postgres Migration Parity Checklist

- Keep semantic version numbers aligned (`v1`, `v2`, ... ) between SQLite and Postgres tracks.
- For each SQLite migration, create a matching Postgres migration with equivalent constraints and indexes.
- Avoid SQLite-only table designs for business-critical data (no JSON blobs for core relations).
- Preserve tenant-scoped uniqueness in both engines (for example `(tenant_id, product_id)`).
- Verify all monetary columns use integer minor units in both engines.
- Verify transaction boundaries used by repositories are portable (`BEGIN`/`COMMIT` behavior matches).
- Validate report queries on both engines for tenant isolation and profit totals.
- Run parity test fixtures before introducing a new migration version.
