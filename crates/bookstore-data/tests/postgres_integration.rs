use std::env;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use bookstore_app::{OrderLineCostSnapshot, ProfitReportRepository};
use bookstore_data::{PostgresProfitReportRepository, bootstrap_postgres};
use bookstore_domain::Money;
use tempfile::TempDir;
use tokio::time::sleep;

struct TestPostgres {
    _dir: TempDir,
    _socket_dir: TempDir,
    child: Child,
    database_url: String,
}

impl TestPostgres {
    async fn start() -> anyhow::Result<Self> {
        let dir = tempfile::tempdir()?;
        let socket_dir = tempfile::tempdir()?;
        let port = free_port()?;
        let user = "scriptorium";
        let password = "scriptorium";
        let database = "scriptorium";

        Command::new(postgres_executable("initdb")?)
            .arg("-D")
            .arg(dir.path())
            .arg("-U")
            .arg(user)
            .arg("--auth=trust")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("initdb failed"))?;

        let mut child = Command::new(postgres_executable("postgres")?)
            .arg("-D")
            .arg(dir.path())
            .arg("-k")
            .arg(socket_dir.path())
            .arg("-h")
            .arg("127.0.0.1")
            .arg("-p")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        wait_for_postgres(socket_dir.path(), port).await?;

        Command::new(postgres_executable("createdb")?)
            .arg("-h")
            .arg(socket_dir.path())
            .arg("-p")
            .arg(port.to_string())
            .arg("-U")
            .arg(user)
            .arg(database)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("createdb failed"))?;

        let database_url = format!("postgresql://{user}:{password}@127.0.0.1:{port}/{database}");

        if !child.try_wait()?.is_none() {
            anyhow::bail!("postgres exited before tests could connect");
        }

        Ok(Self { _dir: dir, _socket_dir: socket_dir, child, database_url })
    }
}

impl Drop for TestPostgres {
    fn drop(&mut self) {
        let _ =
            Command::new(postgres_executable("pg_ctl").unwrap_or_else(|_| PathBuf::from("pg_ctl")))
                .arg("-D")
                .arg(self._dir.path())
                .arg("stop")
                .arg("-m")
                .arg("fast")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn wait_for_postgres(socket_dir: &std::path::Path, port: u16) -> anyhow::Result<()> {
    for _ in 0..50 {
        let status = Command::new(postgres_executable("psql")?)
            .arg("-h")
            .arg(socket_dir)
            .arg("-p")
            .arg(port.to_string())
            .arg("-U")
            .arg("scriptorium")
            .arg("-d")
            .arg("postgres")
            .arg("-c")
            .arg("select 1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Ok(status) = status
            && status.success()
        {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("postgres did not become ready")
}

fn free_port() -> anyhow::Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

fn postgres_executable(name: &str) -> anyhow::Result<PathBuf> {
    if let Some(dir) = env::var_os("POSTGRES_BIN_DIR") {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    which_in_path(name)
        .ok_or_else(|| anyhow::anyhow!("install postgres tools or set POSTGRES_BIN_DIR"))
}

fn which_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path).map(|dir| dir.join(name)).find(|candidate| candidate.exists())
}

#[tokio::test]
async fn postgres_bootstrap_runs_migrations_and_reports_are_tenant_scoped() -> anyhow::Result<()> {
    let postgres = match TestPostgres::start().await {
        Ok(postgres) => postgres,
        Err(error) if error.to_string().contains("install postgres tools") => {
            eprintln!("skipping postgres integration test because postgres tools are unavailable");
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    let pool = bootstrap_postgres(&postgres.database_url).await?;

    let tenants: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants").fetch_one(&pool).await?;
    assert_eq!(tenants, 1);

    let repo = PostgresProfitReportRepository::new(pool.clone());

    repo.record(OrderLineCostSnapshot {
        tenant_id: "church-a".to_string(),
        revenue: Money::from_minor("AUD", 2000)?,
        cost: Money::from_minor("AUD", 1200)?,
    })
    .await?;

    repo.record(OrderLineCostSnapshot {
        tenant_id: "church-b".to_string(),
        revenue: Money::from_minor("AUD", 5000)?,
        cost: Money::from_minor("AUD", 3500)?,
    })
    .await?;

    let report = repo.profit_for_tenant("church-a").await?;
    assert_eq!(report.revenue.minor_units, 2000);
    assert_eq!(report.cogs.minor_units, 1200);
    assert_eq!(report.gross_profit.minor_units, 800);

    Ok(())
}
