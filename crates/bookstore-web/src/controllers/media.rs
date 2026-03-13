use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::i18n;

pub async fn media_asset(
    State(state): State<AppState>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> Result<Response, axum::http::StatusCode> {
    let storage = state.cover_storage.clone().ok_or(axum::http::StatusCode::NOT_FOUND)?;
    let object = storage
        .get(key.trim_start_matches('/'))
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    Ok(([(axum::http::header::CONTENT_TYPE, object.content_type)], object.bytes).into_response())
}

pub async fn i18n_lookup(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<std::collections::HashMap<String, String>> {
    let locale = params.get("locale").map_or("en-AU", String::as_str);
    let key = params.get("key").map_or("checkout.complete", String::as_str);
    let value = i18n::lookup(locale, key);
    let mut payload = std::collections::HashMap::new();
    payload.insert("locale".to_string(), locale.to_string());
    payload.insert("key".to_string(), key.to_string());
    payload.insert("value".to_string(), value.to_string());
    Json(payload)
}
