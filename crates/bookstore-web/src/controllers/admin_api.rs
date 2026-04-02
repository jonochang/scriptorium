use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use bookstore_app::AdminRole;
use bookstore_data::runtime::{
    RuntimeProduct, adjust_inventory, delete_product, get_product_by_id, get_product_by_isbn,
    list_categories, list_orders, list_products, list_stock_movements, list_vendors,
    mark_order_paid, receive_inventory, report_summary, upsert_product,
};

use crate::AppState;
use crate::models::{
    AdminAuthLoginRequest, AdminAuthLoginResponse, AdminCoverUploadResponse, AdminDeleteResponse,
    AdminInventoryAdjustRequest, AdminInventoryReceiveRequest, AdminInventoryReceiveResponse,
    AdminIsbnLookupRequest, AdminIsbnLookupResponse, AdminOrderResponse, AdminProductResponse,
    AdminProductUpsertRequest, AdminReportSummaryResponse, AdminStockMovementResponse,
    AdminTaxonomyListResponse, ApiError,
};
use crate::web_support::{
    bearer_token, is_valid_iso_date, require_same_origin,
};

use super::admin_pages::ADMIN_SESSION_COOKIE;

fn cover_media_url(key: Option<&str>) -> Option<String> {
    key.map(|value| format!("/media/{value}"))
}

fn admin_product_response(product: RuntimeProduct) -> AdminProductResponse {
    AdminProductResponse {
        tenant_id: product.tenant_id,
        product_id: product.product_id,
        title: product.title,
        isbn: product.isbn,
        author: product.author,
        publisher: product.publisher,
        description: product.description,
        public_title: product.public_title,
        public_author: product.public_author,
        public_publisher: product.public_publisher,
        public_description: product.public_description,
        public_cover_image_url: product.public_cover_image_url,
        category: product.category,
        vendor: product.vendor,
        cost_cents: product.cost_cents,
        retail_cents: product.retail_cents,
        quantity_on_hand: product.quantity_on_hand,
        cover_image_key: product.cover_image_key.clone(),
        cover_image_url: cover_media_url(product.cover_image_key.as_deref()),
    }
}

