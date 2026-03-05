use anyhow::Context;
use async_trait::async_trait;
use bookstore_app::{OrderLineCostSnapshot, ProfitReport, ProfitReportRepository};
use bookstore_domain::Money;
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

pub async fn begin_transaction(
    pool: &SqlitePool,
) -> anyhow::Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
    pool.begin().await.context("failed to start sqlite transaction")
}

#[derive(Clone)]
pub struct SqliteProfitReportRepository {
    pool: SqlitePool,
}

impl SqliteProfitReportRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfitReportRepository for SqliteProfitReportRepository {
    async fn record(&self, snapshot: OrderLineCostSnapshot) -> anyhow::Result<()> {
        let mut tx = begin_transaction(&self.pool).await?;

        sqlx::query(
            "INSERT INTO order_line_snapshots (tenant_id, revenue_cents, cost_cents, currency) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(snapshot.tenant_id)
        .bind(snapshot.revenue.minor_units)
        .bind(snapshot.cost.minor_units)
        .bind(snapshot.revenue.currency)
        .execute(&mut *tx)
        .await
        .context("failed to insert order_line_snapshots row")?;

        tx.commit().await.context("failed to commit snapshot transaction")?;
        Ok(())
    }

    async fn profit_for_tenant(&self, tenant_id: &str) -> anyhow::Result<ProfitReport> {
        let row = sqlx::query_as::<_, (Option<i64>, Option<i64>, Option<String>)>(
            "SELECT SUM(revenue_cents), SUM(cost_cents), MIN(currency) FROM order_line_snapshots WHERE tenant_id = ?1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .context("failed to aggregate tenant profit report")?;

        let revenue_cents = row.0.unwrap_or(0);
        let cogs_cents = row.1.unwrap_or(0);
        let currency = row.2.unwrap_or_else(|| "AUD".to_string());

        let revenue = Money::from_minor(&currency, revenue_cents).context("build revenue money")?;
        let cogs = Money::from_minor(&currency, cogs_cents).context("build cogs money")?;
        let gross_profit = Money::from_minor(&currency, revenue_cents - cogs_cents)
            .context("build gross money")?;

        Ok(ProfitReport { revenue, cogs, gross_profit })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bookstore_app::ProfitReportRepository;
    use bookstore_domain::Money;

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

    #[tokio::test]
    async fn sqlite_profit_report_is_tenant_scoped() {
        let pool = bootstrap_sqlite("sqlite::memory:").await.expect("bootstrap should succeed");
        let repo = SqliteProfitReportRepository::new(pool);

        repo.record(OrderLineCostSnapshot {
            tenant_id: "church-a".to_string(),
            revenue: Money::from_minor("AUD", 2000).expect("valid money"),
            cost: Money::from_minor("AUD", 1200).expect("valid money"),
        })
        .await
        .expect("record snapshot");

        repo.record(OrderLineCostSnapshot {
            tenant_id: "church-b".to_string(),
            revenue: Money::from_minor("AUD", 5000).expect("valid money"),
            cost: Money::from_minor("AUD", 3500).expect("valid money"),
        })
        .await
        .expect("record snapshot");

        let report =
            repo.profit_for_tenant("church-a").await.expect("profit report should succeed");

        assert_eq!(report.revenue.minor_units, 2000);
        assert_eq!(report.cogs.minor_units, 1200);
        assert_eq!(report.gross_profit.minor_units, 800);
    }
}
