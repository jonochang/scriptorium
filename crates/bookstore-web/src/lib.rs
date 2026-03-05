use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::routing::{delete, get, post};
use axum::{
    Json, Router,
    response::{Html, Response},
};
use bookstore_app::{
    AdminProduct, AdminRole, AdminService, CatalogService, PosPaymentOutcome, PosService,
    RequestContext, SalesEvent, StorefrontService, WebhookFinalizeStatus,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Default)]
pub struct AppState {
    pub catalog: CatalogService,
    pub pos: PosService,
    pub storefront: StorefrontService,
    pub admin: AdminService,
    pub db_pool: Option<SqlitePool>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/books", get(list_books))
        .route("/context", get(request_context))
        .route("/admin/intake", get(admin_intake_shell))
        .route("/catalog", get(storefront_catalog))
        .route("/catalog/search", get(storefront_search))
        .route("/checkout", get(storefront_checkout))
        .route("/pos", get(pos_shell))
        .route("/api/pos/login", post(pos_login))
        .route("/api/pos/scan", post(pos_scan))
        .route("/api/pos/cart/items", post(pos_quick_item))
        .route("/api/pos/payments/cash", post(pos_pay_cash))
        .route("/api/pos/payments/external-card", post(pos_pay_external_card))
        .route("/api/pos/payments/iou", post(pos_pay_iou))
        .route("/api/storefront/checkout/session", post(storefront_checkout_session))
        .route("/api/payments/webhook", post(payments_webhook))
        .route("/api/admin/products/isbn-lookup", post(admin_isbn_lookup))
        .route("/api/admin/inventory/receive", post(admin_inventory_receive))
        .route("/api/admin/inventory/adjust", post(admin_inventory_adjust))
        .route("/api/admin/inventory/journal", get(admin_inventory_journal))
        .route("/api/admin/auth/login", post(admin_auth_login))
        .route("/api/admin/products", post(admin_product_upsert).get(admin_product_list))
        .route("/api/admin/products/{product_id}", delete(admin_product_delete))
        .route("/api/admin/categories", get(admin_categories_list))
        .route("/api/admin/vendors", get(admin_vendors_list))
        .route("/api/admin/reports/summary", get(admin_report_summary))
        .route("/api/i18n", get(i18n_lookup))
        .layer(middleware::from_fn(request_context_middleware))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Debug, Serialize)]
struct ContextResponse {
    tenant_id: String,
    locale: String,
}

async fn request_context(
    axum::extract::Extension(context): axum::extract::Extension<RequestContext>,
) -> Json<ContextResponse> {
    Json(ContextResponse { tenant_id: context.tenant_id, locale: context.locale })
}

async fn list_books(State(state): State<AppState>) -> Json<Vec<bookstore_domain::Book>> {
    Json(state.catalog.list_books().await)
}

async fn storefront_catalog(State(state): State<AppState>) -> Html<String> {
    let books = state.catalog.list_books().await;
    let items = books
        .into_iter()
        .map(|book| format!("<li>{} - {}</li>", book.title, book.author))
        .collect::<Vec<_>>()
        .join("");
    Html(format!(
        "<!doctype html><html><body><h1>Catalog</h1><form hx-get=\"/catalog/search\" hx-target=\"#results\"><input name=\"q\" /></form><ul id=\"results\">{items}</ul></body></html>"
    ))
}

async fn storefront_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let query = params.get("q").map_or("", String::as_str).to_ascii_lowercase();
    let books = state.catalog.list_books().await;
    let filtered = books
        .into_iter()
        .filter(|book| {
            book.title.to_ascii_lowercase().contains(&query)
                || book.author.to_ascii_lowercase().contains(&query)
        })
        .map(|book| format!("<li>{} - {}</li>", book.title, book.author))
        .collect::<Vec<_>>()
        .join("");
    Html(format!("<ul id=\"results\">{filtered}</ul>"))
}

async fn storefront_checkout() -> Html<&'static str> {
    Html("<!doctype html><html><body><h1>Checkout</h1></body></html>")
}