async fn sync_pos_product(state: &AppState, tenant_id: &str, isbn: &str) {
    let Some(pool) = state.db_pool.as_ref() else {
        return;
    };
    let Ok(Some(product)) = get_product_by_isbn(pool, tenant_id, isbn).await else {
        return;
    };
    state
        .pos
        .upsert_inventory_item(
            &product.isbn,
            &product.product_id,
            &product.title,
            product.retail_cents,
            product.quantity_on_hand,
        )
        .await;
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
    let session = state.admin.require_admin(&request.token).await.map_err(|_| StatusCode::UNAUTHORIZED)?;
    let saved = match state.db_pool.as_ref() {
        Some(pool) => get_product_by_isbn(pool, &session.tenant_id, &request.isbn).await.ok().flatten(),
        None => None,
    };
    let remote = match &state.isbn_lookup {
        Some(client) => client.lookup(&request.isbn).await.ok().flatten(),
        None => None,
    };

    match (saved, remote) {
        (Some(saved), remote) => {
            let remote_title = remote.as_ref().map(|item| item.title.clone()).unwrap_or_default();
            let remote_author = remote.as_ref().map(|item| item.author.clone()).unwrap_or_default();
            let remote_publisher =
                remote.as_ref().map(|item| item.publisher.clone()).unwrap_or_default();
            let remote_description =
                remote.as_ref().map(|item| item.description.clone()).unwrap_or_default();
            let remote_cover = remote.and_then(|item| item.cover_image_url);
            Ok(Json(AdminIsbnLookupResponse {
                isbn: saved.isbn,
                product_id: Some(saved.product_id),
                title: if !saved.title.trim().is_empty() {
                    saved.title.clone()
                } else if !saved.public_title.trim().is_empty() {
                    saved.public_title.clone()
                } else {
                    remote_title.clone()
                },
                author: if !saved.author.trim().is_empty() {
                    saved.author.clone()
                } else if !saved.public_author.trim().is_empty() {
                    saved.public_author.clone()
                } else {
                    remote_author.clone()
                },
                publisher: if !saved.publisher.trim().is_empty() {
                    saved.publisher.clone()
                } else if !saved.public_publisher.trim().is_empty() {
                    saved.public_publisher.clone()
                } else {
                    remote_publisher.clone()
                },
                description: if !saved.description.trim().is_empty() {
                    saved.description.clone()
                } else if !saved.public_description.trim().is_empty() {
                    saved.public_description.clone()
                } else {
                    remote_description.clone()
                },
                public_title: if saved.public_title.trim().is_empty() {
                    remote_title
                } else {
                    saved.public_title
                },
                public_author: if saved.public_author.trim().is_empty() {
                    remote_author
                } else {
                    saved.public_author
                },
                public_publisher: if saved.public_publisher.trim().is_empty() {
                    remote_publisher
                } else {
                    saved.public_publisher
                },
                public_description: if saved.public_description.trim().is_empty() {
                    remote_description
                } else {
                    saved.public_description
                },
                public_cover_image_url: saved.public_cover_image_url.or(remote_cover.clone()),
                category: saved.category,
                vendor: saved.vendor,
                cost_cents: saved.cost_cents,
                retail_cents: saved.retail_cents,
                quantity_on_hand: saved.quantity_on_hand,
                cover_image_key: saved.cover_image_key.clone(),
                cover_image_url: cover_media_url(saved.cover_image_key.as_deref()).or(remote_cover),
            }))
        }
        (None, Some(metadata)) => {
            let public_title = metadata.title.clone();
            let public_author = metadata.author.clone();
            let public_publisher = metadata.publisher.clone();
            let public_description = metadata.description.clone();
            let public_cover_image_url = metadata.cover_image_url.clone();
            Ok(Json(AdminIsbnLookupResponse {
            isbn: metadata.isbn,
            product_id: None,
            title: metadata.title,
            author: metadata.author,
            publisher: metadata.publisher,
            description: metadata.description,
            public_title,
            public_author,
            public_publisher,
            public_description,
            public_cover_image_url,
            category: String::new(),
            vendor: String::new(),
            cost_cents: 0,
            retail_cents: 0,
            quantity_on_hand: 0,
            cover_image_key: None,
            cover_image_url: metadata.cover_image_url,
        }))
        }
        (None, None) => Err(StatusCode::BAD_REQUEST),
    }
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let on_hand =
        receive_inventory(pool, &request.tenant_id, &request.isbn, request.quantity)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    sync_pos_product(&state, &request.tenant_id, &request.isbn).await;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: request.tenant_id,
        isbn: request.isbn,
        on_hand,
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let on_hand = adjust_inventory(pool, &request.tenant_id, &request.isbn, request.delta, &request.reason)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    sync_pos_product(&state, &request.tenant_id, &request.isbn).await;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: request.tenant_id,
        isbn: request.isbn,
        on_hand,
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let items = list_stock_movements(pool, tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
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
) -> Result<Json<AdminProductResponse>, ApiError> {
    require_same_origin(&headers)
        .map_err(|status| ApiError::new(status, "Cross-origin admin requests are not allowed."))?;
    let session =
        state
            .admin
            .require_admin(&request.token)
            .await
            .map_err(|err| ApiError::new(StatusCode::UNAUTHORIZED, err.to_string()))?;
    if session.tenant_id != request.tenant_id {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "You cannot save products for another tenant.",
        ));
    }
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "Database is unavailable."));
    };
    let quantity_on_hand =
        get_product_by_id(pool, &session.tenant_id, &request.product_id)
            .await
            .ok()
            .flatten()
            .map(|item| item.quantity_on_hand)
            .unwrap_or(0);
    let saved = upsert_product(
        pool,
        RuntimeProduct {
            tenant_id: request.tenant_id,
            product_id: request.product_id,
            title: request.title,
            isbn: request.isbn,
            author: request.author,
            publisher: request.publisher,
            description: request.description,
            public_title: request.public_title,
            public_author: request.public_author,
            public_publisher: request.public_publisher,
            public_description: request.public_description,
            public_cover_image_url: request.public_cover_image_url,
            category: request.category,
            vendor: request.vendor,
            cost_cents: request.cost_cents,
            retail_cents: request.retail_cents,
            quantity_on_hand,
            cover_image_key: request.cover_image_key,
            binding: String::new(),
            pages: String::new(),
            is_catalog_visible: true,
        },
    )
    .await
    .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .pos
        .upsert_inventory_item(
            &saved.isbn,
            &saved.product_id,
            &saved.title,
            saved.retail_cents,
            saved.quantity_on_hand,
        )
        .await;
    Ok(Json(admin_product_response(saved)))
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let products = list_products(pool, tenant_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(products.into_iter().map(admin_product_response).collect()))
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let Some(product) = get_product_by_id(pool, tenant_id, &product_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    delete_product(pool, tenant_id, &product_id).await.map_err(|_| StatusCode::NOT_FOUND)?;
    state.pos.remove_inventory_item(&product.isbn).await;
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let values = list_categories(pool, tenant_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let values = list_vendors(pool, tenant_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let orders = list_orders(pool, tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|order| AdminOrderResponse {
            order_id: order.order_id,
            tenant_id: order.tenant_id,
            customer_name: order.customer_name,
            channel: order.channel,
            status: order.status,
            payment_method: order.payment_method,
            total_cents: order.total_cents,
            created_at: order.created_at,
        })
        .collect();
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
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let order = mark_order_paid(pool, tenant_id, &order_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(AdminOrderResponse {
        order_id: order.order_id,
        tenant_id: order.tenant_id,
        customer_name: order.customer_name,
        channel: order.channel,
        status: order.status,
        payment_method: order.payment_method,
        total_cents: order.total_cents,
        created_at: order.created_at,
    }))
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
    let from_str = params.get("from").map(String::as_str);
    let to_str = params.get("to").map(String::as_str);
    if from_str.is_some_and(|date| !is_valid_iso_date(date))
        || to_str.is_some_and(|date| !is_valid_iso_date(date))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let Some(pool) = state.db_pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let report = report_summary(
        pool,
        tenant_id,
        from_str.map(|value| format!("{value} 00:00:00")).as_deref(),
        to_str.map(|value| {
            let end = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|date| date.succ_opt())
                .unwrap_or_else(|| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap());
            format!("{} 00:00:00", end.format("%Y-%m-%d"))
        })
        .as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AdminReportSummaryResponse {
        tenant_id: report.tenant_id,
        sales_cents: report.sales_cents,
        donations_cents: report.donations_cents,
        cogs_cents: report.cogs_cents,
        gross_profit_cents: report.gross_profit_cents,
        sales_by_payment: report.sales_by_payment.into_iter().collect(),
    }))
}
