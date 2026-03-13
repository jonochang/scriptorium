use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use bookstore_app::RequestContext;

use crate::AppState;
use crate::models::ContextResponse;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ready(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    pool.check_ready().await.map(|_| "ready").map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

pub async fn request_context(
    axum::extract::Extension(context): axum::extract::Extension<RequestContext>,
) -> Json<ContextResponse> {
    Json(ContextResponse { tenant_id: context.tenant_id, locale: context.locale })
}

pub async fn list_books(State(state): State<AppState>) -> Json<Vec<bookstore_domain::Book>> {
    Json(state.catalog.list_books().await)
}
