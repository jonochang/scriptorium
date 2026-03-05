use std::net::SocketAddr;

use bookstore_app::{AdminService, CatalogService, PosService, StorefrontService};
use bookstore_data::bootstrap_sqlite;
use bookstore_web::{AppState, app};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://scriptorium.db?mode=rwc".to_string());
    let db_pool = bootstrap_sqlite(&database_url).await?;

    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin: AdminService::new(),
        db_pool: Some(db_pool),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("bookstore-web listening on {addr}");

    axum::serve(listener, app(state)).await?;
    Ok(())
}