async fn admin_intake_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Admin Intake</title>
</head>
<body>
  <main>
    <h1>Admin Inventory Intake</h1>
    <video id="camera" autoplay playsinline></video>
    <form id="intake-form">
      <input id="isbn" name="isbn" placeholder="ISBN" />
      <input id="title" name="title" placeholder="Title" />
      <input id="author" name="author" placeholder="Author" />
      <input id="description" name="description" placeholder="Description" />
      <input id="username" name="username" placeholder="Username" value="admin" />
      <input id="password" name="password" type="password" placeholder="Password" value="admin123" />
      <input id="token" name="token" placeholder="Admin Token" />
      <button type="button" id="login">Login</button>
      <button type="button" id="lookup">Lookup</button>
    </form>
  </main>
  <script>
    async function bootCamera() {
      if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) return;
      const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: "environment" } });
      document.getElementById("camera").srcObject = stream;
      if ("BarcodeDetector" in window) {
        const detector = new BarcodeDetector({ formats: ["ean_13", "upc_a"] });
        setInterval(async () => {
          const video = document.getElementById("camera");
          const barcodes = await detector.detect(video);
          if (barcodes[0] && barcodes[0].rawValue) {
            document.getElementById("isbn").value = barcodes[0].rawValue;
          }
        }, 1000);
      }
    }
    async function login() {
      const username = document.getElementById("username").value;
      const password = document.getElementById("password").value;
      const res = await fetch("/api/admin/auth/login", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ username, password }),
      });
      const json = await res.json();
      document.getElementById("token").value = json.token || "";
    }
    async function lookup() {
      const isbn = document.getElementById("isbn").value;
      const token = document.getElementById("token").value;
      const res = await fetch("/api/admin/products/isbn-lookup", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ token, isbn }),
      });
      const json = await res.json();
      document.getElementById("title").value = json.title || "";
      document.getElementById("author").value = json.author || "";
      document.getElementById("description").value = json.description || "";
    }
    document.getElementById("login").addEventListener("click", login);
    document.getElementById("lookup").addEventListener("click", lookup);
    bootCamera();
  </script>
</body>
</html>"#,
    )
}

async fn pos_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Scriptorium POS</title>
  <style>
    :root {
      --wine: #6B2737;
      --wine-dark: #4A1A26;
      --gold: #B8903A;
      --parchment: #FAF7F2;
      --ink: #2C1810;
      --radius: 12px;
    }
    body {
      margin: 0;
      font-family: "DM Sans", sans-serif;
      background: linear-gradient(180deg, var(--wine-dark), var(--wine));
      color: #fff;
      min-height: 100vh;
      padding: 16px;
    }
    .pos-wrap {
      max-width: 420px;
      margin: 0 auto;
      display: grid;
      gap: 12px;
    }
    .card {
      background: var(--parchment);
      color: var(--ink);
      border-radius: var(--radius);
      padding: 14px;
      box-shadow: 0 4px 18px rgba(0,0,0,.12);
    }
    .pos-button--lg {
      width: 100%;
      min-height: 56px;
      border: 0;
      border-radius: var(--radius);
      font-size: 18px;
      font-weight: 700;
      background: var(--wine);
      color: #fff;
      margin: 6px 0;
    }
    .row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    input {
      width: 100%;
      min-height: 44px;
      border-radius: 10px;
      border: 1px solid #ddd;
      padding: 8px 10px;
      box-sizing: border-box;
    }
    #status { font-weight: 700; color: var(--gold); margin-top: 8px; min-height: 24px; }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import { h, render } from "https://esm.sh/preact@10.25.4";
    import htm from "https://esm.sh/htm@3.1.1";
    import { useState } from "https://esm.sh/preact@10.25.4/hooks";

    const html = htm.bind(h);

    function App() {
      const [token, setToken] = useState("");
      const [barcode, setBarcode] = useState("9780060652937");
      const [status, setStatus] = useState("");

      const post = async (url, payload) => {
        const res = await fetch(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const text = await res.text();
        setStatus(text);
      };

      return html`
        <main class="pos-wrap">
          <section class="card">
            <h2>Scriptorium POS</h2>
            <button class="pos-button--lg" onClick=${async () => {
              const res = await fetch("/api/pos/login", {
                method: "POST",
                headers: { "content-type": "application/json" },
                body: JSON.stringify({ pin: "1234" }),
              });
              const json = await res.json();
              setToken(json.session_token || "");
              setStatus(JSON.stringify(json));
            }}>Start Shift</button>
          </section>
          <section class="card">
            <input value=${barcode} onInput=${(e) => setBarcode(e.target.value)} />
            <button class="pos-button--lg" onClick=${() => post("/api/pos/scan", { session_token: token, barcode })}>Scan Item</button>
            <div class="row">
              <button class="pos-button--lg" onClick=${() => post("/api/pos/cart/items", { session_token: token, item_id: "prayer-card-50c", quantity: 1 })}>Quick Item</button>
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/cash", { session_token: token, tendered_cents: 2000, donate_change: true })}>Pay Cash</button>
            </div>
            <div class="row">
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/external-card", { session_token: token, external_ref: "square-ui" })}>Pay Card</button>
              <button class="pos-button--lg" onClick=${() => post("/api/pos/payments/iou", { session_token: token, customer_name: "Walk In" })}>Put on IOU</button>
            </div>
            <p id="status">${status}</p>
          </section>
        </main>
      `;
    }

    render(html`<${App} />`, document.getElementById("app"));
  </script>
</body>
</html>"#,
    )
}

