use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PosLoginRequest {
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct PosLoginResponse {
    pub session_token: String,
}

#[derive(Debug, Deserialize)]
pub struct PosScanRequest {
    pub session_token: String,
    #[serde(alias = "isbn")]
    pub barcode: String,
}

#[derive(Debug, Deserialize)]
pub struct PosQuickItemRequest {
    pub session_token: String,
    pub item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
pub struct PosCartQuantityRequest {
    pub session_token: String,
    pub item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
pub struct PosCashPaymentRequest {
    pub session_token: String,
    pub tendered_cents: i64,
    pub donate_change: bool,
    #[serde(default)]
    pub discount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct PosExternalCardRequest {
    pub session_token: String,
    pub external_ref: String,
    #[serde(default)]
    pub discount_cents: i64,
}

#[derive(Debug, Deserialize)]
pub struct PosIouRequest {
    pub session_token: String,
    pub customer_name: String,
    #[serde(default)]
    pub discount_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct PosResponse {
    pub status: &'static str,
    pub message: String,
    pub total_cents: i64,
    pub change_due_cents: i64,
    pub donation_cents: i64,
    pub discount_cents: i64,
    pub items: Vec<PosCartItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct PosCartItemResponse {
    pub item_id: String,
    pub title: String,
    pub unit_price_cents: i64,
    pub quantity: i64,
    pub is_quick_item: bool,
}

#[derive(Debug, Serialize)]
pub struct PosConfigResponse {
    pub quick_items: Vec<bookstore_app::seed::SeedQuickItem>,
    pub discount_codes: Vec<bookstore_app::seed::SeedDiscountCode>,
}
