use std::time::Instant;

use axum::http::{HeaderMap, StatusCode, header};
use bookstore_app::{AdminOrder, PosCartItem, PosCartSnapshot};
use chrono::NaiveDateTime;

use crate::models::{AdminOrderResponse, PosCartItemResponse, PosResponse};

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
        channel: order.channel.to_string(),
        status: order.status.to_string(),
        payment_method: order.payment_method.to_string(),
        total_cents: order.total_cents,
        created_at: order.created_at.format("%Y-%m-%d %H:%M").to_string(),
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

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(header::COOKIE).and_then(|value| value.to_str().ok()).and_then(|raw| {
        raw.split(';').find_map(|part| {
            let trimmed = part.trim();
            let (key, value) = trimmed.split_once('=')?;
            if key == name { Some(value.to_string()) } else { None }
        })
    })
}

pub fn current_utc_datetime() -> NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub fn is_valid_iso_date(input: &str) -> bool {
    chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    // --- Property-based tests: is_valid_iso_date ---

    #[quickcheck]
    fn valid_iso_date_round_trips(year: u16, month: u8, day: u8) -> bool {
        let year = (year % 9000) + 1000; // 1000-9999
        let month = (month % 12) + 1;
        let day = (day % 28) + 1; // safe day range
        let s = format!("{year:04}-{month:02}-{day:02}");
        is_valid_iso_date(&s)
    }

    #[quickcheck]
    fn invalid_date_format_rejected(input: String) -> bool {
        // Unless the random string happens to be a valid YYYY-MM-DD date
        let valid = chrono::NaiveDate::parse_from_str(&input, "%Y-%m-%d").is_ok();
        is_valid_iso_date(&input) == valid
    }

    #[quickcheck]
    fn current_utc_datetime_date_is_always_valid() -> bool {
        let dt = current_utc_datetime();
        is_valid_iso_date(&dt.format("%Y-%m-%d").to_string())
    }

    // --- Property-based tests: bearer_token ---

    #[quickcheck]
    fn bearer_token_extracts_after_prefix(token: String) -> bool {
        if token.is_empty() {
            return true; // empty token should be rejected
        }
        let header_val = format!("Bearer {token}");
        let mut headers = HeaderMap::new();
        match header_val.parse() {
            Ok(val) => {
                headers.insert(header::AUTHORIZATION, val);
                match bearer_token(&headers) {
                    Ok(extracted) => extracted == token,
                    Err(_) => true,
                }
            }
            Err(_) => true, // non-visible ASCII chars can't be header values
        }
    }

    #[quickcheck]
    fn bearer_token_rejects_missing_header() -> bool {
        bearer_token(&HeaderMap::new()).is_err()
    }

    #[quickcheck]
    fn bearer_token_rejects_wrong_prefix(raw: String) -> bool {
        if raw.starts_with("Bearer ") && raw.len() > 7 {
            return true; // valid bearer token, skip
        }
        let mut headers = HeaderMap::new();
        if let Ok(val) = raw.parse() {
            headers.insert(header::AUTHORIZATION, val);
            bearer_token(&headers).is_err()
        } else {
            true // unparseable header value
        }
    }

    // --- Property-based tests: cookie_value ---

    #[quickcheck]
    fn cookie_value_finds_matching_name(name: String, value: String) -> bool {
        // Cookie headers only support visible ASCII; skip anything else
        let is_cookie_safe = |s: &str| {
            !s.is_empty()
                && s.chars().all(|c| c.is_ascii_graphic())
                && !s.contains('=')
                && !s.contains(';')
        };
        if !is_cookie_safe(&name) || value.chars().any(|c| !c.is_ascii_graphic() || c == ';') {
            return true;
        }
        let cookie = format!("{name}={value}");
        let mut headers = HeaderMap::new();
        if let Ok(val) = cookie.parse() {
            headers.insert(header::COOKIE, val);
            cookie_value(&headers, &name) == Some(value)
        } else {
            true
        }
    }

    #[quickcheck]
    fn cookie_value_returns_none_for_missing(name: String) -> bool {
        cookie_value(&HeaderMap::new(), &name).is_none()
    }

    // --- Property-based tests: require_same_origin ---

    #[quickcheck]
    fn same_origin_accepts_matching_origin(host: String) -> bool {
        if host.is_empty() || !host.is_ascii() {
            return true;
        }
        let mut headers = HeaderMap::new();
        if let (Ok(h), Ok(o)) = (host.parse(), format!("http://{host}").parse()) {
            headers.insert(header::HOST, h);
            headers.insert(header::ORIGIN, o);
            require_same_origin(&headers).is_ok()
        } else {
            true
        }
    }

    #[quickcheck]
    fn same_origin_accepts_no_origin_header(host: String) -> bool {
        let mut headers = HeaderMap::new();
        if let Ok(h) = host.parse() {
            headers.insert(header::HOST, h);
            require_same_origin(&headers).is_ok()
        } else {
            true
        }
    }
}
