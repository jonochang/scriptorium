use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub struct CatalogQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct StorefrontCheckoutLineItemRequest {
    pub item_id: String,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
pub struct StorefrontCheckoutSessionRequest {
    pub email: String,
    #[serde(default)]
    pub customer_name: String,
    #[serde(default)]
    pub delivery_method: String,
    #[serde(default)]
    pub donation_cents: i64,
    pub line_items: Vec<StorefrontCheckoutLineItemRequest>,
}

#[derive(Debug, Serialize)]
pub struct StorefrontCheckoutSessionResponse {
    pub session_id: String,
    pub order_id: String,
    pub total_cents: i64,
}
