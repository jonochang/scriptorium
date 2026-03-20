use std::net::SocketAddr;

use std::sync::Arc;

use bookstore_app::seed::SeedData;
use bookstore_app::{AdminBootstrap, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_data::bootstrap_database;
use bookstore_web::isbn_lookup::IsbnLookupClient;
use bookstore_web::object_storage::{ObjectStorage, ObjectStorageConfig};
use bookstore_web::{AppState, app};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let seed = match std::env::var("SEED_FILE").ok() {
        Some(path) => SeedData::load(std::path::Path::new(&path))?,
        None => SeedData::default(),
    };

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://scriptorium.db?mode=rwc".to_string());
    let db_pool = bootstrap_database(&database_url).await?;
    let cover_storage = match ObjectStorageConfig::from_env() {
        Some(config) => Some(ObjectStorage::new(config).await?),
        None => None,
    };
    if let Some(storage) = &cover_storage {
        storage.ensure_bucket().await?;
    }

    let state = AppState {
        catalog: CatalogService::from_seed(&seed),
        pos: PosService::from_seed(&seed),
        storefront: StorefrontService::new(),
        admin: AdminService::with_bootstrap_and_seed(AdminBootstrap::from_env(), &seed),
        db_pool: Some(db_pool),
        cover_storage,
        isbn_lookup: Some(IsbnLookupClient::open_library()),
        seed: Arc::new(seed),
    };

    let addr = listen_addr_from_env()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("bookstore-web listening on {addr}");

    axum::serve(listener, app(state)).await?;
    Ok(())
}

fn listen_addr_from_env() -> anyhow::Result<SocketAddr> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{host}:{port}");
    addr.parse().map_err(|error| anyhow::anyhow!("invalid HOST/PORT combination {addr}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::listen_addr_from_env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_host_port_env<T>(
        host: Option<&str>,
        port: Option<&str>,
        test: impl FnOnce() -> T,
    ) -> T {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let previous_host = std::env::var("HOST").ok();
        let previous_port = std::env::var("PORT").ok();

        match host {
            Some(value) => unsafe { std::env::set_var("HOST", value) },
            None => unsafe { std::env::remove_var("HOST") },
        }
        match port {
            Some(value) => unsafe { std::env::set_var("PORT", value) },
            None => unsafe { std::env::remove_var("PORT") },
        }

        let result = test();

        match previous_host {
            Some(value) => unsafe { std::env::set_var("HOST", value) },
            None => unsafe { std::env::remove_var("HOST") },
        }
        match previous_port {
            Some(value) => unsafe { std::env::set_var("PORT", value) },
            None => unsafe { std::env::remove_var("PORT") },
        }

        result
    }

    #[test]
    fn listen_addr_defaults_to_localhost() {
        with_host_port_env(None, None, || {
            let addr = listen_addr_from_env().expect("default bind address should parse");

            assert_eq!(addr.to_string(), "127.0.0.1:8080");
        });
    }

    #[test]
    fn listen_addr_reads_environment() {
        with_host_port_env(Some("0.0.0.0"), Some("9090"), || {
            let addr = listen_addr_from_env().expect("env bind address should parse");

            assert_eq!(addr.to_string(), "0.0.0.0:9090");
        });
    }
}
