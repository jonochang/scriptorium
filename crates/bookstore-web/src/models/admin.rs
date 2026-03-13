use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AdminIsbnLookupRequest {
    pub token: String,
    pub isbn: String,
}

#[derive(Debug, Serialize)]
pub struct AdminIsbnLookupResponse {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminInventoryReceiveRequest {
    pub token: String,
    pub tenant_id: String,
    pub isbn: String,
    pub quantity: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminInventoryReceiveResponse {
    pub tenant_id: String,
    pub isbn: String,
    pub on_hand: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdminAuthLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AdminAuthLoginResponse {
    pub token: String,
    pub tenant_id: String,
    pub role: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AdminCoverUploadResponse {
    pub object_key: String,
    pub asset_url: String,
    pub content_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminInventoryAdjustRequest {
    pub token: String,
    pub tenant_id: String,
    pub isbn: String,
    pub delta: i64,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct AdminStockMovementResponse {
    pub tenant_id: String,
    pub isbn: String,
    pub delta: i64,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminProductUpsertRequest {
    pub token: String,
    pub tenant_id: String,
    pub product_id: String,
    pub title: String,
    pub isbn: String,
    pub category: String,
    pub vendor: String,
    pub cost_cents: i64,
    pub retail_cents: i64,
    pub cover_image_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminProductResponse {
    pub tenant_id: String,
    pub product_id: String,
    pub title: String,
    pub isbn: String,
    pub category: String,
    pub vendor: String,
    pub cost_cents: i64,
    pub retail_cents: i64,
    pub quantity_on_hand: i64,
    pub cover_image_key: Option<String>,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminDeleteResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AdminTaxonomyListResponse {
    pub tenant_id: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminReportSummaryResponse {
    pub tenant_id: String,
    pub sales_cents: i64,
    pub donations_cents: i64,
    pub cogs_cents: i64,
    pub gross_profit_cents: i64,
    pub sales_by_payment: std::collections::BTreeMap<String, i64>,
}

#[derive(Debug, Serialize)]
pub struct AdminOrderResponse {
    pub order_id: String,
    pub tenant_id: String,
    pub customer_name: String,
    pub channel: String,
    pub status: String,
    pub payment_method: String,
    pub total_cents: i64,
    pub created_at: String,
}