#[derive(Debug, Deserialize)]
struct PosLoginRequest {
    pin: String,
}

#[derive(Debug, Serialize)]
struct PosLoginResponse {
    session_token: String,
}

async fn pos_login(
    State(state): State<AppState>,
    Json(request): Json<PosLoginRequest>,
) -> Result<Json<PosLoginResponse>, axum::http::StatusCode> {
    let session_token = state
        .pos
        .login_with_pin(&request.pin)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    Ok(Json(PosLoginResponse { session_token }))
}

#[derive(Debug, Deserialize)]
struct PosScanRequest {
    session_token: String,
    barcode: String,
}

#[derive(Debug, Deserialize)]
struct PosQuickItemRequest {
    session_token: String,
    item_id: String,
    quantity: i64,
}

#[derive(Debug, Deserialize)]
struct PosCashPaymentRequest {
    session_token: String,
    tendered_cents: i64,
    donate_change: bool,
}

#[derive(Debug, Deserialize)]
struct PosExternalCardRequest {
    session_token: String,
    external_ref: String,
}

#[derive(Debug, Deserialize)]
struct PosIouRequest {
    session_token: String,
    customer_name: String,
}

#[derive(Debug, Serialize)]
struct PosResponse {
    status: &'static str,
    total_cents: i64,
    change_due_cents: i64,
    donation_cents: i64,
}

#[derive(Debug, Deserialize)]
struct StorefrontCheckoutSessionRequest {
    total_cents: i64,
    email: String,
}

#[derive(Debug, Serialize)]
struct StorefrontCheckoutSessionResponse {
    session_id: String,
}

async fn storefront_checkout_session(
    State(state): State<AppState>,
    Json(request): Json<StorefrontCheckoutSessionRequest>,
) -> Result<Json<StorefrontCheckoutSessionResponse>, axum::http::StatusCode> {
    let session = state
        .storefront
        .create_checkout_session(request.total_cents, request.email)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(StorefrontCheckoutSessionResponse { session_id: session.session_id }))
}

