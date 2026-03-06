# SQLite Backup And Restore Runbook

## Scope

This runbook covers the MVP deployment shape described in the architecture docs: one Rust service process and one SQLite database file.

## Database Location

- Default local database URL: `sqlite://scriptorium.db?mode=rwc`
- Override with `DATABASE_URL`
- For backup planning, resolve the actual SQLite file path from the URL before running any commands

## Backup Procedure

1. Confirm the service instance and database path.
2. Create a timestamped backup directory:
   - `mkdir -p backups`
3. Use SQLite online backup mode when `sqlite3` is available:
   - `sqlite3 scriptorium.db ".backup backups/scriptorium-$(date +%Y%m%d-%H%M%S).db"`
4. If `sqlite3` is unavailable, stop writes briefly and copy the file atomically:
   - `cp scriptorium.db backups/scriptorium-$(date +%Y%m%d-%H%M%S).db`
5. Record the application revision alongside the backup:
   - `git rev-parse HEAD > backups/scriptorium-$(date +%Y%m%d-%H%M%S).sha`
6. Verify the backup file exists and is non-zero:
   - `ls -lh backups/`

## Restore Procedure

1. Stop the running service.
2. Keep the current damaged file for forensics:
   - `mv scriptorium.db scriptorium.db.before-restore`
3. Copy the chosen backup into place:
   - `cp backups/scriptorium-YYYYMMDD-HHMMSS.db scriptorium.db`
4. Start the service with the same `DATABASE_URL`.
5. Verify:
   - health endpoint returns `200`
   - key BDD smoke tests pass
   - expected recent data is present

## Operational Notes

- Keep at least one daily backup and one pre-release backup.
- Take a fresh backup before running destructive migrations or schema experiments.
- Store off-machine copies for any non-local environment.
- If WAL mode is enabled later, back up the main DB plus SQLite sidecar files consistently.
