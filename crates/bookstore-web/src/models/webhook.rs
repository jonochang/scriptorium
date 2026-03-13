use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaymentsWebhookRequest {
    pub external_ref: String,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct PaymentsWebhookResponse {
    pub status: &'static str,
    pub receipt_sent: bool,
}