#[derive(Debug, Deserialize)]
struct PaymentsWebhookRequest {
    external_ref: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct PaymentsWebhookResponse {
    status: &'static str,
    receipt_sent: bool,
}

#[derive(Debug, Deserialize)]
struct AdminIsbnLookupRequest {
    token: String,
    isbn: String,
}

#[derive(Debug, Serialize)]
struct AdminIsbnLookupResponse {
    isbn: String,
    title: String,
    author: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct AdminInventoryReceiveRequest {
    token: String,
    tenant_id: String,
    isbn: String,
    quantity: i64,
}

#[derive(Debug, Serialize)]
struct AdminInventoryReceiveResponse {
    tenant_id: String,
    isbn: String,
    on_hand: i64,
}

#[derive(Debug, Deserialize)]
struct AdminAuthLoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct AdminAuthLoginResponse {
    token: String,
    tenant_id: String,
    role: &'static str,
}

#[derive(Debug, Deserialize)]
struct AdminInventoryAdjustRequest {
    token: String,
    tenant_id: String,
    isbn: String,
    delta: i64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct AdminStockMovementResponse {
    tenant_id: String,
    isbn: String,
    delta: i64,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct AdminProductUpsertRequest {
    token: String,
    tenant_id: String,
    product_id: String,
    title: String,
    isbn: String,
    category: String,
    vendor: String,
    cost_cents: i64,
    retail_cents: i64,
}

#[derive(Debug, Serialize)]
struct AdminProductResponse {
    tenant_id: String,
    product_id: String,
    title: String,
    isbn: String,
    category: String,
    vendor: String,
    cost_cents: i64,
    retail_cents: i64,
}

#[derive(Debug, Serialize)]
struct AdminDeleteResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct AdminTaxonomyListResponse {
    tenant_id: String,
    values: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AdminReportSummaryResponse {
    tenant_id: String,
    sales_cents: i64,
    donations_cents: i64,
    cogs_cents: i64,
    gross_profit_cents: i64,
    sales_by_payment: Vec<(String, i64)>,
}

async fn payments_webhook(
    State(state): State<AppState>,
    Json(request): Json<PaymentsWebhookRequest>,
) -> Result<Json<PaymentsWebhookResponse>, axum::http::StatusCode> {
    let result = state
        .storefront
        .finalize_webhook(&request.external_ref, &request.session_id)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    if result.status == WebhookFinalizeStatus::Processed {
        state
            .admin
            .record_sales_event(SalesEvent {
                tenant_id: "church-a".to_string(),
                payment_method: "online_card".to_string(),
                sales_cents: 0,
                donations_cents: 0,
                cogs_cents: 0,
                occurred_on: current_utc_date(),
            })
            .await;
    }
    Ok(Json(PaymentsWebhookResponse {
        status: match result.status {
            WebhookFinalizeStatus::Processed => "processed",
            WebhookFinalizeStatus::Duplicate => "duplicate",
        },
        receipt_sent: result.receipt_sent,
    }))
}

async fn admin_isbn_lookup(
    State(state): State<AppState>,
    Json(request): Json<AdminIsbnLookupRequest>,
) -> Result<Json<AdminIsbnLookupResponse>, axum::http::StatusCode> {
    state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    let metadata = state
        .admin
        .lookup_isbn(&request.isbn)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminIsbnLookupResponse {
        isbn: metadata.isbn,
        title: metadata.title,
        author: metadata.author,
        description: metadata.description,
    }))
}

async fn admin_inventory_receive(
    State(state): State<AppState>,
    Json(request): Json<AdminInventoryReceiveRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, axum::http::StatusCode> {
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .receive_inventory(&request.tenant_id, &request.isbn, request.quantity)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

async fn admin_inventory_adjust(
    State(state): State<AppState>,
    Json(request): Json<AdminInventoryAdjustRequest>,
) -> Result<Json<AdminInventoryReceiveResponse>, axum::http::StatusCode> {
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let receipt = state
        .admin
        .adjust_inventory(&request.tenant_id, &request.isbn, request.delta, &request.reason)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminInventoryReceiveResponse {
        tenant_id: receipt.tenant_id,
        isbn: receipt.isbn,
        on_hand: receipt.on_hand,
    }))
}

async fn admin_inventory_journal(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminStockMovementResponse>>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
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

async fn admin_auth_login(
    State(state): State<AppState>,
    Json(request): Json<AdminAuthLoginRequest>,
) -> Result<Json<AdminAuthLoginResponse>, axum::http::StatusCode> {
    let session = state
        .admin
        .login(&request.username, &request.password)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    Ok(Json(AdminAuthLoginResponse {
        token: session.token,
        tenant_id: session.tenant_id,
        role: match session.role {
            AdminRole::Admin => "admin",
            AdminRole::Volunteer => "volunteer",
        },
    }))
}

async fn admin_product_upsert(
    State(state): State<AppState>,
    Json(request): Json<AdminProductUpsertRequest>,
) -> Result<Json<AdminProductResponse>, axum::http::StatusCode> {
    let session = state
        .admin
        .require_admin(&request.token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != request.tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
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
    };
    state
        .admin
        .upsert_product(product.clone())
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(AdminProductResponse {
        tenant_id: product.tenant_id,
        product_id: product.product_id,
        title: product.title,
        isbn: product.isbn,
        category: product.category,
        vendor: product.vendor,
        cost_cents: product.cost_cents,
        retail_cents: product.retail_cents,
    }))
}

async fn admin_product_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<AdminProductResponse>>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let products = state
        .admin
        .list_products(tenant_id)
        .await
        .into_iter()
        .map(|product| AdminProductResponse {
            tenant_id: product.tenant_id,
            product_id: product.product_id,
            title: product.title,
            isbn: product.isbn,
            category: product.category,
            vendor: product.vendor,
            cost_cents: product.cost_cents,
            retail_cents: product.retail_cents,
        })
        .collect();
    Ok(Json(products))
}

async fn admin_product_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(product_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminDeleteResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    state
        .admin
        .delete_product(tenant_id, &product_id)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(AdminDeleteResponse { status: "deleted" }))
}

async fn admin_categories_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_categories(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

async fn admin_vendors_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminTaxonomyListResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let values = state.admin.list_vendors(tenant_id).await;
    Ok(Json(AdminTaxonomyListResponse { tenant_id: tenant_id.to_string(), values }))
}

async fn admin_report_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<AdminReportSummaryResponse>, axum::http::StatusCode> {
    let token = bearer_token(&headers)?;
    let tenant_id = params.get("tenant_id").map_or("default", String::as_str);
    let session = state
        .admin
        .require_admin(&token)
        .await
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    if session.tenant_id != tenant_id {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    let from = params.get("from").map(String::as_str);
    let to = params.get("to").map(String::as_str);
    if from.is_some_and(|date| !is_valid_iso_date(date))
        || to.is_some_and(|date| !is_valid_iso_date(date))
    {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }
    let report = state.admin.report_summary_range(tenant_id, from, to).await;
    Ok(Json(AdminReportSummaryResponse {
        tenant_id: report.tenant_id,
        sales_cents: report.sales_cents,
        donations_cents: report.donations_cents,
        cogs_cents: report.cogs_cents,
        gross_profit_cents: report.gross_profit_cents,
        sales_by_payment: report.sales_by_payment,
    }))
}

async fn i18n_lookup(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<std::collections::HashMap<String, String>> {
    let locale = params.get("locale").map_or("en-AU", String::as_str);
    let key = params.get("key").map_or("checkout.complete", String::as_str);
    let value = match (locale, key) {
        ("en-AU", "checkout.complete") => "Sale Complete",
        ("el-GR", "checkout.complete") => "Η πώληση ολοκληρώθηκε",
        ("en-AU", "admin.intake.title") => "Admin Inventory Intake",
        ("el-GR", "admin.intake.title") => "Παραλαβή αποθέματος διαχειριστή",
        ("en-AU", "storefront.checkout.title") => "Checkout",
        ("el-GR", "storefront.checkout.title") => "Ταμείο",
        (_, "checkout.complete") => "Sale Complete",
        _ => key,
    };
    let mut payload = std::collections::HashMap::new();
    payload.insert("locale".to_string(), locale.to_string());
    payload.insert("key".to_string(), key.to_string());
    payload.insert("value".to_string(), value.to_string());
    Json(payload)
}

async fn pos_scan(
    State(state): State<AppState>,
    Json(request): Json<PosScanRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let total = state
        .pos
        .scan_item(&request.session_token, &request.barcode)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: "cart_updated",
        total_cents: total,
        change_due_cents: 0,
        donation_cents: 0,
    }))
}

