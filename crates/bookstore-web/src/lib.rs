mod admin_intake_ui;
mod admin_pages;
mod admin_ui;
mod catalog_ui;
pub mod controllers;
mod i18n;
pub mod isbn_lookup;
pub mod models;
pub mod object_storage;
mod storefront_ui;
mod ui;
mod views;
mod web_support;

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::Router;
use bookstore_app::{
    AdminService, CatalogService, PosService, RequestContext, StorefrontService,
};
use bookstore_data::DatabasePool;
use controllers::*;
use isbn_lookup::IsbnLookupClient;
use object_storage::ObjectStorage;

#[derive(Clone, Default)]
pub struct AppState {
    pub catalog: CatalogService,
    pub pos: PosService,
    pub storefront: StorefrontService,
    pub admin: AdminService,
    pub db_pool: Option<DatabasePool>,
    pub cover_storage: Option<ObjectStorage>,
    pub isbn_lookup: Option<IsbnLookupClient>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/books", get(list_books))
        .route("/context", get(request_context))
        .route("/admin", get(admin_dashboard_shell))
        .route("/admin/logout", get(admin_logout))
        .route("/admin/orders", get(admin_orders_shell))
        .route("/admin/intake", get(admin_intake_shell))
        .route("/catalog", get(storefront_catalog))
        .route("/catalog/items/{book_id}", get(storefront_product_detail))
        .route("/catalog/search", get(storefront_search))
        .route("/cart", get(storefront_cart))
        .route("/checkout", get(storefront_checkout))
        .route("/orders", get(storefront_orders))
        .route("/pos", get(pos_shell))
        .route("/api/pos/login", post(pos_login))
        .route("/api/pos/scan", post(pos_scan))
        .route("/api/pos/cart/items", post(pos_quick_item))
        .route("/api/pos/cart/quantity", post(pos_set_cart_quantity))
        .route("/api/pos/payments/cash", post(pos_pay_cash))
        .route("/api/pos/payments/external-card", post(pos_pay_external_card))
        .route("/api/pos/payments/iou", post(pos_pay_iou))
        .route("/api/storefront/checkout/session", post(storefront_checkout_session))
        .route("/api/payments/webhook", post(payments_webhook))
        .route("/api/admin/products/isbn-lookup", post(admin_isbn_lookup))
        .route("/api/admin/products/cover-upload", post(admin_cover_upload))
        .route("/api/admin/inventory/receive", post(admin_inventory_receive))
        .route("/api/admin/inventory/adjust", post(admin_inventory_adjust))
        .route("/api/admin/inventory/journal", get(admin_inventory_journal))
        .route("/api/admin/auth/login", post(admin_auth_login))
        .route("/api/admin/products", post(admin_product_upsert).get(admin_product_list))
        .route("/api/admin/products/{product_id}", delete(admin_product_delete))
        .route("/api/admin/categories", get(admin_categories_list))
        .route("/api/admin/vendors", get(admin_vendors_list))
        .route("/api/admin/orders", get(admin_orders_list))
        .route("/api/admin/orders/{order_id}/mark-paid", post(admin_order_mark_paid))
        .route("/api/admin/reports/summary", get(admin_report_summary))
        .route("/api/i18n", get(i18n_lookup))
        .route("/media/{*key}", get(media_asset))
        .layer(middleware::from_fn(request_context_middleware))
        .with_state(state)
}

async fn request_context_middleware(mut request: Request, next: Next) -> Response {
    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();

    let locale = request
        .headers()
        .get("accept-language")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("en-AU")
        .to_string();

    request.extensions_mut().insert(RequestContext { tenant_id, locale });
    next.run(request).await
}
