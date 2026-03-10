use std::time::Instant;

use axum::http::{HeaderMap, StatusCode, header};
use bookstore_app::{AdminOrder, PosCartItem, PosCartSnapshot};

use crate::{AdminOrderResponse, PosCartItemResponse, PosResponse};

pub fn log_checkout_event(
    event: &str,
    status: &str,
    payment_method: &str,
    total_cents: i64,
    started_at: Instant,
) {
    tracing::info!(
        event = event,
        status = status,
        payment_method = payment_method,
        total_cents = total_cents,
        latency_ms = started_at.elapsed().as_millis() as u64,
        "checkout event"
    );
}

pub fn pos_items(items: Vec<PosCartItem>) -> Vec<PosCartItemResponse> {
    items
        .into_iter()
        .map(|item| PosCartItemResponse {
            item_id: item.item_id,
            title: item.title,
            unit_price_cents: item.unit_price_cents,
            quantity: item.quantity,
            is_quick_item: item.is_quick_item,
        })
        .collect()
}

pub fn pos_cart_response(snapshot: PosCartSnapshot, message: impl Into<String>) -> PosResponse {
    PosResponse {
        status: "cart_updated",
        message: message.into(),
        total_cents: snapshot.total_cents,
        change_due_cents: 0,
        donation_cents: 0,
        discount_cents: 0,
        items: pos_items(snapshot.items),
    }
}

pub fn admin_order_response(order: AdminOrder) -> AdminOrderResponse {
    AdminOrderResponse {
        order_id: order.order_id,
        tenant_id: order.tenant_id,
        customer_name: order.customer_name,
        channel: order.channel,
        status: order.status,
        payment_method: order.payment_method,
        total_cents: order.total_cents,
        created_on: order.created_on,
    }
}

pub fn require_same_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty());
    let host = headers.get(header::HOST).and_then(|value| value.to_str().ok()).unwrap_or("");

    if let Some(origin) = origin {
        let expected_http = format!("http://{host}");
        let expected_https = format!("https://{host}");
        if origin != expected_http && origin != expected_https {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(())
}

pub fn bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token = raw.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(token.to_string())
}

pub fn current_utc_date() -> String {
    chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

pub fn is_valid_iso_date(input: &str) -> bool {
    chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").is_ok()
}
