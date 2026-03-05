use anyhow::Context;
use sqlx::SqlitePool;

pub async fn bootstrap_sqlite(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect(database_url)
        .await
        .with_context(|| format!("failed to connect sqlite database at {database_url}"))?;

    let migrations_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations/sqlite");
    let migrator = sqlx::migrate::Migrator::new(migrations_path.as_path())
        .await
        .context("failed to load sqlite migration files")?;

    migrator.run(&pool).await.context("failed to run sqlite migrations")?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bootstrap_runs_initial_migration() {
        let pool = bootstrap_sqlite("sqlite::memory:").await.expect("bootstrap should succeed");
        let row_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tenants'",
        )
        .fetch_one(&pool)
        .await
        .expect("query table metadata");
        assert_eq!(row_count, 1);
    }
}
