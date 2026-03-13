use axum::Json;
use axum::extract::State;
use bookstore_app::{SalesEvent, WebhookFinalizeStatus};
use bookstore_domain::{OrderChannel, OrderStatus, PaymentMethod};
use std::time::Instant;

use crate::AppState;
use crate::models::{PaymentsWebhookRequest, PaymentsWebhookResponse};
use crate::web_support::{current_utc_datetime, log_checkout_event};

pub async fn payments_webhook(
    State(state): State<AppState>,
    Json(request): Json<PaymentsWebhookRequest>,
) -> Result<Json<PaymentsWebhookResponse>, axum::http::StatusCode> {
    let started_at = Instant::now();
    let result = state
        .storefront
        .finalize_webhook(&request.external_ref, &request.session_id)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    if result.status == WebhookFinalizeStatus::Processed {
        let customer_name = if result.session.email.trim().is_empty() {
            "Online Customer"
        } else {
            result.session.email.as_str()
        };
        let today = current_utc_datetime();
        state
            .admin
            .create_order(
                &result.session.tenant_id,
                customer_name,
                OrderChannel::Online,
                OrderStatus::Paid,
                PaymentMethod::OnlineCard,
                result.session.total_cents,
                today,
            )
            .await;
        state
            .admin
            .record_sales_event(SalesEvent {
                tenant_id: result.session.tenant_id.clone(),
                payment_method: PaymentMethod::OnlineCard,
                sales_cents: result.session.sales_cents,
                donations_cents: result.session.donation_cents,
                cogs_cents: 0,
                occurred_at: today,
            })
            .await;
    }
    log_checkout_event(
        "payment_webhook_finalize",
        match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        "online_card",
        result.session.total_cents,
        started_at,
    );
    Ok(Json(PaymentsWebhookResponse {
        status: match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        receipt_sent: result.receipt_sent,
    }))
}
