use std::net::SocketAddr;
use std::sync::Arc;

use bookstore_core::seed_church_bookstore;
use bookstore_web::{AppState, app};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let state = AppState { inventory: Arc::new(RwLock::new(seed_church_bookstore())) };

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("bookstore-web listening on {addr}");

    axum::serve(listener, app(state)).await?;
    Ok(())
}
