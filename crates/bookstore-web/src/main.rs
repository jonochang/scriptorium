use std::net::SocketAddr;

use bookstore_app::{AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_data::bootstrap_sqlite;
use bookstore_web::object_storage::{ObjectStorage, ObjectStorageConfig};
use bookstore_web::{AppState, app};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://scriptorium.db?mode=rwc".to_string());
    let db_pool = bootstrap_sqlite(&database_url).await?;
    let cover_storage = match ObjectStorageConfig::from_env() {
        Some(config) => Some(ObjectStorage::new(config).await?),
        None => None,
    };
    if let Some(storage) = &cover_storage {
        storage.ensure_bucket().await?;
    }

    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin: AdminService::with_bootstrap(AdminBootstrap::from_env()),
        db_pool: Some(db_pool),
        cover_storage,
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("bookstore-web listening on {addr}");

    axum::serve(listener, app(state)).await?;
    Ok(())
}
