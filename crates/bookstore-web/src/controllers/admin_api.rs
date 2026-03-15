use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use bookstore_app::{AdminProduct, AdminRole};

use crate::AppState;
use crate::isbn_lookup;
use crate::models::{
    AdminAuthLoginRequest, AdminAuthLoginResponse, AdminCoverUploadResponse, AdminDeleteResponse,
    AdminInventoryAdjustRequest, AdminInventoryReceiveRequest, AdminInventoryReceiveResponse,
    AdminIsbnLookupRequest, AdminIsbnLookupResponse, AdminOrderResponse, AdminProductResponse,
    AdminProductUpsertRequest, AdminReportSummaryResponse, AdminStockMovementResponse,
    AdminTaxonomyListResponse,
};
use crate::web_support::{
    admin_order_response, bearer_token, is_valid_iso_date, require_same_origin,
};

use super::admin_pages::ADMIN_SESSION_COOKIE;

fn cover_media_url(key: Option<&str>) -> Option<String> {
    key.map(|value| format!("/media/{value}"))
}

pub async fn admin_auth_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminAuthLoginRequest>,
) -> Result<Response, StatusCode> {
    require_same_origin(&headers)?;
    let session = state
        .admin
        .login(&request.username, &request.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok((
        [(
            axum::http::header::SET_COOKIE,
            format!("{ADMIN_SESSION_COOKIE}={}; Path=/; HttpOnly; SameSite=Lax", session.token),
        )],
        Json(AdminAuthLoginResponse {
            token: session.token,
            tenant_id: session.tenant_id,
            role: match session.role {
                AdminRole::Admin => "admin",
                AdminRole::Volunteer => "volunteer",
            },
        }),
    )
        .into_response())
}

pub async fn admin_isbn_lookup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminIsbnLookupRequest>,
) -> Result<Json<AdminIsbnLookupResponse>, StatusCode> {
    require_same_origin(&headers)?;
    state.admin.require_admin(&request.token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    let metadata = match &state.isbn_lookup {
        Some(client) => client.lookup(&request.isbn).await.ok().flatten(),
        None => None,
    };
    let metadata = match metadata {
        Some(metadata) => metadata,
        None => {
            let fallback = state
                .admin
                .lookup_isbn(&request.isbn)
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            isbn_lookup::IsbnLookupRecord {
                isbn: fallback.isbn,
                title: fallback.title,
                author: fallback.author,
                description: fallback.description,
                cover_image_url: None,
            }
        }
    };
    Ok(Json(AdminIsbnLookupResponse {
        isbn: metadata.isbn,
        title: metadata.title,
        author: metadata.author,
        description: metadata.description,
        cover_image_url: metadata.cover_image_url,
    }))
}

pub async fn admin_cover_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<AdminCoverUploadResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let storage = state.cover_storage.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let mut token = String::new();
    let mut tenant_id = String::new();
    let mut file_name = "cover.bin".to_string();
    let mut content_type = "application/octet-stream".to_string();
    let mut file_bytes = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        match field.name().unwrap_or_default() {
            "token" => {
                token = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "tenant_id" => {
                tenant_id = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "file" => {
                file_name = field.file_name().unwrap_or("cover.bin").to_string();
                content_type =
                    field.content_type().unwrap_or("application/octet-stream").to_string();
                file_bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            }
            _ => {}
        }
    }

    if token.is_empty() || tenant_id.is_empty() || file_bytes.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let object_key = storage.key_for_upload(&tenant_id, &file_name);
    storage
        .put(&object_key, file_bytes, &content_type)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(Json(AdminCoverUploadResponse {
        asset_url: storage.asset_url(&object_key),
        object_key,
        content_type,
    }))
}

pub async fn admin_inventory_receive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminInventoryReceiveRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let session =
        state.admin.require_admin(&request.token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .receive_inventory(&request.tenant_id, &request.isbn, request.quantity)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

pub async fn admin_inventory_adjust(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminInventoryAdjustRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let session =
        state.admin.require_admin(&request.token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .adjust_inventory(&request.tenant_id, &request.isbn, request.delta, &request.reason)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

pub async fn admin_inventory_journal(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminStockMovementResponse>>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let items = state
        .admin
        .movement_journal(tenant_id)
        .await
        .into_iter()
        .map(|movement| AdminStockMovementResponse {
            tenant_id: movement.tenant_id,
            isbn: movement.isbn,
            delta: movement.delta,
            reason: movement.reason,
        })
        .collect();
    Ok(Json(items))
}

pub async fn admin_product_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AdminProductUpsertRequest>,
) -> Result<Json<AdminProductResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let session =
        state.admin.require_admin(&request.token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let product = AdminProduct {
        tenant_id: request.tenant_id,
        product_id: request.product_id,
        title: request.title,
        isbn: request.isbn,
        category: request.category,
        vendor: request.vendor,
        cost_cents: request.cost_cents,
        retail_cents: request.retail_cents,
        cover_image_key: request.cover_image_key,
    };
    state.admin.upsert_product(product.clone()).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let quantity_on_hand = state.admin.inventory_on_hand(&session.tenant_id, &product.isbn).await;
    Ok(Json(AdminProductResponse {
        tenant_id: product.tenant_id,
        product_id: product.product_id,
        title: product.title,
        isbn: product.isbn,
        category: product.category,
        vendor: product.vendor,
        cost_cents: product.cost_cents,
        retail_cents: product.retail_cents,
        quantity_on_hand,
        cover_image_key: product.cover_image_key.clone(),
        cover_image_url: cover_media_url(product.cover_image_key.as_deref()),
    }))
}

pub async fn admin_product_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminProductResponse>>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let products = state.admin.list_products(tenant_id).await;
    let mut response = Vec::with_capacity(products.len());
    for product in products {
        response.push(AdminProductResponse {
            quantity_on_hand: state.admin.inventory_on_hand(tenant_id, &product.isbn).await,
            tenant_id: product.tenant_id,
            product_id: product.product_id,
            title: product.title,
            isbn: product.isbn,
            category: product.category,
            vendor: product.vendor,
            cost_cents: product.cost_cents,
            retail_cents: product.retail_cents,
            cover_image_key: product.cover_image_key.clone(),
            cover_image_url: cover_media_url(product.cover_image_key.as_deref()),
        });
    }
    Ok(Json(response))
}

pub async fn admin_product_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(product_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminDeleteResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    state.admin.delete_product(tenant_id, &product_id).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(AdminDeleteResponse { status: "deleted" }))
}

pub async fn admin_categories_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_categories(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

pub async fn admin_vendors_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_vendors(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

pub async fn admin_orders_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminOrderResponse>>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let orders =
        state.admin.list_orders(tenant_id).await.into_iter().map(admin_order_response).collect();
    Ok(Json(orders))
}

pub async fn admin_order_mark_paid(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(order_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminOrderResponse>, StatusCode> {
    require_same_origin(&headers)?;
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let order = state
        .admin
        .mark_order_paid(tenant_id, &order_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(admin_order_response(order)))
}

pub async fn admin_report_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminReportSummaryResponse>, StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state.admin.require_admin(&token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }
    let parse_date = |s: &str| -> Option<chrono::NaiveDateTime> {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
    };
    let from_str = params.get("from").map(String::as_str);
    let to_str = params.get("to").map(String::as_str);
    if from_str.is_some_and(|date| !is_valid_iso_date(date))
        || to_str.is_some_and(|date| !is_valid_iso_date(date))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let from = from_str.and_then(parse_date);
    let to = to_str.and_then(|s| parse_date(s).map(|d| d + chrono::Duration::days(1)));
    let report = state.admin.report_summary_range(tenant_id, from, to).await;
    Ok(Json(AdminReportSummaryResponse {
        tenant_id: report.tenant_id,
        sales_cents: report.sales_cents,
        donations_cents: report.donations_cents,
        cogs_cents: report.cogs_cents,
        gross_profit_cents: report.gross_profit_cents,
        sales_by_payment: report
            .sales_by_payment
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    }))
}
