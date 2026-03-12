use bookstore_app::{AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_data::{DatabasePool, bootstrap_sqlite};
use bookstore_web::{AppState, app};

async fn spawn_app(state: AppState) -> anyhow::Result<String> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, app(state)).await.expect("run readiness test server");
    });
    Ok(format!("http://{addr}"))
}

fn base_state() -> AppState {
    AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin: AdminService::with_bootstrap(AdminBootstrap::local_defaults()),
        db_pool: None,
        cover_storage: None,
        isbn_lookup: None,
    }
}

#[tokio::test]
async fn ready_returns_service_unavailable_without_database() -> anyhow::Result<()> {
    let base = spawn_app(base_state()).await?;

    let response = reqwest::get(format!("{base}/ready")).await?;

    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}

#[tokio::test]
async fn ready_returns_ok_when_database_is_reachable() -> anyhow::Result<()> {
    let pool = bootstrap_sqlite("sqlite::memory:").await?;
    let mut state = base_state();
    state.db_pool = Some(DatabasePool::Sqlite(pool));
    let base = spawn_app(state).await?;

    let response = reqwest::get(format!("{base}/ready")).await?;

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await?, "ready");
    Ok(())
}

#[tokio::test]
async fn ready_returns_service_unavailable_when_configured_database_is_closed() -> anyhow::Result<()>
{
    let pool = bootstrap_sqlite("sqlite::memory:").await?;
    pool.close().await;

    let mut state = base_state();
    state.db_pool = Some(DatabasePool::Sqlite(pool));
    let base = spawn_app(state).await?;

    let response = reqwest::get(format!("{base}/ready")).await?;

    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    Ok(())
}