async fn pos_quick_item(
    State(state): State<AppState>,
    Json(request): Json<PosQuickItemRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let total = state
        .pos
        .add_quick_item(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: "cart_updated",
        total_cents: total,
        change_due_cents: 0,
        donation_cents: 0,
    }))
}

async fn pos_pay_cash(
    State(state): State<AppState>,
    Json(request): Json<PosCashPaymentRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_cash(&request.session_token, request.tendered_cents, request.donate_change)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: "cash".to_string(),
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_on: current_utc_date(),
        })
        .await;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
}

async fn pos_pay_external_card(
    State(state): State<AppState>,
    Json(request): Json<PosExternalCardRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_external_card(&request.session_token, &request.external_ref)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: "external_card".to_string(),
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_on: current_utc_date(),
        })
        .await;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
}

async fn pos_pay_iou(
    State(state): State<AppState>,
    Json(request): Json<PosIouRequest>,
) -> Result<Json<PosResponse>, axum::http::StatusCode> {
    let receipt = state
        .pos
        .checkout_iou(&request.session_token, &request.customer_name)
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::UnpaidIou {
            "iou"
        } else {
            "sale_complete"
        },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
    }))
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

fn bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
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

fn current_utc_date() -> String {
    chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

fn is_valid_iso_date(input: &str) -> bool {
    chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").is_ok()
}
