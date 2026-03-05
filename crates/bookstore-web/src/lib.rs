use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use bookstore_core::{Book, Inventory};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AppState {
    pub inventory: Arc<RwLock<Inventory>>,
}

pub fn app(state: AppState) -> Router {
    Router::new().route("/health", get(health)).route("/books", get(list_books)).with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
    let inventory = state.inventory.read().await;
    Json(inventory.books().to_vec())
}
